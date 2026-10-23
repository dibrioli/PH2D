//! **A foto do cartão Jitter** (dono 23/10: *«Jitter Unit deveria ir para o topo … todos sliders»*): o cartão
//! desenhado na placa (vello) com o pincel de fábrica e gravado em PNG, para o layout se OLHAR antes de ir ao dono —
//! a largura do dock e uma estreita. `PH2D_FOTO_DIR=<pasta> cargo test -p ph2d-panel-painter-layers foto_jitter --
//! --ignored`.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vector::VectorScene;

const H: u32 = 760;

/// O cartão sozinho, de `y = 8`, num fundo do tema, à largura `w`; devolve o RGBA.
fn foto(gpu: &ph2d_gpu::GpuContext, w: u32) -> Vec<u8> {
    let mut pass =
        ph2d_render::VelloPass::new(gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (w, H)).unwrap();
    let brush = ph2d_tool_painter::PainterTool::default().brush_settings();
    crate::state::set_current_brush(Some(brush));
    let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    let viewport = Rect::new(0.0, 0.0, w as f32, H as f32);
    let layout = HeroLayout::for_viewport(viewport);
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let theme = Theme::default();
    {
        let mut ctx = PaintCtx {
            host: &mut host,
            layout: &layout,
            slot: layout
                .slot_rects(ph2d_editor_core::screens::slot::SlotSet::ANY_DOCK)
                .get(ph2d_editor_core::screens::slot::Slot::RightTop),
            viewport,
            scene: &mut scene,
            text_system: &mut text,
        };
        ph2d_editor_core::paint::fill_rounded_rect(
            ctx.scene,
            viewport,
            0.0,
            ph2d_editor_core::paint::resolve(ph2d_tokens::ColorToken::Bg1, theme),
        );
        crate::paint_stroke::jitter_card::paint_jitter_card(
            &mut ctx,
            theme,
            12.0,
            w as f32 - 24.0,
            8.0,
            brush,
        );
    }
    crate::state::set_current_brush(None);
    pass.render_and_readback(gpu, scene.inner(), (w, H))
        .unwrap()
}

#[test]
#[ignore = "foto do cartão Jitter — PH2D_FOTO_DIR=<pasta> e --ignored"]
fn foto_jitter() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    std::fs::create_dir_all(&dir).unwrap();
    for (nome, w) in [("jitter_depois", 340), ("jitter_depois_estreito", 260)] {
        let px = foto(&gpu, w);
        image::save_buffer(
            format!("{dir}/{nome}.png"),
            &px,
            w,
            H,
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
    }
}
