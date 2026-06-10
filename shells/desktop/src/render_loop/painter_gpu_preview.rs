//! Painter GPU live-preview producer (ADR-0045 Phase 3, step 2).
//!
//! The CPU producer in [`super::painter_bridge`] composites the layer stack on
//! the CPU (`take_preview_arc`) and uploads premultiplied bytes into the preview
//! slot. This is the GPU sibling: when the stack is GPU-representable
//! ([`super::painter_gpu_flatten::flatten_for_gpu`] returns `Some`), it
//! composites on the GPU [`LayerCompositor`], premultiplies the straight output
//! via [`PreviewPremul`], and copies the result straight into the SAME
//! `IndividualTextureStore` preview slot — **no CPU readback**. Both producers
//! end in `painter_preview_gpu`; the next frame's `sim_extract` emits the
//! `PreviewOverride` either way.
//!
//! ## Why it wins
//!
//! An adjustment-slider drag changes only `gpu_params` (no layer pixels), so the
//! compositor keeps every layer slice cached (their pixel versions are stable —
//! see `PainterTool::layer_pixel_versions`) and re-runs just the compute
//! (~1.7 ms @1024² vs ~55 ms for the CPU HSB recompose). The straight→premul
//! handoff is the one piece of fresh render work — see [`ph2d_render::premul`]
//! and `HANDOFF_painter_gpu_preview_coord.md`.

use crate::app_state::PainterPreviewGpu;
use ph2d_editor::toast::{Toast, ToastQueue};
use ph2d_gpu::GpuContext;
use ph2d_render::PreviewPremul;
use ph2d_render::SpriteRenderer;
use ph2d_render::layer_compositor::{
    LayerCompositor, LayerOp, LayerPixelProvider, LayerPixels, Region,
};
use ph2d_tool_painter::PainterTool;

/// Per-session GPU state for the Painter live preview: the layer compositor, the
/// straight→premultiplied blit, and a cached [`GpuContext`] handle (cheap
/// `Arc`-backed clone). Created lazily on the first GPU-representable frame and
/// kept for the tool session: the compositor's own slice cache invalidates on a
/// canvas-dims change, and the tool's monotonic pixel versions handle layer-key
/// reuse across sources, so the same instance stays correct across edits.
pub(crate) struct PainterGpuPreview {
    gpu: GpuContext,
    compositor: LayerCompositor,
    premul: PreviewPremul,
}

impl PainterGpuPreview {
    fn new(gpu: &GpuContext) -> Self {
        Self {
            gpu: gpu.clone(),
            compositor: LayerCompositor::new(gpu),
            premul: PreviewPremul::new(gpu),
        }
    }
}

/// Adapts [`PainterTool::preview_layer_pixels`] to the render crate's
/// [`LayerPixelProvider`] — the tool stays decoupled from `ph2d-render`, so the
/// bridge owns this glue.
struct PainterLayerProvider<'a> {
    tool: &'a PainterTool,
}

impl LayerPixelProvider for PainterLayerProvider<'_> {
    fn layer_pixels(&self, key: u64) -> Option<LayerPixels<'_>> {
        self.tool
            .preview_layer_pixels(key)
            .map(|(version, rgba8)| LayerPixels { version, rgba8 })
    }
}

/// Decide GPU-vs-CPU for this frame and, when GPU, recomposite into the preview
/// slot. Returns `true` iff the GPU producer owns the slot (the stack is
/// GPU-representable) — the caller then gates its CPU upload block off. On the
/// CPU branch (no selection, or a stack with mask / clip / reference / masked or
/// non-ported adjustment → `flatten_for_gpu` returns `None`) this is a no-op and
/// the caller runs `take_preview_arc` + the CPU upload path.
///
/// Drains the preview-dirty flag WITHOUT a CPU composite, so it recomposites on
/// the GPU only when the preview actually changed; an idle representable stack
/// keeps its slot. Also drains the tracked dirty-bbox so it can't leak into a
/// later CPU frame.
///
/// A TRIVIAL stack (single visible opaque Normal raster — the composite IS
/// `canvas_rgba`) stays on the CPU path on purpose:
/// - the CPU producer there is zero-composite (`take_preview_arc` hands back the
///   canvas `Arc`) with the B.1 partial dirty-bbox upload, strictly cheaper than
///   a full-slice re-upload + composite + premul + copy per stroke frame;
/// - the fluid E4 zero-readback texture mode only engages on a trivial stack
///   (`preview_is_trivial_stack` — see `painter_fluid_bridge`), so bowing out
///   here guarantees the GPU layer path never claims the preview slot (nor
///   spends a recomposite on intentionally-stale mid-stroke `canvas_rgba`)
///   while a fluid stroke owns the frame.
pub(super) fn try_drive(
    session_slot: &mut Option<PainterGpuPreview>,
    renderer: &mut SpriteRenderer,
    painter: &mut PainterTool,
    selection: Option<u64>,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
) -> bool {
    let Some(sel) = selection else {
        return false;
    };
    let Some((ops, adj_luts)) = gpu_eligible(painter) else {
        return false;
    };
    if painter.take_preview_dirty() {
        let (w, h) = painter.source_size();
        drive(
            session_slot,
            renderer,
            painter,
            sel,
            ops,
            adj_luts,
            w,
            h,
            painter_preview_gpu,
            toasts,
        );
    }
    let _ = painter.take_preview_upload_bbox();
    true
}

/// The GPU-vs-CPU gate, pure half (headless-testable): `Some(ops, adj_luts)` iff
/// this frame's preview should composite on the GPU. The trivial-stack check
/// runs FIRST — `flatten_for_gpu` happily represents a single plain raster, but
/// the CPU path is strictly better there (see [`try_drive`]). Also the gate of
/// the E5 fluid chain ([`drive_fluid_chain`]), so the two drivers can never
/// disagree about who owns a stack shape.
fn gpu_eligible(painter: &PainterTool) -> Option<(Vec<LayerOp>, Vec<f32>)> {
    if painter.preview_is_trivial_stack() {
        return None;
    }
    super::painter_gpu_flatten::flatten_for_gpu(painter.layers())
}

/// **E5 (ADR-0078 S2): the mid-stroke fluid→layer-chain frame for a NON-trivial
/// GPU-representable stack — zero CPU bytes.** Orchestrates the whole chain:
/// `gpu_eligible` (the SAME gate `try_drive` uses, so the two drivers can never
/// disagree about who owns a stack shape) → fluid straight composite
/// (`composite_frame_to_straight_texture` — covers the MONOTONIC dab envelope,
/// `region` ⊇ all earlier frames of the stroke, so the slice is whole even if an
/// early frame fell back to readback) → [`drive_injected`]. On success it drains
/// the tool's preview-dirty flags (the layer producer just recomposited; a later
/// `try_drive` would only redo identical work — harmless, the injected slice
/// survives a same-version provider pass, just wasted GPU). Returns `true` iff
/// the `painter_preview_gpu` slot now holds the recomposite; on `false` the
/// caller falls through to the readback lane (canvas_rgba + the provider path
/// keep the stroke alive with the CPU round-trip).
#[cfg(feature = "fluid")]
#[allow(clippy::too_many_arguments)]
pub(super) fn drive_fluid_chain(
    session_slot: &mut Option<PainterGpuPreview>,
    renderer: &mut SpriteRenderer,
    painter: &mut PainterTool,
    entity_bits: u64,
    fluid: &mut ph2d_painter_fluid::FluidCompositor,
    gpu: &GpuContext,
    grid_region: (u32, u32, u32, u32),
    width: u32,
    height: u32,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
) -> bool {
    let Some((ops, adj_luts)) = gpu_eligible(painter) else {
        return false;
    };
    if fluid
        .composite_frame_to_straight_texture(&gpu.device, &gpu.queue, grid_region)
        .is_none()
    {
        return false;
    }
    let Some(tex) = fluid.straight_texture() else {
        return false;
    };
    if !drive_injected(
        session_slot,
        renderer,
        painter,
        entity_bits,
        ops,
        adj_luts,
        tex,
        width,
        height,
        painter_preview_gpu,
        toasts,
    ) {
        return false;
    }
    let _ = painter.take_preview_dirty();
    let _ = painter.take_preview_upload_bbox();
    true
}

/// **E5 (ADR-0078 S2): mid-stroke fluid drive for a NON-trivial GPU-representable
/// stack — zero CPU bytes.** The fluid compositor hands its STRAIGHT-alpha live
/// texture (the active layer's pre-stroke pixels + the wet wash) here; this
/// injects it GPU→GPU into the layer compositor's cached slice for the ACTIVE
/// layer ([`LayerCompositor::inject_slice_from_texture`]) and then runs the
/// normal composite→premul→slot pipeline ([`drive`] — forced, NOT gated on
/// `take_preview_dirty`: the wet field changed even though no CPU pixel did).
///
/// This module stays the SINGLE owner of the layer compositor + the preview
/// slot: the fluid bridge never builds a second compositor, it only delivers
/// the texture. The injection version is the active layer's CURRENT provider
/// version (NOT bumped) — see the version invariant on
/// `inject_slice_from_texture`: a same-version provider pass (stale mid-stroke
/// `canvas_rgba`) keeps the injection; the pointer-up readback's version bump
/// retires it exactly when `canvas_rgba` catches up.
///
/// Returns `true` iff the slot now holds the injected recomposite (the caller
/// then skips the readback lane this frame and accumulates its catch-up union).
#[cfg(feature = "fluid")]
#[allow(clippy::too_many_arguments)]
fn drive_injected(
    session_slot: &mut Option<PainterGpuPreview>,
    renderer: &mut SpriteRenderer,
    painter: &PainterTool,
    entity_bits: u64,
    ops: Vec<LayerOp>,
    adj_luts: Vec<f32>,
    straight_tex: &wgpu::Texture,
    width: u32,
    height: u32,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
) -> bool {
    if width == 0 || height == 0 {
        return false;
    }
    let Some(active_key) = painter.layers().active().map(|id| id.0) else {
        return false;
    };
    // The injection targets the active layer's slice — if the flatten skipped it
    // (hidden / zero-opacity), there is no slice to feed; readback lane instead.
    if !ops
        .iter()
        .any(|o| matches!(o, LayerOp::Layer { key, .. } if *key == active_key))
    {
        return false;
    }
    let Some((version, _)) = painter.preview_layer_pixels(active_key) else {
        return false;
    };
    let session = session_slot.get_or_insert_with(|| PainterGpuPreview::new(renderer.gpu()));
    if let Err(e) = session.compositor.inject_slice_from_texture(
        &session.gpu,
        &ops,
        active_key,
        straight_tex,
        width,
        height,
        version,
    ) {
        toasts.push(Toast::error(format!(
            "Painter: fluid GPU slice inject falhou ({e}). Caindo no caminho readback."
        )));
        return false;
    }
    drive(
        session_slot,
        renderer,
        painter,
        entity_bits,
        ops,
        adj_luts,
        width,
        height,
        painter_preview_gpu,
        toasts,
    )
}

/// Composite `ops` on the GPU, premultiply, and copy into the preview slot,
/// pointing `painter_preview_gpu` at it. The slot is (re)acquired empty when
/// missing / resized. On any GPU error the slot is released and a toast queued
/// (the CPU path can take over next frame). Returns `true` iff a preview texture
/// is now live in the slot.
#[allow(clippy::too_many_arguments)]
pub(super) fn drive(
    session_slot: &mut Option<PainterGpuPreview>,
    renderer: &mut SpriteRenderer,
    tool: &PainterTool,
    entity_bits: u64,
    ops: Vec<LayerOp>,
    adj_luts: Vec<f32>,
    width: u32,
    height: u32,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
) -> bool {
    if width == 0 || height == 0 {
        return false;
    }
    let session = session_slot.get_or_insert_with(|| PainterGpuPreview::new(renderer.gpu()));

    // 1) GPU composite over the flattened op-list (slices cached by version).
    let provider = PainterLayerProvider { tool };
    if let Err(e) = session.compositor.composite_with_luts(
        &session.gpu,
        &ops,
        &adj_luts,
        &provider,
        width,
        height,
        Region::full(width, height),
    ) {
        toasts.push(Toast::error(format!(
            "Painter: GPU preview composite falhou ({e}). Caindo no caminho CPU."
        )));
        release_slot(renderer, painter_preview_gpu);
        return false;
    }

    // 2) Premultiply the straight `rgba8unorm` output into a COPY_SRC texture
    //    (the sprite preview slot samples PREMULTIPLIED — see `ph2d_render::premul`).
    {
        let Some(comp_out) = session.compositor.output_texture() else {
            return false;
        };
        if session
            .premul
            .run(&session.gpu, comp_out, width, height)
            .is_none()
        {
            return false;
        }
    }

    // 3) Ensure a preview slot of the right size, then COPY the premultiplied
    //    result into it (rgba8unorm → Rgba8UnormSrgb is copy-compatible).
    let slot = ensure_slot(renderer, painter_preview_gpu, entity_bits, width, height);
    let Some(premul_tex) = session.premul.output_texture() else {
        return false;
    };
    if let Err(e) = renderer.copy_texture_into_individual(slot, premul_tex, width, height) {
        toasts.push(Toast::error(format!(
            "Painter: GPU preview copy falhou ({e}). Caindo no caminho CPU."
        )));
        release_slot(renderer, painter_preview_gpu);
        return false;
    }
    true
}

/// Ensure `painter_preview_gpu` holds a slot sized `width × height`; reuse the
/// existing slot when the dims match (the copy overwrites its contents),
/// otherwise release the old slot and acquire a fresh EMPTY one. Returns the
/// slot's `texture_id`. `arc_token` is `0` — the GPU producer has no CPU `Arc`
/// cache token, and the next CPU frame's `arc_token != cache_token` test then
/// correctly forces a full re-upload on a GPU→CPU transition.
fn ensure_slot(
    renderer: &mut SpriteRenderer,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    entity_bits: u64,
    width: u32,
    height: u32,
) -> u32 {
    if let Some(existing) = *painter_preview_gpu
        && existing.width == width
        && existing.height == height
    {
        *painter_preview_gpu = Some(PainterPreviewGpu {
            texture_id: existing.texture_id,
            width,
            height,
            arc_token: 0,
            entity_bits,
        });
        return existing.texture_id;
    }
    if let Some(old) = painter_preview_gpu.take() {
        renderer.individual_mut().release(old.texture_id);
    }
    let id = renderer.acquire_individual_empty(width, height);
    *painter_preview_gpu = Some(PainterPreviewGpu {
        texture_id: id,
        width,
        height,
        arc_token: 0,
        entity_bits,
    });
    id
}

/// Release the preview slot (if any) and clear the GPU cache — on a GPU error so
/// the next frame re-acquires from scratch (or the CPU path takes over). Mirror
/// of `painter_bridge::release_preview_texture`, kept local so the two preview
/// producers don't reach into each other.
fn release_slot(
    renderer: &mut SpriteRenderer,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
) {
    if let Some(gpu) = painter_preview_gpu.take() {
        renderer.individual_mut().release(gpu.texture_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor::tool::RasterEditTool;

    fn sourced_tool() -> PainterTool {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; 4 * 4 * 4], 4, 4);
        t
    }

    #[test]
    fn trivial_stack_stays_on_the_cpu_path() {
        // A single plain raster IS GPU-representable (flatten = Some), but the
        // gate must bow out FIRST: the CPU path is zero-composite + partial
        // bbox upload there, and the fluid E4 texture mode (trivial-only)
        // must never share the frame with a GPU layer recomposite.
        let t = sourced_tool();
        assert!(t.preview_is_trivial_stack());
        assert!(
            super::super::painter_gpu_flatten::flatten_for_gpu(t.layers()).is_some(),
            "precondition: the trivial stack is representable (the gate, not \
             the flatten, must reject it)"
        );
        assert!(gpu_eligible(&t).is_none(), "trivial stack must stay CPU");
    }

    #[test]
    fn non_trivial_representable_stack_is_gpu_eligible() {
        // Opacity < 1 on the single layer breaks triviality without leaving
        // GPU-representability → the GPU path owns the preview.
        let mut t = sourced_tool();
        let active = t.layers().active().expect("set_source creates Layer 1");
        t.set_layer_opacity(active, 0.5);
        assert!(!t.preview_is_trivial_stack());
        let (ops, _luts) = gpu_eligible(&t).expect("representable non-trivial stack → GPU");
        assert!(
            matches!(ops[..], [LayerOp::Layer { opacity, .. }] if (opacity - 0.5).abs() < 1e-6),
            "single half-opacity layer flattens to one Layer op: {ops:?}"
        );
    }

    #[test]
    fn non_representable_stack_is_not_gpu_eligible() {
        // A per-layer mask is outside the GPU op-list v1 → CPU fallback even
        // though the stack is non-trivial.
        let mut t = sourced_tool();
        t.add_mask_to_active().expect("mask on Layer 1");
        assert!(!t.preview_is_trivial_stack());
        assert!(gpu_eligible(&t).is_none(), "masked stack must stay CPU");
    }
}
