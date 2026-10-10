//! O desfazer que não pisa a edição alheia (ADR-0181 §3) — os casos c10–c12 do oráculo da
//! colaboração (`docs/MiroClone/ferramentas/colab_oracle/saidas/excalidraw/`). A «outra pessoa» é
//! uma operação aplicada ao documento sem passar por este `History` (é o que a sessão ao vivo faz).

use super::History;
use crate::{BoardDoc, BoardOp, Element, ElementId, Rgba, Shape, ShapeType, Style};

const RED: Rgba = Rgba([200, 0, 0, 255]);

fn world() -> (BoardDoc, History, ElementId) {
    let mut doc = BoardDoc::default();
    let ink = Rgba([20, 20, 20, 255]);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(Some(ink), None, ink),
        text: Default::default(),
    };
    let el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, [0.0, 0.0, 10.0, 10.0]);
    let id = el.id;
    BoardOp::Put(el).apply(&mut doc);
    (doc, History::default(), id)
}

fn moved(doc: &BoardDoc, id: ElementId, x: f64) -> BoardOp {
    let mut el = doc.get(id).unwrap().clone();
    el.x = x;
    BoardOp::Put(el)
}

fn other_paints(doc: &mut BoardDoc, id: ElementId) {
    let mut el = doc.get(id).unwrap().clone();
    el.style_mut().fill = Some(RED);
    BoardOp::Put(el).apply(doc);
}

/// c11 — eu movo X, outra pessoa pinta-o, eu desfaço: X volta ao sítio e FICA com a cor dela; e o
/// refazer move-o outra vez sem lha tirar.
#[test]
fn undo_puts_back_my_move_and_keeps_the_colour_another_person_gave_since() {
    let (mut doc, mut h, id) = world();
    let op = moved(&doc, id, 50.0);
    h.apply(&mut doc, vec![op]);
    other_paints(&mut doc, id);
    assert!(h.undo(&mut doc));
    let el = doc.get(id).unwrap();
    assert_eq!((el.x, el.style().fill), (0.0, Some(RED)));
    assert!(h.redo(&mut doc));
    let el = doc.get(id).unwrap();
    assert_eq!((el.x, el.style().fill), (50.0, Some(RED)));
}

/// c12 — eu movo X, outra pessoa apaga-o, eu desfaço: X NÃO ressuscita.
#[test]
fn undo_does_not_bring_back_what_another_person_deleted() {
    let (mut doc, mut h, id) = world();
    let op = moved(&doc, id, 50.0);
    h.apply(&mut doc, vec![op]);
    BoardOp::Delete(id).apply(&mut doc);
    h.undo(&mut doc);
    assert!(doc.get(id).is_none(), "ressuscitou");
}

/// c10 — o que outra pessoa mexeu num elemento que EU criei não impede o meu desfazer da criação.
#[test]
fn undo_of_my_creation_removes_it_even_after_another_person_painted_it() {
    let (mut doc, mut h, _) = world();
    let ink = Rgba([1, 1, 1, 255]);
    let shape = Shape {
        kind: ShapeType::Ellipse,
        style: Style::new(Some(ink), None, ink),
        text: Default::default(),
    };
    let el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, [0.0, 0.0, 5.0, 5.0]);
    let mine = el.id;
    h.apply(&mut doc, vec![BoardOp::Put(el)]);
    other_paints(&mut doc, mine);
    h.undo(&mut doc);
    assert!(doc.get(mine).is_none());
}

/// Sozinho, desfazer e refazer são os de sempre: o elemento volta INTEIRO ao de antes.
#[test]
fn alone_undo_and_redo_are_the_whole_steps() {
    let (mut doc, mut h, id) = world();
    let before = doc.get(id).unwrap().clone();
    let mut el = before.clone();
    el.x = 9.0;
    el.style_mut().fill = Some(RED);
    h.apply(&mut doc, vec![BoardOp::Put(el)]);
    h.undo(&mut doc);
    assert!(doc.get(id).unwrap().changed_fields(&before).is_empty());
    h.redo(&mut doc);
    let el = doc.get(id).unwrap();
    assert_eq!((el.x, el.style().fill), (9.0, Some(RED)));
}
