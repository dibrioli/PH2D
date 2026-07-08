# Motion Nodes — Estudo de estado da arte e proposta para a PH2D

**Data:** 2026-07-06 · **Autor:** pesquisa multi-agente (deep-research verificado adversarialmente + verificação dedicada Rive/licenças + mapa do node system interno) · **Status:** estudo para avaliação do Enio — nada implementado.

> **Pergunta:** qual o melhor sistema de motion nodes (estilo Cavalry / Blender Geometry Nodes) para
> a PH2D — extremo poder e performance, fácil para artistas — e o que aproveitar de projetos abertos
> (incluindo Rive)?
>
> **Resposta em uma frase:** a PH2D **já tem o substrato certo e congelado** (cook pull memoizado com
> playhead de 1ª classe, `Effect::Temporal`, feedback `pre` de 1 tick — ADR-0030..0039); o que falta
> não é motor, é **(1) o fio do tempo no shell, (2) keyframes/timeline reais atrás do
> `AttributeEvaluator::sample(t)` que já existe, (3) a camada de UX Cavalry-style (behaviours +
> falloffs) e (4) state machine como nó** — e o estado da arte verificado confirma exatamente essas
> quatro camadas como o padrão vencedor.

---

## 1. O que a PH2D já tem (mapa verificado no repo)

A fundação está mais pronta do que o plano de waves sugere — a vertical Motion foi provada end-to-end
(smoke de 27 sprites confirmado) e o contrato está congelado:

| Peça | Onde | Estado |
|---|---|---|
| Cook **pull, demand-driven, memoizado** | `ph2d-nodegraph/src/cook.rs` | ✅ Fingerprint por nó = revs das entradas + playhead (só se `Temporal`) + tick (só se consome `pre`) + params. Reusa cache se bate |
| **Playhead de 1ª classe** | `EvalCtx::playhead() -> f64` | ✅ no contrato congelado — mas **ninguém o alimenta** (sempre `0.0` fora de teste) |
| **Feedback temporal** (`pre` 1-tick, à la Lustre) | `Edge.delayed` + `advance_tick` | ✅ — é o que springs/inércia/simulação precisam |
| Membrana de efeitos | `Effect::{Pure, Temporal, Stateful}` + `can_feed` | ✅ Stateful nunca alimenta pull-side; presentation isenta de HR-5 |
| Portas algébricas | `PortType { Domain, Dim, Clock }` | ✅ `Clock::Frame` e `Domain::{Instances, Field, Signal}` já existem |
| Stream colunar SoA | `Stream`/`Column` (convenção `P/size/rot/tint`) | ✅ é o "attribute stream" à la Houdini/Blender |
| Expressões (o "VEX") | `ph2d-expr`: `Sin/Cos/Mix/Noise/Select…`, lowering CPU+WGSL | ✅ com `Func::is_deterministic()` separando transcendentais (HR-5) |
| Nós motion | `motion.grid`, `motion.transform`, `motion.clone` | ✅ provados por `ph2d-eval-motion::lower_to_instances` |
| Amostragem no tempo | `AnimValue` (6 variantes, OKLCH lerp) + `AttributeEvaluator::sample(t: f64)` + `AnimationCurveSampler::at(t)` | 🟡 **só o vocabulário** — impls reais são mocks; sem keyframe store, sem timeline |
| Clock determinístico | `ph2d-core::FixedStep` (60 Hz, tick_count u64) | ✅ mas desconectado do cook |
| UI de grafo | — | ❌ inexistente (só painéis de slider); toda a UX do ADR-0038 é greenfield |
| Fan-out | 1 crate `ph2d-node-*` + `cargo run -p ph2d-node-sync` | ✅ ABERTO — nó novo não toca nada central |

**Lacunas exatas:** fio do tempo no shell · keyframe/timeline storage · UI de grafo+timeline ·
`ParamSpec` só `f32` (animar Vec2/cor via param esbarra no freeze — ADR).

---

## 2. Como os melhores funcionam por dentro (verificado)

### 2.1 Blender Geometry Nodes — *o* modelo de avaliação (GPL: só estudar)

- **Lazy pull-based desde 2022** (commit de Jacques Lucke, verificado verbatim): cada node group é
  **compilado num lazy-function graph antes de avaliar**; só computa o que é demandado — ramos atrás
  de um Switch desligado **nunca rodam**. Grafos compõem (um lazy-graph é ele próprio uma
  lazy-function; grupos não são mais inlined).
- **Fields**: um Field é uma **função diferida e imutável** cujo valor só resolve contra um
  `FieldContext` (ex.: a geometria com o atributo `position`) na hora da avaliação. Composição gera
  grafos novos, nunca muta. É o modelo canônico de "valor por-elemento" — o análogo direto do
  `ph2d-expr` avaliado por coluna do `Stream`.
- **Depsgraph** (dirty propagation temporal): só reavalia o que depende do valor modificado; é o
  responsável por f-curves (animação), mas **não** por edições destrutivas one-shot — separação limpa
  entre avaliação procedural/temporal e operação destrutiva. Tudo avalia sobre **cópias
  copy-on-eval** (o original nunca é mutado) → mesma cena em N estados simultâneos, threading
  render/viewport resolvido.

**Leitura para a PH2D:** o Cook já é pull+memo (o mesmo regime). O que o Blender adiciona de valioso
é (a) a *disciplina* animação-como-camada-de-avaliação-sobre-dados-intocados — que a PH2D já
respeita via `Arc` snapshot no `CookValue` — e (b) a ideia de **compilar o grafo num plano de
avaliação** quando ele crescer (hoje o cook interpreta; um "plano pré-alocado por frame" é a resposta
natural ao HR-3 se o interpretador começar a alocar).

### 2.2 Cavalry — *a* camada de UX (proprietário: só comportamento)

- **Behaviours**: nós tipo-efeito **anexados a Layers** ("Behaviours can be thought of as effects…
  used to animate or deform other Layers", docs oficiais). O artista **não abre um grafo**: arrasta
  um behaviour na camada. O dataflow existe por baixo (attribute-driven via Connections), mas a
  superfície é "modificador na camada", como C4D.
- **Falloffs**: campos de influência espacial **empilháveis como camadas com blend modes**
  (Normal/Add, Min, Max, Minus, Multiply, Screen, Overlay), avaliados bottom-up com ordem
  significativa — o análogo artist-friendly dos fields.

**Leitura para a PH2D:** é a resposta ao ADR-0038 (viewport-first / zero jargão / graph-second). A
UX vencedora de motion 2D **não é** expor o grafo cru: é *behaviour-na-camada* + *falloffs
empilháveis*, com o grafo como representação subjacente (e acessível a quem quiser).

### 2.3 Rive — a lição de runtime de produto (MIT: pode ler e derivar)

Ver §4 — análise completa.

### 2.4 Godot (MIT — pode ler E copiar com atribuição)

`AnimationTree` (blend tree + state machine com travel/transições) e `Tween` (sequência+paralelo
sobre property paths) são a **mina de ouro em código ativo e licença compatível**: os padrões de
blend por caminho, travel entre estados e property-path tweening são diretamente transplantáveis
(atribuição MIT no NOTICE). Alvos: `scene/animation/animation_tree.cpp`,
`animation_node_state_machine.cpp`, `scene/animation/tween.cpp`.

### 2.5 Síntese do padrão vencedor

Todos os sistemas maduros convergem em **quatro camadas separadas**:

1. **Avaliador** pull/lazy com cache seletivo (Blender lazy-functions ≈ Cook da PH2D ✅);
2. **Dados por-elemento** como funções diferidas sobre um contexto (Fields ≈ `ph2d-expr` sobre `Stream` ✅);
3. **Tempo** como camada de amostragem sobre dados imutáveis (depsgraph/f-curves ≈ `AttributeEvaluator::sample(t)` 🟡 sem storage);
4. **UX de artista** que esconde o grafo: behaviours/falloffs (Cavalry) + state machines sobre timelines (Rive/Godot) ❌ a construir.

---

## 3. Tabela de projetos avaliáveis (licenças verificadas nos repos, 2026-07-06)

### Usáveis como código (MIT/Apache/BSD — dependência ou cópia com atribuição)

| Projeto | Link | Licença | Estado | O que aproveitar |
|---|---|---|---|---|
| **Graphite / Graphene** | [github.com/GraphiteEditor/Graphite](https://github.com/GraphiteEditor/Graphite) | Apache-2.0 | ativo (alpha) | **A referência Rust mais próxima**: documento-É-grafo, cache por ramo (`MemoNode`), ~88% Rust. Motion graphics ainda é roadmap (keyframes "early 2026") — **estudar arquitetura, não copiar features de motion (não existem)** |
| **Godot** | [github.com/godotengine/godot](https://github.com/godotengine/godot) | **MIT** | ativíssimo | `AnimationTree`/state machine/`Tween` — copiar padrões (e código, com atribuição) de blend tree, travel, property tweening |
| **bevy_animation_graph** | [github.com/mbrea-c/bevy_animation_graph](https://github.com/mbrea-c/bevy_animation_graph) | MIT/Apache-2.0 | ativo (v0.10, fev/2026) | **State machine embutida COMO NÓ no grafo de animação**; cada estado toca seu próprio grafo; transições têm grafo próprio. ⚠️ compat com Bevy 0.18 NÃO confirmada — fonte de padrões, não dep plug-and-play |
| **Rive runtimes** | [github.com/rive-app/rive-runtime](https://github.com/rive-app/rive-runtime) | **MIT** | C++ ativo; rive-rs parado | Ver §4 — modelo (state machine sobre timelines, data binding) + parser .riv de referência em Rust puro |
| **OpenTimelineIO** | [github.com/AcademySoftwareFoundation/OpenTimelineIO](https://github.com/AcademySoftwareFoundation/OpenTimelineIO) | Apache-2.0 | ativo (ASWF) | **Modelo de dados de timeline** da indústria: Timeline→Track→Clip/Gap/Transition + **`RationalTime` (tempo racional exato, sem drift de float — resposta direta ao HR-5)** |
| **Motion Canvas** | [github.com/motion-canvas/motion-canvas](https://github.com/motion-canvas/motion-canvas) | MIT | ativo | Animação procedural **code-first por generators** (`yield*` = passo de tempo) + signals reativos + composição `all/chain/sequence` — modelo para a face Luau/scripting do motion |
| **theatre.js** | [github.com/theatre-js/theatre](https://github.com/theatre-js/theatre) | Apache-2.0 | **dormente** (2024) | Melhor referência aberta de **UI de timeline/keyframe editor** (dope sheet, curvas gráficas, modelo sheet/object/prop) — copiar o design da UI |
| **egui-snarl** | [github.com/zakarumych/egui-snarl](https://github.com/zakarumych/egui-snarl) | MIT/Apache-2.0 | ativo (v0.11, jun/2026) | Padrões de node-graph UI em Rust (pins, wires, multiconn). ⚠️ nossa UI é Vello retained-mode, não egui — estudo de interação, não dep. (`egui_node_graph` está MORTO: repo deletado, versões yanked) |
| **OpenFX** | [github.com/AcademySoftwareFoundation/openfx](https://github.com/AcademySoftwareFoundation/openfx) | BSD-3 | ativo (ASWF) | Modelo de **parâmetros animáveis** de efeitos (property sets). Plugin-ABI em si é contraexemplo (ADR-0075 rejeitou plugin runtime) |
| **Cables.gl** | [github.com/cables-gl](https://github.com/cables-gl) | MIT (⚠️ repo core sem LICENSE na raiz — conferir antes de copiar) | ativo | UX de grafo (trigger-flow vs dataflow, subpatches) |
| **Enso** | [github.com/enso-org/enso](https://github.com/enso-org/enso) | Apache-2.0 | pivotou p/ data-prep | Só a UI de grafo (visualização inline por nó, texto↔visual sincronizado) |
| Crates: `simple_easing` · `bevy_tweening` · `bevy_easings` | crates.io | MIT/Apache-2.0 | mantidos | Funções de easing prontas; padrão **Lens** (bevy_tweening) para tween-de-componente em ECS |
| Crates: `keyframe` · `interpolation` | crates.io | MIT | **órfãos** (2022/2023) | Pequenos e completos — candidatos a cópia interna (625 LOC o `interpolation`), não a dependência |

### Só estudo clean-room (GPL) ou evitar

| Projeto | Licença | Nota |
|---|---|---|
| **Blender** (geometry nodes, depsgraph, f-curves) | GPL-2.0-or-later (headers SPDX) / GPLv3 (whole) | Clean-room only — mesmo regime já aplicado ao `reference/blender-texture-paint/`. É a **melhor documentação de arquitetura** (docs developer.blender.org são abertas e citáveis) |
| **Natron** | GPL-2.0, semi-dormente (release 2022) | Estudo: avaliação lazy por região de interesse + cache por hash de params |
| **vvvv gamma** | editor proprietário; VL.StandardLibs LGPL-3.0 | Evitar (LGPL contamina cópia; o interessante está no editor fechado) |
| **Houdini / TouchDesigner / After Effects / Cavalry** | proprietários | Só comportamento via docs públicas. Cavalry docs ([docs.cavalry.scenegroup.co](https://docs.cavalry.scenegroup.co/nodes/behaviours/)) são explícitas sobre o modelo behaviours/falloffs |

---

## 4. Rive — análise completa (verificado nos repos e docs oficiais)

**Licenciamento e negócio:** todos os runtimes são **MIT** (verificado nos LICENSE de
rive-runtime/rive-rs/rive-bevy/rive-wasm/rive-flutter/rive-react-native/rive-unity). O **editor é
proprietário/SaaS** (editor.rive.app, "Free to create / $9/mo to ship" — exportar .riv para produção
é pago). Modelo: formato aberto + runtimes MIT, autoria fechada e monetizada.

**Estado do caminho Rust: ruim.** `rive-rs` está **~1 ano sem commits** (jul/2025), sem release,
**fora do crates.io**, com issues abertas sem resposta ("crasha na maioria dos .riv de exemplo",
strokes errados, clips errados no backend Vello); a integração do Rive Renderer prometida nunca
landou. `rive-bevy` idem (~11 meses). O investimento real da Rive é no core C++ e nos bindings
mainstream (web/Flutter/Unity/mobile; a integração da **Defold** usa o runtime C++ com renderer
nativo e vive na org da Defold Foundation).

**Formato .riv:** spec binária pública oficial
([rive.app/docs/…/format](https://rive.app/docs/runtimes/advanced-topic/format)) — header `RIVE` +
ToC auto-descritivo (parser pula propriedades desconhecidas), varuint LEB128, objetos sequenciais.
Um **importador independente é viável** (a spec + o parser .riv MIT do rive-rs como referência) —
candidato natural a importador do asset pipeline (§11.10 da SKILL), não a runtime.

**Modelo técnico (o que vale como referência):**
- **States = timelines** ("States are simply timeline animations that can play in your state
  machine"): a state machine é uma camada sobre clips, não um substituto. Múltiplas **layers**
  paralelas por machine. **Blend states 1D** (um number faz crossfade entre N timelines) e **Direct
  Blend** (multi-dimensional).
- **Inputs (number/bool/trigger) foram DEPRECADOS** em favor de **Data Binding** (2024+): *View
  Model* = schema tipado (number, bool, string, enum, color, trigger), *View Model Instance* =
  valores vivos, bindings bidirecionais **desacoplados da hierarquia da cena** ("restructure your
  scene without breaking runtime connections"), com **converters** no caminho.
- **Constraints:** IK, Distance, Transform, Translation, Scale, Rotation.
- **Rive Scripting roda em Luau** (confirmado, blog oficial nov/2025): mesma VM no editor e em todos
  os runtimes, scripts organizados em *Protocols* (Converter, Node, Layout, PathEffect, Test).

**Veredito para a PH2D:**
1. **Como dependência: não.** O caminho Rust está estagnado; integrar o core C++ traria FFI + um
   renderer paralelo ao nosso — contra o norte drop-crate/ECS (ADR-0075).
2. **Como modelo: sim, e muito.** (a) *State machine como camada sobre timelines* é exatamente como
   um nó `motion.state-machine` deve consumir clips; (b) a **trajetória inputs→data-binding é a
   lição mais valiosa** — nascer direto no modelo "contrato tipado que o runtime lê/escreve" (na
   PH2D: params/portas do grafo expostos via `#[lua_export]`/MCP, HR-10) pula uma geração de design;
   (c) **trigger como tipo de 1ª classe** (true-por-1-frame) mapeia para `Clock::Event`; (d) a
   escolha de **Luau valida a nossa** (ADR-0019) — inclusive o padrão "protocols por papel" para
   scripts de converter/efeito.
3. **Como código-fonte de leitura: sim** — MIT permite ler e derivar (state machine, data binding e
   parser .riv do rive-runtime/rive-rs), com atribuição.
4. **Importador .riv:** viável e desejável como formato de entrada do asset pipeline (tabela §11.10
   da SKILL ganharia a linha `RIV → skeletal/state-machine animation`), num crate isolado, quando
   houver demanda.

---

## 5. Proposta arquitetural para a PH2D

### Princípio-guia

**Não construir um "sistema de motion" paralelo** — o motor já existe e está congelado. Motion nodes
= **preencher as 4 lacunas em camadas independentes**, cada uma no regime de mudança correto
(fan-out puro vs ADR/Coordenador), mantendo a hierarquia §18 (performance no hot path > tudo;
determinismo onde prometido).

### Camada 0 — O fio do tempo (pré-requisito de tudo; pequeno e cirúrgico)

O único `playhead: f64` do sistema já é o parâmetro do `Cook::cook()` — falta alimentá-lo:

- **Resource `Playhead`** (novo, foundational leve): `{ time: f64, tick: u64, playing: bool, rate: f64 }`,
  derivado do `FixedStep` existente (tick determinístico → `time = tick as f64 * fixed_dt`), com
  transporte (play/pause/scrub/loop) para o editor.
- **Render bridge**: 1×/frame, `advance_tick(graph, ops, playhead)` + cook dos sinks → upload de
  instâncias (o caminho `ph2d-eval-motion::lower_to_instances` já existe; hoje só roda atrás de
  `PH2D_MOTION_SMOKE=1`). Promover o smoke a caminho de produção no shell.
- **Determinismo (HR-5):** playhead **derivado do tick inteiro** (nunca de wall-clock acumulado em
  float). Para exports/replay, adotar o padrão **`RationalTime` do OpenTimelineIO** (numerador/
  denominador inteiros) como representação canônica de tempo em storage; `f64` só na borda do
  `sample(t)`. Presentation continua isenta de HR-5 (membrana ADR-0030), mas gameplay-motion não.
- **Memoização já resolve o custo:** com o fingerprint atual, subgrafos `Pure` **não recozinham**
  quando só o playhead anda; só a cadeia `Temporal`+dependentes reavalia. É o depsgraph do Blender
  de graça.

### Camada 1 — Keyframes e timeline (dar corpo ao vocabulário que já existe)

- **Crate nova `ph2d-anim`** (drop-crate, satélite): keyframe store + curvas reais implementando os
  traits **já congelados no Vector Module** — `AttributeEvaluator::sample(t) -> AnimValue` e
  `AnimationCurveSampler::at(t)`. Conteúdo: `Track { target, keys: Vec<Key { t: RationalTime, value:
  AnimValue, interp }> }`, interpolação Bézier por-segmento (Hermite/Bézier avaliada por subdivisão —
  transcendental-free, HR-5-friendly), easing presets (copiar de `simple_easing`/`interpolation`,
  MIT), `Clip` = coleção de tracks com duração.
  - HR-3: `sample(t)` com busca binária em `Vec` ordenado, zero alloc; hot path pré-resolve o índice
    do segmento (cursor por track, monotônico durante playback).
  - Referências de código: Godot `Tween`/`Animation` (MIT), theatre.js (modelo sheet/object/prop),
    OTIO (Timeline→Track→Clip).
- **Ponte com o grafo — nó `motion.clip`** (fan-out puro, `Effect::Temporal`, `Clock::Frame`): amostra
  um `Clip` no playhead e emite colunas no `Stream` (ou um `CookValue::Opaque(Arc<Clip>)` para nós
  downstream que blendam clips). **Keyframes viram só mais uma FONTE de sinal no grafo** — igual
  Cavalry (timeline e procedural coexistem) e igual Rive (states tocam timelines).

### Camada 2 — Vocabulário de motion nodes (fan-out puro, zero mudança central)

Todos = 1 crate `ph2d-node-motion-<slug>` cada, `Effect::Temporal` ou `Pure`, briefing existente:

- **Sinais** (o "oscillator rack" do Cavalry): `motion.oscillator` (wave já provado no `debug.wave`),
  `motion.noise` (o `Func::Noise` determinístico do ph2d-expr), `motion.envelope` (ADSR simples),
  `motion.lfo`.
- **Dinâmica com `pre`** (é para isso que o `pre` existe): `motion.spring` (mola crítica/sub-amortecida
  sobre o valor do tick anterior), `motion.smooth-damp`, `motion.inertia`, `motion.trail` (fila de N
  posições passadas).
- **Falloffs Cavalry-style** (`Domain::Field`): `falloff.radial`, `falloff.box`, `falloff.linear`,
  `falloff.noise` — cada um emite uma coluna `weight` por instância; **`falloff.stack`** compõe N
  falloffs com blend modes (Add/Min/Max/Multiply/Screen/Overlay — a lista verificada do Cavalry),
  bottom-up. Modifiers (transform/scale/tint) ganham porta opcional `weight`.
- **Cloner/stagger** (o coração do motion design): estender `motion.clone` com `motion.stagger`
  (offset de tempo por índice — o "delay por clone" que define o look Cavalry), `motion.along-path`
  (reusa o substrato `vector.pattern-along-path`), `motion.grid`→`radial`/`spiral`.
- **`motion.state-machine`** (padrão bevy_animation_graph + Rive): um nó cujo param opaco é a
  definição da máquina; estados referenciam `Clip`s/subgrafos; transições com condições sobre
  inputs tipados (number/bool/**trigger** — trigger = true-por-1-tick, casa com `pre`/`Clock::Event`);
  blend 1D entre estados durante a transição. Começa CPU-side, `Effect::Temporal` (lê playhead) —
  variante `Stateful` (gameplay, HR-5) fica para a IR de gameplay do ADR-0036.

### Camada 3 — UX de artista (ADR-0038 aplicado; é onde se ganha ou perde o jogo)

- **Behaviours, não grafo:** a superfície primária é **"arrastar um behaviour na camada"** (painel no
  Inspector, mesmo padrão do painter-layers/effects de hoje). Cada behaviour é açúcar que instancia
  um mini-grafo pré-conectado (preset result-named — princípio 4 do ADR-0038). O grafo cru é o
  "modo avançado", não a porta de entrada. Isso reusa a musculatura de UI que a PH2D já tem
  (painéis, sliders com `link_slider_number`) **antes** de existir canvas de grafo.
- **Timeline panel** (novo painel, padrão widget-gallery): dope sheet + curvas (referência visual:
  theatre.js). Scrub escreve no `Playhead`; o cook memoizado faz o resto.
- **Node canvas** (fase posterior): editor retained-mode em Vello (padrões de interação: egui-snarl;
  wiring tipado colorido por `PortType` — princípio 6 do ADR-0038 — os 3 eixos `Domain/Dim/Clock`
  já dão a cor/forma do pin de graça).
- **Data binding à la Rive (a lição estratégica):** params do grafo expostos como **contrato tipado**
  para Luau/MCP via `#[lua_export]` (HR-10) — o "View Model" da PH2D já tem onde morar. Não
  reinventar "inputs soltos" que o Rive acabou de deprecar.

### O que esbarra no freeze (ADR novo, Coordenador/Enio — mapeado desde já)

1. **`ParamSpec` só `f32`** — animar cor/Vec2 via param exige estender o contrato congelado
   (ADR-0039 prevê o rito: cap-bump + ADR). Alternativa sem ADR: valores ricos entram por **porta**
   (coluna no `Stream` / `Opaque`), não por param — dá para ir longe assim antes de precisar do bump.
2. **`AnimValue` no substrato?** Não — manter `AnimValue`/keyframes em `ph2d-anim` (satélite, padrão
   ADR: cross-module com 1 consumidor = satélite que só lê contratos). O grafo só vê `Stream`/`Opaque`.
3. **Vetorizar `ph2d-expr`** (Vec2/color nativos) — extensão futura já prevista no próprio crate;
   ADR quando a demanda for real (≥2 consumidores).

### Trade-offs assumidos

- **Pull memoizado vs push reativo:** mantemos pull (já congelado). Custo: scrub para trás recoze a
  cadeia Temporal inteira (sem cache por-playhead). Mitigação futura: cache LRU de frames cozidos
  por playhead quantizado — só se o profile pedir (HR: measure first).
- **Timeline híbrida vs "tudo é nó":** híbrida (Cavalry/Rive), deliberadamente. Keyframe é um dado
  amostrável, não um subgrafo — "tudo é nó" (puro Houdini) perde o artista (ADR-0038, princípio 5).
- **State machine CPU `Temporal` primeiro:** adia a variante gameplay-determinística (que exigirá a
  IR Luau do ADR-0036), destravando o caso motion-graphics (90% do valor) já.
- **Sem dependência Rive/bevy_animation_graph:** só padrões. rive-rs estagnado; bevy_animation_graph
  com compat de versão não confirmada e acoplado ao Bevy inteiro (não só bevy_ecs).

### Sequência sugerida (waves, no modelo funil do plano de nodes)

| Wave | Entrega | Regime |
|---|---|---|
| **M0** | Fio do tempo: `Playhead` resource + render bridge cook-por-frame (promover o smoke) + transporte play/pause/scrub | foundational leve (Modo L, gate testado) |
| **M1** | `ph2d-anim`: keyframe store + curvas + easing implementando `AttributeEvaluator`/`AnimationCurveSampler`; nó `motion.clip` | satélite + fan-out |
| **M2** | Fan-out de nós: oscillator/noise/envelope · spring/smooth-damp (via `pre`) · falloffs+stack · stagger/along-path | **fan-out puro, paralelo** (1 crate/agente) |
| **M3** | Behaviours no Inspector (presets que instanciam mini-grafos) + Timeline panel (dope sheet) | painéis (DIRETRIZ §3.B) |
| **M4** | `motion.state-machine` (padrão Rive/bevy_animation_graph) + triggers | fan-out + design doc |
| **M5** | Node canvas em Vello (wiring tipado por PortType) | UI maior, design system |
| — | Importador `.riv` (asset pipeline) · vetorização do `ph2d-expr` · cache por-playhead | backlog, sob demanda |

---

## 6. MiniCavalryV2 — o MVP do Enio vs este estudo (adendo 2026-07-06)

Análise de `/home/enio/Documentos/Recursos/Nodes/MiniCavalryV2` (~22,6k LOC JS vanilla, sem build,
Canvas2D, ~134 nós em 14 categorias, editor de grafo completo, timeline com keyframes, testes
headless + golden-image Playwright).

**Contexto histórico importante:** o MiniCavalry não é um projeto paralelo — é o **ancestral direto
do `ph2d-nodegraph`**. O `ARQUITETURA_DOIS_MUNDOS.md` (v2.1, 2026-05-21) espelha deliberadamente a
PH2D (ECS geracional, FixedStep 60 Hz, SceneDoc, Luau, HR-5), e o `PROMPT_AVALIACAO_ENGINE_PH2D.md`
é o pedido que originou os ADR-0030..0039 dias depois. O V2, porém, **andou muito além do lado
Rust**: 134 nós vs 19, UI completa vs zero, timeline+keyframes vs nada.

### Convergência com o estudo (validação empírica)

O MVP implementa, funcionando e intuitivo, quase tudo que este estudo recomendou por pesquisa:

| Recomendação do estudo | No MVP | Nota |
|---|---|---|
| Falloff Cavalry-style multiplicativo | `FalloffStrength` como atributo que flui na instância; cada Falloff **multiplica** no upstream (`prev * s`) = empilhamento Cavalry | `src/nodes/falloff.js` — com curvas linear/smoothstep/quad e invert |
| Stagger (o look Cavalry) | rampa min→max por índice × easing × `FalloffStrength`, **aditiva** ao valor atual | `src/nodes/stagger.js` |
| Spring/dinâmica | estado por-instância + guard de time-regression (scrub) + sub-step adaptativo (`subDt²·tension < 0.05` — truque de estabilidade que vale portar) | `src/nodes/spring.js` |
| Keyframes como fonte entre outras | **hierarquia de resolução de param: socket conectado > keyframe > literal** — resolve elegantemente "param animável" | `pull-evaluator.js:133-147` |
| Timeline híbrida | tracks por-param, binary search + easing por-segmento + hold nas pontas; multi-select/copy-paste/easing-menu na UI | `helpers.js::sampleKeyframeTrack` + `timeline.js` (915 LOC) |
| Time remap por subárvore | `nodeDef.processTime` recalcula o tempo dos upstreams; subárvore com tempo divergente **não usa cache** | `pull-evaluator.js:108-113` — é o TimeRemap do AE/Cavalry no modelo pull |
| Atributos nomeados com contrato | schema Houdini (`Position/Rotation/…/FalloffStrength/Skeleton`) + **`reads_attrs`/`writes_attrs` declarados e VALIDADOS** em runtime (violação = badge no nó) | `attr-schema.js` — é o contrato de atributo que falta no lado Rust |
| State machine como nó | `characters/stateMachine.js` | além de IK 2-bone/FABRIK, rubber hose, skin deformer |
| Fan-out de nós | **Node Maker**: artista desenha SVG com vocabulário de layers (`slot:in.value`, `mask:lock`, `behavior:*`) + `Instrução:` → agente gera o nó | espelha nosso briefing de node-crate, mas com o ARTISTA como autor — muito alinhado a HR-10/LLM-first |

O avaliador é **pull com cache por-frame** (`_evalCache` válido quando `time === _frameTime`),
grupos com input/output proxies (subgrafos), promotion de param→socket, conversão de tipos nas
arestas. Mesma família do nosso Cook — mais simples (limpa o cache todo frame em vez de
fingerprint), o que confirma que o nosso memo por-fingerprint é um **upgrade**, não uma divergência.

### O que NÃO portar literalmente (gaps vs Hard Rules)

1. **Estado mutável dentro do `process()`** (`node._springState`, `_lastTime`): no substrato Rust
   isso é exatamente o caso das arestas **`pre`** (ou `Effect::Stateful`). O spring JS integra com
   `dt = time - _lastTime` de wall-clock (cap 0.033) — precisa re-derivar em fixed-step/`pre` para
   ser determinístico.
2. **`setAttr` imutável clona a instância a cada write** (`Object.assign` por atributo): semântica
   correta (pureza), representação proibida (HR-3 — tempestade de alloc). O `Stream` SoA colunar já
   é a representação certa; porta-se a *semântica*, não o objeto-por-instância.
3. **Keyframes amostram `STATE.timeline.currentTime` global**: no Rust o playhead já entra pelo
   `EvalCtx` — o keyframe store vai para o `ph2d-anim` (Camada 1 deste estudo), com o
   `sampleKeyframeTrack` do MVP como spec direta da implementação.
4. **Transcendentais livres** (`Math.sqrt/sin` no motion): ok para presentation (isenta de HR-5),
   mas o lowering gameplay rejeita (o `ph2d-expr` já modela isso via `is_deterministic()`).
5. **Promotion + keyframe por-param** pressionam o `ParamSpec` congelado (só `f32`): o MVP é a
   evidência concreta para o cap-bump/ADR previsto na §5 — e a hierarquia socket>keyframe>literal
   deve ser adotada como ordem canônica de resolução nesse ADR.
6. **Grupos/subgrafos** (input/output proxy): o `Graph` Rust é flat — subgrafo é extensão futura
   (o Blender lazy-graph composável é a referência de como fazer certo).

### Veredito

O MiniCavalryV2 é **a spec executável das camadas 2 e 3 deste estudo** — ele responde com artefato
funcionando a pergunta que a pesquisa respondeu por fontes: behaviours+falloffs+stagger+timeline
híbrida É o modelo certo de UX, e é fácil de usar. A pesquisa externa e o MVP chegaram ao mesmo
lugar por caminhos independentes, o que é o melhor sinal possível.

Estratégia recomendada: o MVP permanece **ferramenta de autoria/spec** (como o próprio doc dele
define — engine é read-only para ele); o port é **por semântica de nó** (1 nó JS → 1 crate
`ph2d-node-motion-*` no fan-out aberto), nunca por tradução do avaliador ou do modelo de instância.
As waves M0/M1 (fio do tempo + `ph2d-anim`) da §5 são exatamente os pré-requisitos para recebê-los;
o inventário de 134 nós do MVP vira o backlog priorizado do M2, e o Node Maker vira o pipeline de
autoria de nós por artista em cima do briefing de fan-out existente.

## 7. Fontes principais

- Blender lazy-function graph: [commit 4130f1e (Jacques Lucke, 2022)](https://github.com/blender/blender/commit/4130f1e674f83fc3d53979d3061469af34e1f873) · [Fields](https://developer.blender.org/docs/features/nodes/fields/) · [Depsgraph](https://developer.blender.org/docs/features/core/depsgraph/)
- Cavalry Behaviours/Falloffs: [docs.cavalry.scenegroup.co/nodes/behaviours](https://docs.cavalry.scenegroup.co/nodes/behaviours/)
- Graphite/Graphene: [repo](https://github.com/GraphiteEditor/Graphite) · [guia Graphene](https://graphite.art/volunteer/guide/graphene/)
- Rive: [rive-runtime](https://github.com/rive-app/rive-runtime) · [rive-rs](https://github.com/rive-app/rive-rs) · [formato .riv](https://rive.app/docs/runtimes/advanced-topic/format) · [state machine](https://rive.app/docs/editor/state-machine/state-machine) · [data binding](https://rive.app/docs/editor/data-binding/overview) · [Luau](https://rive.app/blog/why-scripting-runs-on-luau)
- bevy_animation_graph: [repo](https://github.com/mbrea-c/bevy_animation_graph) · Godot: [repo (MIT)](https://github.com/godotengine/godot) · OTIO: [repo](https://github.com/AcademySoftwareFoundation/OpenTimelineIO) · Motion Canvas: [repo](https://github.com/motion-canvas/motion-canvas) · theatre.js: [repo](https://github.com/theatre-js/theatre) · egui-snarl: [repo](https://github.com/zakarumych/egui-snarl)
- Interno: `ph2d-nodegraph`/`ph2d-expr` (ADR-0030..0039), `ph2d-vector-traits` (`AnimValue`/`sample`), `ph2d-eval-motion`, `docs/plans/2026-05-node-waves.md`

**Claims refutadas na verificação (não usar):** "o field graph do Blender tem exatamente 2 tipos de
nó" (falso — há mais); "bevy_animation_graph suporta Bevy 0.18" (não confirmado).
