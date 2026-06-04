//! Vector Geometry-Graph smoke bridge (W3 T3.1 source → T3.3 boolean) — the
//! **producer** half of the node-graph path that the W2 tool-direct render
//! doesn't cover.
//!
//! Cooks a multi-node graph from the panel's param sliders
//! ([`ph2d_panel_vector_graph::current_graph_params`]) and draws the resulting
//! [`VectorNetwork`] into the shared vector scene via the canonical
//! [`draw_vector_network`] — so moving a slider re-cooks + re-renders live.
//!
//! - **T3.1:** `vector.source` from the 8 sliders.
//! - **T3.3 boolean smoke:** `source(a) + source(b = a rotated 45°) + boolean(op)`.
//!   The source node has no position param, so the second source is a rotated
//!   copy — two concentric shapes overlap and the boolean is visibly non-trivial
//!   (the classic rotated-polygon demo). `op` comes from `PH2D_VECTOR_BOOL_OP`
//!   (`0`=Union … `8`=Outline; default `0`). Independent 2-source authoring + an
//!   in-panel op dropdown are the panel follow-up (handoff §3.C); the exact
//!   engine (the geometry result) IS the cook output — the GPU SDF draft
//!   (handoff §3.A) is a real-time-preview optimization that needs its own ADR
//!   (`ph2d-vector` has no GPU compute layer yet), not part of this smoke.
//!
//! Gated behind the `vector_graph` panel being visible (the shell makes it
//! visible under the `PH2D_VECTOR_GRAPH=1` smoke flag), so the normal app is
//! untouched — mirror of `motion_smoke`'s debug-scaffolding discipline.
//!
//! Builds registry + graph + cook INLINE each frame: cheap for three nodes, and
//! it keeps the smoke self-contained (no state threaded through `init`).
//! Persisting the `Cook` (re-cook only on edit) is the W3 perf follow-up.

use ph2d_host::WindowSize;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_panel_vector_graph::VectorGraphParams;
use ph2d_render::Camera2d;
use ph2d_vector::{VectorScene, draw_vector_network};
use ph2d_vector_doc::{FillSolid, StyleTable, VectorNetwork};
use std::sync::OnceLock;

/// Is the geometry-graph smoke enabled? Read once from `PH2D_VECTOR_GRAPH=1`.
/// The shell drives the `vector_graph` panel's visibility from this (so the
/// sliders appear) — mirror of `motion_smoke::enabled`.
pub(super) fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_VECTOR_GRAPH").is_some_and(|v| v == "1"))
}

/// Per-frame: cook `source(a) + source(b) + boolean(op)` from the panel params
/// and draw the network. No-op unless `visible` (the geometry-graph panel is
/// up). The caller passes the panel's visibility so this never co-borrows the
/// `HeroScreen`.
pub(super) fn dispatch(
    visible: bool,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    if !visible {
        return;
    }
    let p = ph2d_panel_vector_graph::current_graph_params();
    let Some(mut net) = cook_boolean_smoke(&p, bool_op_from_env()) else {
        return;
    };

    // The boolean emits geometry whose regions carry fill refs that resolve in
    // an asset's StyleTable — there's no asset here, so give every region one
    // visible default fill (the smoke renders the silhouette; styling is a
    // follow-up). Open results have no region → nothing fills.
    let mut styles = StyleTable::default();
    let fref = styles.insert_fill(FillSolid::default());
    for region in &mut net.regions {
        region.fill = Some(fref);
    }

    let world_to_screen = camera.world_to_screen_affine(window_size);
    draw_vector_network(vector_scene.inner_mut(), &net, &styles, world_to_screen);
}

/// Cook `source(a) + source(b = a rotated 45°) + boolean(op)` and return the
/// result network, or `None` if registration / connection / cook / downcast
/// fails. Pure (no rendering) so the multi-node fan-in wiring is unit-testable —
/// the part the node's own cook tests don't cover.
fn cook_boolean_smoke(p: &VectorGraphParams, op: f32) -> Option<VectorNetwork> {
    let mut reg = NodeRegistry::new();
    ph2d_node_vector_source::register(&mut reg).ok()?;
    ph2d_node_vector_boolean::register(&mut reg).ok()?;

    let mut g = Graph::new();
    let source_a = add_source(&mut g, p, 0.0);
    let source_b = add_source(&mut g, p, std::f32::consts::FRAC_PI_4);
    let boolean = g.add_node("vector.boolean");
    g.set_param(boolean, "op", op);
    // a → boolean input 0, b → boolean input 1 (MANIFEST input order).
    g.connect(Edge {
        from: (source_a, 0),
        to: (boolean, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (source_b, 0),
        to: (boolean, 1),
        delayed: false,
    })
    .ok()?;

    let mut cook = Cook::new();
    let out = cook.cook(&g, &reg, boolean, 0.0).ok()?;
    Some(
        out.first()?
            .as_any()?
            .downcast_ref::<VectorNetwork>()?
            .clone(),
    )
}

/// Add a `vector.source` node seeded from the panel params, with `rotation`
/// offset by `rot` (so the two smoke sources overlap as rotated copies).
fn add_source(g: &mut Graph, p: &VectorGraphParams, rot: f32) -> NodeId {
    let node = g.add_node("vector.source");
    g.set_param(node, "kind", p.kind);
    g.set_param(node, "width", p.width);
    g.set_param(node, "height", p.height);
    g.set_param(node, "sides", p.sides);
    g.set_param(node, "inner_ratio", p.inner_ratio);
    g.set_param(node, "turns", p.turns);
    g.set_param(node, "samples_per_turn", p.samples_per_turn);
    g.set_param(node, "rotation", p.rotation + rot);
    node
}

/// `vector.boolean` `op` discriminant from `PH2D_VECTOR_BOOL_OP` (`0..=8`),
/// defaulting to `0` (Union). The node's `param_as_count` clamps out-of-range.
fn bool_op_from_env() -> f32 {
    std::env::var("PH2D_VECTOR_BOOL_OP")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooks_source_source_boolean_to_a_network() {
        // The multi-node fan-in (the wiring the boolean node's own cook tests
        // don't exercise): two sources feed the boolean's a/b inputs and the
        // graph cooks to a VectorNetwork. Union (op 0) of two overlapping
        // rotated shapes yields geometry.
        let net = cook_boolean_smoke(&VectorGraphParams::default(), 0.0)
            .expect("source + source + boolean must cook to a VectorNetwork");
        assert!(
            !net.vertices.is_empty(),
            "the boolean of two overlapping sources has geometry"
        );
    }

    #[test]
    fn each_op_discriminant_cooks() {
        // op 0..=8 all flow through the param + cook (clamped by the node).
        for op in 0..=8 {
            assert!(
                cook_boolean_smoke(&VectorGraphParams::default(), op as f32).is_some(),
                "boolean op {op} must cook"
            );
        }
    }
}
