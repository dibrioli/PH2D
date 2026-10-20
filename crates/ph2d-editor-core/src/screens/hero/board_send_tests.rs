//! ⭐ W23 B15 — **de um quadro para outro** pelo caminho REAL do ecrã (doc 17 §2.4): o botão direito
//! numa forma → «Move to board…» / «Copy to board…» → a linha do quadro no modal → a pintura seguinte
//! pousa a selecção no destino (activo) e o aviso «Undo» desfaz os dois quadros.

use super::*;
use crate::screens::hero::document_tabs::register_board;
use ph2d_board_model::BoardId;

impl T {
    /// O botão direito no ponto, pela porta da shell.
    fn right(&mut self, (x, y): (f32, f32)) {
        for kind in [PointerKind::Down, PointerKind::Up] {
            let (h, ts, c) = (&mut self.hero, &mut self.ts, &mut self.clock);
            board_view::pointer(h, inp(ts, c), kind, PointerButton::Secondary, x, y, true);
        }
        self.paint();
    }

    fn count(&self, b: BoardId) -> usize {
        self.hero
            .documents
            .boards()
            .get(b)
            .map_or(0, |b| b.doc.live().count())
    }
}

/// Duas caixas no quadro de nascença, as duas seleccionadas, e um 2.º quadro «Plan» (por trás).
fn two_boxes_and_a_plan() -> (T, BoardId, BoardId, (f32, f32)) {
    let mut t = T::new();
    let first = t.hero.documents.active().expect("o quadro de nascença");
    let plan = t.hero.documents.create("Plan".to_owned());
    register_board(&mut t.hero.store, plan);
    t.hero.documents.activate(Some(first));
    t.paint();
    for (a, b) in [((0.2, 0.4), (0.3, 0.5)), ((0.5, 0.4), (0.6, 0.5))] {
        t.key(BoardKey::Char('r'), Modifiers::default(), Some("r"));
        let (a, b) = (t.at(a.0, a.1), t.at(b.0, b.1));
        t.drag(a, b);
    }
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    t.key(BoardKey::Char('a'), ctrl, None);
    let shape = t.at(0.25, 0.45);
    (t, first, plan, shape)
}

#[test]
fn move_to_board_from_the_menu_lands_on_the_other_board_and_undo_puts_them_back() {
    let (mut t, first, plan, shape) = two_boxes_and_a_plan();
    assert_eq!(t.count(first), 2);
    t.right(shape);
    t.shell_click_id(ids::CTX_MENU_BOARD_EL_MOVE_TO, "Move to board…");
    // O modal não oferece o próprio quadro: a 1.ª linha é o «Plan».
    t.shell_click_id(ids::CTX_SEND_BOARD_ROWS[0], "a linha do «Plan»");
    assert_eq!(t.hero.documents.active(), Some(plan), "o destino fica activo");
    assert_eq!((t.count(first), t.count(plan)), (0, 2), "moveu as duas");
    let ed = t.hero.documents.live.editor.as_ref().unwrap();
    assert_eq!(ed.selection().len(), 2, "chegam seleccionadas");
    t.shell_click_id(ids::BOARD_ALERT_ACTION, "Undo");
    assert_eq!((t.count(first), t.count(plan)), (2, 0), "o Undo devolve-as");
    assert_eq!(t.hero.documents.active(), Some(first));
    assert!(t.find(ids::BOARD_ALERT_ACTION).is_none(), "o aviso fechou");
}

#[test]
fn copy_to_board_keeps_both_and_offers_no_undo_button() {
    let (mut t, first, plan, shape) = two_boxes_and_a_plan();
    t.right(shape);
    t.shell_click_id(ids::CTX_MENU_BOARD_EL_COPY_TO, "Copy to board…");
    t.shell_click_id(ids::CTX_SEND_BOARD_ROWS[0], "a linha do «Plan»");
    assert_eq!((t.count(first), t.count(plan)), (2, 2));
    assert!(t.find(ids::BOARD_ALERT_ACTION).is_none(), "copiar não se desfaz por botão");
}

/// O `Ctrl+Z` de um quadro nunca desfaz noutro (doc 02 §1.4): depois dele, o botão do aviso já não
/// faria o que diz e some.
#[test]
fn ctrl_z_on_the_target_undoes_only_it_and_the_undo_button_goes() {
    let (mut t, first, plan, shape) = two_boxes_and_a_plan();
    t.right(shape);
    t.shell_click_id(ids::CTX_MENU_BOARD_EL_MOVE_TO, "Move to board…");
    t.shell_click_id(ids::CTX_SEND_BOARD_ROWS[0], "a linha do «Plan»");
    assert!(t.find(ids::BOARD_ALERT_ACTION).is_some());
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    t.key(BoardKey::Char('z'), ctrl, None);
    assert_eq!((t.count(first), t.count(plan)), (0, 0));
    assert!(t.find(ids::BOARD_ALERT_ACTION).is_none(), "o botão mentiria");
}

/// `Ctrl+C` num quadro e `Ctrl+V` noutro é a MESMA chegada: no meio da vista do destino.
#[test]
fn ctrl_v_in_another_tab_lands_in_the_middle_of_its_view() {
    let (mut t, first, plan, _) = two_boxes_and_a_plan();
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    t.key(BoardKey::Char('c'), ctrl, None);
    t.hero.documents.activate(Some(plan));
    t.paint();
    t.key(BoardKey::Char('v'), ctrl, None);
    assert_eq!((t.count(first), t.count(plan)), (2, 2));
    let b = t.hero.documents.boards().get(plan).unwrap();
    let (lo, hi) = b.doc.live().fold(
        ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]),
        |(lo, hi), e| {
            (
                [lo[0].min(e.x), lo[1].min(e.y)],
                [hi[0].max(e.x + e.w), hi[1].max(e.y + e.h)],
            )
        },
    );
    let mid = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    let cam = [b.camera.center_x, b.camera.center_y];
    assert!(
        (mid[0] - cam[0]).abs() < 1e-6 && (mid[1] - cam[1]).abs() < 1e-6,
        "{mid:?} longe do meio da vista {cam:?}"
    );
}

/// «New board» no modal: o quadro nasce, recebe a selecção e fica activo.
#[test]
fn move_to_a_new_board_creates_it() {
    let (mut t, first, plan, shape) = two_boxes_and_a_plan();
    t.right(shape);
    t.shell_click_id(ids::CTX_MENU_BOARD_EL_MOVE_TO, "Move to board…");
    t.shell_click_id(ids::CTX_SEND_BOARD_NEW, "New board");
    let fresh = t.hero.documents.active().unwrap();
    assert!(fresh != first && fresh != plan);
    assert_eq!((t.count(first), t.count(fresh)), (0, 2));
}
