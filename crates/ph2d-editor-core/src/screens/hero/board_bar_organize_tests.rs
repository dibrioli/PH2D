//! Organizar (W5) pelo caminho REAL do ecrã: os botões das barras pelo despacho sobre o que o pintor
//! registou, o rato pela porta da shell, as teclas pela do teclado.

use super::*;
use ph2d_board_edit::{Align, Axis};

const FRAME: Item = super::super::organize::FRAME_TOOL;

const NONE: Modifiers = Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    meta: false,
};
const CTRL_SHIFT: Modifiers = Modifiers {
    shift: true,
    ctrl: true,
    alt: false,
    meta: false,
};

fn org(o: OrgItem) -> Item {
    Item::Organize(o)
}

/// Dois rectângulos lado a lado, desalinhados, seleccionados pelo laço.
fn two_rects(t: &mut T) {
    for (a, b) in [((0.30, 0.30), (0.40, 0.40)), ((0.55, 0.45), (0.62, 0.55))] {
        // Pela tecla: dois cliques seguidos no mesmo botão seriam um duplo-clique.
        assert!(t.key(BoardKey::Char('r'), NONE, Some("r")));
        let (p, q) = (t.at(a.0, a.1), t.at(b.0, b.1));
        t.drag(p, q);
    }
    let (p, q) = (t.at(0.2, 0.2), t.at(0.8, 0.8));
    t.drag(p, q);
    assert_eq!(t.elements().len(), 2);
}

fn painted(t: &T, it: Item) -> bool {
    t.find(node(it)).is_some()
}

#[test]
fn the_frame_button_draws_a_frame_that_takes_what_is_drawn_inside() {
    let mut t = T::new();
    t.click_bar(FRAME);
    let (a, b) = (t.at(0.2, 0.2), t.at(0.8, 0.8));
    t.drag(a, b);
    let f = t.elements()[0].clone();
    assert!(
        f.as_frame().is_some(),
        "o botão do Frame não desenhou um frame"
    );
    assert!(t.key(BoardKey::Char('r'), NONE, Some("r")));
    let (a, b) = (t.at(0.4, 0.4), t.at(0.5, 0.5));
    t.drag(a, b);
    let r = t
        .elements()
        .into_iter()
        .find(|e| e.shape().is_some())
        .unwrap();
    assert_eq!(
        r.frame,
        Some(f.id),
        "a forma desenhada dentro vive no frame"
    );
    // A tecla `F` (a do Miro) também o põe na mão.
    assert!(t.key(BoardKey::Char('f'), NONE, Some("f")));
    assert!(painted(&t, FRAME));
}

/// ⭐ Um botão sem efeito não se pinta: com dois soltos há agrupar e alinhar mas não desagrupar;
/// agrupados são UMA unidade — há desagrupar e já não há alinhar.
#[test]
fn group_and_ungroup_show_only_when_they_would_do_something() {
    let mut t = T::new();
    two_rects(&mut t);
    assert!(painted(&t, org(OrgItem::Group)));
    assert!(
        !painted(&t, org(OrgItem::Ungroup)),
        "desagrupar sem grupo: controlo morto"
    );
    assert!(painted(&t, org(OrgItem::Align(Align::Left))));
    assert!(
        !painted(&t, org(OrgItem::Distribute(Axis::Horizontal))),
        "distribuir pede três"
    );
    t.click_bar(org(OrgItem::Group));
    let els = t.elements();
    assert!(els.iter().all(|e| e.outer_group().is_some()));
    assert_eq!(els[0].outer_group(), els[1].outer_group());
    assert!(painted(&t, org(OrgItem::Ungroup)));
    assert!(!painted(&t, org(OrgItem::Group)));
    assert!(
        !painted(&t, org(OrgItem::Align(Align::Left))),
        "um grupo é uma unidade só"
    );
    t.click_bar(org(OrgItem::Ungroup));
    assert!(t.elements().iter().all(|e| e.outer_group().is_none()));
}

#[test]
fn the_align_button_lines_up_the_selection() {
    let mut t = T::new();
    two_rects(&mut t);
    t.click_bar(org(OrgItem::Align(Align::Left)));
    let els = t.elements();
    assert!(
        (els[0].x - els[1].x).abs() < 1e-9,
        "{} ≠ {}",
        els[0].x,
        els[1].x
    );
}

/// As teclas do Miro: `Ctrl+G`, `Ctrl+Shift+L`, `PgDn`; e a do Excalidraw, `Ctrl+Shift+Left`.
#[test]
fn the_organize_keys_group_lock_order_and_align() {
    let mut t = T::new();
    two_rects(&mut t);
    let ids: Vec<_> = t.elements().iter().map(|e| e.id).collect();
    assert!(t.key(BoardKey::Left, CTRL_SHIFT, None));
    let els = t.elements();
    assert!(
        (els[0].x - els[1].x).abs() < 1e-9,
        "Ctrl+Shift+Left alinha à esquerda"
    );
    // O de cima vai para trás de tudo.
    board_view::select(&mut t.hero, [ids[1]]);
    assert!(t.key(BoardKey::PageDown, NONE, None));
    assert_eq!(t.elements()[0].id, ids[1], "PgDn: atrás de tudo");
    assert!(t.key(BoardKey::PageUp, NONE, None));
    assert_eq!(t.elements()[1].id, ids[1], "PgUp: à frente de tudo");
    board_view::select(&mut t.hero, ids.clone());
    assert!(t.key(BoardKey::Char('g'), CTRL, None));
    assert!(t.elements()[0].outer_group().is_some());
    assert!(t.key(BoardKey::Char('l'), CTRL_SHIFT, None));
    assert!(t.elements().iter().all(Element::locked));
    assert!(painted(&t, org(OrgItem::Lock)));
}

/// ⭐ A recusa chega ao ARTISTA: agrupar entre frames diferentes pinta o aviso no quadro.
#[test]
fn a_refused_group_paints_its_notice_on_the_board() {
    let mut t = T::new();
    t.click_bar(FRAME);
    let (a, b) = (t.at(0.1, 0.1), t.at(0.45, 0.9));
    t.drag(a, b);
    for (a, b) in [((0.2, 0.3), (0.3, 0.4)), ((0.6, 0.3), (0.7, 0.4))] {
        // Pela tecla: dois cliques seguidos no mesmo botão seriam um duplo-clique.
        assert!(t.key(BoardKey::Char('r'), NONE, Some("r")));
        let (p, q) = (t.at(a.0, a.1), t.at(b.0, b.1));
        t.drag(p, q);
    }
    let shapes: Vec<_> = t
        .elements()
        .into_iter()
        .filter(|e| e.shape().is_some())
        .map(|e| e.id)
        .collect();
    board_view::select(&mut t.hero, shapes);
    assert!(t.key(BoardKey::Char('g'), CTRL, None));
    assert!(
        t.elements().iter().all(|e| e.outer_group().is_none()),
        "não agrupou"
    );
    assert_eq!(
        t.hero.documents.live.notice.map(|(k, _)| k),
        Some("board.notice.group_across_frames"),
        "o aviso não chegou ao quadro"
    );
}

/// Um frame seleccionado mostra as cores dele — e o branco de nascença para voltar.
#[test]
fn a_selected_frame_offers_its_colours_and_the_paper_to_go_back() {
    let mut t = T::new();
    t.click_bar(FRAME);
    let (a, b) = (t.at(0.2, 0.2), t.at(0.8, 0.8));
    t.drag(a, b);
    t.click_bar(Item::Fill(Some(0)));
    let fill = t.elements()[0].style().fill;
    assert_ne!(
        fill,
        Some(ph2d_board_model::Rgba(ph2d_board_model::FRAME_FILL))
    );
    assert!(
        !painted(&t, org(OrgItem::Order(ph2d_board_edit::Order::Front))),
        "frames ficam atrás"
    );
    t.click_bar(org(OrgItem::Paper));
    assert_eq!(
        t.elements()[0].style().fill,
        Some(ph2d_board_model::Rgba(ph2d_board_model::FRAME_FILL))
    );
}

/// O PRETO no contorno (pedido do dono, 08/10) — preto do documento, nunca a tinta do tema.
#[test]
fn the_stroke_offers_black_and_it_is_document_black() {
    let mut t = T::new();
    assert!(t.key(BoardKey::Char('r'), NONE, Some("r")));
    let (a, b) = (t.at(0.4, 0.4), t.at(0.6, 0.6));
    t.drag(a, b);
    let black = Item::Stroke(Some(Some(super::super::BLACK)));
    t.click_bar(black);
    assert_eq!(
        t.elements()[0].style().stroke,
        Some(ph2d_board_model::Rgba([0, 0, 0, 255]))
    );
}
