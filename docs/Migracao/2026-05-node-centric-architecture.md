# Arquitetura node-centric — PH2D: um substrato unificado, avaliadores plurais, UX artista-primeiro

**Data:** 2026-05-21
**Status:** Proposta de arquitetura (aguardando ratificação do Enio → vira ADR-0030..0038 + plano de waves).
**Decisão do dono:** a engine gira em torno de um **sistema de nós** com **múltiplos domínios paralelos e comunicáveis** (shaders, motion, programming/gameplay, sound) — modelo Houdini/Unreal/Blender — **mais** ferramentas de autoria imperativas (Image Tools, painter) **fora** do grafo. Referência de design: protótipo **MiniCavalryV2** (não é port).
**Como esta versão foi produzida:** 4 rodadas de opinião arquitetural independente (paisagem competitiva · UX de artista · teoria/first-principles dataflow · resolução da tensão grafo-único-vs-contextos), + inspeção de código (`HEAD = c806bd2`) + investigação do modelo Houdini/Blender/TouchDesigner.
**Doc irmão:** [`2026-05-foundational-parallelism-three-bottlenecks.md`](2026-05-foundational-parallelism-three-bottlenecks.md) — o substrato multi-agente que torna esta arquitetura paralelizável.

---

## 0. Tese unificadora — isolamento FBP = unidade multi-agente = nó teoricamente ótimo

A conversa que precede este doc tinha dois fios — **destravar agentes paralelos** e **tornar a engine node-centric** — e a teoria de dataflow os colapsa num **único princípio**:

> Um nó bem-tipado e puro — caixa-preta de **Flow-Based Programming**: **tipos de porta + classe de efeito como ÚNICO contrato, zero estado compartilhado** — é simultaneamente **(a)** o design teoricamente ótimo de nó **e (b)** exatamente a unidade que um agente paralelo constrói em isolamento e testa sozinho.

A engine cresce por **duas famílias de crate isolado**, ambas wire-adas por codegen:

```
ph2d-node-<domínio>-<slug>/   ← NÓ: caixa-preta FBP. Contrato = portas tipadas + efeito + lowerings.
ph2d-tool-<slug>/             ← FERRAMENTA: imperativa, manipulação direta. Fora do grafo (ADR-0027).
```

Zero toque no core, zero colisão (wiring gerado), build incremental pequeno, compila no slot do agente. O agente que implementa um nó vê **só** `(portas_in, portas_out, efeito, clock, lowerings[])` — não vê o resto do grafo, nem estado compartilhado, nem o agendador. Os **três gargalos** (doc irmão) são o **pré-requisito** físico que torna isso real.

---

## 1. Houdini NÃO é o padrão-ouro — a síntese

As 4 opiniões convergem: **Houdini é padrão-ouro de UM eixo** (modelo de atributos + linguagem de compute que compila) e é **modelo ruim** de usabilidade, real-time e determinismo. A separação dele em "contextos como apps" é **contingência histórica** (DCC de VFX de 1996), não ótimo teórico. O north-star é uma **síntese**:

| Roubar | De quem | O quê |
|--------|---------|-------|
| Modelo de dados | **Houdini** | atributos arbitrários por elemento fluindo na aresta (não sockets fixos) |
| Mecanismo de compute | **Blender** | **Fields / atributos anônimos** — compute lazy por-elemento *inline* num socket, sem sub-rede VOP separada |
| Runtime | **Unreal (Material/Niagara)** | **compilar, não interpretar** (apresentação→WGSL); **nunca** interpretar grafo de lógica no hot path (o erro da VM de Blueprint) |
| Áudio | **Unreal (MetaSounds)** / FAUST | grafo de DSP sample-accurate, relógio fixo (synchronous dataflow) |
| UX de nó | **Substance / TouchDesigner** | defaults sãos, result-named, live-preview, viewport-first |
| Loop real-time | **TouchDesigner** | o grafo *é* o runtime; custo por frame previsível |
| **INVENTAR** | ninguém faz | **membrana de determinismo como tipos** + formato textual diffável p/ multi-agente + budget-aware graph |

Copiar Houdini inteiro herdaria a curva de aprendizado e o modelo offline sem ganhar real-time nem segurança de gameplay.

---

## 2. O modelo — um substrato unificado, avaliadores/views plurais

A tensão "grafo único heterogêneo vs contextos tipados separados" é **majoritariamente falsa**: falsa no nível de **dados**, real no nível de **avaliação**. Os três sistemas de referência (Houdini, TouchDesigner, Blender) **já são** esta reconciliação — substrato compartilhado por baixo, famílias tipadas por cima — e movem-se para *mais* substrato e *menos* contextos. Blender é o corte mais claro: framework unificado (socket-geometria único, fields), mas shader-tree / geometry-tree / compositor **separados, porque o alvo de avaliação difere** (GPU-compilado vs CPU-lazy vs imagem).

### 2.1 A fronteira precisa
> **DADOS + CONTRATO = unificado. AVALIAÇÃO (relógio + alvo de compilação + classe de efeito) = plural e tipado.**

```
┌──────────── ph2d-nodegraph — SUBSTRATO ÚNICO (estável, contrato de TODOS os agentes) ────────────┐
│ modelo de atributos · tipos de porta ALGÉBRICOS (carregam domínio+dimensionalidade+RELÓGIO) ·     │
│ sistema de efeitos Pure/Temporal/Stateful + regra da membrana · delay `pre` + aciclicidade ·      │
│ Fields/atributos anônimos (compute compartilhado) · formato textual diffável · registry de portas │
└───────────────────────────────────────────┬───────────────────────────────────────────────────────┘
        ┌───────────────────┬────────────────┼────────────────────┬───────────────────┐
        ▼                   ▼                 ▼                    ▼
  AVALIADOR shader     AVALIADOR audio   AVALIADOR motion    AVALIADOR gameplay   ← PLURAL (por modelo
  pull → WGSL          sync-dataflow     pull-no-playhead    push → Luau            de avaliação)
  (GPU)                relógio fixo      (frame)             (CPU, determinístico)
        +                   +                 +                    +
  view/editor por domínio · paleta por-editor · contexto ESCONDIDO do artista · wiring restrito p/ região
```

### 2.2 A regra de decisão (de bolso)
> "Dois nós deste tipo, ligados, podem ser cozidos pelo **mesmo agendador sem mudar relógio nem alvo de compilação**?"
> **Sim** → mesma região, **porta tipada** no grafo único. **Não** → **domínios separados**, e a conexão entre eles é uma **travessia de membrana tipada** (com `pre`/conversão de clock explícita), nunca canvas comum nem import-node ad-hoc.

Corolário p/ agentes: se o nó precisa de **um único** lowering e **um único** clock → vive numa região. Se precisaria de dois lowerings no mesmo grafo → há fronteira de domínio ali, e ela vira porta de membrana, não um nó que tenta ser as duas coisas.

---

## 3. O substrato unificado (`ph2d-nodegraph`)

Contrato fino, foundational, **estável** (raramente tocado — é o que mantém o paralelismo vivo). Sete coisas unificadas:

1. **Modelo de atributos** (sabor Houdini): arestas carregam colunas tipadas sobre um domínio (pixels, samples, instâncias, entidades) — não um escalar.
2. **Tipos de porta algébricos** que carregam **domínio + dimensionalidade + RELÓGIO**: `Field<f32, clock=Frame>` ≠ `Field<f32, clock=Audio>`. (Conserta o vazamento "mesmo tipo, taxa de amostragem diferente".)
3. **Sistema de efeitos** `Pure | Temporal | Stateful` + **regra da membrana** (`Stateful`/push não conecta no lado pull) — **checado em compile-time**.
4. **Aciclicidade por construção** + operador de delay tipado de 1-tick (`pre`, à la Lustre). Feedback temporal nunca é aresta-de-volta; é `pre`. Sem detecção de ciclo em runtime.
5. **Fields / atributos anônimos compartilhados** (sabor Blender): o compute reusável vive aqui — acaba com o "Wrangle quádruplo" (math/noise/remap reimplementados por domínio).
6. **Formato textual diffável/mergeável**: lista estável de nós + arestas, **IDs estáveis**, **layout segregado da semântica** (posição num campo que nunca afeta cook). Requisito multi-agente E de save/migração.
7. **Registry de tipos de porta**: o vocabulário compartilhado que faz nós de **agentes diferentes, em crates diferentes, encaixarem**.

**Motor de cook único, genérico:** topo-sort + dirty-propagation + cache (modelo incremental/self-adjusting, > dirty-bit ingênuo), com **política de cache explícita por nó + orçamento de memória** (o cache implícito universal é o problema #1 de perf real do Houdini).

---

## 4. O compute compartilhado (`ph2d-expr`) — o "VEX/VOP" da PH2D, refinado

A peça que o Houdini ensina e o Blender melhora: **uma camada de expressão reusada em todos os domínios.**

- **Fields inline, não sub-rede VOP** — o poder por-elemento vive *dentro* do grafo principal (Blender provou que dá; mais intuitivo que entrar numa rede VOP separada).
- **Escape textual obrigatório desde o dia 1** (a lição VEX/wrangle): nó = fluxo de **dados/composição**; texto = fluxo de **controle/expressão/iteração**. Force texto quando o nó codificaria loop, aritmética densa, ou hot-loop. "17 linhas de texto > dezenas de nós ilegíveis."
- **Lowering plural**: a mesma expressão compila para **WGSL** (domínios GPU) ou **Luau/bytecode** (domínios CPU). Um nó é uma **spec de operação abstrata + N lowerings**, não código de um runtime só (senão um nó "que roda em Rust" nunca vai pra GPU).

`ph2d-expr` deve permanecer **mínimo** (expressão, não linguagem completa) — Luau continua a linguagem de gameplay.

---

## 5. Os avaliadores plurais + a membrana como tipo

Cada domínio declara seu **modelo de avaliação** (events=push / behaviors=pull, Conal Elliott); a membrana garante que push e pull só se tocam numa direção:

| Domínio | Modelo de avaliação | Alvo de lowering | Determinismo |
|---------|---------------------|------------------|--------------|
| **Gameplay / logic** | **push** (eventos mutam estado), ordem total estável | Luau/bytecode (CPU) | **DETERMINÍSTICO (HR-5)** — escreve SimWorld |
| **Motion** | **pull** no playhead `t` | Vello / sprite instancing / SDF | isento (apresentação) |
| **Shader / material** | **pull** amostrado por-pixel/frame | **WGSL** (GPU) | isento (visual) |
| **Sound / DSP** | **synchronous dataflow** (relógio fixo, escalonamento estático — **dá o frame budget de graça**) | grafo DSP (`ph2d-audio`) | isento (não-sim) |

**A membrana generalizada, checada por tipo** (ADR-0021 elevado a regra de grafo):

```
SimWorld (canônico, determinístico, HR-5)  ── escrito SÓ por nós push/Stateful do gameplay
        │  extract! (mão única)
        ▼
PresentWorld  ── nós pull/Pure|Temporal leem aqui; comunicam entre domínios por portas
                 tipadas atravessando fronteiras de membrana (com `pre`/clock-conversion):
                 gameplay-export ▶ motion-driver ▶ shader-uniform ;  motion ▶ sound-trigger (Event)
```

- **Só o domínio gameplay escreve o SimWorld** → só ele exige determinismo (Pcg64Mcg, sem HashMap iter, ordem por bits, replay-hash). **Motion/shader/sound são isentos de HR-5** — puramente visuais, como Radiance Cascades. Isso elimina quase toda a preocupação de determinismo cross-platform.
- **Enforcement = arch-gate** (encaixa na cultura de pre-commit que já temos): um teste recusa nó `Stateful` referenciado no lado pull, e força toda travessia de membrana a usar a porta de export designada. A membrana é **provada estaticamente**, não confiada.

---

## 6. O domínio gameplay — ~80% pronto (confirmado em código)

O `ph2d-script::host` já implementa o runtime. **A API canônica `ph2d.*` é a da PH2D existente** (o MiniCavalry espelhou com fidelidade):

| `ph2d.*` | Mecanismo PH2D existente | Nota |
|----------|--------------------------|------|
| `ph2d.set(self,campo,v)` | `EntityWrite` → `drain_writes()` | **escrita DIFERIDA** (drain no fim do tick) — determinismo + membrana |
| `ph2d.get(self,campo)` | `provide_read`/`clear_reads` | leitura de snapshot |
| `ph2d.spawn/despawn/attach_script` | `SpawnQueue` → `drain_spawns()` | fila diferida |
| `ph2d.state_table(self)` | `StateTable` (`lateral.rs`) | HR-16 POD, `pairs_sorted()` |
| `ph2d.input(key)` | `InputSnapshot` | por-frame |
| `broadcast`/`message_send` | `messaging.rs` (Defold) | FIFO same-sender→same-target |
| `espere`/`deslize`/`para sempre` | `Scheduler` + `coroutine.yield` | 1:1; `deslize` passo fixo `1/60` = correto p/ determinismo |

**Duas superfícies de autoria, uma IR, um runtime** (ADR-0036): **blocos** (Scratch-style, sem fio; `ph2d-blocks`, authoring-time, no editor) **e** **node-programming** (Blueprint-style, com fio; domínio do grafo). Ambos compilam para Luau/bytecode; o runtime não conhece a diferença (reusa HR-10/HR-17, superfície de runtime mínima).

**Hats → handlers** (`on_start`/`on_tick`/`on_msg_*`/`on_state_*`/`on_collide_*`): registro direto; `on_state_*` via componente FSM lendo `state_table`; `on_collide_<tag>` via `ph2d-collision2d` emitindo **mensagem** (colisão é sistema ECS produtor de eventos). **Colisão (ADR-0036):** `ph2d-collision2d` lite (grid broadphase + tags + ordem por bits), **não** rapier (reservado p/ dinâmica/física real).

---

## 7. UX artista-primeiro — preocupação arquitetural de 1ª classe

**Nós não REDUZEM carga cognitiva, eles a REALOCAM** (sintaxe → raciocínio espacial). "Fazer com nós" ≠ "intuitivo" — um sistema de nós só é intuitivo se *gerencia ativamente* essa carga. North-star: *"sentir como uma ferramenta de manipulação direta que por acaso tem um grafo atrás."* Os 7 princípios (gates de UX, não opcionais):

1. **Viewport-first, graph-second** — o artista faz 80% das edições sem abrir o grafo (gizmos no viewport editam parâmetros de nó, à la Blender).
2. **Live-preview em TODO nó**, não só na saída — tornar o invisível visível (Houdini display-flag + Cables data-flow ao vivo).
3. **Progressive disclosure via sub-grafos colapsáveis nomeados** — uma rede de 200 nós lê como 6 caixas rotuladas. Sub-grafo com params expostos *é* um preset/tool.
4. **Presets / nós-compostos result-named como porta de entrada**; primitivos são o *escape hatch* que se acha depois. O artista começa no bolo, não na farinha.
5. **Result-named, zero jargão** em nós **e portas** (sem SOP/VOP/"detail attribute"). É o "artista-primeiro" da PH2D — um moat de UX, guardar com gate.
6. **Wiring restrito anti-espaguete** — portas tipadas+coloridas que **recusam conexão inválida** (afordância no momento da conexão, não erro tardio); auto-layout, reroute, comment frames.
7. **Escape textual ("code node") opcional, não a estrada principal** — Houdini erra por tornar VEX *obrigatório*.

**Esconder "contexto" totalmente:** contexto = o editor que você abriu, **nunca** um modo dentro do grafo; zero taxonomia SOP/VOP na UI; **paleta por-editor** (só os nós daquele domínio).

**EVITAR a todo custo:** trabalho de baixo nível obrigatório · taxonomia de contexto exposta · jargão de implementação · onboarding de canvas-em-branco-de-primitivos · paleta-mega única · **posicionar nós como prestígio e ferramentas imperativas como rodinhas** (inverte como o artista trabalha).

---

## 8. As duas famílias de feature

- **Nós** (`ph2d-node-<domínio>-<slug>`): declarativos, pull/sync, caixa-preta FBP, por domínio. A massa do crescimento.
- **Ferramentas** (`ph2d-tool-<slug>`): imperativas, push, manipulação direta (painter, Image Tools, brush, bgremoval). ADR-0027 existente. **São tools TERMINAIS, não rampa pro "tool de verdade"** — para muitas tarefas (pintar, mascarar, retoque) são a ferramenta *correta*, não uma versão inferior do grafo.
- **Bridge bidirecional:** máscara pintada usável como input de nó; saída de nó pintável-por-cima; manipulação direta edita parâmetros de nó; bake/flatten de resultado de nó pra camada imperativa. O artista nunca sente que "trocou de paradigma".

Ambas as famílias: crate isolado + codegen-wired → **paralelizáveis por múltiplos agentes**.

---

## 9. Cook vs live — por-subgrafo (todos os domínios de apresentação)

> Subgrafo que **não lê atributo de sim em runtime** → estático → **cozido em asset nativo** (atlas / cena Vello / clip / material WGSL). Subgrafo que **lê do PresentWorld a cada frame** → **avaliado vivo**, sob budget HR-4 + pools HR-3.

O cooker particiona analisando "lê sim/runtime?". 80% deve cozinhar; live só o dirigido por gameplay. **Atributos de stream ≠ componentes ECS** (efêmeros, sem identidade, por-instância; vivem no avaliador, leem do PresentWorld, **nunca** armazenam no ECS). **Cloner** (1 nó → N instâncias) **não tem análogo ECS** — é multiplicador de stream → baixa para GPU instancing (M5 escala 100k @ 60Hz).

---

## 10. Mapa de crates

| Camada | Crate | Estado |
|--------|-------|--------|
| **Substrato** (dados/portas/efeitos/formato/registry) | `ph2d-nodegraph` 🆕 | foundational, estável |
| **Compute compartilhado** (Fields + texto → WGSL\|Luau) | `ph2d-expr` 🆕 | foundational |
| **Avaliadores plurais** | `ph2d-eval-shader` (→WGSL via naga/`ph2d-gpu`), `ph2d-eval-motion` (→`ph2d-render`/`ph2d-vector`), `ph2d-eval-audio` (sync-dataflow→`ph2d-audio`), gameplay = `ph2d-script` existente | 🆕 (gameplay ✅) |
| **Nós** (caixa-preta FBP, codegen-wired) | `ph2d-node-<domínio>-<slug>` + `ph2d-node-registry-init` (codegen) | 🆕 |
| **Autoria gameplay** | `ph2d-blocks` (blocos→IR→Luau), node-programming (grafo) | 🆕 |
| **Colisão gameplay** | `ph2d-collision2d` 🆕 | 🆕 |
| **Views/editores** | `ph2d-editor*` (4 zonas, paleta-por-editor) | ✅ infra existe |
| **Ferramentas imperativas** | `ph2d-tool-*` (ADR-0027) | ✅ existe |
| **Substrato de runtime já pronto** | `ph2d-ecs` (ADR-0025), `ph2d-script` (~80%), `ph2d-save`, `ph2d-render`/`-vector`/`-audio` | ✅/⏳ |

---

## 11. Divergências vs Hard Rules / correções ao protótipo

1. **Membrana = tipo, não convenção** (sistema de efeitos checado em compile-time + arch-gate).
2. **`Entity::to_bits` NÃO é wire-format** (ADR-0037): id de entidade próprio, estável, versionado no `SceneDoc` (HR-14). Mudar no protótipo.
3. **`SceneDoc` é postcard + tabela `nome→stableTypeId`** (`blake3(nome)[..8]`, HR-6), não JSON. Mudar no protótipo p/ cook bater byte-a-byte.
4. **HR-3 no avaliador:** pools pré-alocados + `bumpalo` reset por frame; sem `Vec::push` realocante.
5. **HR-4 budget:** apresentação viva no sub-budget de render; subgrafo que estoura → cook obrigatório (budget-aware graph).
6. **HR-12/HR-15:** editores de nós/blocos populam AccessKit + Fluent; nomenclatura artista-primeiro.
7. **Formato de grafo textual** (IDs estáveis, layout segregado) — sem isso não há diff/merge entre agentes nem migração de save.

---

## 12. O que INVENTAR (nenhuma ferramenta de referência faz)

1. **Membrana de determinismo codificada no sistema de tipos do grafo** — um nó de apresentação simplesmente *não consegue* conectar de volta na simulação (erro de tipo, não convenção).
2. **Replay determinístico cross-platform do grafo de gameplay** (apresentação isenta).
3. **Save versionado + migração de grafos** (jogos não podem quebrar entre versões; Houdini/Blender quebram).
4. **Budget-aware graph** — degrada/recusa graciosamente em vez de dropar frame.
5. **Formato de grafo textual/diffável/mergeável p/ LLMs e multi-agente** (não blob binário como `.hip`/`.blend`).

Esses cinco são onde a PH2D pode ser **melhor que todas as ferramentas de referência**, porque nenhuma sequer tenta.

---

## 13. ADRs novos a criar

| ADR | Título | Núcleo |
|-----|--------|--------|
| **0030** | Multi-domain node engine — substrato único + avaliadores plurais + fronteira/regra de decisão + membrana como sistema de efeitos checado por tipo | a decisão-mãe |
| **0031** | Nó **e** ferramenta como unidade de feature (crate + contrato FBP + codegen); isolamento FBP = unidade multi-agente | estende ADR-0027 às duas famílias |
| **0032** | `ph2d-nodegraph` — modelo de atributos, portas algébricas (domínio+dim+clock), efeitos, `pre`/aciclicidade, formato textual, registry | substrato |
| **0033** | `ph2d-expr` — compute compartilhado (Fields + escape textual) → WGSL\|Luau; nó = spec + N lowerings | o VEX/VOP |
| **0034** | Avaliadores plurais por modelo-de-avaliação (pull→WGSL / sync-dataflow / pull-playhead / push→Luau) | runtime plural |
| **0035** | Cook vs live por-subgrafo + stream-de-atributos ≠ ECS + cloner; formato do bundle cozido | Rota A/B híbrida |
| **0036** | Autoria gameplay: blocos **e** node-programming → Luau (authoring-time); `ph2d-collision2d`; rapier reservado p/ física | duas superfícies, uma IR |
| **0037** | Id de entidade estável no `SceneDoc` (postcard), desacoplado do `to_bits` | save portável (HR-14) |
| **0038** | UX baseline de nós artista-primeiro (esconder contexto, viewport-first, live-preview, presets-front-door, portas que recusam, bridge bidirecional) | estende ADR-0023 |

---

## 14. Ordem de implementação + riscos

**Pré-requisito:** os três gargalos (doc irmão). Sem contrato fino + codegen + build-por-slot, node/tool-as-crate não é paralelo.

1. **Substrato de save:** fechar `ph2d-save` (`SceneDoc` postcard + id estável, ADR-0037). Desbloqueia cook + formato textual.
2. **Substrato de nó:** `ph2d-nodegraph` (modelo de dados + portas algébricas + sistema de efeitos + `pre`/aciclicidade + formato textual + registry) + `ph2d-node-registry-init` codegen'd. **Foundational, Coordenador-only, estável.** Fundação de todo paralelismo seguinte.
3. **Primeiro domínio vertical — Motion:** avaliador pull-playhead + ~3 nós (generator/cloner/modifier) + lowering p/ sprite instancing/Vello. Prova o modelo. **A partir daqui, agentes adicionam nós em paralelo total.**
4. **Compute compartilhado:** `ph2d-expr` (Fields + texto → WGSL/Luau) — desbloqueia shader e value-plugs com uma base só.
5. **Domínios restantes** (cada um = vertical paralelizável): **Shader** (→WGSL), **Gameplay** (blocos + node-programming → `ph2d-script` existente + `ph2d-collision2d` + FSM), **Sound** (sync-dataflow → `ph2d-audio`).
6. **Cook:** particionador estático→asset por domínio (ADR-0035).
7. **Enforcement:** arch-gate da membrana (efeito `Stateful` não no lado pull).
8. **UX:** os 7 princípios como gates (ADR-0038); ferramentas imperativas seguem ADR-0027 em paralelo.

**Riscos:**
- **HR-4/HR-3 no avaliador vivo** (maior): cook agressivo, pools, rebaixamento p/ cook obrigatório.
- **Over-engineering do avaliador** — runtime completo quando 80% deveria cozinhar. Cook-first.
- **Substrato `ph2d-nodegraph` instável** — se mudar a cada nó, vira god-crate e mata paralelismo. Capar superfície; mudança de substrato = evento raro Coordenador-only.
- **`ph2d-expr` virar segunda linguagem** — manter mínima; Luau continua a de gameplay.
- **Grafo virar linguagem de fluxo-de-controle** (a falha-mãe): arestas são **só dados**; controle vive em nó-texto. **Nunca adicionar "exec pins".**
- **Custo de cache não-limitado** — política de cache explícita por nó + orçamento, não cache implícito universal.

---

## 15. Veredito

**O MiniCavalry é boa base; a expansão para N domínios é o passo certo; e Houdini é referência de poder, não de design final.** A arquitetura é uma **síntese** — modelo de dados do Houdini + Fields do Blender + compile-to-shader/MetaSounds do Unreal + UX do Substance/TouchDesigner — amarrada por **duas invenções próprias que nenhuma ferramenta tem: a membrana de determinismo como tipo, e o formato de grafo textual para multi-agente.**

A percepção que une tudo: **isolamento FBP (portas tipadas + classe de efeito + zero estado compartilhado) é ao mesmo tempo o nó teoricamente ótimo e a unidade que um agente paralelo constrói sozinho.** Construir o **substrato (três gargalos)** + o **`ph2d-nodegraph` unificado** + o **`ph2d-expr` compartilhado** primeiro destrava **todos os domínios e todas as ferramentas** em paralelo. A engine cresce, daí em diante, por adição de crate isolado — que é, literalmente, o "desimpedimento total para múltiplos agentes" da primeira pergunta.
```
