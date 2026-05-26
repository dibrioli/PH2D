# ADR-0051 — Color profile pipeline (sRGB / Linear scRGB / Display P3 / ProPhoto / OKLab invariants)

**Status:** Proposed (2026-05-26)
**Decisor(es):** Enio + Claude (Coord-A, cascata W0 rework regra perfeição).
**Pré-requisitos:** [ADR-0042 — Wave 10 closure (`ph2d-color`)](0042-wave-10-closure.md), [ADR-0044 — Brush Engine GPU](0044-brush-engine-gpu.md), [ADR-0046 — Stroke Vector History](0046-stroke-vector-history.md).
**Spec normativa:** [`docs/Painter_projeto/03_color.md`](../../Painter_projeto/03_color.md) + [`08_performance_memory.md`](../../Painter_projeto/08_performance_memory.md).
**Motivação:** sense-check veterano paint stack 2026-05-26 (gap "color profile management" #3).
**Tags:** painter, wave-0, contract, color-profile, oklab, srgb, display-p3, mixbox, hdr

---

## 1. Contexto

Paint stack profissional opera em **múltiplos color spaces** simultaneamente:

| Espaço | Onde usa | Por quê |
|---|---|---|
| **sRGB** | Default canvas, exports web | Standard universal; gamut limitado |
| **Display P3** | iPad Pro, MacBook Pro M-series, iPhone | Gamut 25% maior; default Apple pro |
| **Linear scRGB / Linear sRGB** | Compositor (blending, multiply, add) | Mistura física correta exige gamma=1 |
| **OKLab** | Stamp.color_oklab (ADR-0044 §2.3) | Lerp perceptualmente uniforme; cor pickers |
| **OKLCH** | UI tokens (ADR-0042 ph2d-color) | Hue + Chroma + Lightness intuitivos |
| **ProPhoto RGB** | Photo retouching pro (raro paint) | Maior gamut; exists para PSD interop |
| **HDR / Rec.2100 PQ** | iPad M4 ProMotion HDR, MacBook Pro XDR | Brilho > 1.0; emergente em 2025+ |

Conversões silenciosas entre esses espaços **destroem cor**. Audit sense-check veterano flagou: "Mixbox opera em sRGB linear (paper) — qual é o pipeline exato? OkLab no Stamp + sRGB no Mixbox + Display P3 no output = 3 conversões silenciosas".

Sem ADR-0051 explicitando o pipeline:

1. **Mixbox em sRGB linear é diferente de Mixbox em Display P3.** Wide-gamut canvas + Mixbox = resultado divergente (pigment math é gamut-dependent).
2. **Stamp.color_oklab → blend no compositor.** OKLab é perceptual, NÃO é linear physical para blending. Quando exatamente OKLab vira Linear scRGB?
3. **Canvas color profile no `.ph2d-painter`** (ADR-0046 §2.7.1 `CanvasInfo.color_profile`) é tipo opaco — qual é o set fixo?
4. **Export para PSD / JPEG / PNG** exige profile-aware conversion. Photoshop assume sRGB se nada especificado; PH2D precisa explicit.
5. **HDR canvas** (Apple XDR + iPad M4 ProMotion) ship em 2025+; paint stack que não suporta é "atrasado em 2026".

Regra "perfeição desde início, sem adiamentos" obriga endereçar **todos** esses gaps na cascata W0.

---

## 2. Decisão

### 2.1 Crate `ph2d-color` (existente, ADR-0042) ganha amend

`ph2d-color` já existe com 4 módulos (linear, srgb, premultiplied, oklch — ADR-0042 §2.3). Esta ADR amenda:

```
crates/ph2d-color/
  src/
    linear.rs       # LinearRgba (existe)
    srgb.rs         # SrgbRgba (existe)
    premultiplied.rs# Premultiplied (existe)
    oklch.rs        # OklchColor (existe)
    oklab.rs        # OklabColor NEW — Stamp consome ADR-0044
    display_p3.rs   # DisplayP3 NEW
    prophoto.rs     # ProPhoto NEW — PSD interop
    hdr.rs          # Rec2100Pq NEW — HDR canvas (opt-in)
    profile.rs      # ColorProfile enum + conversion graph NEW
    mixbox_space.rs # MixboxColorSpace NEW — define onde Mixbox opera
```

LOC cap `ph2d-color` ADR-0042 §2.1 era 1500; revisto para **≤ 2500** com novos módulos (cap-amend ratificado por esta ADR).

### 2.2 `ColorProfile` enum — cap **= 8 variants FROZEN** (v1 = 7)

```rust
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ColorProfile {
    /// sRGB IEC 61966-2-1. Gamut limitado, universal. Default web/print.
    Srgb = 0,
    /// Linear sRGB. Compositor working space — blending physically correct.
    /// Não user-facing; export sempre converte para profile escolhido.
    LinearSrgb = 1,
    /// Display P3 (Apple). Gamut 25% maior que sRGB. Default Mac/iPad Pro.
    DisplayP3 = 2,
    /// Linear Display P3. Compositor variant quando canvas é P3.
    LinearDisplayP3 = 3,
    /// ProPhoto RGB. Gamut maior; PSD interop fotografia.
    ProPhoto = 4,
    /// Rec.2100 PQ (HDR). Apple XDR + iPad M4 ProMotion. Opt-in W17+ post-1.0
    /// se device suporta; canvas tem `hdr_enabled: bool` per-profile.
    Rec2100Pq = 5,
    /// Adobe RGB (1998). Print profissional; export-only (nunca canvas working).
    AdobeRgb = 6,
    // === 1 slot de headroom (ex: Rec.709, Rec.2020 SDR) ===
}
```

**Por que frozen em 8 e não escalável:** lista é state-of-the-art 2026; profiles emergentes ficam em `unsafe` extensions via ADR-amend explícita. Cada profile exige conversion graph completo (vide §2.3) — adicionar custa real engineering.

### 2.3 Conversion graph (todas conversões diretas, sem hops intermediários)

| Source | Target | Função | Implementação |
|---|---|---|---|
| `SrgbRgba` | `LinearSrgb` | `to_linear()` | Gamma decode 2.4 piecewise |
| `LinearSrgb` | `SrgbRgba` | `to_srgb()` | Gamma encode |
| `SrgbRgba` | `DisplayP3` | `srgb_to_p3()` | Matrix 3×3 (sRGB matrix · D65 · P3⁻¹) |
| `DisplayP3` | `SrgbRgba` | `p3_to_srgb()` | Matrix inverse |
| `LinearSrgb` | `OklabColor` | `linear_to_oklab()` | Cubic-root + matrix (paper Björn Ottosson) |
| `OklabColor` | `LinearSrgb` | `oklab_to_linear()` | Matrix + cube |
| `OklabColor` | `OklchColor` | `oklab_to_oklch()` | Cartesian → polar (a,b → C,h) |
| `OklchColor` | `OklabColor` | `oklch_to_oklab()` | Polar → cartesian |
| `LinearDisplayP3` | `OklabColor` | `p3_linear_to_oklab()` | Different matrix vs sRGB path |
| ... | ... | ... | — todas direções pairwise documentadas |

**Cap:** `≤ 32 conversion functions` (v1: ~20 — todas pairs relevantes). Conversions transitivas (Srgb → P3 via LinearSrgb) NÃO são permitidas em hot path — gera 2-hop overhead.

### 2.4 Stamp pipeline — color space invariants (CONGELADOS)

```
┌──────────────────────────────────────────────────────────────┐
│ Color picker (UI) → OklchColor                               │
│   ↓ oklch_to_oklab                                            │
│ Stamp.color_oklab (ADR-0044 §2.3)                            │
│   ↓ oklab_to_linear_<canvas_profile>                          │
│ stamp_color_linear (working space = LinearSrgb OR             │
│                     LinearDisplayP3 conforme canvas)          │
│   ↓ Mixbox (se PigmentMode::Mixbox) ou Lerp (Linear)         │
│ blended_color_linear                                          │
│   ↓ Premultiplied alpha                                        │
│ layer_texture_linear                                          │
│   ↓ tone_map (se HDR) / clamp                                  │
│   ↓ to_<profile> at present time                               │
│ swapchain output (sRGB OR DisplayP3 OR Rec2100Pq)             │
└──────────────────────────────────────────────────────────────┘
```

**Invariants congelados:**

- **I1:** `Stamp.color_oklab` SEMPRE em OklabColor (independente do canvas profile). Lerp em OKLab é perceptually uniform; é o motivo da escolha (ADR-0044 §2.3 layout).
- **I2:** Compositor (blending + layer textures) SEMPRE em **Linear working space** matching o canvas profile (`LinearSrgb` se canvas=Srgb, `LinearDisplayP3` se canvas=DisplayP3). Blending físico correto exige gamma=1.
- **I3:** Mixbox opera em **Linear sRGB** sempre (paper Sochorová+Jamriška 2021 é calibrado em Linear sRGB pigment KS coefs). Wide-gamut canvas (DisplayP3) executa Mixbox em LinearSrgb e re-projeta para LinearDisplayP3 após — overhead ~5 mat ops por sample. Gate `mixbox_color_space_is_linear_srgb`.
- **I4:** Tone mapping aplicado APENAS em HDR canvas (Rec2100Pq), no swapchain present, nunca durante compositor.
- **I5:** Export converte do working space para profile alvo no save (PSD → ProPhoto preserva 16-bit; PNG sRGB clamps; JPEG sRGB 8-bit).

### 2.5 Canvas profile decision

`CanvasInfo.color_profile: ColorProfile` (ADR-0046 §2.7.1). Decisão estrutural:

- **Working space** sempre o **`Linear*` correspondente** ao canvas profile (`Srgb` canvas → `LinearSrgb` working; `DisplayP3` canvas → `LinearDisplayP3` working).
- **Storage bit depth** decidido per-canvas:
  - `Srgb` + `DisplayP3`: 8-bit RGBA tipicamente (`RGBA8UnormSrgb` / `RGBA8UnormSrgb` com matrix); 16-bit opt-in para photo retouch.
  - `ProPhoto`: 16-bit obrigatório (`RGBA16Float`); 8-bit perde gamut.
  - `Rec2100Pq` HDR: 16-bit float obrigatório (`RGBA16Float`).
- **VRAM impact:** 16-bit = 2× memória per layer. Reflected em `MemoryBudget::Painter` adaptação automática (HDR canvas reduz max layers; ADR-0046 §2.10 tabela atualizada).

### 2.6 Mixbox em wide-gamut — algoritmo congelado

Quando canvas profile = `DisplayP3` (working = `LinearDisplayP3`) e brush `pigment_mode = Mixbox`:

```rust
fn mixbox_lerp_p3_aware(a: LinearDisplayP3, b: LinearDisplayP3, t: f32) -> LinearDisplayP3 {
    // 1. Convert P3 → sRGB linear (matrix mul; ~6 ops)
    let a_srgb = p3_linear_to_srgb_linear(a);
    let b_srgb = p3_linear_to_srgb_linear(b);
    // 2. Mixbox lerp em Linear sRGB (canônico do paper)
    let r_srgb = mixbox_lerp_linear(a_srgb, b_srgb, t);
    // 3. Convert back to P3
    srgb_linear_to_p3_linear(r_srgb)
}
```

Overhead total: ~12 mat ops adicionais per sample (vs 0 em canvas sRGB). Stamp pipeline budget é ~3.5ms (08 §8.1.1); adição estimada ~0.05 ms em 4096 stamps. Cabe.

**Gate `mixbox_p3_aware_round_trip`:** Mixbox em P3 + Mixbox em sRGB com mesma input devem visualizar idênticos (módulo gamut clipping em sRGB).

### 2.7 Export profile policy

```rust
pub enum ExportFormat {
    Png { profile: ColorProfile, bit_depth: u8 },  // 8 ou 16; default sRGB 8-bit
    Jpeg { profile: ColorProfile, quality: u8 },   // sempre 8-bit; default sRGB
    Psd { profile: ColorProfile, bit_depth: u8 },  // 8, 16, ou 32-bit float
    Webp { profile: ColorProfile, lossless: bool },
    Tiff { profile: ColorProfile, bit_depth: u8 },
    Hdr { format: HdrFormat },                     // OpenEXR ou Rec2100Pq HEIF
}
```

**Conversion policy at export:**
- Working space → target profile via direct conversion (sem 2-hop).
- 16-bit storage + 8-bit target → dithering Floyd-Steinberg opt-in (default off — Photoshop padrão).
- HDR canvas → SDR target → tone mapping ACES (default) ou Hable (opt-in).
- Gamut out-of-range (P3 canvas → sRGB export com cor fora do sRGB gamut) → soft clipping + warning UX.

### 2.8 Color picker → canvas profile awareness

Color picker (W7 ADR `ph2d-painter-color`) opera em OKLCH (perceptual uniform), MAS deve **mostrar gamut warning** se cor escolhida sai do canvas profile:

```
┌──────────────────────────────────────────┐
│ Color picker                              │
│  [Disc] vibrant red picked: oklch(0.7, 0.3, 30°)│
│                                            │
│  ⚠ Out of gamut for sRGB canvas. Will be   │
│    clipped on export. [Convert canvas to   │
│    Display P3 to preserve.]                │
└──────────────────────────────────────────┘
```

Gate `color_picker_warns_on_gamut_clip`: cor fora do `ColorProfile` ativo do canvas → warning UX visible.

### 2.9 Caps numéricos

| Tipo | Cap |
|---|---|
| `ColorProfile` | **= 8 variants FROZEN** (v1 = 7) |
| Conversion functions cross-profile | ≤ 32 (v1 ≈ 20) |
| `MixboxColorSpace` (apenas LinearSrgb canon) | = 1 FROZEN |
| `ExportFormat` variants | ≤ 8 (v1 = 6) |
| `HdrFormat` variants | ≤ 4 (v1 = 2: OpenExr, HeicRec2100) |
| `ph2d-color` crate LOC | ≤ 2500 (bumped from 1500 ADR-0042 §2.1) |

### 2.10 Arch-gate `painter_contract_surface::color_pipeline`

```rust
mod color_pipeline {
    #[test] fn color_profile_variant_count_is_exact_8()        { /* FROZEN */ }
    #[test] fn conversion_functions_no_2_hop_in_hot_path()     { /* grep textual: stamp pipeline ops são direct */ }
    #[test] fn mixbox_color_space_is_linear_srgb()             { /* I3 §2.4 */ }
    #[test] fn stamp_color_oklab_invariant()                   { /* I1 — Stamp.color_oklab é OklabColor regardless of canvas */ }
    #[test] fn compositor_working_space_matches_canvas_profile() { /* I2 */ }
    #[test] fn tone_map_only_in_present_for_hdr()              { /* I4 */ }
    #[test] fn export_format_variant_count_is_capped()         { /* ≤ 8 */ }
    #[test] fn hdr_canvas_uses_rgba16float_storage()           { /* §2.5 */ }
}
```

### 2.11 Gates de comportamento

| Gate | Crate | Valida |
|---|---|---|
| `mixbox_p3_aware_round_trip` | ph2d-color | Mixbox em P3 ≈ Mixbox em sRGB (within sRGB gamut). |
| `srgb_p3_conversion_no_drift` | ph2d-color | sRGB → P3 → sRGB = identity (within ULP). |
| `oklab_lerp_perceptually_uniform` | ph2d-color | Δoklab linear vs Δsrgb_lerp visually checked baseline. |
| `hdr_tone_map_aces_baseline` | ph2d-color | ACES tone map on Rec2100Pq → sRGB matches reference impl. |
| `gamut_clip_warns_in_picker` | ph2d-panel-painter-color (W7) | Cor out-of-gamut → UX warning visible. |
| `export_psd_preserves_profile` | ph2d-painter-export (W16) | PSD export with `ProPhoto` round-trips. |

---

## 3. Consequências

### Positivas

- **Zero conversões silenciosas.** Cada hop entre color spaces é explícito + auditável + testado.
- **Mixbox em P3-aware canvas funciona corretamente.** Wide-gamut + pigment mixing = state-of-the-art 2026.
- **HDR canvas opt-in desde v1.** Apple XDR + iPad M4 ProMotion target nativos. Painter não é "atrasado" em 2026.
- **Export profile-aware.** Foto retouch pro workflow funciona (ProPhoto 16-bit round-trip).
- **Gamut warning UX honesto.** Usuária sabe quando cor escolhida vai clipar.

### Negativas / Custos

- **`ph2d-color` LOC cap 1500 → 2500.** Bump justificado mas é growth crate foundational. Mitigação: módulos isolados (`display_p3.rs`, `prophoto.rs`, `hdr.rs`), cada um auditável separadamente.
- **Mixbox P3-aware overhead ~12 mat ops/sample.** ~0.05ms em 4096 stamps; aceitável vs ganho cor.
- **HDR storage 2× memory.** Max layers em HDR canvas cai pela metade. UX explica em "Create Canvas" dialog (já planejado em §2.5).
- **8 ColorProfile variants frozen.** Profiles emergentes (e.g., Rec.2020 SDR para TV) precisam ADR-amend — não silently extendable. Aceito vs caos.

### Neutras

- **Working space sempre Linear.** Não user-facing; é detalhe de implementação consistente.

---

## 4. Alternativas consideradas

### 4.1 `ColorProfile` escalável (cap ≤ 16)

**Rejeitada.** Cada profile exige conversion graph completo (de e para cada outro). 16 profiles = 16×15 = 240 funções. Custo engineering > ganho usuário. 8 cobre estado-da-arte 2026.

### 4.2 Mixbox em P3 nativo (sem conversion para sRGB)

**Rejeitada.** Mixbox paper (Sochorová+Jamriška 2021) é calibrado em Linear sRGB KS coefs (Hansa Yellow, Cadmium Red, etc. medidos em sRGB). Mixbox "em P3" exigiria recalibração completa dos KS coefs — trabalho de paper acadêmico, não eng. Conversion sRGB-bridge é canon.

### 4.3 HDR adiado para post-1.0

**Rejeitada.** Regra "perfeição desde início". Apple XDR ship há anos; iPad M4 ProMotion HDR é mainstream em 2026. Adiar = "atrasado" no ship.

### 4.4 Working space único OKLab (sem Linear sRGB/P3)

**Rejeitada.** Blending físico (multiply, screen, add) exige Linear; OKLab não é linear physical. Compositor 100% em OKLab quebra Photoshop-compatible blend modes (ADR-0045 + 22 blend modes spec).

---

## 5. Verificação

```sh
cargo test -p ph2d-color
# 30+ conversion + invariant tests.

cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
# Caps cumulativos.
```

---

## 6. Tracking

- Plano operacional: integra em W1 T-color (novo, paralelo a T-input). W7 ph2d-panel-painter-color consome.
- Spec normativa: `03_color.md`.
- Próxima ADR na cascata W0 rework: [ADR-0052 — Tear-resistant stroke commit](0052-tear-resistant-stroke.md).
