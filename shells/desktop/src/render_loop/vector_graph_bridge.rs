//! Vector Geometry-Graph smoke bridge (W3 T3.1) — the **producer** half of the
//! node-graph path that the W2 tool-direct render doesn't cover.
//!
//! Cooks the `vector.source` node from the panel's 8 param sliders
//! ([`ph2d_panel_vector_graph::current_graph_params`]) and draws the resulting
//! [`VectorNetwork`] into the shared vector scene via the canonical
//! [`draw_vector_network`] — so moving a slider re-cooks + re-renders live (the
//! Day-8 smoke). Gated behind the `vector_graph` panel being visible (the shell
//! makes it visible under the `PH2D_VECTOR_GRAPH=1` smoke flag), so the normal
//! app is untouched — mirror of `motion_smoke`'s debug-scaffolding discipline.
//!
//! Builds registry + graph + cook INLINE each frame: trivial for one node, and
//! it keeps the smoke self-contained (no state threaded through `init`).
//! Memoizing across frames (persisting the `Cook`) is the W3 perf follow-up.

use ph2d_host::WindowSize;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::Graph;
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

/// Per-frame: cook the `vector.source` node from the panel params and draw the
/// network. No-op unless `visible` (the geometry-graph panel is up). The caller
/// passes the panel's visibility so this never co-borrows the `HeroScreen`.
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

    // Build the one-node graph + register the op (lean: just vector.source).
    let mut reg = NodeRegistry::new();
    if ph2d_node_vector_source::register(&mut reg).is_err() {
        return;
    }
    let mut g = Graph::new();
    let node = g.add_node("vector.source");
    g.set_param(node, "kind", p.kind);
    g.set_param(node, "width", p.width);
    g.set_param(node, "height", p.height);
    g.set_param(node, "sides", p.sides);
    g.set_param(node, "inner_ratio", p.inner_ratio);
    g.set_param(node, "turns", p.turns);
    g.set_param(node, "samples_per_turn", p.samples_per_turn);
    g.set_param(node, "rotation", p.rotation);

    let mut cook = Cook::new();
    let Ok(out) = cook.cook(&g, &reg, node, 0.0) else {
        return;
    };
    let Some(net) = out
        .first()
        .and_then(|v| v.as_any())
        .and_then(|a| a.downcast_ref::<VectorNetwork>())
    else {
        return;
    };

    // The primitive emits geometry whose regions carry fill refs that resolve in
    // an asset's StyleTable — there's no asset here, so give every region one
    // visible default fill (the smoke renders the silhouette; styling is a
    // follow-up). Open paths (spiral) have no region → nothing fills (W3 stroke
    // pass is the follow-up).
    let mut styles = StyleTable::default();
    let fref = styles.insert_fill(FillSolid::default());
    let mut net = net.clone();
    for region in &mut net.regions {
        region.fill = Some(fref);
    }

    let world_to_screen = camera.world_to_screen_affine(window_size);
    draw_vector_network(vector_scene.inner_mut(), &net, &styles, world_to_screen);
}
