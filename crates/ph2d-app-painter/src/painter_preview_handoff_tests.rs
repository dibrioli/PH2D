//! Display gates, the PRODUCER-HANDOFF half — the upload plan's pure refusals + the
//! CPU→GPU→CPU dance on real hardware. Split from `painter_preview_pipeline_tests.rs` (HR-18 file
//! LOC cap); the harness (Screen, oracles, the smoke arming) lives there and is shared.

use crate::painter_bridge_upload::{UploadPlan, plan_upload};
use crate::painter_preview_pipeline_tests::{
    ENTITY, assert_screen_equals, cp, impasto_tool, screen_truth,
};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_effects::adjustments::AdjustmentKind;
use ph2d_preview_slot::PreviewGpu as PainterPreviewGpu;
use ph2d_tool_painter::PainterTool;
use ph2d_tool_runtime::PreviewCache as PainterPreview;
use std::sync::Arc;

/// One frame of the app's preview lifecycle, in `dispatch`'s exact order: the GPU producer gets
#[allow(clippy::too_many_arguments)]
pub(super) fn app_frame(
    renderer: &mut ph2d_render::SpriteRenderer,
    painter: &mut PainterTool,
    session: &mut Option<crate::painter_gpu_preview::PainterGpuPreview>,
    preview: &mut Option<PainterPreview>,
    preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ph2d_editor_core::toast::ToastQueue,
) -> bool {
    let gpu_owns = crate::painter_gpu_preview::try_drive(
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
        let mirror = crate::painter_bridge_upload::own_preview_buffer(
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
    let cache_version = painter.canvas_version();
    crate::painter_bridge_upload::upload_cpu_preview(
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
    let mut toasts = ph2d_editor_core::toast::ToastQueue::default();
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
        plan_upload(&preview, Some(gpu_seeded), Some((4, 4, 8, 8)), 1, false),
        UploadPlan::Full { reuse: Some(9) },
        "a valid bbox over a GPU-seeded slot must still plan a FULL upload — a Partial here \
         patches CPU-lit bytes into the GPU compositor's unlit frame"
    );
}

/// **What the artist sees through the GPU producer is what they would have seen through the CPU one
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_gpu_producer_shows_what_the_cpu_producer_shows() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    for papel in [None, Some([196u8, 170, 120])] {
        let size = 240u32;
        let mut renderer = ph2d_render::SpriteRenderer::new(
            gpu.clone(),
            ph2d_render::GameRt::FORMAT,
            ph2d_render::TextureAtlas::dummy(&gpu),
            8,
        );
        let mut t = impasto_tool(size);
        // O PAPEL do documento (2026-10-06): a GPU compõe sobre ele com a lei inteira da CPU, então o
        if let Some([r, g, b]) = papel {
            ph2d_editor_core::tool::Tool::handle_panel_event(
                &mut t,
                ph2d_editor_core::tool::PanelEvent::SelectOption(
                    ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
                    format!("{r},{g},{b}"),
                ),
            );
            assert_eq!(t.papel(), Some([r, g, b]), "controlo: o papel foi aplicado");
        }
        let (mut session, mut preview, mut toasts) =
            (None, None, ph2d_editor_core::toast::ToastQueue::default());
        let mut preview_gpu: Option<PainterPreviewGpu> = None;

        // Sculpt: a curved stroke, so the relief carries slopes in every direction. A straight one would
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
        let truth = screen_truth(&mut t);
        let lit_pixels = shown
            .as_chunks::<4>()
            .0
            .iter()
            .zip(truth.as_chunks::<4>().0.iter())
            .filter(|(a, _)| a[..3] != [255, 255, 255])
            .count();
        assert!(
            lit_pixels > 2_000,
            "the fixture must actually contain a lit stroke — only {lit_pixels} pixels are not bare \
             white paper, and a blank canvas would match a blank oracle perfectly"
        );
        assert_screen_equals(
            &shown,
            &truth,
            size,
            &format!("GPU producer vs CPU producer, papel {papel:?}"),
        );
        // B15.3: o tique da lâmpada não redobra (janela de 1 texel) e a tela é a da CPU com a luz nova. O 1.º
        for angulo in [120.0, 200.0] {
            t.set_impasto_light_angle(angulo);
            let args = (&mut session, &mut preview, &mut preview_gpu, &mut toasts);
            assert!(app_frame(
                &mut renderer,
                &mut t,
                args.0,
                args.1,
                args.2,
                args.3
            ));
        }
        assert_eq!(
            t.preview_gpu_region(),
            Some((0, 0, 1, 1)),
            "a lâmpada redobrou"
        );
        let slot = preview_gpu.expect("o slot da placa");
        let (_, _, aceso) = renderer
            .individual()
            .readback(&gpu, slot.texture_id)
            .expect("readback");
        assert!(aceso != shown, "controlo: a luz nova mudou a tela");
        let truth = screen_truth(&mut t);
        assert_screen_equals(
            &aceso,
            &truth,
            size,
            &format!("tique da lâmpada, papel {papel:?}"),
        );
    }
}

/// **O PAPEL atravessa o produtor da GPU — o gate que faltava, e a razão de o Enio ver o papel morto.**
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_paper_alone_survives_the_gpu_producer() {
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
    let mut t = crate::painter_preview_pipeline_tests::impasto_tool(size);
    t.set_substrate_depth(1.0);
    let (mut session, mut preview, mut toasts) =
        (None, None, ph2d_editor_core::toast::ToastQueue::default());
    let mut preview_gpu: Option<PainterPreviewGpu> = None;

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
        "um canvas com substrato tem de ser GPU-owned — é a rota em que o artista o vê"
    );
    assert!(
        t.impasto_visible(),
        "precondição: o passe de luz roda (senão este gate compara duas telas em branco)"
    );

    let slot = preview_gpu.expect("the GPU lane owns a live slot");
    let (w, h, shown) = renderer
        .individual()
        .readback(&gpu, slot.texture_id)
        .expect("readback of the preview slot");
    assert_eq!((w, h), (size, size), "slot dims match the canvas");

    // ⚠️ O CONTROLE vem antes da paridade: sem tinta nenhuma, duas telas brancas casariam
    let lit = shown
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[..3] != [255, 255, 255])
        .count();
    assert!(
        lit > 2_000,
        "a tela da GPU saiu papel BRANCO com o substrato ligado ({lit} pixels acesos) — o dente não \
         atravessa o fold, ou o shader o apaga por cobertura zero"
    );
    let truth = screen_truth(&mut t);
    assert_screen_equals(
        &shown,
        &truth,
        size,
        "GPU producer vs CPU producer (só o papel)",
    );
}
