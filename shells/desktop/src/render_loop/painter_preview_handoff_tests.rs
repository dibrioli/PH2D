//! Display gates, the PRODUCER-HANDOFF half — the upload plan's pure refusals + the
//! CPU→GPU→CPU dance on real hardware. Split from `painter_preview_pipeline_tests.rs` (HR-18 file
//! LOC cap); the harness (Screen, oracles, the smoke arming) lives there and is shared.

use super::painter_bridge::{UploadPlan, plan_upload};
use super::painter_preview_pipeline_tests::{
    ENTITY, assert_screen_equals, cp, impasto_tool, screen_truth,
};
use crate::app_state::{PainterPreview, PainterPreviewGpu};
use ph2d_editor::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_effects::adjustments::AdjustmentKind;
use ph2d_tool_painter::PainterTool;
use std::sync::Arc;

/// One frame of the app's preview lifecycle, in `dispatch`'s exact order: the GPU producer gets
/// first refusal (`try_drive`), a GPU-owned frame clears the CPU cache, a CPU-owned frame drains
/// the tool and runs the real upload door ([`super::painter_bridge::upload_cpu_preview`]).
/// Returns whether the GPU producer owned the slot this frame.
#[allow(clippy::too_many_arguments)]
fn app_frame(
    renderer: &mut ph2d_render::SpriteRenderer,
    painter: &mut PainterTool,
    session: &mut Option<super::painter_gpu_preview::PainterGpuPreview>,
    preview: &mut Option<PainterPreview>,
    preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ph2d_editor::toast::ToastQueue,
) -> bool {
    let gpu_owns = super::painter_gpu_preview::try_drive(
        session,
        renderer,
        painter,
        Some(ENTITY),
        preview_gpu,
        toasts,
    );
    let mut dirty_bbox = None;
    if gpu_owns {
        *preview = None;
    } else if let Some((drained, w, h)) = painter.take_preview_arc() {
        dirty_bbox = painter.take_preview_upload_bbox();
        // The shell owns its preview buffer (drives the REAL helper), so the tool stays the sole
        // owner of its canvas — exactly as `dispatch` does it.
        let mirror = super::painter_bridge::own_preview_buffer(
            preview.take(),
            ENTITY,
            w,
            h,
            &drained,
            dirty_bbox,
        );
        *preview = Some(PainterPreview {
            entity_bits: ENTITY,
            rgba: mirror,
            width: w,
            height: h,
        });
    }
    // Idle (drain None) reads the unchanged version → the plan Skips; a dirty frame reads the bumped
    // one → it uploads.
    let cache_version = painter.canvas_version();
    super::painter_bridge::upload_cpu_preview(
        renderer,
        preview.as_ref().filter(|_| !gpu_owns),
        dirty_bbox,
        cache_version,
        gpu_owns,
        preview_gpu,
        toasts,
    );
    gpu_owns
}

/// **The screen survives the producer handoffs — CPU→GPU→CPU, on real hardware.**
///
/// The producer handoff dance: the GPU producer takes the slot when the stack is representable, the
/// CPU producer reclaims it when it is not, and **the first CPU frame after a GPU stretch must
/// re-seed the slot whole** — the two defects this phase found (`take_preview_dirty` leaving the CPU
/// composite stale + the partial plan patching a GPU-seeded slot) both live exactly on that seam,
/// latent today only because every eligibility-flipping door happens to invalidate the composite.
/// This gate runs the seam end-to-end — real `try_drive`, real upload door, real wgpu textures, real
/// readback — and holds the final screen bytes to a from-scratch recompose.
///
/// ## The lever keeps moving, the gate does not
///
/// It first flipped eligibility with `impasto_show` (premise: *"the GPU compositor cannot light
/// relief"*), then with a layer **mask** (2026-07-18, once the light port made relief GPU-eligible).
/// Then the GPU Ondas made a mask an OP too (`docs/Painter/25_avaliacao_gpu.md`), so a mask no longer
/// moves a document between producers either — the lever is now an **unported adjustment**
/// (`ColorBalance`: no scalar and no spatial GPU code, so `flatten_for_gpu` still refuses it). Nothing
/// this gate proves has changed: it needs a door that flips eligibility, not any particular door — and
/// each time a wave widened what the GPU can represent, this gate's lever had to be the thing STILL
/// outside it (⚠️ and this gate is GPU-adapter-only, so `ship.sh` never runs it — the mask lever went
/// stale-green-latent for exactly one wave before this run caught it).
///
/// `#[ignore]`: needs a GPU adapter (none on CI); run locally with `--ignored`.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_screen_survives_the_gpu_to_cpu_producer_handoff() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    let size = 240u32;
    let mut renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );
    let mut t = impasto_tool(size);
    let mut session = None;
    let mut preview = None;
    let mut preview_gpu: Option<PainterPreviewGpu> = None;
    let mut toasts = ph2d_editor::toast::ToastQueue::default();
    let counter = std::cell::Cell::new(0u32);
    let frame = |t: &mut PainterTool,
                 renderer: &mut ph2d_render::SpriteRenderer,
                 session: &mut _,
                 preview: &mut Option<PainterPreview>,
                 preview_gpu: &mut Option<PainterPreviewGpu>,
                 toasts: &mut _| {
        let n = counter.get();
        counter.set(n + 1);
        let owns = app_frame(renderer, t, session, preview, preview_gpu, toasts);
        eprintln!(
            "[handoff-diag] frame {n}: gpu_owns={owns} cache={} slot={:?}",
            preview.is_some(),
            preview_gpu.map(|g| (g.texture_id, g.arc_token != 0)),
        );
        owns
    };

    // 1. Trivial stack, nothing painted yet → the CPU producer owns (the trivial bow-out: its lane
    //    is zero-composite there, strictly cheaper than a GPU round trip).
    assert!(
        !frame(
            &mut t,
            &mut renderer,
            &mut session,
            &mut preview,
            &mut preview_gpu,
            &mut toasts
        ),
        "a trivial stack stays on the CPU path"
    );
    t.on_canvas_pointer(cp([80.0, 120.0], PointerPhase::Down));
    frame(
        &mut t,
        &mut renderer,
        &mut session,
        &mut preview,
        &mut preview_gpu,
        &mut toasts,
    );
    for i in 1u8..=4 {
        t.on_canvas_pointer(cp([80.0 + 12.0 * f32::from(i), 120.0], PointerPhase::Move));
        frame(
            &mut t,
            &mut renderer,
            &mut session,
            &mut preview,
            &mut preview_gpu,
            &mut toasts,
        );
    }
    t.on_canvas_pointer(cp([128.0, 120.0], PointerPhase::Up));
    frame(
        &mut t,
        &mut renderer,
        &mut session,
        &mut preview,
        &mut preview_gpu,
        &mut toasts,
    );

    // 2. The stroke put RELIEF on the canvas, which takes the trivial bow-out away (the CPU lane
    //    refuses its zero-composite fast path once `impasto_visible`) → CPU→GPU handoff, with the GPU
    //    doing the lighting. This is the step the light port exists for.
    let l2 = t.add_raster_layer("Layer 2").expect("layer 2");
    t.set_layer_opacity(l2, 0.5);
    assert!(
        frame(
            &mut t,
            &mut renderer,
            &mut session,
            &mut preview,
            &mut preview_gpu,
            &mut toasts
        ),
        "a sculpted, representable stack belongs to the GPU producer — relief and all"
    );

    // 3. …and it keeps the slot while nothing flips eligibility.
    assert!(
        frame(
            &mut t,
            &mut renderer,
            &mut session,
            &mut preview,
            &mut preview_gpu,
            &mut toasts
        ),
        "an idle representable stack keeps its GPU-owned slot"
    );
    t.on_canvas_pointer(cp([60.0, 180.0], PointerPhase::Down));
    frame(
        &mut t,
        &mut renderer,
        &mut session,
        &mut preview,
        &mut preview_gpu,
        &mut toasts,
    );
    for i in 1u8..=4 {
        t.on_canvas_pointer(cp([60.0 + 15.0 * f32::from(i), 180.0], PointerPhase::Move));
        assert!(
            frame(
                &mut t,
                &mut renderer,
                &mut session,
                &mut preview,
                &mut preview_gpu,
                &mut toasts
            ),
            "plain strokes on the representable stack stay GPU-owned"
        );
    }
    t.on_canvas_pointer(cp([120.0, 180.0], PointerPhase::Up));
    frame(
        &mut t,
        &mut renderer,
        &mut session,
        &mut preview,
        &mut preview_gpu,
        &mut toasts,
    );

    // 4. An UNPORTED adjustment (ColorBalance has no scalar and no spatial GPU code) → outside the
    //    GPU op-list → GPU→CPU handoff; then one more impasto stroke so the partial lane runs over
    //    the re-seeded slot. `add_adjustment_layer` leaves the previously-active RASTER as the paint
    //    target, so the follow-up strokes still deposit. (The lever moved off the mask: this line's
    //    own wave made masks GPU-representable, so a mask no longer flips eligibility — see the module
    //    doc.)
    t.add_adjustment_layer(AdjustmentKind::ColorBalance)
        .expect("an unported adjustment above the stack");
    assert!(
        !frame(
            &mut t,
            &mut renderer,
            &mut session,
            &mut preview,
            &mut preview_gpu,
            &mut toasts
        ),
        "an unported-adjustment stack is not GPU-representable: the CPU producer reclaims the slot"
    );
    t.on_canvas_pointer(cp([90.0, 60.0], PointerPhase::Down));
    frame(
        &mut t,
        &mut renderer,
        &mut session,
        &mut preview,
        &mut preview_gpu,
        &mut toasts,
    );
    t.on_canvas_pointer(cp([130.0, 60.0], PointerPhase::Move));
    frame(
        &mut t,
        &mut renderer,
        &mut session,
        &mut preview,
        &mut preview_gpu,
        &mut toasts,
    );
    t.on_canvas_pointer(cp([130.0, 60.0], PointerPhase::Up));
    frame(
        &mut t,
        &mut renderer,
        &mut session,
        &mut preview,
        &mut preview_gpu,
        &mut toasts,
    );
    frame(
        &mut t,
        &mut renderer,
        &mut session,
        &mut preview,
        &mut preview_gpu,
        &mut toasts,
    );

    // The screen (the real slot texture, read back) equals the document composited from scratch.
    let slot = preview_gpu.expect("the CPU lane owns a live slot (no upload error)");
    let (w, h, shown) = renderer
        .individual()
        .readback(&gpu, slot.texture_id)
        .expect("readback of the preview slot");
    assert_eq!((w, h), (size, size), "slot dims match the canvas");
    let truth = screen_truth(&mut t);
    assert_screen_equals(
        &shown,
        &truth,
        size,
        "after the CPU->GPU->CPU producer dance",
    );
}

/// **A slot the GPU producer seeded is never PATCHED — the handoff must re-seed it whole.**
///
/// The GPU preview producer stamps its slots with `arc_token: 0` ("no CPU Arc") precisely so the
/// first CPU frame after a GPU-owned stretch forces a full re-upload — the slot holds the GPU
/// compositor's output (unlit, another era), and a rect patched over it leaves every other pixel
/// to a different producer. The dims/entity guards can't tell the producers apart (the GPU lane
/// reuses the same slot at the same dims), so the token is the ONLY witness. Today this is latent
/// — every door that flips producer eligibility happens to invalidate the tool's composite, so no
/// bbox reaches this plan on a handoff frame — but that safety is an enumeration of doors, and
/// door N+1 (a future eligibility flip that forgets to invalidate) lands on exactly this line.
///
/// **Mutation that must bleed:** drop the `g.arc_token != 0` arm of the partial guard in
/// [`plan_upload`] — the plan below comes back `Partial`.
#[test]
fn a_gpu_seeded_slot_is_reseeded_whole_never_patched() {
    let preview = PainterPreview {
        entity_bits: ENTITY,
        rgba: Arc::new(vec![3u8; 16 * 16 * 4]),
        width: 16,
        height: 16,
    };
    let gpu_seeded = PainterPreviewGpu {
        texture_id: 9,
        width: 16,
        height: 16,
        arc_token: 0, // the GPU producer's stamp (`ensure_slot`)
        entity_bits: ENTITY,
    };
    assert_eq!(
        // A real CPU content version (1) against the GPU stamp (0): the version differs so an upload
        // is due, and the `g.arc_token != 0` arm must still force it FULL rather than Partial.
        plan_upload(&preview, Some(gpu_seeded), Some((4, 4, 8, 8)), 1, false),
        UploadPlan::Full { reuse: Some(9) },
        "a valid bbox over a GPU-seeded slot must still plan a FULL upload — a Partial here \
         patches CPU-lit bytes into the GPU compositor's unlit frame"
    );
}

/// **What the artist sees through the GPU producer is what they would have seen through the CPU one
/// — on a SCULPTED canvas, with the relief lit by the shader.**
///
/// This is the gate the GPU light port is answerable to. Everything else about the port is checked a
/// level down (`ph2d-render`'s `impasto_light_gpu` reconciles the shader against
/// `apply_impasto_light` pixel for pixel), but that gate hands both sides the same synthetic buffer.
/// This one runs the PRODUCT: the real `try_drive`, the real eligibility gate, the real flatten, the
/// real GPU compositor, the real light pass, the real premultiply, the real slot texture — and reads
/// the bytes the sprite shader will sample straight back off the device.
///
/// The oracle is the OTHER producer, which is the only oracle that answers the question actually
/// being asked. Moving a document between two lanes is only safe if the lanes draw the same picture;
/// a gate comparing the GPU lane to a re-derivation of what it ought to draw would agree with any bug
/// the two share, and the whole risk of this change is that they might not share one.
///
/// **Why this could not have been written before 2026-07-18:** the eligibility gate sent every
/// sculpted canvas to the CPU, so there was no GPU-owned frame with relief on it to read back.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_gpu_producer_shows_what_the_cpu_producer_shows() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    let size = 240u32;
    let mut renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );
    let mut t = impasto_tool(size);
    let (mut session, mut preview, mut toasts) =
        (None, None, ph2d_editor::toast::ToastQueue::default());
    let mut preview_gpu: Option<PainterPreviewGpu> = None;

    // Sculpt: a curved stroke, so the relief carries slopes in every direction. A straight one would
    // leave the normal pointing the same way down the whole stroke and a sign error could hide.
    t.on_canvas_pointer(cp([60.0, 90.0], PointerPhase::Down));
    for i in 1u8..=6 {
        let f = f32::from(i);
        t.on_canvas_pointer(cp(
            [60.0 + 22.0 * f, 90.0 + 9.0 * f * f * 0.25],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp([192.0, 171.0], PointerPhase::Up));

    // Run frames until the producers settle. The GPU must be the one holding the slot: if the
    // eligibility gate ever sends a sculpted canvas back to the CPU this reads as a plain failure
    // here rather than as a silent loss of the speed the port was built for.
    let mut gpu_owns = false;
    for _ in 0..4 {
        gpu_owns = app_frame(
            &mut renderer,
            &mut t,
            &mut session,
            &mut preview,
            &mut preview_gpu,
            &mut toasts,
        );
    }
    assert!(
        gpu_owns,
        "a sculpted canvas must be GPU-owned — that is the whole point of the light port"
    );
    assert!(
        t.impasto_visible(),
        "precondition: there IS relief to light (a flat canvas would make this gate vacuous)"
    );

    // The bytes the sprite shader samples, straight off the device.
    let slot = preview_gpu.expect("the GPU lane owns a live slot");
    let (w, h, shown) = renderer
        .individual()
        .readback(&gpu, slot.texture_id)
        .expect("readback of the preview slot");
    assert_eq!((w, h), (size, size), "slot dims match the canvas");

    // …against the CPU producer's own answer for the same document. `screen_truth` drains the tool,
    // so it runs AFTER the readback.
    let truth = screen_truth(&mut t);
    let lit_pixels = shown
        .chunks_exact(4)
        .zip(truth.chunks_exact(4))
        .filter(|(a, _)| a[..3] != [255, 255, 255])
        .count();
    assert!(
        lit_pixels > 2_000,
        "the fixture must actually contain a lit stroke — only {lit_pixels} pixels are not bare \
         white paper, and a blank canvas would match a blank oracle perfectly"
    );
    assert_screen_equals(&shown, &truth, size, "GPU producer vs CPU producer");
}
