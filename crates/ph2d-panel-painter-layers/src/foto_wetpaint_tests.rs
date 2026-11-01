//! **A foto do painel Wet Paint** (E10-B2, F3 §9): os volumes do topo (Size · Dilution · Volume µm) e o cartão Wet
//! Paint (Medium · Pickup · Hardness · … · Leveling · Body) desenhados na placa com o Pincel de fábrica, em PNG.
//! `PH2D_FOTO_DIR=<pasta> cargo test -p ph2d-panel-painter-layers foto_wetpaint -- --ignored`.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vector::VectorScene;

const W: u32 = 340;
const H: u32 = 1700;

fn foto(gpu: &ph2d_gpu::GpuContext) -> Vec<u8> {
    let mut pass =
        ph2d_render::VelloPass::new(gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (W, H)).unwrap();
    let mut t = ph2d_tool_painter::PainterTool::default();
    t.set_paint_media(ph2d_tool_painter::PaintMedia::WetPaint);
    let brush = t.brush_settings();
    crate::state::set_current_brush(Some(brush));
    let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    let viewport = Rect::new(0.0, 0.0, W as f32, H as f32);
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
        let (x, w) = (12.0, W as f32 - 24.0);
        let y = crate::paint_wetpaint_presets::paint_wetpaint_presets(
            &mut ctx, theme, x, w, 8.0, brush,
        );
        crate::paint_wetpaint::paint_wetpaint_section(&mut ctx, theme, x, w, y, brush);
    }
    crate::state::set_current_brush(None);
    pass.render_and_readback(gpu, scene.inner(), (W, H))
        .unwrap()
}

#[test]
#[ignore = "foto do painel Wet Paint — PH2D_FOTO_DIR=<pasta> e --ignored"]
fn foto_wetpaint() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    std::fs::create_dir_all(&dir).unwrap();
    let px = foto(&gpu);
    image::save_buffer(
        format!("{dir}/painel_wet_paint.png"),
        &px,
        W,
        H,
        image::ExtendedColorType::Rgba8,
    )
    .unwrap();
}
