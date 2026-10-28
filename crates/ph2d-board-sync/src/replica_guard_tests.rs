//! O carimbo absurdo de um par (a MESMA lei do Kanban, `lww::MAX_STAMP`): recusado, e o relógio
//! nunca transborda.

use super::*;
use ph2d_board_model::{BoardSet, Rgba, RichText, Shape, ShapeType, Style};

const RED: Rgba = Rgba([200, 0, 0, 255]);

#[test]
fn a_board_write_from_the_far_future_is_refused_and_the_clock_never_overflows() {
    let mut set = BoardSet::default();
    let id = set.create("Quadro".into());
    let mut host = set.get_mut(id).unwrap().clone();
    let ink = Rgba([20, 20, 20, 255]);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(Some(ink), None, ink),
        text: RichText::plain("x"),
    };
    let el = Element::new_shape(host.doc.mint_id(), host.doc.z_on_top(), shape, [0.0; 4]);
    let x = el.id;
    let _ = BoardOp::Put(el).apply(&mut host.doc);
    let mut h = Replica::host(1, &mut host, 100);
    let (mut g, mut guest) = Replica::join(2, &h.snapshot(&host), 101).unwrap();
    let mut el = guest.doc.get(x).unwrap().clone();
    el.style_mut().fill = Some(RED);
    let _ = BoardOp::Put(el).apply(&mut guest.doc);
    let mut ws = g.local(&mut guest);
    assert!(!ws.is_empty());
    for w in &mut ws {
        if let Write::Set { at, .. } | Write::Born { at, .. } = w {
            at.n = u64::MAX;
        }
    }
    h.receive(&mut host, &ws);
    assert_ne!(
        host.doc.get(x).unwrap().style().fill,
        Some(RED),
        "o forjado não entra"
    );
    assert!(h.clock <= crate::lww::MAX_STAMP, "o relógio não salta");
    let mut el = host.doc.get(x).unwrap().clone();
    el.style_mut().fill = Some(Rgba([0, 0, 200, 255]));
    let _ = BoardOp::Put(el).apply(&mut host.doc);
    let mine = h.local(&mut host);
    assert!(!mine.is_empty() && mine.iter().all(|w| w.at().n <= crate::lww::MAX_STAMP));
    // No limite, o relógio satura em vez de transbordar.
    h.clock = u64::MAX;
    let _ = h.tick();
}
