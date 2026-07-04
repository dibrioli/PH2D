---
name: project_painter_texture_brush_stamp_cache
description: "Textured-brush FPS drop (esp. Anchored) = algorithmic, not CPU limit — cache the brush stamp like Blender, don't re-sample per-pixel-per-dab"
metadata: 
  node_type: memory
  type: project
  originSessionId: b1620edb-2d55-49ca-957d-0ad6dc05d957
---

Painter **textured paint** dropped FPS, mainly with **Anchored** (re-stamps its whole footprint
every pointer move). Enio's hint was right: **CPU isn't the limit — Blender does 2D paint on CPU
smoothly via a brush-image cache.** Our anti-pattern was recomputing falloff × texture **per-pixel,
per-dab** (so Anchored re-sampled the full footprint every move; Voronoi was 4× plain).

**Fix (matches Blender's `brush_painter_2d` + `BrushPainterCache`):** a static **View** texture makes
the stamp **scale-invariant** (falloff(d/R) + View-texture(rel/R) only scale with R), so render the
unit mask **once** into a `StampMask` and **bilinear scale-blit** per dab — texture sampled once per
stroke, not per pixel per dab. `crates/ph2d-painter-brush/src/stamp.rs` (render_stamp_mask/blit_stamp)
+ `texture::sample_unit` + `TextureSettings::is_cacheable`; tool cache in
`tool/paint/stamp_cache.rs` (adaptive mask size = pow2(2·radius) capped 1024; re-render only on
appearance/size key change). **Canvas-relative / per-dab mappings (Tiled/Stencil/Random/Rake) and
no-texture keep the per-pixel path** — only View is scale-invariant. Result: textured Anchored ≈ plain
(noise 8.8→5.8, voronoi 14.2→6.6 ms/move on 2048²).

Two earlier attempts the right approach **superseded**: (1) parallelising the per-pixel stamp
(`dab::parallel_band_stamp`, thread::scope disjoint row-bands — kept, deterministic, 3-4× — but still
O(texture) per move); (2) **suppressing the texture during the interactive preview** — Enio rejected
it: **the texture MUST stay visible during the drag.** So perf fixes here may NOT hide/cheapen the
texture visually; make the *work* cheaper instead.

Method that worked (per [[feedback_measure_perf_symptom_scale]]): added ignored `--release` timing
tests (`perf_texture_stamp_cost_on_a_large_dab`, `perf_anchored_drag_per_move_cost`) to establish the
ms BEFORE choosing the fix, and re-measured after each. GPU painter is planned later, but this was a
real CPU/algorithmic win worth doing now.
