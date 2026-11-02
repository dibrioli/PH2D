//! W4 do Kanban — o Kanban do quadro pelo despacho REAL: o botão «Kanban» cria o documento no
//! `KanbanSet` partilhado e a referência; apagar a referência nunca apaga o documento; «Open in tab»
//! pede a aba (ou diz porquê não); com o gancho da família, o 2.º clique no Kanban seleccionado entra
//! nele e o rato e as teclas são do gancho até `Esc`.

use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::doc_surface::{Key, KeyInput, KeyOutcome, Surfaces};
use crate::documents::kanban_tabs::{self as tabs, NO_TAB, install_tab_opener, take_open_requests};
use crate::screens::hero::board_video::kanban::{self as kb, KanbanElementInput, KanbanPointer};
use ph2d_board_model::{ElementId, KanbanBoardView, KanbanId};

fn kanban_el(t: &T) -> Option<Element> {
    t.elements().into_iter().find(|e| e.kanban().is_some())
}

fn click(t: &mut T, (x, y): (f32, f32)) {
    let p = PointerButton::Primary;
    let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
    board_view::pointer(h, inp(ts, c), PointerKind::Down, p, x, y, true);
    *c += 80_000_000;
    board_view::pointer(h, inp(ts, c), PointerKind::Up, p, x, y, true);
    // Longe do duplo clique.
    *c += 1_000_000_000;
    t.paint();
}

/// O ponto de ecrã do meio do Kanban `id`, pelo que o desenho registou.
fn middle(t: &mut T, id: ElementId) -> (f32, f32) {
    let el = kanban_el(t).unwrap();
    let pl = t.hero.documents.live.render_cache.kanbans.on_screen[&id];
    let p = pl.to_screen * ph2d_vector::Point::new(el.w / 2.0, el.h / 2.0);
    (p.x as f32, p.y as f32)
}

/// ⭐ O botão «Kanban» põe um documento NOVO no conjunto partilhado (o mesmo `Rc` da aba) e a
/// referência a ele, seleccionada; apagar a referência (e desfazer) nunca toca no documento.
#[test]
fn the_kanban_button_makes_a_document_and_deleting_its_reference_keeps_it() {
    let mut t = T::new();
    let shared = t.hero.documents.kanbans_shared();
    t.click_bar(Item::Kanban);
    let el = kanban_el(&t).expect("o elemento");
    let k = el.kanban().unwrap().kanban;
    assert!(
        shared.borrow().get(k).is_some(),
        "o documento no Rc partilhado"
    );
    assert_eq!(
        t.hero.documents.kanbans().get(k).unwrap().header.name,
        "Kanban"
    );
    assert!(t.key(BoardKey::Delete, Modifiers::default(), None));
    assert!(kanban_el(&t).is_none(), "a referência saiu");
    assert!(shared.borrow().get(k).is_some(), "o documento ficou");
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert_eq!(kanban_el(&t).map(|e| e.kanban().unwrap().kanban), Some(k));
}

/// «Open in tab» sem quem abra abas de Kanban diz porquê (aviso, botão apagado); com ele, pede a aba.
#[test]
fn open_in_tab_tells_why_without_an_opener_and_asks_for_the_tab_with_one() {
    let mut t = T::new();
    t.click_bar(Item::Kanban);
    let k = kanban_el(&t).unwrap().kanban().unwrap().kanban;
    t.click_bar(Item::OpenKanban);
    assert!(take_open_requests().is_empty());
    assert_eq!(t.hero.documents.live.notice.map(|n| n.0), Some(NO_TAB));
    install_tab_opener();
    t.click_bar(Item::OpenKanban);
    assert_eq!(take_open_requests(), vec![k]);
}

/// O duplo clique no CABEÇALHO pede a aba.
#[test]
fn a_double_click_on_the_header_asks_for_the_tab() {
    let mut t = T::new();
    install_tab_opener();
    t.click_bar(Item::Kanban);
    let el = kanban_el(&t).unwrap();
    let pl = t.hero.documents.live.render_cache.kanbans.on_screen[&el.id];
    let top = pl.to_screen * ph2d_vector::Point::new(el.w / 2.0, 4.0);
    let (x, y) = (top.x as f32, top.y as f32);
    let p = PointerButton::Primary;
    // Dois carregares a 100 ms (o `inp` dos outros anda um segundo por evento: nunca duplo).
    for i in 0..4u128 {
        let kind = if i % 2 == 0 {
            PointerKind::Down
        } else {
            PointerKind::Up
        };
        let input = Input {
            text: &mut t.ts,
            mods: Modifiers::default(),
            now_ns: t.clock + i * 50_000_000,
        };
        board_view::pointer(&mut t.hero, input, kind, p, x, y, true);
    }
    assert_eq!(take_open_requests(), vec![el.kanban().unwrap().kanban]);
}

#[derive(Default)]
struct Spy(Vec<String>);

impl KanbanElementInput for Spy {
    fn pointer(
        &mut self,
        _: &mut Surfaces,
        _: &mut TextSystem,
        _: (ElementId, KanbanId),
        ev: KanbanPointer,
        v: &mut KanbanBoardView,
    ) -> bool {
        let name = match ev {
            KanbanPointer::Down { .. } => "down",
            KanbanPointer::Move { .. } => "move",
            KanbanPointer::Up { .. } => "up",
        };
        self.0.push(name.into());
        v.scroll_x += 1.0;
        true
    }

    fn key(
        &mut self,
        _: &mut Surfaces,
        _: &mut TextSystem,
        _: (ElementId, KanbanId),
        input: KeyInput<'_>,
        _: &mut KanbanBoardView,
    ) -> KeyOutcome {
        if input.pressed {
            self.0.push(format!("{:?}", input.key));
        }
        KeyOutcome {
            consumed: input.key != Key::Escape,
            copy: None,
        }
    }
}

/// ⭐ Com o gancho: o 1.º clique selecciona, o 2.º entra (o rato vai ao gancho e o Kanban não se
/// move); as teclas são dele; o `Esc` que ele não toma sai; clicar fora também.
#[test]
fn a_second_click_enters_the_kanban_and_the_hook_takes_the_mouse_and_keys() {
    let mut t = T::new();
    let spy = Rc::new(RefCell::new(Spy::default()));
    kb::install_kanban_input(spy.clone());
    t.click_bar(Item::Kanban);
    let el = kanban_el(&t).unwrap();
    let at = middle(&mut t, el.id);
    click(&mut t, at);
    let frames = |t: &T| t.hero.documents.live.render_cache.kanbans.active;
    assert_eq!(frames(&t), Some(el.id));
    assert_eq!(spy.borrow().0, ["down", "up"]);
    let now = kanban_el(&t).unwrap();
    assert_eq!([now.x, now.y], [el.x, el.y], "o clique não move o Kanban");
    assert_eq!(
        now.kanban().unwrap().view.scroll_x,
        2.0,
        "a vista é do gancho"
    );
    assert!(t.key(BoardKey::Char('a'), Modifiers::default(), Some("a")));
    assert!(t.key(BoardKey::Escape, Modifiers::default(), None));
    assert_eq!(frames(&t), None, "o Esc sai");
    assert_eq!(spy.borrow().0[2..], ["Char('a')", "Escape"]);
    click(&mut t, at);
    assert_eq!(frames(&t), Some(el.id));
    let out = t.at(0.02, 0.98);
    click(&mut t, out);
    assert_eq!(frames(&t), None, "clicar fora sai");
}

/// ⭐ Desfazer «inserir Kanban» tira só a REFERÊNCIA: o documento, mesmo intocado e vazio, fica no
/// projecto (e nos Assets) até ser apagado — dono 30/10 (D17). Refazer repõe a referência ao MESMO.
#[test]
fn undoing_the_kanban_insert_keeps_the_document_in_the_project() {
    let mut t = T::new();
    let shared = t.hero.documents.kanbans_shared();
    t.click_bar(Item::Kanban);
    let k = kanban_el(&t).unwrap().kanban().unwrap().kanban;
    let pristine = shared.borrow().get(k).cloned();
    assert!(pristine.is_some());
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert!(kanban_el(&t).is_none(), "a referência saiu do quadro");
    assert_eq!(
        shared.borrow().get(k).cloned(),
        pristine,
        "o documento intocado FICA"
    );
    assert!(t.key(BoardKey::Char('y'), CTRL, None));
    assert_eq!(kanban_el(&t).map(|e| e.kanban().unwrap().kanban), Some(k));
    assert_eq!(shared.borrow().get(k).cloned(), pristine);
}

/// O botão direito num Kanban abre o menu dele, com «Open in tab» à cabeça (e a linha pede a aba).
#[test]
fn the_right_click_menu_of_a_kanban_opens_it_in_a_tab() {
    let mut t = T::new();
    install_tab_opener();
    t.click_bar(Item::Kanban);
    let el = kanban_el(&t).unwrap();
    let at = middle(&mut t, el.id);
    let p = PointerButton::Secondary;
    let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
    board_view::pointer(h, inp(ts, c), PointerKind::Down, p, at.0, at.1, true);
    let kind = t.hero.store.context_menu().map(|m| m.kind);
    assert_eq!(kind, Some(crate::interaction::ContextMenuKind::BoardKanban));
    let open = crate::ids::CTX_MENU_BOARD_EL_OPEN_TAB;
    t.hero
        .apply_event(crate::interaction::WidgetEvent::Click(open));
    assert_eq!(take_open_requests(), vec![el.kanban().unwrap().kanban]);
}

/// ⭐ D17 — um Kanban largado dos Assets sobre o quadro nasce com o meio no ponto do ecrã; o `Ctrl+Z`
/// tira só a referência (o documento fica no projecto).
#[test]
fn a_kanban_dropped_from_the_assets_lands_on_the_point_and_undo_keeps_the_document() {
    let mut t = T::new();
    let shared = t.hero.documents.kanbans_shared();
    let k = shared.borrow_mut().create("K");
    let p = t.at(0.3, 0.4);
    let el = kb::link::place_at(&mut t.hero, k.0, [p.0.into(), p.1.into()]).unwrap();
    t.paint();
    let e = kanban_el(&t).unwrap();
    assert_eq!((e.id, e.kanban().unwrap().kanban), (el, k));
    let b = t.hero.documents.active_board().unwrap();
    let area = board_view::area_of(board_view::area(t.hero.last_layout.as_ref().unwrap()));
    let c = b.camera.to_screen(area, e.center());
    assert!((c[0] - f64::from(p.0)).abs() < 1e-3 && (c[1] - f64::from(p.1)).abs() < 1e-3);
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert!(kanban_el(&t).is_none(), "a referência saiu");
    assert!(shared.borrow().get(k).is_some(), "o documento FICA");
}

/// ⭐ D17 — apagar um Kanban (a pergunta dos Assets → Delete) tira-o do projecto, da lista dos Assets
/// e dos quadros, e fecha a aba; o `Ctrl+Z` devolve o documento e a referência com as mesmas chaves.
#[test]
fn deleting_a_kanban_takes_its_references_and_undo_brings_both_back() {
    let mut t = T::new();
    t.click_bar(Item::Kanban);
    let before = kanban_el(&t).unwrap();
    let k = before.kanban().unwrap().kanban;
    let doc = t.hero.documents.kanbans().get(k).cloned();
    assert_eq!(kb::link::boards_showing(&t.hero.documents, k), 1);
    let _ = tabs::take_close_requests();
    t.hero
        .store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::ConfirmDeleteKanban { kanban: k.0 },
        });
    let confirm = WidgetEvent::Click(crate::ids::CTX_MENU_BOARD_DELETE_CONFIRM);
    assert!(crate::screens::hero::document_tabs_menu::apply_event(
        &mut t.hero,
        confirm
    ));
    let listed =
        |t: &T| {
            t.hero.documents.asset_listing().iter().any(|(p, _)| {
                *p == crate::interaction::drag_payload::DragPayload::Kanban { id: k.0 }
            })
        };
    assert!(t.hero.documents.kanbans().get(k).is_none());
    assert!(!listed(&t), "saiu dos Assets");
    assert!(kanban_el(&t).is_none(), "a referência saiu com ele");
    assert_eq!(tabs::take_close_requests(), vec![k], "a aba fecha");
    assert!(t.hero.documents.undo_tab(false).is_some());
    assert_eq!(t.hero.documents.kanbans().get(k).cloned(), doc);
    assert!(listed(&t));
    let after = kanban_el(&t).expect("a referência voltou");
    let shape = |e: &Element| (e.id, e.kanban().map(|k| k.kanban), [e.x, e.y, e.w, e.h]);
    assert_eq!(
        shape(&after),
        shape(&before),
        "a mesma referência, a mesma chave"
    );
}

/// ⭐ D17 — um Quadro apagado pelos Assets usa a pergunta e o `Documents::remove` da aba: o `Ctrl+Z`
/// devolve-o.
#[test]
fn deleting_a_board_from_the_assets_is_the_tab_delete_and_undo_restores_it() {
    let mut t = T::new();
    let b = t.hero.documents.active().unwrap();
    t.hero
        .store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::ConfirmDeleteBoard { board: b.0 },
        });
    let confirm = WidgetEvent::Click(crate::ids::CTX_MENU_BOARD_DELETE_CONFIRM);
    assert!(crate::screens::hero::document_tabs_menu::apply_event(
        &mut t.hero,
        confirm
    ));
    assert!(t.hero.documents.boards().get(b).is_none());
    assert!(t.hero.documents.undo_tab(false).is_some());
    assert!(t.hero.documents.boards().get(b).is_some());
}

/// «Delete» no menu de um Kanban nos Assets, confirmado (o caminho real da pergunta).
fn delete_from_assets(t: &mut T, k: KanbanId) {
    t.hero
        .store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::ConfirmDeleteKanban { kanban: k.0 },
        });
    let confirm = WidgetEvent::Click(crate::ids::CTX_MENU_BOARD_DELETE_CONFIRM);
    assert!(crate::screens::hero::document_tabs_menu::apply_event(
        &mut t.hero,
        confirm
    ));
    t.paint();
}

/// `Ctrl+Z` (ou `Ctrl+Y`) pelo teclado do quadro activo.
fn undo_key(t: &mut T, redo: bool) {
    let c = if redo { 'y' } else { 'z' };
    assert!(t.key(BoardKey::Char(c), CTRL, None), "Ctrl+{c}");
}

/// A posição da referência a `k` no quadro `b` (`None` = não está lá).
fn ref_at(t: &T, b: ph2d_board_model::BoardId, k: KanbanId) -> Option<[f64; 2]> {
    let board = t.hero.documents.boards().get(b)?;
    let el = board
        .doc
        .live()
        .find(|e| e.kanban().is_some_and(|r| r.kanban == k))?;
    Some([el.x, el.y])
}

/// ⭐ D17 — pôr, mover, apagar o Kanban pelos Assets; cada `Ctrl+Z` desfaz UM passo pela ordem
/// inversa (o documento e a referência, onde foi movida; o mover; o pôr — o documento fica) e cada
/// `Ctrl+Y` refá-lo até ao apagado.
#[test]
fn a_moved_kanban_deleted_from_the_assets_undoes_and_redoes_step_by_step() {
    let mut t = T::new();
    let b = t.hero.documents.active().unwrap();
    t.click_bar(Item::Kanban);
    let k = kanban_el(&t).unwrap().kanban().unwrap().kanban;
    let born = ref_at(&t, b, k).unwrap();
    assert!(t.key(BoardKey::Right, Modifiers::default(), None));
    let moved = ref_at(&t, b, k).unwrap();
    assert_ne!(moved, born, "o empurrão moveu");
    delete_from_assets(&mut t, k);
    let has_doc = |t: &T| t.hero.documents.kanbans().get(k).is_some();
    assert!(!has_doc(&t) && ref_at(&t, b, k).is_none());
    undo_key(&mut t, false);
    assert!(has_doc(&t), "o documento voltou");
    assert_eq!(
        ref_at(&t, b, k),
        Some(moved),
        "a referência, onde foi movida"
    );
    undo_key(&mut t, false);
    assert_eq!(ref_at(&t, b, k), Some(born), "o mover desfez-se");
    undo_key(&mut t, false);
    assert_eq!(ref_at(&t, b, k), None, "o pôr desfez-se");
    assert!(has_doc(&t), "o documento fica no projecto");
    undo_key(&mut t, true);
    assert_eq!(ref_at(&t, b, k), Some(born));
    undo_key(&mut t, true);
    assert_eq!(ref_at(&t, b, k), Some(moved));
    undo_key(&mut t, true);
    assert!(
        !has_doc(&t) && ref_at(&t, b, k).is_none(),
        "apagado outra vez"
    );
    undo_key(&mut t, true);
    assert!(
        !has_doc(&t) && ref_at(&t, b, k).is_none(),
        "nada mais a refazer"
    );
}

/// ⭐ D17 — o Kanban em dois quadros, apagado; uma nota no quadro A depois: o 1.º `Ctrl+Z` em A tira a
/// nota, o 2.º devolve o Kanban e as referências em A e em B.
#[test]
fn a_kanban_on_two_boards_deleted_then_a_note_undoes_note_first_then_both_references() {
    let mut t = T::new();
    let a = t.hero.documents.active().unwrap();
    t.click_bar(Item::Kanban);
    let k = kanban_el(&t).unwrap().kanban().unwrap().kanban;
    let bb = t.hero.documents.create("B".into());
    crate::screens::hero::document_tabs::register_board(&mut t.hero.store, bb);
    t.paint();
    assert!(kb::link::place(&mut t.hero, k).is_some_and(|(b, _)| b == bb));
    let (on_a, on_b) = (ref_at(&t, a, k).unwrap(), ref_at(&t, bb, k).unwrap());
    delete_from_assets(&mut t, k);
    assert!(ref_at(&t, a, k).is_none() && ref_at(&t, bb, k).is_none());
    t.hero.documents.activate(Some(a));
    t.paint();
    let note = t.put_note([-400.0, -400.0], "after", 0);
    undo_key(&mut t, false);
    let on = |t: &T| {
        t.hero
            .documents
            .boards()
            .get(a)
            .unwrap()
            .doc
            .get(note)
            .is_some()
    };
    assert!(!on(&t), "a nota saiu primeiro");
    assert!(t.hero.documents.kanbans().get(k).is_none());
    undo_key(&mut t, false);
    assert!(
        t.hero.documents.kanbans().get(k).is_some(),
        "o documento voltou"
    );
    assert_eq!(ref_at(&t, a, k), Some(on_a));
    assert_eq!(
        ref_at(&t, bb, k),
        Some(on_b),
        "e a referência no outro quadro"
    );
    undo_key(&mut t, true);
    assert!(ref_at(&t, a, k).is_none() && ref_at(&t, bb, k).is_none());
    undo_key(&mut t, true);
    assert!(on(&t), "a nota volta");
}

fn doc(t: &mut T) -> &mut ph2d_board_model::BoardDoc {
    &mut t.hero.documents.active_board_mut().unwrap().doc
}

/// ⭐ D17 — num quadro ao vivo, o par recebe a referência a sair (apagar o Kanban) e a voltar
/// (`Ctrl+Z`): as mudanças da sessão levam-na, e a `version` cresce sempre (o par aceita-a).
#[test]
fn a_live_board_sends_the_reference_leaving_and_coming_back() {
    let mut t = T::new();
    let b = t.hero.documents.active().unwrap();
    t.click_bar(Item::Kanban);
    let el = kanban_el(&t).unwrap();
    let k = el.kanban().unwrap().kanban;
    assert_eq!(t.hero.documents.active(), Some(b));
    doc(&mut t).set_session(Some(7));
    let _ = doc(&mut t).take_changes();
    delete_from_assets(&mut t, k);
    let sent = doc(&mut t).take_changes();
    assert!(sent.iter().any(|(id, _)| *id == el.id), "o par vê-a sair");
    let gone = doc(&mut t).get_any(el.id).cloned().unwrap();
    assert!(gone.deleted && gone.version > el.version);
    undo_key(&mut t, false);
    let sent = doc(&mut t).take_changes();
    assert!(sent.iter().any(|(id, _)| *id == el.id), "o par vê-a voltar");
    let back = doc(&mut t).get_any(el.id).cloned().unwrap();
    assert!(!back.deleted && back.version > gone.version);
}

/// ⭐ D17 — pôr o Kanban e desfazê-lo (o documento fica, o refazer espera), apagá-lo pelos Assets:
/// o `Ctrl+Y` do quadro já não ressuscita uma referência a um Kanban que não existe (apagar é um
/// gesto NOVO); o `Ctrl+Z` devolve o documento.
#[test]
fn a_board_redo_never_brings_back_a_reference_to_a_deleted_kanban() {
    let mut t = T::new();
    let b = t.hero.documents.active().unwrap();
    t.click_bar(Item::Kanban);
    let k = kanban_el(&t).unwrap().kanban().unwrap().kanban;
    undo_key(&mut t, false);
    assert!(ref_at(&t, b, k).is_none());
    delete_from_assets(&mut t, k);
    undo_key(&mut t, true);
    let has_doc = t.hero.documents.kanbans().get(k).is_some();
    assert!(
        has_doc || ref_at(&t, b, k).is_none(),
        "uma referência órfã voltou"
    );
    undo_key(&mut t, false);
    assert!(
        t.hero.documents.kanbans().get(k).is_some(),
        "o documento volta"
    );
}

#[path = "board_kanban_orphan_tests.rs"]
mod orphan;
