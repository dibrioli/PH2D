//! **P13 U-Pintura pelo caminho real** — o ink Dither escolhido no painel chega ao traço (o padrão
//! preso ao canvas), `R` três vezes põe o Push na mão e o arrasto empurra a cor, e o *Color Mode*
//! com «Keep ramps» converte pela métrica das rampas (`P13_plano.md` §6, §7).

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_app_pixel::{PixelApp, Tool, mode, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{
    NewSpriteMode, PixelColorMetric, PixelDither, PixelGradientDither, PixelInk, PixelRequest,
};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_pixel_core::dither::{Dither, PincelDither};
use ph2d_pixel_core::{ColorMode, Op, compose_rgba};
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

use crate::comum::{PPM, WINDOW, at, born, canvas, spawn_root, value};

fn arrasta(
    app: &mut PixelApp,
    sim: &SimWorld,
    cam: &Camera2d,
    bits: u64,
    de: (i32, i32),
    a: (i32, i32),
) {
    let c = canvas(sim, cam, bits);
    let p = at(app, sim, cam, bits, de.0, de.1);
    assert_eq!(
        input::press(app, &c, p, Button::Primary, Mods::default()),
        Press::Done
    );
    let q = at(app, sim, cam, bits, a.0, a.1);
    assert!(input::moved(app, &c, q, Mods::default()));
    assert!(input::release(app));
}

/// ⭐ Dither + Level pelo pedido do painel ⇒ o lápis pinta o padrão do canvas: cada célula do
/// traço é a frente onde `PincelDither::primaria` diz, o fundo nas outras.
#[test]
fn o_ink_dither_do_painel_pinta_o_padrao_preso_ao_canvas() {
    let (mut app, sim, cam, bits) = born();
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.mode.enter(bits, ObjectMode::Draw);
    let mut toasts = ToastQueue::new();
    app.tool = Tool::Pencil;
    app.fg = [255, 0, 0, 255];
    app.bg = [0, 0, 255, 255];
    let mut o = app.options;
    o.ink = PixelInk::Dither;
    o.dither_level = 32;
    o.dither_matrix = PixelGradientDither::Bayer2;
    app.pede(PixelRequest::Options(Box::new(o)));
    let _ = panel::frame(&mut app, sim.world(), &mut hero, &mut toasts);
    assert_eq!(app.options.ink, PixelInk::Dither);
    arrasta(&mut app, &sim, &cam, bits, (2, 5), (9, 5));
    let d = PincelDither {
        nivel: 32,
        matriz: Dither::Bayer2,
    };
    let (mut frente, mut fundo) = (0, 0);
    for x in 2..=9 {
        let quer = if d.primaria(x, 5) { app.fg } else { app.bg };
        assert_eq!(value(&app, &sim, bits, x, 5), quer, "célula ({x}, 5)");
        if quer == app.fg {
            frente += 1;
        } else {
            fundo += 1;
        }
    }
    assert!(frente > 0 && fundo > 0, "o padrão tem as duas cores");
}

/// ⭐ `R` alterna Blur → Jumble → Push → Blur; o Push arrasta a cor da célula anterior do traço.
#[test]
fn r_tres_vezes_e_o_push_e_o_arrasto_empurra_a_cor() {
    let (mut app, sim, cam, bits) = born();
    let red = [255, 0, 0, 255];
    app.tool = Tool::Pencil;
    app.fg = red;
    arrasta(&mut app, &sim, &cam, bits, (4, 4), (4, 4));
    assert_eq!(value(&app, &sim, bits, 4, 4), red);
    let antes = value(&app, &sim, bits, 5, 4);
    assert_ne!(antes, red);
    for quer in [
        Tool::Blur,
        Tool::Jumble,
        Tool::Push,
        Tool::Blur,
        Tool::Jumble,
        Tool::Push,
    ] {
        assert!(
            input::key(
                &mut app,
                sim.world(),
                Some(bits),
                KeyCode::KeyR,
                false,
                false
            )
            .is_some()
        );
        assert_eq!(app.tool, quer);
    }
    arrasta(&mut app, &sim, &cam, bits, (4, 4), (6, 4));
    assert_eq!(
        value(&app, &sim, bits, 5, 4),
        red,
        "a cor andou no sentido do traço"
    );
}

/// ⭐ O diálogo *Color Mode* com «Keep ramps» converte pela `PreservaRampas` (a mesma função do
/// núcleo das rampas, byte a byte), e não é a conversão Exact RGB de sempre.
#[test]
fn o_color_mode_com_keep_ramps_converte_pela_lei_das_rampas() {
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    assert!(ph2d_app_pixel::smoke::stage(
        &mut app, &mut sim, 5, spawn_root
    ));
    let (bits, key) = mode::sprites(sim.world())[0];
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.mode.enter(bits, ObjectMode::Draw);
    let mut toasts = ToastQueue::new();
    let mut cam = Camera2d::default();
    app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
    let _ = panel::frame(&mut app, sim.world(), &mut hero, &mut toasts);
    let antes = app.doc(key).expect("doc").sprite.clone();
    assert_eq!(antes.mode(), ColorMode::Rgba);
    let imagem = |s: &ph2d_pixel_core::Sprite| {
        let mut out = Vec::new();
        compose_rgba(s, s.frames()[0].id, None, &mut out);
        out
    };
    // Sem dither, cinza por omissão: o `Convert` que o diálogo abaixo pede.
    let opt = ph2d_pixel_core::convert::Convert::default();
    let com = |w: Option<ph2d_pixel_core::Whole>| {
        let mut s = antes.clone();
        ph2d_pixel_core::apply(&mut s, Op::Whole(Box::new(w.expect("converte")))).expect("aplica");
        imagem(&s)
    };
    let rampas = com(ph2d_pixel_rampa::converte_preservando_rampas(&antes, opt));
    let rgb = com(ph2d_pixel_core::convert::convert(
        &antes,
        ColorMode::Indexed,
        opt,
    ));
    assert_ne!(rampas, rgb, "a foto distingue as duas métricas");
    hero.store.open_color_mode_dialog(NewSpriteMode::Rgba);
    hero.store.set_color_mode_target(NewSpriteMode::Indexed);
    hero.store.set_color_mode_dither(PixelDither::None);
    hero.store
        .set_color_mode_metric(PixelColorMetric::KeepRamps);
    hero.store.request_color_mode();
    let _ = panel::frame(&mut app, sim.world(), &mut hero, &mut toasts);
    let depois = &app.doc(key).expect("doc").sprite;
    assert_eq!(depois.mode(), ColorMode::Indexed);
    assert!(
        imagem(depois) == rampas,
        "o OK com Keep ramps é a PreservaRampas"
    );
}
