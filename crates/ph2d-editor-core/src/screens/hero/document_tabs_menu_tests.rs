//! O que se faz a uma aba de quadro (W0b), pelo caminho REAL: ponteiro e teclado pelo despacho
//! do hero, cada evento pelo `apply_event`, e o ecrã repintado entre gestos (o que o artista vê).

use super::tests::{click, find, painted, ptr, repaint};
use super::*;
use crate::interaction::{ContextMenuKind, KEY_ENTER, KEY_ESCAPE};
use crate::screens::hero::document_tabs_menu;
use bumpalo::Bump;
use ph2d_host::{KeyEvent, KeyKind, Modifiers, PointerButton, PointerKind};

fn apply(hero: &mut HeroScreen, events: Vec<WidgetEvent>) {
    for e in events {
        hero.apply_event(e);
    }
}

fn key(hero: &mut HeroScreen, keycode: u32) {
    let arena = Bump::new();
    let e = KeyEvent {
        keycode,
        modifiers: Modifiers::default(),
        kind: KeyKind::Down,
        timestamp_ns: 0,
    };
    let events = hero.handle_key(e, &arena).to_vec();
    apply(hero, events);
}

fn type_text(hero: &mut HeroScreen, s: &str) {
    for ch in s.chars() {
        let arena = Bump::new();
        let events = hero.handle_text_input(ch, &arena).to_vec();
        apply(hero, events);
    }
}

/// O centro do alvo `id` em qualquer ponto do ecrã (passo de 2 px).
fn locate(hero: &HeroScreen, id: NodeId) -> Option<(f32, f32)> {
    let vp = hero.last_viewport;
    let mut hits = Vec::new();
    let mut y = vp.y;
    while y < vp.y + vp.h {
        let mut x = vp.x;
        while x < vp.x + vp.w {
            if hero.hit_index.hit(x, y) == Some(id) {
                hits.push((x, y));
            }
            x += 2.0;
        }
        y += 2.0;
    }
    let n = hits.len() as f32;
    (!hits.is_empty()).then(|| {
        let sx: f32 = hits.iter().map(|p| p.0).sum();
        let sy: f32 = hits.iter().map(|p| p.1).sum();
        (sx / n, sy / n)
    })
}

/// Down + Up do botão principal sobre `id`, onde quer que ele esteja pintado.
fn click_anywhere(hero: &mut HeroScreen, id: NodeId) {
    let (x, y) = locate(hero, id).unwrap_or_else(|| panic!("{id:?} não está pintado"));
    let arena = Bump::new();
    let mut events = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    events.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    apply(hero, events);
}

fn right_click(hero: &mut HeroScreen, id: NodeId) {
    let (x, y) = find(hero, id).expect("a aba não está na barra");
    let arena = Bump::new();
    let mut events = Vec::new();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let mut e = ptr(kind, x, y);
        e.button = PointerButton::Secondary;
        events.extend_from_slice(hero.handle_pointer(e, &arena));
    }
    apply(hero, events);
}

fn name_of(hero: &HeroScreen, b: BoardId) -> String {
    hero.documents
        .boards()
        .get(b)
        .expect("o quadro existe")
        .name
        .clone()
}

/// `n` quadros novos, a cena repintada.
fn with_boards(n: usize) -> (HeroScreen, TextSystem, Vec<BoardId>) {
    let (mut hero, mut text) = painted();
    for _ in 0..n {
        click(&mut hero, ids::DOC_TAB_NEW);
        repaint(&mut hero, &mut text);
    }
    let ids = hero
        .documents
        .boards()
        .boards()
        .iter()
        .map(|b| b.id)
        .collect();
    (hero, text, ids)
}

#[test]
fn double_click_renames_in_place_and_enter_keeps_the_name() {
    let (mut hero, mut text, b) = with_boards(1);
    click(&mut hero, tab_node_id(b[0]));
    click(&mut hero, tab_node_id(b[0]));
    assert_eq!(
        hero.documents.renaming,
        Some(b[0]),
        "o duplo-clique não abriu o campo"
    );
    repaint(&mut hero, &mut text);
    assert!(
        find(&hero, ids::DOC_TAB_RENAME_INPUT).is_some(),
        "o campo não está no lugar da aba"
    );
    // O nome inteiro vem seleccionado: escrever substitui-o.
    type_text(&mut hero, "Retro");
    key(&mut hero, KEY_ENTER);
    assert_eq!(name_of(&hero, b[0]), "Retro");
    assert_eq!(hero.documents.renaming, None);
    repaint(&mut hero, &mut text);
    assert!(find(&hero, tab_node_id(b[0])).is_some(), "a aba não voltou");
}

#[test]
fn escape_gives_up_and_an_empty_name_is_not_kept() {
    let (mut hero, mut text, b) = with_boards(1);
    document_tabs_menu::begin_rename(&mut hero, b[0]);
    repaint(&mut hero, &mut text);
    type_text(&mut hero, "Nope");
    key(&mut hero, KEY_ESCAPE);
    assert_eq!(name_of(&hero, b[0]), "Board 1", "o Esc gravou");
    assert_eq!(hero.documents.renaming, None);

    document_tabs_menu::begin_rename(&mut hero, b[0]);
    type_text(&mut hero, "   ");
    key(&mut hero, KEY_ENTER);
    assert_eq!(
        name_of(&hero, b[0]),
        "Board 1",
        "um nome em branco foi gravado"
    );
}

#[test]
fn clicking_elsewhere_keeps_what_was_typed() {
    let (mut hero, mut text, b) = with_boards(1);
    document_tabs_menu::begin_rename(&mut hero, b[0]);
    repaint(&mut hero, &mut text);
    type_text(&mut hero, "Ideas");
    click(&mut hero, ids::DOC_TAB_SCENE);
    assert_eq!(name_of(&hero, b[0]), "Ideas");
    assert_eq!(
        hero.documents.active(),
        None,
        "o clique fora também troca de aba"
    );
}

#[test]
fn right_click_opens_the_tab_menu_and_duplicate_copies_it_beside() {
    let (mut hero, mut text, b) = with_boards(2);
    right_click(&mut hero, tab_node_id(b[0]));
    assert_eq!(
        hero.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::BoardTab { board: b[0].0 })
    );
    repaint(&mut hero, &mut text);
    click_anywhere(&mut hero, ids::CTX_MENU_BOARD_DUPLICATE);
    let list = hero.documents.boards().boards();
    assert_eq!(list.len(), 3);
    assert_eq!(list[1].name, "Board 1 copy", "a cópia não está à direita");
    assert_eq!(
        hero.documents.active(),
        Some(list[1].id),
        "a cópia não abriu"
    );
    let copy = list[1].id;
    repaint(&mut hero, &mut text);
    click(&mut hero, tab_node_id(b[1]));
    repaint(&mut hero, &mut text);
    click(&mut hero, tab_node_id(copy));
    assert_eq!(
        hero.documents.active(),
        Some(copy),
        "a aba da cópia nasceu morta"
    );
}

#[test]
fn the_menus_rename_opens_the_same_field() {
    let (mut hero, mut text, b) = with_boards(2);
    right_click(&mut hero, tab_node_id(b[1]));
    repaint(&mut hero, &mut text);
    click_anywhere(&mut hero, ids::CTX_MENU_BOARD_RENAME);
    assert_eq!(hero.documents.renaming, Some(b[1]));
    type_text(&mut hero, "Plan");
    key(&mut hero, KEY_ENTER);
    assert_eq!(name_of(&hero, b[1]), "Plan");
}

#[test]
fn delete_asks_first_and_cancel_keeps_the_board() {
    let (mut hero, mut text, b) = with_boards(2);
    right_click(&mut hero, tab_node_id(b[0]));
    repaint(&mut hero, &mut text);
    click_anywhere(&mut hero, ids::CTX_MENU_BOARD_DELETE);
    assert_eq!(
        hero.documents.boards().boards().len(),
        2,
        "apagou sem perguntar"
    );
    assert_eq!(
        hero.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::ConfirmDeleteBoard { board: b[0].0 })
    );
    repaint(&mut hero, &mut text);
    click_anywhere(&mut hero, ids::CTX_MENU_BOARD_DELETE_CANCEL);
    assert_eq!(hero.documents.boards().boards().len(), 2);
    assert!(hero.store.context_menu().is_none(), "a pergunta não fechou");

    repaint(&mut hero, &mut text);
    right_click(&mut hero, tab_node_id(b[0]));
    repaint(&mut hero, &mut text);
    click_anywhere(&mut hero, ids::CTX_MENU_BOARD_DELETE);
    repaint(&mut hero, &mut text);
    click_anywhere(&mut hero, ids::CTX_MENU_BOARD_DELETE_CONFIRM);
    let left: Vec<BoardId> = hero
        .documents
        .boards()
        .boards()
        .iter()
        .map(|x| x.id)
        .collect();
    assert_eq!(left, vec![b[1]]);
    assert!(hero.store.context_menu().is_none());
}

#[test]
fn deleting_the_open_board_opens_its_neighbour() {
    let mut d = crate::documents::Documents::default();
    let a = d.create("A".into());
    let b = d.create("B".into());
    let c = d.create("C".into());
    let last = d.create("D".into());
    d.activate(Some(b));
    d.remove(b);
    // ⚠️ Com quatro: a vizinha (C) não é a última (D) — com três as duas respostas coincidiam.
    assert_eq!(d.active(), Some(c), "a da direita devia abrir");
    d.remove(last);
    d.remove(c);
    assert_eq!(d.active(), Some(a), "sem direita, a da esquerda");
    d.remove(a);
    assert_eq!(d.active(), None, "sem quadros, a cena");
    let x = d.create("X".into());
    let y = d.create("Y".into());
    d.activate(Some(x));
    d.remove(y);
    assert_eq!(d.active(), Some(x), "apagar outra aba não muda a aberta");
}

/// Down na aba `from`, Move até `to`, Up — o gesto inteiro pelo hero.
fn drag(hero: &mut HeroScreen, from: (f32, f32), to: (f32, f32)) {
    let arena = Bump::new();
    let mut events = hero
        .handle_pointer(ptr(PointerKind::Down, from.0, from.1), &arena)
        .to_vec();
    events.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Move, to.0, to.1), &arena));
    events.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, to.0, to.1), &arena));
    apply(hero, events);
}

fn order(hero: &HeroScreen) -> Vec<BoardId> {
    hero.documents
        .boards()
        .boards()
        .iter()
        .map(|x| x.id)
        .collect()
}

#[test]
fn dragging_a_tab_reorders_it_and_a_nudge_is_still_a_click() {
    let (mut hero, mut text, b) = with_boards(3);
    click(&mut hero, ids::DOC_TAB_SCENE);
    repaint(&mut hero, &mut text);
    let a = find(&hero, tab_node_id(b[0])).unwrap();
    let c = find(&hero, tab_node_id(b[2])).unwrap();
    // Largar passado o meio da última: vai para o fim, e fica aberta.
    drag(&mut hero, a, (c.0 + 4.0, c.1));
    assert_eq!(order(&hero), vec![b[1], b[2], b[0]]);
    assert_eq!(hero.documents.active(), Some(b[0]));

    // Um empurrão abaixo do limiar não reordena — é o clique que troca de aba.
    repaint(&mut hero, &mut text);
    let mid = find(&hero, tab_node_id(b[2])).unwrap();
    drag(&mut hero, mid, (mid.0 + 2.0, mid.1));
    assert_eq!(order(&hero), vec![b[1], b[2], b[0]]);
    assert_eq!(hero.documents.active(), Some(b[2]));

    // Largar longe da barra é desistir.
    repaint(&mut hero, &mut text);
    let first = find(&hero, tab_node_id(b[1])).unwrap();
    drag(&mut hero, first, (first.0 + 200.0, first.1 + 300.0));
    assert_eq!(order(&hero), vec![b[1], b[2], b[0]]);
}

#[test]
fn the_drop_mark_is_drawn_from_the_same_decision() {
    let (mut hero, mut text, b) = with_boards(3);
    repaint(&mut hero, &mut text);
    let bar = hero.last_layout.expect("pintado").top_bar;
    let a = find(&hero, tab_node_id(b[0])).unwrap();
    let c = find(&hero, tab_node_id(b[2])).unwrap();
    let arena = Bump::new();
    let _ = hero.handle_pointer(ptr(PointerKind::Down, a.0, a.1), &arena);
    let _ = hero.handle_pointer(ptr(PointerKind::Move, c.0 + 4.0, c.1), &arena);
    let tabs: Vec<(Target, Rect)> = [b[0], b[1], b[2]]
        .iter()
        .map(|id| {
            let r = hero.hit_index.rect_for(tab_node_id(*id)).unwrap();
            (Target::Board(*id), r)
        })
        .collect();
    let caret = document_tabs_menu::drop_caret(&hero.documents, bar, &tabs)
        .expect("arrasto sobre a barra sem marca");
    let end_of_c = tabs[2].1.x + tabs[2].1.w;
    assert!(
        (caret.x + caret.w * 0.5 - end_of_c).abs() < 0.01,
        "a marca não está no fim"
    );
    let _ = hero.handle_pointer(ptr(PointerKind::Up, c.0 + 4.0, c.1), &arena);
    assert!(
        document_tabs_menu::drop_caret(&hero.documents, bar, &tabs).is_none(),
        "a marca sobreviveu à largada"
    );
}

/// ⚠️ O clique na área do quadro é tomado pelo `board_view::pointer` ANTES do despacho — e tem de
/// largar o teclado do campo como o despacho largaria, senão o nome escrito nunca grava.
#[test]
fn clicking_on_the_board_keeps_what_was_typed() {
    let (mut hero, mut text, b) = with_boards(1);
    document_tabs_menu::begin_rename(&mut hero, b[0]);
    repaint(&mut hero, &mut text);
    type_text(&mut hero, "Sprint");
    let c = hero.last_canvas;
    let (x, y) = (c.x + c.w * 0.5, c.y + c.h * 0.5);
    let input = super::super::board_view::Input {
        text: &mut text,
        mods: ph2d_host::Modifiers::default(),
        now_ns: 0,
    };
    assert!(super::super::board_view::pointer(
        &mut hero,
        input,
        PointerKind::Down,
        PointerButton::Primary,
        x,
        y,
        true,
    ));
    assert_eq!(name_of(&hero, b[0]), "Sprint");
    assert_eq!(hero.store.focus_id(), None, "o campo ficou com o teclado");
}

/// Carregar em *Delete board* e soltar FORA dele é desistir do botão — e a pergunta continua aberta
/// (o Down num botão do diálogo não é um clique fora dele).
#[test]
fn pressing_delete_and_sliding_off_neither_deletes_nor_closes_the_question() {
    let (mut hero, mut text, b) = with_boards(1);
    right_click(&mut hero, tab_node_id(b[0]));
    repaint(&mut hero, &mut text);
    click_anywhere(&mut hero, ids::CTX_MENU_BOARD_DELETE);
    repaint(&mut hero, &mut text);
    let (x, y) = locate(&hero, ids::CTX_MENU_BOARD_DELETE_CONFIRM).expect("botão pintado");
    let arena = Bump::new();
    let mut events = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    events.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y + 200.0), &arena));
    apply(&mut hero, events);
    assert_eq!(
        hero.documents.boards().boards().len(),
        1,
        "apagou sem o clique"
    );
    assert_eq!(
        hero.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::ConfirmDeleteBoard { board: b[0].0 }),
        "a pergunta fechou no Down"
    );
}
