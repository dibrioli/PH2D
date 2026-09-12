//! **Auto-inserted adapters** (Motion Nodes, plan §1.1 item 2) — sibling of
//! `motion_bridge_connect` (shell LOC cap). Declared by the parent as a `#[path]`
//! sibling, so `super` is `render_loop::motion_bridge`.
//!
//! The substrate stays STRICT: `PortType::connects_directly` demands every axis
//! match, and `Graph::connect` + `validate` refuse a wire that mismatches. Instead of
//! leaving the artist with a bare refusal, the editor asks *"is there a node that
//! converts this type into that one?"* — the `CONVERSIONS` table — and, if so, drops
//! that adapter node onto the wire (`from -> adapter -> to`) as ONE undo step. This is
//! the editor's courtesy, not a substrate change; the adapter is an ordinary `Pure`
//! node the artist can see, tune and delete.

use super::{MotionState, reconcile};
use ph2d_editor::{Toast, ToastQueue};
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// A **Stream** port — the point/shape payload every `motion.*` node speaks
/// (`(Instances, Vec2, Frame)`; columns like `tint`/`size` live inside it).
const STREAM: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// A **value** port — a per-instance scalar field (`(Instances, Scalar, Frame)`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// **The conversion table** — `(source type, consumer type, adapter node)`. Each
/// adapter is a single-input, single-output `Pure` node: the source feeds its input
/// port 0, its output port 0 feeds the consumer. Extending it is one line; the
/// pulse-domain converters (`value -> pulse` via `pulse.threshold`, `pulse -> value`
/// via `pulse.counter`) carry a `state` feedback port and are a separate, gated wave.
const CONVERSIONS: &[(PortType, PortType, &str)] = &[
    // Drag a STREAM into a VALUE input: `value.attribute` reads a chosen column
    // (position, speed, opacity…) out of the stream as a scalar. The artist picks
    // which via the node's `attr`/`mode` — the adapter names the ambiguity instead of
    // guessing at it.
    (STREAM, VALUE, "value.attribute"),
];

/// The adapter node type that converts `from` into `to`, if one is registered.
pub(super) fn adapter_for(from: PortType, to: PortType) -> Option<&'static str> {
    CONVERSIONS
        .iter()
        .find(|(f, t, _)| *f == from && *t == to)
        .map(|(_, _, name)| *name)
}

/// The output type of `(node, port)` and the input type of a consumer `(node, port)`,
/// resolved through the node manifests — the same lookup `Graph::validate` does.
fn edge_types(
    motion: &MotionState,
    from_node: u32,
    from_port: u16,
    to_node: u32,
    to_port: u16,
) -> Option<(PortType, PortType)> {
    let g = &motion.doc.graph;
    let out = g
        .node(NodeId(from_node))
        .and_then(|n| motion.registry.resolve(n.type_id()))
        .and_then(|o| o.manifest().outputs.get(from_port as usize).map(|p| p.ty))?;
    let inp = g
        .node(NodeId(to_node))
        .and_then(|n| motion.registry.resolve(n.type_id()))
        .and_then(|o| o.manifest().inputs.get(to_port as usize).map(|p| p.ty))?;
    Some((out, inp))
}

/// **Try to splice an adapter into a type-mismatched forward wire.** Returns `true`
/// iff an adapter was found for the two port types AND the resulting `from -> adapter
/// -> to` graph validates — in which case it is committed as ONE undo step and the
/// pump re-cooks. Returns `false` (the caller then raises the ordinary refusal) when
/// no conversion is registered or the spliced graph would still be illegal.
pub(super) fn try_insert_adapter(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    from_node: u32,
    from_port: u16,
    to_node: u32,
    to_port: u16,
) -> bool {
    let Some((from_ty, to_ty)) = edge_types(motion, from_node, from_port, to_node, to_port) else {
        return false;
    };
    let Some(adapter) = adapter_for(from_ty, to_ty) else {
        return false;
    };

    // The adapter lands on the wire — the midpoint of the two cards it joins, so the
    // artist sees it where the connection was refused (falls back to the source's
    // spot if a position is missing).
    let mut trial = motion.doc.graph.clone();
    let (fp, tp) = (trial.pos(NodeId(from_node)), trial.pos(NodeId(to_node)));
    let mid = match (fp, tp) {
        (Some(a), Some(b)) => Pos {
            x: (a.x + b.x) * 0.5,
            y: (a.y + b.y) * 0.5,
        },
        (Some(a), None) | (None, Some(a)) => a,
        (None, None) => Pos { x: 0.0, y: 0.0 },
    };
    let node = trial.add_node(adapter);
    trial.set_pos(node, mid);
    // Source -> adapter.in(0) -> consumer(to_port). Either failing (a cycle, an
    // occupied input) means we cannot splice cleanly — bail to the plain refusal.
    if trial
        .connect(Edge {
            from: (NodeId(from_node), from_port),
            to: (node, 0),
            delayed: false,
        })
        .is_err()
        || trial
            .connect(Edge {
                from: (node, 0),
                to: (NodeId(to_node), to_port),
                delayed: false,
            })
            .is_err()
    {
        return false;
    }
    // The two new edges are type-legal by construction of `CONVERSIONS`; validating
    // guards against a stale table or a manifest change (never ship a splice that the
    // cook would then reject).
    if trial.validate(&motion.registry).is_err() {
        return false;
    }

    let pre = motion.doc.clone();
    motion.doc.graph = trial;
    // Reconcile plumbs any managed `pre` the new node needs (the AddNode path's job),
    // then the pump re-cooks the changed graph.
    reconcile(motion, &pre.graph);
    motion.history.push_undo(pre);
    motion.pump.mark_dirty();
    // Select the new adapter so its params show at once — the artist lands on the
    // channel picker ("Read: Speed / Opacity / …"), ready to say what to convert.
    ph2d_panel_motion_graph::request_graph_selection(vec![node.0]);
    toasts.push(Toast::info("Inserted an adapter to convert the types"));
    true
}

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
mod tests {
    //! The lookup gate is a pure unit test; the two behavioural gates drive the REAL
    //! intent funnel (`apply_graph_intents` → `apply_connect`), not the doc directly.
    use super::super::apply_graph_intents;
    use super::*;
    use crate::motion_state::MotionState;
    use ph2d_panel_motion_graph::{GraphIntent, drain_intents, push_intent};

    /// The table answers the type question: a registered conversion resolves to its
    /// adapter; an unregistered pair (including same-type) resolves to nothing.
    /// FALSIFIED by a lookup that ignores the types (always Some / always None).
    #[test]
    fn adapter_for_only_answers_registered_conversions() {
        assert_eq!(adapter_for(STREAM, VALUE), Some("value.attribute"));
        assert_eq!(
            adapter_for(VALUE, STREAM),
            None,
            "the reverse is not registered"
        );
        assert_eq!(
            adapter_for(VALUE, VALUE),
            None,
            "same type never needs an adapter"
        );
    }

    /// A two-node doc plus its ids: `a` (an output) and `b` (an input). Fresh history.
    fn two(a: &str, b: &str) -> (MotionState, NodeId, NodeId) {
        let mut m = MotionState::new();
        m.doc = ph2d_motion_doc::MotionDoc::new();
        let na = m.doc.graph.add_node(a);
        let nb = m.doc.graph.add_node(b);
        (m, na, nb)
    }

    /// Push one `Connect` and run it through the funnel.
    fn connect(m: &mut MotionState, from: NodeId, from_port: u16, to: NodeId, to_port: u16) {
        let _ = drain_intents();
        push_intent(GraphIntent::Connect {
            from_node: from.0,
            from_port,
            to_node: to.0,
            to_port,
        });
        apply_graph_intents(
            m,
            &mut ph2d_core::Playhead::default(),
            &mut ph2d_editor::ToastQueue::default(),
            &mut ph2d_editor::screens::layout::CenterSplit::None,
        );
    }

    fn has_edge(m: &MotionState, from: NodeId, to: NodeId) -> bool {
        m.doc
            .graph
            .edges()
            .iter()
            .any(|e| e.from.0 == from && e.to.0 == to)
    }

    /// **Dragging a STREAM output into a VALUE input** — a pure type mismatch the
    /// substrate refuses — auto-inserts the `value.attribute` adapter between them
    /// (`grid -> attr -> gain`), as ONE undo step, and NO direct edge survives.
    /// FALSIFIED by the adapter path not firing (the wire is simply refused).
    #[test]
    fn a_stream_into_a_value_input_splices_the_attribute_adapter() {
        let (mut m, grid, gain) = two("motion.grid", "value.gain");
        assert!(!m.history.can_undo(), "a fresh state has nothing to undo");

        connect(&mut m, grid, 0, gain, 0);

        let attr = m
            .doc
            .graph
            .nodes()
            .iter()
            .find(|n| n.type_name == "value.attribute")
            .expect("the value.attribute adapter was inserted")
            .id;
        assert!(has_edge(&m, grid, attr), "grid feeds the adapter");
        assert!(
            has_edge(&m, attr, gain),
            "the adapter feeds the value input"
        );
        assert!(
            !has_edge(&m, grid, gain),
            "the incompatible direct edge does not exist"
        );
        assert!(m.history.can_undo(), "the whole splice is one undo step");
    }

    /// **The table is directional.** There is no adapter for VALUE → STREAM, so
    /// dragging a value output into a stream input is still refused: no node added, no
    /// edge. The negative control that proves the splice is keyed on the types.
    #[test]
    fn a_value_into_a_stream_input_is_still_refused() {
        // `value.attribute` has a STREAM input; `value.gain` has a VALUE output.
        let (mut m, gain, sink) = two("value.gain", "value.attribute");
        let before = m.doc.graph.nodes().len();

        connect(&mut m, gain, 0, sink, 0);

        assert_eq!(
            m.doc.graph.nodes().len(),
            before,
            "no adapter is inserted for an unregistered conversion"
        );
        assert!(!has_edge(&m, gain, sink), "the direct edge is refused");
        assert!(!m.history.can_undo(), "a refused wire is not an edit");
    }
}
