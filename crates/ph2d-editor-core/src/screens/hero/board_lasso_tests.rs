//! ⭐ W23 B12 — o laço pelo caminho REAL do ecrã: o botão «Lasso» no painel da caneta (o Miro põe-no
//! junto das canetas) → desenhar à volta de uma caixa no quadro → só ela fica seleccionada, e o rasto
//! desenha-se enquanto se arrasta.

use super::*;
use ph2d_board_model::Pen;

#[test]
fn the_lasso_button_draws_a_loop_that_selects_what_is_inside() {
    let mut t = T::new();
    for (a, b) in [((0.2, 0.4), (0.3, 0.5)), ((0.6, 0.4), (0.7, 0.5))] {
        t.key(BoardKey::Char('r'), Modifiers::default(), Some("r"));
        let (a, b) = (t.at(a.0, a.1), t.at(b.0, b.1));
        t.drag(a, b);
    }
    t.key(BoardKey::Escape, Modifiers::default(), None);
    t.click_bar(Item::Tool(Tool::Pen(Pen::Pen)));
    t.click_bar(Item::Pen(PenItem::Lasso));
    assert_eq!(t.hero.documents.live.editor.as_ref().unwrap().tool, Tool::Lasso);
    // Um laço à volta da 1.ª caixa, ponto a ponto, pela porta da shell.
    let loop_pts: Vec<(f32, f32)> = (0..=48)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / 48.0;
            t.at(0.25 + 0.12 * a.cos(), 0.45 + 0.12 * a.sin())
        })
        .collect();
    let p = PointerButton::Primary;
    {
        let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
        assert!(board_view::pointer(h, inp(ts, c), PointerKind::Down, p, loop_pts[0].0, loop_pts[0].1, true));
        for q in &loop_pts[1..] {
            board_view::pointer_move(h, inp(ts, c), q.0, q.1);
        }
    }
    t.paint();
    {
        // A meio do gesto o rasto vai no que se desenha por cima do quadro.
        let (board, live) = t.hero.documents.active_parts().unwrap();
        let ed = live.editor.as_mut().unwrap();
        let o = ed.overlay(&mut board.doc, &mut t.ts);
        assert!(o.lasso.is_some_and(|l| l.len() > 10), "o rasto não se desenha");
    }
    let last = *loop_pts.last().unwrap();
    {
        let (h, ts, c) = (&mut t.hero, &mut t.ts, &mut t.clock);
        board_view::pointer(h, inp(ts, c), PointerKind::Up, p, last.0, last.1, true);
    }
    t.paint();
    let els = t.elements();
    let ed = t.hero.documents.live.editor.as_ref().unwrap();
    assert_eq!(ed.selection().len(), 1, "só a caixa de dentro");
    assert!(ed.selection().contains(&els[0].id));
}
