//! W11 do Kanban — «Convert to card» pelo caminho REAL: o botão direito numa nota abre o menu dela
//! com a linha à cabeça; a linha abre o modal das listas; a busca filtra; o clique de verdade numa
//! lista converte as notas seleccionadas (e um `Esc` desiste sem tocar em nada).

use super::*;
use crate::screens::hero::board_view::{self, Input};
use crate::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_board_model::{BoardOp, Element, ElementId, RichText, Shape, ShapeType};
use ph2d_host::{
    KeyEvent, KeyKind, Modifiers, PointerButton, PointerEvent, PointerKind, PointerSource,
};
use ph2d_kanban_model::{FracKey, KanbanId, KanbanOp, List, ListId, Meta, Record};

struct T {
    hero: HeroScreen,
    ts: TextSystem,
}

impl T {
    fn new() -> Self {
        crate::test_support::ensure_panel_registry();
        let mut t = Self {
            hero: HeroScreen::new(NodeId(1)),
            ts: TextSystem::without_system_fonts(),
        };
        t.paint();
        t.hero.apply_event(WidgetEvent::Click(ids::DOC_TAB_NEW));
        t.paint();
        t
    }

    fn paint(&mut self) {
        let mut scene = VectorScene::new();
        let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        paint_hero_screen(&mut self.hero, vp, &mut scene, &mut self.ts);
    }

    /// Uma nota `kind` de `text` no centro da câmara (um passo).
    fn note(&mut self, text: &str, kind: ShapeType) -> ElementId {
        let (board, live) = self.hero.documents.active_parts().unwrap();
        let (cx, cy) = (board.camera.center_x, board.camera.center_y);
        let shape = Shape {
            kind,
            style: board_view::default_style(),
            text: RichText::plain(text),
        };
        let id = board.doc.mint_id();
        let bx = [cx - 99.5, cy - 99.5, 199.0, 199.0];
        let el = Element::new_shape(id, board.doc.z_on_top(), shape, bx);
        live.histories
            .entry(board.id)
            .or_default()
            .apply(&mut board.doc, vec![BoardOp::Put(el)]);
        id
    }

    fn alive(&self, id: ElementId) -> bool {
        self.hero
            .documents
            .active_board()
            .unwrap()
            .doc
            .get(id)
            .is_some()
    }

    /// Um Kanban `name` com a lista `title`.
    fn kanban(&mut self, name: &str, title: &str) -> (KanbanId, ListId) {
        let shared = self.hero.documents.kanbans_shared();
        let mut set = shared.borrow_mut();
        let k = set.create(name);
        let id = ListId(set.get(k).unwrap().mint());
        let list = List {
            id,
            title: title.into(),
            pos: FracKey::between(None, None),
            color: None,
            card_limit: None,
            archived: false,
            meta: Meta::default(),
            watchers: Vec::new(),
        };
        set.apply(k, vec![KanbanOp::put(Record::List(list))]);
        (k, id)
    }

    fn titles(&self, k: KanbanId, l: ListId) -> Vec<String> {
        let set = self.hero.documents.kanbans();
        let kb = set.get(k).unwrap();
        kb.cards_in(l).iter().map(|c| c.title.clone()).collect()
    }

    /// O botão direito no meio da nota `id`, pela porta do quadro.
    fn right_click(&mut self, id: ElementId) {
        let canvas = self.hero.last_canvas;
        let board = self.hero.documents.active_board().unwrap().clone();
        let el = board.doc.get(id).unwrap().clone();
        let area = board_view::area_of(canvas);
        let mut at = None;
        let mut y = canvas.y;
        while at.is_none() && y < canvas.y + canvas.h {
            let mut x = canvas.x;
            while x < canvas.x + canvas.w {
                let p = board_view::world_pointer(&board, area, x, y, Modifiers::default());
                let [wx, wy] = [p.world[0], p.world[1]];
                let inside = (wx - (el.x + el.w / 2.0)).abs() < 20.0
                    && (wy - (el.y + el.h / 2.0)).abs() < 20.0;
                if inside {
                    at = Some((x, y));
                    break;
                }
                x += 4.0;
            }
            y += 4.0;
        }
        let (x, y) = at.expect("a nota está à vista");
        let input = Input {
            text: &mut self.ts,
            mods: Modifiers::default(),
            now_ns: 1,
        };
        let b = PointerButton::Secondary;
        board_view::pointer(&mut self.hero, input, PointerKind::Down, b, x, y, true);
        self.paint();
    }

    /// O centro do alvo `id` onde o pintor o registou.
    fn locate(&self, id: NodeId) -> (f32, f32) {
        let vp = self.hero.last_viewport;
        let mut hits = Vec::new();
        let mut y = vp.y;
        while y < vp.y + vp.h {
            let mut x = vp.x;
            while x < vp.x + vp.w {
                if self.hero.hit_index.hit(x, y) == Some(id) {
                    hits.push((x, y));
                }
                x += 2.0;
            }
            y += 2.0;
        }
        assert!(!hits.is_empty(), "{id:?} não está pintado");
        let n = hits.len() as f32;
        let sx: f32 = hits.iter().map(|p| p.0).sum();
        let sy: f32 = hits.iter().map(|p| p.1).sum();
        (sx / n, sy / n)
    }

    /// Down + Up sobre `id` pelo despacho do hero, cada evento pelo `apply_event`.
    fn click(&mut self, id: NodeId) {
        let (x, y) = self.locate(id);
        let ptr = |kind| PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        };
        let arena = Bump::new();
        let mut events = self
            .hero
            .handle_pointer(ptr(PointerKind::Down), &arena)
            .to_vec();
        events.extend_from_slice(self.hero.handle_pointer(ptr(PointerKind::Up), &arena));
        for e in events {
            self.hero.apply_event(e);
        }
        self.paint();
    }

    fn kind(&self) -> Option<ContextMenuKind> {
        self.hero.store.context_menu().map(|m| m.kind)
    }
}

/// ⭐ Botão direito numa nota → «Convert to card» à cabeça → o modal lista «Kanban › lista» → o
/// clique numa lista converte a nota (ela sai, o cartão nasce lá); `Ctrl+Z` não é deste teste.
#[test]
fn the_right_click_menu_of_a_note_converts_it_into_the_chosen_list() {
    let mut t = T::new();
    let (k, l) = t.kanban("Sprint", "To Do");
    let (k2, l2) = t.kanban("Art", "Ideas");
    let n = t.note("Fix jump\nThe hero falls", ShapeType::Sticky);
    t.paint();
    t.right_click(n);
    assert_eq!(t.kind(), Some(ContextMenuKind::BoardNote));
    t.click(ids::CTX_MENU_BOARD_EL_TO_CARD);
    assert_eq!(t.kind(), Some(ContextMenuKind::NotesToCard), "o modal");
    let (rows, more, any) = visible(&t.hero);
    assert!(any && more == 0);
    assert_eq!(rows.len(), 2);
    let slot = rows
        .iter()
        .position(|c| (c.kanban, c.list) == (k2, l2))
        .unwrap();
    t.click(ids::CTX_NOTE_CARD_ROWS[slot]);
    assert_eq!(t.kind(), None, "o modal fecha");
    assert!(!t.alive(n), "a nota saiu do Quadro");
    assert_eq!(t.titles(k2, l2), ["Fix jump"]);
    assert!(t.titles(k, l).is_empty(), "só a lista escolhida");
}

/// A busca filtra pelo nome do Kanban ou da lista; `Esc` desiste e a nota fica. Uma forma que não
/// é nota não oferece a linha (a pilha também não: é a fonte de notas novas).
#[test]
fn the_search_narrows_the_lists_and_escape_leaves_the_note() {
    let mut t = T::new();
    t.kanban("Sprint", "To Do");
    t.kanban("Art", "Ideas");
    let n = t.note("Polish", ShapeType::Sticky);
    t.paint();
    t.right_click(n);
    t.click(ids::CTX_MENU_BOARD_EL_TO_CARD);
    for ch in "idea".chars() {
        let arena = Bump::new();
        for e in t.hero.handle_text_input(ch, &arena).to_vec() {
            t.hero.apply_event(e);
        }
    }
    let (rows, ..) = visible(&t.hero);
    assert_eq!(
        rows.iter()
            .map(|c| c.list_name.as_str())
            .collect::<Vec<_>>(),
        ["Ideas"]
    );
    let arena = Bump::new();
    let esc = KeyEvent {
        keycode: crate::interaction::KEY_ESCAPE,
        modifiers: Modifiers::default(),
        kind: KeyKind::Down,
        timestamp_ns: 0,
    };
    for e in t.hero.handle_key(esc, &arena).to_vec() {
        t.hero.apply_event(e);
    }
    assert_eq!(t.kind(), None);
    assert!(t.alive(n), "desistir não converte");

    let stack = t.note("", ShapeType::StickyStack);
    t.hero.store.close_context_menu();
    t.paint();
    t.right_click(stack);
    assert_ne!(t.kind(), Some(ContextMenuKind::BoardNote), "a pilha não");
}

/// Sem Kanban nenhum, o modal diz o que fazer (e não tem linhas).
#[test]
fn without_a_kanban_the_modal_says_to_create_one() {
    let mut t = T::new();
    let n = t.note("Idea", ShapeType::Sticky);
    t.paint();
    t.right_click(n);
    t.click(ids::CTX_MENU_BOARD_EL_TO_CARD);
    let (rows, _, any) = visible(&t.hero);
    assert!(rows.is_empty() && !any);
    assert!(t.alive(n));
}
