# ADR-0062 — Painter ↔ Vector bridge bidirecional + `ph2d-brush-traits` decoupling

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [ADR-0043 — Painter contract](0043-painter-contract.md), [ADR-0044 — Brush Engine GPU](0044-brush-engine-gpu.md), [ADR-0056 — Vector Network](0056-vector-network-data-model.md), [ADR-0058 — Geometry graph](0058-vector-geometry-graph.md), [ADR-0067 — brush-traits decoupling](0067-brush-traits-decoupling.md) (irmã).
**Spec normativa:** [`docs/Vector Module/08_painter_bridge.md`](../../Vector%20Module/08_painter_bridge.md).
**Tags:** vector, painter, wave-0, contract, bridge, bidirectional

---

## 1. Contexto

Inovação #3: **Painter ↔ Vector bridge bidirecional**. Sucessor unificado de Procreate (raster) + Illustrator (vector) com **transição zero-fricção**. 3 bridges:

1. **Paint into vector**: Painter brush stroke → Hobby fitter → vector.pencil path editable.
2. **Vector com brush look**: `vector-pattern-along-path` consome Painter brush library.
3. **Vectorize raster (auto-trace ML)**: comando "Vectorize Layer" no Painter chama `vector-auto-trace` node (3 modos Sketch/Illustration/Basic Shapes).

L6F2 Antigravity 3ª iter absorbed: `ph2d-brush-traits` crate desacoplado (ADR-0067) elimina circular dependency Painter↔Vector.

---

## 2. Decisão

### 2.1 Bridge 1: Paint into vector (W12 T12.1)

```
Stylus input
  → Painter stamp scheduler (live brush trace preview)
  → stylus-up event
  → StrokeRecord (Painter)
  → Hobby fitter (Hobby's algorithm, minimum curvature variation)
  → WidthProfile derivation (pressure → width axes)
  → VectorNetwork emit (Vector Module via EditLog::AddSegment * N)
```

Result: vetor editável com look pintado.

Algorithm details em spec §8.1.

### 2.2 Bridge 2: Vector com brush look (W4 T4.5 + W8 T8.1)

`vector-pattern-along-path` node consome qualquer brush via `ph2d-brush-traits::BrushEngine`:

```rust
inputs: &[
    Input::path("path"),
    Input::brush_ref("brush"),  // ph2d-brush-traits::BrushRef
],
params: &[
    Param::f32("spacing", 1.0, 0.01..=10.0),
    Param::f32("jitter", 0.0, 0.0..=1.0),
    Param::f32("scatter", 0.0, 0.0..=1.0),
],
```

Sample path em N points (per spacing) → emit StampSpec per sample → BrushEngine renders stamps. Output texture cached por `(network_hash, brush_id, params_hash)`.

### 2.3 Bridge 3: Vectorize raster (auto-trace ML) (W12 T12.3)

`vector-auto-trace` node — 3 modos:

| Modo | Algorithm | Target |
|------|-----------|--------|
| **Sketch** | Sobel + Canny edge detection + skeletonization + Bézier fit | Line art / sketches |
| **Illustration** | K-means color quantize (N=8 default) + Potrace-style polygon → cubic | Flat colors |
| **Basic Shapes** | Hough transform + ML primitive fit | Geometric / logos |

Backbone ML: SuperSVG embed (~50 MB) OR LLM4SVG via MCP (vide ADR-0061) OR Potrace pure-Rust (fallback CPU-only).

Output VectorNetwork editável downstream (mesma data structure de paths drawn).

### 2.4 Adjustment layers shared (Painter ADR-0045)

12 Painter adjustments aplicáveis a vector layers via `vector-adjustment` node:

```rust
inputs: &[Input::path("input")],
params: &[
    Param::enum_var("kind", &[
        "HSB", "ColorBalance", "Curves", "GradientMap",
        "BrightnessContrast", "GaussianBlur", "MotionBlur",
        "Noise", "Sharpen", "Bloom", "Halftone", "ChromaticAberration",
    ], "HSB"),
    Param::AdjustmentParams { /* typed per-kind */ },
],
```

Apply pipeline:
1. VectorNetwork → intermediate texture.
2. Adjustment shader applied (reuses Painter compute shaders).
3. Output composed with rest of scene.

### 2.5 Color picker shared (`ph2d-painter-color` reuso)

Vector Module Inspector panel consome `ph2d-painter-color::ClassicPicker` (5 modes Disc/Classic/Harmony/Value/Palettes). Cores OKLCH internally (HR-1 + Painter ADR-0051 convention).

Palette interop `.ph2d-palette` postcard compartilhado entre Painter + Vector Module.

### 2.6 ADR-0067 brush-traits decoupling — crítico para evitar circular dep (L6F2 Antigravity 3ª iter)

`crates/ph2d-brush-traits/` (novo crate W1 T1.1b) expõe contratos abstratos:

```rust
pub struct BrushRef { /* opaque handle */ }
pub struct StampSpec { pos, tangent, pressure, tilt, jitter }
pub trait BrushEngine {
    fn stamp(&self, target: &mut RenderTexture, sample: StampSpec);
    fn brush_handle(&self) -> BrushRef;
}
```

Painter `ph2d-painter-brush` **implementa** `BrushEngine`. Vector `ph2d-node-vector-pattern-along-path` **consome** trait. **Zero circular dep**: ambos crates importam linearly `ph2d-brush-traits`, nem um importa o outro.

### 2.7 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| Auto-trace modes | **3** (Sketch / Illustration / Basic Shapes) | Paridade Linearity Curve |
| Adjustment kinds shared | **12** (mesmo Painter ADR-0045) | Reuso completo |
| Bridge directions | **3** (paint-in / brush-look / vectorize-out) | Cobre fluxo bidirecional |
| `BrushEngine` trait methods | **2** (stamp + brush_handle) | Minimal surface decoupling |
| Brush stamps cache LRU | **100 MB** | Balance perf vs memory |

---

## 3. Consequências

### 3.1 Positivas

- **Sucessor unificado Procreate + Illustrator** — única ferramenta com transição zero-fricção raster ↔ vector.
- **Adjustment layers + color picker shared** elimina duplicação Painter/Vector.
- **ADR-0067 decoupling** elimina circular dep risk Painter↔Vector (L6F2 catch certeiro Antigravity 3ª iter).
- **Auto-trace ML 3 modos** competitivo vs Linearity Curve / Adobe Illustrator Image Trace.

### 3.2 Negativas

- **Hobby fitter computational cost** ~50-200 µs per stroke. Aceptable em commit (não hot-path).
- **ML embed model (~50 MB)** se SuperSVG embedded; OR LLM external dependency. Trade-off escolhido em W12 implementation.
- **Adjustment layer recomposition** adiciona cost compositor; cached por dirty rect propagation (mesma estratégia Painter).

### 3.3 Neutras

- Brush stamps cache 100 MB memory overhead.

---

## 4. Alternativas consideradas

### 4.1 Direct dep Painter ↔ Vector (rejeitada — circular L6F2)

Importar `ph2d-painter-brush` direto em Vector node. **Por que rejeitada**: cria circular dep se Painter algum dia consumes Vector (Vectorize Layer). ADR-0067 trait decoupling mitiga.

### 4.2 Sem auto-trace (rejeitada — UX gap)

Pular auto-trace. **Por que rejeitada**: gap vs Linearity Curve / Illustrator; Painter→Vector pipeline incompleto.

### 4.3 ML model only (rejeitada — Potrace fallback necessário)

SuperSVG OR LLM4SVG sem fallback. **Por que rejeitada**: offline scenarios + low-end devices precisam pure-CPU. Potrace fallback obrigatório.

### 4.4 Adjustment kinds Vector-specific (rejeitada — duplication)

Recriar 12 adjustments. **Por que rejeitada**: Painter ADR-0045 já specs; shared via Painter compute shaders evita duplication.

---

## 5. Implementação (Wave 4/8/12)

- **T1.1b** (W1): `ph2d-brush-traits` crate (ADR-0067).
- **T4.5** (W4): `vector-pattern-along-path` node (consume `ph2d-brush-traits`).
- **T8.1** (W8): Brush library integration end-to-end.
- **T12.1-T12.3** (W12): 3 bridges complete (paint-into-vector + vector-with-brush-look + auto-trace).

Gates ativos: `painter_vector_bridge_no_circular_dep` + `auto_trace_3_modes` + `adjustment_layers_shared_correctness`.

---

## 6. Referências

- Spec normativa: [`docs/Vector Module/08_painter_bridge.md`](../../Vector%20Module/08_painter_bridge.md) (320 linhas).
- Hobby's algorithm: <http://hz2.org/blog/hobby_curve.html>
- SuperSVG paper: <https://arxiv.org/pdf/2406.09794>
- Painter ADR-0043 + ADR-0044 + ADR-0045 (sub-contratos compartilhados).
- ADR-0067 brush-traits (irmã).
