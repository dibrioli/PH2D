use std::collections::BTreeSet;

use crate::{BoardDoc, BoardOp, Element, ElementId, Frame, Rgba, Shape, ShapeType, Style};

fn style() -> Style {
    let ink = Rgba([0, 0, 0, 255]);
    Style::new(Some(Rgba([255, 255, 255, 255])), None, ink)
}

fn frame(doc: &mut BoardDoc, parent: Option<ElementId>, bx: [f64; 4]) -> ElementId {
    let f = Frame {
        name: "f".into(),
        style: style(),
    };
    let mut el = Element::new_frame(doc.mint_id(), doc.z_on_top(), f, bx);
    el.frame = parent;
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

fn rect(doc: &mut BoardDoc, parent: Option<ElementId>, bx: [f64; 4]) -> ElementId {
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: style(),
        text: Default::default(),
    };
    let mut el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, bx);
    el.frame = parent;
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

/// Fora (0..1000) ⊃ meio (100..600) ⊃ dentro (200..400); um rectângulo em cada um e um solto.
struct Nest {
    doc: BoardDoc,
    outer: ElementId,
    mid: ElementId,
    inner: ElementId,
    in_outer: ElementId,
    in_mid: ElementId,
    in_inner: ElementId,
    loose: ElementId,
}

fn nest() -> Nest {
    let mut doc = BoardDoc::default();
    let outer = frame(&mut doc, None, [0.0, 0.0, 1000.0, 1000.0]);
    let mid = frame(&mut doc, Some(outer), [100.0, 100.0, 500.0, 500.0]);
    let inner = frame(&mut doc, Some(mid), [200.0, 200.0, 200.0, 200.0]);
    let in_outer = rect(&mut doc, Some(outer), [800.0, 800.0, 50.0, 50.0]);
    let in_mid = rect(&mut doc, Some(mid), [500.0, 500.0, 50.0, 50.0]);
    let in_inner = rect(&mut doc, Some(inner), [250.0, 250.0, 50.0, 50.0]);
    let loose = rect(&mut doc, None, [2000.0, 0.0, 50.0, 50.0]);
    Nest {
        doc,
        outer,
        mid,
        inner,
        in_outer,
        in_mid,
        in_inner,
        loose,
    }
}

#[test]
fn the_chain_goes_from_the_innermost_frame_out() {
    let n = nest();
    assert_eq!(n.doc.frame_chain(n.in_inner), vec![n.inner, n.mid, n.outer]);
    assert_eq!(n.doc.frame_depth(n.in_mid), 2);
    assert_eq!(n.doc.frame_depth(n.loose), 0);
    assert_eq!(n.doc.frame_depth(n.outer), 0);
}

#[test]
fn descendants_reach_every_depth_and_nothing_outside() {
    let n = nest();
    let want: BTreeSet<_> = [n.mid, n.inner, n.in_outer, n.in_mid, n.in_inner].into();
    assert_eq!(n.doc.descendants(n.outer), want);
    let want: BTreeSet<_> = [n.in_inner].into();
    assert_eq!(n.doc.descendants(n.inner), want);
}

/// ⭐ Um frame nunca entra em si nem em quem vive dentro dele (o ciclo que pararia a cadeia).
#[test]
fn a_frame_cannot_enter_itself_or_its_own_descendant() {
    let n = nest();
    assert!(!n.doc.can_enter(n.outer, n.outer));
    assert!(
        !n.doc.can_enter(n.outer, n.inner),
        "o de fora dentro do neto: ciclo"
    );
    assert!(n.doc.can_enter(n.inner, n.outer));
    assert!(n.doc.can_enter(n.loose, n.inner));
    assert!(
        !n.doc.can_enter(n.loose, n.in_mid),
        "um rectângulo não é frame"
    );
}

/// O frame debaixo do ponteiro é o mais INTERIOR; o que se arrasta (e o que vive nele) não conta.
#[test]
fn the_frame_under_the_pointer_is_the_innermost_outside_what_is_dragged() {
    let n = nest();
    let none = BTreeSet::new();
    assert_eq!(n.doc.deepest_frame_at([300.0, 300.0], &none), Some(n.inner));
    assert_eq!(n.doc.deepest_frame_at([550.0, 150.0], &none), Some(n.mid));
    assert_eq!(n.doc.deepest_frame_at([900.0, 100.0], &none), Some(n.outer));
    assert_eq!(n.doc.deepest_frame_at([5000.0, 0.0], &none), None);
    let dragging: BTreeSet<_> = [n.mid].into();
    assert_eq!(
        n.doc.deepest_frame_at([300.0, 300.0], &dragging),
        Some(n.outer),
        "arrastar o do meio leva o de dentro: nenhum dos dois o recebe"
    );
}

/// Os frames desenham-se do de fora para dentro, seja qual for o `z` (um frame aninhado mais
/// antigo que o de fora continua por cima dele).
#[test]
fn frames_draw_outermost_first_whatever_their_z() {
    let mut doc = BoardDoc::default();
    let inner = frame(&mut doc, None, [10.0, 10.0, 50.0, 50.0]);
    let outer = frame(&mut doc, None, [0.0, 0.0, 100.0, 100.0]);
    let mut el = doc.get(inner).unwrap().clone();
    el.frame = Some(outer);
    BoardOp::Put(el).apply(&mut doc);
    let order: Vec<_> = doc.frames_in_draw_order().iter().map(|e| e.id).collect();
    assert_eq!(order, vec![outer, inner]);
    let panel: Vec<_> = doc.frames_in_panel_order().iter().map(|e| e.id).collect();
    assert_eq!(panel, vec![inner, outer], "o painel segue o z");
}

#[test]
fn group_members_are_the_ones_that_carry_the_group() {
    let mut doc = BoardDoc::default();
    let g = doc.mint_group();
    let a = rect(&mut doc, None, [0.0; 4]);
    let b = rect(&mut doc, None, [0.0; 4]);
    rect(&mut doc, None, [0.0; 4]);
    for id in [a, b] {
        let mut el = doc.get(id).unwrap().clone();
        el.groups_mut().push(g);
        BoardOp::Put(el).apply(&mut doc);
    }
    assert_eq!(doc.group_members(g), vec![a, b]);
    assert_eq!(doc.get(a).unwrap().outer_group(), Some(g));
}
