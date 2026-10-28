//! ⭐ W11 do Kanban pelo hero REAL: abrir cada tipo de ligação chega ao canal da família; notas →
//! cartões e o inverso, cada um UM passo de desfazer do Quadro (pelo `Ctrl+Z` da porta do teclado),
//! com o lado do Kanban a acompanhar em [`serve`].

use super::*;
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::board_keys::{self, BoardKey};
use crate::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_board_model::{BoardOp, Element, Rgba, RichText, STICKY_COLORS, Shape, ShapeType};
use ph2d_host::Modifiers;
use ph2d_kanban_model::{CardRef, KanbanId, ListId, Moment, Record};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

struct T {
    hero: HeroScreen,
    ts: TextSystem,
}

const CTRL: Modifiers = Modifiers {
    shift: false,
    ctrl: true,
    alt: false,
    meta: false,
};

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

    /// `Ctrl+Z` (ou `Ctrl+Shift+Z`) pela porta do teclado do Quadro, e a imagem seguinte.
    fn undo(&mut self, redo: bool) {
        let mods = Modifiers {
            shift: redo,
            ..CTRL
        };
        let k = BoardKey::Char('z');
        board_keys::key(&mut self.hero, &mut self.ts, k, mods, true, None, None);
        board_keys::key(&mut self.hero, &mut self.ts, k, mods, false, None, None);
        serve(&mut self.hero);
        self.paint();
    }

    /// Uma nota de `text` e cor `c` posta no Quadro activo (um passo) e seleccionada com as outras.
    fn note(&mut self, text: &str, c: usize, y: f64) -> ElementId {
        let theme = self.hero.theme;
        let (board, live) = self.hero.documents.active_parts().unwrap();
        let mut style = crate::screens::hero::board_view::default_style();
        style.fill = Some(Rgba(STICKY_COLORS[c]));
        let shape = Shape {
            kind: ShapeType::Sticky,
            style,
            text: RichText::plain(text),
        };
        let id = board.doc.mint_id();
        let el = Element::new_shape(id, board.doc.z_on_top(), shape, [0.0, y, 199.0, 199.0]);
        live.histories
            .entry(board.id)
            .or_default()
            .apply(&mut board.doc, vec![BoardOp::Put(el)]);
        let ed = crate::screens::hero::board_view::editor(&mut live.editor, theme);
        let mut sel: Vec<_> = ed.selection().iter().copied().collect();
        sel.push(id);
        ed.select(&board.doc, sel);
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

    /// Um Kanban com a lista «To Do».
    fn kanban(&mut self) -> (KanbanId, ListId) {
        let shared = self.hero.documents.kanbans_shared();
        let mut set = shared.borrow_mut();
        let k = set.create("Sprint");
        let id = ListId(set.get(k).unwrap().mint());
        let list = ph2d_kanban_model::List {
            id,
            title: "To Do".into(),
            pos: ph2d_kanban_model::FracKey::between(None, None),
            color: None,
            card_limit: None,
            archived: false,
            meta: ph2d_kanban_model::Meta::default(),
            watchers: Vec::new(),
        };
        set.apply(
            k,
            vec![ph2d_kanban_model::KanbanOp::put(Record::List(list))],
        );
        (k, id)
    }

    fn titles(&self, k: KanbanId, l: ListId) -> Vec<String> {
        let set = self.hero.documents.kanbans();
        set.get(k)
            .unwrap()
            .cards_in(l)
            .iter()
            .map(|c| c.title.clone())
            .collect()
    }
}

/// Abrir cada tipo chega ao canal da família dele; o Quadro com elemento activa-o, centra e
/// selecciona (pelo `BoardRequest::Show`), sem elemento só o activa.
#[test]
fn opening_each_kind_reaches_its_familys_channel() {
    let mut t = T::new();
    let span = Target::Span {
        object: Some(41),
        strip: Some(3),
        from: 12,
        to: 48,
    };
    for x in [
        Target::Object(41),
        Target::Script(7),
        Target::Macro("Walk".into()),
        span.clone(),
    ] {
        open(&x);
    }
    assert_eq!(take(Family::Script), vec![Target::Script(7)]);
    assert_eq!(take(Family::Scene), vec![Target::Object(41)]);
    assert_eq!(take(Family::Timeline), vec![span]);
    assert_eq!(take(Family::Macro), vec![Target::Macro("Walk".into())]);
    assert!(take(Family::Scene).is_empty(), "drenado uma vez");

    let first = t.hero.documents.active().unwrap();
    let el = t.note("Idea", 2, 500.0);
    t.hero.apply_event(WidgetEvent::Click(ids::DOC_TAB_NEW));
    t.paint();
    assert_ne!(t.hero.documents.active(), Some(first));
    open(&Target::Board {
        board: first.0,
        element: Some(el.0),
    });
    crate::screens::hero::board_video::kanban::link::serve(&mut t.hero);
    assert_eq!(t.hero.documents.active(), Some(first), "foi ao Quadro");
    let cam = &t.hero.documents.active_board().unwrap().camera;
    assert!((cam.center_y - 599.5).abs() < 1e-6, "centrou no elemento");
    let second = t.hero.documents.boards().boards()[1].id;
    open(&Target::Board {
        board: second.0,
        element: None,
    });
    assert!(serve(&mut t.hero));
    assert_eq!(t.hero.documents.active(), Some(second));
    let cat = catalog(&t.hero.documents);
    assert!(cat.boards[&first.0].elements.contains(&el.0));
}

/// Três notas (duas cores) viram três cartões pela ordem de leitura, com etiqueta da cor e a
/// descrição em markdown; as notas saem. `Ctrl+Z` devolve as notas e tira os cartões; refazer volta.
#[test]
fn notes_become_cards_and_one_undo_brings_the_notes_back() {
    let mut t = T::new();
    let (k, l) = t.kanban();
    let a = t.note("Fix jump\nThe hero falls", 2, 400.0);
    let b = t.note("Polish", 2, 0.0);
    let c = t.note("Sound", 12, 800.0);
    assert_eq!(
        selected_notes(&mut t.hero),
        vec![b, a, c],
        "ordem de leitura"
    );
    assert!(
        choices(&t.hero)
            .iter()
            .any(|x| x.kanban == k && x.list == l)
    );
    let cards = notes_to_cards(&mut t.hero, k, l, Moment(5));
    assert_eq!(cards.len(), 3);
    assert_eq!(t.titles(k, l), ["Polish", "Fix jump", "Sound"]);
    assert!(![a, b, c].iter().any(|n| t.alive(*n)), "as notas saíram");
    {
        let set = t.hero.documents.kanbans();
        let kb = set.get(k).unwrap();
        let fix = &kb.cards[&cards[1]];
        assert_eq!(fix.description, "The hero falls\n");
        let la = &kb.labels[&fix.labels[0]];
        assert_eq!(
            kb.cards[&cards[0]].labels, fix.labels,
            "a mesma cor reusa a etiqueta"
        );
        assert_ne!(
            kb.cards[&cards[2]].labels, fix.labels,
            "outra cor, outra etiqueta"
        );
        assert_eq!(la.color.unwrap().hue, ph2d_kanban_model::Hue::Yellow);
    }
    t.undo(false);
    assert!(
        [a, b, c].iter().all(|n| t.alive(*n)),
        "um Ctrl+Z devolve as notas"
    );
    assert!(t.titles(k, l).is_empty(), "e tira os cartões");
    t.undo(true);
    assert_eq!(t.titles(k, l), ["Polish", "Fix jump", "Sound"], "refazer");
    assert!(!t.alive(a));
}

/// O cartão vira uma nota no Quadro (cor da etiqueta, título + descrição) e sai do Kanban; desfazer
/// devolve-o. Um cartão MEXIDO depois do desfazer não se apaga no refazer.
#[test]
fn a_card_becomes_a_note_and_undo_gives_the_card_back() {
    let mut t = T::new();
    let (k, l) = t.kanban();
    t.note("Fix jump\nThe hero", 3, 0.0);
    let cards = notes_to_cards(&mut t.hero, k, l, Moment(5));
    let r = CardRef {
        kanban: k,
        card: cards[0],
    };
    request_card_to_note(r);
    t.paint();
    assert!(t.titles(k, l).is_empty(), "o cartão saiu");
    let note = selected_notes(&mut t.hero);
    assert_eq!(note.len(), 1, "a nota nova, seleccionada");
    let el = t
        .hero
        .documents
        .active_board()
        .unwrap()
        .doc
        .get(note[0])
        .unwrap()
        .clone();
    let s = el.shape().unwrap();
    assert_eq!(s.text.as_str(), "Fix jump\nThe hero");
    assert_eq!(
        s.style.fill,
        Some(Rgba(STICKY_COLORS[3])),
        "a cor volta à da nota"
    );
    t.undo(false);
    assert!(!t.alive(note[0]));
    assert_eq!(t.titles(k, l), ["Fix jump"], "desfazer devolve o cartão");
    let mut card = t.hero.documents.kanbans().get(k).unwrap().cards[&r.card].clone();
    card.title = "Fix jump (edited)".into();
    let op = ph2d_kanban_model::KanbanOp::put(Record::card(card));
    t.hero
        .documents
        .kanbans_shared()
        .borrow_mut()
        .apply(k, vec![op]);
    t.undo(true);
    assert!(t.alive(note[0]), "a nota volta");
    assert_eq!(
        t.titles(k, l),
        ["Fix jump (edited)"],
        "e o trabalho no cartão nunca se apaga"
    );
}

/// ⭐ A nota que um cartão vira nasce já da altura do texto (a medida do ajuste ao texto), não
/// quadrada à espera da próxima edição; um cartão curto fica na quadrada de nascença.
#[test]
fn a_note_born_from_a_card_fits_its_text_at_birth() {
    let mut t = T::new();
    let (k, l) = t.kanban();
    let long = format!("Long\n{}", "the hero falls through the floor ".repeat(12));
    t.note(&long, 3, 0.0);
    t.note("Short", 3, 400.0);
    let cards = notes_to_cards(&mut t.hero, k, l, Moment(5));
    for c in &cards {
        request_card_to_note(CardRef {
            kanban: k,
            card: *c,
        });
    }
    t.paint();
    let board = t.hero.documents.active_board().unwrap();
    let notes: Vec<_> = board
        .doc
        .live()
        .filter(|e| e.shape().is_some_and(|s| s.kind == ShapeType::Sticky))
        .map(|e| (e.shape().unwrap().text.as_str().to_owned(), e.w, e.h))
        .collect();
    assert_eq!(notes.len(), 2, "os dois cartões viraram notas");
    let (_, w, h) = notes.iter().find(|n| n.0.starts_with("Long")).unwrap();
    assert!(h > &(w * 1.5), "a longa cresceu ao nascer: {w} × {h}");
    let (_, w, h) = notes.iter().find(|n| n.0 == "Short").unwrap();
    assert!((h - w).abs() < 1e-9, "a curta fica quadrada: {w} × {h}");
}

/// Uma macro pedida abre o painel Macros (fechado por omissão) e fica à espera de o painel a
/// drenar ao pintar (`ph2d-panel-macros`, `seam_kanban`); o catálogo tem a biblioteca publicada.
#[test]
fn a_macro_asked_by_a_card_opens_the_macros_panel() {
    let mut t = T::new();
    assert!(
        !t.hero.is_panel_visible("macros"),
        "controlo: nasce fechado"
    );
    publish_macros(["Walk".to_owned()].into());
    open(&Target::Macro("Walk".into()));
    t.paint();
    assert!(t.hero.is_panel_visible("macros"));
    assert!(pending(Family::Macro), "quem drena é o painel");
    assert!(catalog(&t.hero.documents).macros.contains("Walk"));
    assert_eq!(take(Family::Macro), vec![Target::Macro("Walk".into())]);
}
