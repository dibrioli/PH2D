# 02 — Geometry Graph (domain `vector` no `ph2d-nodegraph`)

> Spec técnico do **graph procedural** de operações geométricas. Domain `vector` adicionado ao `ph2d-nodegraph` (irmão de `motion` que já existe). **18 nodes canon** (+1 Trim Path absorvido Antigravity L5F2 2ª iteração 2026-05-28), drop-crate fan-out com consolidação seletiva (32 crates totais; vide [17 §24](17_plano_de_implementacao.md)), modifier stack ortogonal (Cavalry-inspired). Toda operação destrutiva do Illustrator vira **nó vivo no graph** — animável, scriptável, replayável.
>
> **ADRs ratificadores:** ADR-0058 (Vector geometry graph) + ADR-0039 (Nodegraph contract freeze; possível cap-bump amendment).
> **Spec gêmeo:** [`01_data_model.md`](01_data_model.md) (VectorNetwork sobre o qual os nodes operam).

## 2.1 Domain `vector` no `ph2d-nodegraph`

### 2.1.1 Posicionamento

O `ph2d-nodegraph` já tem domain `motion` (3 crates Stateful: motion-grid, motion-clone, motion-transform). Vector é o segundo domain a wide fan-out.

```
ph2d-nodegraph (core engine)
├── domain: debug (debug-const, debug-wave)
├── domain: motion (motion-grid, motion-clone, motion-transform)
└── domain: vector  ← NOVO em W3+
    ├── vector-source (consolidado 5 primitives)
    ├── vector-boolean
    ├── vector-offset
    ├── vector-outline-stroke
    ├── vector-roughen
    ├── vector-twist
    ├── vector-bend-path
    ├── vector-pattern-along-path
    ├── vector-scatter
    ├── vector-width-profile
    ├── vector-hatch
    ├── vector-mirror
    ├── vector-corner-round
    ├── vector-warp
    ├── vector-recolor
    ├── vector-llm-shape (W13)
    └── vector-luau-script (W4 / W9)
```

### 2.1.2 Contrato congelado (ADR-0039) + carrier de geometria (ADR-0058-amendment-1)

Caps de NodeOp / OpResolver / NodeManifest (= 2 / 1 / 8) continuam válidos para domain `vector` — **intactos** (gate `architecture_contract_surface` verde). O carrier de geometria vive nos internos *ungated* do cook, não na superfície que o nó implementa.

**Carrier (real):** o output de um nó vetorial (`VectorNetwork`) trafega na edge pelo canal opaco type-erased do cook — `CookValue::Opaque(Arc<dyn Any + Send + Sync>)` — com acessores tipados na crate de borda `ph2d-vector-graph`: `VectorEvalExt::{emit_network, input_network}` + a constante `VECTOR_PORT` (`Domain::Vector` / `Clock::Static`). O substrato `ph2d-nodegraph` permanece domain-agnostic (zero deps; não conhece `VectorNetwork`). Detalhe normativo: **[ADR-0058-amendment-1](../architecture/decisions/0058-amendment-1.md)**.

**Param vocabulary (real):** `ParamSpec` congelado é **`f32`-only**. Params de seleção/contagem (`kind`, `sides`, `turns`, `samples_per_turn`) viajam como **discriminante/contagem `f32`** via `param_as_count` (conversão total/saturating já no substrato). Vocabulário tipado (`u32`/`String`/`Color`/`Path`-ref) é refinamento futuro **fora** do contrato atual (amendment §2.4) — não é pré-requisito dos nós W3/W4.

**Clock:** geometria estática usa `Clock::Static` ("cooked once, re-cook on param edit"). Não existe `Clock::None`.

Effect kinds: `Pure` (most nodes), `Temporal` (animação), `Stateful` (cache-aware multi-frame).

---

## 2.2 Os 17 nodes canon

Para cada node: ID + manifest + params + algorithm + complexity.

### 2.2.1 `vector-source` (consolidado 5 primitives)

**Crítica A absorvida (vide [README §11.B](README.md)):** 5 primitives (rect / ellipse / polygon / star / spiral) consolidados num único crate multi-variant — não 5 crates triviais.

**API real (implementada, `crates/ph2d-node-vector-source`):** o `NodeManifest` usa os 8 campos congelados; o output é `VECTOR_PORT` (não há `Output::path`); params são `f32` (não há `Param::enum_var/u32`); `eval` **emite** pelo canal opaco (não retorna `Result<VectorNetwork>`). `kind` é discriminante `f32` (`0`=Rect … `4`=Spiral).

```rust
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.source"),
    name: "vector.source",
    inputs: &[],
    outputs: &[PortSpec { name: "out", ty: VECTOR_PORT }], // Domain::Vector / Clock::Static
    effect: Effect::Pure,
    clock: Clock::Static,
    params: &[ // f32-only: kind=discriminante 0..=4; width/height/inner_ratio/rotation; sides/turns/samples_per_turn=contagens
        ParamSpec { name: "kind", default: 0.0 },
        ParamSpec { name: "width", default: 100.0 },
        ParamSpec { name: "height", default: 100.0 },
        ParamSpec { name: "sides", default: 6.0 },
        ParamSpec { name: "inner_ratio", default: 0.4 },
        ParamSpec { name: "turns", default: 3.0 },
        ParamSpec { name: "samples_per_turn", default: 24.0 },
        ParamSpec { name: "rotation", default: 0.0 },
    ],
    lowerings: &[LoweringKind::Cpu],
};

// `kind` → 1 dos 5 geradores de `ph2d-vector-doc::primitives` (rect/ellipse/
// polygon/star/spiral), depois snap Q16.16 (cross-OS bit-identical), emit opaco.
fn eval(&self, ctx: &mut EvalCtx<'_>) {
    let net = source_network(
        param_as_count(ctx.param("kind"), KIND_MAX),
        ctx.param("width"), ctx.param("height"),
        param_as_count(ctx.param("sides"), MAX_SIDES) as u32,
        ctx.param("inner_ratio"), ctx.param("turns"),
        param_as_count(ctx.param("samples_per_turn"), MAX_SAMPLES_PER_TURN) as u32,
        ctx.param("rotation"),
    );
    ctx.emit_network(net); // VectorEvalExt → CookValue::Opaque(Arc<VectorNetwork>)
}
```

**Complexity:** O(1) per primitive (constant vertex count, exceto spiral = O(turns·samples)).

### 2.2.2 `vector-boolean` — 9 variants

```rust
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.boolean"),
    name: "Boolean",
    inputs: &[Input::path("a"), Input::path("b")],
    outputs: &[Output::path("network")],
    effect: Effect::Stateful,  // result cached by hash
    clock: Clock::None,
    params: &[
        Param::enum_var("op", &[
            "Union", "Subtract", "Intersect", "Exclude",
            "Divide", "Trim", "Merge", "Crop", "Outline",
        ], "Union"),
    ],
    lowerings: Lowerings::Stateful(eval),
};
```

**Algorithm**: pipeline draft+reconcile (resolve crítica C Antigravity):
1. **Draft (CPU naive cubic clipping)** — instantâneo, ≤ 1 ms. Usado em hot-path slider drag.
2. **SDF Hybrid (GPU compute)** — real-time interactive + gameplay morphing (vide [`08_painter_bridge.md`](08_painter_bridge.md) §SDF + [ADR-0065](../architecture/decisions/0065-vector-sdf-hybrid.md)).
3. **Linesweeper exato (async)** — background worker debounced. Topology canônica.

**Cache:** hash by `(input_a_hash, input_b_hash, op)` → result network. LRU 50 MB.

**Complexity:** Linesweeper O((N+K) log N) onde K = intersections. SDF compute O(pixels).

### 2.2.3 `vector-offset` (parallel / contour)

```rust
inputs: &[Input::path("input")],
params: &[
    Param::f32("offset", 10.0, -1000.0..=1000.0),  // negative = inset
    Param::enum_var("join", &["Round", "Bevel", "Miter"], "Round"),
    Param::f32("miter_limit", 4.0, 1.0..=20.0),
],
```

**Algorithm:** GPU Euler-spiral approximation (Levien+Uguray 2024) ou CPU fallback via kurbo `Offset`. Live edit no slider.

### 2.2.4 `vector-outline-stroke`

Converte stroke (line) em filled path (region).

```rust
inputs: &[Input::path("input")],
params: &[
    Param::f32("width", 4.0, 0.1..=200.0),
    Param::enum_var("cap", &["Butt", "Round", "Square"], "Round"),
    Param::enum_var("join", &["Miter", "Round", "Bevel"], "Round"),
],
```

### 2.2.5 `vector-roughen`

Perturbação organic parametrizada.

```rust
params: &[
    Param::f32("frequency", 1.0, 0.1..=10.0),
    Param::f32("amplitude", 5.0, 0.0..=100.0),
    Param::f32("smoothness", 0.5, 0.0..=1.0),
    Param::u32("seed", 0, 0..=u32::MAX),  // determinismo opt-in
],
```

**Algorithm:** subdivide each segment in N samples, displace via simplex noise sampled at sample position × frequency. `seed` makes deterministic.

### 2.2.6 `vector-twist`

```rust
params: &[
    Param::f32("angle", 0.0, -360.0..=360.0),
    Param::vec2("center", Vec2::ZERO),
    Param::f32("falloff_radius", 100.0, 0.0..=10000.0),
],
```

### 2.2.7 `vector-bend-path`

Bend any shape along envelope path (Illustrator Art brush equivalente).

```rust
inputs: &[Input::path("input"), Input::path("envelope")],
params: &[
    Param::f32("stretch", 1.0, 0.1..=10.0),
],
```

### 2.2.8 `vector-pattern-along-path`

Distribui pattern ao longo do path. **Bridge com Painter** — consome `ph2d-painter-brush` library (vide [`08_painter_bridge.md`](08_painter_bridge.md) §8.2).

```rust
inputs: &[
    Input::path("path"),
    Input::brush("brush"),  // BrushRef from ph2d-painter-brush
],
params: &[
    Param::f32("spacing", 1.0, 0.01..=10.0),
    Param::f32("jitter", 0.0, 0.0..=1.0),
    Param::f32("scatter", 0.0, 0.0..=1.0),
],
```

### 2.2.9 `vector-scatter`

Duplicate + distribute (radial / grid / random / along-path).

```rust
inputs: &[Input::path("input"), Input::path("target_optional")],
params: &[
    Param::enum_var("mode", &["Radial", "Grid", "Random", "AlongPath"], "Radial"),
    Param::u32("count", 6, 1..=1000),
    Param::f32("radius", 100.0, 0.0..=10000.0),  // radial
    Param::vec2("spacing", Vec2::new(50.0, 50.0)),  // grid
    Param::u32("seed", 0, 0..=u32::MAX),  // random
],
```

### 2.2.10 `vector-width-profile`

1D variable-font-style axes para width.

```rust
inputs: &[Input::path("input")],
params: &[
    Param::f32("base_width", 4.0, 0.1..=200.0),
    Param::f32("pressure_weight", 1.0, 0.0..=1.0),
    Param::f32("taper_start", 0.0, 0.0..=1.0),
    Param::f32("taper_end", 0.0, 0.0..=1.0),
    Param::f32("contrast", 0.5, 0.0..=1.0),
    Param::f32("jitter", 0.0, 0.0..=1.0),
],
```

### 2.2.11 `vector-hatch`

Parametric hatch fill (Inkscape Hatch Fill equivalente).

```rust
inputs: &[Input::region("input")],
params: &[
    Param::f32("angle", 45.0, 0.0..=180.0),
    Param::f32("spacing", 5.0, 0.5..=100.0),
    Param::f32("stroke_width", 0.5, 0.1..=10.0),
    Param::enum_var("pattern", &["Lines", "CrossHatch", "Dots", "Wave"], "Lines"),
],
```

### 2.2.12 `vector-mirror`

Live symmetry (V / H / Quadrant / Radial).

```rust
inputs: &[Input::path("input")],
params: &[
    Param::enum_var("kind", &["Vertical", "Horizontal", "Quadrant", "Radial"], "Vertical"),
    Param::u32("radial_count", 4, 2..=16),  // radial mode
    Param::vec2("axis_center", Vec2::ZERO),
],
```

### 2.2.13 `vector-corner-round`

Per-node rounding (live, Inkscape Corner Rounding equivalente).

```rust
inputs: &[Input::path("input")],
params: &[
    Param::f32("radius", 5.0, 0.0..=100.0),
    Param::bool("smart_radius", true),  // adapta a vertex spacing
    Param::f32("threshold_angle", 90.0, 0.0..=180.0),  // só corners < threshold
],
```

### 2.2.14 `vector-warp`

Perspective / mesh warp / liquify-style (live).

```rust
inputs: &[Input::path("input")],
params: &[
    Param::enum_var("kind", &["Perspective", "Mesh", "Liquify"], "Perspective"),
    Param::mat3("perspective_matrix", Mat3::IDENTITY),  // perspective
    // mesh + liquify usam handles editáveis no canvas
],
```

### 2.2.15 `vector-recolor`

Color harmony rules across subgraph (Illustrator Recolor Artwork equivalente).

```rust
inputs: &[Input::path("input")],
params: &[
    Param::enum_var("harmony", &[
        "Complementary", "Triadic", "Analogous",
        "SplitComplementary", "Tetradic", "Monochromatic",
    ], "Analogous"),
    Param::color("base_color", Color::rgb(0.5, 0.5, 0.5)),
    Param::f32("variance", 0.2, 0.0..=1.0),
],
```

### 2.2.16-bis `vector-trim-path` ✨ (NEW — Antigravity L5F2 2ª iteração 2026-05-28)

Trim path — corta dinamicamente o percentual de start/end de stroke. **Essencial** para motion designers (After Effects / Cavalry / Rive todos têm; faltava no spec original — gap crítico em UX persona B).

```rust
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.trim_path"),
    name: "Trim Path",
    inputs: &[Input::path("input")],
    outputs: &[Output::path("network")],
    effect: Effect::Pure,  // animable
    clock: Clock::None,
    params: &[
        Param::f32("trim_start", 0.0, 0.0..=1.0),   // % start
        Param::f32("trim_end", 1.0, 0.0..=1.0),     // % end
        Param::f32("offset", 0.0, -1.0..=1.0),       // rotational offset
        Param::enum_var("mode", &["Sequential", "Individually"], "Sequential"),
    ],
    lowerings: Lowerings::Pure(eval),
};
```

**Algorithm**: arc-length parameterize input network; emit subset entre `[trim_start + offset, trim_end + offset]` (wraps com modulo se offset crossover). Mode `Individually` aplica trim per-region independente.

**Use cases canônicos**:
- "Linha que se desenha" animation: trim_end animado 0→1 em 2 seg.
- "Apagar linha": trim_start animado 0→1 em sequência.
- Logo intro: trim em órbita via motion-wave.

**Complexity**: O(N segments) — trivial. Performance < 100 µs / 1000-segment path.

### 2.2.18 `vector-llm-shape` ✨

LLM-driven shape node (vide [`09_scripting_mcp.md`](09_scripting_mcp.md) §9.5).

```rust
inputs: &[Input::optional_image("style_ref")],
params: &[
    Param::string("prompt", "spiral with 8 arms golden ratio"),
    Param::u64("seed", 42),
    // Output: VectorNetwork via LLM4SVG semantic tokens.
],
effect: Effect::Stateful,  // result cached; re-prompt = re-bake
```

### 2.2.19 `vector-luau-script`

Custom modifier em Luau (HR-10).

```rust
inputs: &[Input::path("input")],
params: &[
    Param::script("script", "
        function modify(network, dt)
            -- arbitrary Luau code
            return modified_network
        end
    "),
],
```

---

## 2.3 Modifier stack pattern (Cavalry-inspired)

### 2.3.1 Stack semantics

Cada layer no `ph2d-layer-stack` (ou stack equivalente em `ph2d-editor-core`) pode ter **N modifiers em ordem** (não simultaneously — sequential pipeline). Modifier output vira input do próximo.

```
source: vector-source(rect)
  → vector-corner-round(radius=8)
  → vector-roughen(amplitude=2, frequency=5)
  → vector-recolor(harmony="Complementary")
  → output: final VectorNetwork rendered
```

### 2.3.2 Reorder live

User pode arrastar modifier para cima/baixo no stack panel — re-render imediato (cache invalidated apenas a partir do modifier movido).

### 2.3.3 Modifier toggle

Cada modifier tem checkbox "enabled" — disable bypass o node, useful para A/B comparison.

---

## 2.4 Cavalry taxonomy: Generators / Modifiers / Behaviors / Falloffs / Duplicators

### 2.4.1 Generators

Nodes que produzem VectorNetwork from scratch.
- `vector-source` (5 primitives)
- `vector-llm-shape`

### 2.4.2 Modifiers

Nodes que transformam input VectorNetwork.
- `vector-roughen`, `vector-twist`, `vector-bend-path`, `vector-corner-round`, `vector-warp`, `vector-offset`, `vector-outline-stroke`, `vector-mirror`, `vector-width-profile`, `vector-hatch`, `vector-recolor`, `vector-luau-script`.

### 2.4.3 Combiners

Nodes que combinam múltiplos inputs.
- `vector-boolean` (2 inputs → 1 output).
- `vector-pattern-along-path` (path + brush → painted path).

### 2.4.4 Distributors / Duplicators

Nodes que distribuem cópias.
- `vector-scatter`.

### 2.4.5 Falloffs (sub-system)

Falloffs **não são nodes per se** — são **sub-graph que modula param de outro modifier**. E.g., `radial_falloff(center, radius)` modula `vector-roughen.amplitude` espacialmente — vertices perto do center recebem mais amplitude.

Implementação: cada Modifier expõe params como animable; falloff é animator que avalia per-vertex (não per-frame). Documentado em [`06_animation.md`](06_animation.md).

---

## 2.5 Falloff system

### 2.5.1 Tipos

| Type | Description | Usage |
|------|-------------|-------|
| `Radial` | Distance from center → 0..1 | "Roughen mais próximo do centro" |
| `Linear` | Project onto axis → 0..1 | "Width taper ao longo de Y" |
| `Noise` | Simplex noise sample → 0..1 | "Jitter random spatial" |
| `Shape` | Distance from reference path → 0..1 | "Modify só dentro de region X" |

### 2.5.2 Aplicação

```
modifier: vector-roughen
  param: amplitude
    type: f32
    base: 5.0
    falloff: Radial(center=vec2(0,0), radius=100.0, curve=cubic-ease-out)
```

→ amplitude per-vertex = `5.0 × falloff.sample(vertex_pos)`.

### 2.5.3 Performance

Falloff eval per-vertex em CPU (paraleliza via rayon se >1k vertices). Cached por hash do falloff params + network hash.

---

## 2.6 Cache strategy

### 2.6.1 Hash key

```
cache_key = blake3(
    node_id +
    serialized_params +
    input_hash_a +
    input_hash_b +
    ...
)
```

### 2.6.2 Cache levels

- **L1 (in-memory hashmap)**: 50 MB LRU.
- **L2 (on-disk)**: `~/.cache/ph2d/vector-graph/<hash>.postcard`. 1 GB cap.

### 2.6.3 Invalidation

- Edit `Param::*` em node → cache invalidate apenas o output deste node + downstream.
- Edit upstream → propaga invalidation pelo dirty propagation.

### 2.6.4 Animation hot loop

Quando param animado em 60 FPS, cache hit é improvável (cada frame = novo hash). **Mitigação**: para Stateful nodes (boolean), pipeline draft+reconcile — frame N usa SDF GPU draft; frame em commit (mouse-up) usa Linesweeper exato + cache result.

---

## 2.7 Custom modifier em Luau (`vector-luau-script`)

### 2.7.1 API exposta a Luau

```lua
-- Available in modify():
network.vertices  -- array of {id, pos, kind}
network.segments  -- array of {id, start, end, tangents}
network.regions   -- array of {id, segments, winding}

ph2d.vector.add_vertex(network, pos, kind)
ph2d.vector.move_vertex(network, id, new_pos)
ph2d.vector.add_segment(network, start, end, tangents)
ph2d.vector.compute_bbox(network)
ph2d.vector.sample_at(network, t)  -- t in [0..1] along total length
```

### 2.7.2 Sandbox

- Luau strict mode (ADR-0019).
- Sem `os.execute`, `io.*` (HR-8).
- Timeout 5 segundos (kill se exceeds).

### 2.7.3 Determinismo

Quando network deterministic, Luau roda em fixed-point mode (ADR-0019 §8 + HR-5 + HR-16).

---

## 2.8 LLM node (`vector-llm-shape`)

Detalhe em [`09_scripting_mcp.md §9.5`](09_scripting_mcp.md). Resumo:
- Prompt → LLM emite LLM4SVG semantic tokens → parser converte to VectorNetwork.
- Editable downstream (slider em outro modifier afeta output do LLM!).
- Re-promptable.

---

## 2.9 Determinismo cascading

**Quando upstream node é determinístico** (`motion-wave` em SimWorld), **vector output também é** (HR-5).

Implementação:
- Cada node declara `effect.deterministic_capable: bool` no manifest.
- Quando `VectorNetwork::deterministic=true`, toda chain upstream precisa ser deterministic_capable, ou eval falha com error.
- Test: `tests/determinism/vector_graph_cascading.rs`.

---

## 2.10 Pegadinhas (memory feedback)

### 2.10.1 Param via `ctx.param("nome")`

Sempre via `ctx.param(...)`, **nunca** `MANIFEST.params[..].default` (lê static, ignora overrides).

```rust
// ❌ Wrong
let radius = MANIFEST.params[0].default_f32();

// ✓ Right
let radius = ctx.param_f32("radius")?;
```

### 2.10.2 Cap counts via `param_as_count`

Nodes que alocam baseados em count param devem usar `ctx.param_as_count(name, max)` (cap protege contra OOM em adversarial input).

```rust
let count = ctx.param_as_count("count", 10_000)?;  // cap 10k mesmo se user pede 100k
let mut buf = Vec::with_capacity(count);
```

### 2.10.3 SmallVec inline budget

Nodes que retornam paths simples (rect → 4 vertices) usam SmallVec inline. Não use Vec direto.

---

## 2.11 Performance gates

### 2.11.1 Graph com 50+ nodes < 3.5 ms

`tests/budget/vector_graph_50_nodes.rs` — fixture com 50 nodes encadeados, render frame budget medido. Falha se > 3.5 ms.

### 2.11.2 Deeper nodes off-thread

Nodes pesados (Boolean Linesweeper exato, LLM call) rodam em rayon thread pool. Frame thread continua live com draft preview.

### 2.11.3 Worst-case dimensioning

- Network max segments: 100k (caps em edit_log).
- Boolean: 50k segments per side em Linesweeper exato (acima disso, force SDF mode only).
- Scatter: max 1k duplicates.
- LLM: max 1 call per frame (rate limited).

---

## Fim do Geometry Graph

17 nodes canon fan-out drop-crate (DIRETRIZ §3.A). Modifier stack ortogonal. Cache by hash robusto. Determinismo cascading via traits + opt-in.

**Next:** [03_renderer.md](03_renderer.md) (Vello + GPU stroke + Linesweeper + SDF Hybrid pipeline).
