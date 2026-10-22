//! **A foto do preview vivo do pincel** (dono 22/10: «o preview dos brushes precisa refletir essas novas
//! features»): o painel REAL com o quadro final do preview publicado, desenhado na placa e gravado em PNG,
//! em três casos — o Digital com Grain Colors, o Pastel Side do Wet e a Aquarela com Grain Colors.
//! `PH2D_FOTO_DIR=<pasta> PH2D_FOTO_SUFIXO=<depois> cargo test -p ph2d-app-painter --features
//! panel-painter-layers foto_do_preview -- --ignored`.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::{PaintCtx, Panel, PanelHostInternal};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::{
    PainterLayersPanel, PainterLayersPanelState, set_current_brush, set_current_wet_preview,
};
use ph2d_tool_painter::{PaintMedia, PainterTool, pintar_preview};
use ph2d_ui_testkit::MockPanelHost;
use std::sync::Arc;

const W: u32 = 380;
const H: u32 = 640;

fn foto(gpu: &ph2d_gpu::GpuContext, painter: &PainterTool) -> Vec<u8> {
    foto_rolada(gpu, painter, 0.0, false)
}

/// [`foto`] com o corpo rolado `rolagem` px e o pino da preview `fixada` (o estado posto no store, como
/// o clique o deixa).
fn foto_rolada(
    gpu: &ph2d_gpu::GpuContext,
    painter: &PainterTool,
    rolagem: f32,
    fixada: bool,
) -> Vec<u8> {
    let mut pass =
        ph2d_render::VelloPass::new(gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (W, H)).unwrap();
    set_current_brush(Some(painter.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    host.set_panel_visible(PainterLayersPanel::ID, true);
    if let Some(InteractiveState::Toggle { on, .. }) = host
        .store_mut()
        .get_mut(ph2d_tool_painter::ids::PAINTER_BRUSH_PREVIEW_PIN)
    {
        *on = fixada;
    }
    host.store_mut()
        .set_panel_scroll(ph2d_editor_core::ids::PAINTER_LAYERS_PANEL, rolagem);
    let mut st = PainterLayersPanelState;
    let viewport = Rect::new(0.0, 0.0, W as f32, H as f32);
    let layout = HeroLayout::for_viewport(viewport);
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = ph2d_text::TextSystem::without_system_fonts();
    {
        let mut ctx = PaintCtx {
            host: &mut host,
            layout: &layout,
            slot: viewport,
            viewport,
            scene: &mut scene,
            text_system: &mut text,
        };
        let theme = ph2d_tokens::Theme::default();
        ph2d_editor_core::paint::fill_rounded_rect(
            ctx.scene,
            viewport,
            0.0,
            ph2d_editor_core::paint::resolve(ph2d_tokens::ColorToken::PanelBg, theme),
        );
        PainterLayersPanel::paint(&mut st, &mut ctx);
    }
    set_current_brush(None);
    pass.render_and_readback(gpu, scene.inner(), (W, H))
        .unwrap()
}

/// O Grain xadrez e a rampa verde → vermelho (as Grain Colors, `ae082ffb2e`).
fn grain_colors(t: &mut PainterTool) {
    t.set_brush_texture_kind(ph2d_painter_brush::TextureKind::Checker.to_u8());
    t.ramp_set_stop_color(0, [0, 255, 0, 255]);
    t.ramp_set_stop_color(1, [255, 0, 0, 255]);
    t.set_texture_ramp_enabled(true);
}

fn preset_de_fabrica(t: &mut PainterTool, nome: &str) {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == nome)
        .expect("o pincel de fábrica")
        .preset;
    assert!(t.preset_aplicar(p).is_ok());
}

/// Um caso da foto: o nome, o meio e como se arma o pincel.
type Caso<'a> = (&'a str, PaintMedia, &'a dyn Fn(&mut PainterTool));

#[test]
#[ignore = "foto do preview vivo — PH2D_FOTO_DIR=<pasta> e --ignored"]
fn foto_do_preview() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let sufixo = std::env::var("PH2D_FOTO_SUFIXO").unwrap_or_else(|_| "depois".into());
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    std::fs::create_dir_all(&dir).unwrap();
    let casos: [Caso; 3] = [
        ("digital_grain_colors", PaintMedia::Digital, &grain_colors),
        ("wet_pastel_side", PaintMedia::WetPaint, &|t| {
            preset_de_fabrica(t, "Pastel Side");
        }),
        ("watercolor_grain_colors", PaintMedia::Watercolor, &|t| {
            grain_colors(t);
            t.toggle_granulation_use_paper();
        }),
    ];
    for (nome, meio, arma) in casos {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
        t.set_paint_media(meio);
        t.set_brush_size_px(16.0);
        t.set_brush_color_srgb8([30, 60, 220]);
        arma(&mut t);
        let mut ultimo = None;
        assert!(pintar_preview(
            &t.wet_preview_cena(),
            &|| false,
            &mut |px, w, h| ultimo = Some((Arc::new(px.to_vec()), w, h))
        ));
        set_current_wet_preview(ultimo);
        let px = foto(&gpu, &t);
        set_current_wet_preview(None);
        image::save_buffer(
            format!("{dir}/preview_{nome}_{sufixo}.png"),
            &px,
            W,
            H,
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
    }
}

/// **A foto do pino da preview** (dono 22/10): o Wet Paint com o Pastel Side, desafixado sem rolar, desafixado
/// rolado (a preview sai por cima) e fixado rolado (a preview fica no topo e o corpo passa por baixo).
/// `PH2D_FOTO_DIR=<pasta> cargo test -p ph2d-app-painter --features panel-painter-layers foto_do_pino --
/// --ignored`.
#[test]
#[ignore = "foto do pino da preview — PH2D_FOTO_DIR=<pasta> e --ignored"]
fn foto_do_pino() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    std::fs::create_dir_all(&dir).unwrap();
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    preset_de_fabrica(&mut t, "Pastel Side");
    let mut ultimo = None;
    assert!(pintar_preview(
        &t.wet_preview_cena(),
        &|| false,
        &mut |px, w, h| ultimo = Some((Arc::new(px.to_vec()), w, h))
    ));
    set_current_wet_preview(ultimo);
    for (nome, rolagem, fixada) in [
        ("desafixado", 0.0, false),
        ("desafixado_rolado", 220.0, false),
        ("fixado_rolado", 220.0, true),
    ] {
        let px = foto_rolada(&gpu, &t, rolagem, fixada);
        image::save_buffer(
            format!("{dir}/pino_{nome}.png"),
            &px,
            W,
            H,
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
    }
    set_current_wet_preview(None);
}
