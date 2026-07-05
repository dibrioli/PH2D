# HANDOFF — Per-Layer Color (layers-as-brush): slowness + rectangular stripe artifacts

> **UPDATE 2026-07-04 (noite) — CPU chegou ao teto NOVO: o kernel foi paralelizado por bandas; pior caso
> 105 ms → ~8 ms/move.** Medido em `--release` no Ryzen 9950X (32 threads): baseline pós-fused
> `per_layer_perf_worst` = **95,4 ms/move** (vs 105,5 no Mac — igual nas duas máquinas porque o kernel era
> SERIAL, a dica do Enio); depois de `accumulate_color_stamps_fused_batch` (uma chamada por Move, dabs ×
> layers em BANDAS de linhas disjuntas — bit-idêntico por construção, gate
> `batched_fused_accumulate_is_bit_identical_to_sequential`) + o recomposite band-paralelo no tool =
> **7,9 ms/move**; o sweep inteiro (256²/1024² × r8..100 × N2/16) fica ≤ **8,8 ms/move**. No Mac (8 cores)
> a projeção é ~15–25 ms no extremo — bem melhor, mas o **plano GPU (§4.2) continua o endgame para 4K**.
> §4.1-I (re-stamp só do span editado) foi **avaliado e NÃO implementado**: exige clip de kernel + diff de
> topologia (regra two-strikes) e não melhora o pior caso (linha de 2 pontos re-stampa tudo); a
> paralelização por bandas cobre TODOS os casos, inclusive esse, com risco menor. Próximo gargalo CPU
> visível no sweep: `take_preview_arc` em stack não-trivial (composite_region, 4–12 ms) — só aparece com
> doc-layers extras; fica para a migração GPU do preview.

> **UPDATE 2026-07-04 — GPU is now the PLAN OF RECORD; CPU bridge micro-opt reverted.**
> The cross-tool bridge change `2c64ba80` ("kill the full-canvas Arc deep-copy per move") was **reverted**
> (`461dcafd`): live smoke showed it **regressed BOTH Per-Layer Color AND Warp together** — the tell-tale
> of a shared display-path change (it was the only local edit touching `painter_bridge.rs`). The
> in-place-`make_mut` win was theoretical and never visually confirmed; empirically it backfired, so it's
> out. **No more CPU micro-opt on this path** — per Enio, Per-Layer Color goes GPU. The implementation
> target is **§4.2 (GPU-resident painting)** below; that migration deletes the whole CPU dirty-rect
> machinery (`stamp_color_cache.rs`/`_dynamic.rs` accumulate + the per-move recomposite + the partial-upload
> dance) and the perf cliff dissolves there. §1.R's numbers stand as the CPU baseline to beat; the extreme
> `r100·N16` = 105 ms case was already flagged GPU-only. Do NOT reland a CPU Arc/upload optimization on the
> shared bridge without a per-tool visual smoke of Warp *and* the paint path first.
>
> **Status:** **CLOSED ON CPU — moving to a full GPU painter migration (Enio 2026-06-28).** CPU work is
> done; no further CPU mitigations (Enio explicitly declined spacing/brush/layer guidance). See §1.R for
> the measured story and the FOLLOW-UP block below for the live-smoke results that closed it.
>
> **What landed (CPU):** (1) fused alpha-only accumulate kernel = **3.2–4.5×** on the 96.5% bottleneck
> (§1.R); (2) **per-frame pointer coalescing** for the restore-based fill methods (Curve/Line/Circle/
> Polygon) in the shell — collapses the per-event whole-shape re-stamp storm to ONE stamp/frame; (3) a
> **widened HUD `paint ms`** (all stamps + dispatch, not just the flush) + an `ev/stamp` counter + a
> `PH2D_PAINT_FULL_UPLOAD` bisection toggle, for ongoing diagnosis; (4) **rectangular-artifact RESOLVED** —
> clear-on-alloc of the GPU preview slot (see FOLLOW-UP). The whole Bug #2 is now closed on CPU.
>
> **Live smoke verdict (Enio):** at the tested config (Shape image 512² × 3 layers, painting a 1024²
> sprite, **cached path** — no per-dab dynamics) FPS is still low because a SINGLE cached stamp ≈ 110 ms;
> coalescing can't beat single-stamp cost, and small Spacing (<0.1 → huge dab count) + big brush (S∝r²)
> dominate `O(D·N·S)`.
>
> **Artifact — REFINED diagnosis (Enio drew a mockup 2026-06-28).** Symptom: thin horizontal **slivers of
> the actual shape content** appear below the shape at rect-aligned positions, **transient (gone within a
> frame), and ONLY on the first few uses of a shape — then never again after many shapes.** It **persists
> under `PH2D_PAINT_FULL_UPLOAD=1`** → it is in the CPU preview buffer, NOT the partial GPU upload (§3-A
> upload **RULED OUT**) and NOT perf-tearing (which would be persistent at low FPS, not first-uses-only).
> Root: a **stale preview BASE in the dirty-rect "recompose-only-the-brush-bbox" optimization**
> (`take_preview_arc` patches only `dirty_rect` into the persistent `composited` cache; before that cache
> is fully overwritten early in a session, un-recomposed regions show stale content — self-heals once the
> base is fully written, hence "first few times then stops"). The common restore/recompose paths are
> tested-correct (`*_leaves_no_trail`, `*_reverts_all_pixels` green); this is a rare early-session
> warmup transient. **NOTE for the FULL_UPLOAD toggle:** it forces full *upload* but NOT full *recompose*
> — a stale `composited` cache uploads stale-full, so the toggle can't clear this class (a
> `PH2D_PAINT_FULL_RECOMPOSE` toggle would; not added — see decision).
>
> **Mitigation LANDED (Enio asked for it 2026-06-28):** `PainterTool::reseed_preview_base()`
> (`tool/layers/cache.rs`) forces a full recompose + full upload on the FIRST frame of every shape
> session — wired into the "no session yet" creation block of Curve / **Free Hand** (shares the Curve
> editor) / Circle / Polygon. So a new shape never patches a possibly-stale `composited` cache; it always
> starts from a byte-correct base. Lighter than `invalidate_composite` (no `edited_since_bind`, no
> adjustment-cache drop — nothing was painted yet). Cost: one full recompose/upload per shape creation
> (a user click) — imperceptible. 238 tests green. This closes the early-session sliver window on CPU.
>
> **ARTIFACT RESOLVED — clear-on-alloc was the fix all along (Enio confirmed 2026-06-29: "tested several
> times, the bug/artifact did not reappear", `play.command` clean rebuild).** Signature was an uninitialized
> GPU read (virtual rectangle; garbage only the FIRST time a region is painted; clean forever after;
> NON-deterministic — undefined memory is sometimes transparent, sometimes visible). Root cause: the
> `IndividualTextureStore` slot (sprite samples it via `PreviewOverride`) was created WITHOUT clearing; the
> GPU-preview path acquires it empty (`acquire_empty`) and fills it by a later region copy, so a region
> sampled before the first copy reads garbage. **Fix:** `clear_all_mips_transparent`
> (`individual.rs::create_entry_empty` → `texture_clear.rs`) — render-pass clear of EVERY mip on creation
> (`regen_mips` runs only after the first upload, so all levels must be seeded). Guard:
> `acquire_empty_slot_reads_back_transparent_not_garbage` (empty slot now reads all-zero).
>
> **Lesson — the false negative cost 3 rounds.** The slot clear was the FIRST right hypothesis, but a
> STALE BINARY ("alarme falso, ainda existe") made me discard it and chase `out`/premul (verified clean —
> `cs_flat`/`cs_main` write every texel) + runtime repro. A non-deterministic bug + incremental build =
> "still appears" can just be the OLD binary. **Verify a CLEAN rebuild before declaring a fix dead.**
>
> **GPU-migration caveat (still captured):** the GPU painter MUST clear every preview texture on alloc and
> seed the FULL base every session — never rely on "the first write covers it". The GPU migration (§4.2)
> deletes the whole CPU dirty-rect machinery; the perf cliff dissolves there.
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

## §1.R — MEASURED (2026-06-28, `--release`, Apple Silicon 8 GiB)

Harness landed: `crates/ph2d-tool-painter/src/tool/paint/tests.rs` mod `per_layer_perf`
(`#[ignore]`, drives the real `on_canvas_pointer` API, no GUI). Reproduce:
```
cargo test -p ph2d-tool-painter --release per_layer_perf_sweep -- --ignored --nocapture
PH2D_PAINT_PROF=1 cargo test -p ph2d-tool-painter --release per_layer_perf_worst -- --ignored --nocapture
```
Each row = µs per pointer-Move (a Curve in draw mode re-fills the whole anchor→cursor line every
Move). Diagonal vs Horizontal are EQUAL length ⇒ equal dab count, so D/H isolates bbox-bound cost.

| canvas | radius | N layers | **move µs** | D/H ratio | take_preview_arc µs |
|---|---|---|---|---|---|
| 1024² | 8  | 2  | 6 300 | 1.2× | ≤3.6 ms |
| 1024² | 8  | 16 | 49 400 | 1.2× | ≤3.6 ms |
| 1024² | 40 | 16 | 194 000 | 1.0× | ≤4.6 ms |
| 1024² | 100| 2  | 60 000 | 1.0× | ≤6.5 ms |
| 1024² | 100| 16 | **474 000** | 1.0× | ≤6.5 ms |

**Phase split at the worst config (1024² · r100 · N16 · diagonal, 31 dabs, bb 626×626):**
`accumulate_us ≈ 454 000 (96.5%)` · `recomposite_us ≈ 16 200 (3.4%)` · `take_preview_arc ≈ 0.4 ms`.

### AFTER the CPU constant-win (`accumulate_color_stamps_fused`, 2026-06-28)
Fused per-layer pass: all stamps share `size` ⇒ bilinear coords + the 4 texel offsets computed ONCE per
canvas pixel (not ×N), and only the ALPHA channel is sampled (per-layer-colour discards the stamp RGB).
Byte-identical to the old per-layer loop — gate `fused_per_layer_accumulate_is_bit_identical_to_sequential`
in `ph2d-painter-brush`. Re-measured `move µs` (1024²):

| radius·N | before | after | × |
|---|---|---|---|
| r8 · N2  | 6 300 | 1 982 | 3.2× |
| r8 · N16 | 49 400 | 11 712 | 4.2× |
| r40 · N16 | 194 000 | 44 504 | 4.4× |
| r100 · N16 | 474 000 | 105 557 | 4.5× |

Moderate (N≤4, r≤40) now 2–12 ms (usable). The extreme r100·N16 = 105 ms is the deferred GPU case (§4.2).

### What the numbers PROVE (and what they REFUTE)
- **The bottleneck is ONE kernel: `accumulate_color_stamp_coverage` = 96.5%.** D dabs × N layers ×
  the ~(2r)² footprint, each a discarded-RGB bilinear `sample_color_mask` (~22 ns/px). Cost ≈
  **O(D · N · S)**, re-done for the WHOLE shape every Move.
- **Cost ∝ N (linear):** ×7.9 from N=2→16. The per-layer loop is the multiplier.
- **Cost ∝ radius (≈linear):** via D·S = length·radius (spacing scales with radius, so D∝1/r, S∝r²).
- **NOT bbox-bound — REFUTES §2.2 + §2.3 as the cliff.** D/H ≈ 1.0 even though the diagonal bbox is
  ~17× the horizontal: the save/restore memcpy (§2.2) and the O(bbox·N) recomposite *sweep* incl. the
  O(N)-per-empty-pixel skip (§2.3) are **negligible**. The zero-coverage skip already works.
- **`take_preview_arc` / dirty-rect / partial-upload is NOT the perf cliff** (≤6.5 ms, ~0 on a trivial
  doc stack). The §3-A "rectangular optimization" is **not where the time goes**.
- **Stripe (§3): evidence favors §3-D (perf-induced tearing).** The composite/upload path is
  self-consistent and cheap; a 50–474 ms/Move stall tears frames. Still owes a visual smoke to confirm,
  but it is NOT a rect-bound bug in the measured paths. Fixing the accumulate should dissolve it.

### Consequence for §4 (revised ranking)
- §4.1-**III** (tight recomposite / kill the skip) — **LOW value**: it targets the 3.4% recomposite,
  not the 96.5% accumulate. Drop it.
- §4.1-**I** (re-stamp only the changed span) — reduces D, but only for LOCAL edits on multi-point
  curves; the worst measured case is a 2-point line whose whole geometry re-stamps every Move → no
  help there. Medium value, high risk (topology rebuild → two-strikes).
- **Cheap CPU constant-win (NEW, do first):** the accumulate discards RGB yet samples it. An
  alpha-only `sample_color_mask` + radial early-out (skip the ~21% out-of-disk box corners pre-sample)
  + fuse the N per-layer passes into ONE footprint pass (compute (u,v)/idx once, inner loop over N).
  Estimated ~2–3× on the 96.5% kernel. Zero invariant change, isolated to `ph2d-painter-brush` +
  the `stamp_dabs_cached_color` call site. Makes MODERATE configs (N≤4, r≤40) usable; does **not**
  save the extreme.
- **§4.2 GPU is the only path to interactive at the extreme** (big brush × N16 × 4K). The 96.5% kernel
  is 20 M independent per-pixel samples — an ideal compute shader (`cs_accumulate` per dab-batch,
  per-layer coverage in parallel, single-submit, no readback). Matches Enio's strategic note. Needs a
  bit-parity gate vs the CPU kernel before it can replace it.

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
