//! Arrange the WHOLE motion graph for the value-domain smoke scenes, and MARK
//! the node the reviewer should judge.
//!
//! Each value demo ADDS its comparison rows to the editor's boot document — the
//! snow particle scene (`motion_demo_strobe`), a whole branchy graph with a
//! `pre` feedback loop. Hand-placed rows landed on top of that scene's own
//! hand-placed rows, and the result read as one crammed tangle. This delegates
//! to the general layered auto-layout in `ph2d-nodegraph` — which lays the boot
//! scene AND the demo rows out into clean, non-overlapping bands — and stamps
//! `>> EVALUATE <<` on the one node under evaluation, so the reviewer knows
//! where to look without counting cards.

use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::layout;

/// The label stamped on the node to evaluate. ASCII only (the tofu gate) and
/// English (the UI-language rule); the brackets make it pop among plain names.
const MARK: &str = ">> EVALUATE <<";

/// Lay the whole graph out (the general layered arrange), then mark each hero
/// with the evaluate label. A reference row contributes no hero.
pub(crate) fn arrange_and_mark(g: &mut Graph, heroes: &[NodeId]) {
    layout::arrange(g);
    for &h in heroes {
        g.set_label(h, MARK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::graph::{Edge, Pos};

    /// The hero carries the mark and its position is arranged; a non-hero is
    /// left unmarked. (The layout itself is proven in `ph2d-nodegraph::layout`.)
    #[test]
    fn it_arranges_and_marks_only_the_hero() {
        let mut g = Graph::new();
        let a = g.add_node("motion.grid");
        let b = g.add_node("motion.output");
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .expect("edge");
        g.set_pos(a, Pos { x: 999.0, y: 999.0 });

        arrange_and_mark(&mut g, &[b]);

        assert_eq!(g.label(b), Some(MARK), "the hero is marked");
        assert_ne!(g.label(a), Some(MARK), "a non-hero is not marked");
        assert!(g.pos(a).expect("pos").x < 100.0, "the graph was arranged");
    }
}
