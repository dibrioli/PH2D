═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · Bloom + Shadows/Highlights — ref CPU pronta, GPU é tua
Autor: Implementador Painter (jornada 2026-06-06) · fecha o W4 (24/24 kinds vivos)
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
**W4 está 100%** (CPU). Os 2 últimos kinds — **Bloom** e **ShadowsHighlights** — landaram
funcionais no caminho CPU (commit `33d428e`, `ph2d-painter-brush` + combine em
`ph2d-tool-painter`). São as **refs CPU canônicas** pra tu acelerar na GPU (espelho do
que fizemos com Gaussian). Ambos rodam hoje via **CPU fallback** (`gpu_code`/`gpu_spatial_code`
= None → flatten devolve None). Não-bloqueante; só queres GPU pra performance.

## §1 — Bloom (ref CPU: `spatial::apply_bloom`)
Pipeline: **bright-pass → blur → glow aditivo**, tudo **premultiplicado**.
1. Bright-pass: `w = smoothstep(threshold, threshold+falloff, display_luma(px))`;
   `bright_premult = color·alpha·w` (texel transparente → 0).
2. Blur do bright-pass: separável Gaussian `radius` (a mesma `separable_blur_premul`).
3. Add: `out_premult = base_premult + intensity·blur_premult`; un-premultiplica.
Params `[threshold, intensity, radius, falloff]`. **Bloom FEATHERA cobertura** (o glow
haloa pra fora na transparência) → no combine usa o alpha do kernel (ver §3).
**Tua infra GPU:** o blur de raio grande quer **mip/Kawase pyramid** (downsample→blur→
upsample) — é o único pedaço de infra novo. O resto (bright-pass + add) é per-pixel.
Sugiro `SPATIAL_BLOOM` novo + a pyramid; reconcilia contra `apply_bloom`.

## §2 — ShadowsHighlights (ref CPU: `spatial::apply_shadows_highlights`)
**Contraste LOCAL** (a sacada vs uma curva global):
1. `luma = display_luma(px)`; blur dela em 2 mapas: `local_lo` (raio shadows),
   `local_hi` (raio highlights) — separável escalar (`separable_blur_scalar`).
2. Membership pela tonalidade LOCAL: `ws = 1 - smoothstep(0, shadows_tonal_width, local_lo)`
   (forte em sombra local), `wh = smoothstep(1-highlights_tonal_width, 1, local_hi)`.
3. `new_l = clamp(0.5 + (l + shadows_amount·ws - highlights_amount·wh - 0.5)·(1+midtone_contrast))`.
4. Re-tom: escala o RGB display por `new_l/l` (preserva matiz); `color_correction` mexe
   saturação nas regiões corrigidas (`·(ws+wh)`).
Params (8): `[shad_amt, shad_width, shad_radius, high_amt, high_width, high_radius,
color_correction, midtone_contrast]`. **Cobertura PRESERVADA** (op tonal, não borra a
imagem — só uma luma interna) → combine per-pixel.
**Tua infra GPU:** blur-do-canal (luma) + **combine variante** (lê os 2 mapas locais).
O blur da luma reusa tua máquina separável; o combine é um shader novo. Reconcilia
contra `apply_shadows_highlights`.

## §3 — MUDANÇA NO COMBINE (já fiz no CPU; espelha na GPU)
Introduzi **`AdjustmentKind::feathers_coverage()`** (mod.rs): true pra blur-family
**+ Bloom**, false pra ShadowsHighlights / tonais / per-pixel. O combine em `compose.rs`
trocou `gpu_spatial_code().is_some()` por `feathers_coverage()`:
- **feathers=true** → adota o alpha (feathered) do kernel (lerp dos 4 canais).
- **feathers=false** → preserva `base.a` (S/H, Noise, Halftone, cor).
**No GPU:** teu `cs_combine` do `SpatialAdjustment` deve usar o alpha feathered pros
kernels feathering (Gaussian/Sharpen/Motion/Chroma **+ Bloom**) e preservar base pra
S/H. (Se fizeres S/H como pass-graph, marca-o como NÃO-feathering no combine.)

## §4 — POSSE / SEGUIMENTO
Mexi só nas MINHAS crates (`ph2d-painter-brush`, `ph2d-tool-painter`). `ph2d-render`
(pyramid Bloom + combine S/H) é TEU. Sem push (tu shipas). Refs: `apply_bloom` /
`apply_shadows_highlights` + os testes (`bloom_*`, `shadows_highlights_*`,
`feathers_coverage_set_*`). **W4 fechado**: 24/24 kinds vivos no produto (UI + compute);
os GPU-accel de Noise/Halftone/Bloom/S/H são teus follow-ups de performance, não-bloqueantes.

**Smoke do Enio (CPU, já vale):** Bloom numa layer transparente → halo de luz suave;
Shadows/Highlights → levanta sombra / recupera highlight sem achatar contraste local.
═══════════════════════════════════════════════════════════════════
