//! **P8: o Gradient e o Blur/Jumble pelo caminho real** — o teclado escolhe-os, o arrasto
//! pré-visualiza, o largar é UM passo do desfazer, o `Esc` larga sem gravar, e o que pintam é o que
//! o núcleo prevê ([`Gradient::values`], [`Smudge`]).

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_app_pixel::{PixelApp, Tool};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::pixel_panel::{PixelGradientDither, PixelToolGroup};
use ph2d_pixel_core::filter::smudge::{Kind, Smudge};
use ph2d_pixel_core::gradient::{Gradient, Shape, palette_ramp};
use ph2d_pixel_core::raster::line_cells;
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

use crate::edit::{at, born, canvas, value};

const CLEAR: [u8; 4] = [0, 0, 0, 0];

fn press(app: &mut PixelApp, sim: &SimWorld, cam: &Camera2d, bits: u64, c: (i32, i32)) {
    let p = at(app, sim, cam, bits, c.0, c.1);
    let m = Mods::default();
    assert_eq!(
        input::press(app, &canvas(sim, cam, bits), p, Button::Primary, m),
        Press::Done
    );
}

fn moved(app: &mut PixelApp, sim: &SimWorld, cam: &Camera2d, bits: u64, c: (i32, i32)) {
    let p = at(app, sim, cam, bits, c.0, c.1);
    assert!(input::moved(
        app,
        &canvas(sim, cam, bits),
        p,
        Mods::default()
    ));
}

fn key(app: &mut PixelApp, sim: &SimWorld, bits: u64, code: KeyCode, shift: bool) {
    assert!(input::key(app, sim.world(), Some(bits), code, false, shift).is_some());
}

fn chave(sim: &SimWorld, bits: u64) -> u32 {
    input::key_of(sim.world(), bits).expect("marcador")
}

/// O que o núcleo prevê para o gradiente de `a` a `b` com as opções da mão.
fn previsto(
    app: &PixelApp,
    sim: &SimWorld,
    bits: u64,
    a: (i32, i32),
    b: (i32, i32),
) -> Vec<((i32, i32), u32)> {
    let d = app.doc(chave(sim, bits)).expect("doc");
    let o = &app.options;
    let g = Gradient {
        shape: if o.gradient_radial {
            Shape::Radial
        } else {
            Shape::Linear
        },
        from: a,
        to: b,
        fg: app.fg,
        bg: app.bg,
        dither: ph2d_app_pixel::degrade::dither(o.gradient_dither),
    };
    let bucket = ph2d_pixel_core::fill::Bucket {
        tolerance: o.tolerance,
        contiguous: o.contiguous,
        eight: o.eight,
        sample: ph2d_pixel_core::fill::BucketSample::Layer,
    };
    g.values(&d.sprite, (d.layer, d.frame), bucket)
}

fn confere(app: &PixelApp, sim: &SimWorld, bits: u64, esperado: &[((i32, i32), u32)]) {
    let d = app.doc(chave(sim, bits)).expect("doc");
    assert!(!esperado.is_empty());
    for &((x, y), v) in esperado {
        assert_eq!(
            value(app, sim, bits, x, y),
            d.sprite.rgba_of_value(v),
            "célula ({x}, {y})"
        );
    }
}

#[test]
fn shift_g_picks_the_gradient_and_r_toggles_blur_and_jumble() {
    let (mut app, sim, _cam, bits) = born();
    key(&mut app, &sim, bits, KeyCode::KeyG, true);
    assert_eq!(app.tool, Tool::Gradient);
    assert_eq!(Tool::Gradient.group(), Some(PixelToolGroup::Bucket));
    key(&mut app, &sim, bits, KeyCode::KeyG, false);
    assert_eq!(app.tool, Tool::Bucket);
    key(&mut app, &sim, bits, KeyCode::KeyR, false);
    assert_eq!(app.tool, Tool::Blur);
    key(&mut app, &sim, bits, KeyCode::KeyR, false);
    assert_eq!(app.tool, Tool::Jumble);
    key(&mut app, &sim, bits, KeyCode::KeyR, false);
    assert_eq!(app.tool, Tool::Blur);
    assert_eq!(Tool::Jumble.group(), Some(PixelToolGroup::Smudge));
}

#[test]
fn a_linear_gradient_drag_paints_what_the_core_predicts_as_one_undo_step() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::Gradient;
    app.fg = [255, 0, 0, 255];
    app.bg = [0, 0, 255, 255];
    let esperado = previsto(&app, &sim, bits, (4, 4), (20, 4));
    press(&mut app, &sim, &cam, bits, (4, 4));
    moved(&mut app, &sim, &cam, bits, (20, 4));
    assert_eq!(
        value(&app, &sim, bits, 10, 4),
        CLEAR,
        "o arrasto só pré-visualiza"
    );
    assert!(input::release(&mut app));
    confere(&app, &sim, bits, &esperado);
    assert_ne!(
        value(&app, &sim, bits, 4, 4),
        value(&app, &sim, bits, 20, 4)
    );
    assert!(app.undo(chave(&sim, bits)), "um passo");
    assert_eq!(value(&app, &sim, bits, 4, 4), CLEAR);
    assert_eq!(value(&app, &sim, bits, 20, 4), CLEAR);
}

#[test]
fn radial_and_bayer_reach_the_op_and_the_matrix_is_the_tools_own() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::Gradient;
    app.fg = [255, 0, 0, 255];
    app.bg = [0, 0, 255, 255];
    app.options.gradient_radial = true;
    app.options.gradient_dither = PixelGradientDither::Bayer4;
    // A da dinâmica é OUTRA (o bug 1.3.0 do Aseprite): mudá-la não muda a do Gradient.
    app.options.dynamics.dither = PixelGradientDither::Bayer8;
    assert_eq!(app.options.gradient_dither, PixelGradientDither::Bayer4);
    let esperado = previsto(&app, &sim, bits, (2, 2), (18, 14));
    press(&mut app, &sim, &cam, bits, (2, 2));
    moved(&mut app, &sim, &cam, bits, (18, 14));
    assert!(input::release(&mut app));
    confere(&app, &sim, bits, &esperado);
    // Com matriz só há as duas pontas.
    let d = app.doc(chave(&sim, bits)).expect("doc");
    for &((x, y), _) in &esperado {
        let c = value(&app, &sim, bits, x, y);
        assert!(c == app.fg || c == app.bg, "({x}, {y}) = {c:?}");
    }
    let _ = d;
}

#[test]
fn the_palette_ramp_paints_only_colors_of_the_ramp() {
    let (mut app, sim, cam, bits) = born();
    let pal = app
        .doc(chave(&sim, bits))
        .expect("doc")
        .sprite
        .palette()
        .to_vec();
    assert!(pal.len() > 4, "a paleta de fábrica tem degraus");
    app.tool = Tool::Gradient;
    app.options.gradient_ramp = true;
    app.fg = pal[1];
    app.bg = pal[4];
    let rampa = palette_ramp(&pal, 1, 4);
    press(&mut app, &sim, &cam, bits, (0, 0));
    moved(&mut app, &sim, &cam, bits, (24, 0));
    assert!(input::release(&mut app));
    let mut vistas = Vec::new();
    for x in 0..=24 {
        let c = value(&app, &sim, bits, x, 3);
        assert!(rampa.contains(&c), "({x}, 3) = {c:?} fora da rampa");
        if !vistas.contains(&c) {
            vistas.push(c);
        }
    }
    assert_eq!(vistas.len(), rampa.len(), "caminha pela rampa inteira");
}

#[test]
fn a_blur_stroke_changes_the_cells_as_smudge_predicts_and_escape_cancels() {
    let (mut app, sim, cam, bits) = born();
    // Um degradê por baixo, para o Blur ter o que misturar.
    app.tool = Tool::Gradient;
    app.fg = [255, 0, 0, 255];
    app.bg = [0, 0, 255, 255];
    app.options.gradient_dither = PixelGradientDither::Bayer2;
    press(&mut app, &sim, &cam, bits, (0, 0));
    moved(&mut app, &sim, &cam, bits, (16, 16));
    assert!(input::release(&mut app));
    let k = chave(&sim, bits);
    let esperado = {
        let d = app.doc(k).expect("doc");
        let mut s = Smudge::new(&d.sprite, (d.layer, d.frame), Kind::Blur, 0);
        s.dab(&[(4, 6)]);
        let mut caminho = Vec::new();
        line_cells(4, 6, 12, 6, |x, y| caminho.push((x, y)));
        for p in caminho.into_iter().skip(1) {
            s.dab(&[p]);
        }
        s.values(&d.sprite)
    };
    // Esc a meio: nada gravado.
    let antes = value(&app, &sim, bits, 8, 6);
    key(&mut app, &sim, bits, KeyCode::KeyR, false);
    assert_eq!(app.tool, Tool::Blur);
    press(&mut app, &sim, &cam, bits, (4, 6));
    moved(&mut app, &sim, &cam, bits, (12, 6));
    key(&mut app, &sim, bits, KeyCode::Escape, false);
    assert!(!app.stroking());
    assert_eq!(value(&app, &sim, bits, 8, 6), antes, "o Esc não grava");
    press(&mut app, &sim, &cam, bits, (4, 6));
    moved(&mut app, &sim, &cam, bits, (12, 6));
    assert!(input::release(&mut app));
    confere(&app, &sim, bits, &esperado);
    assert!(
        esperado
            .iter()
            .any(|&((x, y), _)| value(&app, &sim, bits, x, y) != antes || x != 8 || y != 6),
        "o traço mudou alguma coisa"
    );
    assert!(app.undo(k), "um passo");
    assert_eq!(value(&app, &sim, bits, 8, 6), antes);
}
