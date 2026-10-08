//! A navegação de um quadro grande (W5) pelo caminho REAL: os botões do canto e as linhas do painel
//! pelo despacho, o minimapa pela porta do rato da shell, a busca e as etiquetas pelos campos.

use super::*;
use crate::interaction::InteractiveState;
use crate::screens::hero::board_nav::{self, NavButton};
use ph2d_board_edit::Notice;
use ph2d_board_model::{BoardOp, ElementId, Frame, Rgba, STICKY_COLORS, Shape, Style};

const NONE: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    meta: false,
};
const ALT: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: true,
    meta: false,
};
const CTRL_ALT: Modifiers = Modifiers {
    shift: false,
    ctrl: true,
    alt: true,
    meta: false,
};

impl T {
    fn put(&mut self, el: Element) -> ElementId {
        let id = el.id;
        let board = self.hero.documents.active_board_mut().unwrap();
        BoardOp::Put(el).apply(&mut board.doc);
        id
    }

    fn note(&mut self, text: &str, at: [f64; 2]) -> ElementId {
        let fill = Rgba(STICKY_COLORS[2]);
        let doc = &mut self.hero.documents.active_board_mut().unwrap().doc;
        let shape = Shape {
            kind: ShapeType::Sticky,
            style: Style::new(Some(fill), None, fill.readable_ink()),
            text: text.into(),
        };
        let el = Element::new_shape(
            doc.mint_id(),
            doc.z_on_top(),
            shape,
            [at[0], at[1], 199.0, 199.0],
        );
        self.put(el)
    }

    fn frame(&mut self, name: &str, bx: [f64; 4]) -> ElementId {
        let doc = &mut self.hero.documents.active_board_mut().unwrap().doc;
        let ink = Rgba(ph2d_board_model::DEFAULT_INK);
        let f = Frame {
            name: name.into(),
            style: Style::new(Some(Rgba(ph2d_board_model::FRAME_FILL)), None, ink),
        };
        let el = Element::new_frame(doc.mint_id(), doc.z_on_top(), f, bx);
        self.put(el)
    }

    fn camera(&self) -> ph2d_board_model::Camera {
        self.hero.documents.active_board().unwrap().camera
    }

    fn el(&self, id: ElementId) -> Element {
        self.hero
            .documents
            .active_board()
            .unwrap()
            .doc
            .get(id)
            .cloned()
            .unwrap()
    }

    /// Down + Up pelo despacho real sobre o id `id` já pintado.
    fn click_id(&mut self, id: NodeId, double: bool) {
        let (x, y) = self
            .find(id)
            .unwrap_or_else(|| panic!("{id:?} não está pintado"));
        let arena = Bump::new();
        let mut ev = self
            .hero
            .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
            .to_vec();
        ev.extend_from_slice(self.hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
        if double {
            ev.extend_from_slice(
                self.hero
                    .handle_pointer(ptr(PointerKind::Down, x, y), &arena),
            );
            ev.extend_from_slice(self.hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
        }
        for e in ev {
            self.hero.apply_event(e);
        }
        self.paint();
    }

    /// Escreve `text` no campo `id` (o que o teclado faria) e manda o evento.
    fn type_into(&mut self, id: NodeId, text: &str) {
        self.hero.store.register(
            id,
            InteractiveState::TextInput {
                state: crate::widget::TextInputState::Focused,
                text: text.to_owned(),
                caret: text.len(),
                selection_anchor: None,
            },
        );
        self.hero.apply_event(WidgetEvent::TextChanged(id));
        self.paint();
    }

    fn submit(&mut self, id: NodeId) {
        self.hero.apply_event(WidgetEvent::Submit(id));
        self.paint();
    }

    /// O ponto `world` está à vista?
    fn on_screen(&self, world: [f64; 2]) -> bool {
        let a = board_view::area_of(self.hero.last_canvas);
        let p = self.camera().to_screen(a, world);
        p[0] >= a[0] && p[1] >= a[1] && p[0] <= a[0] + a[2] && p[1] <= a[1] + a[3]
    }
}

#[test]
fn every_board_notice_has_a_phrase() {
    for n in Notice::ALL {
        assert_ne!(ph2d_i18n::tr(n.key()), n.key(), "{n:?}: a frase não existe");
    }
}

/// ⭐ O texto das notas, os nomes dos frames e as etiquetas, sem distinguir maiúsculas, por ordem
/// de leitura; o filtro por etiqueta.
#[test]
fn search_finds_text_frame_names_and_tags_in_reading_order() {
    let mut t = T::new();
    let low = t.note("Ideia B", [500.0, 900.0]);
    let high = t.note("ideia a", [900.0, 100.0]);
    let other = t.note("outra coisa", [0.0, 0.0]);
    let f = t.frame("Ideias", [2000.0, 2000.0, 400.0, 300.0]);
    let mut tagged = t.el(other);
    tagged.tags_mut().push(ph2d_board_model::Tag {
        name: "ideia".into(),
        color: Rgba([1, 2, 3, 255]),
    });
    t.put(tagged);
    let doc = &t.hero.documents.active_board().unwrap().doc;
    assert_eq!(
        board_nav::find(doc, "IDEIA", None),
        vec![other, high, low, f]
    );
    assert_eq!(board_nav::find(doc, "", Some("ideia")), vec![other]);
    assert!(
        board_nav::find(doc, "  ", None).is_empty(),
        "nada pedido, nada achado"
    );
}

/// `Ctrl+F` abre o campo; escrever acha; `Enter` leva a vista ao primeiro achado e depois ao seguinte.
#[test]
fn ctrl_f_finds_and_enter_takes_the_view_to_each_result() {
    let mut t = T::new();
    let a = t.note("alvo um", [50_000.0, 50_000.0]);
    let b = t.note("alvo dois", [-80_000.0, 90_000.0]);
    t.paint();
    assert!(t.key(BoardKey::Char('f'), CTRL, None));
    assert_eq!(
        t.hero.store.focus_id(),
        Some(crate::ids::BOARD_SEARCH_INPUT)
    );
    t.type_into(crate::ids::BOARD_SEARCH_INPUT, "alvo");
    assert_eq!(t.hero.documents.live.nav.results, vec![a, b]);
    t.submit(crate::ids::BOARD_SEARCH_INPUT);
    assert!(
        t.on_screen(t.el(a).center()),
        "o 1.º achado não ficou à vista"
    );
    t.submit(crate::ids::BOARD_SEARCH_INPUT);
    assert!(t.on_screen(t.el(b).center()), "Enter não foi ao seguinte");
    assert!(!t.on_screen(t.el(a).center()));
}

/// ⭐ O minimapa (`M`): carregar nele põe a vista ali; arrastar leva-a.
#[test]
fn the_minimap_moves_the_view_where_it_is_pressed() {
    let mut t = T::new();
    t.note("canto", [0.0, 0.0]);
    t.note("longe", [100_000.0, 60_000.0]);
    assert!(t.key(BoardKey::Char('m'), NONE, Some("m")));
    let r = board_nav::minimap_rect(t.hero.last_canvas);
    let (a, b) = (
        (r.x + r.w * 0.9, r.y + r.h * 0.5),
        (r.x + r.w * 0.1, r.y + r.h * 0.5),
    );
    t.drag(a, b);
    let after = t.camera();
    assert!(
        after.center_x < 50_000.0,
        "o arrasto para a esquerda leva a vista para lá: {after:?}"
    );
    assert_eq!(
        t.elements().len(),
        2,
        "o arrasto no minimapa não chegou ao editor"
    );
    // Um carregar à direita leva a vista para a direita.
    t.drag(a, a);
    assert!(t.camera().center_x > 50_000.0, "{:?}", t.camera());
}

/// `Alt+1` e o botão «enquadrar tudo» põem o quadro inteiro à vista; `Alt+2` a selecção.
#[test]
fn fit_all_and_fit_selection_frame_the_board() {
    let mut t = T::new();
    let a = t.note("a", [-50_000.0, 0.0]);
    let b = t.note("b", [70_000.0, 30_000.0]);
    t.paint();
    assert!(t.key(BoardKey::Char('1'), ALT, None));
    assert!(t.on_screen(t.el(a).center()) && t.on_screen(t.el(b).center()));
    board_view::select(&mut t.hero, [b]);
    assert!(t.key(BoardKey::Char('2'), ALT, None));
    assert!(t.on_screen(t.el(b).center()) && !t.on_screen(t.el(a).center()));
    t.click_id(board_nav::button_node(NavButton::Fit), false);
    assert!(t.on_screen(t.el(a).center()) && t.on_screen(t.el(b).center()));
}

/// ⭐ O painel de Frames: clicar vai ao frame e selecciona-o, duplo-clique renomeia (UM passo),
/// as setas trocam a ordem.
#[test]
fn the_frames_panel_goes_renames_and_reorders() {
    let mut t = T::new();
    let f1 = t.frame("Primeiro", [-90_000.0, 0.0, 800.0, 450.0]);
    let f2 = t.frame("Segundo", [90_000.0, 0.0, 800.0, 450.0]);
    t.click_id(board_nav::button_node(NavButton::Frames), false);
    let row = |f: ElementId| board_nav::row_node(f, board_nav::RowPart::Row);
    t.click_id(row(f2), false);
    assert!(t.on_screen(t.el(f2).center()) && !t.on_screen(t.el(f1).center()));
    t.click_id(row(f1), true);
    assert_eq!(t.hero.documents.live.nav.renaming, Some(f1));
    t.type_into(crate::ids::BOARD_FRAME_RENAME_INPUT, "Abertura");
    t.submit(crate::ids::BOARD_FRAME_RENAME_INPUT);
    assert_eq!(t.el(f1).as_frame().unwrap().name, "Abertura");
    let down = board_nav::row_node(f1, board_nav::RowPart::Down);
    t.click_id(down, false);
    let order: Vec<_> = t
        .hero
        .documents
        .active_board()
        .unwrap()
        .doc
        .frames_in_panel_order()
        .iter()
        .map(|f| f.id)
        .collect();
    assert_eq!(order, vec![f2, f1]);
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert!(t.key(BoardKey::Char('z'), CTRL, None));
    assert_eq!(
        t.el(f1).as_frame().unwrap().name,
        "Primeiro",
        "cada um, UM passo"
    );
}

/// ⭐ `Ctrl+Alt+K` e o clique no destino ligam; o ícone de link leva lá; destino apagado = aviso.
#[test]
fn a_link_is_made_by_ctrl_alt_k_and_a_click_and_its_icon_goes_there() {
    let mut t = T::new();
    let a = t.note("daqui", [0.0, 0.0]);
    let b = t.note("para ali", [400.0, 0.0]);
    t.paint();
    board_view::select(&mut t.hero, [a]);
    assert!(t.key(BoardKey::Char('k'), CTRL_ALT, None));
    let to = {
        let a = board_view::area_of(t.hero.last_canvas);
        let p = t.camera().to_screen(a, t.el(b).center());
        (p[0] as f32, p[1] as f32)
    };
    t.drag(to, to);
    assert_eq!(t.el(a).link(), Some(b));
    // Longe dos dois, o ícone leva ao destino.
    let board = t.hero.documents.active_board_mut().unwrap();
    board.camera.center_x = -1000.0;
    board.camera.zoom = 1.0;
    t.paint();
    let board = t.hero.documents.active_board_mut().unwrap();
    board.camera.center_x = 0.0;
    t.paint();
    t.click_id(board_nav::link_node(a), false);
    assert!(t.on_screen(t.el(b).center()));
    // O destino apagado: o aviso.
    let board = t.hero.documents.active_board_mut().unwrap();
    BoardOp::Delete(b).apply(&mut board.doc);
    board.camera.center_x = 0.0;
    t.paint();
    t.click_id(board_nav::link_node(a), false);
    assert_eq!(
        t.hero.documents.live.notice.map(|(k, _)| k),
        Some(Notice::LinkGone.key())
    );
}

/// ⭐ A etiqueta: o botão da barra das notas abre o campo, `Enter` põe-na; a fita da etiqueta na
/// busca filtra por ela.
#[test]
fn a_tag_goes_on_the_selected_note_and_the_search_filters_by_it() {
    let mut t = T::new();
    let n = t.note("com etiqueta", [0.0, 0.0]);
    let m = t.note("sem", [400.0, 0.0]);
    t.paint();
    board_view::select(&mut t.hero, [n]);
    t.paint();
    t.click_bar(Item::Organize(OrgItem::Tag));
    assert!(t.hero.documents.live.nav.tagging);
    t.type_into(crate::ids::BOARD_TAG_INPUT, "urgente");
    t.submit(crate::ids::BOARD_TAG_INPUT);
    assert_eq!(t.el(n).tags().len(), 1);
    assert!(t.el(m).tags().is_empty());
    assert!(t.key(BoardKey::Char('f'), CTRL, None));
    t.click_id(board_nav::chip_node(0), false);
    assert_eq!(t.hero.documents.live.nav.results, vec![n]);
}

/// ⛔ Com UMA nota seleccionada a letra `m` escreve nela (o idioma do Miro) — não abre o minimapa.
#[test]
fn m_types_into_a_selected_note_instead_of_opening_the_minimap() {
    let mut t = T::new();
    let n = t.note("", [0.0, 0.0]);
    t.paint();
    board_view::select(&mut t.hero, [n]);
    assert!(t.key(BoardKey::Char('m'), NONE, Some("m")));
    assert!(
        !t.hero.documents.live.nav.minimap_open,
        "abriu o minimapa em vez de escrever"
    );
    assert_eq!(t.el(n).shape().unwrap().text.as_str(), "m");
}
