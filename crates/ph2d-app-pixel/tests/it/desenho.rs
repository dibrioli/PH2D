//! **P1 pelo caminho real**: as ferramentas de desenho com o ponteiro e o teclado que a shell
//! entrega — a forma pré-visualiza sem tocar o documento, acaba como UM passo do desfazer, e as
//! opções do painel (simetria, ladrilho, balde) chegam ao gesto.

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_app_pixel::{PixelApp, Tool};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::pixel_panel::{PIXEL_MAX_BRUSH, PixelRequest};
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

use crate::edit::{PPM, WINDOW, at, born, canvas, spawn_root, value};

const CLEAR: [u8; 4] = [0, 0, 0, 0];
const WHITE: [u8; 4] = [255, 255, 255, 255];

fn press(app: &mut PixelApp, sim: &SimWorld, cam: &Camera2d, bits: u64, c: (i32, i32), m: Mods) {
    let p = at(app, sim, cam, bits, c.0, c.1);
    assert_eq!(
        input::press(app, &canvas(sim, cam, bits), p, Button::Primary, m),
        Press::Done
    );
}

fn moved(app: &mut PixelApp, sim: &SimWorld, cam: &Camera2d, bits: u64, c: (i32, i32), m: Mods) {
    let p = at(app, sim, cam, bits, c.0, c.1);
    assert!(input::moved(app, &canvas(sim, cam, bits), p, m));
}

fn click(app: &mut PixelApp, sim: &SimWorld, cam: &Camera2d, bits: u64, c: (i32, i32)) {
    press(app, sim, cam, bits, c, Mods::default());
    input::release(app);
}

fn drag(
    app: &mut PixelApp,
    sim: &SimWorld,
    cam: &Camera2d,
    bits: u64,
    a: (i32, i32),
    b: (i32, i32),
    m: Mods,
) {
    press(app, sim, cam, bits, a, m);
    moved(app, sim, cam, bits, b, m);
    assert!(input::release(app));
    // P6d: a forma fica viva até ao Enter.
    if app.live_shape().is_some() {
        key(app, sim, bits, KeyCode::Enter, false);
    }
}

fn undo(app: &mut PixelApp, sim: &SimWorld, bits: u64) -> bool {
    let key = input::key_of(sim.world(), bits).expect("marcador");
    app.undo(key)
}

fn key(app: &mut PixelApp, sim: &SimWorld, bits: u64, code: KeyCode, shift: bool) {
    assert!(input::key(app, sim.world(), Some(bits), code, false, shift).is_some());
}

#[test]
fn every_aseprite_shortcut_picks_its_tool_and_the_same_key_never_switches_to_a_filled_one() {
    let (mut app, sim, _cam, bits) = born();
    for (code, shift, want) in [
        (KeyCode::KeyB, true, Tool::Spray),
        (KeyCode::KeyG, false, Tool::Bucket),
        (KeyCode::KeyL, false, Tool::Line),
        (KeyCode::KeyL, true, Tool::Curve),
        (KeyCode::KeyU, false, Tool::RECTANGLE),
        (KeyCode::KeyU, false, Tool::RECTANGLE),
        (KeyCode::KeyU, true, Tool::ELLIPSE),
        (KeyCode::KeyU, true, Tool::ELLIPSE),
        (KeyCode::KeyD, false, Tool::Contour),
        (KeyCode::KeyD, true, Tool::Polygon),
        (KeyCode::KeyB, false, Tool::Pencil),
    ] {
        key(&mut app, &sim, bits, code, shift);
        assert_eq!(app.tool, want, "{code:?} shift={shift}");
    }
    assert_eq!(
        Tool::ALL.len(),
        16 + ph2d_tool_vector::shapes::SHAPES.len(),
        "10 de desenho + 5 de selecção + Transform (P6c: sem Move nem as cheias) + as formas do \
         Vector (P6e: o catálogo inteiro)"
    );
    assert_eq!(PIXEL_MAX_BRUSH, ph2d_pixel_core::brush::MAX_BRUSH);
}

#[test]
fn a_rectangle_previews_while_dragging_and_commits_one_undo_step() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::RECTANGLE;
    press(&mut app, &sim, &cam, bits, (4, 4), Mods::default());
    moved(&mut app, &sim, &cam, bits, (20, 20), Mods::default());
    moved(&mut app, &sim, &cam, bits, (10, 8), Mods::default());
    assert_eq!(
        value(&app, &sim, bits, 4, 4),
        CLEAR,
        "o documento não muda a meio do arrasto"
    );
    assert!(input::release(&mut app));
    assert_eq!(
        value(&app, &sim, bits, 4, 4),
        CLEAR,
        "P6d: largar deixa a forma viva — só o Enter a assenta"
    );
    key(&mut app, &sim, bits, KeyCode::Enter, false);
    for c in [(4, 4), (10, 4), (4, 8), (10, 8), (7, 4)] {
        assert_eq!(value(&app, &sim, bits, c.0, c.1), WHITE, "{c:?}");
    }
    assert_eq!(value(&app, &sim, bits, 7, 6), CLEAR, "oco");
    assert_eq!(
        value(&app, &sim, bits, 20, 20),
        CLEAR,
        "a pré-visualização maior não ficou"
    );
    assert!(undo(&mut app, &sim, bits));
    assert_eq!(value(&app, &sim, bits, 4, 4), CLEAR);
    assert!(!undo(&mut app, &sim, bits), "um passo só");
}

#[test]
fn shift_squares_the_shape_and_ctrl_draws_it_from_the_center() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::RECTANGLE;
    app.options.shape_fill = true;
    let shift = Mods {
        shift: true,
        ..Mods::default()
    };
    drag(&mut app, &sim, &cam, bits, (2, 2), (8, 4), shift);
    assert_eq!(value(&app, &sim, bits, 8, 8), WHITE, "Shift: quadrado de 7");
    assert_eq!(value(&app, &sim, bits, 9, 8), CLEAR);
    let ctrl = Mods {
        ctrl: true,
        ..Mods::default()
    };
    drag(&mut app, &sim, &cam, bits, (20, 20), (22, 21), ctrl);
    assert_eq!(value(&app, &sim, bits, 18, 19), WHITE, "Ctrl: do centro");
    assert_eq!(value(&app, &sim, bits, 17, 19), CLEAR);
}

#[test]
fn the_curve_takes_two_phases_and_escape_or_enter_end_the_pending_shapes() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::Curve;
    drag(
        &mut app,
        &sim,
        &cam,
        bits,
        (2, 20),
        (28, 20),
        Mods::default(),
    );
    assert!(app.stroking(), "depois da corda, o controlo");
    assert_eq!(value(&app, &sim, bits, 2, 20), CLEAR);
    moved(&mut app, &sim, &cam, bits, (15, 2), Mods::default());
    click(&mut app, &sim, &cam, bits, (15, 2));
    assert!(!app.stroking());
    assert_eq!(value(&app, &sim, bits, 2, 20), WHITE);
    assert_eq!(value(&app, &sim, bits, 15, 20), CLEAR, "a curva subiu");
    let top = (0..20).find(|&y| value(&app, &sim, bits, 15, y) == WHITE);
    assert!(
        top.is_some_and(|y| y < 15),
        "o meio da curva está em cima: {top:?}"
    );
    // Esc larga sem gravar; Enter acaba.
    drag(
        &mut app,
        &sim,
        &cam,
        bits,
        (2, 28),
        (28, 28),
        Mods::default(),
    );
    key(&mut app, &sim, bits, KeyCode::Escape, false);
    assert!(!app.stroking());
    assert_eq!(value(&app, &sim, bits, 2, 28), CLEAR);
    drag(
        &mut app,
        &sim,
        &cam,
        bits,
        (2, 28),
        (28, 28),
        Mods::default(),
    );
    key(&mut app, &sim, bits, KeyCode::Enter, false);
    assert_eq!(
        value(&app, &sim, bits, 15, 28),
        WHITE,
        "Enter acaba com o controlo no meio"
    );
}

#[test]
fn the_polygon_closes_on_its_first_corner_and_fills() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::Polygon;
    for c in [(4, 4), (24, 4), (14, 24)] {
        click(&mut app, &sim, &cam, bits, c);
        assert!(app.stroking());
    }
    assert_eq!(value(&app, &sim, bits, 14, 10), CLEAR, "ainda aberto");
    click(&mut app, &sim, &cam, bits, (4, 4));
    assert!(!app.stroking(), "o primeiro canto fecha");
    assert_eq!(value(&app, &sim, bits, 14, 10), WHITE, "cheio");
    assert_eq!(value(&app, &sim, bits, 4, 20), CLEAR, "fora");
    assert!(undo(&mut app, &sim, bits));
    assert!(!undo(&mut app, &sim, bits), "o polígono inteiro é um passo");
}

#[test]
fn switching_tool_finishes_a_pending_polygon() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::Polygon;
    for c in [(4, 4), (24, 4), (14, 24)] {
        click(&mut app, &sim, &cam, bits, c);
    }
    key(&mut app, &sim, bits, KeyCode::KeyB, false);
    assert!(!app.stroking());
    assert_eq!(value(&app, &sim, bits, 14, 10), WHITE, "acabou como estava");
}

#[test]
fn the_bucket_fills_inside_a_closed_rectangle_and_never_leaks() {
    let (mut app, sim, cam, bits) = born();
    app.tool = Tool::RECTANGLE;
    drag(
        &mut app,
        &sim,
        &cam,
        bits,
        (4, 4),
        (20, 20),
        Mods::default(),
    );
    app.tool = Tool::Bucket;
    app.fg = [255, 0, 0, 255];
    click(&mut app, &sim, &cam, bits, (10, 10));
    assert_eq!(value(&app, &sim, bits, 10, 10), [255, 0, 0, 255]);
    assert_eq!(value(&app, &sim, bits, 4, 4), WHITE, "a parede fica");
    assert_eq!(value(&app, &sim, bits, 25, 25), CLEAR, "não vazou");
    // Controlo: sem contíguo enche toda a cor, dentro e fora.
    app.options.contiguous = false;
    app.fg = [0, 0, 255, 255];
    click(&mut app, &sim, &cam, bits, (25, 25));
    assert_eq!(value(&app, &sim, bits, 0, 0), [0, 0, 255, 255]);
    assert_eq!(
        value(&app, &sim, bits, 10, 10),
        [255, 0, 0, 255],
        "outra cor fica"
    );
}

#[test]
fn the_symmetry_option_mirrors_and_tiled_mode_wraps() {
    let (mut app, sim, cam, bits) = born();
    app.options.sym_h = true;
    click(&mut app, &sim, &cam, bits, (3, 5));
    assert_eq!(value(&app, &sim, bits, 28, 5), WHITE, "32 − 1 − 3");
    app.options.sym_h = false;
    app.options.tile_x = true;
    app.options.size = 3;
    app.options.shape = ph2d_editor_core::pixel_panel::PixelBrushShape::Square;
    click(&mut app, &sim, &cam, bits, (0, 20));
    assert_eq!(value(&app, &sim, bits, 31, 20), WHITE, "deu a volta");
    assert_eq!(value(&app, &sim, bits, 0, 21), WHITE, "o pincel de 3");
}

#[test]
fn dragging_the_symmetry_axis_snaps_to_half_cells_and_never_paints() {
    let (mut app, sim, cam, bits) = born();
    app.options.sym_h = true;
    // O eixo no meio (x = 16,0): o ponto de ecrã da aresta entre as células 15 e 16.
    let key = input::key_of(sim.world(), bits).expect("marcador");
    let m = input::to_screen(
        sim.world(),
        bits,
        app.doc(key).expect("doc"),
        &cam,
        WINDOW,
        PPM,
    )
    .expect("afim");
    let pt = |x: f64, y: f64| {
        let p = m * ph2d_vector::Point::new(x, y);
        (p.x as f32, p.y as f32)
    };
    let c = canvas(&sim, &cam, bits);
    assert_eq!(
        input::press(
            &mut app,
            &c,
            pt(16.0, 10.5),
            Button::Primary,
            Mods::default()
        ),
        Press::Done
    );
    assert!(input::moved(&mut app, &c, pt(10.4, 10.5), Mods::default()));
    assert!(input::release(&mut app));
    assert_eq!(
        app.doc(key).expect("doc").axis2,
        Some((21, 32)),
        "10,5 em meias células"
    );
    for x in 0..32 {
        assert_eq!(value(&app, &sim, bits, x, 10), CLEAR, "o eixo não pinta");
    }
    click(&mut app, &sim, &cam, bits, (3, 0));
    assert_eq!(value(&app, &sim, bits, 17, 0), WHITE, "21 − 1 − 3");
}

#[test]
fn the_spray_keeps_spraying_while_held_and_the_panel_options_reach_the_family() {
    let (mut app, mut sim, mut cam, bits) = born();
    let mut o = app.options;
    o.spray_speed = 4;
    app.aplica(PixelRequest::Options(Box::new(o)));
    assert_eq!(app.options, o);
    app.tool = Tool::Spray;
    press(&mut app, &sim, &cam, bits, (16, 16), Mods::default());
    for _ in 0..30 {
        app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
    }
    assert!(input::release(&mut app));
    let painted = (0..32)
        .flat_map(|y| (0..32).map(move |x| (x, y)))
        .filter(|&(x, y)| value(&app, &sim, bits, x, y) == WHITE)
        .count();
    assert!(painted > 30, "parado, o jato continuou: {painted}");
}

#[test]
fn the_smoke_house_is_six_steps_and_its_walls_hold_the_bucket() {
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    assert!(ph2d_app_pixel::smoke::stage(
        &mut app, &mut sim, 2, spawn_root
    ));
    let (key, doc) = app.docs().next().expect("o sprite");
    assert_eq!((doc.sprite.width(), doc.sprite.height()), (64, 48));
    assert_eq!(app.tool, Tool::RECTANGLE);
    let bits = ph2d_app_pixel::mode::sprites(sim.world())
        .first()
        .map(|(b, _)| *b)
        .expect("o objecto");
    let mut cam = Camera2d::default();
    app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
    app.tool = Tool::Bucket;
    app.fg = [255, 0, 0, 255];
    click(&mut app, &sim, &cam, bits, (18, 30));
    assert_eq!(value(&app, &sim, bits, 18, 30), [255, 0, 0, 255]);
    assert_eq!(value(&app, &sim, bits, 5, 30), CLEAR, "a casa fecha");
    let mut steps = 0;
    while app.undo(key) {
        steps += 1;
    }
    assert_eq!(steps, 7, "seis formas + o balde");
}
