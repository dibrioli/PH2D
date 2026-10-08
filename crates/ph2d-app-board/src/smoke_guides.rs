//! Cena 7 — GUIAS e alinhamento de objectos rodados (W5b), módulo filho do roteador [`super`].
//!
//! Cada lição no seu FRAME (um filho só alinha com a casa dele — `03_guias_e_alinhamento.md` L2 —,
//! por isso uma lição não acende guias da outra), e cada «arraste-me» a 40 do alinhamento que
//! ensina, sem outro mais perto (o gate `every_lesson_teaches_its_own_alignment` conduz cada uma):
//! - «Tilted shapes»: dois rectângulos a 30°; B desliza ao longo da inclinação até a borda esquerda
//!   ficar na linha inclinada da de A;
//! - «Rotate»: um a 30°, um a 45° e um direito para RODAR (encaixa em 30°, 45° e direito);
//! - um frame RODADO a 20°: o 3.º da fila (vão 80 → 40, o espaçamento igual nos eixos do frame) e o
//!   de baixo (a esquerda 40 fora da do 1.º);
//! - «Equal spacing»: uma fila de três com vãos 60 e 100;
//! - «Frame A» e «Frame B» com os topos 40 desencontrados.

use super::{default_name, organize::frame, put_shape};
use ph2d_board_model::{BoardDoc, BoardOp, BoardSet, ElementId, Rgba, ShapeType, Style};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_nav;
use ph2d_editor_core::screens::hero::board_view::default_style;
use ph2d_editor_core::screens::hero::document_tabs;
use ph2d_editor_core::widget::panel_chrome::HIGHLIGHTER_RGBA;

/// O que se arrasta em cada lição (o gate conduz estes).
#[cfg_attr(not(test), expect(dead_code, reason = "só o gate da cena lê as lições"))]
pub(crate) struct Lessons {
    pub tilt_a: ElementId,
    pub tilt_b: ElementId,
    pub rotate: ElementId,
    pub row: [ElementId; 3],
    pub align: [ElementId; 2],
    pub spaced: [ElementId; 3],
    pub frames: [ElementId; 2],
}

/// O deslocamento de ensinar: 40 (de longe, a ~0,4, são 16 px de ecrã; a 1, 40).
pub(crate) const OFF: f64 = 40.0;
/// O ângulo da lição inclinada e o do frame rodado.
pub(crate) const TILT: f64 = 30.0;
pub(crate) const FRAME_TILT: f64 = 20.0;

fn tr(key: &str) -> String {
    ph2d_i18n::tr(key).to_owned()
}

/// O ponto a `(du, dv)` de `c` nos eixos rodados de `deg` graus.
pub(crate) fn local(c: [f64; 2], deg: f64, du: f64, dv: f64) -> [f64; 2] {
    let (s, k) = deg.to_radians().sin_cos();
    [c[0] + du * k - dv * s, c[1] + du * s + dv * k]
}

fn centred(c: [f64; 2], [w, h]: [f64; 2]) -> [f64; 4] {
    [c[0] - w / 2.0, c[1] - h / 2.0, w, h]
}

/// Uma forma com o texto `key`, na cor `c`, rodada `deg` graus, a viver em `home`.
fn put(
    doc: &mut BoardDoc,
    c: usize,
    key: &str,
    bx: [f64; 4],
    deg: f64,
    home: ElementId,
) -> ElementId {
    let mut st: Style = default_style();
    st.fill = Some(Rgba(HIGHLIGHTER_RGBA[c]));
    st.text_color = Rgba(HIGHLIGHTER_RGBA[c]).readable_ink();
    let id = put_shape(doc, ShapeType::Rectangle, st, key, bx);
    if let Some(mut el) = doc.get(id).cloned() {
        el.angle = deg.to_radians();
        el.frame = Some(home);
        BoardOp::Put(el).apply(doc);
    }
    id
}

fn lesson(doc: &mut BoardDoc, key: &str, bx: [f64; 4], deg: f64) -> ElementId {
    let f = frame(doc, &tr(key), bx, None);
    if let Some(mut el) = doc.get(f).cloned() {
        el.angle = deg.to_radians();
        BoardOp::Put(el).apply(doc);
    }
    f
}

/// Monta as lições no documento.
pub(crate) fn build(doc: &mut BoardDoc) -> Lessons {
    // 1 · Inclinados: a esquerda de B a OFF da de A, nos eixos de 30°; os centros a 80, as direitas
    // a 120 (B é mais largo).
    let l1 = lesson(
        doc,
        "board.smoke.guide.f_tilt",
        [0.0, 0.0, 1100.0, 650.0],
        0.0,
    );
    let ca = [320.0, 180.0];
    let tilt_a = put(
        doc,
        0,
        "board.smoke.guide.tilt_a",
        centred(ca, [320.0, 140.0]),
        TILT,
        l1,
    );
    let cb = local(ca, TILT, -160.0 + OFF + 200.0, 240.0);
    let tilt_b = put(
        doc,
        0,
        "board.smoke.guide.tilt_b",
        centred(cb, [400.0, 120.0]),
        TILT,
        l1,
    );
    // 2 · Rodar: na casa, 30°, 45° e o frame direito.
    let l2 = lesson(
        doc,
        "board.smoke.guide.f_rotate",
        [1250.0, 0.0, 1100.0, 650.0],
        0.0,
    );
    let small = [240.0, 110.0];
    put(
        doc,
        1,
        "board.smoke.guide.tilt_a",
        centred([1450.0, 180.0], small),
        30.0,
        l2,
    );
    put(
        doc,
        1,
        "board.smoke.guide.tilt_45",
        centred([2150.0, 180.0], small),
        45.0,
        l2,
    );
    let bx = centred([1800.0, 430.0], [300.0, 130.0]);
    let rotate = put(doc, 1, "board.smoke.guide.rotate_me", bx, 0.0, l2);
    // 3 · O frame rodado: a fila com vãos 40 e 80; o de baixo com a esquerda 40 antes da do 1.º.
    let cf = [550.0, 1180.0];
    let l3 = lesson(
        doc,
        "board.smoke.guide.f_tilted",
        centred(cf, [1100.0, 520.0]),
        FRAME_TILT,
    );
    let kid = |doc: &mut BoardDoc, key: &str, du: f64, dv: f64| {
        let bx = centred(local(cf, FRAME_TILT, du, dv), [200.0, 120.0]);
        put(doc, 3, key, bx, FRAME_TILT, l3)
    };
    let k1 = kid(doc, "board.smoke.guide.inside", -330.0, -80.0);
    let k2 = kid(doc, "board.smoke.guide.inside", -90.0, -80.0);
    let k3 = kid(
        doc,
        "board.smoke.guide.gap_me",
        10.0 + 2.0 * OFF + 100.0,
        -80.0,
    );
    let k4 = kid(doc, "board.smoke.guide.align_me", -330.0 - OFF, 130.0);
    // 4 · Espaçamento igual: vãos 60 e 100 (o 3.º anda OFF para a esquerda).
    let l5 = lesson(
        doc,
        "board.smoke.guide.f_equal",
        [1250.0, 800.0, 1100.0, 650.0],
        0.0,
    );
    let spaced = [1350.0, 1630.0, 1910.0 + OFF].map(|x| {
        let key = if x > 1910.0 {
            "board.smoke.guide.space_me"
        } else {
            "board.smoke.guide.equal"
        };
        put(doc, 2, key, [x, 1000.0, 220.0, 120.0], 0.0, l5)
    });
    // 5 · Dois frames com os topos OFF desencontrados (soltos no quadro, como as lições).
    let fa = lesson(
        doc,
        "board.smoke.guide.f_a",
        [0.0, 1750.0, 600.0, 450.0],
        0.0,
    );
    put(
        doc,
        4,
        "board.smoke.guide.inside",
        [100.0, 1890.0, 240.0, 120.0],
        0.0,
        fa,
    );
    let fb = lesson(
        doc,
        "board.smoke.guide.f_b",
        [750.0, 1750.0 + OFF, 600.0, 450.0],
        0.0,
    );
    let inner = [850.0, 1890.0 + OFF, 240.0, 120.0];
    put(doc, 4, "board.smoke.guide.inside", inner, 0.0, fb);
    Lessons {
        tilt_a,
        tilt_b,
        rotate,
        row: [k1, k2, k3],
        align: [k1, k4],
        spaced,
        frames: [fa, fb],
    }
}

pub(super) fn scene_guides(hero: &mut HeroScreen) {
    let mut set = BoardSet::default();
    let id = set.create(default_name(1));
    let board = set.get_mut(id).expect("acabou de nascer");
    build(&mut board.doc);
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show(hero, false, false);
}

#[cfg(test)]
#[path = "smoke_guides_tests.rs"]
mod tests;
