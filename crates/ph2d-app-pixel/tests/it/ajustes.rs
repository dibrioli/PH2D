//! **P6f pelo caminho real**: a forma viva até ao fim — a etiqueta do Vector e a caixa só-contorno
//! no arrasto de uma alça, a barra da moldura a mostrá-la e a editá-la, gravar assenta-a, e o
//! `Ctrl+Z` desfaz o último ajuste dela (um arrasto = um passo, a lei do Vector) antes de a largar.

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, KeyAnswer};
use ph2d_app_pixel::smoke_ajustes::{POINTS, SETTLED, STAR, points_handle};
use ph2d_app_pixel::{Tool, mode, panel, persist, viva};
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{
    PixelEditAction, PixelPanelInfo, PixelRequest, PixelShape, PixelTransformField, field_name,
};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_tool_vector::shape_handles::handles_of;
use ph2d_tool_vector::shape_handles_face::Glyph;
use ph2d_tool_vector::shapes::desc;
use winit::keyboard::KeyCode;

use crate::edit::canvas;
use crate::viva::Mesa;

const CLEAR: [u8; 4] = [0, 0, 0, 0];
const STAR_CENTER: (i32, i32) = ((STAR.0.0 + STAR.1.0) / 2, (STAR.0.1 + STAR.1.1) / 2);
const RECT_CELL: (i32, i32) = (
    (SETTLED.0.0 + SETTLED.1.0) / 2,
    (SETTLED.0.1 + SETTLED.1.1) / 2,
);

/// O campo das pontas da estrela e o rótulo dele como o painel o pinta.
fn pontas_campo() -> (usize, &'static str) {
    let k = points_handle().expect("a estrela tem a alça das pontas");
    let i = handles_of(PixelShape::STAR.kind())[k].fields[0];
    (i, field_name(desc(PixelShape::STAR.kind()).fields[i].label))
}

fn pontas(m: &Mesa) -> f64 {
    m.app
        .options
        .shapes
        .get(PixelShape::STAR)
        .get(pontas_campo().0)
}

fn ctrl(m: &mut Mesa, k: KeyCode, shift: bool) -> Option<KeyAnswer> {
    input::key(&mut m.app, m.sim.world(), Some(m.bits), k, true, shift)
}

fn readout(m: &Mesa) -> Option<(String, (f32, f32))> {
    let c = canvas(&m.sim, &m.cam, m.bits);
    viva::readout(&m.app, &c, m.key)
}

/// O painel da shell sobre a mesa: o `hero` em Draw no objecto.
fn painel(m: &mut Mesa, hero: &mut HeroScreen) -> PixelPanelInfo {
    panel::frame(&mut m.app, m.sim.world(), hero, &mut ToastQueue::new())
}

#[test]
fn scene_14_shows_the_points_readout_the_frame_numbers_and_one_adjustment_to_undo() {
    let m = Mesa::cena(14);
    let v = m.app.live_shape().expect("a estrela está viva");
    assert_eq!(v.forma(), PixelShape::STAR);
    assert_eq!(v.steps(), 1, "um ajuste guardado: 5 → 7 pontas");
    assert_eq!(pontas(&m), POINTS.1);
    assert_eq!(v.info().angle, 15, "a barra diz o giro, anti-horário");
    assert!(!v.outline_only(), "sem arrasto a caixa tem as alças");
    let (texto, onde) = readout(&m).expect("a alça das pontas mostra a etiqueta");
    assert_eq!(texto, format!("{}: 7", pontas_campo().1));
    let c = canvas(&m.sim, &m.cam, m.bits);
    let alca = viva::handles_on_screen(&m.app, &c, m.key)[points_handle().unwrap()];
    assert_eq!(Some(onde), alca, "a etiqueta pousa na alça mostrada");
}

#[test]
fn dragging_a_shape_handle_shows_its_readout_at_the_cursor_with_an_outline_only_box_as_one_step() {
    let mut m = Mesa::nova();
    m.desenha(Tool::Shape(PixelShape::STAR), (4, 4), (40, 34));
    let alca = m.alca(Glyph::Count);
    let c = canvas(&m.sim, &m.cam, m.bits);
    let longe = m.celula((2, 2));
    assert!(!input::moved(&mut m.app, &c, longe, Mods::default()));
    assert_eq!(readout(&m), None, "longe das alças, nenhuma etiqueta");
    let c = canvas(&m.sim, &m.cam, m.bits);
    assert!(!input::moved(&mut m.app, &c, alca, Mods::default()));
    let (texto, _) = readout(&m).expect("sob o cursor, antes do arrasto (F12 do Vector)");
    assert!(
        texto.starts_with(&format!("{}: ", pontas_campo().1)),
        "{texto}"
    );
    m.press(alca);
    let para = (alca.0 + 40.0, alca.1 + 25.0);
    let c = canvas(&m.sim, &m.cam, m.bits);
    assert!(input::moved(&mut m.app, &c, para, Mods::default()));
    let v = m.app.live_shape().expect("viva");
    assert!(
        v.outline_only(),
        "a meio do arrasto, só o contorno da caixa"
    );
    let (texto, onde) = readout(&m).expect("a etiqueta segue o arrasto");
    assert_eq!(onde, para, "junto do cursor");
    assert_eq!(texto, format!("{}: {}", pontas_campo().1, pontas(&m)));
    assert!(input::release(&mut m.app));
    let v = m.app.live_shape().expect("largar a alça não assenta");
    assert!(!v.outline_only(), "largou: a caixa volta a ter as alças");
    assert_eq!(v.steps(), 1, "um arrasto = um passo");
}

#[test]
fn the_frame_bar_shows_the_live_shape_and_a_field_moves_it_at_once_as_one_step() {
    let mut m = Mesa::cena(14);
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.mode.enter(m.bits, ObjectMode::Draw);
    let info = painel(&mut m, &mut hero);
    let v = m.app.live_shape().expect("viva");
    assert!(info.live_shape);
    let t = info
        .transform
        .expect("a barra da moldura mostra a forma viva");
    assert_eq!(t, v.info());
    assert_eq!((t.angle, t.w, t.h), (15, 60, 60));
    let min_x = |m: &Mesa| {
        let (contorno, _) = m.app.live_shape().expect("viva").cells(&m.app.options);
        contorno.iter().map(|c| c.0).min().expect("células")
    };
    let antes = min_x(&m);
    m.app.pede(PixelRequest::TransformField {
        field: PixelTransformField::X,
        value: t.x + 30,
    });
    let depois = painel(&mut m, &mut hero).transform.expect("barra");
    assert_eq!(depois.x, t.x + 30, "o campo muda-a na hora");
    let v = m.app.live_shape().expect("o campo não assenta");
    assert_eq!(v.steps(), 2, "um campo da barra = um passo");
    assert_eq!(
        min_x(&m),
        antes + 30,
        "a estrela andou 30 células para a direita"
    );
    assert!(matches!(
        ctrl(&mut m, KeyCode::KeyZ, false),
        Some(KeyAnswer::Quiet)
    ));
    assert_eq!(
        m.app.live_shape().expect("viva").info(),
        t,
        "o Ctrl+Z volta-a"
    );
    // «Apply» da barra assenta; «Cancel» largaria.
    m.app.pede(PixelRequest::Edit(PixelEditAction::Commit));
    painel(&mut m, &mut hero);
    assert!(m.app.live_shape().is_none(), "o Apply assenta-a");
    assert_ne!(m.v(STAR_CENTER), CLEAR);
    assert!(m.app.undo(m.key));
    assert_eq!(m.v(STAR_CENTER), CLEAR, "um passo do documento");
}

#[test]
fn saving_with_a_live_shape_settles_it_first_as_one_step_and_the_file_carries_it() {
    let mut m = Mesa::cena(14);
    assert_eq!(m.v(STAR_CENTER), CLEAR, "viva, ainda não é do documento");
    let bytes = persist::to_bytes(&mut m.app, m.sim.world());
    assert!(m.app.live_shape().is_none(), "gravar assentou-a");
    let estrela = m.v(STAR_CENTER);
    assert_eq!(estrela, STAR.2, "em células, cheia");
    assert!(m.app.undo(m.key));
    assert_eq!(m.v(STAR_CENTER), CLEAR, "assentar foi UM passo");
    assert_ne!(m.v(RECT_CELL), CLEAR, "o passo de antes continua");
    let lido = persist::read(&bytes).expect("o ficheiro lê-se");
    persist::install(&mut m.app, lido);
    assert_eq!(
        m.v(STAR_CENTER),
        estrela,
        "o ficheiro leva a estrela em células"
    );
}

#[test]
fn ctrl_z_undoes_the_last_adjustment_then_drops_the_shape_and_ctrl_shift_z_brings_both_back() {
    let mut m = Mesa::cena(14);
    let quiet = Some(KeyAnswer::Quiet);
    assert_eq!(ctrl(&mut m, KeyCode::KeyZ, false), quiet);
    assert_eq!(pontas(&m), POINTS.0, "o 1.º Ctrl+Z desfaz o ajuste");
    assert!(m.app.live_shape().is_some(), "e ela continua viva");
    assert_eq!(ctrl(&mut m, KeyCode::KeyZ, false), quiet);
    assert!(
        m.app.live_shape().is_none(),
        "sem ajustes, o Ctrl+Z larga-a"
    );
    assert_eq!(m.v(STAR_CENTER), CLEAR);
    assert_ne!(m.v(RECT_CELL), CLEAR, "o documento não andou");
    assert_eq!(ctrl(&mut m, KeyCode::KeyZ, false), quiet);
    assert_eq!(m.v(RECT_CELL), CLEAR, "o 3.º desfaz o passo do documento");
    assert_eq!(ctrl(&mut m, KeyCode::KeyZ, true), quiet);
    assert_ne!(m.v(RECT_CELL), CLEAR, "refaz primeiro o documento");
    assert!(m.app.live_shape().is_none());
    assert_eq!(ctrl(&mut m, KeyCode::KeyZ, true), quiet);
    let v = m.app.live_shape().expect("e depois a forma viva volta");
    assert_eq!(v.forma(), PixelShape::STAR);
    assert_eq!(m.app.tool, Tool::Shape(PixelShape::STAR));
    assert_eq!(pontas(&m), POINTS.0);
    assert_eq!(ctrl(&mut m, KeyCode::KeyZ, true), quiet);
    assert_eq!(pontas(&m), POINTS.1, "e o ajuste dela");
}

#[test]
fn a_committed_step_after_dropping_the_shape_forgets_it() {
    let mut m = Mesa::cena(14);
    for _ in 0..2 {
        ctrl(&mut m, KeyCode::KeyZ, false);
    }
    assert!(m.app.live_shape().is_none());
    m.desenha(Tool::Pencil, (100, 70), (110, 70));
    assert_eq!(ctrl(&mut m, KeyCode::KeyZ, false), Some(KeyAnswer::Quiet));
    let r = ctrl(&mut m, KeyCode::KeyZ, true);
    assert_eq!(r, Some(KeyAnswer::Quiet));
    assert!(
        m.app.live_shape().is_none(),
        "a largada não volta por cima de outro passo"
    );
}

#[test]
fn panel_edits_of_the_same_field_join_in_one_step_and_another_field_is_its_own() {
    let mut m = Mesa::cena(14);
    let (i, _) = pontas_campo();
    let star = PixelShape::STAR;
    let pede = |m: &mut Mesa, j: usize, n: f64| {
        let mut o = m.app.options;
        o.shapes.set(star, o.shapes.get(star).with(j, n));
        m.app.aplica(PixelRequest::Options(Box::new(o)));
    };
    let passos = |m: &Mesa| m.app.live_shape().expect("viva").steps();
    pede(&mut m, i, 8.0);
    assert_eq!(passos(&m), 2);
    pede(&mut m, i, 9.0);
    assert_eq!(passos(&m), 2, "o deslizar do mesmo campo é um passo");
    let outro = (0..desc(star.kind()).fields.len())
        .find(|&j| j != i)
        .unwrap();
    let mais = m.app.options.shapes.get(star).get(outro) + 0.1;
    pede(&mut m, outro, mais);
    assert_eq!(passos(&m), 3, "outro campo, outro passo");
    ctrl(&mut m, KeyCode::KeyZ, false);
    assert_eq!(pontas(&m), 9.0);
    ctrl(&mut m, KeyCode::KeyZ, false);
    assert_eq!(pontas(&m), POINTS.1);
}
