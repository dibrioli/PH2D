# ADR-0066 — Variable Font Glyph as Vector Network (typography generativa)

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [HR-15 — i18n via Fluent](../../SKILL_Stack_PH2D_Definitiva.md), [ADR-0056 — Vector Network](0056-vector-network-data-model.md), [ADR-0058 — Geometry graph](0058-vector-geometry-graph.md).
**Spec normativa:** [`docs/Vector Module/14_inovacoes_extraordinarias.md §14.7`](../../Vector%20Module/14_inovacoes_extraordinarias.md).
**Tags:** vector, wave-0, contract, typography, variable-fonts, animation

---

## 1. Contexto

Inovação #6 (Proposta 3 Antigravity 1ª iter): **glifo individual = vector network nativo**. Eixos OTF de variable fonts (`weight`, `width`, `slant`, `optical-size`, `GRAD`, custom axes) expostos como **parâmetros dinâmicos do graph**, animáveis em curve, atualizáveis por motion fields ou Luau scripts.

Primeira ferramenta vetorial onde **tipografia É vetor animável**, não substituto rasterizado.

[Differentiable Variable Fonts (arXiv 2510.07638 Oct 2025)](https://arxiv.org/html/2510.07638v1) formaliza variable font interpolation como differentiable function — habilita gradient descent em font-axis space.

---

## 2. Decisão

### 2.1 Crate foundational `ph2d-vector-font`

```
crates/ph2d-vector-font/
├── Cargo.toml                       (deps: skrifa, ph2d-vector-doc, ph2d-vector-traits)
├── src/
│   ├── lib.rs                       VariableFontAxis trait + GlyphVectorNetwork
│   ├── skrifa_bridge.rs             skrifa font parsing → glyph contours
│   ├── glyph_to_network.rs          Contour → VectorNetwork (with topology)
│   ├── axis_animation.rs            Axes como AnimValue inputs
│   └── fallback_chain.rs            HR-15 locale-aware font fallback
└── tests/
    ├── glyph_to_network_golden.rs
    └── axis_interpolation_smooth.rs
```

### 2.2 Glifo = VectorNetwork nativo

```rust
pub struct GlyphVectorNetwork {
    pub network: VectorNetwork,                        // standard vector data model
    pub glyph_id: skrifa::GlyphId,
    pub axes: SmallVec<[VariableFontAxis; 8]>,         // axes presentes na fonte
    pub current_axis_values: BTreeMap<AxisTag, f32>,   // OT axis tag → current value
}

pub trait VariableFontAxis {
    fn name(&self) -> &str;                  // "Weight", "Width", "Slant", "Optical Size"
    fn tag(&self) -> AxisTag;                // OT 4-byte tag (wght, wdth, slnt, opsz)
    fn min(&self) -> f32;
    fn max(&self) -> f32;
    fn default(&self) -> f32;
    fn current(&self) -> f32;
    fn set(&mut self, value: f32) -> Result<(), AxisOutOfRangeError>;
}
```

Cada glifo emite VectorNetwork standard com:
- Contour outlines → Region per closed cycle.
- Tangentes preservadas via Levien Béz fitting (ADR-0056 §2.4).
- Winding rule NonZero (SVG canon, OT standard).

### 2.3 Render path via skrifa + Vello (sem font rasterization intermediária)

```rust
fn render_variable_glyph(
    glyph: &GlyphVectorNetwork,
    transform: Affine,
    scene: &mut vello::Scene,
) {
    // Step 1: skrifa apply axes → glyph contours em current axis values
    let outlines = skrifa::outlines(&font, glyph.glyph_id, &glyph.current_axis_values)?;

    // Step 2: contours → kurbo::BezPath via Levien fitting
    let bez_paths = outlines_to_kurbo_bez_paths(&outlines);

    // Step 3: Vello rasterize direct (no intermediate bitmap)
    for path in bez_paths {
        scene.fill(Fill::NonZero, transform, &fill_brush(), None, &path);
    }
}
```

Cache glyph → BezPath conversion por `(glyph_id, axes_hash)` LRU 100 MB.

### 2.4 Axes como graph inputs (animation hook)

```rust
// Em geometry graph:
//   motion-wave(2 Hz, amp=900) → variable-font.weight (range 100..900)
//   motion-radial-falloff(mouse_pos, r=100) → variable-font.slant
//   ph2d.expr "sin(time) * 50 + 400" → variable-font.weight

impl AttributeEvaluator for VariableFontAxisCurve {
    fn sample(&self, t: f64) -> AnimValue {
        AnimValue::Float(self.curve.sample_at(t))   // axis value lerped per keyframes
    }
}
```

Mudança de axis value via UBO update (ADR-0060 §2.3) — zero recompile do shader graph.

### 2.5 HR-15 i18n locale-aware fallback (long-tail)

`PlatformHost::system_fonts()` consulta fallback chain per locale:

```rust
pub trait PlatformHost {
    fn system_fonts(&self) -> Vec<FontFamily>;
    fn fallback_chain(&self, locale: &Locale) -> Vec<FontFamily>;
}
```

Quando glyph não está em font primary, tenta fallback chain sequencial. CJK / Devanagari / Thai etc. supported via locale-aware fallback.

### 2.6 Use cases canônicos (vide spec §14.7.4)

- **Logo animation**: weight pulsa com música (motion-wave → `variable-font.weight`).
- **HUD urgency**: number ammunition fica mais grosso conforme `predicted_pressure / max`.
- **Letterform morph**: proximity-driven via radial falloff → `slant` axis em real-time.
- **Variable font axes "instrumenting" text** (Differentiable VF approach): gradient descent em axis space para otimizar visual.

### 2.7 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| Max axes per glyph | **8** (SmallVec inline) | OT spec common; custom axes acima exigem heap |
| Glyph cache LRU | **100 MB** | Balance perf vs memory |
| Axis interpolation precision | **f32** (via `AnimValue::Float`) | OT spec uses f2dot14 normalized; f32 mais que suficiente |
| Render path | **skrifa + Vello direct** (sem rasterize intermediário) | Performance + quality |

---

## 3. Consequências

### 3.1 Positivas

- **Primeira ferramenta vetorial onde tipografia É vetor animável** — diferencial absoluto vs After Effects (rasteriza), Illustrator (sem animation axes), bitmap fonts em game engines.
- **Variable fonts axes integrados ao graph** — motion nodes drive typography natively.
- **HR-15 i18n locale-aware fallback** mantido (CJK / RTL / etc.).
- **Render direto via skrifa + Vello** preserve infinite zoom + GPU compute path.

### 3.2 Negativas

- **skrifa dep** já no stack PH2D (parley wrapper). Custo zero extra.
- **Glyph → VectorNetwork conversion** ~100 µs / glyph; cached. Acceptable.
- **OT axes count varia per font** — 8 inline cobre Inter/Roboto/JetBrains Mono típicos; fonts custom-axes heavy heap.

### 3.3 Neutras

- Embed Differentiable VF paper approach (gradient descent axes) é V2.0+ stretch (não v1.0).

---

## 4. Alternativas consideradas

### 4.1 Rasterize glyph + treat as image (rejeitada — não é vector)

Render glyph para texture + apply transform. **Por que rejeitada**: perdemos vetor (infinite zoom, animation editing, downstream modifier). Contradiz inovação #6 propósito.

### 4.2 Glyph como path único (não vector network) (rejeitada — topology lost)

Subpath model. **Por que rejeitada**: glifos com holes (B, D, O, P, etc.) precisam multiple regions com winding rule. Vector network model elegante.

### 4.3 Custom font rasterizer (rejeitada — skrifa já é canon Linebender)

Escrever próprio. **Por que rejeitada**: skrifa é state-of-the-art Linebender; PH2D já consome via `ph2d-text` (parley wrapper).

### 4.4 Embed Differentiable VF gradient descent (rejeitada — V2.0 stretch)

Optimize axis space gradients. **Por que adiada**: research-grade; v1.0 ship sem.

---

## 5. Implementação (Wave 10)

- **T10.3**: `ph2d-vector-font` crate (this ADR).
- Animation hook integration: vide ADR-0058 cross-domain motion → vector params.

Gates: `variable_font_axis_interpolation_smooth` + `variable_font_glyph_to_network_golden` + `variable_font_fallback_chain_locale_aware`.

---

## 6. Referências

- Spec normativa: [`docs/Vector Module/14_inovacoes_extraordinarias.md §14.7`](../../Vector%20Module/14_inovacoes_extraordinarias.md).
- Differentiable Variable Fonts (arXiv 2510.07638): <https://arxiv.org/html/2510.07638v1>
- skrifa crate (Linebender): <https://docs.rs/skrifa>
- MDN variable fonts: <https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Fonts/Variable_fonts>
- OT variable fonts spec: <https://learn.microsoft.com/en-us/typography/opentype/spec/otvaroverview>
- Antigravity Proposta 3 (1ª iter) absorvido em §14.7.
