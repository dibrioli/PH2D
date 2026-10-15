//! **A foto da secção Shape Color** (prompt 38, smoke do dono 15/10: *«o layout ficou todo bagunçado»*):
//! a secção desenhada na placa (vello) para cada estado das camadas e gravada em PNG, para o layout se
//! OLHAR antes de ir ao dono. `PH2D_FOTO_DIR=<pasta> cargo test -p ph2d-panel-painter-layers
//! foto_shape_color -- --ignored`.

use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_tool_painter::{BrushSettings, PANEL_RAMP_STOPS, ids};
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vector::VectorScene;

const W: u32 = 340;
const H: u32 = 900;

fn stops(a: [f32; 3], b: [f32; 3]) -> [[f32; 6]; PANEL_RAMP_STOPS] {
    let mut s = [[0.0f32; 6]; PANEL_RAMP_STOPS];
    s[0] = [0.0, a[0], a[1], a[2], 1.0, 0.0];
    s[1] = [1.0, b[0], b[1], b[2], 1.0, 1.0];
    s
}

fn estado(textura: bool, queda: bool, queda_primeiro: bool, bw: bool) -> BrushSettings {
    BrushSettings {
        shape_color_ramp_enabled: true,
        shape_color_ramp_bw: bw,
        shape_color_ramp_stops: stops([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        shape_color_ramp_stop_count: 2,
        shape_falloff_ramp_stops: stops([0.0, 0.8, 0.0], [1.0, 1.0, 0.0]),
        shape_falloff_ramp_stop_count: 2,
        shape_texture_color: textura,
        shape_falloff_color: queda,
        shape_falloff_first: queda_primeiro,
        ..crate::paint_brush::FALLBACK_BRUSH
    }
}

/// A secção sozinha, de `y = 8`, num fundo do tema; devolve o RGBA.
fn foto(gpu: &ph2d_gpu::GpuContext, brush: BrushSettings) -> Vec<u8> {
    let mut pass =
        ph2d_render::VelloPass::new(gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (W, H)).unwrap();
    crate::state::set_current_brush(Some(brush));
    let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    host.store_mut()
        .set_collapsed(ids::PAINTER_SHAPE_RAMP_SECTION, false);
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
        crate::paint_shape_ramp::paint_shape_ramp_section(
            &mut ctx,
            theme,
            12.0,
            W as f32 - 24.0,
            8.0,
            brush,
        );
    }
    crate::state::set_current_brush(None);
    pass.render_and_readback(gpu, scene.inner(), (W, H))
        .unwrap()
}

#[test]
#[ignore = "foto da secção Shape Color — PH2D_FOTO_DIR=<pasta> e --ignored"]
fn foto_shape_color() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    std::fs::create_dir_all(&dir).unwrap();
    for (nome, b) in [
        ("1_textura", estado(true, false, false, false)),
        ("2_ambas", estado(true, true, false, false)),
        ("3_ambas_queda_em_cima", estado(true, true, true, false)),
        ("4_so_queda", estado(false, true, false, false)),
        ("5_bw", estado(true, true, false, true)),
        (
            "6_sem_textura",
            BrushSettings {
                inercias: ph2d_tool_painter::PainterTool::default()
                    .brush_settings()
                    .inercias,
                ..estado(true, true, false, false)
            },
        ),
    ] {
        let px = foto(&gpu, b);
        image::save_buffer(
            format!("{dir}/{nome}.png"),
            &px,
            W,
            H,
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
    }
}
