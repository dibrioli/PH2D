//! Arrange the WHOLE motion document for the value-domain smoke scenes, and MARK
//! the node the reviewer should judge.
//!
//! Each value demo ADDS its comparison rows to the editor's boot document — the
//! snow particle scene (`motion_demo_strobe`), a whole branchy graph with a `pre`
//! feedback loop AND a collapsed group ("Age & Fade"). Hand-placed rows landed on
//! top of that scene's own hand-placed rows, and the group card floated away from
//! its chain. This delegates to the **subgraph-aware** layout in `ph2d-motion-doc`
//! (which lays every canvas out — the root and each group's interior — treating a
//! group as one inline card) and stamps `>> EVALUATE <<` on the node under
//! evaluation, so the reviewer knows where to look without counting cards.

use ph2d_motion_doc::MotionDoc;
use ph2d_nodegraph::graph::NodeId;

/// The label stamped on the node to evaluate. ASCII only (the tofu gate) and
/// English (the UI-language rule); the brackets make it pop among plain names.
const MARK: &str = ">> EVALUATE <<";

/// Lay the whole document out (the subgraph-aware layered arrange), then mark
/// each hero with the evaluate label. A reference row contributes no hero.
pub(crate) fn arrange_and_mark(doc: &mut MotionDoc, heroes: &[NodeId]) {
    ph2d_motion_doc::layout::arrange(doc);
    for &h in heroes {
        doc.graph.set_label(h, MARK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::graph::{Edge, Pos};

    /// The hero carries the mark and its position is arranged; a non-hero is
    /// left unmarked. (The layout itself is proven in `ph2d-nodegraph::layout`
    /// and `ph2d-motion-doc::layout`.)
    #[test]
    fn it_arranges_and_marks_only_the_hero() {
        let mut doc = MotionDoc::new();
        let a = doc.graph.add_node("motion.grid");
        let b = doc.graph.add_node("motion.output");
        doc.graph
            .connect(Edge {
                from: (a, 0),
                to: (b, 0),
                delayed: false,
            })
            .expect("edge");
        doc.graph.set_pos(a, Pos { x: 999.0, y: 999.0 });

        arrange_and_mark(&mut doc, &[b]);

        assert_eq!(doc.graph.label(b), Some(MARK), "the hero is marked");
        assert_ne!(doc.graph.label(a), Some(MARK), "a non-hero is not marked");
        assert!(doc.graph.pos(a).expect("pos").x < 100.0, "the graph was arranged");
    }
}
