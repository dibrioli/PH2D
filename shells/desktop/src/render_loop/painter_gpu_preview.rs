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
/// - the zero-readback texture mode only engages on a trivial stack
///   (`preview_is_trivial_stack`), so bowing out here guarantees the GPU layer
///   path never claims the preview slot (nor spends a recomposite on
///   intentionally-stale mid-stroke `canvas_rgba`).
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
        // CPU/hover path: a full recomposite (no live wet envelope) — seed the whole
        // canvas (`seed_full = true`, full slot copy), byte-identical to the old
        // `Region::full` composite.
        drive(
            session_slot,
            renderer,
            painter,
            sel,
            ops,
            adj_luts,
            w,
            h,
            (0, 0, w, h),
            true,
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
/// the CPU path is strictly better there (see [`try_drive`]).
fn gpu_eligible(painter: &PainterTool) -> Option<(Vec<LayerOp>, Vec<f32>)> {
    if painter.preview_is_trivial_stack() {
        return None;
    }
    // Impasto's light pass is CPU-side (it reads the height field, which the GPU compositor knows
    // nothing about). Taking the GPU path with relief on screen would composite the layers correctly
    // and drop the shading on the floor — the artist would sculpt and see nothing. Fall back to the
    // CPU compositor, which is already the supported path and already what a mask scratch uses.
    // (The GPU light pass is named and deferred: a new `LayerOp`, reconciled bit-for-bit against this
    // CPU one — `docs/Painter/16…` §6.)
    if painter.impasto_visible() {
        return None;
    }
    super::painter_gpu_flatten::flatten_for_gpu(painter.layers())
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
    rect: (u32, u32, u32, u32),
    seed_full: bool,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
) -> bool {
    if width == 0 || height == 0 {
        return false;
    }
    let session = session_slot.get_or_insert_with(|| PainterGpuPreview::new(renderer.gpu()));

    // The composite region: the WHOLE canvas on a seed frame (so the persistent
    // `out` + the slot hold a valid full backdrop), else just the wet envelope.
    let region = if seed_full {
        Region::full(width, height)
    } else {
        Region {
            x: rect.0,
            y: rect.1,
            w: rect.2,
            h: rect.3,
        }
    };

    // 1) GPU composite into the PERSISTENT canvas-sized `out`, writing at canvas
    //    coords — a region dispatch refreshes only `region`, leaving the rest from
    //    the prior frame (the 4K multi-layer cost goes O(canvas×layers) → O(env)).
    let provider = PainterLayerProvider { tool };
    if let Err(e) = session.compositor.composite_region_into_canvas(
        &session.gpu,
        &ops,
        &adj_luts,
        &provider,
        width,
        height,
        region,
    ) {
        toasts.push(Toast::error(format!(
            "Painter: GPU preview composite falhou ({e}). Caindo no caminho CPU."
        )));
        release_slot(renderer, painter_preview_gpu);
        return false;
    }

    // 2) Premultiply the straight `rgba8unorm` output into a COPY_SRC texture
    //    (the sprite preview slot samples PREMULTIPLIED — see `ph2d_render::premul`).
    //    Full-canvas: a single cheap pass; out-of-region texels are the unchanged
    //    (still-valid) prior composite, so re-premultiplying them is a no-op.
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
    //    result into it (rgba8unorm → Rgba8UnormSrgb is copy-compatible). The slot
    //    persists across the stroke, so a seed frame copies the WHOLE canvas and
    //    later frames copy only the wet envelope rect on top of it.
    let slot = ensure_slot(renderer, painter_preview_gpu, entity_bits, width, height);
    let Some(premul_tex) = session.premul.output_texture() else {
        return false;
    };
    let copy = if seed_full {
        renderer.copy_texture_into_individual(slot, premul_tex, width, height)
    } else {
        renderer.copy_texture_region_into_individual(
            slot, premul_tex, rect.0, rect.1, rect.0, rect.1, rect.2, rect.3,
        )
    };
    if let Err(e) = copy {
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
        let (ops, _luts) = gpu_eligible(&t).expect("representable non-trivial stack -> GPU");
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
