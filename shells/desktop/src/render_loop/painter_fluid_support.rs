//! Support pieces for the W15.3 GPU fluid drive (`painter_fluid_bridge`) — the
//! opt-in per-phase profiler, bbox helpers, and the preview-slot plumbing. Split
//! out of the bridge for HR-18 (≤600 LOC per shell file); the bridge keeps the
//! per-frame drive logic, this module keeps the leaf utilities.

use super::painter_gpu_preview::{self, PainterGpuPreview};
use super::sim_extract::PreviewOverride;
use crate::app_state::PainterPreviewGpu;
use ph2d_editor::toast::ToastQueue;
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

/// **Readback lane** — any frame the zero-readback texture paths (E4/E5) didn't run:
/// composite PIPELINED (async readback, no per-frame `device.poll(wait)` stall; the band
/// returned is the PRIOR frame's — imperceptible, the same lag the preview always had;
/// `begin_stroke` drains the in-flight map before reuse) and keep `canvas_rgba` current.
///
/// E4 catch-up: the FIRST readback frames after the texture→readback switch must
/// composite the FULL union the texture path covered (`texture_mode_dirty`), so
/// `canvas_rgba` catches up on everything it skipped. Because the band returned NOW is
/// from the PRIOR submission, the union keeps being fed until 2 CONSECUTIVE bands
/// returned while feeding it — the 2nd is necessarily from a submission that included it.
///
/// Transition hand-off: while `texture_published`, the (one-frame-stale) fluid texture
/// override is re-emitted so the stroke never flickers back to the pre-catch-up CPU
/// preview; the flag drops on the first applied band. Finally, with the pointer up and
/// Show Wet on, the view-only wet sheen is recomposited + republished
/// ([`publish_wet_sheen_between_strokes`]).
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
    epoch: u64,
    painter_gpu_preview_session: &mut Option<PainterGpuPreview>,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
) -> Option<PreviewOverride> {
    let mut override_out = None;
    let rb_region = match sess.texture_mode_dirty {
        Some(d) => union_bbox(region, d),
        None => region,
    };
    let (band, rect) = sess
        .compositor
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
        // canvas_rgba just caught up, but the CPU upload happens LATER this frame and
        // sim_extract samples the CPU slot with a 1-frame lag — keep the fluid texture
        // override for THIS frame only, then hand the preview back to the CPU slot.
        if sess.texture_published {
            sess.texture_published = false;
            if let (Some((id, _, _)), Some(entity_bits)) = (sess.preview_slot, override_entity) {
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
        // No band yet (first readback frame after the switch — nothing was pending):
        // the CPU preview is still pre-catch-up, so keep showing the fluid texture
        // to avoid a stroke-disappears flicker.
        if sess.texture_published
            && let (Some((id, _, _)), Some(entity_bits)) = (sess.preview_slot, override_entity)
        {
            override_out = Some(PreviewOverride {
                entity_bits,
                texture_id: id,
                premultiplied: true,
            });
        }
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
            epoch,
            painter_gpu_preview_session,
            painter_preview_gpu,
            toasts,
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
/// keep-wet) and then "dries lighter". Trivial stacks publish the premultiplied
/// preview slot directly; a non-trivial GPU-representable stack mirrors the E5 lane
/// via [`painter_gpu_preview::drive_fluid_chain`] (which fills `painter_preview_gpu`,
/// whose override the render loop emits itself → returns `None` here).
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
    epoch: u64,
    painter_gpu_preview_session: &mut Option<PainterGpuPreview>,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
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
        // E5 mirror: the chain owns the layer compositor + the `painter_preview_gpu`
        // slot; on failure the plain (sheen-free) CPU preview keeps the frame alive.
        let _ = painter_gpu_preview::drive_fluid_chain(
            painter_gpu_preview_session,
            renderer,
            painter,
            entity_bits,
            compositor,
            gpu,
            region,
            cw,
            ch,
            epoch,
            painter_preview_gpu,
            toasts,
        );
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
    /// **All-time wet bbox of this stroke** (union of the sporadic `read_field_stats` bboxes,
    /// reset on a new epoch). The capillary fringe wicks the wet region OUTWARD past the dab
    /// bboxes; the water bbox also recedes as the wash dries, so a SUPERSET-correct envelope
    /// must take the monotonic union (the §3.4 / §2.2 lesson). `None` until the first read.
    pub(super) wet_bbox: Option<(u32, u32, u32, u32)>,
    /// **E4 (ADR-0078 S2): the `IndividualTextureStore` slot** the mid-stroke texture path
    /// GPU-copies the compositor's premultiplied preview into — `(texture_id, w, h)` so a
    /// canvas-size change releases + re-acquires. `None` until the first texture-mode frame
    /// (lazy, mirroring `painter_gpu_preview::ensure_slot`). MUST be released via
    /// [`Self::release_preview_slot`] before the session is dropped or rebuilt.
    pub(super) preview_slot: Option<(u32, u32, u32)>,
    /// **E4 catch-up accumulator**: union of every GRID region the zero-readback texture
    /// path composited (canvas_rgba is intentionally stale over it mid-stroke). The first
    /// readback frames after the texture→readback switch feed this union into
    /// `composite_frame_pipelined` so `canvas_rgba` catches up on EVERYTHING the texture
    /// path skipped; cleared only once the (1-frame-late) pipelined path has returned 2
    /// consecutive bands while the union was being fed (the 2nd is necessarily from a
    /// submission that included it).
    pub(super) texture_mode_dirty: Option<(u32, u32, u32, u32)>,
    /// Consecutive non-empty pipelined bands applied while `texture_mode_dirty` was being
    /// fed (reset by an empty band and by every texture-mode frame). At 2 the union is
    /// provably applied → cleared.
    pub(super) catchup_bands: u8,
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
    /// Consecutive frames with the pointer up AND no dabs (perf block 2b). Paces the
    /// idle decimation: with a live field but nothing being painted, the sim step +
    /// composite/sheen run only every `IDLE_STEP_EVERY` frames (the field barely moves
    /// idle; recompositing the whole envelope per frame was the dominant idle GPU cost,
    /// §4b). Reset to 0 by any dab or active stroke.
    pub(super) idle_frames: u64,
}

impl FluidSession {
    pub(super) fn new(device: &wgpu::Device, dims: (u32, u32)) -> Self {
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
            keep_wet: false,
            idle_frames: 0,
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
    }
}
