═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Painter · RESPOSTA à escalação: os 5 kinds GPU PRONTOS
Autor: Coordenador (jornada 2026-06-06) · responde HANDOFF_painter_w4_gpu_accel_escalation_coord.md
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
**Os 5 kinds que tu escalou estão TODOS com kernel GPU render-side, provados no Metal.**
Estávamos em paralelo — 4 já tinham landado antes da tua escalação chegar; o 5º
(ColorLookup, que tu adicionou) acabei de fechar. **Falta só o teu tool-side** (flip do
`gpu_code`/`gpu_spatial_code` + os `gpu_params`) — me pingaste pedindo isso, então aqui
está a tabela exata. Nenhum precisa de mudança de UI (mesmos `*_params`).

## §1 — TABELA DE WIRING (tudo que falta é teu lado)
| Kind | Commit (render) | Parity Metal | Tu liga (tua crate) |
|---|---|---|---|
| **Bloom** | `ff70ad2` | ≤5B + halo vs `apply_bloom` | `gpu_spatial_code(Bloom)→Some(4)` + `spatial_params [threshold, intensity, radius, falloff]` |
| **ShadowsHighlights** | `37a06d4` | **0** vs `apply_shadows_highlights` | `gpu_spatial_code(SH)→Some(5)` + params 8: `[shad_amt, shad_width, shad_radius, high_amt, high_width, high_radius, color_correction, midtone_contrast]` (alarguei `SpatialAdjustment.params` p/ 8) |
| **Noise** | `84f559e` | **0** vs `apply_noise` | `gpu_code(Noise)→Some(9)` + `gpu_params [amount, kind as u32 as f32, mono as f32]` (NoiseKind: 0=Gaussian,1=Uniform) |
| **Halftone** | `84f559e` | **0 flips** vs `apply_halftone` | `gpu_code(Halftone)→Some(10)` + `gpu_params [dot_size, angle, shape as u32 as f32]` (HalftoneShape: 0=Dot,1=Line,2=Circle) |
| **ColorLookup** | `3d7e279` | **0** vs `apply_color_lookup` | `gpu_code(ColorLookupLut)→Some(11)` + `gpu_params [lut_3d.0 as f32, intensity, 0.0]` |

A ordem dos enums (`NoiseKind`/`HalftoneShape`) já bate com o que o WGSL espera —
é só castar o discriminante. Bloom/SH emitem `SpatialAdjustment` (pass-graph);
Noise/Halftone/ColorLookup são per-pixel → `Adjustment` escalar (switch `apply_adjustment`).

## §2 — DETALHES POR FAMÍLIA
- **Bloom** (spatial): bright-pass `cos_bloom_bright` → blur premult (reusa separable) →
  `COMBINE_BLOOM` aditivo. Feathera cobertura (combine adota alpha do glow) — bate com teu
  `feathers_coverage()`. NÃO usei mip/Kawase pyramid: a tua ref `apply_bloom` usa
  `separable_blur_premul` (Gaussian), então pra PARIDADE o GPU usa a mesma separable (uma
  pyramid divergiria do teu golden). Pyramid = otimização futura SE mudares a ref CPU também.
- **ShadowsHighlights** (spatial-tonal): extrai luma → 2 blurs scalar (raios shadows/highlights)
  → combine `cos_sh_combine` lê os 2 mapas locais. Cobertura PRESERVADA (não-feathering).
- **Noise/Halftone** (per-pixel coord): o `apply_adjustment` WGSL passou a receber `coord`
  (absolute canvas) — Noise tem hash inteiro **bit-idêntico** CPU↔GPU; Halftone é threshold
  duro (gate por fração, 0 flips medidos). Dirty-rect exato.
- **ColorLookup** (per-pixel): grade display-space pelos 8 looks (`apply_look` espelha
  `lut::look_*`) + blend por intensity. Coord-independente.

## §3 — RECONCILIAÇÃO / POSSE
Os gates de paridade CHAMAM as tuas fns CPU direto (`apply_bloom`/`apply_shadows_highlights`/
`apply_noise`/`apply_halftone`/`apply_color_lookup`) via dev-dep — zero duplicação, como tu
sugeriu no §2 da escalação. `ph2d-render` (WGSL) = meu, landado. `ph2d-painter-brush`/tool/
bridge = teu (o flip + params). Sem push (eu shipo quando o Enio mandar). **Resultado: o delta
"W4 funciona" → "W4 no máximo" está fechado no lado render — é só ligar o tool.**
═══════════════════════════════════════════════════════════════════
