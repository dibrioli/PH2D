use super::{BoardOp, apply_batch};
use crate::{BoardDoc, Element, ElementKind, Rgba};

fn rect(doc: &mut BoardDoc, x: f64) -> Element {
    Element {
        id: doc.mint_id(),
        kind: ElementKind::Rect {
            fill: Rgba([200, 180, 40, 255]),
        },
        x,
        y: 0.0,
        w: 10.0,
        h: 10.0,
        z: doc.z_on_top(),
        version: 0,
        nonce: 0,
        deleted: false,
    }
}

#[test]
fn put_then_its_inverse_returns_the_document_to_where_it_was() {
    let mut doc = BoardDoc::default();
    let before = doc.clone();
    let el = rect(&mut doc, 1.0);
    let id = el.id;
    let undo = BoardOp::Put(el).apply(&mut doc).expect("mudou");
    assert_eq!(doc.live_len(), 1);
    assert_eq!(undo, BoardOp::Delete(id));
    undo.apply(&mut doc);
    assert_eq!(doc.live_len(), 0);
    assert!(doc.get(id).is_none());
    // a lápide fica (Etapa 2); só o conteúdo VIVO volta a ser o de antes
    assert_eq!(doc.live_in_z_order(), before.live_in_z_order());
}

#[test]
fn a_move_undoes_to_the_previous_pose_and_the_version_only_climbs() {
    let mut doc = BoardDoc::default();
    let el = rect(&mut doc, 1.0);
    let id = el.id;
    BoardOp::Put(el.clone()).apply(&mut doc);
    let v1 = doc.get(id).unwrap().version;
    let moved = Element {
        x: 50.0,
        ..doc.get(id).unwrap().clone()
    };
    let undo = BoardOp::Put(moved).apply(&mut doc).unwrap();
    assert_eq!(doc.get(id).unwrap().x, 50.0);
    let v2 = doc.get(id).unwrap().version;
    undo.apply(&mut doc);
    assert_eq!(doc.get(id).unwrap().x, 1.0);
    assert!(
        v1 < v2 && v2 < doc.get(id).unwrap().version,
        "a versão nunca desce, nem ao desfazer"
    );
}

#[test]
fn deleting_twice_changes_nothing_the_second_time() {
    let mut doc = BoardDoc::default();
    let el = rect(&mut doc, 1.0);
    let id = el.id;
    BoardOp::Put(el).apply(&mut doc);
    assert!(BoardOp::Delete(id).apply(&mut doc).is_some());
    assert!(BoardOp::Delete(id).apply(&mut doc).is_none());
}

#[test]
fn a_batch_undoes_in_reverse_order() {
    let mut doc = BoardDoc::default();
    let a = rect(&mut doc, 1.0);
    let b = rect(&mut doc, 2.0);
    let (ia, ib) = (a.id, b.id);
    let undo = apply_batch(
        &mut doc,
        vec![BoardOp::Put(a), BoardOp::Put(b), BoardOp::Delete(ia)],
    );
    assert_eq!(doc.live_len(), 1);
    apply_batch(&mut doc, undo);
    assert_eq!(doc.live_len(), 0);
    assert!(doc.get(ia).is_none() && doc.get(ib).is_none());
}

#[test]
fn z_on_top_draws_last() {
    let mut doc = BoardDoc::default();
    for x in 0..5 {
        let el = rect(&mut doc, f64::from(x));
        BoardOp::Put(el).apply(&mut doc);
    }
    let xs: Vec<f64> = doc.live_in_z_order().iter().map(|e| e.x).collect();
    assert_eq!(xs, vec![0.0, 1.0, 2.0, 3.0, 4.0]);
}

/// ⛔ O `z_on_top` era O(n) por chamada — montar 100 mil elementos custava 100 mil × 100 mil
/// comparações. Agora é a chave lembrada: montar 100 mil tem de ser instantâneo e manter a ordem.
#[test]
fn a_hundred_thousand_puts_on_top_stay_ordered() {
    let mut doc = BoardDoc::default();
    for i in 0..100_000 {
        let el = rect(&mut doc, f64::from(i));
        BoardOp::Put(el).apply(&mut doc);
    }
    let xs: Vec<f64> = doc.live_in_z_order().iter().map(|e| e.x).collect();
    assert!(
        xs.windows(2).all(|w| w[0] < w[1]),
        "a ordem de z não é a de chegada"
    );
}
