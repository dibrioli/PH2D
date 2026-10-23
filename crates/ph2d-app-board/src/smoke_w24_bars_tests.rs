//! ⭐ A cena 29 abre com a caixa da esquerda seleccionada e a barra dela pintada à direita da barra
//! curta; com a gaveta das notas aberta, nenhum botão da barra de estilo cai sobre um da gaveta.

use super::*;
use crate::smoke::fac_live::tests::{Hand, hero};
use ph2d_editor_core::screens::hero::board_bar::{self, Item};

fn meets(a: ph2d_editor_core::zones::Rect, b: ph2d_editor_core::zones::Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

#[test]
fn the_w24_scene_opens_with_the_left_box_and_its_bar() {
    let mut h = hero("Ana");
    let s = scene_bars(&mut h);
    Hand::new().paint(&mut h);
    assert_eq!(h.documents.active(), Some(s.board));
    let fill = board_bar::node(Item::Fill(None));
    let bar = h.hit_index.rect_for(fill).expect("a barra da caixa da esquerda");
    let shapes = board_bar::node(Item::Pick(ShapeType::Ellipse));
    assert!(h.hit_index.rect_for(shapes).is_some(), "a gaveta das formas abre com a cena");
    let area = h.last_canvas;
    let tools = board_bar::toolbar_rects(area);
    let right = tools.iter().map(|(_, r)| r.x + r.w).fold(0.0, f32::max);
    assert!(bar.x > right, "a barra {bar:?} à direita da barra curta (até {right})");
    // A caixa da esquerda fica rente à barra curta: as gavetas abrem por cima da barra DELA.
    let note = Item::Tool(ph2d_board_edit::Tool::Shape(ShapeType::Sticky));
    h.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(board_bar::node(note)));
    Hand::new().paint(&mut h);
    let style = h.hit_index.rect_for(fill).expect("a barra continua com a gaveta aberta");
    let drawer = board_bar::node(Item::NoteSize(0));
    assert!(h.hit_index.rect_for(drawer).is_some(), "a gaveta das notas abriu");
    for (id, r) in h.hit_index.iter_registrations() {
        assert!(id == fill || !meets(style, r), "{id:?} {r:?} tapado pela barra em {style:?}");
    }
}
