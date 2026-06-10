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

use super::painter_fluid_support::{PROFILE, copy_preview_into_slot, grow_bbox, union_bbox};
use super::painter_gpu_preview::{self, PainterGpuPreview};
use super::sim_extract::PreviewOverride;
use crate::app_state::PainterPreviewGpu;
use ph2d_editor::ToolRegistry;
use ph2d_editor::toast::ToastQueue;
use ph2d_gpu::GpuContext;
use ph2d_painter_fluid::{DabGpu, FluidCompositor, FluidSolver};
use ph2d_render::SpriteRenderer;
use std::cell::RefCell;
use std::time::Instant;

/// How often (in frames) the resident path reads the GPU field stats back for the
/// dry-check. Drying takes ~0.3 s (≈ 18 frames @ 60 Hz), so a few frames of latency
/// on the drop is invisible — and the readback is the only per-frame sync we still
/// want sporadic (4K real-time arch §4 / E3). The composite envelope does NOT use
/// it (it's grown from the dab list), so cadence only affects when a dry field drops.
/// At 20: drying (~18 frames after pen-up) still drops the field promptly, but the
/// per-frame avg of the stats `device.poll(wait)` (a queue-drain, ~2.5 ms when it
/// fires) is ~3× lower than at 6 — it was the largest per-frame phase post-pipeline.
const DRY_CHECK_EVERY: u64 = 20;

/// Grid-cell pad added around the composite envelope for the capillary fringe (ADR-0078 S5).
/// The water wicks OUTWARD past the dab bboxes, so the dab-union envelope alone would clip the
/// fringe into a rectangle (the §2.2 bug). Growing the envelope by this pad (only when the
/// brush's capillary layer is on) keeps the soft fringe inside both the composite read region
/// and the solver dispatch (which adds its own `SOLVER_REGION_PAD` on top, preserving the
/// solver ⊇ composite invariant). It covers the fringe growth between the sporadic wet-bbox
/// reads; the monotonic `wet_bbox` union then tracks larger (extreme-param) fringes exactly.
const CAPILLARY_FRINGE_PAD: u32 = 8;

/// Per-session GPU state for the live wet field: the resident solver, the K–M
/// compositor, the field size, the stroke epoch it was last reset for, a frame
/// counter pacing the sporadic dry-check readback, and the monotonic wet-bbox the
/// capillary envelope grows from (ADR-0078 S5).
struct FluidSession {
    solver: FluidSolver,
    compositor: FluidCompositor,
    dims: (u32, u32),
    epoch: u64,
    frame: u64,
    /// **All-time wet bbox of this stroke** (union of the sporadic `read_field_stats` bboxes,
    /// reset on a new epoch). The capillary fringe wicks the wet region OUTWARD past the dab
    /// bboxes; the water bbox also recedes as the wash dries, so a SUPERSET-correct envelope
    /// must take the monotonic union (the §3.4 / §2.2 lesson). `None` until the first read.
    wet_bbox: Option<(u32, u32, u32, u32)>,
    /// **E4 (ADR-0078 S2): the `IndividualTextureStore` slot** the mid-stroke texture path
    /// GPU-copies the compositor's premultiplied preview into — `(texture_id, w, h)` so a
    /// canvas-size change releases + re-acquires. `None` until the first texture-mode frame
    /// (lazy, mirroring `painter_gpu_preview::ensure_slot`). MUST be released via
    /// [`Self::release_preview_slot`] before the session is dropped or rebuilt.
    preview_slot: Option<(u32, u32, u32)>,
    /// **E4 catch-up accumulator**: union of every GRID region the zero-readback texture
    /// path composited (canvas_rgba is intentionally stale over it mid-stroke). The first
    /// readback frames after the texture→readback switch feed this union into
    /// `composite_frame_pipelined` so `canvas_rgba` catches up on EVERYTHING the texture
    /// path skipped; cleared only once the (1-frame-late) pipelined path has returned 2
    /// consecutive bands while the union was being fed (the 2nd is necessarily from a
    /// submission that included it).
    texture_mode_dirty: Option<(u32, u32, u32, u32)>,
    /// Consecutive non-empty pipelined bands applied while `texture_mode_dirty` was being
    /// fed (reset by an empty band and by every texture-mode frame). At 2 the union is
    /// provably applied → cleared.
    catchup_bands: u8,
    /// `true` while the fluid preview SLOT holds the freshest composite (texture mode ran,
    /// and the readback path hasn't yet handed the preview back to the CPU slot). Keeps the
    /// PreviewOverride on the fluid slot across the texture→readback transition so the
    /// stroke never flickers back to the pre-stroke CPU preview.
    texture_published: bool,
}

impl FluidSession {
    fn new(device: &wgpu::Device, dims: (u32, u32)) -> Self {
        Self {
            solver: FluidSolver::new(device, dims.0, dims.1),
            compositor: FluidCompositor::new(device),
            dims,
            epoch: u64::MAX,
            frame: 0,
            wet_bbox: None,
            preview_slot: None,
            texture_mode_dirty: None,
            catchup_bands: 0,
            texture_published: false,
        }
    }

    /// Release the E4 preview slot (if any) back to the `IndividualTextureStore`.
    /// MUST run before the session is dropped or rebuilt (grid-size change) —
    /// the slot id is refcounted and would otherwise leak its canvas-res texture.
    fn release_preview_slot(&mut self, renderer: &mut SpriteRenderer) {
        if let Some((id, _, _)) = self.preview_slot.take() {
            renderer.individual_mut().release(id);
        }
        self.texture_published = false;
    }
}

thread_local! {
    /// Rebuilt when the field resizes (a new canvas); reset (resident pigment zeroed
    /// + paper re-uploaded) on a new stroke epoch; dropped when no live field.
    /// Thread-local because the render loop is single-threaded (the established
    /// painter-bridge pattern for per-tool GPU state).
    static SESSION: RefCell<Option<FluidSession>> = const { RefCell::new(None) };
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
    let substeps = painter.fluid_idle_substeps();
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
                let ss = 2;
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
                );
            }
            sess.epoch = epoch;
        }

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
        let region = if capillary_active {
            let env = match sess.wet_bbox {
                Some(wb) => union_bbox(dab_region, wb),
                None => dab_region,
            };
            grow_bbox(env, CAPILLARY_FRINGE_PAD, dims)
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
        let t0 = profile.then(Instant::now);
        // Region-scoped (ADR-0078 S1): the sim runs only over the wet envelope (padded
        // inside the solver to ⊇ the composite region), so the per-frame cost is
        // O(wet frontier), not O(grid) — the dominant 4K cost (per the perf bench).
        sess.solver
            .step_resident_splat(&gpu.device, &gpu.queue, &gpu_dabs, substeps, region);
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
        let stroke_active = painter.is_stroke_active();
        let mut override_out: Option<PreviewOverride> = None;
        let mut texture_frame = false;
        // TRIVIAL-STACK GATE: the fluid texture contains ONLY the active layer composited
        // over its own backdrop. For a trivial stack that IS the whole preview; with multiple
        // layers (or opacity/blend/mask) the on-screen preview must be the FLATTENED stack,
        // which only the readback path (canvas_rgba → drain_preview re-composite) produces.
        // Multi-layer zero-readback = the E5 LayerCompositor chain (follow-up).
        if stroke_active
            && painter.preview_is_trivial_stack()
            && let Some(entity_bits) = override_entity
            && sess
                .compositor
                .composite_frame_to_texture(&gpu.device, &gpu.queue, region)
                .is_some()
        {
            // Catch-up accumulator: union every grid region the texture path
            // composited (canvas_rgba is stale over it until the readback
            // frames after pen-up replay the union — see the readback arm).
            sess.texture_mode_dirty = Some(match sess.texture_mode_dirty {
                Some(d) => union_bbox(d, region),
                None => region,
            });
            sess.catchup_bands = 0;
            // Re-fetched every frame: `begin_stroke` recreates the compositor's
            // preview texture on a canvas resize.
            if let Some(tex) = sess.compositor.preview_texture()
                && let Some(id) =
                    copy_preview_into_slot(renderer, &mut sess.preview_slot, tex, cw, ch)
            {
                sess.texture_published = true;
                texture_frame = true;
                override_out = Some(PreviewOverride {
                    entity_bits,
                    texture_id: id,
                    premultiplied: true,
                });
            }
            // On a copy failure we fall through to the readback path below
            // (canvas_rgba + the CPU preview keep the stroke alive, 1-frame lag).
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
        if !texture_frame {
            // Composite PIPELINED (async readback, no per-frame device.poll(wait)
            // stall) → ~240 FPS. The pipeline is 1-frame-late (returns the PRIOR
            // frame's band — imperceptible, the same lag the preview always had).
            // begin_stroke drains the prior stroke's in-flight map before reuse.
            //
            // E4 catch-up: the FIRST readback frames after the texture→readback
            // switch must composite the FULL region the texture path covered, so
            // canvas_rgba catches up on everything it skipped. Because the band
            // returned NOW is from the PRIOR submission, the union keeps being fed
            // until 2 CONSECUTIVE bands returned while feeding it — the 2nd is
            // necessarily from a submission that included the union.
            let rb_region = match sess.texture_mode_dirty {
                Some(d) => union_bbox(region, d),
                None => region,
            };
            let (band, rect) =
                sess.compositor
                    .composite_frame_pipelined(&gpu.device, &gpu.queue, rb_region);
            if !band.is_empty() {
                painter.fluid_apply_gpu_composite_rows(&band, rect);
                if sess.texture_mode_dirty.is_some() {
                    sess.catchup_bands += 1;
                    if sess.catchup_bands >= 2 {
                        sess.texture_mode_dirty = None;
                        sess.catchup_bands = 0;
                    }
                }
                // Transition hand-off (texture → readback): canvas_rgba just
                // caught up and `fluid_apply_gpu_composite_rows` marked the
                // preview dirty — but the CPU upload happens LATER this frame and
                // sim_extract samples the CPU slot with a 1-frame lag, so keep the
                // (one-frame-stale) fluid texture override for THIS frame only,
                // then hand the preview back to the CPU slot for good.
                if sess.texture_published {
                    sess.texture_published = false;
                    if let (Some((id, _, _)), Some(entity_bits)) =
                        (sess.preview_slot, override_entity)
                    {
                        override_out = Some(PreviewOverride {
                            entity_bits,
                            texture_id: id,
                            premultiplied: true,
                        });
                    }
                }
            } else {
                // "2 consecutive" discipline: an empty band resets the count.
                sess.catchup_bands = 0;
                // No band yet (first readback frame after the switch — nothing
                // was pending): the CPU preview is still pre-catch-up, so keep
                // showing the fluid texture to avoid a stroke-disappears flicker.
                if sess.texture_published
                    && let (Some((id, _, _)), Some(entity_bits)) =
                        (sess.preview_slot, override_entity)
                {
                    override_out = Some(PreviewOverride {
                        entity_bits,
                        texture_id: id,
                        premultiplied: true,
                    });
                }
            }
        }
        let t2 = profile.then(Instant::now);
        // Sporadic dry-check: the GPU reduces max-water (no CPU O(grid) scan); when
        // it's below the dry threshold AND the stroke has ended, the field drops.
        sess.frame = sess.frame.wrapping_add(1);
        let mut stats_us = 0u64;
        if sess.frame % DRY_CHECK_EVERY == 0 {
            let ts = profile.then(Instant::now);
            // Threshold 1e-4 (not 1e-3) so the wet bbox tracks the THIN capillary fringe film
            // where the wick carries pigment — keeps the grown envelope covering it. `max_water`
            // (the dry-check) is threshold-independent (whole-field max), so the drop is unaffected.
            let stats = sess
                .solver
                .read_field_stats(&gpu.device, &gpu.queue, 1.0e-4);
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
        }
        override_out
    })
}
