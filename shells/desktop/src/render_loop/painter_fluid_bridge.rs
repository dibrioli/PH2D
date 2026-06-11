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
    FluidSession, PROFILE, ensure_preview_slot, grow_bbox, run_readback_lane, union_bbox,
};
use super::painter_gpu_preview::{self, PainterGpuPreview};
use super::sim_extract::PreviewOverride;
use crate::app_state::PainterPreviewGpu;
use ph2d_editor::ToolRegistry;
use ph2d_editor::toast::ToastQueue;
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
/// the bloom to develop, short enough that a settled wash stops costing. `PH2D_FLUID_ACTIVE_WINDOW`
/// overrides for live tuning.
const ACTIVE_WINDOW_FRAMES: u64 = 90;
/// Pad (grid cells) grown around the active-dab window to cover the capillary wick advancing
/// outward from the recent dabs. `PH2D_FLUID_ACTIVE_PAD` overrides for live tuning.
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
        sess.catchup_bands = 0;
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
/// **E5 (ADR-0078 S2):** on a NON-trivial GPU-representable stack the mid-stroke
/// frame instead feeds the fluid's STRAIGHT composite texture into the
/// `painter_gpu_preview` driver (the single owner of the layer compositor + the
/// `painter_preview_gpu` slot) — that fills `painter_preview_gpu`, whose
/// override the render loop already emits, so this fn returns `None` on those
/// frames. `painter_gpu_preview_session` / `painter_preview_gpu` / `toasts` are
/// the SAME slots `painter_bridge::dispatch` hands to `try_drive` later in the
/// frame (sequential borrows).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drive_fluid_gpu(
    tools: &mut ToolRegistry,
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    override_entity: Option<u64>,
    painter_gpu_preview_session: &mut Option<PainterGpuPreview>,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
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
    // Watercolor v2 bisection (ADR-0085): `PH2D_FLUID_SUBSTEPS` overrides the sub-step
    // count — `0` runs splat+combine+composite but NONE of the diffuse/advect/transfer/
    // capillary passes, isolating the per-frame SIM cost from the COMPOSITE cost without
    // trusting the TBDR-inflated pass spans. (substeps=0 recovers FPS ⇒ the sim passes are
    // the cost; still drops ⇒ it's the composite/copy/base path.)
    let substeps = std::env::var("PH2D_FLUID_SUBSTEPS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| {
            // Watercolor v2 (ADR-0085): use the PAINTING sub-step count (1) while a stroke is
            // live — the GPU drive was wrongly running the idle count (2) for both, doubling
            // the ~40-pass chain every painting frame. Idle keeps 2 (it's decimated anyway).
            if painter.is_stroke_active() {
                painter.fluid_painting_substeps()
            } else {
                painter.fluid_idle_substeps()
            }
        });
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
        if sess.epoch != epoch {
            // Resident path: pigment + water + deposited start each stroke empty (water
            // is GPU-resident now — `cs_splat` adds it, `cs_evaporate` dries it).
            sess.solver
                .clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
            sess.solver
                .clear_resident_water_gpu(&gpu.device, &gpu.queue);
            sess.solver
                .clear_resident_deposited_gpu(&gpu.device, &gpu.queue);
            // ADR-0078 S3d: the shallow-water velocity + pressure start each stroke at rest
            // (no leftover momentum).
            sess.solver
                .clear_resident_velocity_gpu(&gpu.device, &gpu.queue);
            sess.frame = 0;
            // New stroke → forget the previous stroke's wet envelope (ADR-0078 S5).
            sess.wet_bbox = None;
            sess.active_history.clear();
            // E4: `begin_stroke` below recreates the compositor's preview texture
            // (seeded with the new premultiplied backdrop), so nothing published yet
            // this stroke. `texture_mode_dirty` is intentionally KEPT across epochs:
            // if a new stroke begins before the previous catch-up completed, the
            // union still names rows canvas_rgba never received — feeding the
            // superset region stays correct (just slightly larger).
            sess.texture_published = false;
            sess.catchup_bands = 0;
            // ADR-0079: drive ALL 15 solver controls (base diffusion + deposition +
            // shallow-water flow) from the ACTIVE BRUSH's per-brush `WatercolorParams`
            // (projected to `DiffusionParams`), replacing the old `FluidParams::default()`
            // + global `WATERCOLOR_*` consts. The artist's Brush Studio "Watercolor"
            // sliders now drive the live wash. `cs_combine` still feeds the compositor
            // `flowing + deposited` via `total_buffer()`; the velocity layer is dormant if
            // the brush sets `velocity = 0`.
            let dp = painter.fluid_diffusion_params();
            sess.solver.set_from_diffusion(&gpu.queue, &dp);
            // Keep-wet rides `dp` (evaporation forced to 0 in `fluid_diffusion_params`);
            // remember what was uploaded so a mid-field toggle re-uploads below.
            sess.keep_wet = painter.fluid_keep_wet();
            if let Some(paper) = painter.fluid_paper() {
                sess.solver.upload_paper(&gpu.queue, &paper);
            }
            // ADR-0084 backdrop lift: when the brush's `lift > 0`, seed the donor (`lift_source`)
            // from the current canvas backdrop vs the session's original PAPER (downsampled to the
            // grid by the SAME free fn the CPU seed uses → bit-identical CPU↔GPU, HR-5) and zero
            // `lifted_frac`; the wet brush then re-mobilizes that dry PAINT into the wash (+ the
            // compositor reveals the paper under the lifted pixels — never transparency). When no
            // paper snapshot exists, `paper == backdrop` ⇒ empty donor ⇒ inert (the safe fallback).
            // On the non-lift path zero BOTH so a fresh stroke with `lift = 0` has an inert donor +
            // `lifted_frac ≡ 0` → the compositor is byte-identical (the non-destructive default).
            if dp.lift > 0.0 {
                if let Some(backdrop) = painter.fluid_backdrop() {
                    let paper = painter.fluid_paper_base().unwrap_or(backdrop);
                    let cells = ph2d_painter_brush::diffusion::backdrop_to_lift_source(
                        backdrop, paper, cw, ch, dims.0, dims.1,
                    );
                    sess.solver.clear_lift_gpu(&gpu.device, &gpu.queue);
                    sess.solver.upload_lift_source(&gpu.queue, &cells);
                } else {
                    sess.solver.clear_lift_gpu(&gpu.device, &gpu.queue);
                }
            } else {
                sess.solver.clear_lift_gpu(&gpu.device, &gpu.queue);
            }
            if let Some(backdrop) = painter.fluid_backdrop() {
                // ADR-0080: pigment colour is per-pixel (reduced from the field), so no
                // per-stroke brush — `begin_stroke` just binds the field + backdrop.
                // Coverage supersampling: 2 at EVERY scale (matches the CPU reference's
                // `WET_COMPOSITE_SS = 2`). The old "1 at full-res (edge already 1px-fine)"
                // assumption is wrong for DARK pigments (ADR-0079 re-tune 2026-06-09): their
                // steep coverage curve compresses the visible edge below a cell, so at scale 1
                // the per-cell rim texture rendered with ZERO filtering = pixel teeth. ss=2
                // averages sub-cell coverage (measured: dark contour residual −33..42%) for 4×
                // the K-M ALU, which the region-scoped composite absorbs.
                // Coverage supersampling (N×N glaze samples/pixel). Watercolor v2 (ADR-0085):
                // at scale=1 the field IS canvas-res, so every sub-sample of a pixel reads the
                // SAME centre cell → the 4 samples are byte-identical and average to themselves.
                // ss=2 there is a 4× redundant K-M spectral evaluation per pixel — the dominant
                // full-canvas composite cost. ss=1 at scale=1 is byte-identical + 4× cheaper;
                // scale>1 (low-res grid) keeps ss=2 for real sub-cell coverage AA.
                let ss = if scale == 1 { 1 } else { 2 };
                sess.compositor.begin_stroke(
                    &gpu.device,
                    &gpu.queue,
                    dims.0,
                    dims.1,
                    cw,
                    ch,
                    scale,
                    coverage_k,
                    ss,
                    // Composite the TOTAL (flowing + deposited) so edge-darkening +
                    // granulation are visible (ADR-0078 S3c); equals flowing when
                    // nothing is deposited, so non-deposition strokes are unchanged.
                    sess.solver.total_buffer(),
                    backdrop,
                    // ADR-0084 paper-reveal: the session's original canvas content — lifted
                    // pixels lerp back toward it (never toward transparency). Falls back to the
                    // backdrop itself (`mix(b, b, lf) = b` ⇒ exact no-op) when no snapshot exists.
                    painter.fluid_paper_base().unwrap_or(backdrop),
                    // ADR-0084: bind the lift accumulator so the compositor reveals the paper
                    // where dry paint was lifted. All-zero (cleared above) when `lift = 0`
                    // ⇒ byte-identical output.
                    Some(sess.solver.lifted_frac_buffer()),
                    // Wet-paper sheen: bind the resident water so the preview-texture
                    // passes can darken wet regions + brighten the meniscus (view-only;
                    // the flag is driven per frame via `set_wet_sheen` below).
                    Some(sess.solver.water_buffer()),
                );
            }
            sess.epoch = epoch;
        }

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
        // preview texture. Env-tunable for the bloom-extent ↔ perf trade-off.
        let win = std::env::var("PH2D_FLUID_ACTIVE_WINDOW")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(ACTIVE_WINDOW_FRAMES);
        let pad = std::env::var("PH2D_FLUID_ACTIVE_PAD")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(ACTIVE_REGION_PAD);
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
            sess.active_history.push_back((sess.frame, (x0, y0, x1, y1)));
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
            match sess.active_history.iter().map(|(_, b)| *b).reduce(union_bbox) {
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
            || (sess.idle_frames > IDLE_WARMUP_FRAMES
                && sess.idle_frames % IDLE_STEP_EVERY != 0);
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
        if trivial_hot
            && let Some(entity_bits) = override_entity
        {
            let enc = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("fluid frame (R1 single submit)"),
                });
            // (1) Sim step (splat + substeps + combine) — encoded, not submitted.
            let mut enc = sess.solver.encode_resident_splat_step(
                &gpu.device,
                &gpu.queue,
                enc,
                &gpu_dabs,
                substeps,
                region,
            );
            // (2) To-texture composite over the wet region (SAME encoder). `None` ⇒
            // empty region: still submit the stepped field, fall through to readback.
            if let Some((px_lo, py_lo, px_hi, py_hi)) = sess
                .compositor
                .encode_frame_to_texture(&gpu.queue, &mut enc, region)
            {
                // (3) Acquire/resize the slot (no GPU work); a FRESH or never-seeded
                // slot gets the full backdrop seeded ONCE, then per-frame dirty-rect
                // refreshes of only the wet rect (ADR-0085 §2.3-I2). All in `enc`.
                let (id, fresh) = ensure_preview_slot(renderer, &mut sess.preview_slot, cw, ch);
                let seed = fresh || !sess.preview_slot_seeded;
                let copy_ok = if let Some(tex) = sess.compositor.preview_texture() {
                    if seed {
                        renderer
                            .encode_copy_into_individual(&mut enc, id, tex, cw, ch)
                            .is_ok()
                    } else {
                        renderer
                            .encode_copy_region_into_individual(
                                &mut enc,
                                id,
                                tex,
                                px_lo,
                                py_lo,
                                px_lo,
                                py_lo,
                                px_hi - px_lo,
                                py_hi - py_lo,
                            )
                            .is_ok()
                    }
                } else {
                    false
                };
                gpu.queue.submit([enc.finish()]); // ← the SINGLE submit
                if copy_ok {
                    sess.preview_slot_seeded = true;
                    // Catch-up accumulator: union every grid region the texture path
                    // composited (canvas_rgba is stale over it until the readback
                    // frames after pen-up replay the union — see the readback arm).
                    sess.texture_mode_dirty = Some(match sess.texture_mode_dirty {
                        Some(d) => union_bbox(d, region),
                        None => region,
                    });
                    sess.catchup_bands = 0;
                    sess.texture_published = true;
                    texture_frame = true;
                    override_out = Some(PreviewOverride {
                        entity_bits,
                        texture_id: id,
                        premultiplied: true,
                    });
                } else {
                    // Copy failed (or no preview texture yet) → release the just-acquired slot
                    // and drop the seed, mirroring `copy_preview_into_slot`'s failure path (C5:
                    // else the unused canvas-res slot is held until teardown while the readback
                    // lane runs). The readback path (canvas_rgba + the CPU preview) keeps the
                    // stroke alive at a 1-frame lag; a recovered frame re-acquires + re-seeds.
                    if let Some((old, _, _)) = sess.preview_slot.take() {
                        renderer.individual_mut().release(old);
                    }
                    sess.preview_slot_seeded = false;
                }
            } else {
                gpu.queue.submit([enc.finish()]);
            }
        }
        // ── E5 (ADR-0078 S2): NON-trivial GPU-representable stack mid-stroke ──
        // The fluid straight composite goes GPU→GPU into the LayerCompositor's
        // cached slice for the ACTIVE layer and the stack recomposites into the
        // `painter_preview_gpu` slot (whose override the render loop maps
        // itself — `override_out` stays None). Mutually exclusive with the E4
        // arm (`gpu_eligible` is None on trivial stacks); a non-representable
        // stack falls through to today's readback lane (guard #5). Logic lives
        // in `painter_gpu_preview` (the single owner of compositor + slot).
        if !texture_frame
            && stroke_active
            && let Some(entity_bits) = override_entity
            && painter_gpu_preview::drive_fluid_chain(
                painter_gpu_preview_session,
                renderer,
                painter,
                entity_bits,
                &mut sess.compositor,
                gpu,
                region,
                cw,
                ch,
                epoch,
                painter_preview_gpu,
                toasts,
            )
        {
            // canvas_rgba stays stale over the composited region (same E4
            // catch-up: pointer-up readback frames replay the union into it).
            sess.texture_mode_dirty = Some(match sess.texture_mode_dirty {
                Some(d) => union_bbox(d, region),
                None => region,
            });
            sess.catchup_bands = 0;
            texture_frame = true;
        }
        if !texture_frame && idle_skip {
            // Idle-skipped frame (perf block 2b): the field didn't step, so there is
            // nothing new to composite or bake — keep showing the already-published
            // fluid preview texture (it persists in the slot across frames).
            if sess.texture_published
                && let (Some((id, _, _)), Some(entity_bits)) = (sess.preview_slot, override_entity)
            {
                override_out = Some(PreviewOverride {
                    entity_bits,
                    texture_id: id,
                    premultiplied: true,
                });
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
                epoch,
                painter_gpu_preview_session,
                painter_preview_gpu,
                toasts,
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
            // Grow the all-time wet envelope (ADR-0078 S5): the capillary fringe pushes the
            // wet bbox out; union it (never shrink — drying recedes it) so the composite keeps
            // covering the fringe. Only consumed while the brush's capillary layer is on.
            if let Some(b) = stats.bbox {
                sess.wet_bbox = Some(sess.wet_bbox.map_or(b, |prev| union_bbox(prev, b)));
            }
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
