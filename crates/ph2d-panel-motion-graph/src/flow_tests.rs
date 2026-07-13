//! Guards for the F3 readings (doc 46). `super` is `flow`.

use super::*;
use crate::snapshot::{GraphEdgeView, GraphNodeView, GraphViewSnapshot};
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
use ph2d_nodegraph::port::Domain;

fn node(id: u32, is_sink: bool, count: Option<u32>) -> GraphNodeView {
    GraphNodeView {
        kind: crate::snapshot::NodeViewKind::Node,
        id,
        display_name: format!("n{id}"),
        category: NodeUiCategory::Utility,
        silhouette: NodeSilhouette::Rect,
        x: 0.0,
        y: 0.0,
        inputs: Vec::new(),
        outputs: Vec::new(),
        readout: None,
        count,
        hot: false,
        is_sink,
        preview: None,
    }
}

fn edge(from_node: u32, to_node: u32, delayed: bool) -> GraphEdgeView {
    GraphEdgeView {
        from_node,
        from_port: 0,
        to_node,
        to_port: 0,
        delayed,
        out_domain: Domain::Instances,
    }
}

fn snap(nodes: Vec<GraphNodeView>, edges: Vec<GraphEdgeView>) -> GraphViewSnapshot {
    GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes,
        edges,
        probe: None,
        backdrops: Vec::new(),
        now: 0.0,
    }
}

/// **A branch that reaches no sink is inert** — the cook never pulls it, so nothing it does
/// reaches the canvas. `1 → 2 → OUT` is live; `3 → 4` (wired to each other and to nothing
/// else) is not, and neither is the wire between them.
///
/// FALSIFIED by walking FORWARDS from the sources, which marks everything a source feeds —
/// including every dead end — i.e. the exact opposite of the question.
#[test]
fn a_branch_that_reaches_no_sink_is_inert() {
    let s = snap(
        vec![
            node(1, false, None),
            node(2, false, None),
            node(9, true, None), // the sink
            node(3, false, None),
            node(4, false, None),
        ],
        vec![
            edge(1, 2, false),
            edge(2, 9, false),
            edge(3, 4, false), // an island: it feeds nothing that is consumed
        ],
    );
    let live = live_set(&s);
    assert!(live.contains(&1) && live.contains(&2) && live.contains(&9));
    assert!(
        !live.contains(&3) && !live.contains(&4),
        "the island is not cooked, so it is inert: {live:?}"
    );

    // And the WIRE reading follows from the same set: a wire is live iff what it FEEDS is.
    assert!(edge_is_live(&live, 9), "2 -> OUT carries the scene");
    assert!(!edge_is_live(&live, 4), "3 -> 4 goes nowhere");
}

/// A node feeding only a `pre` (delayed) head is **cooking** — the state loop is real
/// consumption, and the editor draws the loop as portal badges rather than a wire, which is no
/// reason to call the node dead.
///
/// FALSIFIED by skipping delayed edges in the walk: every sequential node (integrate, spring,
/// trail) would go dark the moment its only consumer was its own state loop.
#[test]
fn a_pre_edge_keeps_its_source_alive() {
    let s = snap(
        vec![node(1, false, None), node(9, true, None)],
        vec![edge(1, 9, true)],
    );
    assert!(live_set(&s).contains(&1), "a pre head consumes its source");
}

/// **The wire's width is the stream's mass.** A scalar VALUE wire is a thread; a heavy stream
/// is a cable; and it saturates rather than growing without bound.
#[test]
fn a_heavier_stream_draws_a_thicker_wire() {
    let thread = wire_width(Some(1));
    let some = wire_width(Some(200));
    let heavy = wire_width(Some(5_000));
    assert!(
        thread < some && some < heavy,
        "1 < 200 < 5000 instances: {thread} {some} {heavy}"
    );
    assert_eq!(
        heavy,
        wire_width(Some(50_000)),
        "past the reference mass it saturates - 'a lot' is the reading, not a bar chart"
    );
    assert_eq!(
        wire_width(None),
        thread.min(wire_width(None)),
        "an inert wire carries nothing and draws as a thread"
    );
}

/// **The dashes march**, and they march from the SOURCE towards the target: as the phase
/// grows, the pattern advances along the wire.
///
/// FALSIFIED by adding the phase instead of subtracting it (the flow would run backwards, and
/// a graph that reads left-to-right would show its data running right-to-left).
#[test]
fn the_dashes_march_from_source_to_target() {
    let line: Vec<(f32, f32)> = vec![(0.0, 0.0), (100.0, 0.0)];
    let at = |phase: f32| dashes(&line, 20.0, 8.0, phase)[1][0].0;

    let (t0, t1) = (at(0.0), at(4.0));
    assert!(
        (t1 - t0 - 4.0).abs() < 1e-3,
        "a phase of +4 advances the pattern by exactly 4 along the wire: {t0} -> {t1}"
    );
    // …and it is periodic: one whole period later the pattern is where it started.
    assert!((at(20.0) - t0).abs() < 1e-3, "one period on, it repeats");
}

/// A dash is cut out of the very polyline the wire is drawn along — so on a bent wire it BENDS
/// (it keeps the vertices it spans), and the lit fraction is the duty cycle it was asked for.
#[test]
fn dashes_follow_the_bend_and_light_the_duty_cycle() {
    let bend: Vec<(f32, f32)> = vec![(0.0, 0.0), (30.0, 0.0), (30.0, 30.0)];
    let segs = dashes(&bend, 20.0, 10.0, 5.0);
    let lit: f32 = segs
        .iter()
        .map(|s| {
            s.windows(2)
                .map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt())
                .sum::<f32>()
        })
        .sum();
    // 60 units of wire, 10-on / 20-period: 3 dashes of 10.
    assert!(
        (lit - 30.0).abs() < 1e-2,
        "half the wire is lit (10 on, 20 period): {lit}"
    );
    // The dash spanning the corner keeps the corner vertex, so it turns with the wire.
    let corner = segs
        .iter()
        .find(|s| s.contains(&(30.0, 0.0)))
        .expect("a dash spans the bend");
    assert!(corner.len() >= 3, "it kept the bend: {corner:?}");
}

/// A wire with no length (both ends on the same point — a node wired to itself before the
/// `pre` badge takes over) has no dashes, and does not divide by zero trying.
#[test]
fn a_degenerate_wire_has_no_dashes() {
    assert!(dashes(&[(5.0, 5.0), (5.0, 5.0)], 20.0, 8.0, 0.0).is_empty());
    assert!(dashes(&[(5.0, 5.0)], 20.0, 8.0, 0.0).is_empty());
}

// ── Influence: what the selection touches (doc 47) ───────────────────────────

/// **Selecting a node lights what feeds it and what it changes** — its ancestors AND its
/// descendants — and everything else recedes. This is the question you ask before you dare
/// edit a big graph: *if I touch this, what moves?*
///
/// FALSIFIED by walking only downstream (the artist would not see what feeds the node, so
/// "what will break if I unplug this" stays unanswered) and by walking only upstream.
#[test]
fn selecting_a_node_lights_what_feeds_it_and_what_it_changes() {
    //  1 -> 2 -> 3 -> OUT(9)      and a parallel branch  5 -> 6 -> OUT(9)
    let s = snap(
        vec![
            node(1, false, None),
            node(2, false, None),
            node(3, false, None),
            node(9, true, None),
            node(5, false, None),
            node(6, false, None),
        ],
        vec![
            edge(1, 2, false),
            edge(2, 3, false),
            edge(3, 9, false),
            edge(5, 6, false),
            edge(6, 9, false),
        ],
    );
    let inf = influence_set(&s, &BTreeSet::from([2]));

    assert!(inf.contains(&1), "what feeds 2 is in the influence");
    assert!(inf.contains(&3) && inf.contains(&9), "…and what 2 changes");
    // The sibling branch reaches the SAME output, and node 2 still cannot touch it. (It is
    // reachable through the sink if you ignore direction — which is the mutant.)
    assert!(
        !inf.contains(&5) && !inf.contains(&6),
        "the parallel branch is untouched by node 2: {inf:?}"
    );

    // A wire belongs to the influence only if BOTH ends do: 6 -> OUT lands on a node in the
    // set, and still carries nothing that 2 can change.
    assert!(edge_in_influence(&inf, 2, 3));
    assert!(
        !edge_in_influence(&inf, 6, 9),
        "a wire into the influence from outside is not part of it"
    );
}

/// With NOTHING selected, nothing is dimmed for influence — the canvas only ever veils what is
/// genuinely inert. (A graph that went grey the moment you clicked empty canvas would punish
/// the most common gesture there is.)
#[test]
fn an_empty_selection_dims_nothing() {
    let s = snap(
        vec![node(1, false, None), node(9, true, None)],
        vec![edge(1, 9, false)],
    );
    assert!(
        influence_set(&s, &BTreeSet::new()).is_empty(),
        "no selection, no focus"
    );
}
