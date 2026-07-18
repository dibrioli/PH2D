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
use ph2d_render::{ImpastoLamp, ImpastoLightInput, ImpastoLightPass};
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
    /// Impasto's light — the relief made visible. Runs BETWEEN the composite and the premultiply, which
    /// is exactly where the CPU pass runs (`runtime::take_preview_arc`: composite, light, overlay).
    light: ImpastoLightPass,
    premul: PreviewPremul,
}

impl PainterGpuPreview {
    fn new(gpu: &GpuContext) -> Self {
        Self {
            gpu: gpu.clone(),
            compositor: LayerCompositor::new(gpu),
            light: ImpastoLightPass::new(gpu),
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
/// `canvas_rgba`) stays on the CPU path on purpose: the CPU producer there is
/// zero-composite (`take_preview_arc` hands back the canvas `Arc`) with the B.1
/// partial dirty-bbox upload, strictly cheaper than a full-slice re-upload +
/// composite + premul + copy per stroke frame.
///
/// **Unless the canvas carries relief**, which makes the composite something other
/// than `canvas_rgba` and takes the zero-composite premise away — see
/// [`gpu_eligible`].
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
    // The trivial bow-out rests on the CPU path being ZERO-composite there — and relief falsifies that
    // premise. `take_preview_arc` already refuses the fast lane when `impasto_visible()`, because that
    // lane hands back the raw `canvas_rgba` Arc and the light may never write into the artist's own
    // pixels. So a sculpted single-layer document — the most ordinary way to use Impasto — pays a FULL
    // CPU composite plus a full CPU light on every dirty frame, and the GPU is strictly better.
    //
    // The two lanes have to agree about this or the work goes to whichever one is worse: the guard here
    // mirrors `runtime.rs`'s, deliberately and visibly.
    if painter.preview_is_trivial_stack() && !painter.impasto_visible() {
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

    // 1-3) Composite, light, premultiply — everything that turns the layer stack into the bytes the
    //      slot will hold. On any failure the slot is released and the CPU producer takes the frame.
    if let Err(e) = compose_light_premul(session, tool, &ops, &adj_luts, width, height, region) {
        toasts.push(Toast::error(format!(
            "Painter: GPU preview falhou ({e}). Caindo no caminho CPU."
        )));
        release_slot(renderer, painter_preview_gpu);
        return false;
    }

    // 4) Ensure a preview slot of the right size, then COPY the premultiplied
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

/// Composite, light, premultiply — everything that turns the layer stack into the bytes the preview
/// slot will hold, with no slot and no `SpriteRenderer` in sight. The finished pixels are
/// `session.premul.output_texture()`.
///
/// Extracted so the end-to-end parity gate can drive the REAL chain
/// (`the_gpu_producer_shows_what_the_cpu_producer_shows`) rather than a mirror of it. A gate that
/// re-assembled these three calls itself would go on passing after this function stopped doing them in
/// that order — which is exactly the class of bug it exists to catch.
///
/// # Errors
///
/// A description of which stage refused. Every one of them is a reason to hand the frame back to the CPU
/// producer, which can always draw it.
fn compose_light_premul(
    session: &mut PainterGpuPreview,
    tool: &PainterTool,
    ops: &[LayerOp],
    adj_luts: &[f32],
    width: u32,
    height: u32,
    region: Region,
) -> Result<(), String> {
    // 1) GPU composite into the PERSISTENT canvas-sized `out`, writing at canvas
    //    coords — a region dispatch refreshes only `region`, leaving the rest from
    //    the prior frame (the 4K multi-layer cost goes O(canvas×layers) → O(env)).
    let provider = PainterLayerProvider { tool };
    session
        .compositor
        .composite_region_into_canvas(
            &session.gpu,
            ops,
            adj_luts,
            &provider,
            width,
            height,
            region,
        )
        .map_err(|e| format!("composite: {e}"))?;

    // 2) Impasto: light the freshly-composited region. Same place in the chain as the CPU pass, and
    //    for the same reason — lighting is NOT idempotent, so it must see pixels that were composited
    //    from the layers this frame, never pixels that were already lit. The compositor rewrites
    //    `region` from scratch on every dispatch, so what it feeds here is always fresh.
    //
    //    `None` planes is the ordinary case, not an error: no relief, the pass switched off, or every
    //    lamp dark. Those are the CPU pass's own bails, and they leave the composite untouched.
    //
    //    The match YIELDS the finished texture rather than setting a flag the next step re-reads. That
    //    is not style: "which texture holds the finished canvas" must have exactly one answer, and a
    //    second derivation of it that got the condition backwards would ship an unlit painting with
    //    every gate still green — the failure has no symptom except the artist seeing flat paint.
    let comp_out = session
        .compositor
        .output_texture()
        .ok_or("composite produced no texture")?;
    let finished: &wgpu::Texture = match tool.impasto_gpu_planes() {
        None => comp_out,
        Some(planes) => {
            let lamps: Vec<ImpastoLamp> = planes
                .lamps
                .iter()
                .map(|l| ImpastoLamp {
                    dir: l.dir,
                    half: l.half,
                    tint: l.tint,
                })
                .collect();
            let input = ImpastoLightInput {
                width,
                height,
                // The WHOLE canvas, even when the composite refreshed only a region — and the asymmetry
                // is the point. The light owns a SECOND persistent canvas, so its freshness cannot be
                // inherited from whichever rectangle the compositor happened to touch: one frame with
                // the relief hidden (planes `None`, the compositor refreshes a region, the light never
                // runs) and the next partial lit frame would carry pixels from before that update, in
                // the corner nobody was looking at.
                //
                // It costs nothing today — every dispatch here is already full-canvas — and it stays
                // correct if a partial lane is ever added, merely not optimal, which is the safe
                // direction to be wrong in. It is also always sound: the pass reads the compositor's
                // output (valid everywhere, unlit) and writes its own, so it never re-lights a pixel.
                region: Region::full(width, height),
                relief: &planes.relief,
                cover: &planes.cover,
                mat0: &planes.mat0,
                mat1: &planes.mat1,
                lamps: &lamps,
                spec_lut: planes.spec_lut,
                lut_width: planes.lut_width,
                rough_levels: planes.rough_levels,
            };
            // Falling through UNLIT would composite the layers perfectly and drop the shading on the
            // floor: the artist would sculpt and see nothing, with no error anywhere.
            session
                .light
                .run(&session.gpu, comp_out, &input)
                .map_err(|e| format!("impasto light: {e:?}"))?
        }
    };

    // 3) Premultiply the straight `rgba8unorm` output into a COPY_SRC texture
    //    (the sprite preview slot samples PREMULTIPLIED — see `ph2d_render::premul`).
    //    Full-canvas: a single cheap pass; out-of-region texels are the unchanged
    //    (still-valid) prior composite, so re-premultiplying them is a no-op.
    session
        .premul
        .run(&session.gpu, finished, width, height)
        .ok_or("premultiply produced no texture")?;
    Ok(())
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

    /// A single-layer canvas with a real impasto stroke on it. Big enough that a brush actually deposits
    /// relief — a 4x4 canvas would leave `heights` empty and every gate below would pass by doing
    /// nothing.
    fn sculpted_tool() -> PainterTool {
        use ph2d_editor::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
        let cp = |pos: [f32; 2], phase: PointerPhase| CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 48 * 48 * 4], 48, 48);
        t.toggle_brush_impasto();
        t.set_brush_impasto_depth(0.8);
        t.set_brush_size_px(9.0);
        t.on_canvas_pointer(cp([14.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([34.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([34.0, 24.0], PointerPhase::Up));
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

    /// A sculpted SINGLE-LAYER document — the most ordinary way to use Impasto — must now take the GPU
    /// path. It is the whole point of the light port, and it is the case the old `impasto_visible()` bail
    /// sent to the CPU compositor together with every layer, blend mode and adjustment in the document.
    ///
    /// The premise this rests on is checked here rather than assumed: the stack IS trivial, so without
    /// the relief the gate would (correctly) bow out.
    #[test]
    fn a_sculpted_document_goes_to_the_gpu_even_when_the_stack_is_trivial() {
        let t = sculpted_tool();
        assert!(
            t.preview_is_trivial_stack(),
            "precondition: one plain raster layer — the relief is the ONLY reason this goes GPU"
        );
        assert!(
            t.impasto_visible(),
            "precondition: the canvas carries relief"
        );
        assert!(
            gpu_eligible(&t).is_some(),
            "a sculpted canvas belongs on the GPU: the CPU lane is not zero-composite here \
             (runtime.rs refuses its fast path when impasto_visible), so it pays a full composite \
             AND a full CPU light every dirty frame"
        );
    }

    /// …and the light has something to hand the shader. A gate on eligibility alone would stay green if
    /// the planes came back `None`, which is the shape of "the GPU path claims the frame and draws it
    /// unlit" — the exact failure the old bail existed to prevent.
    #[test]
    fn a_gpu_eligible_sculpted_document_hands_over_planes() {
        let t = sculpted_tool();
        let p = t
            .impasto_gpu_planes()
            .expect("a lit, sculpted canvas must produce planes for the shader");
        assert_eq!(p.relief.len(), (p.width as usize) * (p.height as usize));
        assert!(
            p.cover.iter().any(|c| *c > 0),
            "…and the planes must carry actual paint, not an empty canvas"
        );
        assert!(!p.lamps.is_empty(), "…lit by at least one lamp");
    }

    /// Switching the pass OFF must put the canvas back on the CPU's zero-composite fast lane. Otherwise
    /// hiding the relief would silently cost a full GPU recomposite per frame to draw a picture the CPU
    /// hands over by cloning an `Arc`.
    #[test]
    fn hiding_the_relief_returns_a_trivial_document_to_the_cpu() {
        let mut t = sculpted_tool();
        t.toggle_impasto_show();
        assert!(!t.impasto_visible());
        assert!(
            gpu_eligible(&t).is_none(),
            "no relief on screen -> the trivial bow-out applies again"
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
