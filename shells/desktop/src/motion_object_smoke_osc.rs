//! **O sufixo GPU do modo `=5`** — `source.object → duplicator ← grid → oscillator →
//! output`. Irmão do despachante pelo teto de LOC do HR-18.

use super::build_stamp_graph;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Como [`build_stamp_graph`], mas com um **`motion.oscillator`** (um deformer
/// GPU por-elemento) entre o duplicator e o output: `source.object → duplicator
/// ← grid → oscillator → output`. É esse sufixo GPU que torna o grafo **Hybrid**
/// — o `duplicator` (CPU) vira o boundary, o oscillator roda no device, e o
/// lowering carrega o `texture_id` do objeto até a word 41 (esta wave). Sem o
/// oscillator o sufixo é só o `output` passthrough (0 dispatch → rota CPU).
pub(super) fn build_stamp_graph_osc(graph: &mut Graph, name: &str) -> NodeId {
    let out = build_stamp_graph(graph, name);
    // `out`'s input 0 is the duplicator; splice an oscillator in between.
    let dup = graph
        .edges()
        .iter()
        .find(|e| e.to == (out, 0))
        .map(|e| e.from.0)
        .expect("duplicator -> output edge");
    let osc = graph.add_node("motion.oscillator");
    graph.set_pos(
        osc,
        Pos {
            x: 315.0,
            y: -200.0,
        },
    );
    graph.set_param(osc, "channel", 1.0); // Y
    graph.set_param(osc, "amplitude", 0.5);
    graph.set_param(osc, "frequency", 0.4);
    graph.set_label(osc, "Wave");
    // Re-route dup → out into dup → osc → out.
    graph.disconnect(out, 0);
    let wire = |g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16| {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: false,
        })
        .expect("connect");
    };
    wire(graph, dup, 0, osc, 0);
    wire(graph, osc, 0, out, 0);
    out
}
