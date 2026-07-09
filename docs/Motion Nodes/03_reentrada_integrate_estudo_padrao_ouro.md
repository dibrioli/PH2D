# 03 — Por que o Integrate tem reentrada? Estudo e padrão-ouro

**Data:** 2026-07-09 · **Linha:** `line/MotionNodes` · **Gatilho:** pergunta do Enio sobre a foto do
grafo da fonte — *"essa reentrada é justamente uma das coisas que o design de nós evita (ou mesmo
proíbe) para que o sistema seja intuitivo"*.

**Resposta curta:** a reentrada visível **não é exigida nem pela física nem pelo substrato** — é o
custo de autoria da topologia que o M2 escolheu. O substrato manda o feedback ser `pre`
(ADR-0032 §2.4: *"feedback temporal é `pre`, nunca aresta-de-volta"*) e a física manda
acumular-então-integrar; mas **nenhuma referência** — nem os nossos docs de nós, nem o
MiniCavalryV2, nem Houdini/Blender/Cavalry — faz o **artista** desenhar o laço. O laço é verdade
de máquina, não gesto de usuário. A proposta (§5) o mantém no substrato e o tira da autoria.

---

## §1 Cadeia causal: por que o laço existe hoje

1. **ADR-0032:** o lado pull **não tem estado oculto por nó** — de propósito. A memoização
   (`Fingerprint{input_revs, playhead, tick, params}`), o replay bit-exato e o scrub dependem de
   o cook ser função dos inputs. `Effect::Stateful` é o lado push/gameplay e é **proibido** de
   alimentar o pull (`effect.rs`: `Stateful.can_feed(Pure) == false`). Logo, estado de simulação
   só viaja **no próprio stream**, e ciclo só fecha com a aresta atrasada `pre` (à la Lustre).
2. **Doc 02 §1:** composição correta de forças = todas leem o MESMO estado, somam numa coluna
   `accel`, e UM integrador aplica (semântica Houdini POP). Rejeitamos o modelo do MVP
   (N integradores independentes, um por força) por ser fisicamente errado e não-replayável.
3. **Consequência topológica:** as forças precisam ler o estado **vivo** (drag lê `vel` simulada;
   attractor lê `P` simulada) → o estado precisa passar POR elas → nasce o loop de M2:
   `integrate.out --pre--> força₁ → … → forçaₙ --fwd--> integrate.state`.
4. **O sintoma da foto:** a aresta `pre` (Integrate→Wind) é a fina, discreta. O **laço branco**
   é a aresta de RETORNO `drag.out → integrate.state` — uma aresta forward comum que o usuário
   precisa desenhar, e que atravessa o canvas inteiro (branca porque estava em hover;
   `paint.rs:264` pinta hover em `Text1`).

O passo 1 e o passo 2 são inegociáveis (determinismo + física). O passo 3 escolheu **onde** o
estado entra na cadeia — e o passo 4 é só a superfície de autoria que essa escolha produziu.
É a superfície que está errada, não o substrato.

## §2 O que as referências fazem (3 colunas)

### Coluna A — Nossos docs de referência (`/home/enio/Documentos/Recursos/Nodes`)

| Fonte | O que diz |
|---|---|
| `DESIGN_NODE_GRAPH_PH2D.md` | DAG por construção em todo o doc; **P4**: controle de fluxo por **zonas com brackets** (padrão Blender), nunca fios especiais; §9.2 "Recursão proibida. Compiler detecta ciclos e rejeita"; silhuetas Source (trapézio-baixo) / Sink (trapézio-cima) para marcar onde dado nasce/morre. |
| `DOCS/_NODES.md` (catálogo Physics) | Campos de força são **modifiers em cadeia linear** `in→out`: *"Em geral, ordem importa: campos aplicam FORÇA (drag depois pra estabilizar). PinConstraint sobrescreve — coloque por último"*; dragField: *"Coloque DEPOIS dos campos que aceleram"*; attractorField *"combina bem com DragField (estabilizar)"*. `particleEmitter` é source com **gravidade embutida**. Simuladores (verletRope, softBody, boids) são **nós autocontidos**. **Zero menção a fio de retorno** em 1064 linhas de catálogo. |
| `ARQUITETURA_DOIS_MUNDOS.md` §6 | Pull evaluator = *"pull, puro, DAG, cache/frame"*; *"Aciclicidade … sem arestas de volta"*. |

### Coluna B — MiniCavalryV2 (implementação, clean-room: só comportamento)

- **`_forceProcess`** (`src/core/helpers.js:1156`): cada nó de força tem **estado oculto por-nó**
  (`node._state[pid] = {vx, vy, dx, dy}`), dt de wall-clock clampado a 0.05, e `time < lastTime`
  → wipe (scrub reseta a sim). GC dos pids não vistos no frame.
- Cada força **integra a própria contribuição** e emite `x = in.x + s.dx`. Partículas em chain
  usam acumulador amortecido (`exp(-dt/0.6)`, deslocamento terminal `f·τ²/m` — vento não produz
  balística, produz "swing com teto"); livres usam Euler clássico. Drag aproxima a velocidade
  combinada lendo `(inst.vx||0) + s.vx` — reconhecidamente um hack de composição.
- **O grafo do usuário é linear**: `emitter → attractor → drag → render`. O evaluator nem rejeita
  ciclo — um `visited` set corta a recursão silenciosamente (`pull-evaluator.js:27`); o catálogo
  simplesmente **nunca pede** um ciclo.
- Veredito do doc 02 mantido: a **UX linear é a lição certa**; o mecanismo (estado oculto,
  wall-clock, N integradores independentes) é o que rejeitamos por replay/scrub/física.

### Coluna C — Indústria (pesquisa web 2026-07-09, fontes primárias; URLs no rodapé)

| Ferramenta | Ciclo visível? | Onde vive o feedback | O que o artista liga (partículas + forças) |
|---|---|---|---|
| **Houdini** (POP) | Não (DAG top-down) | No **POP Solver**: microsolvers de força "accumulate forces" no atributo `force` do objeto persistente; o solver "updates particles for one timestep". No **Solver SOP**, o interior do subnet recebe o estado anterior como um **nó-fonte chamado `Prev_Frame`** | Cadeia linear `POP Source → POP Force → POP Wind → POP Drag →` inputs verdes do solver; "there can only be one, solid wire from every generator node to the POP Solver" |
| **Blender** Geometry Nodes 3.6+ | Não — **rejeitado por design** | **Simulation Zone**: par bracket `Simulation Input`/`Output`, pareados por id **invisível** ("internally linked by their `bNode.identifier`"); o que entra no Output reaparece no Input no frame seguinte. Racional: "not possible to have any link going towards outside… necessary to make caching and sub-frame interpolation work" | Nós comuns DENTRO da zona (integração manual: `Set Position` + `velocity × Delta Time` como state item) |
| **Cavalry** | Não (docs nem discutem ciclo; "an attribute… can only have one input") | Dentro do **Particle Shape** (o nó dono da sim): `Start Frame`, `Seed`, cache `.sdcache` + `Cache Offset` p/ scrub/retime | Emitters no array **`Emitters`**; forças no array **`Modifiers`** ("stacked where each Modifier adds to the one below it": Force, Flow Field, Turbulence, Vortex…); Gravity/Drag básicos são atributos do próprio Particle Shape |
| **Bifrost** | Não | **Feedback ports**: PAR de portas input/output **marcadas** na borda de um compound — "the output value is used as the input the next time the compound is evaluated". Nunca um laço desenhado; artistas consomem solvers já empacotados | `source_particles → simulate_particles`; forças = "influences" **encadeadas** no port `influences` do solver |
| **Softimage ICE** | Não | Estado = atributos do point cloud na região "Simulation" do stack ("calculations are based on the previous frame") | Forças → hub **Add Forces** ("simply added together") → port `Forces` do **Simulate Particles**, que integra (mass-aware) |
| **TouchDesigner** | Não (cook cycle = "things randomly break") | **Feedback TOP/POP referencia o alvo por parâmetro** (path), servindo o frame anterior; Feedback CHOP abençoa um laço desenhado só porque "copies its input without cooking it first" | `feedback → filtros → target` linear |
| **Unreal Niagara** | Não (nem é grafo de fios — é **stack**) | Payload por-partícula persistente (`Particles.*`); Simulation Stages p/ grids | Módulos de força (Gravity, Curl Noise, Drag, Vortex, Wind…) empilhados em ordem no **Particle Update** |
| **Notch** | Não (hierarquia) | **Particle Root** possui o pool ("positions, velocities, colours… in a pool") | Affectors pendurados no Root ou em Emitters |
| **Nuke / Fusion / Grasshopper** | Não (estrito; GH: "Recursive data stream found") | Nuke: pull-DAG + nós temporais (FrameHold/TimeEcho); GH: **Anemone** Loop Start/End (bracket) e **Kangaroo2** = solver-nó com goals declarativos + inputs Reset/Restart | — |
| **vvvv / Max / Pure Data** | **Sim — mas só através de um nó de delay explícito** | vvvv: proíbe o laço "unless a FrameDelay node is in the loop"; PD/Max: ciclo de áudio exige `send~`/`tapin~`/`feedback~` (delay de 1 bloco); gen~ exige o operador `history` (1 sample) | — |

**Síntese (convergência das duas varreduras):** no padrão-ouro, **ninguém faz o artista fechar o
laço da simulação**. O padrão dominante para partículas-com-forças é: **um nó-solver stateful com
as forças plugadas nele como cadeia/hub lateral** (inputs verdes do POP Solver, `influences` do
Bifrost, `Forces` do ICE, `Modifiers` do Cavalry, stack do Niagara). O mecanismo genérico de
frame-anterior, quando oferecido, é **feedback-por-referência ou par-de-portas marcado** (TD,
Bifrost) ou **zona bracket** (Blender, Anemone) ou **subnet com nó `Prev_Frame`** (Houdini Solver
SOP). Laço desenhado só existe no mundo áudio/patching realtime — e mesmo lá SÓ através de um nó
de delay explícito, com a latência escondida sendo a armadilha clássica de iniciante (PD: "DSP
loop detected"). O nosso `pre` é a formalização Lustre correta do que todos escondem — o vvvv
FrameDelay é literalmente o nosso `pre` —; o erro do M2 foi expô-lo como **gesto de autoria**
do caso comum, em vez de mecanismo de máquina.

Nota de scrub/caching que a pesquisa trouxe de graça: Cavalry precisa de cache em disco
(`.sdcache`) e Blender justifica o bracket pelo caching — o nosso desenho (emitter stateless +
estado no stream via `pre` + fingerprint por tick) já é **melhor posicionado** que estado-oculto
para scrub determinístico; o checkpoint M2.N2 completa isso.

## §3 Diagnóstico

| Camada | Veredito |
|---|---|
| Substrato (`pre`, ADR-0032) | **CERTO.** É exatamente como synchronous dataflow formaliza o que Houdini/Blender escondem. Determinismo, replay, scrub e memoização dependem dele. Não mexer. |
| Física (accumulate-then-integrate, id-pairing) | **CERTA.** Falsificada em M2 (desligar o pairing → 0 partículas saem do bocal). Não mexer. |
| Superfície de autoria (porta `state` + fechar o retorno na mão + laço atravessando o canvas) | **ERRADA** para o caso comum. O usuário não deveria (i) ver uma porta chamada `state`, (ii) desenhar o fechamento, (iii) ler um laço cruzando o grafo. A foto é o sintoma. |

## §4 Opções estudadas

### O1 — Cadeia linear + plumbing `pre` automática e renderizada como portal (RECOMENDADA)

Grafo visível vira DAG puro:

```text
Emitter → Integrate → Tint → Output          (trunk)
⟲ Wind → Curl → Drag → Integrate.forces      (ramo de forças; ⟲ = chip de estado)
```

- A porta `state` do integrate ganha rótulo **`forces`** (semântica: "cadeia que produz accel").
- **Editor cria e mantém a aresta `pre`**: ao conectar um ramo em `forces`, o bridge acha a(s)
  cabeça(s) do ramo (input `in` solto) e liga `integrate.out --pre--> cabeça` sozinho; ao
  desconectar/deletar, remove/re-heala. O usuário **nunca** desenha o laço (o precedente já
  existe: `wire_state_self_loop` no AddNode).
- **Arestas `pre` deixam de ser splines**: viram um **par de chips/portais** (⟲ no integrate,
  chip "state" na cabeça do ramo); hover/click destaca o par e mostra "estado da sim, 1 tick
  atrás". Vale também pro self-loop do spring (hoje um espaguete em volta do próprio nó).
- **Zero mudança de substrato, cook, ops ou crates de força** — o grafo ESTOCADO mantém a mesma
  forma (mesmas arestas); muda quem as cria (bridge) e como se desenham (painel). Testes de
  determinismo/física atuais permanecem válidos.
- **Generalidade preservada:** qualquer nó de stream ainda pode sentar no ramo (um clamp no
  caminho do estado é um clamp persistente — coisa que Cavalry não faz); e o auto-`pre` no
  `WouldCycle` continua existindo como caminho expert (feedback arbitrário à la TouchDesigner,
  agora legível como portal em vez de lasso).
- **Precedentes diretos de cada peça:** forças como cadeia lateral plugada no solver = inputs
  verdes do POP Solver / `influences` do Bifrost / `Modifiers` do Cavalry (o padrão dominante
  segundo as duas varreduras); aresta de estado como par marcado em vez de fio = **feedback
  ports do Bifrost**; o `pre` em si = FrameDelay do vvvv. Se um dia quisermos uma cabeça
  explícita e ensinável pro ramo, o precedente é o nó `Prev_Frame` do Solver SOP do Houdini /
  `Simulation Input` do Blender — fica como upgrade opcional de teachability (chip primeiro:
  zero crate nova, menos fricção de autoria).

Custo: painel (chips + hit), bridge (auto-pre no connect/disconnect em `forces`), demos
relayoutados, testes novos (falsificação: quebrar o auto-pre → fonte morre). ~1 wave.

### O2 — Só cosmético: portais para `pre`, resto igual

Melhora a leitura, mas o usuário **continua** fechando o ciclo na mão (a aresta de retorno
continua sendo gesto de autoria). Resolve metade. Subconjunto de O1 — sem razão para parar aqui.

### O3 — Cavalry/ICE-style: forças viram descritores opacos plugados em `forces`

Forças deixariam de processar stream e emitiriam um `ForceSpec` (via `CookValue::Opaque`);
o integrate avaliaria os campos. Tem precedente forte (hub **Add Forces** do ICE soma vetores
"order not important"; `Modifiers` do Cavalry; goals do Kangaroo2) e dá DAG puro sem plumbing
NENHUMA — MAS fecha o vocabulário (força = campo `f(P, vel, t)`; perde-se "qualquer nó no
caminho do estado"), exige trait foundational novo + refactor das 5 crates de força, e o teto
fica abaixo de Houdini/zonas. **Rejeitada como estado final** (a ambição do DESIGN doc é
high-ceiling); fica registrada como fallback se O1 revelar custo oculto.

### O4 — Zona de Simulação (bracket Blender)

O teto mais alto: região demarcada onde TUDO roda por-tick sobre o estado vivo (permite merge de
newborns na frente das forças, kill nodes, edições arbitrárias per-tick — supera até o O1).
Alinha com o **P4 do DESIGN doc** (zonas já são o idioma de controle de fluxo escolhido) — e o
Blender confirmou a generalização de propósito: o bracket da sim virou Repeat Zone (4.0) e
For-Each Zone (4.3). Exige `cook_scoped` (o neck M2.N1, já roadmapado para o time-remap) + UI de
zona. **É o horizonte natural** quando `cook_scoped` chegar; O1 não conflita — é um degrau na
direção (o ramo de forças de O1 é o "interior da zona" de O4 com boundary implícita).

Lições de UX já documentadas pelo campo do Blender, para quando formos lá (devtalk oficial):
inputs da zona **congelam após o frame 1** ("even just a simulation zone that does nothing
freezes the mesh" — surpresa nº 1 dos usuários); atributos que não mudam **precisam tunelar**
pela zona; e a zona crua é **low-level demais** sem building blocks de alto nível ("in many
other DCCs building a particle system means manipulating velocities, positions…"). Tradução
pro PH2D: zona genérica NÃO substitui os nós densos (emitter/integrate/forças) — ela os
complementa por baixo.

## §5 Recomendação

> **STATUS: O1 IMPLEMENTADO (2026-07-09, mesma sessão).** Porta `forces` no integrate;
> plumbing `pre` engine-managed em `shells/desktop/src/render_loop/motion_bridge_plumbing.rs`
> (reconcile before/after: self-loop quando vazio, entrada de estado na cabeça do ramo,
> re-heal em disconnect/delete, badge não-deletável à mão, `pre` expert intocado);
> badges de portal em `ph2d-panel-motion-graph` (paint + hit compartilham geometria);
> demos lineares. Falsificado: sem a entrada de estado a fonte morre
> (`the_fountain_dies_without_its_state_entry`); sem o gesto do bridge o fio no `forces`
> é recusado (controle em `wiring_a_chain_into_forces_replumbs_the_state_entry`).
> Modelo canônico agora em SKILL §11.13; evidências da indústria no doc 04.

**O1 agora; O4 como evolução quando `cook_scoped` existir.** O1 entrega o padrão-ouro de
facilidade (grafo linear, igual ao catálogo de referência e a Houdini/Cavalry/Blender) sem
abrir mão de nada do que o M2 conquistou (determinismo, física composta, generalidade do ramo).
A mudança é de **convenção de autoria + rendering** — contratos congelados intocados, substrato
intocado.

Passos concretos (1 wave):
1. Painel: renderizar `Edge::delayed` como par de chips/portais (+ hit/hover/probe + a11y).
2. Bridge: rótulo `forces` na porta 1 do integrate; auto-`pre` no connect/disconnect do ramo
   (recompute das cabeças; regra: 1 integrate por ramo — segunda conexão em outro `forces`
   recusa com toast, invariante uma-aresta-por-porta já cobre).
3. Demos (`motion_state.rs` + `motion_demo_particles.rs`): re-autorar linear; posições novas.
4. Testes: (a) conectar ramo em `forces` cria o pre na cabeça certa; (b) deletar força do meio
   re-heala; (c) falsificar: sem auto-pre a fonte não voa; (d) replay determinístico inalterado.
5. Doc/handoff: atualizar 02 §3 (topologia) com ponte pra este doc.

## §6 O que este estudo NÃO muda

- `pre` no substrato (ADR-0032) — continua o único jeito legal de fechar ciclo, e continua
  disponível ao usuário expert via auto-pre no `WouldCycle`.
- Semântica de `motion.integrate`/`motion.spring`/forças — mesmas crates, mesmos testes.
- Determinismo/replay/scrub — o grafo estocado mantém a mesma topologia de arestas.
- A porta `state` como **convenção de manifest** (nós sequenciais futuros) — só o rótulo exibido
  e quem liga o fio mudam.

---

## Fontes primárias (seleção; levantadas por 2 varreduras independentes, 2026-07-09)

- Houdini: [POP Solver](https://www.sidefx.com/docs/houdini/nodes/dop/popsolver.html) ·
  [POP Force](https://www.sidefx.com/docs/houdini/nodes/dop/popforce.html) ·
  [Solver SOP (`Prev_Frame`)](https://www.sidefx.com/docs/houdini/nodes/sop/solver.html) ·
  [Streams](https://www.sidefx.com/docs/houdini/dopparticles/streams.html)
- Blender: [Simulation Zone (manual)](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/simulation/simulation_zone.html) ·
  [PR #104924 (design + racional do caching)](https://projects.blender.org/blender/blender/pulls/104924) ·
  [Geometry Nodes Workshop 2022](https://code.blender.org/2022/11/geometry-nodes-workshop-2022/) ·
  [devtalk: críticas de UX](https://devtalk.blender.org/t/simulation-nodes/29791)
- Cavalry: [Particle Shape (`Emitters`/`Modifiers`, cache)](https://cavalry.studio/docs/nodes/shapes/particle-shape/) ·
  [Particle Emitter](https://cavalry.studio/docs/nodes/utilities/particle-emitter/) ·
  [Forge Dynamics (Fields, `.sdcache`)](https://cavalry.studio/docs/nodes/shapes/forge-dynamics/forge-dynamics-shape/) ·
  [Connections (um input por atributo)](https://cavalry.studio/docs/getting-started/key-concepts/connections)
- Bifrost: [feedback ports (custom sims)](https://help.autodesk.com/cloudhelp/ENU/Bifrost-Common/files/build-a-graph/Bifrost_Common_build_a_graph_custom_sims_html.html) ·
  [particle sim setup](https://help.autodesk.com/cloudhelp/2024/ENU/Bifrost-Common/files/simulate-dynamic-effects/create-particle-simulations/Bifrost_Common_simulate_dynamic_effects_create_particle_simulations_set_up_a_particle_simulation_html.html) ·
  [wind_influence](https://help.autodesk.com/cloudhelp/2019/ENU/Bifrost-Common/files/reference/geometries/Bifrost_Common_reference_geometries_wind_influence_html.html)
- ICE: [Forces (Add Forces hub)](https://download.autodesk.com/global/docs/softimage2014/en_us/userguide/files/ice_simulation_ICEForces.htm) ·
  [ICE framework (região Simulation)](https://download.autodesk.com/global/docs/softimage2014/en_us/userguide/files/ICE_basics_UnderstandingtheICEFramework.htm)
- TouchDesigner: [Feedback TOP](https://docs.derivative.ca/Feedback_TOP) ·
  [Feedback CHOP](https://docs.derivative.ca/Feedback_CHOP) ·
  [Feedback POP](https://docs.derivative.ca/Feedback_POP) ·
  [forum: cook dependency loop](https://forum.derivative.ca/t/cook-dependancy-loop-and-creating-feedback/186746)
- Niagara: [key concepts (stack)](https://dev.epicgames.com/documentation/en-us/unreal-engine/key-concepts-in-niagara-effects-for-unreal-engine) ·
  [Particle Update (forças stock)](https://dev.epicgames.com/documentation/en-us/unreal-engine/particle-update-group-reference-for-niagara-effects-in-unreal-engine)
- Grasshopper: [Kangaroo2 Solver](https://grasshopperdocs.com/components/kangaroo2/solver.html) ·
  [Anemone](https://www.food4rhino.com/en/app/anemone) · fóruns "Recursive data stream found"
- vvvv: [Creating Feedback Loops (FrameDelay)](https://beta.vvvv.org/using-vvvv/patching/creating-feedback-loops.html) ·
  Pure Data: [FAQ DSP loop](https://puredata.info/docs/faq/dsp-loop) ·
  Max/RNBO: [feedback~ (1 vetor de delay)](https://rnbo.cycling74.com/learn/delays-in-rnbo) ·
  Notch: [Particles (Root possui o pool)](https://manual.notch.one/2026.1/en/docs/reference/nodes/particles/)
- Nuke: [NDK: pull-DAG](https://learn.foundry.com/nuke/developers/63/ndkdevguide/intro/oparchitecture.html) ·
  [FrameHold](https://learn.foundry.com/nuke/content/reference_guide/time_nodes/framehold.html)
