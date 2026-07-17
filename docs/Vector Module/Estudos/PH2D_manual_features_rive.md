# Manual de Implementação — Features Runtime-First (Rive) para o PH2D

> **Documento técnico / manual para agente de implementação. Volume 3 da série** (Vol. 1: desenho vetorial expressivo; Vol. 2: features de sistema do Figma).
> Escopo: o Rive é o **parente arquitetural mais próximo do PH2D** e, ao contrário do Figma, é **genuinamente open source** — os runtimes (`rive-runtime` em C++, `rive-rs` em Rust) e o formato `.riv` são MIT e públicos. Isso muda a natureza deste volume: em vez de inferir arquitetura por engenharia reversa, podemos ler a **fonte de verdade**. Como o PH2D é Rust, o `rive-rs` é referência direta (e, nota importante, ele renderiza via **Vello** — o mesmo que o teu ADR de UI pina).
> Escopo das features: state machines, skeletal rigging + mesh vetorial, constraints, data binding, nested artboards + events, o modelo runtime-first (formato `.riv`), e as adições recentes (layouts responsivos, feathering).
>
> **Como usar (para o agente):** este volume é o mais próximo de "copiar a arquitetura certa". A recomendação transversal é **estudar `rive-runtime` diretamente** para casos de borda; aqui damos o mapa, os algoritmos e as decisões de porte para o stack wgpu. Reuse pesadamente os Vols. 1 e 2 (o node graph não-destrutivo, a indireção de variables, os IDs estáveis).

---

## 0. A arquitetura Rive em uma página (leia antes de tudo)

O `rive-runtime` inteiro se apoia em **quatro decisões** que o PH2D deveria adotar quase literalmente:

1. **Modelo de objetos "Core".** Tudo (Node, Shape, Bone, KeyFrame, StateMachine, ViewModel…) é um **Core object** com um **type key** único e um conjunto de **property keys** tipadas. A serialização é gerada a partir de definições JSON ("core defs"). Isso dá reflexão, animação de qualquer propriedade, e serialização uniformes de graça.
2. **Um único `advance(dt)`.** State machines, animações, constraints e data binding **todos mutam a mesma hierarquia de objetos**, e uma única chamada `Artboard::advance(dt)` resolve as mudanças de forma eficiente (ordem de dependência). Não há um "sistema de animação" e outro "de layout" e outro "de constraints" — há uma hierarquia e um solve.
3. **Renderer abstrato com paths retidos.** A interface `Renderer` recebe comandos de path de alto nível com **objetos de path retidos** para minimizar recomputação; o backend concreto (Metal/Vulkan/D3D/GL, ou Vello no `rive-rs`, ou o teu wgpu) decide como rasterizar.
4. **Formato resiliente a versão (ToC).** O `.riv` carrega uma **tabela de conteúdo** que permite a um runtime **pular propriedades/objetos que não entende** — arquivos novos carregam em runtimes velhos, com features desconhecidas virando no-ops, sem crash.

> ⚠️ **Recomendação opinativa nº 1 (a mais importante deste volume):** **adote o padrão de formato Core + ToC do Rive como o formato de save do PH2D.** Um game engine *shippa binários*; usuários terão arquivos criados em versões diferentes do editor. O design "type key + backing type na ToC + objetos em ordem de dependência + property key 0 como terminador" é uma aula de formato runtime-first com forward/backward-compat. Custa pouco no dia 1 e evita a dor crônica de migração de formato. (Detalhes na §6.)

**Sobre o `rive-rs` + Vello:** o `rive-rs` já prova o caminho Rust→Vello, mas o próprio README lista limitações do backend Vello em **image meshes** (inconsistências nas bordas de triângulo, gaps, overdraw em meshes transparentes) e em número muito alto de clips, além de forçar joins/caps redondos. **Tradução para o PH2D:** use Vello para o chrome (como já decidido), mas a malha vetorial deformável (§2) e o canvas criativo pedem o **pipeline wgpu bespoke** — é justamente onde o `rive-rs` sofre, e onde o teu diferencial de pintura se paga.

---

## 1. State Machines (a camada de comportamento)

**O quê:** grafos visuais de comportamento com **inputs** (bool, number, trigger) que dirigem animação **em runtime**. É a ponte que elimina o round-trip arte→engine: o artista autora o comportamento; o código só alimenta inputs. É "o diferenciador que eleva o Rive acima de formatos de animação tradicionais" — uma camada de *lógica* sobre as timelines.

**Referência open-source:** `rive-runtime` (`StateMachine`, `StateMachineInstance`) e docs. Anatomia:
- **Inputs** (o contrato designer↔dev): `SMIBool` (`.value`), `SMINumber` (`.value` float), `SMITrigger` (`.fire()`, verdadeiro por 1 frame).
- **States**: cada state é uma timeline (ou um **blend state**, ou placeholders `Entry`/`Exit`/`AnyState`). `AnyState` habilita *interrupts* (transicionar de qualquer lugar — ex.: "morreu" a qualquer momento).
- **Transitions**: têm **conditions** (bool true/false, number `>`/`<`/`==`, trigger fired), **duration**, **exit time** (fração do state que deve tocar antes de sair), e **interpolation** (linear/cubic/hold). Múltiplas transitions entre dois states = "or"; múltiplas conditions numa transition = "and".
- **Layers**: várias camadas de state machine rodam **simultaneamente** (análogo a threads) — ex.: locomoção numa layer, expressão facial noutra.
- **Blend states**: **1D** (um number mistura N animações — idle→walk→run) e **direct/aditivo** (multi-dimensional — "face space" X/Y seguindo o cursor).

```rust
// Contrato de runtime (espelhando a API Rive)
let sm = artboard.state_machine("Locomotion");
sm.get_number("speed").set(0.0..=100.0);   // 1D blend idle/walk/run
sm.get_bool("grounded").set(true);
sm.get_trigger("jump").fire();
// dirige tudo com um único passo:
sm.advance(dt);   // avalia conditions, dispara transitions, interpola, aplica à hierarquia
```

**Algoritmo principal — hierarchical state machine com camadas + avaliação por `advance(dt)`.**
Cada layer mantém um **state atual** e avalia, a cada frame: (1) as transitions saindo do state atual (e de `AnyState`) cujas conditions são satisfeitas e cujo exit time foi atingido; (2) se uma dispara, entra em **transição** — que é ela mesma uma interpolação das propriedades do state de origem para as do destino, ao longo de `duration` com a easing configurada; (3) aplica o resultado (mix de timelines) à hierarquia de objetos Core; (4) triggers consumidos (voltam a false). Blend states avaliam os pesos das sub-animações a partir do input e as somam (1D: interpolação por vizinhança; aditivo: soma ponderada sobre um base).

**Algoritmo alternativo — behavior tree (para lógica de jogo mais rica).**
State machines são ótimas para *animação reativa*, mas escalam mal quando a lógica vira "sequências com fallback e prioridade" (IA de inimigo, quests). Uma **behavior tree** (nós Sequence/Selector/Parallel/Decorator sobre folhas de ação) modela isso melhor, com composição e reuso superiores. Trade-off: menos intuitiva para o artista (é uma ferramenta de *dev*), e overkill para transições de UI/personagem. A escolha certa: **state machine para o artista autorar comportamento visual**; behavior tree opcional na camada de gameplay que *alimenta os inputs* da state machine.

**Notas de integração PH2D:** a state machine **é o teu node graph aplicado ao tempo** — não construa um sistema separado. Modele states/transitions como nodes; o `advance(dt)` é o recompute do grafo com o tempo como entrada. Exponha os inputs como a API pública do asset (é literalmente o contrato com o código do jogo). Casos de jogo que o Rive já valida: mascote de HUD cujo rosto vai de "confiante" a "derrotado" via 1D blend na vida; personagem cujo `look` segue o cursor via blend 2D. Conecte com o **Smart Animate do Vol. 2 §5**: uma transition *é* um Smart Animate entre dois states — reuse o mesmo interpolador (transform decomposto, cor em OKLab).

---

## 2. Skeletal Rigging + Mesh Vetorial (personagem pintado → animável)

**O quê:** ossos, IK e **deformação de malha** aplicados sobre arte **vetorial** (e imagens raster mapeadas na malha). O personagem pintado vira animável sem sair da ferramenta — a espinha da tese "arte→engine sem round-trip".

**Referência open-source:** `rive-runtime` (`Bone`, `Skin`, `Tendon`, `Weight`, `Mesh`) e docs do editor. Pontos-chave do modelo Rive:
- **Bones** hierárquicos: root bone tem posição; child bones herdam do pai (rotacionar o pai reposiciona os filhos). Rotacionar um osso muda rotação *e* comprimento.
- **Binding + weighting**: liga-se ossos a um path/mesh e atribui a cada **vértice** um peso por osso; **os pesos somam sempre 100%**. O detalhe único do Rive: pesa-se também os **handles Bézier** do vértice, e pesá-los *diferente* do vértice cria efeitos "3D" (o handle se move em velocidade diferente).
- **Mesh**: gera-se uma malha (auto ou custom via pen tool, com forced edges), e a arte (vetor ou imagem) é deformada por ossos ou por movimento direto de vértices. Faz "pele flexionar, tecido ondular, cabelo fluir".
- Conexão **hierárquica** (sem weighting) para arte rígida presa a um osso (escudo, acessório) é mais barata que binding+weighting.

**Algoritmo principal — Linear Blend Skinning (LBS) sobre vértices vetoriais.**
Para cada vértice `v` (na pose de bind, coordenadas do modelo), a posição deformada é a combinação ponderada das transformações dos ossos:

```
v' = ( Σ_b  w_{b} · T_b · T_bind_b⁻¹ ) · v      com  Σ_b w_{b} = 1
```

onde `T_b` é a transform atual do osso `b` e `T_bind_b⁻¹` desfaz a pose de bind. Tipicamente ≤4 ossos por vértice têm peso não-nulo, então empacota-se `(bone_ids: vec4, weights: vec4)` por vértice e faz-se o skinning no **vertex shader** (wgpu) — uma matriz de paleta de ossos como uniform/storage buffer. Para arte vetorial, aplica-se o mesmo aos **pontos de controle Bézier e handles** (com pesos possivelmente distintos, como o Rive faz), e a tesselação (lyon/kurbo) roda *depois* da deformação, ou deforma-se a malha já tesselada.

**Algoritmo alternativo — Dual Quaternion Skinning (DQS) ou Velocity Skinning.**
LBS sofre o "candy-wrapper" (colapso de volume) em torções fortes de junta. **DQS** interpola transformações como quaternions duais, preservando volume nas juntas — melhor para membros que torcem muito, ao custo de mais matemática por vértice. **Velocity Skinning** (Rohmer et al. 2021) adiciona *automaticamente* follow-through/squash a partir da velocidade dos ossos (pesos de "floppiness"/"squashiness" pintáveis) — dá vida secundária de graça, rodando em uma passada de malha. Para um app que mira expressividade artística, Velocity Skinning é um diferencial de deleite; comece com LBS e ofereça DQS/VS como upgrade.

**Notas de integração PH2D:** este é o ponto onde o rigging encontra a **deformação de malha do Vol. 1 §5.1 (warp/MLS)** — unifique-os: warp por pinos (MLS/ARAP) e skinning por ossos (LBS) são dois "deformers" sobre a mesma malha; modele um **stack de deformers** por objeto. Faça o skinning no vertex shader (essencial para animar em runtime a 60fps no iPad). Pese vértices *e* handles Bézier (herde o truque do Rive — dá o efeito pseudo-3D barato). Como a arte pode ser gradient-mesh (Vol. 1 §2.1), a deformação move os pontos de controle da mesh de cor junto — vetor pintado *e* riggado, o que nem Rive nem Live2D entregam com pintura profissional.

---

## 3. Constraints (restrições procedurais reutilizáveis)

**O quê:** restrições procedurais reaplicáveis entre rigs — **Translation, Scale, Rotation, Transform, Distance, IK, Follow Path**. Reduzem trabalho manual e criam relações vivas entre objetos (um objeto "segue" outro, um osso "aponta" para um alvo).

**Referência open-source:** `rive-runtime` (família `Constraint`: `IKConstraint`, `DistanceConstraint`, `TransformConstraint`, `FollowPathConstraint`, etc.). Propriedades comuns: **strength** (0–100%, blendável e *animável* — permite misturar constraints ou ligar/desligar por frame) e **ordem** (constraints resolvem em ordem; a de baixo pode cancelar a de cima se ambas a 100%).

**Algoritmo principal (a família geral) — solve em ordem de dependência dentro do `advance`.**
Cada constraint é uma função `constrain(&mut target, sources, strength)` aplicada durante o passe de solve, *depois* que os sources foram atualizados e *antes* que os dependentes do target sejam. É o mesmo **solve topológico** do node graph (Vol. 1 §7) e do próprio `advance` do Rive. Os casos:
- **Translation/Scale/Rotation/Transform**: copiam (total ou parcialmente, por strength) a componente correspondente da transform de um source para o target — a base de "siga/espelhe/acompanhe".
- **Distance**: mantém o target a uma distância (mín/máx/exata) de um source.
- **Follow Path**: posiciona o target ao longo de um path por `distance percent` (reusa a reparametrização por arclength do Vol. 1 §1.2).

**Algoritmo principal (IK) — two-bone analítico + FABRIK para cadeias.**
IK inverte a cinemática: em vez de rotacionar cada osso (FK), põe-se um **target** no fim da cadeia e resolve-se as rotações dos pais.
- **Cadeia de 2 ossos** (braço/perna): solução **analítica** por lei dos cossenos — dado o alcance ao target, calcula-se o ângulo do cotovelo/joelho e a orientação do ombro/quadril diretamente. Rápido e estável; `invert direction` escolhe o cotovelo pra cima/baixo.
- **Cadeias longas**: **FABRIK** (Forward And Backward Reaching Inverse Kinematics) — iterativo, sem trigonometria pesada, converge rápido e comporta-se bem. O `bone count` do Rive define quantos ossos a montante o IK afeta; `strength` mistura entre o resultado IK e a pose FK.

```
// two-bone IK (esboço): raiz A, junta B, efetor C, alvo T
let l1 = |B-A|; let l2 = |C-B|; let d = clamp(|T-A|, |l1-l2|, l1+l2);
let a = acos((l1*l1 + d*d - l2*l2) / (2*l1*d));   // ângulo no ombro (lei dos cossenos)
// oriente A→T, gire por ±a (invert_direction) para achar B; oriente B→T
```

**Algoritmo alternativo — IK por Jacobiano (CCD / Jacobian transpose/DLS).**
Para rigs com muitos DOF, restrições de ângulo, ou múltiplos alvos, métodos baseados em Jacobiano (ou **CCD**, Cyclic Coordinate Descent, mais simples) generalizam além do que o analítico/FABRIK cobre. Mais caros e com tuning de convergência, mas necessários para rigs complexos (cauda com limites, tentáculo com rigidez variável). Comece com two-bone+FABRIK; adicione CCD se surgir demanda.

**Notas de integração PH2D:** constraints são **nodes de um tipo especial** (target ← f(sources, strength)) que participam do mesmo solve topológico do node graph — não são um subsistema à parte. Torne `strength` uma propriedade animável/bindável (herde do Rive) para que constraints possam ser dirigidas por state machine (§1) e data binding (§4) — ex.: um inimigo cuja mira (IK) engaja quando `sees_player` fica true. Follow Path reusa a infra de arclength do Vol. 1. Ordem de resolução importa: exponha reordenação (drag-and-drop) como o Rive.

---

## 4. Data Binding (propriedades ligadas a estado externo) — a ponte central

**O quê:** propriedades da arte **ligadas a dados/estado externo** — barra de vida, contadores, temas dinâmicos, avatares. É o padrão **MVVM** (Model-View-ViewModel) construído no formato: o artista mapeia dados estruturados na UI sem o dev fiar cada conexão à mão. **É a feature mais relevante deste volume para a tua tese** — a generalização, em runtime, das variables/modes do Vol. 2 §4.

**Referência open-source:** `rive-runtime` — arquitetura `ViewModel` / `ViewModelInstance` / `ViewModelProperty` / `DataBind` / `BindableProperty` (o DeepWiki do repo documenta o fluxo). Modelo:
- **ViewModel**: o *schema* — um conjunto de propriedades tipadas: **bool, number, string, color, enum, trigger, image, artboard, nested view model, list**.
- **ViewModelInstance**: os *valores vivos* daquele schema (várias instâncias por schema — um `ProductCard` schema, N instâncias).
- **Bind**: liga uma propriedade da instância a uma propriedade de um componente da arte (`artboard.bind_view_model_instance(vmi)` / `state_machine.bind(vmi)`).
- **Fluxo**: o app muta a instância → chama `advance()`/`advance_and_apply()` → as mudanças propagam pelas bindings → afetam a arte/state machine. Acesso aninhado por **path** estilo URI (`"player/ui/health"`).
- **Converters**: math, number→list, formatters (número→texto), e **time-based** (interpolação animada do valor). Uma propriedade pode bindar a *outra* (comprimento de um path → tamanho de fonte).
- **Bidirecional**: **listeners** de state machine escrevem de volta nas propriedades do ViewModel — a arte manda dados *de volta*.

```rust
// MVVM em runtime (espelhando rive-runtime)
let vm  = file.view_model("PlayerHUD");
let vmi = vm.create_instance();            // valores vivos
artboard.bind_view_model_instance(&vmi);
vmi.number("health").set(0.75);            // 0..1 → dirige a barra + cor + animação
vmi.string("name").set("Enio");
vmi.color("theme").set(oklab(...));        // tema dinâmico
artboard.advance_and_apply(dt);            // propaga binding → arte
```

**Algoritmo principal — MVVM com propagação por observação + dirty flags, resolvida no `advance`.**
Cada `BindableProperty` observa uma propriedade do ViewModelInstance; ao mudar o valor, marca-se *dirty* o alvo; no `advance`, os dirties propagam em ordem de dependência (mesma disciplina do node graph / constraints). Converters são nós no caminho da propagação (transformam o valor entre source e target). Acesso por path resolve descendo a árvore de ViewModels aninhados.

**Algoritmo alternativo — reactive signals / dataflow (push-pull).**
Em vez de dirty flags + solve por frame, um grafo de **signals** (estilo SolidJS/Leptos, ou FRP) recomputa automaticamente só o que depende do valor mudado, na hora da escrita (push) ou da leitura (pull). Mais elegante e granular; casa bem com Rust (crates como `leptos_reactive`/`futures-signals`). Trade-off: overhead por-signal e complexidade de agendamento; para milhares de propriedades a 60fps, o modelo dirty+advance do Rive costuma ser mais previsível. Escolha por perfil.

**Notas de integração PH2D:** data binding é **a fusão dos três volumes**. É a versão *runtime e dirigida por estado* das variables/modes do Vol. 2 §4 (lá, o valor resolvia por *mode* estático; aqui, por *estado do jogo* ao vivo). Unifique: um "token" e uma "propriedade de ViewModel" são a mesma coisa vista em dois tempos — implemente **um** sistema de propriedades tipadas que resolve tanto por mode quanto por binding runtime. Casos de jogo diretos: barra de vida, contador de score, minimapa, tema por facção, avatar por `image` property, inventário por `list` property (com **virtualização** — ver §5). Cor sempre em OKLab. Bidirecional (arte→código via listeners) fecha o loop com os **events** (§5). Esta é a peça que faz "o jogo só alimenta inputs/dados" virar realidade.

---

## 5. Nested Artboards + Events (composição + comunicação bidirecional)

**O quê:** **composição** (artboard dentro de artboard — hoje chamados **Components** no Rive) e **events** disparados da timeline de volta ao código host. Junto: composabilidade + comunicação bidirecional (o host dirige inputs/dados; a arte emite sinais de volta).

**Referência open-source:** `rive-runtime` — `NestedArtboard` (estende `AdvancingComponent`, `ResettingComponent`, `ArtboardHost`, `ArtboardReferencer`), `ArtboardComponentList` (listas virtualizadas), e o sistema de `Event` (`GeneralRiveEvent`, `OpenURLRiveEvent`).

**Composição — nested artboards / Components:**
- Um NestedArtboard **referencia** um artboard-fonte; ao instanciar o pai, `clone()` produz uma `ArtboardInstance` fresca; o host (`ArtboardHost`) propaga tamanho/transform pra cima e participa do mesmo `advance` (via o vetor `m_ArtboardHosts` — o ponto único de propagação de data-bind).
- **Components** podem conter suas próprias **state machines** (lógica self-contained — botão, toggle) e funcionam com **data binding** através de arquivos.
- **Export granular:** só artboards marcados como Component (mais o principal) são exportados → runtime enxuto, sem bloat de artboards de rascunho.
- **Component lists** (§4 `list`): variam em nº de itens dirigidos por data binding, com **virtualização** — só o window visível tem instâncias vivas (via `ScrollConstraint`), com pools de recursos. Escala para milhares de itens (inventário, leaderboard) sem custo.

**Events — comunicação de volta ao host:**
- Sinais nomeados disparados de timelines/state machines/listeners no **design time**, subscritos em **runtime** com **propriedades customizadas tipadas** (number, string, bool) e delay opcional.
- Tipos: **General** (payload arbitrário) e **OpenURL** (o runtime pode abrir a URL automaticamente).

```rust
// Composição + eventos
let hud = file.artboard("HUD");                 // pai
// … NestedArtboard "AmmoCounter" referencia outro artboard, com sua própria state machine …
artboard.on_event(|ev| match ev {               // arte → host
    Event::General { name, props } if name == "LevelComplete" => {
        let score = props.number("score");      // metadata tipada
        game.on_level_complete(score);
    }
    Event::OpenUrl { url } => open(url),
    _ => {}
});
```

**Algoritmo principal — árvore de instâncias com propagação host + fila de eventos drenada por frame.**
Composição = uma **árvore de ArtboardInstances**; cada nested participa do `advance` do pai e propaga transform/tamanho/data-bind pelo canal de host. Eventos = uma **fila** preenchida durante o `advance` (quando uma timeline/listener dispara) e **drenada** ao fim do frame, entregue via callback ao host — desacopla o disparo (dentro do solve) do tratamento (no código do jogo), evitando reentrância.

**Algoritmo alternativo — event bus tipado / ECS observers (para gameplay em larga escala).**
Para um game engine, os eventos da arte podem ser publicados num **bus tipado** (ou em observers de um ECS, estilo Bevy) em vez de callbacks diretos — melhor desacoplamento, múltiplos assinantes, e integração com o loop de sistemas do jogo. Trade-off: mais infra; para começar, callbacks diretos (modelo Rive) bastam. A composição, análoga, pode virar **prefabs aninhados** do ECS.

**Notas de integração PH2D:** nested artboards = os **Components do Vol. 2 §3** vistos como *runtime* (o Rive inclusive convergiu a nomenclatura para "Components"). Unifique: prefab do editor e nested artboard do runtime são a mesma entidade. A **virtualização de listas** é requisito para inventários/leaderboards de jogo — implemente o pool + window desde o design da lista. Events são o **retorno** do data binding (§4): juntos dão o loop bidirecional completo (host→arte via inputs/dados; arte→host via events/listeners), que é exatamente a "comunicação bidirecional" da tese. Modele events sobre um bus tipado se já tiveres ECS; senão, callbacks diretos.

---

## 6. Runtime-First (o formato `.riv` e o modelo de execução)

**O quê:** o arquivo é minúsculo (fração de vídeo/GIF/Lottie JSON; ícones típicos 5–30 KB) e **toca dentro do app/engine** — não é export de sprite. **Vetor permanece vetor em execução**, rasterizado na GPU. Este não é uma feature isolada; é a *filosofia* que torna todas as outras possíveis num engine.

**Referência open-source (fonte de verdade):** `rive-runtime` + a doc de formato. Design do `.riv`:
- **Header:** fingerprint ASCII `"RIVE"`, versão (major.minor), e uma **ToC (table of contents)** listando as property IDs presentes e seus **backing types**.
- **Corpo:** lista de objetos; cada objeto é um **varuint type key** seguido de suas propriedades (`property_key` varuint + valor), terminadas por **property key 0**. Little-endian.
- **Core defs:** objetos e propriedades definidos em JSON que geram (de)serialização e código de animação. Type keys são **globais e estáveis** (Shape = 3; Node.X = 13, sempre um float).
- **Contexto implícito por ordem:** objetos vêm em ordem de dependência; o pai é o "último lido" do tipo apropriado (um Shape pertence ao último Artboard lido; um KeyFrame, à última LinearAnimation). Elimina IDs de parent redundantes.
- **Resiliência de versão:** com a ToC, um runtime **pula** propriedades/objetos que não conhece (sabe o backing type, logo sabe quantos bytes pular). Arquivos novos carregam em runtimes velhos; features desconhecidas viram **no-ops**, sem crash.

**Modelo de execução:** carrega objetos Core → monta a hierarquia do Artboard → state machines/animações/constraints/data-bind **mutam a hierarquia** → **um** `Artboard::advance(dt)` resolve as mudanças em ordem de dependência → o **Renderer abstrato** (paths retidos) desenha via o backend.

**Algoritmo principal — formato Core: type keys + ToC + ordem de dependência.**
Serialize cada objeto como `varuint(type_key)` + sequência de `varuint(property_key), value` + `0`. Gere o código de (de)serialização a partir de definições declarativas (as "core defs"). Na leitura, mantenha o "último objeto de cada tipo" para dar contexto de parent; use a ToC para pular o desconhecido.

**Algoritmo alternativo — schema binário genérico (FlatBuffers / Cap'n Proto).**
Em vez de um formato Core artesanal, um schema binário pronto (FlatBuffers: zero-copy, evolução de schema via campos opcionais) dá (de)serialização gerada e forward-compat "de graça". Trade-off: menos compacto e menos "sob medida" que o formato Core do Rive (que explora contexto implícito e property keys globais), e a evolução por-campo é menos elegante que a ToC para pular blocos arbitrários. Para prototipar, FlatBuffers acelera; para o formato *definitivo* de um engine vetorial, o modelo Core paga-se em tamanho e controle.

**Notas de integração PH2D:** **este é o coração do teu ADR de formato de save.** Um engine precisa que arquivos criados hoje abram amanhã — o par **type key + ToC** entrega isso. Reuse o padrão: todo node do teu graph (Vol. 1 §7), toda variable (Vol. 2 §4), toda state machine (§1) é um objeto Core com type key estável e propriedades tipadas — o *mesmo* mecanismo serializa arte, comportamento e dados. O modelo "mutar hierarquia → um `advance(dt)` → renderer abstrato" deve ser a espinha do teu loop: já tens o renderer abstrato (wgpu/Vello), já tens o graph (o "advance"); falta cristalizar o formato Core. Mantém vetor como vetor até a GPU (tesselação lyon/kurbo no fim), preservando a promessa runtime-first.

---

## 7. Adições recentes: Layouts Responsivos + Feathering

**O quê:** duas adições recentes que aproximam o Rive do **pictórico** e do **responsivo**:
- **Layouts responsivos** (+ scrolling, N-slicing): artboards que se recompõem por tamanho — via um motor de layout (o Rive usa flexbox; o `awesome-rive` inclusive vendoriza **Yoga**). É o **Auto Layout do Vol. 2 §2 aplicado ao runtime de animação**.
- **Feathering (bordas suaves no vetor):** suavização de borda por-shape, dando transições pictóricas macias em vez do corte duro do vetor — parte do novo **Rive Renderer**.

**Algoritmo principal (layout) — flexbox no `advance`.**
Reuse **Taffy** (Vol. 2 §2). O layout roda como parte do solve: computa retângulos, que viram transforms dos objetos, propagados na mesma passada de dependência. N-slicing (nine-slice) escala bordas/cantos sem distorcer — geometria paramétrica clássica de UI.

**Algoritmo principal (feathering) — falloff por SDF na borda.**
Feathering = alpha que decai suavemente na borda do shape. Duas rotas GPU: (a) computar a **distância assinada à borda** no fragment (via SDF do shape, reusando o Vol. 1 §3.3) e mapear `alpha = smoothstep(0, feather_width, sdf)`; (b) em cobertura por AA analítico, alargar a rampa de cobertura pela largura de feather. Cai perfeitamente no teu pipeline wgpu com shader nodes — é, de novo, onde o bespoke supera Vello/Skia.

**Algoritmo alternativo (feathering) — blur pós-processo mascarado.**
Rasterize o shape, aplique um **blur gaussiano** separável só na região da borda (mascarada), componha por cima. Mais simples e independente da geometria; custo de uma passada extra e menos preciso que o SDF. Bom para efeito global; o SDF é melhor para feather por-shape controlável.

**Notas de integração PH2D:** feathering por-shape é um **quick win pictórico** de alto valor para o teu público (aproxima vetor de pintura, o teu diferencial) e é barato no wgpu via SDF — priorize. Layout responsivo reusa o Taffy do Vol. 2; unifique o motor de layout entre editor de UI e runtime de arte. Ambos entram no mesmo `advance`.

---

## 8. Tabela-resumo: algoritmo principal × alternativo

| Feature | Papel no PH2D | Algoritmo principal | Alternativo | Fonte de verdade |
|---|---|---|---|---|
| State machines | Camada de comportamento (= node graph no tempo) | HSM com layers + `advance(dt)` | Behavior tree (gameplay) | `rive-runtime` StateMachine |
| Rigging + mesh | Personagem pintado animável | LBS no vertex shader | DQS / Velocity Skinning | `rive-runtime` Skin/Bone |
| Constraints | Relações procedurais | Solve topológico; IK two-bone + FABRIK | IK por Jacobiano/CCD | `rive-runtime` Constraint |
| Data binding | Estado runtime → arte (fusão dos 3 vols) | MVVM + dirty propagation no `advance` | Reactive signals / FRP | `rive-runtime` ViewModel |
| Nested artboards | Composição / prefabs runtime | Árvore de instâncias + host + lista virtualizada | Prefabs ECS | `rive-runtime` NestedArtboard |
| Events | Arte → host (loop bidirecional) | Fila drenada por frame → callback | Event bus tipado / ECS observers | `rive-runtime` Event |
| Runtime-first (formato) | **Formato de save do engine** | Core: type keys + ToC + ordem de dependência | FlatBuffers/Cap'n Proto | doc `.riv` + core defs |
| Layout responsivo | UI/arte responsiva no runtime | Flexbox (Taffy) no `advance` | Constraints/anchors | Rive Layouts / Yoga |
| Feathering | Quick win pictórico | Falloff por SDF na borda (wgpu) | Blur mascarado | Rive Renderer |

---

## 9. Ordem de implementação sugerida (dependências)

1. **Formato Core + `advance(dt)` (§6, §0)** — a fundação. Type keys estáveis, ToC, "mutar hierarquia → um solve → renderer abstrato". Cristaliza o teu ADR de save e o loop.
2. **Data binding / propriedades tipadas (§4)** — unifique com as variables do Vol. 2. Habilita quase tudo depois; é a peça de maior alavancagem.
3. **State machines (§1)** sobre o node graph (Vol. 1 §7) com o tempo como entrada; reuse o interpolador do Smart Animate (Vol. 2 §5).
4. **Rigging + mesh (§2)** — LBS no vertex shader, unificado com o warp do Vol. 1 §5.1 num stack de deformers.
5. **Constraints (§3)** — nodes de target no mesmo solve; IK two-bone + FABRIK; `strength` bindável.
6. **Nested artboards + events (§5)** — composição = prefabs (Vol. 2 §3) no runtime; fecha o loop bidirecional com o data binding.
7. **Layout responsivo + feathering (§7)** — Taffy (reuso do Vol. 2) e feather por SDF (reuso do Vol. 1 §3.3). Feathering primeiro (quick win pictórico).

---

## 10. Síntese final da trilogia

Os três volumes compõem a visão inteira do PH2D, e o Rive mostra por que a *arquitetura* importa tanto quanto as features:

- **Vol. 1 — o que se pinta:** vetor expressivo e não-destrutivo (brushes, gradient mesh, appearance stack). A camada de **pintura profissional** que Rive/Figma **não têm**.
- **Vol. 2 — como se organiza:** grafo topológico, prefabs, tokens, layout. A camada de **sistema declarativo** que Procreate/Illustrator **não têm**.
- **Vol. 3 — como ganha vida e toca:** comportamento (state machines), corpo (rigging), relações (constraints), dados (binding), composição (nested), e o **formato runtime-first** que faz tudo caber num arquivo minúsculo que *executa* no engine.

A cadeia completa: **vetor pintado (V1) → estruturado em grafo e prefab (V2) → riggado e restrito (V3) → dirigido por state machine e data binding (V3) → composto e comunicando via events (V3) → serializado em formato Core e tocado no engine (V3)** — tudo sem sair do canvas, e tudo não-destrutivo/editável.

O Rive provou a metade "runtime-first + comportamento". O Figma provou a metade "sistema". Nenhum tem a camada de **pintura** do Vol. 1. **O PH2D é a interseção dos três** — e, sendo um *engine*, transforma o output de "asset animado" em "jogo".

---

## Referências principais

- **`rive-runtime`** (C++, MIT) — runtime de baixo nível: Core objects, `Artboard::advance`, Renderer abstrato, state machines, constraints, ViewModel, NestedArtboard. github.com/rive-app/rive-runtime
- **`rive-rs`** (Rust, MIT) — runtime Rust; renderiza via **Vello** (com limitações notadas em image meshes/clips). github.com/rive-app/rive-rs
- **DeepWiki rive-runtime** — *ViewModel Architecture* (MVVM: ViewModel/Instance/DataBind/BindableProperty) e *Nested Artboards and Component Lists* (host, virtualização).
- **Formato `.riv`** — doc oficial: header, fingerprint "RIVE", ToC, Core type keys, ordem de dependência, resiliência de versão. rive.app/docs/runtimes/advanced-topic/format
- **State Machines / Inputs / Transitions / Blend states** — docs do editor e runtime Rive.
- **Bones / Meshes / IK / Constraints** — docs do editor Rive (binding+weighting, pesos somam 100%, IK strength/bone-count, família de constraints).
- **Data Binding** — *"a shared language for designers and developers"*, *Lists/Images/Artboards*, *core concepts* (blog Rive) + docs de runtime.
- **Events** — General vs OpenURL, propriedades customizadas tipadas, listeners bidirecionais.
- **Skinning:** LBS clássico; DQS; *Velocity Skinning* (Rohmer et al. 2021) para vida secundária.
- **IK:** two-bone analítico (lei dos cossenos); FABRIK; CCD/Jacobiano como alternativas.
- Vols. 1 e 2 desta série: `kurbo`, `lyon`, `Clipper2`, `vtracer` (V1); `taffy`, DCEL, DTCG/variables, IDs estáveis (V2).
