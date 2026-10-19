//! B5(4) — o que NÃO entra no desfazer das abas.

use super::*;
use ph2d_board_model::BoardSet;

/// Abrir outro projecto: os ids recomeçam, e um passo do anterior cairia sobre outro quadro.
#[test]
fn loading_a_project_forgets_the_tab_steps() {
    let mut d = Documents::default();
    d.create("A".into());
    assert!(d.tab_undo_at().is_some());
    d.replace(BoardSet::default());
    assert_eq!(d.tab_undo_at(), None);
    assert_eq!(d.undo_tab(false), None);
}

/// Entrar numa sala ao vivo traz o quadro dela numa aba nova — tirá-lo com `Ctrl+Z` seria sair da sala.
#[test]
fn a_board_that_arrived_from_a_live_room_is_not_a_step() {
    let mut d = Documents::default();
    let mut other = BoardSet::default();
    let id = other.create("Sala".into());
    let board = other.get(id).unwrap().clone();
    d.adopt_board(board);
    assert_eq!(d.tab_undo_at(), None);
}

/// O passo de um quadro que saiu por outra via gasta-se e o seguinte responde no MESMO `Ctrl+Z`.
#[test]
fn a_step_whose_board_is_gone_is_spent_and_the_next_answers() {
    let mut d = Documents::default();
    let a = d.create("A".into());
    let b = d.create("B".into());
    assert!(d.rename(a, "A2".into()));
    let _ = d.take_board(a);
    assert_eq!(d.undo_tab(false), Some(TabChange::Gone(b)), "o criar de B");
}
