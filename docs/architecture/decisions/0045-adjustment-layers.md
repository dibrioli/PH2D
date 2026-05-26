# ADR-0045 — Adjustment Layers contract (12 non-destructive + 5 destructive-only)

**Status:** Proposed (2026-05-26)
**Decisor(es):** Enio + Claude (Coord-A, sessão Painter W0).
**Pré-requisitos:** [ADR-0043 — Painter contract](0043-painter-contract.md), [ADR-0044 — Brush Engine GPU](0044-brush-engine-gpu.md).
**Spec normativa:** [`docs/Painter_projeto/02_layers.md`](../../Painter_projeto/02_layers.md) §2.10.X (Crítica A absorvida) + [`06_selection_transform_adjustments.md`](../../Painter_projeto/06_selection_transform_adjustments.md) §6.3.
**Tags:** painter, wave-0, contract, adjustment-layers, non-destructive, psd-interop

---

## 1. Contexto

Procreate é destrutivo por inércia técnica de 2011 (VRAM/CPU iPad apertados), não por tese de design. Em desktop / iPad Pro M1+ / Android top-tier há orçamento de sobra para **adjustment layers Photoshop-style não-destrutivas**. Workflow profissional exige reajuste após feedback; "duplicate-and-bake" explode VRAM em canvases grandes. Crítica A do doc [`avaliacao_e_melhorias.md`](../../Painter_projeto/avaliacao_e_melhorias.md) absorvida integralmente em W4.

Sem contrato congelado:

1. **AdjustmentKind cresce ad-hoc.** Cada wave adiciona um adjustment "que esqueceram"; o compositor vira `match` infinito.
2. **PSD interop fica caso-a-caso.** Sem mapping table congelada, export gera surpresas (adjustment salva diferente da próxima vez).
3. **Compositor recomposition strategy fica implícita.** "Cache os intermediários" sem contrato vira "cache tudo" → memória explode.
4. **5 destructive-only sem razão documentada** = pressão social pra "fazer adjustment layer pra tudo" → spec adia ou cede a re-bake massivo em features que não cabem (Liquify, Mesh Warp).

ADR-0043 §2.5 cedeu este território a esta ADR.

---

## 2. Decisão

### 2.1 Localização canônica

`AdjustmentLayer` + `AdjustmentKind` + `AdjustmentParams` vivem em **`ph2d-painter-brush::adjustments`** módulo (consistente com gates `adjustment_layer_*` listados em [`02_layers.md §2.10.X.7`](../../Painter_projeto/02_layers.md)). Se W3 extrair um crate `ph2d-painter-canvas` para a `LayerStack`, módulo migra com — amendment desta ADR (`0045-amendment-1.md`) ratifica.

### 2.2 `AdjustmentLayer` struct — cap **≤ 12 fields**

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdjustmentLayer {
    pub id: LayerId,
    pub name: String,
    pub kind: AdjustmentKind,
    pub params: AdjustmentParams,           // typed per-kind (variant matches kind)
    pub mask: Option<MaskData>,             // optional mask (raster layer mask shape)
    pub opacity: f32,                       // [0, 1]
    pub blend_mode: BlendMode,              // 22 modes §2.2 (ph2d-painter-brush::blend)
    pub visible: bool,
    pub locked: bool,
    pub clipped_by: Option<LayerId>,        // clipping mask para layer específica
    pub version: u32,                       // HR-14 — v1 = 1
    // === 1 slot de headroom ===
}
```

**Sub-cap:** `Option<MaskData>` é serializado por boundary (Some/None) — `MaskData` é estrutura própria (raster) que herda cap do tipo Layer raster (não tratado aqui).

### 2.3 `AdjustmentKind` enum — cap **≤ 24 variants** (v1 usa 12 non-destructive)

```rust
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AdjustmentKind {
    // === Non-destructive v1 (suportam AdjustmentLayer) — 12 ===
    HueSaturationBrightness,
    ColorBalance,
    Curves,
    GradientMap,
    BrightnessContrast,
    GaussianBlur,
    MotionBlur,
    Bloom,
    Noise,
    Sharpen,
    Halftone,
    ChromaticAberration,
    // === 12 slots de headroom — pista da longa cauda Photoshop ===
    // Reserved (sem ADR-amend obrigatória, dentro do cap):
    //   Vibrance, ColorLookupLUT, PhotoFilter, Posterize, Threshold,
    //   Invert, Levels, SelectiveColor, ChannelMixer, Exposure,
    //   ShadowsHighlights, BlackAndWhite.
    // Cada um exige sub-Params struct + tab entry §2.6 + sub-gate.
}
```

**Razão do cap 24 (bump de 16 — audit 2026-05-26):** "sucessor do Procreate" precisa cobrir longa cauda Photoshop (~25 non-destructive ships) sem amendment a cada um. Procreate só tem ~6 adjustments; Photoshop tem 25+. PH2D na ambição padrão-ouro fica no meio do espectro com folga.

**Nota crucial:** os **5 destructive-only** (Liquify, Clone, Recolor, Glitch, Mesh Warp) **NÃO** são variants de `AdjustmentKind` — vivem em **`DestructiveAdjustment`** enum separado (§2.4). Razão: o gate textual `adjustment_kind_variant_count_is_capped` audita variants que CONSTROEM `AdjustmentLayer`; ter destructive aí seria fonte de bug ("crie AdjustmentLayer kind=Liquify" → silenciosamente impossível). Separar em dois enums é o que o **type system** já queria nos contar.

### 2.4 `DestructiveAdjustment` enum (separado de `AdjustmentKind`) — cap **≤ 8 variants** (v1 usa 5)

```rust
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DestructiveAdjustment {
    /// Distorção destrutiva. Como layer, exigiria re-baking massivo a cada
    /// move de layer abaixo (cf. mesh-driven displacement field).
    Liquify,
    /// Source point é destrutivo (Photoshop equivalent). Smart Object clone
    /// fora de escopo (12_fora_de_escopo §10).
    Clone,
    /// Mascarable via Curves layer; manter destructive simplifica.
    Recolor,
    /// Stylistic; aceitável destrutivo (artistas usam como "selo final").
    Glitch,
    /// Distorção destrutiva via mesh handles (idem Liquify).
    MeshWarp,
    // === 3 slots de headroom ===
}
```

Cada `DestructiveAdjustment` aplica em modos **Layer (destructive)** ou **Pencil (Adjustment Brush stroke)** — vide [`06_selection_transform_adjustments.md §6.3.1`](../../Painter_projeto/06_selection_transform_adjustments.md). Nunca cria layer.

### 2.5 `AdjustmentParams` enum — cap **≤ 16 variants** (matches `AdjustmentKind`)

Discriminated union onde **variant name == AdjustmentKind variant**:

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AdjustmentParams {
    HueSaturationBrightness(HsbParams),
    ColorBalance(ColorBalanceParams),
    Curves(CurvesParams),
    GradientMap(GradientMapParams),
    BrightnessContrast(BrightnessContrastParams),
    GaussianBlur(GaussianBlurParams),
    MotionBlur(MotionBlurParams),
    Bloom(BloomParams),
    Noise(NoiseParams),
    Sharpen(SharpenParams),
    Halftone(HalftoneParams),
    ChromaticAberration(ChromaticAberrationParams),
    // === 4 slots de headroom (pair com AdjustmentKind) ===
}
```

**Cada sub-Params struct tem seu próprio cap individual** (cf. tabela §2.6). Cap composto não acumula — soma exata é cap × `N_max` se algum dia precisar (improvável; sub-structs são focados).

**Invariant runtime:** `AdjustmentLayer.kind` e `AdjustmentLayer.params` precisam matching variant. Gate `adjustment_layer_kind_params_match`:

```rust
fn matches(layer: &AdjustmentLayer) -> bool {
    matches!((layer.kind, &layer.params),
        (AdjustmentKind::HueSaturationBrightness, AdjustmentParams::HueSaturationBrightness(_)) |
        (AdjustmentKind::Curves, AdjustmentParams::Curves(_)) |
        /* … todos os 12 pares … */
    )
}
```

### 2.6 Sub-params caps por kind

| Kind | Sub-struct | Cap fields |
|---|---|---|
| `HueSaturationBrightness` | `HsbParams { h, s, b }` | ≤ 6 |
| `ColorBalance` | `ColorBalanceParams { cyan_red, magenta_green, yellow_blue, scope: ShadowsMidtonesHighlights, preserve_luminosity }` | ≤ 8 |
| `Curves` | `CurvesParams { points_rgb: ControlPoints, points_r: ControlPoints, points_g: ControlPoints, points_b: ControlPoints }` (cada `ControlPoints` ≤ 8) | ≤ 6 |
| `GradientMap` | `GradientMapParams { stops: Vec<ColorStop>, interpolation: GradientInterp }` (stops ≤ 16) | ≤ 4 |
| `BrightnessContrast` | `BrightnessContrastParams { brightness, contrast, legacy }` | ≤ 4 |
| `GaussianBlur` | `GaussianBlurParams { radius }` | ≤ 4 |
| `MotionBlur` | `MotionBlurParams { distance, angle }` | ≤ 4 |
| `Bloom` | `BloomParams { threshold, intensity, radius, falloff }` | ≤ 6 |
| `Noise` | `NoiseParams { amount, kind: NoiseKind, monochromatic }` | ≤ 6 |
| `Sharpen` | `SharpenParams { amount, radius, mask_edges }` | ≤ 4 |
| `Halftone` | `HalftoneParams { dot_size, angle, shape: HalftoneShape }` | ≤ 6 |
| `ChromaticAberration` | `ChromaticAberrationParams { red_shift, green_shift, blue_shift, falloff_center }` | ≤ 6 |

Caps por sub-struct preservam composabilidade (`AdjustmentParams` size estável; serialização postcard sem deslocamento de ID).

### 2.7 Compositor recomposition strategy

Algoritmo top-down em `composite_stack` (já em [`02_layers.md §2.10.X.3`](../../Painter_projeto/02_layers.md)). Esta ADR fixa o **protocolo de cache**:

```rust
/// Strategy congelada: cached intermediates per "cut point".
/// Cada AdjustmentLayer é um cut point. Mudança em layer N → invalida
/// caches de cut points >= N. Acima ficam válidos.
pub struct CompositorCache {
    /// Cache do acumulador antes de cada AdjustmentLayer (chave = LayerId
    /// do adjustment). Invalidado quando layer abaixo é dirty.
    cuts: BTreeMap<LayerId, CachedTexture>,
    /// Dirty rect tracking — recompose só na bbox do dab/transform.
    dirty_rect: Option<Rect>,
}

impl CompositorCache {
    /// Invalidação: chamado quando layer N muda. Invalida cuts >= idx(N).
    fn invalidate_from(&mut self, changed_layer: LayerId, stack: &LayerStack) { /* … */ }

    /// Recompose: do bottom até a primeira cut válida, depois aplica
    /// adjustments do topo. Custo: O(cuts dirty) ≪ O(layers).
    fn recompose(&mut self, stack: &LayerStack, dirty: Rect) -> Texture { /* … */ }
}
```

**HR-5 (no HashMap em compositor):** `BTreeMap<LayerId, CachedTexture>` — chave estável + iteração determinística. NÃO `std::HashMap`.

**Budget:** slider drag em adjustment layer @ 4K, 10 layers, recompose ≤ 1 ms (gate `adjustment_layer_recomposition_perf_4k` em §2.11). Esse número é o **upper bound**, não target.

### 2.8 PSD interop mapping (W16) — congelado

12 adjustments → PSD adjustment layer types. Tabela definitiva:

| `AdjustmentKind` PH2D | PSD type key | Mapping |
|---|---|---|
| `HueSaturationBrightness` | `hsbr` ("Hue/Saturation") | **1:1** |
| `ColorBalance` | `cobl` ("Color Balance") | **1:1** |
| `Curves` | `curv` ("Curves") | **1:1** |
| `GradientMap` | `grdm` ("Gradient Map") | **1:1** |
| `BrightnessContrast` | `brit` ("Brightness/Contrast") | **1:1** |
| `GaussianBlur` | (no direct adjustment layer) | **Baked** on export |
| `MotionBlur` | (idem) | **Baked** |
| `Bloom` | (PS-specific filter; sem layer equivalente) | **Baked** |
| `Noise` | (filter, não layer) | **Baked** |
| `Sharpen` | (filter, não layer) | **Baked** |
| `Halftone` | (filter, não layer) | **Baked** |
| `ChromaticAberration` | (filter, não layer) | **Baked** |

**5 mapeiam 1:1 / 7 baked**, exatamente como spec §2.10.X.5. Warning logado no export para os 7 baked (usuária sabe o que está acontecendo). Import PSD reverso: 5 adjustments-1:1 voltam como `AdjustmentLayer`; baked não-detectáveis viram raster (Photoshop comportamento esperado).

### 2.9 5 destructive-only — razões técnicas congeladas

| Adjustment | Razão (registrada como `// 0045-§2.9` em [`docs/Painter_projeto/02_layers.md`](../../Painter_projeto/02_layers.md)) |
|---|---|
| `Liquify` | Distorção via mesh displacement field; toda layer abaixo seria re-warpada a cada move de stack abaixo. Custo: O(canvas_pixels × mesh_resolution) por recompose. Inaceitável em 4K. |
| `Clone` | Source point é destrutivo (paridade Photoshop). Smart Object clone fora de escopo (`12_fora_de_escopo.md` §10) — adiar exigiria capa de abstração que não atinge o sabor v1.0. |
| `Recolor` | Funcionalmente subsumido por `Curves` (per-channel) + mask + blend mode `Color`. Manter destructive evita duplicação de UI sem ganho técnico. |
| `Glitch` | Stylistic adjustment final. Aceitação artística do destrutivo (artistas tratam Glitch como "selo final" — re-edit raríssimo). |
| `MeshWarp` | Idem Liquify — distorção via mesh handles é destrutiva por natureza geométrica. |

**Política:** se W4+ identificar técnica viável para um deles virar non-destructive (e.g., `Liquify` como displacement field cacheado), amendment desta ADR (não silenciosa migração).

### 2.10 Arch-gate `painter_contract_surface::adjustments`

Adicionado ao homestead `crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs` (localização congelada per ADR-0043 §2.4):

```rust
mod adjustments {
    #[test] fn adjustment_layer_field_count_is_capped()       { /* ≤ 12 */ }
    #[test] fn adjustment_kind_variant_count_is_capped()      { /* ≤ 24 */ }
    #[test] fn destructive_adjustment_variant_count_is_capped() { /* ≤ 8 */ }
    #[test] fn adjustment_params_variant_count_is_capped()    { /* ≤ 16 */ }
    #[test] fn adjustment_layer_kind_params_match()           { /* invariant runtime */ }
    #[test] fn psd_mapping_is_canonical()                     { /* 5 layered + 7 baked */ }
}
```

### 2.11 Gates de comportamento (separados, em `tests/adjustments_*.rs`)

| Gate | Spec |
|---|---|
| `adjustment_layer_curves_recomposition` | Curves layer com mudança em ponto → recompose produz output esperado (bit-identical com referência). |
| `adjustment_layer_psd_export_mapping` | 5 kinds mapeiam 1:1; 7 baked com warning. Verifica `.psd` output via psd-rs parser. |
| `adjustment_layer_clip_to_below` | Clip mode aplica adjustment somente à layer imediatamente abaixo. |
| `adjustment_layer_mask_aware` | Mask do adjustment limita efeito à região mask=white. |
| `adjustment_layer_recomposition_perf_4k` | Slider drag em adjustment layer @ 4K, 10 layers, recompose ≤ 1 ms. Soft em W4 (warning); hard em W5+. |

---

## 3. Consequências

### Positivas

- **AdjustmentKind congelado em 12 + 4 headroom.** Adicionar non-destructive futuro = drop variant + sub-params struct, sem amend desta ADR (até bater cap 16). 4 slots cabem 1-2 waves de extensão.
- **Type system bloqueia `kind=Liquify` em AdjustmentLayer.** Erro de design vira erro de compilação. Inestimável.
- **PSD interop é tabela fechada.** W16 vira engenharia direta; sem decisões em runtime.
- **Compositor cache contractual (`BTreeMap`, dirty rect).** HR-5 mantido. Sem `HashMap` no compositor (ADR-0022 — vide [SKILL_Stack](../../SKILL_Stack_PH2D_Definitiva.md) HR-5).
- **Razões técnicas dos 5 destructive-only documentadas.** Pressão social ("fazer adjustment layer pra tudo") tem resposta fixa: amendment ou bake.

### Negativas / Custos

- **`AdjustmentLayer.params` precisa matching com `kind`.** Gate runtime `adjustment_layer_kind_params_match` é necessário (não compile-time-enforceable em discriminated union typed). Custo: 1 test + 1 helper. Aceito.
- **Cache pode crescer memória em canvases com muitas adjustments.** Estimativa: cache textura completa por cut point = N adjustments × canvas RGBA = N × 4 × W × H bytes. Em 4K com 5 adjustments = 5 × 32 MB = 160 MB. Mitigação: cache LRU + invalidação por dirty rect (cuts >= primeiro dirty); typical-case canvas com ≤ 3 adjustments = 96 MB, dentro do budget HR-13.
- **Variants 13-16 de `AdjustmentKind` ficam reservadas.** Tentação de "preencher headroom" deve ser resistida — adicionar com critério (cada um custa Sub-Params struct + 1 sub-gate). Discipline test: cada novo variant exige 4 horas de design.

### Neutras

- **`DestructiveAdjustment` enum é separado de `AdjustmentKind`.** Custo: 2 enums em vez de 1. Ganho: type system bloqueia erro inteiro. Trade-off largamente positivo.
- **Adjustments aplicáveis a Selection** (`06_selection_transform_adjustments.md §6.3.6`) **em Layer mode** mantêm comportamento destructive — não criam AdjustmentLayer. Esta ADR não muda esse flow.

---

## 4. Alternativas consideradas

### 4.1 `AdjustmentKind` única (inclui destructive)

**Rejeitada.** Permitiria criar `AdjustmentLayer { kind: Liquify, .. }` válido pelo type system mas invalido por design — bug latente. Separação em 2 enums é o que o type system queria nos contar (cf. F# "make illegal states unrepresentable").

### 4.2 `AdjustmentParams` único struct com `Option<…>` por kind

**Rejeitada.** Sparse struct com 12 `Option<KindParams>` desperdiça memória + permite estados ilegais (2 Some simultâneos). Discriminated union é canon Rust.

### 4.3 Cache de textura inteira por cut point (sem dirty rect)

**Rejeitada.** Custo memória escala linearmente com cuts × canvas; sem dirty rect, slider drag em adjustment recompose canvas inteiro (4K × 10 layers, ~50 ms — fora do budget). Dirty rect cap recompose à bbox do dab, ~1 ms.

### 4.4 Permitir todos os 17 como AdjustmentLayer (rebake forçado)

**Rejeitada.** Liquify / Mesh Warp em layer mode exigiriam re-warp de tudo abaixo a cada move (mesh displacement). Custo proibitivo em 4K — re-bake forçado fere o sabor "responsivo Procreate-style".

### 4.5 PSD adjustments baked sem warning

**Rejeitada.** Round-trip silencioso (`export .psd → import .psd → 7 adjustments viraram raster`) destrói confiança do usuário. Warning explícito é UX honesto.

---

## 5. Verificação

```sh
cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
# 16 sub-tests cumulativos (ADR-0043 + 0044 + 0045). Adjustments mod = 6 testes.

cargo test -p ph2d-painter-brush
# adjustment_layer_kind_params_match runtime invariant + sub-Params Pod/Default + PSD mapping.

cargo test -p ph2d-painter-brush --test adjustment_layer_recomposition_perf_4k
# Perf gate: slider drag → recompose ≤ 1 ms. Soft em W4; hard em W5+.
```

### 5.1 Definição de "Accepted"

Esta ADR transita `Proposed → Accepted` no mesmo evento T0.9 (cascata W0).

---

## 6. Tracking

- Plano operacional: [`docs/Painter_projeto/15_plano_de_implementacao.md`](../../Painter_projeto/15_plano_de_implementacao.md) §3 (T0.3) + §6 (W4).
- Spec normativa: [`02_layers.md §2.10.X`](../../Painter_projeto/02_layers.md) + [`06 §6.3`](../../Painter_projeto/06_selection_transform_adjustments.md).
- Crítica A original (motivação): [`avaliacao_e_melhorias.md`](../../Painter_projeto/avaliacao_e_melhorias.md).
- Próxima ADR na cascata W0: [ADR-0046 — Stroke Vector History](0046-stroke-vector-history.md) (T0.4).
