//! Cena 20 — o VÍDEO com os NOSSOS controles (W19), módulo filho do roteador [`super`]: num frame
//! «YouTube» (um diapositivo: a apresentação toca-o) o vídeo das medições, com espaço por baixo para a
//! barra; por baixo uma nota por lição. O gate `the_w19_scene_does_what_its_lessons_say` conduz o que a cena
//! arma.

use super::team::{note, put, tr};
use ph2d_board_model::{
    Board, BoardId, BoardSet, Element, ElementId, Embed, Rgba, RichText, STICKY_SIDE, Shape,
    ShapeType, Style,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// O que o gate conduz.
#[cfg_attr(not(test), expect(dead_code, reason = "só o gate da cena lê os ids"))]
pub(crate) struct Lessons {
    pub board: BoardId,
    pub video: ElementId,
    pub frame: ElementId,
    pub lessons: [ElementId; 5],
}

const GAP: f64 = 40.0;
/// O espaço por baixo do vídeo onde a barra cabe (no zoom com que a cena abre).
const BAR_ROOM: f64 = 90.0;
/// A margem da vista: à esquerda passa a coluna de ferramentas (foto de 12/10: a lição 1 cortava).
const MARGIN_X: f64 = 200.0;
const MARGIN_Y: f64 = 90.0;
const LESSONS: usize = 5;
/// As lições em duas linhas (3 + 2): a cena cabe na área do quadro com os dois painéis abertos.
const PER_ROW: usize = 3;

/// Monta o quadro em `b`; devolve os ids e a vista que enquadra tudo.
pub(crate) fn build(b: &mut Board) -> (Lessons, [f64; 4]) {
    let side = STICKY_SIDE;
    let video = ph2d_board_embed::youtube(super::w18::LINK).expect("o link da cena é um vídeo");
    let [vw, vh] = ph2d_board_embed::natural_size(&video);
    let top = GAP + 20.0;
    let fw = vw + 2.0 * GAP;
    let fh = top + vh + BAR_ROOM;
    let frame = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.w19.youtube"),
        [0.0, 0.0, fw, fh],
        None,
    );
    let mut st = Style::new(None, None, Rgba(ph2d_board_model::DEFAULT_INK));
    st.round = true;
    let e = Embed {
        link: super::w18::LINK.to_owned(),
        video,
        style: st,
    };
    let mut v = Element::new_embed(b.doc.mint_id(), b.doc.z_on_top(), e, [GAP, top, vw, vh]);
    v.frame = Some(frame);
    let video_id = v.id;
    put(b, v);
    let y = fh + GAP;
    let lessons: [ElementId; LESSONS] = std::array::from_fn(|i| {
        let key = format!("board.smoke.w19.l{}", i + 1);
        let (col, row) = ((i % PER_ROW) as f64, (i / PER_ROW) as f64);
        let at = [col * (side + GAP), y + row * (side + GAP), side, side];
        note(b, &key, 1, at, false, None)
    });
    let right = fw.max(PER_ROW as f64 * (side + GAP) - GAP);
    let bottom = y + LESSONS.div_ceil(PER_ROW) as f64 * (side + GAP) - GAP;
    let title = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, None, super::team::INK),
        text: RichText::plain(&tr("board.smoke.w19.title")),
    };
    let el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        title,
        [0.0, -80.0, right, 40.0],
    );
    put(b, el);
    let view = [
        -MARGIN_X,
        -80.0 - MARGIN_Y,
        right + MARGIN_X,
        bottom + MARGIN_Y,
    ];
    let l = Lessons {
        board: b.id,
        video: video_id,
        frame,
        lessons,
    };
    (l, view)
}

/// A cena 20 no hero. Devolve os ids (os testes conduzem-na pelo MESMO caminho do smoke).
pub(super) fn scene_w19(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(tr("board.smoke.w19.board"));
    let (l, view) = build(set.get_mut(id).expect("acabou de nascer"));
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, view);
    l
}

#[cfg(test)]
#[path = "smoke_w19_tests.rs"]
mod tests;
