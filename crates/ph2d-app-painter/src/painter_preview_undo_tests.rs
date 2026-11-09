//! **O que a tela mostra DEPOIS de um undo** — o report de resquício do smoke `PH2D_IMPASTO_SMOKE=2`.
//!
//! Enio, 2026-07-25, com foto: *"UNDO deixa resquícios na tela"* — um retângulo branco, axis-aligned,
//! recortado no traço perto do cursor. Um retângulo na saída é a impressão digital de trabalho limitado
//! por REGIÃO, e a pista GPU tem três: o upload por-camada (Onda 5b), o fold do relevo (esta wave) e o
//! `dirty` que os dois leem.
//!
//! Este módulo reproduz a sequência no caminho REAL (o mesmo `app_frame` que o gate de paridade usa:
//! `try_drive` de verdade, compositor de verdade, passe de luz de verdade, slot de verdade, readback do
//! device) e confronta o que o produtor GPU mostra com o que o produtor CPU mostraria para o MESMO
//! documento — o único oráculo que responde à pergunta que o report faz.

use crate::painter_preview_handoff_tests::app_frame;
use crate::painter_preview_pipeline_tests::{assert_screen_equals, cp, impasto_tool, screen_truth};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_preview_slot::PreviewGpu as PainterPreviewGpu;
use ph2d_tool_painter::PainterTool;
use ph2d_tool_runtime::PreviewCache as PainterPreview;

/// **Um undo deixa a tela idêntica ao que o outro produtor desenharia.**
///
/// A hipótese que este gate testa é a que a foto sugere: o undo troca o canvas INTEIRO
/// (`restore_model` reinstala `canvas_rgba`, `heights`, `covers`, `mats`), então qualquer upload ou fold
/// que ainda esteja limitado ao retângulo do último dab mostraria a arte nova dentro dele e a VELHA
/// fora — um retângulo, exatamente como o da foto.
///
/// ⚠️ O fixture tem de conter DOIS traços e desfazer o segundo: desfazer o único traço de um canvas
/// deixa a tela branca, e branco casa com branco por qualquer caminho errado
/// ([[feedback_a_fixture_only_proves_what_it_contains]]).
///
/// `#[ignore]`: precisa de adapter; rode com `--release --ignored`.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_screen_after_an_undo_is_what_the_other_producer_would_draw() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    // 240² prova a LEI; 1024² e 4096² provam que ela não depende da escala — e efeito de região
    // escala com a tela, então um fixture pequeno pode não conter o fenômeno que o smoke viu.
    let size: u32 = std::env::var("PH2D_UNDO_REPRO_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(240);
    let mut renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );
    let mut t = impasto_tool(size);
    let (mut session, mut preview, mut toasts) =
        (None, None, ph2d_editor_core::toast::ToastQueue::default());
    let mut preview_gpu: Option<PainterPreviewGpu> = None;
    let run =
        |t: &mut PainterTool,
         renderer: &mut ph2d_render::SpriteRenderer,
         session: &mut _,
         preview: &mut Option<PainterPreview>,
         preview_gpu: &mut Option<PainterPreviewGpu>,
         toasts: &mut _| { app_frame(renderer, t, session, preview, preview_gpu, toasts) };

    // Dois traços, cada um dirigido frame a frame como a mão faz.
    for (y, x0) in [(90.0f32, 40.0f32), (150.0, 40.0)] {
        t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
        for i in 1u8..=6 {
            t.on_canvas_pointer(cp([x0 + 28.0 * f32::from(i), y], PointerPhase::Move));
            run(
                &mut t,
                &mut renderer,
                &mut session,
                &mut preview,
                &mut preview_gpu,
                &mut toasts,
            );
        }
        t.on_canvas_pointer(cp([x0 + 168.0, y], PointerPhase::Up));
        run(
            &mut t,
            &mut renderer,
            &mut session,
            &mut preview,
            &mut preview_gpu,
            &mut toasts,
        );
    }

    // O undo que o shell dispara no Cmd+Z (`painter_bridge`: `painter.undo_last()`).
    t.undo_last();
    let mut gpu_owns = false;
    for _ in 0..3 {
        gpu_owns = run(
            &mut t,
            &mut renderer,
            &mut session,
            &mut preview,
            &mut preview_gpu,
            &mut toasts,
        );
    }
    assert!(
        gpu_owns,
        "precondição: o documento esculpido continua com o produtor GPU depois do undo — se ele \
         voltou para a CPU, este gate está medindo outra coisa"
    );

    let slot = preview_gpu.expect("o produtor GPU tem um slot vivo");
    let (w, h, shown) = renderer
        .individual()
        .readback(&gpu, slot.texture_id)
        .expect("readback do slot de preview");
    assert_eq!((w, h), (size, size));

    // O oráculo é o OUTRO produtor, para o MESMO documento pós-undo. `screen_truth` drena a tool, então
    // roda depois do readback.
    let truth = screen_truth(&mut t);
    let painted = truth
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[..3] != [255, 255, 255])
        .count();
    assert!(
        painted > 2_000,
        "o fixture tem de sobrar com tinta depois do undo — só {painted} pixels não são papel nu, e \
         uma tela branca casa com um oráculo branco por qualquer caminho errado"
    );
    assert_screen_equals(
        &shown,
        &truth,
        size,
        "pós-undo: produtor GPU vs produtor CPU",
    );
}

/// **O produtor devolve o frame para a GPU depois de a CPU ter sido dona — e os planos têm de estar
/// atuais.** Report do Enio (2026-07-25, smoke 4096²): o retângulo «aparece pintando» e fica até nova
/// pintura, depois de alternar Digital ↔ Impasto.
///
/// Mecanismo (bisseção; cura `gpu_lane_stale`): o `dirty_rect` é compartilhado pelas duas pistas e
/// consumido por quem drena; enquanto a CPU é dona ela o leva, e no frame em que a GPU retoma o rect
/// descreve só o ÚLTIMO frame da CPU. Os dois consumidores de `preview_dirty_region` (compositor e fold)
/// adoeciam juntos. A cura espelha a da Fase D: toda drenagem da CPU levanta `gpu_lane_stale` e a
/// próxima da GPU o consome declarando não-confinado (`None` = faça inteiro).
///
/// Mutação que sangra: `self.gpu_lane_stale = true` → `false` em `take_preview_arc` devolve os 197.172
/// bytes exatos. O oráculo é o OUTRO produtor (re-derivar o esperado concordaria com o bug).
///
/// `#[ignore]`: precisa de adapter; rode com `--release --ignored`.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_planes_are_current_when_the_gpu_lane_takes_the_frame_back() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    let size: u32 = std::env::var("PH2D_UNDO_REPRO_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(512);
    let mut renderer = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        ph2d_render::GameRt::FORMAT,
        ph2d_render::TextureAtlas::dummy(&gpu),
        8,
    );
    let mut t = impasto_tool(size);
    let (mut session, mut preview, mut toasts) =
        (None, None, ph2d_editor_core::toast::ToastQueue::default());
    let mut preview_gpu: Option<PainterPreviewGpu> = None;
    let stroke = |t: &mut PainterTool,
                  renderer: &mut ph2d_render::SpriteRenderer,
                  session: &mut _,
                  preview: &mut Option<PainterPreview>,
                  preview_gpu: &mut Option<PainterPreviewGpu>,
                  toasts: &mut _,
                  y: f32|
     -> bool {
        let mut owns = false;
        t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
        for i in 1u8..=6 {
            t.on_canvas_pointer(cp([40.0 + 60.0 * f32::from(i), y], PointerPhase::Move));
            owns = app_frame(renderer, t, session, preview, preview_gpu, toasts);
        }
        t.on_canvas_pointer(cp([400.0, y], PointerPhase::Up));
        owns = app_frame(renderer, t, session, preview, preview_gpu, toasts) || owns;
        owns
    };

    // 1. Impasto: o relevo nasce, a pista GPU assume e SEMEIA os planos.
    let gpu_1 = stroke(
        &mut t,
        &mut renderer,
        &mut session,
        &mut preview,
        &mut preview_gpu,
        &mut toasts,
        100.0,
    );
    assert!(
        gpu_1,
        "precondição: o traço com relevo vai para a pista GPU"
    );

    // 2. Undo: o relevo some, o documento volta a ser trivial -> a pista CPU reassume.
    t.undo_last();
    for _ in 0..2 {
        app_frame(
            &mut renderer,
            &mut t,
            &mut session,
            &mut preview,
            &mut preview_gpu,
            &mut toasts,
        );
    }

    // 3. Pintar em DIGITAL enquanto a CPU é dona — é AQUI que o `dirty_rect` é consumido por ela.
    t.set_paint_media(ph2d_tool_painter::PaintMedia::Digital);
    let gpu_2 = stroke(
        &mut t,
        &mut renderer,
        &mut session,
        &mut preview,
        &mut preview_gpu,
        &mut toasts,
        200.0,
    );
    assert!(
        !gpu_2,
        "precondição: sem relevo o documento trivial é da pista CPU — se a GPU o pegou, este gate \
         não está montando o cenário que o report descreve"
    );

    // 4. Impasto de novo: a GPU retoma com `planes_seeded` ainda true.
    t.set_paint_media(ph2d_tool_painter::PaintMedia::Impasto);
    let gpu_3 = stroke(
        &mut t,
        &mut renderer,
        &mut session,
        &mut preview,
        &mut preview_gpu,
        &mut toasts,
        300.0,
    );
    assert!(gpu_3, "a pista GPU retoma o frame");

    let slot = preview_gpu.expect("o produtor GPU tem um slot vivo");
    let (w, h, shown) = renderer
        .individual()
        .readback(&gpu, slot.texture_id)
        .expect("readback do slot");
    assert_eq!((w, h), (size, size));
    let truth = screen_truth(&mut t);
    let painted = truth
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[..3] != [255, 255, 255])
        .count();
    assert!(
        painted > 5_000,
        "o fixture tem de ter tinta dos DOIS trechos (só {painted} pixels não são papel)"
    );
    assert_screen_equals(
        &shown,
        &truth,
        size,
        "GPU retomando o frame depois de a CPU ter sido dona",
    );
}
