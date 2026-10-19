//! ⭐ B5(4) — o `Ctrl+Z` das operações de aba (doc 15), pelo caminho REAL: a aba pelo menu e pelo
//! arrasto do despacho, o `Ctrl+Z` pela porta do teclado do quadro (`board_keys::key`).

use super::menu_tests::{click_anywhere, drag, name_of, order, right_click, with_boards};
use super::tests::{click, find, repaint};
use super::*;
use crate::screens::hero::board_keys::{self, BoardKey};
use ph2d_board_model::{BoardOp, Element, ElementId, Rgba, Shape, ShapeType, Style};
use ph2d_host::Modifiers;

/// `Ctrl+Z` (`Ctrl+Shift+Z` com `redo`) no quadro activo, carregar e largar.
fn ctrl_z(hero: &mut HeroScreen, text: &mut TextSystem, redo: bool) {
    let mods = Modifiers {
        ctrl: true,
        shift: redo,
        ..Modifiers::default()
    };
    let k = BoardKey::Char('z');
    assert!(board_keys::key(hero, text, k, mods, true, None, None).consumed);
    board_keys::key(hero, text, k, mods, false, None, None);
    repaint(hero, text);
}

/// Um rectângulo no quadro activo, como um gesto o deixa: aplicado e guardado no desfazer dele.
fn draw(hero: &mut HeroScreen) -> ElementId {
    let (board, live) = hero.documents.active_parts().expect("um quadro aberto");
    let ink = Rgba([20, 20, 20, 255]);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(Some(ink), None, ink),
        text: Default::default(),
    };
    let doc = &mut board.doc;
    let el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, [0.0, 0.0, 10.0, 10.0]);
    let id = el.id;
    let h = live.histories.entry(board.id).or_default();
    h.apply(doc, vec![BoardOp::Put(el)]);
    id
}

fn has(hero: &HeroScreen, b: BoardId, el: ElementId) -> bool {
    hero.documents
        .boards()
        .get(b)
        .is_some_and(|b| b.doc.get(el).is_some())
}

fn delete(hero: &mut HeroScreen, text: &mut TextSystem, b: BoardId) {
    right_click(hero, tab_node_id(b));
    repaint(hero, text);
    click_anywhere(hero, ids::CTX_MENU_BOARD_DELETE);
    repaint(hero, text);
    click_anywhere(hero, ids::CTX_MENU_BOARD_DELETE_CONFIRM);
    repaint(hero, text);
}

/// O tldraw (`paginas_P5`): o apagado volta com o conteúdo, a posição e o id, e aberto se o estava. E
/// volta com o desfazer DELE — o `Ctrl+Z` seguinte tira o rectângulo; e os refazer, pela ordem inversa.
#[test]
fn ctrl_z_brings_a_deleted_board_back_whole_with_its_own_undo() {
    let (mut hero, mut text, b) = with_boards(3);
    click(&mut hero, tab_node_id(b[1]));
    repaint(&mut hero, &mut text);
    let el = draw(&mut hero);
    delete(&mut hero, &mut text, b[1]);
    assert_eq!(order(&hero), vec![b[0], b[2]]);
    assert_eq!(hero.documents.active(), Some(b[2]), "a vizinha abriu");

    ctrl_z(&mut hero, &mut text, false);
    assert_eq!(order(&hero), b, "voltou ao mesmo lugar, com o mesmo id");
    assert_eq!(
        hero.documents.active(),
        Some(b[1]),
        "estava aberto: volta aberto"
    );
    assert!(has(&hero, b[1], el), "com o que tinha");
    assert!(
        find(&hero, tab_node_id(b[1])).is_some(),
        "a aba voltou à barra"
    );

    ctrl_z(&mut hero, &mut text, false);
    assert!(!has(&hero, b[1], el), "o desfazer dele voltou com ele");
    ctrl_z(&mut hero, &mut text, true);
    assert!(has(&hero, b[1], el));
    ctrl_z(&mut hero, &mut text, true);
    assert_eq!(
        order(&hero),
        vec![b[0], b[2]],
        "o último refazer apaga-o de novo"
    );
    assert!(find(&hero, tab_node_id(b[2])).is_some());
}

/// O tldraw (`paginas_P6`): apagar uma aba que NÃO está aberta e desfazer não muda a aberta.
#[test]
fn undoing_the_delete_of_another_tab_keeps_the_open_one() {
    let (mut hero, mut text, b) = with_boards(2);
    delete(&mut hero, &mut text, b[0]);
    assert_eq!(hero.documents.active(), Some(b[1]));
    ctrl_z(&mut hero, &mut text, false);
    assert_eq!(order(&hero), b);
    assert_eq!(hero.documents.active(), Some(b[1]));
}

/// ⭐ A lei do doc 15: o `Ctrl+Z` desfaz o MAIS RECENTE entre o último passo do quadro aberto e a última
/// operação de abas — nos dois sentidos.
#[test]
fn the_newer_of_the_board_step_and_the_tab_step_answers_ctrl_z() {
    let (mut hero, mut text, b) = with_boards(2);
    let el = draw(&mut hero);
    assert!(hero.documents.rename(b[0], "Plano".into()));
    ctrl_z(&mut hero, &mut text, false);
    assert_eq!(
        name_of(&hero, b[0]),
        name_of_fresh(1),
        "o nome, que era o mais recente"
    );
    assert!(has(&hero, b[1], el), "o desenho fica");
    ctrl_z(&mut hero, &mut text, false);
    assert!(!has(&hero, b[1], el));

    // Ao contrário: a aba antes, o desenho depois.
    ctrl_z(&mut hero, &mut text, true);
    ctrl_z(&mut hero, &mut text, true);
    assert_eq!(name_of(&hero, b[0]), "Plano");
    let el2 = draw(&mut hero);
    ctrl_z(&mut hero, &mut text, false);
    assert!(!has(&hero, b[1], el2), "o desenho, que é o mais recente");
    assert_eq!(name_of(&hero, b[0]), "Plano");
}

fn name_of_fresh(n: usize) -> String {
    ph2d_i18n::tr_with("board.tab.default_name", &[("n", &n)])
}

/// Criar, duplicar e arrastar desfazem-se; criar e duplicar devolvem a aba que estava aberta.
#[test]
fn create_duplicate_and_drag_come_back_with_ctrl_z() {
    let (mut hero, mut text, b) = with_boards(2);
    click(&mut hero, ids::DOC_TAB_NEW);
    repaint(&mut hero, &mut text);
    let made = hero.documents.active().unwrap();
    assert_ne!(made, b[1]);
    ctrl_z(&mut hero, &mut text, false);
    assert_eq!(order(&hero), b, "a aba criada sai");
    assert_eq!(
        hero.documents.active(),
        Some(b[1]),
        "e abre a que estava aberta"
    );
    assert!(
        find(&hero, tab_node_id(made)).is_none(),
        "nem pintada nem clicável"
    );
    ctrl_z(&mut hero, &mut text, true);
    assert_eq!(
        order(&hero),
        vec![b[0], b[1], made],
        "o refazer devolve-a, com o mesmo id"
    );
    ctrl_z(&mut hero, &mut text, false);

    right_click(&mut hero, tab_node_id(b[0]));
    repaint(&mut hero, &mut text);
    click_anywhere(&mut hero, ids::CTX_MENU_BOARD_DUPLICATE);
    repaint(&mut hero, &mut text);
    assert_eq!(order(&hero).len(), 3);
    ctrl_z(&mut hero, &mut text, false);
    assert_eq!(order(&hero), b, "a cópia sai");
    assert_eq!(hero.documents.active(), Some(b[1]));

    let from = find(&hero, tab_node_id(b[0])).unwrap();
    let to = find(&hero, tab_node_id(b[1])).unwrap();
    drag(&mut hero, from, (to.0 + 4.0, to.1));
    repaint(&mut hero, &mut text);
    assert_eq!(order(&hero), vec![b[1], b[0]]);
    ctrl_z(&mut hero, &mut text, false);
    assert_eq!(order(&hero), b, "a ordem de antes do arrasto");
    ctrl_z(&mut hero, &mut text, true);
    assert_eq!(order(&hero), vec![b[1], b[0]]);
}

/// O tldraw (`paginas_P9`): desfazer o apagar e DESENHAR no quadro que voltou mata o refazer — senão o
/// `Ctrl+Shift+Z` apagava o trabalho novo.
#[test]
fn a_tab_redo_dies_when_the_board_changes_after_it() {
    let (mut hero, mut text, b) = with_boards(2);
    delete(&mut hero, &mut text, b[1]);
    ctrl_z(&mut hero, &mut text, false);
    assert!(hero.documents.tab_redo_at().is_some());
    let el = draw(&mut hero);
    assert_eq!(hero.documents.tab_redo_at(), None);
    ctrl_z(&mut hero, &mut text, true);
    assert_eq!(order(&hero), b, "o quadro fica");
    assert!(has(&hero, b[1], el));
}

/// O tldraw (`paginas_P11b`): um gesto que não muda nada não é passo — senão o `Ctrl+Z` gasta-se a
/// desfazer o nada.
#[test]
fn a_rename_to_the_same_name_or_a_nudge_is_not_a_step() {
    let (mut hero, _text, b) = with_boards(2);
    let before = hero.documents.tab_undo_at();
    assert!(before.is_some(), "criar é passo");
    let name = name_of(&hero, b[0]);
    assert!(hero.documents.rename(b[0], name));
    assert!(hero.documents.move_tab(b[0], 0), "já está na posição 0");
    assert_eq!(hero.documents.tab_undo_at(), before);
}
