//! **A foto do navegador de presets** (dono 22/10): o painel REAL, com a vista que esta ponte publica
//! sobre a fábrica, desenhado na placa (vello) e gravado em PNG — o cartão do topo (Paint Mode acima do
//! Preset) e o popover aberto com os títulos de categoria, para se OLHAR antes do smoke.
//! `PH2D_FOTO_DIR=<pasta> cargo test -p ph2d-app-painter --features panel-painter-layers foto_do_navegador
//! -- --ignored`.

use super::*;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::{PaintCtx, Panel, PanelHostInternal};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::{PainterLayersPanel, PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::ids;
use ph2d_ui_testkit::MockPanelHost;

const W: u32 = 380;
const H: u32 = 1000;

/// O painel inteiro num quadro `W × H`; `aberto` abre o chip do Preset rolado de `rolagem` px.
fn foto(gpu: &ph2d_gpu::GpuContext, painter: &PainterTool, aberto: Option<f32>) -> Vec<u8> {
    foto_com(gpu, painter, aberto, |_| {})
}

/// [`foto`] com `antes` a mexer no host antes da última pintura (um arrasto a meio).
fn foto_com(
    gpu: &ph2d_gpu::GpuContext,
    painter: &PainterTool,
    aberto: Option<f32>,
    mut antes: impl FnMut(&mut MockPanelHost),
) -> Vec<u8> {
    let mut pass =
        ph2d_render::VelloPass::new(gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (W, H)).unwrap();
    set_current_brush(Some(painter.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    host.set_panel_visible(PainterLayersPanel::ID, true);
    let mut st = PainterLayersPanelState;
    let viewport = Rect::new(0.0, 0.0, W as f32, H as f32);
    let layout = HeroLayout::for_viewport(viewport);
    let mut pinta = |host: &mut MockPanelHost| {
        let mut scene = ph2d_vector::VectorScene::new();
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        {
            let mut ctx = PaintCtx {
                host,
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
        scene
    };
    let _ = pinta(&mut host);
    if let Some(rolagem) = aberto {
        if let Some(InteractiveState::Dropdown { open, .. }) =
            host.store_mut().get_mut(ids::PAINTER_WETPAINT_PRESET)
        {
            *open = true;
        }
        host.store_mut()
            .set_panel_scroll(ids::PAINTER_WETPAINT_PRESET, rolagem);
        let _ = pinta(&mut host);
    }
    antes(&mut host);
    let scene = pinta(&mut host);
    set_current_brush(None);
    pass.render_and_readback(gpu, scene.inner(), (W, H))
        .unwrap()
}

#[test]
#[ignore = "foto do navegador de presets — PH2D_FOTO_DIR=<pasta> e --ignored"]
fn foto_do_navegador() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    std::fs::create_dir_all(&dir).unwrap();
    let uti = std::env::temp_dir().join(format!("ph2d_foto_navegador_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&uti);
    std::fs::create_dir_all(&uti).unwrap();
    for (meio, nome) in [
        (PaintMedia::Digital, "digital"),
        (PaintMedia::WetPaint, "wetpaint"),
    ] {
        let mut t = PainterTool::default();
        t.set_paint_media(meio);
        let mut toasts = ToastQueue::default();
        let p = Presets::abrir_em(pasta_de_fabrica(), Some(uti.clone()), &mut t, &mut toasts);
        set_current_wet_presets(Some(p.vista.clone()));
        for (sufixo, aberto) in [
            ("1_topo", None),
            ("2_popover", Some(0.0)),
            ("3_popover_rolado", Some(600.0)),
        ] {
            let px = foto(&gpu, &t, aberto);
            image::save_buffer(
                format!("{dir}/navegador_{nome}_{sufixo}.png"),
                &px,
                W,
                H,
                image::ExtendedColorType::Rgba8,
            )
            .unwrap();
        }
    }
    set_current_wet_presets(None);
    let _ = std::fs::remove_dir_all(&uti);
}

/// **A foto da biblioteca** (23/10): a fileira «Preset library» sobre um pincel do artista (Import · Export ·
/// Delete · Reset positions), o mesmo Delete armado, e o popover com o pincel do artista ARRASTADO para
/// cima do Wet Round, a barra da queda a meio do gesto.
/// `PH2D_FOTO_DIR=<pasta> cargo test -p ph2d-app-painter --features panel-painter-layers foto_da_biblioteca
/// -- --ignored`.
#[test]
#[ignore = "foto da biblioteca de presets — PH2D_FOTO_DIR=<pasta> e --ignored"]
fn foto_da_biblioteca() {
    use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    std::fs::create_dir_all(&dir).unwrap();
    let uti = std::env::temp_dir().join(format!("ph2d_foto_biblioteca_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&uti);
    std::fs::create_dir_all(&uti).unwrap();
    let round = "WetPainter/Watercolor/Fluid/Wet Round.ph2d-brush.png";
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    let mut toasts = ToastQueue::default();
    let mut p = Presets::abrir_em(pasta_de_fabrica(), Some(uti.clone()), &mut t, &mut toasts);
    p.atender(&mut t, PedidoPreset::Escolher(round.into()), &mut toasts);
    p.atender(&mut t, PedidoPreset::GuardarComoNovo, &mut toasts);
    let meu = p.vista.ativo.clone().unwrap();
    p.atender(
        &mut t,
        PedidoPreset::Mover {
            arrastado: meu.clone(),
            alvo: "WetPainter/Watercolor/Fluid/Wet Drip.ph2d-brush.png".into(),
            depois: false,
        },
        &mut toasts,
    );
    set_current_wet_presets(Some(p.vista.clone()));
    let grava = |nome: &str, px: Vec<u8>| {
        image::save_buffer(
            format!("{dir}/biblioteca_{nome}.png"),
            &px,
            W,
            H,
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
    };
    grava("1_fileira", foto(&gpu, &t, None));
    grava(
        "2_delete_armado",
        foto_com(&gpu, &t, None, |host| {
            let r = host
                .registos_da_ultima_pintura()
                .into_iter()
                .find(|(id, _)| *id == ids::PAINTER_WETPAINT_PRESET_DELETE)
                .map(|(_, r)| r)
                .expect("o Delete do pincel do artista");
            for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
                let mut st = PainterLayersPanelState;
                host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
            }
            let _ = host.drained_actions();
        }),
    );
    grava(
        "3_arrasto",
        foto_com(&gpu, &t, Some(300.0), |host| {
            let regs = host.registos_da_ultima_pintura();
            let rect = |rel: &str| {
                regs.iter()
                    .find(|(id, _)| *id == ids::painter_wetpaint_preset_option_id(2, rel))
                    .map(|(_, r)| *r)
                    .unwrap_or_else(|| panic!("{rel} visível no popover rolado"))
            };
            let (de, para) = (rect(&meu), rect(round));
            for (kind, y) in [
                (PointerKind::Down, de.y + de.h * 0.5),
                (PointerKind::Move, para.y + para.h * 0.3),
            ] {
                host.dispatch_pointer_event(PointerEvent {
                    x: de.x + de.w * 0.5,
                    y,
                    pressure: 1.0,
                    kind,
                    source: PointerSource::Mouse,
                    button: PointerButton::Primary,
                    timestamp_ns: 0,
                });
            }
        }),
    );
    set_current_wet_presets(None);
    let _ = std::fs::remove_dir_all(&uti);
}
