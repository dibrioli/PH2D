# Plano — o Vector vira a ferramenta de UI/UX do PH2D

> **Volume de PLANO**, não de estudo. Os três manuais desta pasta (`_vetoriais`, `_figma`, `_rive`)
> dizem *o que existe no mundo*; este diz **o que construir aqui, em que ordem, com que porta única,
> e com que número ao lado**. Ele consome o Vol. 2 (Figma) inteiro, mais o §7 do Vol. 3 (Rive:
> layouts responsivos) e o item aberto *booleana não-destrutiva animável*.
>
> **Pedido do Enio (2026-08-01), verbatim no que decide:** *"tornar o nosso módulo Vector em algo tão
> eficiente e capaz como o Figma para gerar UI/UX/interfaces que podem ser usadas diretamente para
> criação de apps funcionais … deve ser capaz de gerar arquivos UI/UX úteis para esse nosso projeto
> (PH2D engine) pois a versão final da UI eu mesmo desenharei … UI animada, responsiva e com layout
> responsivo … também será usada para criar a UI dos games."*
>
> Escrito na `line/Vector` em 2026-08-01. Todo número aqui foi **medido nesta worktree** ou tem a
> linha de código citada; o que não foi medido está marcado **MEDIR** e é pré-requisito da wave.

---

## §0 — O pedido, e as três coisas que ele decide

A frase do Enio tem três metades, e cada uma corta o plano num lugar diferente:

1. **"gerar UI que pode ser usada diretamente para criação de apps funcionais"** — o produto não é
   um *mockup*. Um retângulo azul com a palavra "Salvar" não é um botão; é o desenho de um botão. A
   diferença entre este plano e o Vol. 2 do Figma inteiro está no **§2**.
2. **"gerar arquivos UI/UX úteis para esse nosso projeto … a versão final da UI eu mesmo desenharei"**
   — o consumidor nº 1 é **o próprio editor do PH2D**, que é Rust escrito à mão com 44 widgets, um
   `Panel` tipado e uma bateria de gates de costura. Um exportador que cospe SVG não serve para nada
   aqui. O que serve está no **§4/W6 e W8**.
3. **"UI animada, responsiva e com layout responsivo … também para os games"** — o consumidor nº 2 é
   o **runtime**, que não tem editor, não tem painel e não pode pagar um passe de autoria. Isso
   obriga o modelo a ser **dado**, não código, e obriga a resolução (layout, token, estado) a ser um
   passe determinista dentro de um `advance(dt)` (Vol. 3 §6).

E há uma quarta coisa, que o Enio não precisou dizer porque é a lei do repo (CLAUDE.md §0): **o teto
é o do hardware, e todo limite se mede antes de se escrever.** Toda constante deste plano ou vem com
a medição ao lado, ou vem com a palavra **MEDIR** e a sonda que a produz.

---

## §1 — O CENSO: o que já existe (medido hoje), e o que falta

⚠️ **Este censo é a metade mais valiosa do documento.** Seis achados mudaram o plano depois de
escrito o primeiro esboço, e cinco deles mudaram-no *para menos trabalho*. Quem retomar isto: **não
re-derive o censo por leitura do Vol. 2** — ele descreve o Figma, não este repo.

### 1.1 O que JÁ EXISTE e não deve ser reconstruído

| Peça | Onde | Estado |
|---|---|---|
| Modelo de documento vetorial | `ph2d-vec-scene` | `VecScene` = pilha plana `Vec<VecPath>` + `next_id`; a **ÁRVORE** é a Hierarquia ECS (ADR-0110) |
| Identidade que sobrevive a undo e a save | `VecPathId = u64` (`lib.rs:376`) | ✅ e há **precedente explícito** (`vec_label.rs`, `vec_connector.rs`): *"`Entity::to_bits()` é id de ALOCAÇÃO, e o undo respawna"* |
| Pose = `Transform` da entidade, geometria LOCAL | ADR-0111 | ✅ |
| Fonte autorada ≠ geometria cozida | ADR-0121 (`VecPath::cooked()`) | ✅ — é o pré-requisito de tudo que é não-destrutivo |
| Pilha de efeitos não-destrutivos | ADR-0132 (`effect.rs`, 19 kinds) | ✅ |
| Geometria DERIVADA no z da fonte | `LiveGeometry = BTreeMap<VecPathId, Vec<VecPath>>` (`ph2d-vec-render/src/lib.rs:114`) | ✅ **6 produtores** (offset/pattern/contour/symmetry/profile/**align**) |
| Formas VIVAS paramétricas | `VecShape::Param{kind,w,h,values:[f64;8]}` | ✅ — forma nova entra no catálogo e já salva/desfaz/re-cozinha |
| **TEXTO** como objeto vivo | `VecShape::Text(VecTextParams)` | ✅ família, tamanho, peso, entrelinha, tracking, alinhamento, **eixos variáveis** |
| Booleana robusta | `ph2d-vec-boolean` (`apply`, `apply_many`, `Arrangement`) | ✅ mas **destrutiva** (consome os operandos) |
| Recorte de subárvore | `ph2d_ecs::{ClipChildren, ClipMode, Mask2D}` | ✅ — a moldura ganha *clip* de graça |
| **Tokens de design** | crate **`ph2d-tokens`** + `docs/design/tokens.json` | ✅ **350 folhas, 4 temas, OKLCH, `build.rs` que gera os consts, e um gate de sync com parser INDEPENDENTE** |
| Cor perceptual | `ph2d-color` (`oklab.rs`, `oklch.rs`) | ✅ |
| Easing | `ph2d-anim::{Easing, EasingFamily, EasingMode}` | ✅ (inclui `Elastic`, que é uma *curva*, não um solver de mola) |
| Timeline com clips, containers, faixas, crossfade, sinais | `ph2d-timeline` + `ph2d-anim` + painel | ✅ |
| Correspondência de formas + interpolação | `ph2d-vec-blend` (Hungarian + espiral logarítmica) | ✅ — **é meio Smart Animate já construído** |
| Widgets nativos | `ph2d-editor-core/src/widget/` | ✅ **44**: slider, dropdown, tree_view, color picker, tabs, modal, … |
| Contrato de painel | `Panel` (`panel/panel_trait.rs`): `State: Default`, `ID`, `NODE_ID`, `paint`, `apply_event`, `populate` | ✅ |
| Texto real (shaping) | `ph2d-text` (**parley** 0.6), `ph2d-vector-font` | ✅ |
| Envelope de arquivo por SEÇÕES | `line/runtime` (`37ff53467`, `LEGACY_SCHEMA_FINAL = 48`) | ⚠️ **noutra linha** — ver §6.3 |

### 1.2 O que NÃO existe (verificado por grep, não por memória)

| Falta | Prova |
|---|---|
| **Moldura / artboard / frame** | `grep -rn "artboard\|Artboard" crates/ shells/` → **0 ocorrências** |
| Motor de layout | `grep "taffy\|cassowary"` em todo `Cargo.toml` → **0** |
| Arena de ids estáveis | `grep "slotmap"` → **0** |
| **Largura de quebra do texto** | `VecTextParams` tem `size/weight/line_height/tracking/align/axes` e **nenhum `width`** ⇒ o texto não é uma CAIXA, é uma linha |
| Componentes / instâncias / variants | nenhum componente ECS deles (registro tem **41**, `scene/registry.rs:374`) |
| Tokens no DOCUMENTO do artista | o `fill` de uma forma é `Paint::Solid(Rgba8)` **literal**; `ph2d-tokens` só serve o chrome do editor |
| Aliases / math / modos autoráveis / DTCG | `ph2d-tokens` tem 4 temas **fixos no enum `Theme`**, sem alias e sem import/export |
| Canais animáveis de UI | `PropKind` tem **9** variantes (`TranslationX/Y`, `Rotation`, `ScaleX/Y`, `Opacity`, `TimeRemap`, `Morph`, `Position`) — **nada de tamanho, raio, fill, stroke ou token** |
| Máquina de estados de UI | nenhuma (a de física é outra coisa; o nodegraph é CONGELADO, §6) |
| Booleana viva | `apply()` consome os operandos; não há componente nem produtor |
| Runtime que toca UI sem editor | `ph2d-runtime` não existe (Front 2 não construída) |

### 1.3 Os seis achados que mudaram o plano

1. ⚠️ **`ph2d-tokens` já é metade do §4 do Vol. 2, e está mais adiantado do que o estudo supõe.** 350
   folhas, **4 modos** (`Forge`/`Workshop`/`Sunstone`/`Blueprint`), **OKLCH na fonte**,
   `ColorToken::resolve(theme)` como porta de resolução, um **`build.rs`** que gera os consts a
   partir do JSON, e `tests/design_token_sync.rs`, que **re-parseia o JSON com serde_json — um
   parser independente** — e afirma que a API pública concorda com ele. Isto é exatamente a
   disciplina que o Vol. 2 §4 pede, já shipada. **A wave dos tokens deixa de ser "construa o sistema
   de tokens" e passa a ser "faça o DOCUMENTO referenciar o que já existe, e faça a tabela ser
   AUTORÁVEL".**
2. ⚠️ **E o doc-comment do `ph2d-tokens` MENTE sobre isso.** `lib.rs` diz *"(manual sync — future
   codegen via build.rs)"* e *"Automatic codegen tokens.json → this crate (planned; sync is manual
   for now)"* — o `build.rs` **existe** e o gate documenta que ele existe. Comentário velho mente
   (CLAUDE.md), e este mandou-me planejar uma wave que já estava feita. **Corrigir no primeiro
   commit que tocar a crate.**
3. ✅ **Suprimir o desenho de um filho já é possível, sem uma linha nova.** `dispatch`
   (`ph2d-vec-render/src/lib.rs:174`) faz `if let Some(items) = live.get(&path.id)` e itera `items`
   — um `Vec` **vazio** no mapa desenha **nada**. É o mecanismo de que a booleana viva precisa para
   apagar os operandos sem os apagar do documento.
4. ✅ **A booleana cabe no frame, e o número é de hoje** (`cargo test -p ph2d-vec-boolean --test
   measure_aligned_stroke --release -- --ignored`, nesta worktree):

   | cena | traço centrado | banda 2w | **INNER** | **OUTER** |
   |---|---|---|---|---|
   | rosquinha (w=0.2) | 0.053 ms | 0.053 | **0.067** | 0.067 |
   | estrela 5 pontas (w=0.2) | 0.157 | 0.158 | **0.185** | 0.181 |
   | estrela 24 pontas (w=0.2) | 0.799 | 1.376 | **1.440** | 1.453 |

   Contra **16,6 ms** de um quadro a 60 fps. O caso patológico (24 pontas) é **8,7% do quadro**, e o
   caso real é **0,4%**. ⇒ *booleana não-destrutiva animada em tempo real* não é aposta: é
   aritmética. (O que MEDIR ainda: **N** booleanas animadas ao mesmo tempo — §9.)
5. ⚠️ **A identidade que o Vol. 2 §0 exige já existe aqui, mas o repo escolheu o OPOSTO em dois
   lugares.** A timeline (`wire_id`) e os joints de física casam por **`stable_name_id` = hash FNV-1a
   do `Name`** — matching por NOME, exatamente a fragilidade que o Vol. 2 §0 manda não herdar
   (*"renomear um corpo DESACOPLA os joints dele"*, CLAUDE.md). A escolha foi certa **lá** (uma
   entidade sem path não tem outro id durável). Para UI **não é**: toda peça de UI carrega um
   `VecPathRef` ⇒ tem `VecPathId`, que sobrevive ao clone do undo e ao postcard do save. **Todo
   endereçamento deste plano — override de instância, matching de Smart Animate, alvo de binding —
   usa `VecPathId`.** O precedente já está escrito em `vec_label.rs` e `vec_connector.rs`; este plano
   só o segue.
6. ⚠️ **A UI do editor é feita de WIDGETS, não de retângulos.** 44 widgets com estado, foco,
   teclado, a11y e cinco gates de costura cada. Isto é o §2 inteiro.

---

## §2 — A decisão que o estudo do Figma NÃO podia tomar: desenho ≠ widget

O Vol. 2 descreve uma ferramenta que produz **imagens de interfaces**. O Figma não tem esse problema
porque o produto dele *é* a imagem: o engenheiro do outro lado reimplementa tudo à mão. O Enio pediu
o contrário — *"UI que pode ser usada diretamente"*, e *"a versão final da UI eu mesmo desenharei"*,
onde "a UI" é a deste app, que já existe, funciona, e é feita de 44 widgets tipados.

Há três respostas possíveis e só uma sobrevive:

| Resposta | O que acontece |
|---|---|
| **(a) O desenho VIRA o widget** (o retângulo desenhado passa a ser um slider) | ⛔ Um slider tem *drag*, *focus ring*, *teclado*, *a11y*, *clamp*, *undo*. Desenhar isso é reimplementar o `ph2d-editor-core` no canvas. Duas respostas a *"o que é um slider?"*, e a do canvas nasce pior. |
| **(b) O widget VIRA desenho** (o editor passa a interpretar um documento vetorial) | ⛔ Troca 44 widgets testados por um interpretador; e o Enio quer **desenhar** a UI final, não trocar o motor dela. |
| **(c) O desenho é a PELE; o widget é o COMPORTAMENTO; o token é a ponte** | ✅ |

**A escolha é (c), e ela tem uma consequência que organiza o plano inteiro:**

> **O artista não desenha um slider. Ele desenha a APARÊNCIA de um slider — e essa aparência é um
> conjunto de tokens e uma geometria nomeada que o widget nativo consome ao pintar.**

Isso é possível **hoje, e barato**, porque o caminho já está construído e gateado:
`tokens.json → build.rs → consts → 44 widgets`. Editar um token re-veste **todo o app**, com um gate
de parser independente provando que o número que o widget usa é o número que o artista escreveu.

Daí a escada de três degraus da W6, e a razão de o degrau 1 valer sozinho:

1. **Re-vestir por TOKEN** (a tabela inteira, autorável no canvas com preview ao vivo). Zero código
   de pintura novo. Já entrega *"eu mesmo desenharei a UI"* para cor, espaçamento, raio, tipografia,
   densidade e movimento — **350 folhas**.
2. **Pele por-widget** (`VecWidget`): a geometria autorada de um botão vira os parâmetros de pintura
   daquele widget — cantos, sombra, gradiente, estados. O comportamento continua sendo o do
   `ph2d-editor-core`.
3. **Layout e composição** (W8/codegen): a árvore de frames autorada emite o `populate`/`paint`/
   `apply_event` de um `Panel` real, usando os widgets do catálogo.

E a UI dos **jogos** não tem esse problema: lá não há catálogo nativo a respeitar, o desenho **é** a
UI, e o consumidor é o runtime (W8a). ⇒ **um documento, dois consumidores**, e é por isso que o §4
não bifurca o modelo.

---

## §3 — A espinha: uma pergunta, uma porta

As portas únicas que este plano institui. Cada linha é uma pergunta que **não pode** ter duas
respostas no código; a coluna da direita é onde ela vive.

| Pergunta | Porta ÚNICA |
|---|---|
| *Que tamanho tem esta moldura?* | o `w`/`h` do `VecShape::Param{Rect}` da própria moldura — **`VecFrame` não guarda tamanho** |
| *Onde este objeto está, depois do layout?* | `LayoutPose` (mapa derivado por frame), **nunca** o `Transform` autorado |
| *Que geometria este caminho desenha?* | `LiveGeometry` (a booleana viva é o **7º produtor**) |
| *Que valor concreto tem este token, neste modo?* | `ph2d_tokens::resolve` (estendida), **nunca** uma 2ª tabela |
| *Qual é o valor desta propriedade?* | `bind::resolve(entity, prop)` — literal, ou token, se houver binding |
| *Este objeto é o mesmo objeto do outro estado?* | `VecPathId` (nunca nome, nunca bits de entidade) |
| *Que aparência tem este widget?* | o catálogo de widgets, alimentado pelos tokens (**um** caminho de pintura) |
| *Quanto tempo passou?* | o `Playhead` (relógio único, W4.T7) — a máquina de estados **não** tem relógio próprio |

⚠️ **A regra que o repo cobra e que este plano herda:** *todo canal novo é side-metadata no registry,
nunca contrato.* Nenhuma wave aqui apenda campo a `Paint`, a `StrokeSpec` ou a `NodeManifest` — o
binding é uma **tabela lateral**, exatamente como o `KernelResolver` ganhou 6 canais sem mover
`NodeOp`/`OpResolver`/`NodeManifest`.

---

## §5 — O que fica FORA, e por quê (cercas de Chesterton)

| Fora | Motivo |
|---|---|
| **Vector networks (Vol. 2 §1)** | É o item **mais caro** do estudo (o próprio Vol. 2 o chama de *"a feature tecnicamente mais difícil de replicar"*) e o **menos alinhado** com o pedido: ele serve a *edição de canvas*, não a *geração de UI*. E o que ele entrega em UX já tem resposta aqui: o **Shape Builder** já pinta regiões de um arrangement, o compound path já existe, `path_cut`/`path_join` já fazem topologia, e o *Live Corners* já dá a quina. ⇒ **linha própria, quando o desenho expressivo for a prioridade** — não neste plano. |
| **Cassowary / solver de constraints** | O próprio Vol. 2 §2 diz que âncoras bastam para 90% do HUD; o caso relacional já tem resposta (align/distribute, snap 2-D). Um simplex incremental para o que dois `f64` resolvem é over-engineering. |
| **Colaboração multi-usuário** | Fora da tese (o PH2D é o engine do artista solo — `user_role`). |
| **Import de `.fig`** | Formato fechado. |
| **Solver de mola físico** | ⚠️ **Condicional**: só se o smoke da W7 mostrar que a curva `Elastic` não dá o *feel*. MEDIR antes. |

---

## §6 — Contratos congelados, schema, e o preço de cada bump

### 6.1 Nada do §6 é tocado (verificado estruturalmente)

| Contrato | Gate | Este plano |
|---|---|---|
| Nodes (`NodeOp=2`/`OpResolver=1`/`NodeManifest=8`) | `ph2d-nodegraph/tests/architecture_contract_surface.rs` | ⚠️ **intocado** — a HSM da W7 é crate nova, **não** um grafo de nós |
| Tools (`Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4`) | `ph2d-editor-core/tests/architecture_tool_contract_surface.rs` | intocado — os modos novos são internos ao `VectorTool` |
| Vector doc (`VectorOp≤16`/`Vertex`/`Segment`/…) | `ph2d-vector-doc/tests/architecture_vector_contract_surface.rs` | intocado — **o gate varre só `ph2d-vector-doc` + `-traits`**, e todas as waves vivem em `ph2d-vec-*`, `ph2d-ecs`, `ph2d-panel-vector` e `shells/desktop` |

### 6.2 O registro de componentes

**41 hoje** (`ph2d-ecs/src/scene/registry.rs:374`). O plano acrescenta **9**: `VecFrame`,
`VecBoolGroup`, `VecLayout`, `VecLayoutItem`, `VecAnchors`, `VecBindings`, `VecComponentMain`,
`VecInstance`, `VecWidget` → **50**. ⚠️ **O contador é TRÊS** (`ph2d-ecs`, `ph2d-render`, `ph2d-script`,
cada um rodando na suíte da própria crate) — a família que já ficou **vermelho-latente duas vezes** na
`line/Vector`. Todo commit que acrescenta componente bate os três.

### 6.3 Os bumps, e o que cada um custa

⚠️ **A regra que este plano segue e que a §5 do CLAUDE.md documenta em sete lugares:** *componente
NOVO não bumpa nada* (cunha `stable_type_id = blake3(NOME)[..8]`); *campo apendado a componente
EXISTENTE bumpa*, porque o postcard é posicional **dentro** do blob — e **um bump recusa todo projeto
já salvo**.

| Wave | O que muda | Bump |
|---|---|---|
| W0, W1, W3, W5, W6 | componentes NOVOS | **nenhum** |
| W2a | `VecTextParams.wrap_width` — campo apendado a componente **existente** | ⚠️ **um bump global hoje** |
| W4 | `PropKind::Token` (variante apendada) | `DOC_VERSION` +1 (quebra dura, a política da timeline) |
| W4, W8a | a tabela de tokens e o documento de UI viajam no arquivo | **seções novas** do envelope |

⚠️ **A dependência entre linhas, e é a mais importante deste plano:** a **F1.W1** da `line/runtime`
(*uma versão por `ComponentBlob`* — 18 dos 37 bumps históricos pousaram lá dentro) **apaga o custo da
W2a**. Se a W2a for antes, ela paga um bump global por um campo que 99% dos documentos não têm.
⇒ **ordenar a `line/runtime` antes da W2**, ou aceitar o bump com o motivo escrito.

⚠️ **E há uma colisão de número já em voo:** esta linha tem `PROJECT_SCHEMA = 50` (provisório) e a
`line/runtime` **matou o número** (`LEGACY_SCHEMA_FINAL = 48` + envelope). *O valor se CONTA contra o
`main` do dia, e aqui o que se conta é qual das duas formas sobrevive* — decisão de integração, não
de linha.

---

## §7 — Dependências novas

| Crate | Wave | Por quê | Risco |
|---|---|---|---|
| **`taffy`** | W2 | Flexbox/Grid/Block da spec CSS, Rust puro, sem build-deps; o motor de Dioxus/Zed/Bevy/Blitz. Escrever o nosso é reproduzir uma spec inteira | **ADR próprio** (dep + escolha foundational). ⚠️ Fixar a versão no ADR e **MEDIR o custo de build** — o repo é sensível a isso (§2 do CLAUDE.md) |
| `serde_json` | W9 (DTCG) | ✅ **já está na árvore** (`ph2d-mcp`, `ph2d-asset`, `ph2d-tokens`) | nenhum |
| `parley` | W2a (quebra de linha) | ✅ **já está na árvore** (`ph2d-text` 0.6) | nenhum |

⚠️ `deny.toml` barra `unknown-git` ⇒ tudo vem de crates.io com licença conhecida. E `machete` cobra
dep declarada e não usada — a dep entra **no commit que a usa**.

---

## §8 — A ordem, e por que ela

```
W1 booleana viva ────────────────────────────► (independente; o Enio pediu; a menor)
W0 moldura ──┬──► W2 layout ──┬──► W3 âncoras
             │   (+ W2a texto)│
             │                └──► W8b codegen ──► (o editor)
             └──► W5 componentes ──► W6 widget ──► W7 interação ──► W8a runtime ──► (os jogos)
W4 tokens ───┴──► (atravessa TUDO: W4 é pré-requisito de W6 e de W7)
```

1. **W1 (booleana viva)** primeiro: é independente, foi pedida, e o número que a viabiliza já está
   medido. Fecha um item aberto sem esperar nada.
2. **W0 (moldura)**: nada de responsivo existe sem ela.
3. **W4 (tokens)** cedo, e não tarde: é a de **maior alavancagem** (Vol. 2 §4), já tem metade
   construída, e é a que sozinha entrega *"eu mesmo desenharei a UI"* (degrau 1 da W6). ⚠️ Fazê-la
   depois da W5/W6 significaria autorar componentes com literais e migrá-los depois.
4. **W2 + W3**: a responsividade, que é requisito de plataforma (iPad × desktop), não luxo.
5. **W5 → W6**: o prefab e depois o vínculo funcional (o vínculo só faz sentido sobre instâncias).
6. **W7**: depende de casar por id (W0/W5) e de tokens (W4).
7. **W8**: os dois backends, quando há o que exportar.
8. **W9**: interop por último — ninguém importa DTCG antes de haver tabela autorável.

⚠️ **A `line/runtime` (F1.W1) atravessa isto na W2a** — §6.3.

---

## §9 — As medições que faltam (fazer ANTES da wave, não depois)

Cada uma é uma sonda `#[ignore]`, e cada uma decide um número que este plano **deliberadamente não
escreveu**:

| # | Medição | Decide |
|---|---|---|
| M1 | `N` booleanas vivas animadas em simultâneo (5 / 20 / 100) | se a W1 precisa de orçamento por frame, ou se o memo basta |
| M2 | passe de layout com 10 / 100 / 1000 nós; árvore reconstruída × memoizada | o teto de nós de uma moldura, e se a memoização é necessária |
| M3 | custo da quebra de linha (parley) por bloco de texto, por frame | se o texto precisa de cache de medição |
| M4 | resolução de binding: 0 / 100 / 1000 propriedades bindadas | o custo do §4, e se a tabela achatada é obrigatória já no editor |
| M5 | memória de 1000 instâncias (dhat) — prototipal × cópia | confirma a tabela do Vol. 2 §3 **neste** modelo |
| M6 | `Elastic` × mola física num smoke lado a lado | se o solver de mola se constrói ou fica fora |
| M7 | custo de build do `taffy` (cold + warm) | entra no ADR |
| M8 | tamanho do arquivo: uma tela de UI típica como seção | o formato, e o carry |

---

## §10 — Riscos nomeados

1. ⚠️ **Duas árvores.** A do ECS (autorada) e a do taffy (derivada). Mitigação: a segunda é
   **memoizada e reconstruída**, nunca autorada — e há gate provando que ninguém escreve nela.
2. ⚠️ **Duas poses.** O `Transform` autorado e a pose de layout. Mitigação: a de layout **nunca**
   toca o `Transform`; arrastar dentro de um fluxo **reordena**. Sem isso, o undo enche de passos
   espúrios (é o defeito exacto que o `canonicalize` do undo global existe para matar).
3. ⚠️ **Duas aparências para um widget** (o desenho e o pintor nativo). Mitigação: a prévia do canvas
   chama **o pintor real**.
4. ⚠️ **Codegen que o CI recusa.** Mitigação: os gates do repo correm sobre o código gerado, e isso é
   requisito de aceitação da W8b, não polimento.
5. ⚠️ **O bump da W2a** (§6.3) — ordenar a `line/runtime` antes, ou pagar com o motivo escrito.
6. ⚠️ **Escopo.** São nove waves; o Vol. 2 inteiro é maior que qualquer wave já feita nesta linha.
   Mitigação: **W1 e W0 entregam valor sozinhas**, e a W4 entrega *"desenhe a UI do app"* antes de
   qualquer componente existir. Nada aqui precisa de um *big bang*.

---

## §11 — Tabela-resumo

| Wave | Entrega | Componentes | Schema | Dep | ADR | Smoke |
|---|---|---|---|---|---|---|
| **W1** ✅ | booleana não-destrutiva, animável | +1 (**42**) | — | — | — | `=48` |
| **W0** ✅ | a moldura | +1 (**43**) | — | — | — | `=49` |
| **W4a** ✅ | o binding de token no documento + o modo re-veste | +1 (**44**) | — | — | — | `=51` |
| **W4b.1** ✅ | **o ALIAS** — um token SEGUE outro, com detecção de ciclo na PORTA | `tokens_link_id` | `PROJECT_SCHEMA` 56→57 | — | **nenhum** | `=51` |
| **W4b.2** ✅ | o readout de **CONTRASTE** — a lei WCAG vira DADO, e o painel a mede onde a escolha é feita | — | — | — | **nenhum** | `=51` |
| **W4b.3+/c** | o resto: math · DTCG · tokens de ESCALA (a fronteira `const fn`) · animar token | — | `DOC_VERSION`, seção | — | sim (indireção) | — |
| **W2** ✅ | auto layout | +2 (**46**) | — (W2a fica) | **taffy** | **0153** | `=50` |
| **W3** ✅ | âncoras | +1 (**47**) | — | — | — | `=52` |
| **W5a** ✅ | mestre + instância + override esparso | +2 (**50**) | — | — | — | `=53` |
| **W5b** ✅ | a lista de PEÇAS (a porta do override) · Update Main · Swap | — | — | — | — | `=56` |
| **Z-order** ✅ | a lei das game engines: DFS = ordem de desenho + o **Z global** na Arrange | — | — | — | — | `=57` |
| **W5c** ✅ | variants (a instância escolhe QUAL versão) | — | **—** | — | **—** | `=58` |
| **W6.1** ✅ | a tabela de COR vira AUTORÁVEL (o degrau 1 do §2) | — | ⚠️ **bump** | — | **—** | `=59` |
| **W6.2** ✅ | pele por-widget (`VecWidget`): a forma veste um controle do catálogo | +1 | — | — | — | `=60` |
| **W6.3** ✅ | layout: a árvore autorada vira `Panel` — ⚠️ **é a W8b**, entregue pelas W8b.1+W8b.2 (§ degrau 3) | — | — | — | **nenhum** | `=62` |
| **W7** 🔨 | estados + Smart Animate (**autoria**; runtime não) | — | ⚠️ `PROJECT_SCHEMA`, **não** `DOC_VERSION` | — | **nenhum** | `=61` |
| **W8b.1** ✅ | o codegen: a árvore descreve um painel e o app escreve o código dele | — | — | — | — | `=62` |
| **W8b.2** ✅ | o gerado vira painel VIVO (crate, registro, runtime das rows) | — | — | — | **nenhum** | `=62` |
| **W8b.3** ✅ | a row MEXE na arte (o vínculo row → forma, derivado do tipo) + o pill **UI** | +1 (**53**) | — | — | **nenhum** | `=62` |
| **W8b.4** ✅ | a POSIÇÃO do controle é autorada (sobrevive ao arquivo e ao Ctrl+Z) | +1 (**54**) | — | — | **nenhum** | `=62` |
| **W8a** | o runtime (para os jogos) | — | seção | — | sim (fronteira) | — |
| **W9** | DTCG / SVG / export | — | — | — | — | — |

⚠️ **A linha da W7 traz TRÊS previsões que a construção refutou, e elas são o padrão desta
tabela:** o schema seria `DOC_VERSION` e foi **`PROJECT_SCHEMA`** (a tabela de estados viaja no
`ProjectState`, que é a unidade do undo — não num blob que carrega a própria versão); o ADR era
dado como certo (*"sim (HSM)"*) e **nenhum foi escrito** (nada do §6 é tocado, nenhuma dep externa
entra, e as decisões vivem no doc da crate — a linha fica **fora** da disputa de número); e a cena
prevista era `=55`, mas o número livre no roteador era o **`=61`**. *Uma tabela de previsões é útil
enquanto a construção puder contradizê-la por escrito.*

⚠️ **Os números de CENA desta tabela são previsões, e uma delas já colidiu.** O W2 e o W4a
previam os dois `=50`; o auto layout chegou primeiro e a cena dos TOKENS ficou **inalcançável em
silêncio** (o roteador é uma lista de `if level == N` e o primeiro vence). Ela mudou para `=51`, e
as previsões seguintes deslizaram um. O gate `no_two_smoke_scenes_claim_the_same_level` é o que
torna a próxima colisão alta em vez de silenciosa — **confira o número livre no roteador**, não
nesta tabela.

⚠️ Os números de ADR são **PROVISÓRIOS** até a integração — o valor se conta contra o `main` do dia
(seis renumerações já aconteceram neste repo). ⚠️ **E o mesmo vale para os números de cena de smoke:**
`=48..=56` estão livres *nesta linha* (a `=47` é o alinhamento, a última daqui), mas outra linha
paralela pode reivindicar os mesmos — conte contra o `build_smoke_router` do `main` do dia.

---

## Referências

- `Estudos/PH2D_manual_features_figma.md` — Vol. 2 (vector networks, auto layout, components,
  variables, smart animate).
- `Estudos/PH2D_manual_features_rive.md` §1, §4, §6, §7 — state machines, data binding, formato
  runtime-first, layouts responsivos.
- `Estudos/PH2D_manual_features_vetoriais.md` — Vol. 1 (o que se desenha).
- ADR-0108 (reposicionamento), ADR-0110 (paths são entidades ECS), ADR-0111 (geometria local, pose no
  `Transform`), ADR-0121 (fonte autorada ≠ cozida), ADR-0132 (pilha de efeitos), ADR-0148 (perfil de
  largura é componente ECS + um assador).
- `docs/design/tokens.json` (350 folhas, 4 temas) + `crates/ph2d-tokens/` (`build.rs`,
  `tests/design_token_sync.rs`).
- `docs/Runtime/` (`line/runtime`) — o envelope por seções (F1.W0) e a versão por `ComponentBlob`
  (F1.W1).

---

## ⛔ Recusas MEDIDAS — 8, e nenhuma volta à fila

> ⚠️ **Cortado em 2026-08-18** — o §4 (as waves, 1.235 linhas) foi **verbatim** para o arquivo,
> com remontagem conferida por sha256. Estas recusas ficaram lá; o índice mantém-nas alcançáveis.

| onde | a recusa |
|---|---|
| [(topo)](../../archive/docs-2026-08-18/Vector%20Module/PLANO_UI_UX_padrao_figma.md#L10) | ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma |
| [W4 (original) — OS TOKENS CHEGAM AO DOCUMENTO](../../archive/docs-2026-08-18/Vector%20Module/PLANO_UI_UX_padrao_figma.md#L455) | um alias resolve · um **ciclo** é recusado com erro nomeado (nunca stack overflow) · trocar de modo |
| [A terceira rodada: **o Z é a ÚNICA porta, e o campo estava MUDO** ✅ **](../../archive/docs-2026-08-18/Vector%20Module/PLANO_UI_UX_padrao_figma.md#L673) | Enio: *"Z-index não funcionou e os botões de Arrange não foram convertidos para funcionar com |
| [A terceira rodada: **o Z é a ÚNICA porta, e o campo estava MUDO** ✅ **](../../archive/docs-2026-08-18/Vector%20Module/PLANO_UI_UX_padrao_figma.md#L707) | foram recusadas pela própria lei (renumerar os vizinhos é escrever no número de um objeto que o |
| [W7 — A INTERAÇÃO: máquina de estados + Smart Animate 🔨 **A METADE DE A](../../archive/docs-2026-08-18/Vector%20Module/PLANO_UI_UX_padrao_figma.md#L1135) | ⛔ **A MOLA foi MEDIDA e o solver NÃO se constrói** (a M6 do §9, fechada). A forma já está no |
| [W7 — A INTERAÇÃO: máquina de estados + Smart Animate 🔨 **A METADE DE A](../../archive/docs-2026-08-18/Vector%20Module/PLANO_UI_UX_padrao_figma.md#L1150) | ⛔ **O mouse não dirige nada** — e a ausência é **decisão**, não esquecimento. Um hover que |
| [W7 — A INTERAÇÃO: máquina de estados + Smart Animate 🔨 **A METADE DE A](../../archive/docs-2026-08-18/Vector%20Module/PLANO_UI_UX_padrao_figma.md#L1155) | ⛔ **O input de runtime → token → arte** (a ponte do Vol. 3 §4 que o smoke desta wave pedia) é a |
| [W7 — A INTERAÇÃO: máquina de estados + Smart Animate 🔨 **A METADE DE A](../../archive/docs-2026-08-18/Vector%20Module/PLANO_UI_UX_padrao_figma.md#L1157) | ⛔ **A hierarquia** (um menu que abre, um card que expande com sub-estados) — ver acima. |
