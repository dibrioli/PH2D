//! A marca do canto mede da posição ORIGINAL (dono 11/11).

use super::*;

/// ⭐ A linha verde/vermelha marca a posição ORIGINAL do canto (dono 11/11): um segundo arrasto
/// mede a partir dela, não de onde o canto ficou — senão a referência perde-se ao mexer outra vez.
/// *Mutação: `edge_move` sem o `start_mark` ⇒ o `from` volta a ser o canto de agora.*
#[test]
fn a_second_drag_keeps_measuring_from_the_original_corner() {
    for (code, stretch, edge, at) in [(1u8, false, 1u8, 3.0), (5, true, 0, 1.0)] {
        let _ = state::drain_intents();
        let mut s = snap();
        s.lanes[0].strips[0].marks[ph2d_timeline::mark_index(stretch, edge)] = 0.5;
        let mut st = TimelinePanelState::default();
        for (phase, x) in [(GesturePhase::Begin, 100.0), (GesturePhase::End, 250.0)] {
            apply(&mut st, 100.0, &s, 0, 7, code, gesture(code, phase, x));
        }
        let from = state::drain_intents().iter().find_map(|i| match i {
            TimelineIntent::TrimStrip { from, .. } | TimelineIntent::StretchStrip { from, .. } => {
                Some(*from)
            }
            _ => None,
        });
        assert_eq!(
            from,
            Some(at + 0.5),
            "grip {code}: a origem é a do 1.º arrasto"
        );
    }
}
