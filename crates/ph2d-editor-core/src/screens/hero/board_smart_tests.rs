//! ⭐ W23 B13 — o QuickShape do vetor na caneta do quadro pelo caminho REAL do ecrã: o interruptor no
//! painel da caneta (o `pencil_quickshape` do vetor) e uma caixa desenhada e SEGURADA no fim.

use super::*;
use ph2d_board_model::{Pen, ShapeType};

#[test]
fn the_quickshape_switch_and_a_held_box_through_the_screen() {
    let mut t = T::new();
    t.click_bar(Item::Tool(Tool::Pen(Pen::Pen)));
    let on = |t: &T| t.hero.documents.live.editor.as_ref().unwrap().quickshape;
    assert!(on(&t), "ligado de nascença, como no vetor");
    t.click_bar(Item::Pen(PenItem::QuickShape));
    assert!(!on(&t));
    t.click_bar(Item::Pen(PenItem::QuickShape));
    assert!(on(&t));
    // Uma caixa pela porta da shell, um ponto a cada 20 px de ecrã (o relógio do arnês anda 1 s por
    // evento: pontos mais juntos que a folga do vetor contariam como mão parada).
    let (a, b) = (t.at(0.3, 0.3), t.at(0.6, 0.6));
    let corners = [(a.0, a.1), (b.0, a.1), (b.0, b.1), (a.0, b.1), (a.0, a.1)];
    let mut pts = Vec::new();
    for w in corners.windows(2) {
        let n = ((w[1].0 - w[0].0).abs().max((w[1].1 - w[0].1).abs()) / 20.0).ceil() as usize;
        for k in 1..=n {
            let u = k as f32 / n as f32;
            pts.push((
                w[0].0 + (w[1].0 - w[0].0) * u,
                w[0].1 + (w[1].1 - w[0].1) * u,
            ));
        }
    }
    let p = PointerButton::Primary;
    let end = *pts.last().unwrap();
    {
        let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
        assert!(board_view::pointer(
            h,
            inp(ts, c),
            PointerKind::Down,
            p,
            a.0,
            a.1,
            true
        ));
        for q in &pts {
            board_view::pointer_move(h, inp(ts, c), q.0, q.1);
        }
        // Parado no fim (o relógio do arnês: 1 s depois, no mesmo sítio) — a forma nasce.
        board_view::pointer_move(h, inp(ts, c), end.0, end.1);
    }
    t.paint();
    assert!(
        t.hero.documents.live.editor.as_ref().unwrap().is_quick(),
        "a forma não nasceu"
    );
    {
        let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
        board_view::pointer(h, inp(ts, c), PointerKind::Up, p, end.0, end.1, true);
    }
    t.paint();
    let els = t.elements();
    assert_eq!(els.len(), 1);
    assert_eq!(
        els[0].shape().map(|s| s.kind),
        Some(ShapeType::Rectangle),
        "{:?}",
        els[0].kind
    );
}
