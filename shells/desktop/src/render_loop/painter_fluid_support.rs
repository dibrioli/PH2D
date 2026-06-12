//! Support pieces for the W15.3 GPU fluid drive (`painter_fluid_bridge`) — the
//! opt-in per-phase profiler, bbox helpers, and the preview-slot plumbing. Split
//! out of the bridge for HR-18 (≤600 LOC per shell file); the bridge keeps the
//! per-frame drive logic, this module keeps the leaf utilities.

use super::sim_extract::PreviewOverride;
use ph2d_gpu::GpuContext;
use ph2d_painter_fluid::{FluidCompositor, FluidSolver};
use ph2d_render::SpriteRenderer;
use std::cell::RefCell;

/// Opt-in per-phase profiler for the fluid drive (`PH2D_FLUID_PROFILE=1`). Confirms
/// where the per-frame wall-clock goes — sim step vs the composite (whose `device.poll`
/// readback is the suspected sync stall) vs the sporadic stats readback — before any
/// structural change. Prints averaged ms to stderr every `WINDOW` active frames.
pub(super) struct FluidProfile {
    on: Option<bool>,
    frames: u32,
    step_us: u64,
    comp_us: u64,
    stats_us: u64,
}

impl FluidProfile {
    const WINDOW: u32 = 120;
    const fn new() -> Self {
        Self {
            on: None,
            frames: 0,
            step_us: 0,
            comp_us: 0,
            stats_us: 0,
        }
    }
    pub(super) fn enabled(&mut self) -> bool {
        if self.on.is_none() {
            self.on = Some(std::env::var("PH2D_FLUID_PROFILE").is_ok_and(|v| v != "0"));
        }
        self.on == Some(true)
    }
    pub(super) fn record(&mut self, step_us: u64, comp_us: u64, stats_us: u64) {
        self.frames += 1;
        self.step_us += step_us;
        self.comp_us += comp_us;
        self.stats_us += stats_us;
        if self.frames >= Self::WINDOW {
            let f = f64::from(self.frames);
            let (s, c, st) = (
                self.step_us as f64 / f / 1000.0,
                self.comp_us as f64 / f / 1000.0,
                self.stats_us as f64 / f / 1000.0,
            );
            eprintln!(
                "[fluid] per-frame avg over {} frames: step={s:.3}ms composite(+readback)={c:.3}ms stats={st:.3}ms total={:.3}ms",
                self.frames,
                s + c + st
            );
            self.frames = 0;
            self.step_us = 0;
            self.comp_us = 0;
            self.stats_us = 0;
        }
    }
}

thread_local! {
    pub(super) static PROFILE: RefCell<FluidProfile> = const { RefCell::new(FluidProfile::new()) };
}

/// Inclusive union of two grid-cell bboxes `(x0, y0, x1, y1)`.
pub(super) fn union_bbox(a: (u32, u32, u32, u32), b: (u32, u32, u32, u32)) -> (u32, u32, u32, u32) {
    (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
}

/// The fluid preview-slot override for the render loop — `Some` iff a slot id AND an override
/// entity both exist. The texture→readback hand-off (`run_readback_lane`) and the idle-skip
/// republish (the bridge) both emit exactly this; one definition keeps them from drifting.
pub(super) fn slot_override(
    preview_slot: Option<(u32, u32, u32)>,
    override_entity: Option<u64>,
) -> Option<PreviewOverride> {
    match (preview_slot, override_entity) {
        (Some((id, _, _)), Some(entity_bits)) => Some(PreviewOverride {
            entity_bits,
            texture_id: id,
            premultiplied: true,
        }),
        _ => None,
    }
}

/// Grow an inclusive grid-cell bbox by `pad` cells on each side, clamped to `dims`.
pub(super) fn grow_bbox(
    b: (u32, u32, u32, u32),
    pad: u32,
    dims: (u32, u32),
) -> (u32, u32, u32, u32) {
    (
        b.0.saturating_sub(pad),
        b.1.saturating_sub(pad),
        (b.2 + pad).min(dims.0.saturating_sub(1)),
        (b.3 + pad).min(dims.1.saturating_sub(1)),
    )
}

/// **R1 (ADR-0085 §2.3-I1/I2): lazily acquire/resize the preview slot WITHOUT any
/// GPU work**, so the single-submit hot path can encode the seed/refresh copy into
/// its own shared encoder. Returns `(id, fresh)` — `fresh == true` when the slot
/// was (re)created this call, meaning the caller must FULL-copy the backdrop once
/// before switching to per-frame dirty-rect refreshes. Mirrors the acquire half of
/// [`copy_preview_into_slot`] (which still owns the readback-lane / sheen path).
pub(super) fn ensure_preview_slot(
    renderer: &mut SpriteRenderer,
    slot: &mut Option<(u32, u32, u32)>,
    cw: u32,
    ch: u32,
) -> (u32, bool) {
    match *slot {
        Some((id, w, h)) if w == cw && h == ch => (id, false),
        _ => {
            if let Some((old, _, _)) = slot.take() {
                renderer.individual_mut().release(old);
            }
            let id = renderer.acquire_individual_empty(cw, ch);
            *slot = Some((id, cw, ch));
            (id, true)
        }
    }
}

/// E4 (ADR-0078 S2): lazily acquire/resize the `IndividualTextureStore` slot and
/// GPU-copy the fluid compositor's premultiplied preview texture into it (the
/// rgba8unorm → Rgba8UnormSrgb copy is format-compatible; the renderer samples
/// it this same frame, before it is ever sampled — the `acquire_empty` contract).
/// Returns the slot id, or `None` on a copy error (slot released; the caller
/// falls back to the readback path, which keeps the preview alive).
pub(super) fn copy_preview_into_slot(
    renderer: &mut SpriteRenderer,
    slot: &mut Option<(u32, u32, u32)>,
    tex: &wgpu::Texture,
    cw: u32,
    ch: u32,
) -> Option<u32> {
    let id = match *slot {
        Some((id, w, h)) if w == cw && h == ch => id,
        _ => {
            if let Some((old, _, _)) = slot.take() {
                renderer.individual_mut().release(old);
            }
            let id = renderer.acquire_individual_empty(cw, ch);
            *slot = Some((id, cw, ch));
            id
        }
    };
    match renderer.copy_texture_into_individual(id, tex, cw, ch) {
        Ok(()) => Some(id),
        Err(e) => {
            eprintln!("warn: fluid preview texture->slot copy failed ({e}); using readback path");
            renderer.individual_mut().release(id);
            *slot = None;
            None
        }
    }
}

/// **Readback lane** — any frame the zero-readback texture path (E4) didn't run: composite
/// PIPELINED (async readback, no per-frame `device.poll(wait)` stall; the band returned is the
/// PRIOR frame's — imperceptible, the same lag the preview always had; `begin_stroke` drains the
/// in-flight map before reuse) and keep `canvas_rgba` current.
///
/// **Transition catch-up (ADR-0085, single sync bake):** the texture path leaves `canvas_rgba`
/// stale over everything it composited (`texture_mode_dirty`). The FIRST readback frame after the
/// texture path stops bakes that whole union ONCE, SYNCHRONOUSLY, then clears it — subsequent
/// drying frames composite only their own `region`. This replaces the old per-frame union
/// recomposite + the racy "2 consecutive bands" auto-clear (which never reliably engaged under
/// GPU backpressure, so the union grew unbounded and was re-composited every frame). The
/// `flush_pending_bake` at the next pointer-down stays as the cross-stroke backstop.
///
/// Transition hand-off: the same frame, the (one-frame-stale, but now-current-bytes) fluid
/// texture override is re-emitted so the stroke doesn't flicker to the CPU preview while
/// `sim_extract` catches up its 1-frame CPU-slot lag. Finally, with the pointer up and Show Wet
/// on, the view-only wet sheen is recomposited + republished ([`publish_wet_sheen_between_strokes`]).
#[allow(clippy::too_many_arguments)]
pub(super) fn run_readback_lane(
    sess: &mut FluidSession,
    renderer: &mut SpriteRenderer,
    gpu: &GpuContext,
    painter: &mut ph2d_tool_painter::PainterTool,
    override_entity: Option<u64>,
    region: (u32, u32, u32, u32),
    stroke_active: bool,
    cw: u32,
    ch: u32,
) -> Option<PreviewOverride> {
    let mut override_out = None;
    // Transition catch-up: bake the texture path's whole skipped union into `canvas_rgba` ONCE,
    // synchronously, on the first readback frame after the texture path stopped. (One-time
    // ~0.14 ms poll(wait) at the transition, like `flush_pending_bake`; not per frame.)
    if let Some(dirty) = sess.texture_mode_dirty.take() {
        let (band, rect) = sess
            .compositor
            .composite_frame(&gpu.device, &gpu.queue, dirty);
        if !band.is_empty() {
            painter.fluid_apply_gpu_composite_rows(&band, rect);
        }
    }
    // The normal drying readback: pipelined composite of THIS frame's region (1-frame-late).
    let (band, rect) = sess
        .compositor
        .composite_frame_pipelined(&gpu.device, &gpu.queue, region);
    if !band.is_empty() {
        painter.fluid_apply_gpu_composite_rows(&band, rect);
    }
    // Texture→readback hand-off: re-emit the slot override for THIS frame (sim_extract samples
    // the CPU slot with a 1-frame lag) so the stroke doesn't flicker to the pre-bake CPU preview.
    // The sync transition bake above already made `canvas_rgba` current, so drop the flag now.
    if sess.texture_published {
        sess.texture_published = false;
        override_out = slot_override(sess.preview_slot, override_entity);
    }
    // Wet-sheen between strokes: `canvas_rgba` is kept current (sheen-free, the bake)
    // by the readback above; the sheen is view-only, so ALSO composite into the preview
    // texture + publish the fluid override this frame. One extra region-scoped composite
    // per drying frame; no `texture_mode_dirty` feeding (the readback lane runs in
    // parallel).
    if !stroke_active
        && painter.fluid_show_wet()
        && let Some(entity_bits) = override_entity
        && let Some(ov) = publish_wet_sheen_between_strokes(
            renderer,
            gpu,
            painter,
            entity_bits,
            &mut sess.compositor,
            &mut sess.preview_slot,
            &mut sess.texture_published,
            region,
            cw,
            ch,
        )
    {
        override_out = Some(ov);
    }
    override_out
}

/// **Wet-sheen between strokes (drying / keep-wet).** While the field stays wet with
/// the pointer UP, the readback lane keeps `canvas_rgba` current exactly as before —
/// but `canvas_rgba` is sheen-free by design (the sheen is view-only), so the wet look
/// would vanish at pen-up. This runs IN ADDITION to the readback bake each drying
/// frame: one extra region-scoped composite into the preview texture (the sheen flag
/// rides the `cs_premul_tex`/`cs_straight_tex` passes) and a publish of the fluid
/// override, so the wash visibly stays wet until it dries (or indefinitely under
/// keep-wet) and then "dries lighter". Trivial stacks publish the premultiplied preview slot
/// directly; ADR-0085: a non-trivial GPU-representable stack no longer gets the cosmetic sheen
/// (the E5 lane is gone) — the readback lane keeps `canvas_rgba` current and this returns `None`.
#[allow(clippy::too_many_arguments)]
pub(super) fn publish_wet_sheen_between_strokes(
    renderer: &mut SpriteRenderer,
    gpu: &GpuContext,
    painter: &mut ph2d_tool_painter::PainterTool,
    entity_bits: u64,
    compositor: &mut FluidCompositor,
    preview_slot: &mut Option<(u32, u32, u32)>,
    texture_published: &mut bool,
    region: (u32, u32, u32, u32),
    cw: u32,
    ch: u32,
) -> Option<PreviewOverride> {
    if painter.preview_is_trivial_stack() {
        compositor.composite_frame_to_texture(&gpu.device, &gpu.queue, region)?;
        let tex = compositor.preview_texture()?;
        let id = copy_preview_into_slot(renderer, preview_slot, tex, cw, ch)?;
        *texture_published = true;
        Some(PreviewOverride {
            entity_bits,
            texture_id: id,
            premultiplied: true,
        })
    } else {
        // ADR-0085 (E5 lane removed): a non-trivial GPU-representable stack no longer gets the
        // view-only wet-sheen between strokes — the readback lane keeps `canvas_rgba` (the actual
        // paint) current via the standard provider path; only the cosmetic sheen is dropped here.
        None
    }
}

/// Per-session GPU state for the live wet field: the resident solver, the K–M
/// compositor, the field size, the stroke epoch it was last reset for, a frame
/// counter pacing the sporadic dry-check readback, and the monotonic wet-bbox the
/// capillary envelope grows from (ADR-0078 S5).
pub(super) struct FluidSession {
    pub(super) solver: FluidSolver,
    pub(super) compositor: FluidCompositor,
    pub(super) dims: (u32, u32),
    pub(super) epoch: u64,
    pub(super) frame: u64,
    /// **E4 (ADR-0078 S2): the `IndividualTextureStore` slot** the mid-stroke texture path
    /// GPU-copies the compositor's premultiplied preview into — `(texture_id, w, h)` so a
    /// canvas-size change releases + re-acquires. `None` until the first texture-mode frame
    /// (lazy, mirroring `painter_gpu_preview::ensure_slot`). MUST be released via
    /// [`Self::release_preview_slot`] before the session is dropped or rebuilt.
    pub(super) preview_slot: Option<(u32, u32, u32)>,
    /// **R1 dirty-rect seed flag** (ADR-0085 §2.3-I2): `true` once the preview slot
    /// holds the FULL premultiplied backdrop, so the single-submit hot path can
    /// refresh only the wet dirty-rect per frame (no full-canvas copy) instead of
    /// re-copying the whole canvas. Reset whenever the slot is (re)acquired or
    /// released; the next hot-path frame re-seeds with a one-time full copy.
    pub(super) preview_slot_seeded: bool,
    /// **E4 catch-up accumulator**: union of every GRID region the zero-readback texture
    /// path composited (canvas_rgba is intentionally stale over it mid-stroke). The first
    /// readback frames after the texture→readback switch feed this union into
    /// path skipped; baked into `canvas_rgba` ONCE synchronously on the first readback frame
    /// after the texture path stops (`run_readback_lane`), then cleared. Also drained by
    /// `flush_pending_bake` at the next pointer-down (the cross-stroke undo bracket).
    pub(super) texture_mode_dirty: Option<(u32, u32, u32, u32)>,
    /// `true` while the fluid preview SLOT holds the freshest composite (texture mode ran,
    /// and the readback path hasn't yet handed the preview back to the CPU slot). Keeps the
    /// PreviewOverride on the fluid slot across the texture→readback transition so the
    /// stroke never flickers back to the pre-stroke CPU preview.
    pub(super) texture_published: bool,
    /// Last keep-wet value uploaded to the solver. The solver params normally upload
    /// once per epoch (`set_from_diffusion` at stroke begin), but the Keep Wet pill can
    /// flip MID-FIELD (between dabs / while drying) — on change the bridge re-uploads
    /// `fluid_diffusion_params()` (the chokepoint that zeroes evaporation) so the
    /// toggle takes effect immediately on the live wash.
    pub(super) keep_wet: bool,
    /// **Current wet bbox (ADR-0085 — the rectangular-artifact fix).** The latest GPU
    /// `read_field_stats` water bbox (grid cells, ≥ the visible-fringe threshold), **non-
    /// monotonic** — it tracks where the wash ACTUALLY is and SHRINKS as it dries (unlike the old
    /// monotonic all-time union that ran away to the whole canvas). Unioned into the per-frame
    /// work region so the sim + composite follow the CAPILLARY WICK, not just the recent dabs —
    /// else the wick bleeds past the dab window and the region's straight edge clips it into a
    /// rectangle ("borda reta"). `None` until the first stats read of a stroke.
    pub(super) wet_bbox: Option<(u32, u32, u32, u32)>,
    /// Consecutive frames with the pointer up AND no dabs (perf block 2b). Paces the
    /// idle decimation: with a live field but nothing being painted, the sim step +
    /// composite/sheen run only every `IDLE_STEP_EVERY` frames (the field barely moves
    /// idle; recompositing the whole envelope per frame was the dominant idle GPU cost,
    /// §4b). Reset to 0 by any dab or active stroke.
    pub(super) idle_frames: u64,
    /// **Watercolor v2 (ADR-0085) — active-region window.** The per-frame work region (sim +
    /// composite) follows the brush: `(frame, this-frame-dab-bbox)` for the last
    /// `ACTIVE_WINDOW_FRAMES` frames, unioned. This bounds the sim to the actively-blooming
    /// front instead of the monotonic all-time `wet_bbox` (which under Keep Wet grows to the
    /// whole canvas → ~1M settled cells re-simulated every painting frame). Settled areas
    /// freeze; their last composite stays in the preview texture. Reset on a new stroke.
    pub(super) active_history: std::collections::VecDeque<(u64, (u32, u32, u32, u32))>,
}

impl FluidSession {
    pub(super) fn new(device: &wgpu::Device, dims: (u32, u32)) -> Self {
        Self {
            solver: FluidSolver::new(device, dims.0, dims.1),
            compositor: FluidCompositor::new(device),
            dims,
            epoch: u64::MAX,
            frame: 0,
            preview_slot: None,
            preview_slot_seeded: false,
            texture_mode_dirty: None,
            texture_published: false,
            keep_wet: false,
            wet_bbox: None,
            idle_frames: 0,
            active_history: std::collections::VecDeque::new(),
        }
    }

    /// Release the E4 preview slot (if any) back to the `IndividualTextureStore`.
    /// MUST run before the session is dropped or rebuilt (grid-size change) —
    /// the slot id is refcounted and would otherwise leak its canvas-res texture.
    pub(super) fn release_preview_slot(&mut self, renderer: &mut SpriteRenderer) {
        if let Some((id, _, _)) = self.preview_slot.take() {
            renderer.individual_mut().release(id);
        }
        self.texture_published = false;
        self.preview_slot_seeded = false;
    }
}
