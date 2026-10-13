//! Cena 21 — o VÍDEO que se arrasta, que o pan não pausa e que se decora (W20), módulo filho do roteador
//! [`super`]: o vídeo das medições com uma borda grossa e um fundo (a decoração do rectângulo), espaço à volta
//! para o arrastar e, por baixo, uma nota por lição. O gate `the_w20_scene_does_what_its_lessons_say` conduz
//! o que a cena arma.

use super::team::{note, put, tr};
use ph2d_board_model::{
    Board, BoardId, BoardSet, Element, ElementId, Embed, Rgba, RichText, STICKY_COLORS,
    STICKY_SIDE, Shape, ShapeType, Style,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};
use ph2d_editor_core::widget::panel_chrome::HIGHLIGHTER_RGBA;

/// O que o gate conduz.
#[cfg_attr(not(test), expect(dead_code, reason = "só o gate da cena lê os ids"))]
pub(crate) struct Lessons {
    pub board: BoardId,
    pub video: ElementId,
    pub lessons: [ElementId; LESSONS],
}

const GAP: f64 = 40.0;
/// A borda da cena (unidades do quadro): grossa o bastante para se ver o tocador recuado dentro dela.
const BORDER: f64 = 8.0;
/// O espaço por baixo do vídeo onde a barra cabe, e à direita para o arrastar.
const BAR_ROOM: f64 = 90.0;
const DRAG_ROOM: f64 = 240.0;
/// A margem da vista: à esquerda passa a coluna de ferramentas (a lição 1 da cena 20 cortava).
const MARGIN_X: f64 = 200.0;
const MARGIN_Y: f64 = 90.0;
pub(crate) const LESSONS: usize = 6;
/// As lições em duas linhas (3 + 3): a cena cabe na área do quadro com os dois painéis abertos.
const PER_ROW: usize = 3;
/// A borda vermelha das notas e o fundo amarelo do marcador (as cores do quadro, não hex soltos).
const RED: usize = 11;
const YELLOW: usize = 0;

/// Monta o quadro em `b`; devolve os ids e a vista que enquadra tudo.
pub(crate) fn build(b: &mut Board) -> (Lessons, [f64; 4]) {
    let side = STICKY_SIDE;
    let video = ph2d_board_embed::youtube(super::w18::LINK).expect("o link da cena é um vídeo");
    let [vw, vh] = ph2d_board_embed::natural_size(&video);
    let mut st = Style::new(
        Some(Rgba(HIGHLIGHTER_RGBA[YELLOW])),
        Some(Rgba(STICKY_COLORS[RED])),
        Rgba(ph2d_board_model::DEFAULT_INK),
    );
    (st.stroke_width, st.round) = (BORDER, true);
    let e = Embed {
        link: super::w18::LINK.to_owned(),
        video,
        style: st,
    };
    let v = Element::new_embed(b.doc.mint_id(), b.doc.z_on_top(), e, [0.0, 0.0, vw, vh]);
    let video_id = put(b, v);
    let y = vh + BAR_ROOM;
    let lessons: [ElementId; LESSONS] = std::array::from_fn(|i| {
        let key = format!("board.smoke.w20.l{}", i + 1);
        let (col, row) = ((i % PER_ROW) as f64, (i / PER_ROW) as f64);
        let at = [col * (side + GAP), y + row * (side + GAP), side, side];
        note(b, &key, 1, at, false, None)
    });
    let right = (vw + DRAG_ROOM).max(PER_ROW as f64 * (side + GAP) - GAP);
    let bottom = y + LESSONS.div_ceil(PER_ROW) as f64 * (side + GAP) - GAP;
    let title = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, None, super::team::INK),
        text: RichText::plain(&tr("board.smoke.w20.title")),
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
        lessons,
    };
    (l, view)
}

/// A cena 21 no hero. Devolve os ids (os testes conduzem-na pelo MESMO caminho do smoke).
pub(super) fn scene_w20(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(tr("board.smoke.w20.board"));
    let (l, view) = build(set.get_mut(id).expect("acabou de nascer"));
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, view);
    l
}

#[cfg(test)]
#[path = "smoke_w20_tests.rs"]
mod tests;
