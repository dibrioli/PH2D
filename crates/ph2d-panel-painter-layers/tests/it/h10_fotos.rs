//! A foto de cada estado que a cena do app não semeia: `PH2D_FOTO_DIR=<pasta>`, `--ignored`, por nome `h10_foto`.

use super::h1_os_cartoes::{Mesa, pincel};
use super::h5_os_cartoes::com_rampa;
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::{PaintCtx, Panel, PanelHostInternal};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::screens::slot::SlotSet;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::state::set_current_layers;
use ph2d_panel_painter_layers::{FALLBACK_BRUSH, PainterLayersPanel};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{BrushSettings, Falloff, LayerStack, PaintMedia, TextureKind};
use ph2d_vector::VectorScene;

const JANELA: Rect = Rect::new(0.0, 0.0, 1600.0, 2400.0);

pub(super) fn foto(gpu: &ph2d_gpu::GpuContext, dir: &str, nome: &str, m: &mut Mesa) {
    let (w, h) = (JANELA.w as u32, JANELA.h as u32);
    let mut pass =
        ph2d_render::VelloPass::new(gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (w, h)).unwrap();
    let layout = HeroLayout::for_viewport(JANELA);
    let slot = layout
        .slot_rects(SlotSet::of(PainterLayersPanel::DEFAULT_SLOT))
        .get(PainterLayersPanel::DEFAULT_SLOT);
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    let mesa = &mut *m;
    let mut ctx = PaintCtx {
        host: &mut mesa.host,
        layout: &layout,
        slot,
        viewport: JANELA,
        scene: &mut scene,
        text_system: &mut text,
    };
    ph2d_editor_core::paint::fill_rounded_rect(
        ctx.scene,
        JANELA,
        0.0,
        ph2d_editor_core::paint::resolve(ColorToken::Bg1, Theme::default()),
    );
    ctx.host.hit_index_mut().clear_for_frame();
    PainterLayersPanel::paint(&mut mesa.st, &mut ctx);
    let px = pass
        .render_and_readback(gpu, scene.inner(), (w, h))
        .unwrap();
    // Só a coluna do painel: o resto é a janela de teste, vazia.
    let (x0, cw) = (slot.x.max(0.0) as u32, slot.w as u32);
    let ch = h.min((slot.y + slot.h) as u32);
    let mut corte = Vec::with_capacity((cw * ch * 4) as usize);
    for y in 0..ch {
        let i = ((y * w + x0) * 4) as usize;
        corte.extend_from_slice(&px[i..i + (cw * 4) as usize]);
    }
    image::save_buffer(
        format!("{dir}/{nome}.png"),
        &corte,
        cw,
        ch,
        image::ExtendedColorType::Rgba8,
    )
    .unwrap();
}

fn digital() -> BrushSettings {
    pincel(PaintMedia::Digital, "brush").0
}

fn estados() -> Vec<(&'static str, BrushSettings, Vec<NodeId>)> {
    let mut grao = digital();
    grao.texture_kind = TextureKind::COUNT - 2;
    grao.texture_ramp_enabled = true;
    let mut curva = digital();
    curva.falloff = Falloff::Custom.to_u8();
    let mut camadas = digital();
    camadas.shape_kind = TextureKind::Image.to_u8();
    camadas.shape_has_image = true;
    camadas.shape_layer_count = 2;
    camadas.shape_per_layer_color = true;
    camadas.shape_layer_color_on = [true; 16];
    let mut mascara = digital();
    mascara.is_mask = true;
    let mut escorre = pincel(PaintMedia::WetPaint, "brush").0;
    escorre.wet_tilt_on = true;
    let forma = p::PAINTER_SHAPE_SECTION;
    vec![
        (
            "h10_grao_rampa",
            grao,
            vec![
                p::PAINTER_BRUSH_TEXTURE_SECTION,
                p::PAINTER_BRUSH_COLOR_RAMP_SECTION,
            ],
        ),
        ("h10_forma_curva", curva, vec![forma]),
        ("h10_forma_camadas", camadas, vec![forma]),
        ("h10_tom", com_rampa(), vec![p::PAINTER_SHAPE_RAMP_SECTION]),
        ("h10_mascara", mascara, vec![p::PAINTER_MASK_SECTION]),
        (
            "h10_clone",
            pincel(PaintMedia::Digital, "clone").0,
            vec![p::PAINTER_CARD_TOPO, p::PAINTER_CARD_BASICOS],
        ),
        (
            "h10_topo_sync",
            digital(),
            vec![p::PAINTER_CARD_TOPO, p::PAINTER_CARD_BASICOS],
        ),
        ("h10_wet_tilt", escorre, vec![p::PAINTER_WETPAINT_SECTION]),
    ]
}

#[test]
#[ignore = "fotos: PH2D_FOTO_DIR=<pasta> e --ignored"]
fn h10_foto() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    for (nome, b, abertos) in estados() {
        let mut m = Mesa::na_janela(true, b, false, JANELA);
        m.so_abertos(&abertos);
        m.assenta();
        foto(&gpu, &dir, nome, &mut m);
    }
    let mut s = LayerStack::new();
    let _ = s.add_raster("Base", 8, 8);
    let _ = s.add_raster("Middle", 8, 8);
    let _ = s.add_raster("Top", 8, 8);
    let mut m = Mesa::na_janela(true, FALLBACK_BRUSH, true, JANELA);
    set_current_layers(Some(s));
    m.assenta();
    foto(&gpu, &dir, "h10_pilha", &mut m);
    set_current_layers(None);
}
