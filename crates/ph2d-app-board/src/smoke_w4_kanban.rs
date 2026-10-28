//! Cena 30 — o Kanban do TrelloClone (W4, ADR-0195 §3) VIVO no canvas: o quadro «Production board» com
//! o Kanban da cena 1 do Kanban («Short film — production», a «Review» recolhida NESTA vista) ao lado
//! de três notas, e a aba do MESMO Kanban aberta atrás. A zoom 1 é o Kanban inteiro (seleccionar, 2.º
//! clique para entrar, arrastar, escrever, abrir o verso); de mais longe, títulos e capas; de longe,
//! blocos de cor por lista. O gate `the_w4_kanban_scene_puts_the_scene_1_kanban_beside_notes` conduz-a.

use ph2d_board_model::{BoardId, BoardKanban, BoardOp, BoardSet, Element, ElementId, ShapeType};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_video::kanban as kb;
use ph2d_editor_core::screens::hero::board_view::default_style;
use ph2d_editor_core::screens::hero::document_tabs;
use ph2d_i18n::tr;
use ph2d_board_model::KanbanId;

/// O que o gate (e as fotos) conduzem.
#[cfg_attr(not(test), expect(dead_code, reason = "só o gate e as fotos leem os ids"))]
pub(crate) struct W4Kanban {
    pub board: BoardId,
    pub kanban: KanbanId,
    pub element: ElementId,
}

/// O Kanban no canvas: quatro listas abertas a zoom 1 (a aba mostra-as a 1200 px).
pub(crate) const KANBAN: [f64; 4] = [0.0, 0.0, 1160.0, 640.0];
/// As notas, à direita dele.
const NOTES_X: f64 = 1240.0;
const NOTE_W: f64 = 260.0;
const NOTE_GAP: f64 = 40.0;
/// O meio da vista inicial (o Kanban e as notas) e o zoom inteiro.
const VIEW: [f64; 3] = [760.0, 320.0, 1.0];

pub(crate) fn scene_w4_kanban(hero: &mut HeroScreen) -> W4Kanban {
    let shared = hero.documents.kanbans_shared();
    let (kanban, review) = ph2d_app_kanban::smoke::short_film_into(&mut shared.borrow_mut());
    let mut set = BoardSet::default();
    let board = set.create(tr("board.smoke.w4k.board").to_owned());
    let doc = &mut set.get_mut(board).expect("acabou de nascer").doc;
    let mut k = BoardKanban::new(kanban, default_style());
    k.view.collapsed.extend(review);
    let el = Element::new_kanban(doc.mint_id(), doc.z_on_top(), k, KANBAN);
    let element = el.id;
    BoardOp::Put(el).apply(doc);
    for (i, key) in ["board.smoke.w4k.n1", "board.smoke.w4k.n2", "board.smoke.w4k.n3"]
        .into_iter()
        .enumerate()
    {
        let y = KANBAN[1] + i as f64 * (NOTE_W + NOTE_GAP);
        super::put_note(doc, ShapeType::Sticky, i * 3, super::note_text(key), [NOTES_X, y, NOTE_W]);
    }
    document_tabs::load(hero, set);
    hero.documents.activate(Some(board));
    if let Some(b) = hero.documents.active_board_mut() {
        [b.camera.center_x, b.camera.center_y, b.camera.zoom] = VIEW;
    }
    // A aba do MESMO Kanban, aberta atrás do quadro (a família abre-a no 1.º quadro de imagem).
    kb::open_in_background(kanban);
    W4Kanban {
        board,
        kanban,
        element,
    }
}

#[cfg(test)]
#[path = "smoke_w4_kanban_tests.rs"]
mod tests;
