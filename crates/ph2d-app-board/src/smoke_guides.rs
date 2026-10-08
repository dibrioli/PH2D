//! Cena 7 — GUIAS e alinhamento de objectos rodados (W5b), módulo filho do roteador [`super`].
//!
//! Tudo já QUASE alinhado, para o dono arrastar poucos px e ver a guia acender:
//! - dois rectângulos a 30° com as bordas esquerdas 18 px fora da mesma linha inclinada;
//! - um a 45° e um DIREITO para rodar perto dos 30°/45° (encaixa no ângulo, `Shift` = 15°);
//! - um frame rodado a 20° com quatro filhos: vãos 40 e 62 (o espaçamento igual nos eixos do frame)
//!   e um filho com a esquerda 18 px fora da de outro;
//! - dois frames direitos com os topos 25 px desencontrados;
//! - uma fila de três com vãos 60 e 75 (o espaçamento igual).

use super::{default_name, put_shape};
use ph2d_board_model::{
    BoardDoc, BoardOp, BoardSet, Element, ElementId, Frame, Rgba, ShapeType, Style,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_nav;
use ph2d_editor_core::screens::hero::board_view::default_style;
use ph2d_editor_core::screens::hero::document_tabs;
use ph2d_editor_core::widget::panel_chrome::HIGHLIGHTER_RGBA;

fn tr(key: &str) -> String {
    ph2d_i18n::tr(key).to_owned()
}

/// O ponto a `(du, dv)` de `c` nos eixos rodados de `deg` graus.
fn local(c: [f64; 2], deg: f64, du: f64, dv: f64) -> [f64; 2] {
    let (s, k) = deg.to_radians().sin_cos();
    [c[0] + du * k - dv * s, c[1] + du * s + dv * k]
}

fn centred(c: [f64; 2], [w, h]: [f64; 2]) -> [f64; 4] {
    [c[0] - w / 2.0, c[1] - h / 2.0, w, h]
}

/// Uma forma com o texto `key`, rodada `deg` graus, a viver em `home`.
fn tilted(
    doc: &mut BoardDoc,
    style: &Style,
    key: &str,
    bx: [f64; 4],
    deg: f64,
    home: Option<ElementId>,
) -> ElementId {
    let id = put_shape(doc, ShapeType::Rectangle, style.clone(), key, bx);
    if let Some(mut el) = doc.get(id).cloned() {
        el.angle = deg.to_radians();
        el.frame = home;
        BoardOp::Put(el).apply(doc);
    }
    id
}

fn frame(doc: &mut BoardDoc, key: &str, bx: [f64; 4], deg: f64) -> ElementId {
    let paper = Rgba(ph2d_board_model::FRAME_FILL);
    let f = Frame {
        name: tr(key),
        style: Style::new(Some(paper), None, paper.readable_ink()),
    };
    let mut el = Element::new_frame(doc.mint_id(), doc.z_on_top(), f, bx);
    el.angle = deg.to_radians();
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

pub(super) fn scene_guides(hero: &mut HeroScreen) {
    let mut set = BoardSet::default();
    let id = set.create(default_name(1));
    let board = set.get_mut(id).expect("acabou de nascer");
    let doc = &mut board.doc;
    let pastel = |i: usize| {
        let mut st = default_style();
        st.fill = Some(Rgba(HIGHLIGHTER_RGBA[i]));
        st.text_color = Rgba(HIGHLIGHTER_RGBA[i]).readable_ink();
        st
    };
    // 1 · Inclinados: B arrasta-se para a linha inclinada de A. 2 · Rodar: o direito encaixa nos
    // 30° de A, nos 45° do rosa e de volta a direito.
    let ca = [300.0, 250.0];
    let cb = local(ca, 30.0, -160.0 + 18.0 + 130.0, 260.0);
    let loose = [
        (0, "board.smoke.guide.tilt_a", ca, [320.0, 140.0], 30.0),
        (0, "board.smoke.guide.tilt_b", cb, [260.0, 130.0], 30.0),
        (
            1,
            "board.smoke.guide.tilt_45",
            [850.0, 200.0],
            [260.0, 130.0],
            45.0,
        ),
        (
            1,
            "board.smoke.guide.rotate_me",
            [850.0, 520.0],
            [300.0, 130.0],
            0.0,
        ),
    ];
    for (c, key, at, size, deg) in loose {
        tilted(doc, &pastel(c), key, centred(at, size), deg, None);
    }
    // 3 · O frame rodado a 20°: vãos 40 e 62 (o 3.º 14 fora da fila) e o de baixo 18 fora da
    // esquerda do 1.º — nos eixos do frame.
    let cf = [600.0, 1250.0];
    let f = frame(
        doc,
        "board.smoke.guide.f_tilted",
        centred(cf, [1000.0, 520.0]),
        20.0,
    );
    let kids = [
        ("board.smoke.guide.inside", -330.0, -60.0),
        ("board.smoke.guide.inside", -90.0, -60.0),
        ("board.smoke.guide.gap_me", 172.0, -46.0),
        ("board.smoke.guide.align_me", -312.0, 120.0),
    ];
    for (key, du, dv) in kids {
        let bx = centred(local(cf, 20.0, du, dv), [200.0, 120.0]);
        tilted(doc, &pastel(3), key, bx, 20.0, Some(f));
    }
    // 4 · Dois frames direitos com os topos 25 desencontrados.
    for (key, x, y) in [
        ("board.smoke.guide.f_a", 1150.0, 0.0),
        ("board.smoke.guide.f_b", 1750.0, 25.0),
    ] {
        let fr = frame(doc, key, [x, y, 500.0, 400.0], 0.0);
        let bx = [x + 100.0, y + 140.0, 240.0, 120.0];
        tilted(
            doc,
            &pastel(4),
            "board.smoke.guide.inside",
            bx,
            0.0,
            Some(fr),
        );
    }
    // 5 · Espaçamento igual numa fila: vãos 60 e 75.
    for (x, key) in [
        (1150.0, "board.smoke.guide.equal"),
        (1430.0, "board.smoke.guide.equal"),
        (1725.0, "board.smoke.guide.space_me"),
    ] {
        tilted(doc, &pastel(2), key, [x, 900.0, 220.0, 120.0], 0.0, None);
    }
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show(hero, false, false);
}
