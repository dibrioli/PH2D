//! W4 do Kanban — o Kanban do quadro pelo despacho REAL: o botão «Kanban» cria o documento no
//! `KanbanSet` partilhado e a referência; apagar a referência nunca apaga o documento; «Open in tab»
//! pede a aba (ou diz porquê não); com o gancho da família, o 2.º clique no Kanban seleccionado entra
//! nele e o rato e as teclas são do gancho até `Esc`.

use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::doc_surface::{Key, KeyInput, KeyOutcome, Surfaces};
use crate::screens::hero::board_video::kanban::{
    self as kb, KanbanElementInput, KanbanPointer, NO_TAB,
};
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
    assert!(kb::take_open_requests().is_empty());
    assert_eq!(t.hero.documents.live.notice.map(|n| n.0), Some(NO_TAB));
    kb::install_tab_opener();
    t.click_bar(Item::OpenKanban);
    assert_eq!(kb::take_open_requests(), vec![k]);
}

/// O duplo clique no CABEÇALHO pede a aba.
#[test]
fn a_double_click_on_the_header_asks_for_the_tab() {
    let mut t = T::new();
    kb::install_tab_opener();
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
    assert_eq!(kb::take_open_requests(), vec![el.kanban().unwrap().kanban]);
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

/// ⭐ Desfazer «inserir Kanban» leva o documento que o gesto criou se ele ficou intocado (no MESMO
/// passo); refazer repõe-no com o mesmo id; um documento mexido fica (vive só em aba).
#[test]
fn undoing_the_kanban_insert_takes_the_untouched_document_with_it() {
    let mut t = T::new();
    let shared = t.hero.documents.kanbans_shared();
    t.click_bar(Item::Kanban);
    let k = kanban_el(&t).unwrap().kanban().unwrap().kanban;
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert!(kanban_el(&t).is_none());
    assert!(
        shared.borrow().get(k).is_none(),
        "o órfão intocado saiu com a referência"
    );
    assert!(t.key(BoardKey::Char('y'), CTRL, None));
    assert_eq!(kanban_el(&t).map(|e| e.kanban().unwrap().kanban), Some(k));
    assert!(
        shared.borrow().get(k).is_some(),
        "o refazer repõe o MESMO documento"
    );
    // Mexido (renomeado na aba, por exemplo), já não é só do gesto: o desfazer deixa-o.
    shared.borrow_mut().doc_mut(k).unwrap().header.name = "Mine".into();
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert!(kanban_el(&t).is_none());
    assert_eq!(
        shared.borrow().get(k).map(|k| k.header.name.clone()),
        Some("Mine".into())
    );
}

/// O botão direito num Kanban abre o menu dele, com «Open in tab» à cabeça (e a linha pede a aba).
#[test]
fn the_right_click_menu_of_a_kanban_opens_it_in_a_tab() {
    let mut t = T::new();
    kb::install_tab_opener();
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
    assert_eq!(kb::take_open_requests(), vec![el.kanban().unwrap().kanban]);
}
