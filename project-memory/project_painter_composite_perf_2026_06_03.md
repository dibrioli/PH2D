---
name: project-painter-composite-perf-2026-06-03
description: Painter adjustment slider-drag FPS — measured release breakdown + the CPU-reference-on-hot-path root cause
metadata: 
  node_type: memory
  type: project
  originSessionId: f62b2ebc-7a68-4f88-85e9-410a9b7a12ae
---

Painter adjustment-layer slider-drag was slow (Enio smoke, 2026-06-03). Measured, **release** (`opt-level=3`+thin-LTO), `composite()` @1024² full recompose:

- base only (decode+blend+encode): ~15 ms
- + adjustment arm (`acc.to_vec()` + blend-back): +9 ms (~24 ms for cheap kinds = 40 fps)
- + Brightness/Contrast math: +1 ms
- + **HSB OKLab `cbrt` round-trip: +30 ms** → HSB ~55 ms = 18 fps

Lessons:
- **Always measure perf in `--release`.** `[profile.dev] opt-level=0` → the CPU compositor is ~7× slower in debug (1024²+HSB = 373 ms debug vs 53 ms release). Enio runs release.
- **`powf` is cheap on Apple Silicon (~4 ns).** LUT-ing the compositor sRGB decode/encode (commit `902a6cb`, byte/bit-exact, in `ph2d-tool-painter::compositor`) only saved ~24 ms (80→56). Don't assume `powf` dominates — instrument first (I guessed wrong twice).
- **Root cause is architectural:** the painter live-preview calls the CPU **reference** `composite()` every frame (`take_preview_arc`→`composite`). The real-time path is the **GPU `ph2d-render` LayerCompositor**. The CPU path is "clear over fast" by design (module doc). HSB `cbrt` @4K ≈ 480 ms → the `≤1ms@4K` gate is impossible on CPU; only the GPU closes it.
- **`CompositorCache` (ADR-0045 §2.7, Coord, commit `2b68ab2`) is NOT wired into the tool's drain** — it caches the acc *below* the adjustment (removes base re-decode) but NOT the encode or the adjustment's own apply. Wiring it → cheap kinds ~60 fps; HSB still cbrt-bound.

**RESOLUTION (2026-06-04): GPU adjustment pipeline built + proven.** The real-time answer (Enio: "most powerful game engine; effects must animate with no perceptible cost; WebGL was faster"). The GPU `ph2d-render::layer_compositor` (a compute shader, already had 22 blend modes + groups but was DORMANT — only a comment in `painter_bridge`) now applies adjustments:
- `LayerOp::Adjustment{kind:u8, params:[f32;3], blend, opacity}` + binding-5 params storage + `apply_adjustment(kind,params,acc)` in `layer_composite.wgsl` (7 kinds; HSB/Vibrance OKLab via `pow`, B/C, Invert/Posterize/Threshold display-space, Exposure). Mirrors the CPU `composite_into` adjustment arm. Commits `e0a81c9`,`afe210f`.
- Tool↔GPU contract: `AdjustmentKind::gpu_code()->Option<u8>` + `AdjustmentParams::gpu_params()->[f32;3]` (commit `18a85a1`).
- **PROVEN on real Metal:** parity vs canonical CPU `apply_adjustment` (7 kinds ±4 bytes, partial-opacity exact); **base+HSB full 1024² = 1.7 ms (vs 55 ms CPU, ~32×)**, 2048² = 3.2 ms. Gate `gpu_adjustment_matches_cpu_reference_each_kind` + `gpu_adjustment_drag_full_canvas_perf` (GPU lane, `#[ignore]`); no-GPU: `shader_adjustment_coefficients_bit_identical_with_rust`.
- WGSL literals MUST be `ph2d_color`'s rounded f32 (NOT full-precision OKLab spec values — full-precision drifts parity past tolerance; the B/C pivot too). Pinned by the coefficient gate.
- **Phase 3 (route preview→GPU):** Coord landed the flatten + GPU-vs-CPU gate (`render_loop::painter_gpu_flatten::flatten_for_gpu`, commit `6044cc1`) — `None` (→ CPU) for mask/clip/reference/masked-adj/non-ported-kind (op-list v1 can't represent those). REMAINING (STEP 2, Coord): tool `preview_layer_pixels` provider + bridge owns a `LayerCompositor` + the straight-rgba8unorm→premul-Rgba8UnormSrgb pass for `PreviewOverride` (the delicate render piece). Handoff `docs/HANDOFF_painter_gpu_preview_coord.md`.

**Bloom slider FPS (2026-06-06, Enio: "não pode cair FPS com um único efeito"):** TWO bugs. (1) Bloom/S-H/Noise/Halftone/ColorLookup had GPU kernels (render-side) but `gpu_spatial_code`/`gpu_code` returned None → `flatten_for_gpu` bailed → the WHOLE preview fell to CPU every slider frame. Fix = wire them (commit `7157aec`; `spatial_params` widened to `[f32;8]` for S-H's 8 params). (2) Even on GPU, my Bloom blur was a DIRECT separable = O(radius): 2048²/radius100 = 53ms (19 fps). Fix = **radius-independent Bloom** (commit `e5eb54c`): downsample the glow by a power-of-2 `factor` (so low-res blur radius ≤16px) → blur at low res → bilinear upsample. radius now FREE — r100 == r20; 2048²/r100 = 19→**94 fps** (5×); 1024² ~310 fps any radius. `factor==1` (radius≤16) degenerates to the direct blur (parity gate stays byte-exact vs `apply_bloom`); radius>16 is the pyramid (structural gate — a DIFFERENT valid algorithm from the Gaussian, so the CPU `apply_bloom` must converge to the same down/blur/up for masked-Bloom CPU/GPU consistency — impl is already downsampling). The residual full-canvas cost is the bandwidth floor (per-pixel passes), not radius.

Foundational (compositor/cache/GPU) = Coord. Handoffs: `docs/HANDOFF_painter_w4_compositor_cache_coord.md` + `docs/HANDOFF_painter_gpu_preview_coord.md`. Related: [[project-painter-w3-block2-persist-ktx2-2026-06-01]] (composite bandwidth-bound), [[project-painter-w4-spatial-gpu-bloom-sh]].
