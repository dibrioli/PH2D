//! Cena 29 — W24 (`docs/MiroClone/handoffs/HANDOFF_CONTINUACAO_W24_*`), módulo filho do roteador
//! [`super`]: o quadro «Bars» com um frame e duas caixas — a da ESQUERDA rente à barra curta (onde as
//! gavetas abrem) e já seleccionada, com a gaveta das formas e o painel dos frames abertos, a da
//! DIREITA por baixo de onde abrem os painéis do canto — e as três lições por baixo. O gate `the_w24_scene_opens_with_the_left_box_and_its_bar` conduz o que a
//! cena arma.

use ph2d_board_model::{BoardId, BoardSet, ShapeType};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_view::{self, default_style};
use ph2d_editor_core::screens::hero::{board_bar, board_nav, document_tabs};
use ph2d_i18n::tr;

/// O que o gate conduz.
#[cfg_attr(not(test), expect(dead_code, reason = "só o gate da cena lê os ids"))]
pub(crate) struct Bars {
    pub board: BoardId,
}

/// O frame: largo, para a caixa da esquerda cair rente à barra curta e a da direita sob os painéis.
pub(crate) const FRAME: [f64; 4] = [0.0, 0.0, 1800.0, 1400.0];
const BOX: [f64; 2] = [240.0, 150.0];
const INSET: f64 = 40.0;
const LESSON_GAP: f64 = 120.0;
const LESSON_W: f64 = 500.0;
/// As lições começam depois da gaveta das formas, que abre por cima do canto esquerdo (foto de 23/10).
const LESSON_X: f64 = 220.0;

/// A cena; devolve os ids que o gate conduz.
pub(crate) fn scene_bars(hero: &mut HeroScreen) -> Bars {
    let mut set = BoardSet::default();
    let board = set.create(tr("board.smoke.w24.board").to_owned());
    let mut style = default_style();
    style.stroke = Some(super::team::INK);
    style.text_color = super::team::INK;
    style.stroke_width *= 3.0;
    let doc = &mut set.get_mut(board).expect("acabou de nascer").doc;
    super::organize::frame(doc, tr("board.smoke.w24.frame"), FRAME, None);
    // A caixa da esquerda em baixo, à altura da gaveta das formas: a barra dela, por cima, cairia
    // sobre a gaveta (a foto de 23/10 com ela no alto não provava nada).
    let y = FRAME[1] + FRAME[3] - INSET - BOX[1];
    let boxes = [FRAME[0] + LESSON_X, FRAME[0] + FRAME[2] - INSET - BOX[0]].map(|x| {
        let shape = ph2d_board_model::Shape {
            kind: ShapeType::Rectangle,
            style: style.clone(),
            text: Default::default(),
        };
        let at = [x, y, BOX[0], BOX[1]];
        let el = ph2d_board_model::Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, at);
        let id = el.id;
        ph2d_board_model::BoardOp::Put(el).apply(doc);
        id
    });
    for (i, key) in ["board.smoke.w24.l1", "board.smoke.w24.l2", "board.smoke.w24.l3"]
        .into_iter()
        .enumerate()
    {
        super::put_note(
            doc,
            ShapeType::StickyWide,
            0,
            super::note_text(key),
            [
                FRAME[0] + LESSON_X + i as f64 * (LESSON_W + 40.0),
                FRAME[1] + FRAME[3] + LESSON_GAP,
                LESSON_W,
            ],
        );
    }
    document_tabs::load(hero, set);
    hero.documents.activate(Some(board));
    board_nav::show(hero, true, false);
    board_view::select(hero, [boxes[0]]);
    // A gaveta das formas ABERTA à abertura (a foto não clica): a barra da caixa foge dela.
    let more = board_bar::node(board_bar::Item::MoreShapes);
    hero.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(more));
    Bars { board }
}

#[cfg(test)]
#[path = "smoke_w24_bars_tests.rs"]
mod tests;
