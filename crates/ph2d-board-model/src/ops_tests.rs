use super::{BoardOp, apply_batch};
use crate::{BoardDoc, Element, Rgba, Shape, ShapeType, Style};

fn rect(doc: &mut BoardDoc, x: f64) -> Element {
    let ink = Rgba([200, 180, 40, 255]);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(Some(ink), None, ink),
        text: Default::default(),
    };
    Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, [x, 0.0, 10.0, 10.0])
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

#[test]
fn history_undoes_and_redoes_a_gesture_as_one_step() {
    let mut doc = BoardDoc::default();
    let mut h = crate::History::default();
    let a = rect(&mut doc, 1.0);
    let b = rect(&mut doc, 2.0);
    let (ia, ib) = (a.id, b.id);
    h.apply(&mut doc, vec![BoardOp::Put(a), BoardOp::Put(b)]);
    let mut moved = doc.get(ia).unwrap().clone();
    moved.x = 50.0;
    h.apply(&mut doc, vec![BoardOp::Put(moved)]);
    assert!(h.undo(&mut doc));
    assert_eq!(doc.get(ia).unwrap().x, 1.0, "o movimento desfez-se");
    assert!(h.undo(&mut doc));
    assert_eq!(doc.live_len(), 0, "a criação dos dois é UM passo");
    assert!(!h.undo(&mut doc));
    assert!(h.redo(&mut doc));
    assert_eq!(doc.live_len(), 2);
    assert!(h.redo(&mut doc));
    assert_eq!(doc.get(ia).unwrap().x, 50.0);
    assert!(doc.get(ib).is_some());
    assert!(!h.redo(&mut doc));
    // Um gesto novo depois de desfazer apaga o refazer.
    h.undo(&mut doc);
    h.apply(&mut doc, vec![BoardOp::Delete(ib)]);
    assert!(!h.can_redo());
}

#[test]
fn rotation_round_trips_and_the_aabb_holds_the_rotated_corners() {
    let mut doc = BoardDoc::default();
    let mut el = rect(&mut doc, 0.0);
    el.w = 20.0;
    el.angle = std::f64::consts::FRAC_PI_2;
    let p = [3.0, 4.0];
    let q = el.rotate(el.unrotate(p));
    assert!((q[0] - p[0]).abs() < 1e-9 && (q[1] - p[1]).abs() < 1e-9);
    let [x0, y0, x1, y1] = el.aabb();
    // 20×10 rodado 90° à volta de (10, 5): 10×20 com o mesmo centro.
    assert!((x1 - x0 - 10.0).abs() < 1e-9 && (y1 - y0 - 20.0).abs() < 1e-9);
    assert!(((x0 + x1) / 2.0 - 10.0).abs() < 1e-9 && ((y0 + y1) / 2.0 - 5.0).abs() < 1e-9);
}

#[test]
fn the_readable_ink_is_dark_on_pastels_and_light_on_dark_fills() {
    let dark = Rgba(crate::DEFAULT_INK);
    let light = Rgba(crate::DEFAULT_PAPER);
    for pastel in [
        [0xFF, 0xF5, 0x9D, 0xFF],
        [0xBB, 0xDE, 0xFB, 0xFF],
        [0xC8, 0xE6, 0xC9, 0xFF],
    ] {
        assert_eq!(Rgba(pastel).readable_ink(), dark);
    }
    for deep in [[0x1E, 0x1E, 0x2E, 0xFF], [0x3D, 0x3D, 0x8B, 0xFF]] {
        assert_eq!(Rgba(deep).readable_ink(), light);
    }
}
