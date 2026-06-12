//! W15.3 GPU fluid drive (ADR-0049, feature `fluid`) — steps the painter's live
//! wet field on the GPU AND composites the Kubelka–Munk glaze on the GPU each
//! frame, removing the per-frame pigment readback + the CPU composite (the stalls
//! that capped large-canvas perf).
//!
//! Feature-gated: the default build excludes this module (and `ph2d-painter-fluid`),
//! so the painter runs the CPU diffusion+composite path. With `--features fluid`
//! the render loop calls [`drive_fluid_gpu`] right after the active tool's `on_tick`.
//!
//! ## The resident-composite flow (per frame, no pigment readback)
//! 1. The pigment stays GPU-resident in the solver's `pig_a`; this frame's dabs
//!    (the tool grid's pigment) are uploaded as an additive `deposit` and the field
//!    diffuses/advects ON the GPU. Water is the CPU mirror (uploaded for the gate;
//!    the CPU owns evaporation + the dry-check) — so no water readback either.
//! 2. The GPU compositor reads `pig_a` directly + the pre-stroke backdrop → the
//!    canvas RGBA, and we read back ONLY the wet row band into `canvas_rgba` (the
//!    canonical layer the existing preview upload + Apply/undo consume).
//!
//! Downcasts to the concrete `PainterTool` (allowlisted bridge, same exception
//! class as `painter_bridge.rs`): the GPU drive needs the concrete fluid hooks.

use super::painter_fluid_support::{
    FluidSession, PROFILE, grow_bbox, run_readback_lane, union_bbox,
};
use super::sim_extract::PreviewOverride;
use ph2d_editor::ToolRegistry;
use ph2d_gpu::GpuContext;
use ph2d_painter_fluid::DabGpu;
use ph2d_render::SpriteRenderer;
use std::cell::RefCell;
use std::time::Instant;

/// How often (in frames) the resident path reads the GPU field stats back for the
/// dry-check. Drying takes ~0.3 s (≈ 18 frames @ 60 Hz), so a few frames of latency
/// on the drop is invisible — and now that the stats readback is PIPELINED
/// (`read_field_stats_pipelined`, async — returns the prior fire's stats with no
/// `poll(wait)`), this cadence no longer gates any blocking queue-drain. It only
/// sets how often the dry-check / wet-bbox refresh; the composite envelope is grown
/// from the dab list, not from this. (Before pipelining, the blocking `poll(wait)`
/// here was a multi-ms queue-drain that *grew* into a runaway FPS collapse.)
const DRY_CHECK_EVERY: u64 = 20;

/// **Watercolor v2 (ADR-0085) — active-region window.** A grid area stays in the per-frame
/// WORK region (sim + composite) for this many frames after the last dab landed there, then
/// freezes (its composite persists in the preview texture). ~1.5 s at 60 fps — long enough for
/// the bloom to develop, short enough that a settled wash stops costing.
const ACTIVE_WINDOW_FRAMES: u64 = 90;
/// Pad (grid cells) grown around the active-dab window to cover the capillary wick advancing
/// outward from the recent dabs.
const ACTIVE_REGION_PAD: u32 = 48;

/// **Idle decimation cadence (perf block 2b).** With a live field but the pointer up and
/// no dabs, the sim step + composite/sheen run only every Nth frame — idle the field
/// barely moves (drying, or near-equilibrium under Keep Wet), yet stepping + recompositing
/// the whole wet envelope EVERY frame was the dominant idle GPU cost (§4b: `comp_tex` +
/// `comp_pipe` + step ≈ the entire present-stall). Skipped frames republish the persistent
/// preview texture as-is (nothing changed to recomposite). The visible effect is idle
/// dynamics evolving at ⅓ wall-clock rate — drying cadence is low-stakes (the handoff's
/// "aceitável-a-validar"). Painting frames (dabs or stroke) are NEVER decimated.
const IDLE_STEP_EVERY: u64 = 3;

/// First idle frames after pen-up that always run at full cadence (no decimation), so the
/// pipelined readback gets immediate shots at landing the E4/E5 catch-up bake into
/// `canvas_rgba`. NOT a correctness gate — `flush_pending_bake` (pointer-down) is the
/// correctness backstop; under GPU backpressure the pipelined bands return erratically
/// (empty/non-empty interleave), so any "wait until caught up" condition can stall forever
/// (the 2026-06-10d bug: decimation gated on `texture_mode_dirty == None` never engaged).
const IDLE_WARMUP_FRAMES: u64 = 6;

thread_local! {
    /// Rebuilt when the field resizes (a new canvas); reset (resident pigment zeroed
    /// + paper re-uploaded) on a new stroke epoch; dropped when no live field.
    /// Thread-local because the render loop is single-threaded (the established
    /// painter-bridge pattern for per-tool GPU state).
    static SESSION: RefCell<Option<FluidSession>> = const { RefCell::new(None) };
}

/// **Undo correctness (deferred-bake flush).** With the GPU texture lanes (E4/E5)
/// `canvas_rgba` is baked LATE — the per-frame catch-up readback only lands the wet
/// band a few frames AFTER pen-up. If a NEW stroke begins before that bake finishes,
/// the next stroke's undo pre-image (`begin_stroke` clones `canvas_rgba`) would
/// snapshot a STALE document missing the previous stroke's paint, so one undo could
/// revert several strokes. Call this in the pointer-down handler BEFORE
/// `PainterTool::begin_stroke`: it composites the whole pending catch-up union
/// SYNCHRONOUSLY into `canvas_rgba`, so each stroke's snapshot brackets exactly one
/// stroke. The per-frame deferral exists only to avoid a MID-stroke stall; one sync
/// pass at stroke-start (~0.14 ms) is invisible. No-op when no field / no pending
/// bake (the common case — single strokes already drained by the drying frames).
#[cfg(feature = "fluid")]
pub(crate) fn flush_pending_bake(gpu: &GpuContext, painter: &mut ph2d_tool_painter::PainterTool) {
    SESSION.with(|cell| {
        let mut slot = cell.borrow_mut();
        let Some(sess) = slot.as_mut() else {
            return;
        };
        let Some(region) = sess.texture_mode_dirty.take() else {
            return;
        };
        // `composite_frame` is the SYNCHRONOUS variant (composites + reads back the
        // band in one submission); one pass over the union covers everything the
        // texture lane skipped. Needs the compositor's stroke state alive — true
        // exactly in the case that matters (a new stroke starting while the prior
        // field is still wet; a dried field's catch-up already completed).
        let (band, rect) = sess
            .compositor
            .composite_frame(&gpu.device, &gpu.queue, region);
        if !band.is_empty() {
            painter.fluid_apply_gpu_composite_rows(&band, rect);
        }
    });
}

/// Drive + composite the live wet field on the GPU, called each frame after the
/// active tool's `on_tick`. No-op without an active painter or a live field (and it
/// then releases the session + hands the field back to the CPU path).
///
/// **E4 (ADR-0078 S2) return:** `Some(PreviewOverride)` when the fluid preview
/// SLOT holds the freshest composite this frame (mid-stroke texture mode + the
/// short texture→readback hand-off) — the caller gives it precedence over the
/// CPU-uploaded painter preview in `sim_extract`. `None` = the existing
/// readback → `canvas_rgba` → CPU-preview path owns the frame.
/// `override_entity` is the entity whose source the painter holds
/// (`last_painter_pushed_entity`) — the sprite the override suppresses.
///
/// ADR-0085: the E5 mid-stroke straight-texture lane is removed — a non-trivial
/// GPU-representable stack falls to the readback lane (the layer-preview driver +
/// `canvas_rgba`), so this fn no longer touches the `painter_gpu_preview` slots.
pub(crate) fn drive_fluid_gpu(
    tools: &mut ToolRegistry,
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    override_entity: Option<u64>,
) -> Option<PreviewOverride> {
    let painter = tools.active_mut().and_then(|t| {
        t.as_any_mut()
            .downcast_mut::<ph2d_tool_painter::PainterTool>()
    })?;
    // **Graceful degrade (ADR-0049 §2.8/§2.9).** Only a software/CPU adapter is ruled
    // incapable; every real GPU stays eligible. VRAM-free probing isn't portable in
    // wgpu, so the floor is assumed met on any real GPU (refine with telemetry). When
    // incapable, the field falls back to the CPU path (the tool's on_tick).
    let tier = match gpu.adapter.get_info().device_type {
        wgpu::DeviceType::Cpu => ph2d_host::MemoryTier::Low,
        wgpu::DeviceType::IntegratedGpu | wgpu::DeviceType::VirtualGpu => {
            ph2d_host::MemoryTier::Mid
        }
        _ => ph2d_host::MemoryTier::High,
    };
    let capable = ph2d_host::MemoryBudget {
        vram_free_mb: 256,
        tier,
    }
    .fluid_capable();
    // W15.3 full-res: a capable GPU runs the NEXT fluid field at full canvas
    // resolution. Set every frame (even with no live field) so it's in effect before
    // the next `begin_stroke`.
    painter.set_fluid_hires(capable);

    let eligible = ph2d_painter_fluid::fluid_pass_eligible(true, capable, f32::INFINITY);
    if !eligible || !painter.fluid_brush_enabled() {
        // Not a GPU-fluid scenario → hand back to the CPU path + free the session
        // (releasing its E4 preview slot first — refcounted store).
        painter.set_gpu_fluid_driven(false);
        SESSION.with(|s| {
            let mut slot = s.borrow_mut();
            if let Some(sess) = slot.as_mut() {
                sess.release_preview_slot(renderer);
            }
            *slot = None;
        });
        return None;
    }

    // ── PRE-WARM ──────────────────────────────────────────────────────────────
    // Fluid brush selected but no live field yet (hovering / between strokes): build
    // the solver + compositor NOW so the big composite shader compiles BEFORE the
    // first dab — no hitch when the stroke starts. Keep the session warm.
    if !painter.has_wet_field() {
        painter.set_gpu_fluid_driven(false);
        // Pre-generate + cache the paper-tooth field NOW (hovering) so the first
        // `begin_stroke` (the click) doesn't pay the O(grid) `grain_noise` (the ~⅓ s
        // click→stroke delay at 4K). Off the click path.
        painter.fluid_prewarm_paper();
        if let Some(dims) = painter.fluid_prewarm_dims() {
            SESSION.with(|cell| {
                let mut slot = cell.borrow_mut();
                if slot.as_ref().map(|s| s.dims) != Some(dims) {
                    if let Some(old) = slot.as_mut() {
                        old.release_preview_slot(renderer);
                    }
                    *slot = Some(FluidSession::new(&gpu.device, dims));
                }
            });
        }
        return None;
    }

    painter.set_gpu_fluid_driven(true);
    let dims = painter.fluid_grid_dims()?;
    // Watercolor v2 (ADR-0085): the PAINTING sub-step count (1) while a stroke is live — the
    // GPU drive once ran the idle count (2) for both, doubling the ~40-pass chain every painting
    // frame. Idle keeps 2 (it's decimated anyway).
    let substeps = if painter.is_stroke_active() {
        painter.fluid_painting_substeps()
    } else {
        painter.fluid_idle_substeps()
    };
    let epoch = painter.fluid_stroke_epoch();
    let scale = painter.fluid_field_scale();
    let coverage_k = painter.fluid_coverage_k();
    let (cw, ch) = painter.source_size();
    if cw == 0 || ch == 0 {
        return None;
    }

    let profile = PROFILE.with(|p| p.borrow_mut().enabled());
    SESSION.with(|cell| {
        let mut slot = cell.borrow_mut();
        // (Re)build the session on a grid-size change (pre-warm usually already did);
        // the old session's E4 preview slot is released first (refcounted store).
        if slot.as_ref().map(|s| s.dims) != Some(dims) {
            if let Some(old) = slot.as_mut() {
                old.release_preview_slot(renderer);
            }
            *slot = Some(FluidSession::new(&gpu.device, dims));
        }
        let sess = slot.as_mut()?;
        // ── New stroke (epoch change): cheap per-stroke setup, ONCE ──
        // (Extracted to `painter_fluid_drive` for HR-18; clears the resident field, uploads the
        // brush params/paper/lift donor + binds the compositor. No-op mid-stroke.)
        super::painter_fluid_drive::maybe_begin_fluid_stroke(
            sess, gpu, painter, epoch, dims, cw, ch, scale, coverage_k,
        );

        // Keep-wet (watercolor UX): re-upload the solver params when the pill flips
        // MID-FIELD — `fluid_diffusion_params()` zeroes evaporation while it's on, so
        // the live wash stops (or resumes) drying immediately, not at the next stroke.
        let keep_wet = painter.fluid_keep_wet();
        if sess.keep_wet != keep_wet {
            sess.keep_wet = keep_wet;
            sess.solver
                .set_from_diffusion(&gpu.queue, &painter.fluid_diffusion_params());
        }
        // Wet-paper sheen (view-only): drive the preview-texture flag from the tool's
        // Show Wet pill each frame. Only `cs_premul_tex`/`cs_straight_tex` consume it —
        // `out_buf` (and so the canvas bake) stays sheen-free, so the wash dries lighter.
        sess.compositor.set_wet_sheen(painter.fluid_show_wet());

        // ── Per-frame hot loop (resident, no O(grid) CPU work, no upload) ──
        // Drain this frame's dabs + the monotonic dab-bbox envelope; `None` ⇒ never wet.
        let capillary_active = painter.fluid_capillary_active();
        let (dabs, dab_region) = painter.fluid_take_dabs()?;
        // Capillary fringe (ADR-0078 S5 §2.2): the water wicks OUTWARD past the dab bboxes, so
        // the composite must follow it or it clips the soft fringe into a rectangle. Grow the
        // envelope = union(dab bboxes, all-time wet bbox) + a fringe pad. The wet-bbox union
        // (from the sporadic stats) tracks the real fringe extent (incl. extreme params); the
        // pad covers its read-to-read lag. A non-capillary brush keeps the bare dab envelope
        // (zero change to the validated look). The solver pads this by SOLVER_REGION_PAD on
        // top, so solver ⊇ composite still holds.
        // ── Active-region window (ADR-0085) ──────────────────────────────────────
        // The per-frame WORK region (sim + composite) follows the BRUSH: this frame's dab
        // bbox is pushed and the last ACTIVE_WINDOW_FRAMES are unioned + a wick pad. This is
        // the fix for "full-canvas wet drops FPS": the old region unioned the MONOTONIC
        // `wet_bbox`, which under Keep Wet grows to the whole canvas → ~1M settled cells
        // re-simulated every painting frame (the profiler's `region=(0,0)-(W,H)` with dabs=2).
        // Settled areas (outside the window) freeze; their last composite persists in the
        // preview texture.
        let win = ACTIVE_WINDOW_FRAMES;
        let pad = ACTIVE_REGION_PAD;
        if !dabs.is_empty() {
            // THIS frame's dab bbox (grid coords) — NOT the monotonic envelope `dab_region`.
            let (mut x0, mut y0, mut x1, mut y1) = (dims.0 - 1, dims.1 - 1, 0u32, 0u32);
            for d in &dabs {
                let lo_x = (d.cx - d.r).floor().clamp(0.0, (dims.0 - 1) as f32) as u32;
                let lo_y = (d.cy - d.r).floor().clamp(0.0, (dims.1 - 1) as f32) as u32;
                let hi_x = (d.cx + d.r).ceil().clamp(0.0, (dims.0 - 1) as f32) as u32;
                let hi_y = (d.cy + d.r).ceil().clamp(0.0, (dims.1 - 1) as f32) as u32;
                x0 = x0.min(lo_x);
                y0 = y0.min(lo_y);
                x1 = x1.max(hi_x);
                y1 = y1.max(hi_y);
            }
            sess.active_history
                .push_back((sess.frame, (x0, y0, x1, y1)));
        }
        let cur_frame = sess.frame;
        while sess
            .active_history
            .front()
            .is_some_and(|&(f, _)| cur_frame.saturating_sub(f) > win)
        {
            sess.active_history.pop_front();
        }
        let region = if capillary_active {
            match sess
                .active_history
                .iter()
                .map(|(_, b)| *b)
                .reduce(union_bbox)
            {
                Some(active) => grow_bbox(active, pad, dims),
                // No recent dabs: the wash is settling/drying. `dab_region` (the monotonic
                // envelope) is the safe composite cover for the still-visible pigment; under
                // Keep Wet the `settled` skip above means this region is never re-simulated
                // (C6 — the monotonic envelope no longer drives idle cost), and when drying it
                // is bounded by the stroke (the field drops once dry).
                None => dab_region,
            }
        } else {
            dab_region
        };
        // Map the tool's plain dabs → GPU dabs (the `r.max(0.5)` / radius>0 guard lives
        // in `DabGpu::new`, mirroring the CPU splat). `cs_splat` adds them to the
        // resident water + pigment; then diffuse/advect/evaporate run on the GPU.
        let gpu_dabs: Vec<DabGpu> = dabs
            .iter()
            .filter_map(|d| DabGpu::new(d.cx, d.cy, d.r, d.water, d.color, d.mass, d.staining))
            .collect();
        // Idle decimation + Keep-Wet settle-freeze: pointer up + no dabs ⇒ after a short
        // full-cadence warmup (bake catch-up gets immediate shots), step/composite run only
        // every IDLE_STEP_EVERY frames. **ADR-0085 C1:** the Keep-Wet field now reaches a real
        // equilibrium (the shallow-water FlowOutward force is surface-tension-PINNED, so the wash
        // settles instead of creeping), so the freeze is driven by PHYSICS, not a frame timeout:
        // once the active-region window has emptied (no dab landed for ACTIVE_WINDOW frames) and
        // the pointer is up, the pinned field is at rest ⇒ freeze it whole (its composite
        // persists in the preview texture; nothing left to recompute). Any dab / active stroke
        // refills `active_history` and unfreezes it. This subsumes the old KEEP_WET_SETTLE_FRAMES
        // timeout AND the monotonic-envelope idle cost (C6): a settled wash is never re-simulated.
        let stroke_active = painter.is_stroke_active();
        if stroke_active || !gpu_dabs.is_empty() {
            sess.idle_frames = 0;
        } else {
            sess.idle_frames = sess.idle_frames.saturating_add(1);
        }
        let settled = keep_wet && !stroke_active && sess.active_history.is_empty();
        let idle_skip = settled
            || (sess.idle_frames > IDLE_WARMUP_FRAMES && sess.idle_frames % IDLE_STEP_EVERY != 0);
        let t0 = profile.then(Instant::now);
        // Region-scoped (ADR-0078 S1): the sim runs only over the wet envelope (padded
        // inside the solver to ⊇ the composite region), so the per-frame cost is
        // O(wet frontier), not O(grid) — the dominant 4K cost (per the perf bench).
        // ── R1 single-submit hot path (ADR-0085 §2.3-I1/I2) ──────────────────
        // For the common case (pointer down, trivial layer stack — `trivial_hot`),
        // the sim step, the to-texture composite AND the preview-slot copy fold into
        // ONE encoder / ONE `queue.submit` (the merged block below), collapsing the
        // ~4 per-frame fluid submits that backpressured `acquire_frame` (the 50 ms
        // present-stall, perf §1) to 1, and the slot copy becomes a DIRTY-RECT (only
        // the wet envelope) instead of the whole canvas. The math is byte-identical.
        // The standalone `step_resident_splat` runs only when the merged path doesn't
        // (idle skip / E5 non-trivial stack / readback lane).
        let trivial_hot =
            stroke_active && painter.preview_is_trivial_stack() && override_entity.is_some();
        if !idle_skip && !trivial_hot {
            sess.solver
                .step_resident_splat(&gpu.device, &gpu.queue, &gpu_dabs, substeps, region);
        }
        let t1 = profile.then(Instant::now);
        // ── E4 step 2 (ADR-0078 S2): two preview modes ────────────────────────
        //
        // **MID-STROKE (pointer down)**: composite straight into the compositor's
        // premultiplied preview TEXTURE (`composite_frame_to_texture`, zero
        // readback) and GPU-copy it into our `IndividualTextureStore` slot; the
        // returned `PreviewOverride` samples the slot in place of the sprite THIS
        // frame. `canvas_rgba` is NOT touched mid-stroke (stale on purpose).
        //
        // **PEN-UP / DRYING (and any frame the stroke isn't active)**: the
        // EXISTING pipelined-readback path (`composite_frame_pipelined` +
        // `fluid_apply_gpu_composite_rows`) keeps `canvas_rgba` current by the
        // time anything reads it (next-stroke backdrop snapshot, undo, commit,
        // thumbnails). RATIONALE: baking only at explicit pen-up risks a stale
        // backdrop snapshot if a new stroke begins before the bake; falling back
        // to the readback path whenever the pointer is up makes canvas_rgba
        // eventually-current with zero new ordering hazards (the readback is
        // pipelined, ~0.14 ms — the drying cadence is low-stakes).
        let mut override_out: Option<PreviewOverride> = None;
        let mut texture_frame = false;
        // TRIVIAL-STACK GATE: the fluid texture contains ONLY the active layer composited
        // over its own backdrop. For a trivial stack that IS the whole preview; with multiple
        // layers (or opacity/blend/mask) the on-screen preview must be the FLATTENED stack,
        // which only the readback path (canvas_rgba → drain_preview re-composite) produces.
        // Multi-layer zero-readback = the E5 LayerCompositor chain (follow-up).
        if trivial_hot && let Some(entity_bits) = override_entity {
            // R1 single-submit hot path (extracted to `painter_fluid_drive` for HR-18): one
            // encoder / one submit for sim + to-texture composite + slot copy.
            let (o, tf) = super::painter_fluid_drive::encode_single_submit_frame(
                sess,
                gpu,
                renderer,
                region,
                &gpu_dabs,
                substeps,
                entity_bits,
                cw,
                ch,
            );
            override_out = o;
            texture_frame = tf;
        }
        // ADR-0085: the E5 mid-stroke straight-texture lane is removed. A non-trivial
        // GPU-representable stack now falls straight through to the readback lane (below) — the
        // same 1-frame-lag path it used on a non-representable stack — collapsing four preview
        // lanes to two (single-submit texture for trivial stacks + pipelined readback otherwise).
        if !texture_frame && idle_skip {
            // Idle-skipped frame (perf block 2b): the field didn't step, so there is
            // nothing new to composite or bake — keep showing the already-published
            // fluid preview texture (it persists in the slot across frames).
            if sess.texture_published {
                override_out =
                    super::painter_fluid_support::slot_override(sess.preview_slot, override_entity);
            }
        } else if !texture_frame {
            // Readback lane (pipelined composite → canvas_rgba bake + E4 catch-up +
            // transition hand-off + wet sheen) — extracted to `run_readback_lane`.
            if let Some(ov) = run_readback_lane(
                sess,
                renderer,
                gpu,
                painter,
                override_entity,
                region,
                stroke_active,
                cw,
                ch,
            ) {
                override_out = Some(ov);
            }
        }
        let t2 = profile.then(Instant::now);
        // Sporadic dry-check: the GPU reduces max-water (no CPU O(grid) scan); when
        // it's below the dry threshold AND the stroke has ended, the field drops.
        sess.frame = sess.frame.wrapping_add(1);
        let mut stats_us = 0u64;
        if sess.frame % DRY_CHECK_EVERY == 0 {
            let ts = profile.then(Instant::now);
            // Threshold = the visible-fringe contour (perf block 2a; was 1e-4 "to track the
            // THIN fringe film"). That film WAS the envelope runaway: the monotonic union grew
            // on accumulating numeric mist until it saturated the canvas (§4b of the perf-block
            // handoff). The solver now epsilon-clamps sub-WATER_EPS water every substep (the
            // hard brake) and the bbox tracks only water ≥ the real fringe contour; pigment
            // reaches only ~2 cells past it (§2.2 envelope-invariant test) — covered by
            // CAPILLARY_FRINGE_PAD = 8 ≫ that margin. `max_water` (the dry-check) is
            // threshold-independent (whole-field max), so the drop is unaffected.
            let stats = sess.solver.read_field_stats_pipelined(
                &gpu.device,
                &gpu.queue,
                ph2d_painter_brush::diffusion::WET_BBOX_WATER_THRESHOLD,
            );
            painter.fluid_dry_check_and_drop_gpu(stats.max_water);
            stats_us = ts.map_or(0, |t| t.elapsed().as_micros() as u64);
        }
        if profile {
            let step_us = t1.unwrap().duration_since(t0.unwrap()).as_micros() as u64;
            let comp_us = t2.unwrap().duration_since(t1.unwrap()).as_micros() as u64;
            PROFILE.with(|p| p.borrow_mut().record(step_us, comp_us, stats_us));
            // Context line for the `[gpu]` pass table: how much WORK was asked of the
            // GPU this frame (region cells × substeps), so a fixed-cost pass (cells
            // small, time large) is distinguishable from an O(area) one.
            if sess.frame % 120 == 1 {
                let cells = u64::from(region.2 - region.0 + 1) * u64::from(region.3 - region.1 + 1);
                eprintln!(
                    "[fluid-ctx] grid={}x{} scale={scale} region=({},{})-({},{}) ({cells} \
                     cells) substeps={substeps} dabs={} stroke_active={stroke_active} \
                     texture_frame={texture_frame}",
                    dims.0,
                    dims.1,
                    region.0,
                    region.1,
                    region.2,
                    region.3,
                    gpu_dabs.len(),
                );
            }
        }
        override_out
    })
}
