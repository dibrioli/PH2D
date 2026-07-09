> **PROVENIÊNCIA:** cópia de `/home/enio/Documentos/Recursos/Nodes/DESIGN_NODE_GRAPH_PH2D.md`
> (v1.0, 2026-05-17), trazida pro repo em 2026-07-09 a pedido do Enio. É o doc de design
> canônico PRÉ-implementação do sistema de nós; os princípios P1–P10, as zonas (P4) e os
> anti-padrões (§13) continuam norte válido. Detalhes de implementação foram superseded
> pelos ADR-0030..0039 (substrato real: `ph2d-nodegraph`, Cook, `pre`, contrato congelado
> ADR-0039). Para o modelo canônico de MOTION, vide `03_reentrada_integrate_estudo_padrao_ouro.md`.

# PH2D — Sistema de Nós — Design Canônico

**Versão:** 1.0 — 2026-05-17
**Status:** Pronto para virar ADR-0029 + plano operacional + DIRETRIZ §4
**Autor:** Claude Opus 4.7 (1M context) com base em 6 pesquisas profundas + handoff com agente PH2D
**Audiência primária:** time PH2D (Enio + agentes implementadores)
**Audiência secundária:** futuros agentes Claude lendo este doc para implementar M14+

> Doc opinionado por design. Reverter decisão central exige ADR superseding.
> Ambição declarada: **o melhor sistema de nós já construído em uma engine 2D**.
> Não "mais um Cavalry". Não "outro Blueprint". Algo categoricamente novo.

---

## 1. Visão e ambição

Um único sistema de grafos na PH2D que cobre **três domínios distintos** (Cena, Lógica, Shader) com **paradigma visual unificado**, **tipos e evaluators isolados por contexto**, e **LLM como co-autor de primeira classe** desde o dia 1.

A frase que diferencia:

> **PH2D = Cavalry × Cursor × Houdini × Godot.**
> Motion design dentro da engine (Cavalry). LLM autora grafos COM o humano via MCP (Cursor). Procedural multi-contexto (Houdini). Acessível a iniciantes (Godot's curve, sem o erro do VisualScript).

Nenhuma engine atual faz isso. Unreal tem Blueprint + Material + Niagara como 3 ferramentas isoladas. Unity tem Visual Scripting bolt-on. Godot removeu VisualScript em 4.0. Houdini é poderoso mas mira TDs. Cavalry tem motion design, sem lógica. Blender separa em editores. **PH2D pode preencher esse vácuo.**

A engine venceu três spikes que viabilizam esta ambição:
- **Luau via mlua** (ADR-0019): replay cross-platform validado, GC 277× sob budget, hot reload Defold-style funcionando.
- **MCP first-class** (HR-10): paridade Luau↔MCP via `ph2d-bindgen` em CI.
- **Convention-by-discovery** (ADR-0027): tool crates auto-registráveis sem editar registry central — pattern que escala node-types.

O sistema de nós herda esses três pilares.

---

## 2. Contexto: PH2D e onde o sistema vive

### 2.1 PH2D em uma página

- Engine 2D em **Rust 2024**, MSRV 1.92, toolchain 1.95.
- **Vello 0.8 + wgpu 28** para rasterização vetorial GPU-compute (kurbo paths + peniko paint).
- **bevy_ecs 0.18** standalone, two-world model **SimWorld** (canônico/determinístico) + **PresentWorld** (presentation/visual), ponte one-way via `extract!` macro (ADR-0021).
- **Luau** (mlua 0.10) como linguagem gameplay canônica (ADR-0019); ScriptHost por mundo; hot reload reset+restore.
- **MCP server** first-class (HR-10); `ph2d-bindgen` gera `.d.luau` + schema MCP das mesmas anotações.
- **Plataformas:** iOS/macOS/Android/Windows/Linux/Web (todas com WebGPU/Metal/Vulkan/D3D12 — sem fallback legacy).
- **Editor** próprio em Vello+parley (HR-7 editor=engine); design system canônico já entregue (M12-M13, 32 widgets, hero screen, AccessKit wired).
- **Hard Rules** HR-1..HR-18 inegociáveis.

### 2.2 Onde o sistema de nós se encaixa

Status atual do canon (SKILL §11.9, linha 575): *"Out of scope até M13+: ... node graph editor ..."*. Design system canônico (tokens + 32 widgets + hero screen) entregue. **Janela de implementação é agora**, antes de M14 começar.

Implicação prática: cada decisão deste doc precisa respeitar HRs já estabelecidas. Onde uma HR força trade-off não-óbvio, está marcado abaixo.

---

## 3. Princípios de design — os 10 inegociáveis

Cada princípio carrega `Why` (origem) e `How` (mecanismo). Reverter exige ADR superseding.

### P1 — Low floor + wide walls + high ceiling
**Why:** Resnick (Scratch). Os 5 primeiros minutos produzem algo visível ("Pulsar" + "Render" + bang). O mesmo paradigma cobre Scene/Logic/Shader/Audio/Physics (futuro). Profissionais não batem teto.
**How:** Catálogo domain-dense (não Math.Sin); preview ao vivo no canvas; subnet/composite para escalar (P10).

### P2 — Conexão imediata (Bret Victor)
**Why:** "Stop Drawing Dead Fish" — criadores precisam de feedback imediato entre ação e resultado.
**How:** Render do canvas atualiza por frame; **probe em qualquer wire** (clicar pinia mini-readout do valor corrente); preview-on-node para shader e gerador.

### P3 — Densidade de domínio > paradigma visual
**Why:** Godot VisualScript morreu (0.5% adoção em 4.0) porque expunha `Add`/`Multiply`/`Set` genéricos. Blueprint sobrevive porque tem `Spawn Actor`, `On Begin Overlap`, `Play Animation` — nós **densos em domínio**.
**How:** Catálogo seed v1 inteiro composto de nós high-level específicos de motion design (`Pulsar`, `Onda`, `Falloff`, `Brilho`, `Stagger`). Operações genéricas (`+`, `*`) ficam em drawer "Avançado" + nó `Custom Rust`/`Custom WGSL` como escape hatch.

### P4 — Uma só espécie de fio
**Why:** Blueprints separa execução (branca grossa) de dados (fina colorida) — necessário para AAA games, **excessivo para artista/criança**. Blender resolve com **zonas** (Repeat Zone, If Zone) e fluxo único.
**How:** Sem exec wire. Controle de fluxo via **zonas com brackets coloridos** (Blender Geometry Nodes pattern): `If { ... } else { ... }`, `Repeat N { ... }`, `Forever { ... }`. Trigger e value coexistem no mesmo fio, diferenciados por **espessura** (Pure Data pattern): fino = trigger, grosso = stream de valor.

### P5 — Contextos isolados, bridges explícitas
**Why:** Cables.gl unificou tudo e virou mush para iniciantes. Houdini separou em 10 contextos e ficou para TDs. **Sweet spot: poucos contextos, fronteiras explícitas.**
**How:** `Context` enum N-variants extensível (Scene, Shader, Logic — Audio/Physics futuros). Sockets de contextos diferentes **não conectam diretamente**; passagem via nós-bridge nomeados (`Commit Transform to Sim`, `Material`, `Uniform`, `Trigger Animation`).

### P6 — Determinismo onde prometido (HR-5)
**Why:** Rollback/Lockstep multiplayer quebra silenciosamente em 1 ULP. HR-5 inegociável.
**How:** Cada `NodeManifest` declara `sim_safe: bool`. Grafos targeting SimWorld em modo Rollback/Lockstep **compilam apenas** nós `sim_safe=true` — rejeição em compile time, não runtime. GPU compute proibida; HashMap proibida (ADR-0022); RNG seedado (Pcg64Mcg); `pairs_sorted()` em iteração Luau.

### P7 — Hot path sem alocação (HR-3)
**Why:** Eval por frame não pode jitter. Audio thread morre com 1 alloc.
**How:** Evaluator usa `bumpalo::Bump` arena reset/frame (pattern já em uso no editor input pipeline). `Node::eval(&self, ctx: &mut EvalCtx<'a>)` aloca no arena `ctx.arena: &'a Bump`. Stress test: 1000 nós × 60 frames, allocações fora do arena = 0 (dhat-rs em CI).

### P8 — Editor = engine, MCP = humano (HR-7 + HR-10)
**Why:** Não há fork "engine vs runtime". Não há fork "humano vs LLM". A mesma API serve os dois.
**How:** Toda operação editorial (criar nó, conectar, desconectar, mover, deletar, agrupar, expandir) é uma `Tool MCP` (`graph.add_node`, `graph.connect`, etc.). Cada `NodeManifest` declara `#[mcp_expose]`; `ph2d-bindgen` gera schema em CI. Em release de jogo (`editor=off`), o eval do grafo permanece — só o autoramento desaparece.

### P9 — A11y day-1 (HR-12)
**Why:** Editor sem AccessKit não passa em CI (HR-12). VoiceOver/Narrator precisa navegar nós, pinos, fios. UE Accessibility Act 2025 + AppStore review dependem disso.
**How:** Cada `Node`, `Pin`, `Edge` implementa `accesskit::Node` builder. Wire navigation via teclado (Tab cycla pinos; Enter follows wire). Probe lê valor em Braille. ARIA labels via Fluent (HR-15).

### P10 — Composição é fundacional (Ω9)
**Why:** Godot VisualScript não escalava porque não tinha subnet. Houdini HDA, TouchDesigner Component, Cavalry Composition, Blender Node Group — todos existem porque grafos reais têm centenas de nós.
**How:** Subnet é **nó-tipo especial** no catálogo. Internamente referencia outro `Asset::Graph` via handle. Recursão proibida (compiler rejeita ciclos em build time). Mesma API entre subnet e grafo top-level.

---

## 4. Arquitetura: sistema de contextos extensível

### 4.1 O `Context` enum

```rust
/// Discriminator de contexto do grafo. v1 ship apenas Scene.
/// Shader, Logic, Audio, Physics são adições aditivas (não reescrita).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Reflect, Saveable)]
pub enum Context {
    Scene,    // v1 — motion design, PresentWorld
    Shader,   // v1.5 — fragment effects, GPU pipeline
    Logic,    // v2 — visual scripting compila Luau
    // futuros (não shippar):
    // Audio,    // DAW-style, ph2d-audio quando wired
    // Physics,  // XPBD/particles, ph2d-physics-soft quando wired
}
```

**v1 = Scene-only.** Decisão D2 fundamentada: Scene é o **diferenciador competitivo** (motion design dentro da engine — Unreal/Unity não têm). Shader é commoditizado (todas as engines maiores têm). Logic exige D1 fechado + decisões sobre eval determinismo. Cada contexto incremental é **aditivo, não refactor**.

### 4.2 Os 3 contextos canônicos

| Contexto | Domínio | World | Tipos de socket | Direção do fluxo | Output node |
|---|---|---|---|---|---|
| **Scene** | Motion design (animar transforms, sprites, modifiers, filtros) | PresentWorld puro | shape, point, value, color, gradient, pulse, skeleton | Esq → Dir | `Render` (terminal) |
| **Shader** | Pixel/fragment effects (postFX, materials) | GPU pipeline | float, vec2, vec3, vec4, sampler2D, matrix | Esq → Dir, output node à direita | `Material Output` (color + alpha + distortion vec2) |
| **Logic** | Gameplay scripting (events, conditions, state machines) | SimWorld OR PresentWorld (gated por target+sim_safe) | trigger, bool, number, string, reference&lt;Entity&gt;, reference&lt;Asset&gt; | Esq → Dir | `Sink` (event handler, no return) |

**Não há contexto unificado.** Tentativa de "tudo conecta com tudo" mata o sistema (Cables.gl + Quartz Composer + Godot VisualScript todos provaram).

### 4.3 Bridges explícitas entre contextos

Bridges são **nós normais no catálogo** com semântica especial:

```
Scene → SimWorld:
  [Commit Transform to Sim]   — escreve Transform em SimWorld via extract reverse (audit log)
  [Commit Color to Sim]
  [Snapshot State]

Scene → Shader:
  [Material]                  — encapsula sub-grafo Shader; output é input do Render
  [Uniform Bind]              — expõe param Scene como uniform GPU

Logic → Scene:
  [Set Property]              — escreve parâmetro de nó Scene
  [Trigger Animation]         — reseta tempo/dispara play em nó Scene
  [Spawn Sprite]              — cria entidade via comando

Logic → Shader:
  [Uniform Push]              — atualiza uniform GPU a cada frame
  [Shader Switch]             — troca material de uma entidade

Logic → SimWorld:
  Quando Logic compila para SimWorld target, todos os nós da árvore são
  sim_safe. Bridge é implícita (target do compile). HR-11 audit log.
```

**O fluxo principal nunca atravessa contextos.** Bridges nomeadas tornam a transição visível, auditável, e ensináveis para LLM.

### 4.4 Direção e layout (vertical vs horizontal)

**Decisão:** **horizontal** (esq→dir) como direção dominante em todos os contextos. Razão: nós têm sliders/dropdowns/swatches internos; pin + value-field na mesma linha cabe em horizontal, não em vertical (Houdini SOPs precisam de inspector lateral porque são verticais; PH2D não quer essa cisão).

**Exceção Nuke-pattern:** nós **junction/merge** (Mixer, Group, FalloffBoolean) aceitam inputs pela borda **superior**, fluxo principal segue horizontal. Visualmente: "este nó é uma confluência de afluentes". 95% dos nós: pin esq + pin dir. ~5% (junctions): pin topo + pin dir.

---

## 5. Modelo de dados

### 5.1 Tipos canônicos (em `ph2d-nodegraph-core`)

```rust
/// Identificador estável de um nó dentro de um grafo. Hash do (graph_id, node_local_id).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Reflect, Saveable)]
pub struct NodeId(u64);

/// Identificador estável de um tipo de nó. Hash FNV-1a 64-bit do "scene.pulsar".
/// Mesma estrutura que ph2d-tool-registry::node_id (ADR-0027).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeTypeId(u64);

/// Identificador de socket dentro de um nó.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PinId { pub node: NodeId, pub local: u16 }

/// Aresta entre dois pinos. Direção: from.output → to.input.
#[derive(Copy, Clone, Reflect, Saveable)]
pub struct Edge { pub from: PinId, pub to: PinId }

/// Manifest declarativo de um tipo de nó. Espelha ToolManifest (ADR-0027).
/// Cada tipo de nó exporta `pub const MANIFEST: NodeManifest`.
pub struct NodeManifest {
    pub type_id: NodeTypeId,            // hash de "scene.pulsar"
    pub label_key: LabelKey,            // "node.scene.pulsar.label" — HR-15
    pub category: NodeCategory,         // header color via this
    pub silhouette: NodeShape,          // Rect/Circle/Diamond/Cigar/TrapezoidDown/TrapezoidUp/Tabbed
    pub icon_fn: fn() -> BezPath,       // Vello path
    pub inputs: &'static [PinSpec],
    pub outputs: &'static [PinSpec],
    pub params: &'static [ParamSpec],
    pub context: Context,               // Scene/Shader/Logic — wire compat enforced
    pub sim_safe: bool,                 // P6 — gate para grafos SimWorld
    pub reads_attrs: &'static [AttrId], // P-influence layer: o que esse nó lê
    pub writes_attrs: &'static [AttrId],// o que esse nó escreve
    pub a11y_role: Role,                // HR-12
    pub memory_budget: MemoryBudget,    // HR-13
    pub mcp: McpExposure,               // HR-10/HR-11
    pub eval_fn: fn(&EvalCtx, &Params, &Inputs) -> Outputs,
    pub emit_luau_fn: Option<fn(&LuauEmitter, &Params) -> ()>,    // None para Scene/Shader
    pub emit_wgsl_fn: Option<fn(&WgslEmitter, &Params) -> ()>,    // None para Scene/Logic
}

/// Especificação de um socket.
pub struct PinSpec {
    pub name: &'static str,
    pub label_key: LabelKey,
    pub data_type: DataType,            // float, shape, trigger, etc.
    pub cardinality: PinCardinality,    // Single (círculo) ou Stream (diamante) — P-visual
    pub side: PinSide,                  // Left/Right/Top (Nuke junctions)
}

/// O grafo em si — vive como Asset, hash blake3 (HR-6).
#[derive(Reflect, Saveable)]
#[saveable(version = 1)]
pub struct Graph {
    pub context: Context,
    pub nodes: Vec<NodeInstance>,
    pub edges: Vec<Edge>,
    pub default_params: BTreeMap<NodeId, ParamMap>,  // BTreeMap, não HashMap — ADR-0022
    pub timeline: Option<Handle<Timeline>>,          // Ω3 — companheiro opcional
}

/// Instância de grafo no ECS. Component em PresentWorld (ou SimWorld para Logic).
#[derive(Component, Reflect, Saveable)]
#[saveable(version = 1)]
pub struct GraphInstance {
    pub asset_id: Handle<Graph>,
    pub param_overrides: SmallVec<[(NodeId, ParamName, Value); 8]>,
    pub runtime_state: GraphRuntimeState,  // HR-16 POD-only
}
```

**Notas:**
- `BTreeMap` substitui `HashMap` em todo state simulado (ADR-0022). Iteração determinística cross-OS.
- `SmallVec<[...; 8]>` permite 8 overrides inline sem heap (caso comum); spillover heap apenas em casos extremos.
- `version: u32` em `Graph` e `GraphInstance` para migração HR-14.

### 5.2 Atributos canônicos (do `reads_attrs`/`writes_attrs`)

Pré-requisito da **Camada 1 de Inteligibilidade** (Influence Highlight, §10). Declarar atributos como enum estável:

```rust
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum AttrId {
    // Scene attrs
    Transform, Color, Alpha, Scale, Rotation, Anchor, Pivot,
    FalloffStrength,        // o atributo central do Mini Cavalry research
    FilterBlur, FilterHue, FilterSaturate, /* etc */
    Index, Count, Seed,
    // Logic attrs
    Trigger, Bool, Number, Reference,
    // Shader attrs
    Color3, Alpha1, Distortion2,
}
```

Mini Cavalry mapeou 70 atributos implícitos no JS protótipo. Em PH2D, esses ~30 atributos canônicos são **declarados explicitamente** por cada nó-tipo. **Sem isso, Influence Highlight (Camada 1) não funciona** — e Influence Highlight é o ganho UX mais alto-impacto.

---

## 6. Camada visual — as 5 sub-camadas

Cada nó comunica em 5 canais visuais ortogonais. Mini Cavalry usava 1 (ícone colorido). Houdini usa 5+. PH2D usa exatamente 5 para evitar overload (lição: >7 estados visuais por nó = ruído).

### 6.1 Silhueta (forma do nó)

Atribuída automaticamente por `NodeManifest::silhouette`. Lê em zoom-out sem decifrar texto. Inspirado em Houdini, reduzido ao **conjunto semanticamente útil**:

```
┌──────────────┐         ___           __________
│  RECT        │        /     \       /          \
│  (genérico)  │       | NULL  |     /  SOURCE    \    — Trapezoid down
└──────────────┘        \_____/      \____________/

   /\                ___________________
  /  \              /                   \
 < DI >            ( CIGAR (multi-merge) )    — Cigar / oval longo
  \  /              \___________________/
   \/  branch                                  __________
                                              /          \
 ___                                         /  SINK      \  — Trapezoid up
|   \_____________                          /______________\
| TAB (file/external)|
|____________________|
```

**Inventário canônico v1 (7 silhuetas, não 30):**

| Silhueta | Uso | Exemplos |
|---|---|---|
| `Rect` | Processamento genérico (95% dos nós) | Spin, Move, Bloom, Falloff |
| `Circle` | Waypoint / null / output marker | `OUT_*`, Render |
| `Diamond` | Branch / switch / decisão | `If`, `Switch`, `FalloffBoolean` |
| `Cigar` | Multi-input merge | Mixer, Group |
| `TrapezoidDown` | Source — dado nasce | Shape, Random, ParticleEmitter, DataSource |
| `TrapezoidUp` | Sink — dado termina | `Commit*` bridges, Logic event sinks |
| `Tabbed` | I/O externo (asset, arquivo) | Image, Font, Audio sample |

Atribuição **100% automática** pelo manifest. Sem hotkey (criança não decora `Z`); usuário pode override em v2.

### 6.2 Cor — header colorido por categoria

Categorias funcionais com hex calibrado em OKLCH (Wave 4 paleta semântica, dark mode `#1e1e1e`):

| Categoria | Hex header | Significado | Exemplos |
|---|---|---|---|
| Source | `#3E7F4F` verde-musgo | "dado nasce aqui" | Shape, Random, Noise, AudioReact, DataSource |
| Distribute | `#5A8A6C` verde-jade | "espalha no espaço" | DistributeGrid, Path, Fibonacci, Duplicator |
| Transform | `#4A6FA5` azul-aço | "modifica em tempo/espaço" | Spin, Wiggle, Move, Stagger, LookAt |
| Focus | `#A87C3A` âmbar | "foca ou modula influência" | Falloff, FalloffBoolean, NumberRange |
| Filter/FX | `#9C4A8B` ametista | "pós-processo visual" | Bloom, Blur, Glow, HueShift, RgbSplit |
| Logic | `#6E5A8A` ametista frio | "controle/evento" (contexto Logic) | OnClick, If, Repeat, Wait |
| Shader | `#B85A5A` rosa-quente | "fragment effect" (contexto Shader) | Sample, Combine vec3, BSDF, Mix |
| Output | `#B0463A` vermelho-tijolo | "termina o grafo" | Render, Material Output, Sink |
| Bridge | `#888888` cinza-grafite neutro | "cruza contexto" | Commit *, Material, Uniform, Trigger Animation |

**Header colorido + body cinza neutro `#2a2a2a`.** Ícone monocromático `#cfcfcf` (não duplica canal de cor). Borda livre para **estado**: amarelo `#FFD60A` quando selecionado, vermelho `#FF5252` pulsando quando erro (HR-12 implica feedback claro).

### 6.3 Forma de pino

| Forma | Cardinalidade | Exemplo |
|---|---|---|
| Círculo `○` | `PinCardinality::Single` — passa 1 valor | constantes, números, cores únicas |
| Diamante `◇` | `PinCardinality::Stream` — passa N elementos | streams de instâncias, arrays |

Inspiração: Blender Geometry Nodes (2024-2025 socket shape redesign). Custo: ~30 linhas paint. Ganho: tipo lido em 0.3s.

### 6.4 Cor de pino (por `DataType`)

Convenção emergente Unity/Babylon NME, validada cross-pesquisa:

```
shape       roxo     #8b5cf6
point       azul     #38bdf8
value/float cinza    #5eead4 (era ciano)
color/vec3  amarelo  #facc15
gradient    laranja  #fb923c
pulse/trig  lima     #a3e635
skeleton    rosa     #ec4899
vec2        verde    #4ade80
vec4        rosa-vec #f472b6
sampler2D   vermelho #ef4444
matrix      azul-esc #1d4ed8
bool        roxo-bool #a78bfa
trigger     vermelho fino #dc2626
reference   verde-ref #16a34a
```

Lint enforced em `tests/architecture/node_pin_color_canonical.rs` (Ω8 — espelha ADR-0028 codegen design canonical).

### 6.5 Espessura de fio

Pure Data lesson:

- **Fio fino vermelho** = trigger/event (fires at moments)
- **Fio grosso colorido** = stream de valor animado (varia toda hora)

Mesma cor da categoria do tipo; espessuras diferentes. Criança capta em 30 segundos: "fino = clique, grosso = posição".

---

## 7. Avaliação e runtime

### 7.1 Três motores, um modelo

Cada contexto tem seu evaluator. **Não tentar unificar** (Quartz Composer morreu nessa armadilha):

```
Scene context     ──► PullLazyEvaluator   ──► PresentWorld extract
Logic context     ──► LuauCompiler+Push    ──► SimWorld (sim_safe) OU PresentWorld
Shader context    ──► WgslCompiler         ──► GPU pipeline (cooked asset)
```

### 7.2 Scene evaluator — pull lazy com bumpalo

```rust
pub struct EvalCtx<'a> {
    pub arena: &'a Bump,                    // P7 zero-alloc HR-3
    pub time: f32,
    pub mouse: Vec2,
    pub frame: u64,
    pub present_world: &'a PresentWorld,
    pub probes: &'a mut ProbeSink,          // P2 + Camada 5 inteligibilidade
}

pub trait SceneNode {
    fn eval<'a>(&'a self, ctx: &mut EvalCtx<'a>, inputs: Inputs<'a>) -> Outputs<'a>;
}
```

Pull-lazy via DAG traversal a partir do `Render` terminal. Cache por frame (mesmo padrão Mini Cavalry `_evalCache`). Arena reset no fim do frame.

### 7.3 Logic evaluator — compile to Luau bytecode (D1 / L1)

```rust
pub struct LuauEmitter<'a> {
    pub buf: BumpString<'a>,    // arena-backed string builder
    pub depth: u32,
    pub sym_table: SymbolTable, // local var naming
}

// Cada nó-tipo Logic implementa:
fn emit_luau(&self, w: &mut LuauEmitter, params: &Params, inputs: &[Sym]) -> Sym { ... }
```

Compilador percorre o DAG topologicamente, emite Luau source em `BumpString`, mlua compila para bytecode (optimization_level=2, debug_level=0 em release), bytecode roda no `ScriptHost` existente. Herda:

- Replay cross-platform (C9 do spike)
- GC budget (HR-9, p99 0.005ms)
- Hot reload Defold-style (reset+restore via snapshot HR-16)
- MCP exposure automática (ph2d-bindgen)
- Sandbox (`Lua::sandbox(true)`)

**Hot path nuance:** nós Logic em loops apertados (collision per-frame) **não são Luau-compiled** — são `Rust-backed node-types` que o grafo referencia por handle. Grafo orquestra; folhas pesadas são Rust. Mesmo modelo Unreal Blueprint → C++.

### 7.4 Shader evaluator — compile to WGSL via naga

```rust
pub struct WgslEmitter<'a> {
    pub bindings: BindingTable,     // @group(0)/(1)/(2) per toji.dev
    pub fragment_body: BumpString<'a>,
    pub vertex_body: BumpString<'a>,
}
```

Compilador gera WGSL string; naga compila para SPIR-V/MSL/HLSL/GLSL no build. `PipelineLayoutDescriptor` sempre explícito (§10.5 do SKILL — `auto` proibido). Convenção bind groups: `@group(0)` frame, `@group(1)` material, `@group(2)` draw. Cooked como asset blake3-addressed via `tools/shader-cooker` (já planejado).

### 7.5 Determinismo (HR-5)

`sim_safe: bool` em cada `NodeManifest`. Compilador Logic com `target: SimWorld` em modo `Rollback`/`Lockstep`:

```rust
for node in graph.nodes() {
    let m = registry.manifest(node.type_id);
    if target.requires_determinism() && !m.sim_safe {
        return Err(CompileError::NotSimSafe {
            node_type: m.label_key,
            reason: "uses GPU compute / HashMap / FMA / thread_rng"
        });
    }
}
```

Erro **claro, compile-time, não runtime**. Mensagem sugere alternativa ("use `Glow Sim` em vez de `Bloom`; passe via `Commit Color to Sim`").

---

## 8. LLM-co-author como pilar fundacional (Ω1)

**A vantagem competitiva real.** Não é polish M20; é fundação M14.

### 8.1 Reframing

Nenhuma engine atual tem LLM co-autora de grafos como feature de primeira classe. Unreal Blueprint + AI é bolt-on (Copilot lendo .uasset). PH2D pode ser a primeira onde:

- LLM **autora** grafos via MCP tools
- LLM **explica** grafos em linguagem natural
- LLM **refatora** grafos (extrair subnet, renomear, simplificar)
- LLM **debuga** ("por que o sprite não está girando?" → analisa grafo, identifica `Spin` sem `falloffStrength` chegando)
- Humano valida + refina; LLM aprende com correção

### 8.2 MCP tool surface (mínimo v1)

| Tool MCP | Semântica | HR-11 destructive |
|---|---|---|
| `graph.list` | lista grafos no projeto | non-destructive |
| `graph.inspect(graph_id)` | retorna estrutura JSON (nós, edges, params) | non-destructive |
| `graph.explain(graph_id)` | LLM-side: retorna linguagem natural ("This graph makes sprite breathe at 0.5Hz") | non-destructive |
| `graph.add_node(graph_id, type_id, position)` | adiciona nó | non-destructive |
| `graph.connect(from_pin, to_pin)` | conecta | non-destructive |
| `graph.disconnect(edge_id)` | desconecta | non-destructive |
| `graph.set_param(node_id, param, value)` | edita | non-destructive |
| `graph.move_node(node_id, position)` | rearranja | non-destructive |
| `graph.collapse(nodes[]) → subnet` | extrai subnet | non-destructive |
| `graph.expand(subnet_id)` | expande inline | non-destructive |
| `graph.delete_node(node_id)` | **destructive** | requires token (HR-11) |
| `graph.clear(graph_id)` | **destructive** | requires token (HR-11) |
| `graph.commit(graph_id)` | persiste no AssetDb (blake3) | non-destructive |

Cada tool gera audit log entry em destructive (HR-11). Tokens single-use 5min (igual ao resto do MCP).

### 8.3 LLM affordances no UI

- **Sidebar "Co-author"** mostra histórico de ações MCP recentes
- **Comando-K palette** ("show command palette"): humano digita "make this sprite breathe" → LLM chama tools, grafo aparece
- **Explain mode**: hover qualquer subnet → tooltip gerado por LLM em pt-BR/en-US
- **Auto-doc**: subnet salva como asset ganha doc gerada pela LLM ao criar

### 8.4 Por que isso muda a API de `ph2d-nodegraph-core`

API precisa ser **MCP-shaped desde o dia 1**:
- Identificadores estáveis (LLM precisa referenciar nó "node_42" entre tools)
- Operações **incremental e reversível** (undo/redo no AssetDb; cada operação MCP é uma transação)
- Serialização sempre canonicalizada (mesma ordem de campos, mesma indentação JSON) para LLM diffar
- Erros estruturados (LLM precisa entender "porque essa conexão falhou" para corrigir)

Retrofitar isso depois é caro. Fazer agora é **uma decisão de design**, não trabalho extra.

---

## 9. Subnet, composição e reuso (Ω9)

**Sem isso o sistema não escala** — Godot VisualScript provou.

### 9.1 Subnet como nó-tipo especial

```rust
pub struct SubnetNode {
    pub graph_asset: Handle<Graph>,
    pub param_overrides: SmallVec<[(InternalNodeId, ParamName, Value); 8]>,
}
```

Subnet é nó normal no catálogo, mas seu `eval` chama recursivamente o eval do graph referenciado. Inputs/outputs do subnet são os `InputProxy`/`OutputProxy` do graph filho (mesmo padrão Mini Cavalry).

### 9.2 Restrições

- **Recursão proibida.** Compiler detecta ciclos no build do registry e rejeita.
- **Cross-context proibido.** Subnet de Scene só contém Scene. (Bridges entre contextos são nós explícitos no nível superior.)
- **Versionamento por hash.** Asset blake3 garante que subnet referenciada não mude silenciosamente. Save snapshot inclui hash da subnet asset; mismatch ao carregar = erro claro.

### 9.3 UX de entrar/sair

- **Duplo-clique** em subnet → entra
- **Breadcrumb no topo**: `Project / Scene Graph "Hero Idle" / Subnet "Breathe"`
- **Esc** ou clique no breadcrumb → sai
- **Animação 200ms zoom-in** ao entrar (confirma cognitivamente que trocou)

### 9.4 Material Function paralelo

Em contexto Shader, subnet = Material Function (Unreal pattern). Em Logic, subnet = "Custom Block" (Snap! pattern). **Mesmo mecanismo, diferentes nomes** para encaixar mental model do usuário.

---

## 10. Inteligibilidade — as 8 camadas de UX

Recapitulação consolidada do plano que veio do research Mini Cavalry. Cada camada resolve uma forma específica de opacidade.

### Camada 1 — Influence Highlight on-demand ★★★★★

Ao selecionar um nó, o sistema:
1. Consulta `reads_attrs` / `writes_attrs` declarados no manifest
2. Calcula downstream que **lê** algum atributo que o selecionado **escreve**
3. **Pinta nós afetados com a cor do selecionado**, dessaturra os outros
4. Anima pulse nos wires entre eles

Resolve diretamente o caso clássico Mini Cavalry "Falloff afeta Spin/Stagger mas não Bloom". Custo baixo (reaproveita DAG traversal). Modo "Substance Designer Highlight Flow" + "active viewer wire Blender".

### Camada 2 — `reads`/`writes` declarativos no NodeManifest

Pré-requisito da Camada 1. Decisão de design forte: **explícito, não inferido**.

```rust
const MANIFEST: NodeManifest = NodeManifest {
    type_id: NodeTypeId::of("scene.falloff"),
    /* ... */
    reads_attrs: &[],
    writes_attrs: &[AttrId::FalloffStrength],
    /* ... */
};
```

Habilita: Camada 1, tooltip "este nó toca: X, Y, Z", validação no momento da conexão, detecção de "nó upstream inútil" (escreve attr que nada downstream lê).

### Camada 3 — Attribute tags inline (estilo Nuke channel bars)

Mini-bandeirinhas no rodapé do nó, **sempre visíveis**:

```
.═══════════════════════════════════.
║ Spin                              ║
+-----------------------------------+
| 🌀  Velocidade  2.0 ━━━━●━━━━     |
|                                   |
|  ◇in                       out◇   |
+-----------------------------------+
| lê: [F]   escreve: [X][Y][R]      |   <-- mini chips canônicos
```

Chips: **F** Falloff, **T** Tempo, **C** Cor, **P** Posição, **R** Rotação, **S** Escala, **α** Alpha. Máximo 7 (limite cognitivo). Cinza para "lê", dourado para "escreve". Sem clicar, usuário vê quais atributos o nó toca.

### Camada 4 — Heatmap por instância (para Focus nodes)

Quando nó Focus (Falloff, NumberRange, Modulate) está selecionado, **tinge cada instância no canvas** com gradiente azul→vermelho conforme `falloffStrength`. Padrão Blender Weight Paint + C4D Color Effector + Fields Color.

Modo toggle: botão "Visualizar Peso" no header do nó Focus. Combina com overlay espacial da região (já existe para Falloff).

### Camada 5 — Probe em qualquer fio (Node-RED pattern)

Clicar um fio: pinia mini-readout do valor corrente naquele frame. Mostra:
- Para `value`: número ao vivo + sparkline dos últimos 60 frames
- Para `shape`/`stream`: count + thumb do estado
- Para `trigger`: histograma de fires recentes

Probe é **transitório** (some ao deselecionar) ou **fixado** (Shift+click → permanece). Bret Victor's "immediate connection" puro.

### Camada 6 — Overlays espaciais por nó-tipo

Hoje só Falloff tem `drawFalloffOverlay`. Estender para:
- `DistributePath`/`Circle`/`Grid` → desenha o caminho/grade overlay
- `VectorField` → seta direcional grid
- `Attractor` → ícone centro + raio
- `Pivot`/`Anchor` → cruz no ponto
- `FollowTarget` → linha origem→alvo

Padrão: overlay aparece **só com nó selecionado**, cor do header do nó. Custo trivial por nó.

### Camada 7 — Backdrops coloridos com emoji

Caixas grandes (Nuke pattern) arrastáveis sobre cluster de nós, com emoji + label custom: **"✨ Brilho"**, **"🌊 Onda"**, **"🎯 Foco"**. Move junto. Z-order. Organização visual sem hierarquia.

Trivial. Crianças organizam por cor e emoji muito antes de por convenção textual.

### Camada 8 — Sidebar "Quem escreve / quem lê" + Co-author log

Painel lateral lista cada `AttrId` do projeto. Expandir mostra:

```
▼ FalloffStrength
  Escrito por: Falloff #1 (graph "Hero Idle")
  Lido por: Spin #1 (✓ atinge), Stagger #1 (✓ atinge)
            Bloom #1 (✗ não lê — alerta amarelo)
```

Inversão pedagógica: em vez de "quem o Falloff afeta?", o usuário pergunta "quem usa FalloffStrength?". Combina com aba **"Co-author"** mostrando histórico das chamadas MCP recentes (LLM atividade).

---

## 11. Estrutura de crates

Decisão M3: **3 infra crates + 1 init + 3 category crates** = 7 crates totais (extensível).

```
crates/
├── ph2d-nodegraph-core/           ~800 LOC
│   ├── lib.rs                     (exports)
│   ├── graph.rs                   (Graph, NodeInstance, Edge)
│   ├── manifest.rs                (NodeManifest, PinSpec, ParamSpec)
│   ├── context.rs                 (Context enum extensível)
│   ├── data_type.rs               (DataType enum: float, vec3, shape, trigger…)
│   ├── attr_id.rs                 (AttrId enum — Camada 2)
│   ├── registry.rs                (NodeRegistry: BTreeMap<NodeTypeId, &'static Manifest>)
│   └── node_id.rs                 (NodeTypeId hash FNV-1a — espelha ADR-0027)
│
├── ph2d-nodegraph-eval/           ~600 LOC
│   ├── lib.rs
│   ├── eval_ctx.rs                (EvalCtx<'a> com bumpalo arena — P7)
│   ├── scene_evaluator.rs         (pull lazy DAG, cache por frame)
│   ├── luau_emitter.rs            (Logic — compile-to-Luau D1)
│   ├── wgsl_emitter.rs            (Shader — compile-to-WGSL)
│   ├── probe_sink.rs              (Camada 5)
│   └── hot_reload.rs              (reset+restore Defold-style — Ω5)
│
├── ph2d-nodegraph-editor/         ~2000 LOC (decomposto em mod, HR-18 cap 600/file)
│   ├── lib.rs
│   ├── canvas.rs                  (Vello canvas, pan/zoom)
│   ├── node_paint.rs              (silhuetas, header colorido, pin shapes — §6)
│   ├── wire_paint.rs              (espessura, cor, dashed — §6.5)
│   ├── zone_paint.rs              (If/Repeat/Forever brackets — P4)
│   ├── influence_highlight.rs     (Camada 1)
│   ├── heatmap.rs                 (Camada 4)
│   ├── probe.rs                   (Camada 5)
│   ├── overlay.rs                 (Camada 6 — espacial por nó-tipo)
│   ├── backdrop.rs                (Camada 7)
│   ├── sidebar.rs                 (Camada 8 — co-author + attrs)
│   ├── breadcrumb.rs              (subnet navigation — Ω9)
│   ├── interaction.rs             (drag/connect, espelha ADR-0024 WidgetStore pattern)
│   ├── mcp_actions.rs             (Camada 8 + Ω1 — graph.* tools)
│   └── a11y.rs                    (HR-12 — Node/Pin/Edge → accesskit::Node)
│
├── ph2d-nodegraph-init/           ~100 LOC
│   └── lib.rs                     (register_all_nodes — append-only)
│
├── ph2d-nodes-scene/              ~3000 LOC, 1 file/nó
│   ├── lib.rs                     (pub mod pulsar; pub mod wave; ... — append-only)
│   ├── pulsar.rs                  (1 nó por arquivo, ~60 LOC: const MANIFEST + eval)
│   ├── wave.rs
│   ├── falloff.rs
│   ├── spin.rs
│   ├── wiggle.rs
│   ├── stagger.rs
│   ├── bloom.rs
│   ├── render.rs                  (terminal output)
│   ├── ... (~50 nós seed na v1+v1.5)
│   └── bridges/
│       ├── commit_transform_sim.rs
│       └── material.rs
│
├── ph2d-nodes-shader/             ~1500 LOC (v1.5, M17)
│   └── ... (15 nós seed)
│
└── ph2d-nodes-logic/              ~1500 LOC (v2, M18)
    └── ... (10 nós seed + compiler)
```

**Por que não A (1 crate por nó):** workspace overhead massivo (50+ crates), build time degrada, `register_all_nodes` vira 50 linhas com conflito de merge constante.

**Por que B (agrupado):** 1 arquivo por nó dentro do crate de categoria. 2 agentes em nós diferentes = zero conflito. `pub mod X;` em lib.rs é append-only (mesmo problema, mesma solução do ADR-0027).

**Quando promover para crate próprio:** se um nó precisa de dep pesada (ex: FFT lib em AudioReact), promove para `crates/ph2d-node-audioreact/`. Pattern híbrido — convention-by-discovery permite.

---

## 12. Plano de implementação — marcos M14-M18

### M14 — Infra core + LLM-co-author foundation

**Crates:** `ph2d-nodegraph-core`, `ph2d-nodegraph-eval`, `ph2d-nodegraph-editor`, `ph2d-nodegraph-init`.
**Escopo:**
- Tipos canônicos (Graph, NodeManifest, NodeInstance, Edge, PinSpec, AttrId)
- Pull lazy evaluator com bumpalo (P7 zero-alloc bench)
- Editor canvas Vello + drag/connect (reusa ADR-0024 input pipeline)
- 7 silhuetas + header colorido + pin shapes + wire thickness (§6)
- A11y day-1: `Node`/`Pin`/`Edge` → `accesskit::Node` (HR-12)
- **MCP tools `graph.*`** — 13 ferramentas (§8.2)
- Subnet/composite (Ω9) com asset Handle
- Hot reload reset+restore (Ω5)
- Save format versionado + migrator stub v1→v2 (HR-14)
- Lint-as-spec: `node_pin_color_canonical.rs`, `node_pin_shape_consistency.rs` (Ω8)

**Gate:** abre canvas vazio; usuário arrasta 2 nós (Source + Render) e conecta; MCP `graph.add_node` cria nó programaticamente; subnet collapse/expand funciona; cross-OS replay hash idêntico em fixture sintético.
**Tempo:** ~4-6 semanas. Single agente sequencial.

### M15 — 30 Scene nodes seed + Camadas 1-3 de Inteligibilidade

**Crate:** `ph2d-nodes-scene/` (30 nós seed).
**Escopo:**
- 30 nós motion design canônicos (Pulsar, Onda, Brilho, Falloff, Spin, Wiggle, Stagger, Wave, Move, Transform, Color, ColorArray, NumberRange, Modulate, Noise, Random, Oscillator, Distribute*, Duplicator, Group, Mixer, Bloom, Blur, Glow, HueShift, Sepia, Vignette, Render, Source primitivos)
- Cada nó com `reads_attrs`/`writes_attrs` declarados (Camada 2)
- Camada 1: Influence Highlight on-demand ★★★★★
- Camada 3: attribute tags inline (mini chips F/T/C/P/R/S/α)

**Gate:** tutorial Mini Cavalry "01-estrela-giratoria.md" reprodutível em PH2D editor; LLM via MCP cria grafo "make sprite breathe" em ≤ 5 tool calls.
**Tempo:** ~6-8 semanas. Sequencial inicialmente; paralelizável após primeiros 10 nós (template estabilizado).

### M16 — Camadas 4-8 + 20 Scene nodes adicionais

**Crate:** `ph2d-nodes-scene/` cresce.
**Escopo:**
- 20 nós a mais (Particles emitter, Instancer, Path follow, FollowTarget, LookAt, Snap, Clamp, Delay, Spring, easing curves, time sources)
- Camada 4: heatmap por instância para Focus nodes
- Camada 5: probe em qualquer fio
- Camada 6: overlays espaciais (DistributePath, VectorField, Attractor, Pivot, FollowTarget)
- Camada 7: backdrops com emoji
- Camada 8: sidebar Quem-escreve-quem-lê + co-author log

**Gate:** todos os 5 tutoriais Mini Cavalry reprodutíveis; canvas com 50+ nós permanece legível.
**Tempo:** ~4 semanas. Paralelizável (cookbook estabilizado).

### M17 — Shader context

**Crates:** `ph2d-nodes-shader/`, expande `ph2d-nodegraph-eval` (WgslEmitter).
**Escopo:**
- 15 nós shader (Sample, UV, Combine vec3, Split, Swizzle, Math {Mul/Add/Lerp/Smoothstep}, Noise, Voronoi, Gradient, Step, Fresnel-2D, Mix, Material Output, Custom WGSL)
- Compilador WGSL → naga → MSL/SPIR-V/HLSL
- Preview por nó (Substance Designer pattern) renderizando em mini sphere/plane no canto
- Bridge node `Material` aceita subnet shader; integrado a `Render` em Scene

**Gate:** cooked WGSL asset blake3-hash-stable cross-OS; tonemap shader existente (sprite.wgsl) reproduzível como grafo.
**Tempo:** ~3-4 semanas.

### M18 — Logic context + Luau compiler

**Crates:** `ph2d-nodes-logic/`, `ph2d-nodegraph-eval::luau_emitter`.
**Escopo:**
- 10 nós Logic high-level domain (`OnClick`, `OnFrame`, `OnKey`, `OnCollision`, `If` zona, `Repeat N` zona, `Wait`, `Spawn`, `Trigger Animation`, `Set Property`)
- Compilador `LuauEmitter` (AST→Luau source→mlua bytecode)
- `sim_safe: bool` enforced em compile time para SimWorld target
- Audit log HR-11 em modos Rollback/Lockstep
- Hot reload reset+restore validado (Defold-style herdado)

**Gate:** "On click sprite, spawn 10 particles, after 2s fade out" em 5 nós; LLM via MCP cria grafo equivalente em ≤ 7 tool calls; replay determinístico em fixture multiplayer mockada.
**Tempo:** ~6-8 semanas.

**Total M14-M18:** ~6 meses calendário. Cada marco mergeável independentemente.

---

## 13. Anti-padrões — não faça

Síntese das 6 pesquisas + handoff agente. Cada bullet é cicatriz documentada de outro projeto.

1. ❌ **Tudo no mesmo grafo unificado** (Cables.gl, Quartz Composer). Funciona para creative coders adultos, mata iniciantes.
2. ❌ **Visual scripting genérico** (`+`, `*`, `set`). Godot VisualScript morreu por isso. Só nós domain-dense.
3. ❌ **Dual exec/data wires** (Blueprints). Necessário para AAA games, excessivo para artista/criança. Use zonas.
4. ❌ **Subscrição por nome de atributo silenciosa** (Houdini `@falloff`). Fatal pra iniciante. Subscrição **explícita** via `reads_attrs`/`writes_attrs`.
5. ❌ **NaN silencioso** (Pure Data). Wire inválido = vermelho com tooltip. Valor indefinido = badge "?".
6. ❌ **Pintar tudo** (Tüske: "color as few as possible"). Cor é anotação, não taxonomia automática.
7. ❌ **>7 estados visuais por nó** (Houdini-overload). Restrinja a 3-4 dimensões ativas.
8. ❌ **Atalho de teclado como única revelação** (Blender Ctrl+Shift+Click é genial mas invisível). Sempre botão paralelo.
9. ❌ **Doc por screenshot** (Godot lesson — não escala 6 meses). Invista em projetos-exemplo navegáveis.
10. ❌ **HashMap em runtime simulado** (ADR-0022). Use BTreeMap/Vec.
11. ❌ **Alocação no hot path** (HR-3). Use bumpalo arena.
12. ❌ **GPU compute para output que entra em SimWorld** (HR-5 + ADR-0021). Cai fora de determinismo.
13. ❌ **Editar registry central ao adicionar nó** (ADR-0027). Convention-by-discovery: 1 arquivo + 1 linha em `register_all_nodes`.
14. ❌ **File > 600 LOC ou fn > 200 LOC** (HR-18). Decompõe.
15. ❌ **Async no core além de loader/transport** (HR-1 + §10.1). Sync por default.
16. ❌ **Subnet ausente.** Sem composição o sistema não escala (Godot VisualScript 0.5% adoção).
17. ❌ **MCP exposure retrofit.** API tem que ser MCP-shaped desde dia 1 (Ω1).
18. ❌ **A11y como afterthought** (HR-12). Cada Node/Pin/Edge → accesskit::Node day-1.
19. ❌ **Cross-context wire silenciosa.** Bridges nomeadas obrigatórias.
20. ❌ **Save sem migração** (HR-14). Grafos versionados desde v1.

---

## 14. Riscos e mitigações

| Risco | Probabilidade | Mitigação |
|---|---|---|
| Luau emitter complex demais (D1 L1) | média | Fixture seed com 5 nós Logic emitting Luau valida cedo em M18. Se travar, fallback para L2 (camada acima) — degradação aceita. |
| Vello canvas com 500+ nós performance | média | Bumpalo arena + virtualização de viewport (só renderiza visíveis); HR-3 bench em 1000 nós × 60 frames; Vello é GPU-compute, deve aguentar. |
| MCP tool surface explode | baixa | 13 tools v1 documentadas em §8.2; novas exigem ADR. |
| A11y para wires/pins difícil | média | Pesquisa AccessKit + protótipo M14; se necessário, fallback "linear list view" alternativa. |
| Subnet asset hash mismatch após edit | baixa | Mesmo padrão Material Instance Unreal; testado com fixture 100 ciclos hot reload. |
| LLM gera grafo nonsense | média | `graph.commit` valida estrutura; LLM recebe erro estruturado; humano sempre revisa antes de salvar. |
| Cross-platform replay quebra | baixa | C9 do spike Luau já provou; Logic herda automaticamente; CI matrix Linux+Mac+Windows. |
| HR-3 zero-alloc difícil em probe | baixa | Probe sink usa pre-allocated SmallVec; flush no fim do frame. |
| Determinismo em GPU shader output entrando em SimWorld | alta se descuidado | Compilador rejeita: shader output nunca conecta em socket que escreve em SimWorld; bridge `Commit *` valida `sim_safe`. |
| Time pressure M14-M18 6 meses | média | Marcos são mergeáveis individualmente; se M18 atrasar, v1 ainda ship Scene + Shader (8 meses calendário aceitos). |

---

## 15. Resumo executivo (recap final)

### Em uma frase
O melhor sistema de nós para uma engine 2D já desenhado — **Cavalry × Cursor × Houdini × Godot** — com LLM co-autora de primeira classe, contextos isolados extensíveis, determinismo verificado, e zero-alloc no hot path.

### As 10 decisões canônicas

| ID | Decisão |
|---|---|
| **D1** | Logic compila para Luau bytecode via `mlua::Compiler` (herda C9 cross-platform, GC, hot reload, MCP) |
| **D2** | `Context` enum N-variants extensível; v1 ship **Scene-only**; Shader v1.5; Logic v2; Audio/Physics futuros |
| **D3** | Scene = PresentWorld puro; bridges explícitas (`Commit Transform to Sim`, etc.) para cruzar para SimWorld |
| **D4** | `Asset::Graph` blake3-addressed + `GraphInstance` Component com `param_overrides` (Material Instance pattern) |
| **D5** | Single domain-dense catalog (`Pulsar`, `Onda`, `Falloff` — NÃO `Math.Sin`); `Custom Rust`/`Custom WGSL` como escape hatch |
| **D6** | `sim_safe: bool` per-node-manifest; compilador rejeita não-sim-safe em grafos SimWorld em modo Rollback/Lockstep |
| **Ω1** | **LLM-co-author é foundation M14**, não polish: 13 tools MCP `graph.*` desde dia 1 |
| **Ω9** | **Subnet/composite é fundacional**: Graph referencia outros Graph via Asset::Graph handle; recursão proibida |
| **Ω4** | Evaluator zero-alloc via `bumpalo::Bump` arena reset/frame; HR-3 bench em 1000 nós × 60 frames |
| **Ω7** | A11y day-1: cada Node/Pin/Edge implementa `accesskit::Node` builder; wire navigation via teclado |

### As 10 vitórias visuais

1. **7 silhuetas semânticas** (rect/circle/diamond/cigar/trapezoid×2/tabbed), automáticas por manifest
2. **Header colorido** por categoria funcional (9 cores OKLCH dark-mode-safe)
3. **Pin shapes** (círculo singleton, diamante stream — Blender)
4. **Pin colors** por DataType (convenção emergente Unity/Babylon)
5. **Wire thickness** (fino trigger, grosso valor animado — Pure Data)
6. **Zonas** em vez de exec wires (`If {}`, `Repeat N {}` — Blender)
7. **Bridges** explícitas entre contextos como nós nomeados
8. **Influence highlight on-demand** (clicar nó destaca downstream realmente afetado)
9. **Attribute tags inline** (mini chips F/T/C/P/R/S/α — Nuke channel bars)
10. **Heatmap por instância** para Focus nodes (Blender Weight Paint + C4D Color Effector)

### As 8 camadas de inteligibilidade

1. Influence Highlight on-demand
2. `reads`/`writes` declarativos no NodeManifest
3. Attribute tags inline (mini chips)
4. Heatmap por instância para Focus nodes
5. Probe em qualquer fio (live readout)
6. Overlays espaciais por nó-tipo (DistributePath, VectorField, Attractor…)
7. Backdrops coloridos com emoji
8. Sidebar Quem-escreve-quem-lê + Co-author log

### Os 7 crates da arquitetura

- `ph2d-nodegraph-core` (~800 LOC) — modelo de dados
- `ph2d-nodegraph-eval` (~600 LOC) — evaluators Scene/Shader/Logic
- `ph2d-nodegraph-editor` (~2000 LOC) — UI Vello
- `ph2d-nodegraph-init` (~100 LOC) — append-only register
- `ph2d-nodes-scene` (~3000 LOC) — 50 nós motion design
- `ph2d-nodes-shader` (~1500 LOC) — 15 nós shader (v1.5)
- `ph2d-nodes-logic` (~1500 LOC) — 10 nós + Luau compiler (v2)

### O caminho — 5 marcos / 6 meses

- **M14** (~4-6 sem): infra core + MCP foundation + subnet + A11y + lint-as-spec
- **M15** (~6-8 sem): 30 Scene nodes + Camadas 1-3 inteligibilidade
- **M16** (~4 sem): +20 Scene nodes + Camadas 4-8 inteligibilidade
- **M17** (~3-4 sem): Shader context + 15 nós + WGSL emitter
- **M18** (~6-8 sem): Logic context + 10 nós + Luau emitter

### O reframing central

PH2D não é Cavalry educacional. **É a primeira engine onde LLM autora grafos COM o humano via MCP tools, em três domínios isolados mas paradigma único, com determinismo verificado e zero-alloc no hot path.** Nenhuma outra engine 2D faz isso. Esse é o **competitive moat**, e justifica os 6 meses de M14-M18.

### Próximos passos

1. Extrair deste doc o **ADR-0029** (decisões inegociáveis D1-D6 + Ω1/Ω4/Ω7/Ω9 + M3/M4)
2. Extrair o **plano operacional** `docs/plans/2026-XX-nodegraph.md` (marcos M14-M18 com gates)
3. Extrair a receita **DIRETRIZ §4 "Adicionar node novo"** (cookbook: 1 arquivo + 1 linha em `register_all_nodes`)
4. Ratificar ADR-0029 com Enio
5. Começar M14

---

**Fim do doc canônico.**
