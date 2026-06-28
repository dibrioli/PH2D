# HANDOFF — Per-Layer Color (layers-as-brush): slowness + rectangular stripe artifacts

> **Status:** OPEN. Evaluation only — no fix applied in this pass (Enio 2026-06-28).
> **Owner of next step:** a single implementer agent (this is a perf + correctness bug in the painter
> stamp path; stays inside `ph2d-tool-painter` + possibly `ph2d-painter-brush`).
> **Author of this handoff:** evaluated the whole path statically + verified the off-file invariant the
> previous diagnosis leaned on. Read §0 and §6 FIRST — they will save you a wasted round.

---

## §0 — TL;DR (read this)

**Two distinct problems, reported together but with different root causes.**

1. **Slowness** — *structural and well-understood.* The shape methods (Curve / Line / Circle / Polygon /
   Free Hand) are **non-incremental**: every pointer **Move** rebuilds the ENTIRE dab list (perimeter ÷
   spacing = hundreds–thousands of dabs) and re-stamps all of them. With **Per-Layer Color** each of those
   dabs is composited across **N≤16 layers** and the whole dirty bbox is **recomposited per move**. The
   non-per-layer fill avoids the N× + recomposite, which is why only Per-Layer Color is "extremely slow".
   The previous fix (`eb6b0470`) removed the *per-move full-canvas clone + N re-allocations* but did **not**
   remove the **per-move whole-shape re-stamp** or the **per-move O(bbox·N) recomposite** — those are the
   remaining cliff. This is the thing to actually fix.

2. **Stripe artifacts ("listras" tied to the drawing rectangles)** — *NOT yet isolated; the obvious
   suspect is REFUTED (see §6).* Static analysis shows the interactive preview path is **self-consistent**
   in steady state (the per-layer coverage maps are fully self-cleared every move — proof in §6). So the
   stripe is most likely either (a) a **performance-induced rendering artifact** (the per-move cost is high
   enough that the dirty-rect *partial GPU upload* / *partial composite* shows incomplete/torn frames along
   their rectangular boundaries), or (b) a real bug in a path **not exercised by the unit tests** (the
   commit/bake path, multi-move-per-frame accumulation, or the GPU-bridge sub-rect upload). **You must
   reproduce + instrument before touching code** (§3). Do not re-attempt the "stale coverage map" theory —
   it's been checked and disproven (§6).

**Strategic note from Enio:** the final solution may be the **GPU migration of the painting path** (planned
separately). This handoff gives you (i) a correct CPU-side fix that will make it usable now, and (ii) the
framing for the GPU migration so whichever happens first is informed. Pick per §4.

---

## §1 — Reproduce, then MEASURE THE SCALE (do this first — non-negotiable)

Lesson from prior painter rounds (`feedback_measure_perf_symptom_scale`, `feedback_tool_unit_green_integration_dead`):
**establish the millisecond number before chasing a cause.** A 4–16 ms/frame symptom is a different class of
bug than a ⅓-second stall. Bench-green ≠ live-green; instrument the ACTIVE path.

**Repro (needs the GUI + pen/mouse):**
1. Open the painter, capture ≥2 document layers as the brush **Shape** (the "Use as Brush Shape" / Shape
   layers capture), enable **Per-Layer Color** (`toggle_brush_shape_per_layer_color`).
2. Select **Stroke: Method = Curve** (or Line/Circle/Polygon/Free Hand).
3. Draw a LARGE shape and drag a control point / the radius around. Compare a **diagonal** line (worst:
   bbox ≈ whole canvas) vs a **horizontal/vertical** line (thin band). Watch FPS and the stripes.
4. Toggle Per-Layer Color OFF and repeat — confirms the N×/recomposite is the slow part, not the fill.

**Measure (build `--release`; dev is opt-level 0 and lies about perf — `project_painter_composite_perf`):**
- Time **one `curve_move`** end-to-end and break it into: (a) dab generation (`fill_*_preview`), (b)
  `stamp_drag_preview` save+restore memcpy, (c) `stamp_dabs_cached_color` / `_dynamic` accumulate, (d) the
  recomposite loop, (e) `take_preview_arc` composite + GPU upload. Print µs per phase for a diagonal vs a
  thin line, at small vs large shape. The DOMINANT phase tells you whether to attack the re-stamp, the
  recomposite, or the upload.
- Headless option: the GPU/parity tests run headless on Metal (`reference_gpu_tests_run_headless_metal`),
  and you can drive the tool's pointer API directly from a `#[test]` (see `tool/paint/tests.rs` —
  `on_canvas_pointer(cp(...))`) to time the CPU phases without the GUI. Only the *visual* stripe needs the
  pen; the *timing* does not.

Write the numbers into this file before proposing a fix.

---

## §2 — The slowness: root cause (structural)

### 2.1 Shape methods re-stamp the WHOLE shape every Move
- `curve_move` → `curve_refill` → `curve_fill` clones the full control polygon, flattens the **entire**
  spine, fills spaced dabs along **all** of it, and re-stamps:
  `crates/ph2d-tool-painter/src/tool/paint/curve.rs` — `curve_fill` (`stroke.fill_polyline_preview(&spine, &mut dabs); self.stamp_drag_preview(&dabs);`). The doc says "Fresh-per-fill ⇒ deterministic" — full rebuild **by design**.
- Circle: `circle.rs` `circle_refill` → `fill_ellipse_preview`. Polygon: `polygon.rs` `polygon_refill` →
  `fill_polygon_preview`. Line/FreeHand route the same way (`paint.rs` `stamp_stroke_dabs` →
  `stamp_drag_preview`; FreeHand re-fills the captured path through the curve editor).
- Contrast — **incremental** methods (Space/Dots/Airbrush): `paint_extend` → `stroke.extend(...)` appends
  only the few dabs since the last sample → `stamp_dabs` (cumulative, no restore). The brush keeps cursor
  state so each move emits only the new segment.

So per Move a shape emits `≈ perimeter / spacing` dabs and re-stamps **all** of them. There is no
"append only the changed part" for shapes today.

### 2.2 The drag-preview save/restore is O(union-bbox) memcpy, twice, per move
`stamp_drag_preview` (`paint.rs`): each move (a) `restore_region(prev.rect, prev.pixels)`, (b)
`save_region(bbox)`, (c) `stamp_dabs_inner`. `bbox` = **union of every dab's `dab_bbox`** ≈ the shape's
bounding box. Two full-bbox RGBA memcpy passes (save + restore) per move. For a **diagonal** line the union
bbox ≈ the whole canvas, so this alone is O(canvas) per move regardless of how thin the painted stroke is.
(`dab_bbox` = `floor(c−r) .. ceil(c+r)+1`; `union_region` is the min/max merge.)

### 2.3 Per-Layer Color adds an N× per-dab accumulate + an O(bbox·N) per-move recomposite
`crates/ph2d-tool-painter/src/tool/paint/stamp_color_cache.rs` (`stamp_dabs_cached_color`) and its dynamic
twin `stamp_color_dynamic.rs` (`stamp_dabs_per_layer_dynamic`). For one Move with D dabs, N layers,
P=canvas px, B=union-bbox area, S=dab footprint:
- **Per-dab accumulate:** `for d in dabs { for (i,stamp) in stamps { accumulate_color_stamp_coverage(...) } }`
  → **O(D · N · S)**. The dynamic path resamples silhouette×grain×colour per pixel per dab → heavier still.
- **Per-move recomposite** (`for py in bb.y.. { for px in bb.x.. { ... } }`): iterates the **whole bbox**;
  the zero-coverage skip is `if !cov.iter().take(n).any(|m| m[idx] != 0) { continue; }` — note the skip test
  is itself **O(N) on every empty pixel**. Covered pixels additionally pay the z-compose (O(N)) + one
  `blend_over` + the self-clear (O(N)). → **O(B · N)**, and for a diagonal B ≈ P.
- Total per move ≈ **O(D·N·S) + O(B·N)**, repeated EVERY pointer move. N up to `MAX_SHAPE_LAYERS = 16`
  (`shape_layers.rs`).

### 2.4 What `eb6b0470` fixed and what it left
Fixed (in the fill branch): the per-move **full-canvas clone** of `pre` and the per-move **N map
re-allocations** (now: maps reused, `pre` empty, base read from the live canvas which the drag-preview
already restored), plus the zero-coverage skip. Left untouched: **the whole-shape re-stamp** (2.1), **the
two O(bbox) memcpy passes** (2.2), and **the O(bbox·N) recomposite incl. the O(N)-per-empty-pixel skip**
(2.3). Those are the remaining cost. The fix made it *less catastrophic*, not *fast*.

---

## §3 — The stripe artifact: reproduce + instrument (do NOT guess)

The user describes **stripes tied to the rectangles of the drawing areas**, "probably a rectangular
optimization". The rectangular optimizations in play are: `dab_bbox`/`union_region` (save/restore +
dirty), the per-layer recomposite `bb`, the **dirty-rect partial GPU upload** (`preview_upload_bbox`), and
the **partial composite** (`composite_region` + `blit_region`). Static analysis (§6) shows the
preview-path coverage maps are self-consistent, so lead with instrumentation:

1. **Capture which pixels are wrong.** After a Move that visibly stripes, dump the active layer's
   `canvas_rgba` AND the composited preview to PNG (add a temporary debug hook). Diff against a
   **full-canvas recomposite** (force `bb = whole canvas`, disable the skip, disable partial upload).
   - If the **full recomposite** removes the stripe → the bug is in a **rectangular region bound**
     (which rect is too small): instrument `dab_bbox`, the per-layer `bb`, `dirty_rect`, and
     `preview_upload_bbox` and find which one fails to cover the changed pixels.
   - If the stripe persists under full recomposite → it's in the **stamp math / base read**, not a rect.

2. **Prime suspects, ranked (after §6 refutes the stale-cov theory):**

   **(A) Dirty-rect / partial-GPU-upload coupling (most consistent with "rectangular optimization").**
   `runtime.rs` `take_preview_arc` non-trivial-stack path: when a composite is cached and `dirty` is
   `Some(bbox)`, it recomposites ONLY `bbox` via `composite_region(...)` then `blit_region(cache, w,
   &region, bbox)`, and sets `preview_upload_bbox = Some(bbox)`. The bridge then uploads only that sub-rect
   of the GPU texture. **Verify:** does `dirty_rect` (accumulated across *all moves since the last drain*)
   cover EVERY changed composite pixel? Per move the changed pixels are `restore`d `prev.rect` ∪
   recomposited `bb`; both are `mark_dirty`'d — but confirm there is no move where a write skips
   `mark_dirty`, and that multi-move-per-frame accumulation unions correctly. A sub-rect upload that misses
   a band leaves a **stale rectangular stripe** on screen even though the CPU buffer is correct → exactly
   the reported symptom.

   **(B) `blit_region` source stride vs `composite_region` clamp (latent; trips if any bbox is unclamped).**
   `tool/internal.rs` `blit_region` strides the SOURCE by `bbox.w` (`src_off = ry*bbox.w*4`,
   `row_bytes = bbox.w*4`), while `composite_region_linear` (`compositor/compose.rs`) CLAMPS the region to
   `width-rx`/`height-ry` and returns a buffer sized `rw*rh`. Safe today only because `dab_bbox`/`union_region`
   keep the bbox in-bounds (so `rw == bbox.w`). If ANY dirty rect reaches `take_preview_arc` un-clamped
   (e.g. a future effect, or an off-by-one), the source stride ≠ the buffer width → **sheared/striped rows**
   (or OOB panic). Add a debug assert `region.len() == bbox.w*bbox.h*4` and clamp defensively.

   **(C) `dab_bbox` ⊇ brush write-bounds invariant (documented prior stripe family).**
   `paint.rs` `dab_bbox` comment records the exact "thin horizontal trail" symptom from a
   `round±(ceil(r)+1)` box that **missed the high edge by 1px**. Current formula `floor(c−r)..ceil(c+r)+1`
   matches the brush-side accumulate bounds (`stamp_color.rs` — same `floor/ceil()+1`), so it's consistent
   *now*, but it is **load-bearing**: if anyone changes either side, thin-line stripes return. Re-verify the
   two formulas are byte-identical before/after any edit. (This is most likely to bite the **horizontal /
   vertical** line case, where a 1px edge miss is a full-width visible stripe.)

   **(D) Performance-induced tearing.** If §1's measurement shows the per-move cost ≫ frame budget, the
   "stripes" may simply be partial-frame presentation during the stall. Fixing §2 (perf) makes them vanish.
   Rule this in/out by checking whether the stripe survives when you artificially cap the shape size (few
   dabs, sub-millisecond move).

3. **Tiling amplifier (note, not necessarily the cause).** `stamp_drag_preview` replicates dabs for Tiling
   BEFORE the bbox fold, so a dab near one edge + its wrapped copy near the opposite edge make
   `union_region` span the **full canvas width**. This widens save/restore/recomposite/upload to an
   edge-to-edge band — magnifying both §2 perf and any §3 rectangular-bound bug. Reproduce with Tiling OFF
   first to isolate.

---

## §4 — Recommended solutions (pick by §1's numbers)

### 4.1 CPU-side, ship-now fixes (do these unless the GPU migration is imminent)

Ordered by leverage; each is independently shippable.

- **(I) Stop re-stamping the whole shape every move — re-stamp only the CHANGED span.** The biggest win.
  When a single control point / radius / side-count changes, only the dabs near the edited region change.
  Options: (a) diff the new spine vs the previous spine and re-stamp only the changed arc (restore just that
  sub-region); (b) keep the brush-side `Stroke` and re-`extend` only the delta. This collapses per-move cost
  from O(whole shape) to O(edited span) and benefits ALL shape methods, per-layer or not.

- **(II) For the CACHED path, pre-bake ONE colored premultiplied stamp and blit it per dab like a normal
  textured brush — drop the per-layer maps + recomposite entirely WHEN it is correct to do so.** The reason
  the code keeps N maps + a stroke-wide recomposite is the invariant "the top layer paints above ALL
  accumulated lower-layer coverage across the whole stroke" (so overlapping dabs composite layer-wise, not
  dab-wise). For **non-overlapping** dabs (spacing ≥ tip size) or when the user doesn't need stroke-wide
  z-mixing, a single baked colored stamp is byte-equivalent and ~N× cheaper + no recomposite. Detect the
  safe case (spacing vs footprint) and fast-path it; fall back to the current path only when dabs overlap.
  (Validate against `per_layer_color_top_layer_paints_above_all_lower_painting_across_the_stroke`.)

- **(III) Kill the O(N)-per-empty-pixel skip + the O(bbox) sweep.** Track, per move, the **union of the
  covered sub-rects** (you already have each `accumulate` return rect) and recomposite only the tight cover,
  not the whole `bb`. Or maintain a 1-bit "touched" mask. Removes the diagonal-line "bbox ≈ canvas" sweep.

- **(IV) Defensive correctness:** clamp every rect handed to `take_preview_arc`/`blit_region` to the canvas
  and assert `region.len() == bbox.w*bbox.h*4` (§3-B); add a debug "full recomposite" toggle to bisect
  stripes; keep `dab_bbox` ≡ the brush accumulate bounds (§3-C) under an executable check, not a comment.

### 4.2 Strategic: GPU-resident painting (the planned migration)

Today the **layer compositor** has GPU paths for adjustments/blend (`ph2d-painter-effects` WGSL,
`apply_adjustment`, ~32× vs CPU on Metal — `project_painter_composite_perf`), but **dab stamping into the
active layer is CPU** (`stamp_color_cache.rs`/`_dynamic.rs`). The per-layer-color recomposite is an
embarrassingly-parallel per-pixel op over a bbox — an ideal compute-shader kernel:
- Upload the N layer stamps + colors once; for each dab batch run a `cs_stamp` that accumulates per-layer
  coverage and composites in one pass; keep the active layer GPU-resident so there is no per-move readback.
- This dissolves BOTH problems: the recomposite is GPU-parallel (perf) and the dirty-rect/partial-upload
  CPU dance (the stripe surface) goes away (the layer lives on the GPU; you blit dab-local regions).
- Caveats from prior GPU painter work (`project_watercolor_v2_gpu_first_refactor`,
  `project_painter_fluid_4k_perf_architecture`): the cost was **submit/readback-bound**, not compute — so
  the migration must be **single-submit / direct-render / no per-stroke readback**, not a naive port. Shape
  parity (`gpu_parity`-style) must be proven against the CPU kernels bit-for-bit before it can replace them.

**Decision rule:** if §1 shows the dominant cost is the **re-stamp + memcpy** (CPU orchestration), do 4.1-I
first — it's small and helps everything. If the dominant cost is the **recomposite kernel** at large
canvases and the GPU migration is on the near roadmap, fold this into that migration rather than
micro-optimizing the CPU kernel twice.

---

## §5 — File map + invariants to preserve + tests

**Files (all `crates/ph2d-tool-painter/src/tool/paint/` unless noted):**
- `shape_layers.rs` — `ShapeLayers` (captured stack, `per_layer_color`, per-layer `color_on`/`color`,
  `version`); `is_color_mode()` gate; `resolved_colors`; `MAX_SHAPE_LAYERS = 16`.
- `stamp_route.rs` — routing: `is_color_mode() && shape_silhouette_active` → dynamic if any per-dab
  dynamic (Shape Rake/Random, Grain jitter-rotate, Randomize Color, canvas-fixed Grain) else cached.
- `stamp_color_cache.rs` — CACHED path (`stamp_dabs_cached_color`, `ensure_color_stamp_cache`,
  `PerLayerStroke{pre, cov}`); 1 B/px coverage maps.
- `stamp_color_dynamic.rs` — DYNAMIC per-dab path; 4 B/px premul-RGBA maps.
- `paint.rs` — `stamp_drag_preview`, `save_region`/`restore_region`, `dab_bbox`, `union_region`,
  `mark_dirty`, `DragPreview`.
- `curve.rs`/`circle.rs`/`polygon.rs` — `*_refill` / `*_fill` (whole-shape re-emit).
- `tool/runtime.rs` — `take_preview_arc` (trivial vs non-trivial; partial recomposite + `preview_upload_bbox`).
- `tool/internal.rs` — `blit_region`. `compositor/compose.rs` — `composite_region_linear` (the clamp).
- `crates/ph2d-painter-brush/src/stamp_color.rs` — `accumulate_color_stamp_coverage` /
  `accumulate_shape_layer_rgba` (the per-dab kernels; their returned `DirtyRect` is a SUPERSET of bytes
  written — see §6).

**Invariants you must not break (executable-gate them if you touch the path):**
1. `dab_bbox` (save/restore/upload region) ⊇ the brush-side accumulate write bounds — both are
   `floor(c−r)..ceil(c+r)+1` today; a divergence reopens thin-line stripes (§3-C).
2. Every canvas write goes through `mark_dirty`, so `dirty_rect` ⊇ all changed pixels for the partial upload.
3. The per-layer recomposite/clear must cover every coverage byte written this move (holds because the
   accumulate return rect is a superset — §6).
4. Z-order/blend/opacity output is identical to the current 2-stage composite (layers source-over among
   themselves, then `brush.blend` once onto the base).

**Tests guarding correctness (keep green; add a perf-shape variant):**
`per_layer_color_top_layer_paints_above_all_lower_painting_across_the_stroke`,
`per_layer_color_respects_brush_blend_mode`, `per_layer_color_dynamic_randomize_color_tints_per_dab`,
`per_layer_color_fill_method_uses_canvas_base_and_self_clears` (the `eb6b0470` guard),
`resetting_the_shape_clears_the_per_layer_color_state`.

---

## §6 — Dead ends (don't repeat) + the verified invariant

- **"Stale coverage maps leave residue" — REFUTED.** The previous diagnosis suspected the per-move
  self-clear (`if !incremental { for m in cov { m[idx]=0 } }`, gated by the zero-coverage skip + the `bb`
  loop) leaves coverage bytes uncleared outside `bb`, which resurface as stripes. **This was checked and
  does not happen.** `bb` is the union of `accumulate_color_stamp_coverage`'s **returned rects**, and that
  function writes bytes ONLY within `[x0,x1)×[y0,y1)` (it `continue`s on `a <= 0`) and **returns that exact
  rect whenever it wrote anything** (`touched` flag; returns `None` only if it wrote nothing). Therefore
  `bb ⊇ every byte written this move`, so the self-clear inside `bb` zeroes **every** byte the move set. The
  maps are globally clean after each fill move. Combined with the drag-preview `restore_region` reverting
  the previous footprint to pristine, the interactive preview path is **self-consistent in steady state**.
  Do not spend a round re-deriving this — instrument the runtime instead (§3).
- **Micro-optimizing the recomposite inner loop without removing the whole-shape re-stamp** — the prior
  round already trimmed allocations; the remaining cost is the re-stamp + the O(bbox·N) sweep, not the
  per-pixel constant. Attack §4.1-I/III, not the arithmetic.
- **Assuming dev-build timings** — measure `--release`; dev opt-0 inverts which phase dominates.
- **"Fix the stripe" before reproducing it** — `feedback_visual_bug_debug` /
  `feedback_gizmo_verify_hit_target_before_transform_math`: capture the wrong pixels + bisect with a
  full-recomposite toggle FIRST; the rectangular boundary will point at the offending rect.

---

## §7 — Suggested order of work for the next agent

1. Repro + **measure** (§1); write the µs-per-phase table into this file.
2. Repro the stripe; **capture wrong pixels** + bisect with a forced full recomposite (§3-1) to classify it
   as a rect-bound bug (which rect) vs perf-tearing vs stamp-math.
3. If perf-bound: implement §4.1-I (re-stamp only the changed span) and re-measure. Likely also kills the
   stripe if it was tearing.
4. If a specific rect is too small: fix that bound (most likely the partial-upload `preview_upload_bbox` /
   `dirty_rect` coupling, §3-A) + add the defensive clamp/assert (§3-B).
5. Re-run the §5 tests + add a large-diagonal-shape perf-shape test (assert the recomposite touches O(cover)
   not O(canvas)).
6. If the numbers say the recomposite kernel is the wall and the GPU migration is near — hand the kernel
   spec (§4.2) to that effort instead of optimizing it twice.

*Everything in §2/§3/§6 is from a static read of the current tree + verification of the brush-side
accumulate invariant; §1's live numbers are still owed and gate the choice in §4.*
