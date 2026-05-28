# 08 — Painter ↔ Vector Bridge (bidirecional)

> Spec dos **bridges entre Painter e Vector Module**. Sucessor unificado de Procreate (raster) + Illustrator (vector) com **transição zero-fricção**.
>
> **ADR ratificador:** ADR-0062 (Painter ↔ Vector bridge).
> **Inovação #3 (vide [`14_inovacoes_extraordinarias.md §14.4`](14_inovacoes_extraordinarias.md)).**

## 8.0 Os 3 bridges

| Bridge | Direção | Detalhe |
|--------|---------|---------|
| **8.1** | Painter brush stroke → vector network | Paint into vector |
| **8.2** | Vector path → Painter brush look | Pattern along path |
| **8.3** | Painter raster layer → vector network | Auto-trace ML |

Plus integração transversal: **Adjustment layers shared** (§8.5) + **Color picker shared** (§8.6).

---

## 8.1 Paint into vector (Painter brush → vector.pencil node)

### 8.1.1 Conceito

Usuário pinta com Painter brush dentro do canvas Vector Module. Cada Painter stroke → automatic Hobby fit → `vector.pencil` path. Resultado: **vetor editável com look pintado**.

### 8.1.2 Pipeline

```
Stylus input (pressure/tilt) [Painter source]
   ↓ [via PH2D_PAINTER_BRIDGE flag]
Painter stamp scheduler (live brush trace preview no canvas)
   ↓ [stylus-up event]
StrokeRecord (Painter's per-stroke history)
   ↓ [conversion]
Hobby fitter (raw samples → cubic Bézier chain)
   ↓
WidthProfile derivation (pressure → width axes)
   ↓
VectorNetwork emit (single new region OR network append)
   ↓
EditLog::AddVertex + AddSegment ops (multi-step transaction)
```

### 8.1.3 Stroke record translation

```rust
fn painter_stroke_to_vector_pencil(stroke: &PainterStrokeRecord) -> VectorNetwork {
    // 1. Extract raw samples
    let samples: Vec<RawSample> = stroke.samples().collect();
    
    // 2. Hobby fit
    let cubics = hobby_fit(&samples, default_hobby_weight());
    
    // 3. Build VectorNetwork
    let mut network = VectorNetwork::new();
    for cubic in cubics {
        let v0 = network.add_vertex(cubic.start, VertexKind::Auto);
        let v1 = network.add_vertex(cubic.end, VertexKind::Auto);
        network.add_segment(v0, v1, TangentsCubic {
            out_at_start: cubic.c1,
            in_at_end: cubic.c2,
        });
    }
    
    // 4. Derive width profile from pressure samples
    let width_profile = WidthProfile {
        base_width: stroke.average_width(),
        pressure_weight: 1.0,  // pressure fully drives width
        taper_start: 0.0,
        taper_end: 0.0,
        contrast: 0.5,
        jitter_amount: 0.0,
    };
    
    // 5. Attach style
    let style_ref = network.add_stroke_style(StrokeStyle {
        width: width_profile,
        color: stroke.color_oklch(),
        cap: StrokeCap::Round,
        join: StrokeJoin::Round,
        miter_limit: 4.0,
        dashes: None,
    });
    for seg in &mut network.segments {
        seg.style_ref = Some(style_ref);
    }
    
    network
}
```

### 8.1.4 Stroke → multiple segments

Hobby fitter produz cubic chain (1 cubic per ~10 input samples). 1 stroke pode virar 5-50 segments dependendo de stroke length + complexity.

### 8.1.5 Pressure curves preserved

Painter `StrokeRecord` tem per-sample pressure. WidthProfile derivation usa best-fit:
- Mean pressure → `base_width`.
- Variance pressure → `pressure_weight` (high variance = more pressure-driven).
- Taper detection (pressure low → high → low) → `taper_start` / `taper_end`.

### 8.1.6 UX integration

Ao trocar para Vector Module quando previously em Painter:
- Modal "Convert Painter strokes em Vector?" se layer ativa Painter raster.
- "Yes" → batch convert all strokes do Painter layer; layer transforma para Vector layer.
- "No" → coexist (Painter raster layer + new Vector layer above).

### 8.1.7 Edge cases

- Stroke muito curto (< 5 samples): converte para single short segment.
- Stroke com pressure 0 (palm rejection failure?): descartado.
- Stroke com loops (figure-8): Hobby handles; cubic chain self-intersects (preserved em VectorNetwork via auto-resolve crossings).

---

## 8.2 Vector with brush look (vector-pattern-along-path consumes Painter brush)

### 8.2.1 Conceito

`vector-pattern-along-path` node (vide [02 §2.2.8](02_geometry_graph.md)) consome qualquer brush do `ph2d-painter-brush` library:

```rust
inputs: &[
    Input::path("path"),
    Input::brush("brush"),  // BrushRef from ph2d-painter-brush
],
```

### 8.2.2 Algoritmo

```rust
fn pattern_along_path(network: &VectorNetwork, brush: &PainterBrush, params: PatternParams) -> RenderTexture {
    // 1. Sample path em N points (per spacing param)
    let mut samples = Vec::new();
    let total_length = network.total_length();
    let n = (total_length / params.spacing).round() as u32;
    for i in 0..n {
        let t = i as f32 / n as f32;
        let pos = network.sample_at(t);
        let tangent = network.tangent_at(t);
        samples.push(StampSample {
            pos,
            tangent,
            pressure: 1.0,  // OR params-driven
            jitter: random_jitter_at(t, params.jitter),
        });
    }
    
    // 2. Brush stamps via Painter brush engine
    let mut texture = create_render_texture(network.bounding_box());
    for sample in samples {
        brush.stamp(&mut texture, sample);
    }
    
    texture
}
```

### 8.2.3 Live edit

Mover vertex em VectorNetwork → re-renderiza brush stamps automaticamente (cache invalidation propaga).

### 8.2.4 Resultado visual

Path traçado parece **pintado a mão** com brush real (pencil_2b, oil_round, etc.), mas é **vetor editável**: move vertex → stamps reposicionam; trocar brush em params → re-render com novo brush.

### 8.2.5 Stamp jitter / scatter

Params expostos via [02 §2.2.8](02_geometry_graph.md): `spacing`, `jitter`, `scatter`. Cada stamp recebe perturbação per-sample (deterministic com seed se HR-5 ativa).

### 8.2.6 Performance

Brush stamps em GPU compute (Painter pattern). Render 100-stamp path ≤ 1 ms cached; re-render after edit ≤ 2 ms.

### 8.2.7 Cache

Output texture cached by (network_hash, brush_id, params_hash). Invalidate on edit upstream.

---

## 8.3 Vectorize raster (auto-trace ML — `vector-auto-trace` node)

### 8.3.1 Comando "Vectorize Layer"

No Painter, command "Vectorize layer" (acessível via menu Image → Vectorize):
1. Painter raster layer → input image.
2. `vector-auto-trace` node executa.
3. Output VectorNetwork inserted em layer Vector novo.

### 8.3.2 3 modos (Linearity Curve pattern)

#### Modo: Sketch

Otimizado para line art / sketches.
- Detection: Sobel + Canny edge detection.
- Skeletonization: thinning algorithm.
- Path tracing: connected component analysis + Bézier fit.

#### Modo: Illustration

Otimizado para color illustration / flat colors.
- Color quantize (k-means N=8 default).
- Boundary extraction per color region.
- Potrace-style polygon → cubic Bézier fit.

#### Modo: Basic Shapes

Otimizado para geometric / logos.
- Primitive detection (Hough transform for lines/circles).
- ML fit para rect/ellipse/poly/star/spiral.
- Output usa `vector-source` primitives (não free-form paths).

### 8.3.3 ML backbone (W12 stretch)

- **SuperSVG** ([arXiv 2406.09794](https://arxiv.org/pdf/2406.09794)) — superpixel decomposition + path refinement. Embed model ~50 MB.
- **LLM4SVG** ([ximinng.github.io/LLM4SVGProject](https://ximinng.github.io/LLM4SVGProject/)) — LLM emits structured SVG. Via MCP call.

Fallback: **Potrace** port (CPU-only, ~500 LOC port, sem ML).

### 8.3.4 Editability preserved

Output VectorNetwork é standard (vertices + segments + regions). Editável downstream do node. User pode mover vertex, trocar fill, apply boolean — tudo funciona como path-desenhado.

### 8.3.5 Performance

- Sketch mode (Sobel + Canny): ≤ 200 ms / 1080p image.
- Illustration mode (color quantize + Potrace): ≤ 1 s.
- Basic Shapes (Hough + ML): ≤ 500 ms.
- Async (UI mostra progress bar).

### 8.3.6 Edge cases

- Image com noise → false-positive edges. Mitigação: pre-blur via gaussian.
- Photos (continuous tone) → tons of segments; auto-trace warns user "this image is too complex; consider Painter raster instead".

---

## 8.4 Bidirectional flow

### 8.4.1 Vector → Painter raster bake

User edita vetor + decide quer raster (e.g., para apply destructive Painter brushes não-vector-compatible):
1. Right-click on vector layer → "Convert to Raster".
2. Render VectorNetwork → bitmap (canvas resolution).
3. Substitute Vector layer com Painter raster layer.
4. **Lossy** (vector → raster perde editability). Warning UI.

### 8.4.2 Painter raster → Vector via auto-trace

§8.3 (vectorize layer command).

### 8.4.3 Coexist mode

Mais comum: ambos coexistem como layers separados no compositor. Vector layer above raster (e.g., line art over painted background) OR raster above vector (e.g., texture over geometric base).

---

## 8.5 Adjustment layers shared

### 8.5.1 12 Painter adjustments aplicáveis a vector layers

Painter introduz 12 adjustment layers (HSB, Color Balance, Curves, Gradient Map, Brightness/Contrast, Gaussian Blur, Motion Blur, Noise, Sharpen, Bloom, Halftone, Chromatic Aberration) — vide [Painter ADR-0045](../architecture/decisions/0045-adjustment-layers.md).

Vector Module reusa via `vector-adjustment` node:

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

### 8.5.2 Apply pipeline

1. VectorNetwork renderizada to intermediate texture.
2. Adjustment shader applied to texture (reuses Painter compute shaders).
3. Output texture composed with rest of scene.

### 8.5.3 Compositor recompose strategy

Dirty rect propagation — só re-render da region afetada pelo adjustment. Cache results by (vector_hash, adjustment_kind, params_hash).

---

## 8.6 Color picker shared

### 8.6.1 Reuso `ph2d-painter-color`

Vector Module Inspector panel reusa `ph2d-painter-color::ClassicPicker` (e.g., 5 modos Disc / Classic / Harmony / Value / Palettes).

### 8.6.2 OKLCH internal (HR-1 + Painter convention)

Cores em OKLCH internamente (perceptual uniform). Display sRGB / Display P3 detectado per-device. Mesma semantics que Painter (vide [Painter ADR-0051](../architecture/decisions/0051-color-profile-pipeline.md)).

### 8.6.3 Palette interop

`.ph2d-palette` (postcard) compartilhado entre Painter + Vector Module. User pode importar Procreate `.swatches`, Adobe `.ase`, GIMP `.gpl` — todos accessible em ambos modules.

---

## 8.7 Conclusão — sucessor unificado Procreate + Illustrator

Bridge 1 (paint into vector) + Bridge 2 (vector com brush look) + Bridge 3 (vectorize raster) + Adjustment layers shared + Color picker shared = **transição zero-fricção** entre raster (Painter) e vector (Vector Module).

Nenhum competitor entrega os 3 bridges + shared infra. Linearity Curve tem só Bridge 3 (auto-trace). Procreate tem zero. Affinity Designer tem dual persona mas sem bridge nativo brush ↔ vector.

---

## Fim do Painter bridge spec

3 bridges + shared adjustment + shared color picker. Resultado: experiência unificada de arte digital, com vetor e raster como faces de mesma moeda.

**Next:** [`09_scripting_mcp.md`](09_scripting_mcp.md) (Luau + MCP + LLM-as-graph-node).
