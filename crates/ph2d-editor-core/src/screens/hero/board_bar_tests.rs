//! O quadro pelo caminho REAL do ecrã (W1): o clique nas barras passa pelo despacho sobre o que o
//! pintor registou, o rato no quadro pela porta da shell (`board_view::pointer`), as teclas pela
//! porta do teclado (`board_keys::key`) — e o ecrã é repintado entre gestos.

use super::*;
use crate::ids;
use crate::screens::hero::board_keys::{self, BoardKey};
use crate::screens::hero::board_view::{self, Input};
use crate::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use bumpalo::Bump;
use ph2d_board_model::{Dash, Element};
use ph2d_host::{Modifiers, PointerButton, PointerEvent, PointerKind, PointerSource};

struct T {
    hero: HeroScreen,
    ts: TextSystem,
    clock: u128,
}

fn ptr(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

impl T {
    fn new() -> Self {
        crate::test_support::ensure_panel_registry();
        let mut t = Self {
            hero: HeroScreen::new(NodeId(1)),
            ts: TextSystem::without_system_fonts(),
            clock: 1_000_000_000_000,
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

    /// O centro de `id` no ecrã (o que o pintor registou).
    fn find(&self, id: NodeId) -> Option<(f32, f32)> {
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
        let n = hits.len() as f32;
        (!hits.is_empty()).then(|| {
            (
                hits.iter().map(|p| p.0).sum::<f32>() / n,
                hits.iter().map(|p| p.1).sum::<f32>() / n,
            )
        })
    }

    /// Down + Up pelo despacho real sobre o controlo `item` das barras.
    fn click_bar(&mut self, item: Item) {
        let id = node(item);
        let (x, y) = self
            .find(id)
            .unwrap_or_else(|| panic!("{item:?} não está pintado"));
        let arena = Bump::new();
        let mut ev = self
            .hero
            .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
            .to_vec();
        ev.extend_from_slice(self.hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
        assert!(
            ev.iter()
                .any(|e| matches!(e, WidgetEvent::Click(i) if *i == id)),
            "{ev:?}"
        );
        for e in ev {
            self.hero.apply_event(e);
        }
        self.paint();
    }

    /// Arrasto com o botão principal no quadro, em px de ecrã, pela porta da shell.
    fn drag(&mut self, a: (f32, f32), b: (f32, f32)) {
        let (h, ts, c) = (&mut self.hero, &mut self.ts, &mut self.clock);
        let (down, up) = (PointerKind::Down, PointerKind::Up);
        let p = PointerButton::Primary;
        assert!(board_view::pointer(h, inp(ts, c), down, p, a.0, a.1, true));
        board_view::pointer_move(h, inp(ts, c), (a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
        board_view::pointer_move(h, inp(ts, c), b.0, b.1);
        board_view::pointer(h, inp(ts, c), up, p, b.0, b.1, true);
        self.paint();
    }

    fn key(&mut self, k: BoardKey, mods: Modifiers, text: Option<&str>) -> bool {
        let out = board_keys::key(&mut self.hero, &mut self.ts, k, mods, true, text, None);
        board_keys::key(&mut self.hero, &mut self.ts, k, mods, false, None, None);
        self.paint();
        out.consumed
    }

    fn elements(&self) -> Vec<Element> {
        self.hero
            .documents
            .active_board()
            .map(|b| b.doc.live_in_z_order().into_iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Um ponto da área do quadro, em fracções dela.
    fn at(&self, fx: f32, fy: f32) -> (f32, f32) {
        let c = self.hero.last_canvas;
        (c.x + c.w * fx, c.y + c.h * fy)
    }
}

/// O que a shell traria, com o relógio a andar um segundo por evento (nada é duplo-clique sem querer).
fn inp<'a>(ts: &'a mut TextSystem, clock: &mut u128) -> Input<'a> {
    *clock += 1_000_000_000;
    Input {
        text: ts,
        mods: Modifiers::default(),
        now_ns: *clock,
    }
}

const CTRL: Modifiers = Modifiers {
    shift: false,
    ctrl: true,
    alt: false,
    meta: false,
};

#[test]
fn the_toolbar_tool_draws_a_shape_and_the_style_bar_restyles_it_and_ctrl_z_undoes() {
    let mut t = T::new();
    t.click_bar(Item::Tool(Tool::Shape(ShapeType::Rectangle)));
    let (a, b) = (t.at(0.4, 0.4), t.at(0.6, 0.6));
    t.drag(a, b);
    let els = t.elements();
    assert_eq!(els.len(), 1, "a ferramenta da barra não desenhou");
    assert_eq!(els[0].shape().unwrap().kind, ShapeType::Rectangle);
    // A barra de estilo está por cima da selecção: o tracejado aplica-se à forma.
    t.click_bar(Item::Dash(Dash::Dashed));
    assert_eq!(t.elements()[0].shape().unwrap().style.dash, Dash::Dashed);
    // Um fundo claro troca a letra para a tinta escura (a do tema escuro sumia nele — foto 06/10).
    t.click_bar(Item::Fill(Some(0)));
    let st = t.elements()[0].shape().unwrap().style.clone();
    assert_eq!(st.text_color, st.fill.unwrap().readable_ink());
    t.key(BoardKey::Char('z'), CTRL, None);
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert_eq!(t.elements()[0].shape().unwrap().style.dash, Dash::Solid);
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert!(t.elements().is_empty(), "o desenho não se desfez");
}

#[test]
fn the_more_shapes_grid_opens_and_picks_a_flowchart_shape() {
    let mut t = T::new();
    assert!(
        t.find(node(Item::Pick(ShapeType::Cylinder))).is_none(),
        "a grelha nasce aberta"
    );
    t.click_bar(Item::MoreShapes);
    t.click_bar(Item::Pick(ShapeType::Cylinder));
    assert!(
        t.find(node(Item::Pick(ShapeType::Cylinder))).is_none(),
        "a grelha não fechou"
    );
    let (a, b) = (t.at(0.3, 0.3), t.at(0.5, 0.5));
    t.drag(a, b);
    assert_eq!(t.elements()[0].shape().unwrap().kind, ShapeType::Cylinder);
}

#[test]
fn one_letter_shortcuts_delete_and_typing_belong_to_the_board() {
    let mut t = T::new();
    assert!(t.key(BoardKey::Char('o'), Modifiers::default(), Some("o")));
    let (a, b) = (t.at(0.3, 0.3), t.at(0.5, 0.5));
    t.drag(a, b);
    assert_eq!(t.elements()[0].shape().unwrap().kind, ShapeType::Ellipse);
    // Uma letra solta com a forma seleccionada é ATALHO (o idioma do Excalidraw): para escrever,
    // `Enter` (ou duplo-clique).
    assert!(t.key(BoardKey::Char('x'), Modifiers::default(), Some("x")));
    assert!(
        t.elements()[0].shape().unwrap().text.is_empty(),
        "a letra solta escreveu"
    );
    assert!(t.key(BoardKey::Enter, Modifiers::default(), None));
    assert!(t.key(BoardKey::Char('h'), Modifiers::default(), Some("H")));
    assert!(t.key(BoardKey::Char('i'), Modifiers::default(), Some("i")));
    assert_eq!(t.elements()[0].shape().unwrap().text, "Hi");
    assert!(
        t.key(BoardKey::Escape, Modifiers::default(), None),
        "o Esc não terminou o texto"
    );
    assert!(t.key(BoardKey::Delete, Modifiers::default(), None));
    assert!(t.elements().is_empty());
}

#[test]
fn a_double_click_on_a_shape_writes_in_it() {
    let mut t = T::new();
    t.key(BoardKey::Char('r'), Modifiers::default(), Some("r"));
    let (a, b) = (t.at(0.3, 0.3), t.at(0.6, 0.6));
    t.drag(a, b);
    let mid = t.at(0.45, 0.45);
    for _ in 0..2 {
        let i = Input {
            text: &mut t.ts,
            mods: Modifiers::default(),
            now_ns: t.clock, // o MESMO instante: é um duplo-clique
        };
        board_view::pointer(
            &mut t.hero,
            i,
            PointerKind::Down,
            PointerButton::Primary,
            mid.0,
            mid.1,
            true,
        );
        let i = Input {
            text: &mut t.ts,
            mods: Modifiers::default(),
            now_ns: t.clock,
        };
        board_view::pointer(
            &mut t.hero,
            i,
            PointerKind::Up,
            PointerButton::Primary,
            mid.0,
            mid.1,
            true,
        );
    }
    t.paint();
    assert!(
        t.key(BoardKey::Char('o'), Modifiers::default(), Some("o")),
        "a letra não foi do texto"
    );
    assert_eq!(t.elements()[0].shape().unwrap().text, "o");
    assert_eq!(
        t.hero.documents.live.editor.as_ref().unwrap().tool,
        Tool::Select,
        "o «o» virou atalho de elipse a meio do texto"
    );
}

#[test]
fn space_drag_pans_the_view_and_moves_nothing() {
    let mut t = T::new();
    t.key(BoardKey::Char('r'), Modifiers::default(), Some("r"));
    let (a, b) = (t.at(0.3, 0.3), t.at(0.5, 0.5));
    t.drag(a, b);
    let before = t.elements()[0].x;
    let cam = t.hero.documents.active_board().unwrap().camera;
    board_keys::key(
        &mut t.hero,
        &mut t.ts,
        BoardKey::Space,
        Modifiers::default(),
        true,
        Some(" "),
        None,
    );
    t.drag(t.at(0.4, 0.4), t.at(0.45, 0.4));
    board_keys::key(
        &mut t.hero,
        &mut t.ts,
        BoardKey::Space,
        Modifiers::default(),
        false,
        None,
        None,
    );
    assert_eq!(t.elements()[0].x, before, "o Espaço+arrastar moveu a forma");
    assert_ne!(
        t.hero.documents.active_board().unwrap().camera,
        cam,
        "a vista não andou"
    );
}

#[test]
fn each_board_undoes_only_itself() {
    let mut t = T::new();
    let first = t.hero.documents.active().unwrap();
    t.key(BoardKey::Char('r'), Modifiers::default(), Some("r"));
    let (a, b) = (t.at(0.3, 0.3), t.at(0.5, 0.5));
    t.drag(a, b);
    t.hero.apply_event(WidgetEvent::Click(ids::DOC_TAB_NEW));
    t.paint();
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    t.hero.documents.activate(Some(first));
    t.paint();
    assert_eq!(t.elements().len(), 1, "o Ctrl+Z do 2.º quadro desfez o 1.º");
}

#[test]
fn every_bar_item_has_a_distinct_id_that_maps_back() {
    let all = items();
    let mut seen = std::collections::BTreeSet::new();
    for it in &all {
        let id = node(*it);
        assert!(seen.insert(id), "{it:?} repete um id");
        assert_eq!(item_of(id), Some(*it));
        for b in 1..10_000 {
            let tab = super::super::document_tabs::tab_node_id(ph2d_board_model::BoardId(b));
            assert_ne!(tab, id, "{it:?} é a aba do quadro {b}");
        }
    }
}

/// ⛔ A barra de estilo não tapa a pega de RODAR (foto da cena 2, 06/10: tapava).
#[test]
fn the_style_bar_does_not_cover_the_rotate_knob() {
    let mut t = T::new();
    t.key(BoardKey::Char('r'), Modifiers::default(), Some("r"));
    let (a, b) = (t.at(0.4, 0.5), t.at(0.6, 0.6));
    t.drag(a, b);
    let board = t.hero.documents.active_board().unwrap();
    let ed = t.hero.documents.live.editor.as_ref().unwrap();
    let f = ed.frame(&board.doc).unwrap();
    let area = super::super::board_view::area_of(t.hero.last_canvas);
    let px = 1.0 / board.camera.zoom;
    let knob = f.handle(ph2d_board_edit::Handle::Rotate, px, ed.metrics());
    let [kx, ky] = board.camera.to_screen(area, knob);
    let half = (ed.metrics().handle / 2.0) as f32;
    // Quem está por cima da pega no ecrã é ela mesma — nenhum botão da barra.
    let (kx, ky) = (kx as f32, ky as f32);
    for dy in [-half, 0.0, half] {
        let hit = t.hero.hit_index.hit(kx, ky + dy);
        assert!(
            hit.and_then(item_of).is_none(),
            "a barra tapa a pega de rodar: {:?}",
            hit.and_then(item_of)
        );
    }
}
