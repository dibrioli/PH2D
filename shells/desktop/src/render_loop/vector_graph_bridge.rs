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
//!
//! - **T3.1 SDF draft+reconcile (ADR-0065 Phase 3):** while a slider is actively
//!   moving the params **and** the op has an SDF form (Union/Subtract/Intersect/
//!   Exclude/Outline), draw the cheap [`ph2d_vector_sdf`] silhouette contour (the
//!   *draft*) instead of cooking the exact boolean. The frame the params settle,
//!   fall back to the exact Linesweeper reconcile. The four topology-only ops
//!   (Divide/Trim/Merge/Crop) have no SDF → always exact. CPU SDF at draft-res is
//!   the least-friction path (the bridge has no `GpuContext` threaded through);
//!   GPU SDF is the scale follow-up.

use ph2d_host::WindowSize;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_panel_vector_graph::VectorGraphParams;
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Scene, Stroke, VectorScene, draw_vector_network};
use ph2d_vector_doc::{FillSolid, StyleTable, VectorNetwork};
use ph2d_vector_sdf::{Bounds, SdfOp, boolean_sdf, marching_contour, network_sdf};
use std::cell::RefCell;
use std::sync::OnceLock;

/// Draft SDF grid resolution per side. 96² is ~1 ms on CPU for the smoke's
/// few-edge sources — enough contour fidelity for a drag preview (ADR-0065 §2.2).
const DRAFT_RES: u32 = 96;
/// World-pixel margin around the silhouette so it never clips the grid edge.
const DRAFT_PAD: f32 = 16.0;
/// Half-width (world px) of the `Outline` op's draft band (the op carries no
/// radius param in the smoke).
const DRAFT_OUTLINE_RADIUS: f32 = 6.0;
/// Draft contour stroke width in world px — a thin "this is a preview" outline.
const DRAFT_STROKE_WORLD: f64 = 1.5;
/// Draft contour color: a distinct accent so the preview reads as not-final.
const DRAFT_RGB: (u8, u8, u8) = (79, 195, 247);
const DRAFT_ALPHA: u8 = 230;

thread_local! {
    /// Last `(params, op)` `dispatch` saw, so it can tell an active slider drag
    /// (params changing frame-to-frame) from a settled state. The render loop is
    /// single-threaded; this is the bridge's only per-frame state (handoff §2.B).
    static LAST_KEY: RefCell<Option<(VectorGraphParams, i64)>> = const { RefCell::new(None) };
}

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
    let op = bool_op_from_env();
    let op_key = op.round() as i64;
    let world_to_screen = camera.world_to_screen_affine(window_size);

    // Draft+reconcile gate (ADR-0065 §2.2): while params are CHANGING and the op
    // has an SDF form, show the cheap silhouette draft; once they settle (≥1
    // stable frame), fall through to the exact reconcile below. The op (from env)
    // is constant per run, so `changed` tracks slider drags.
    if let Some(sdf_op) = draft_op(params_changed((p, op_key)), op_key)
        && draw_draft(&p, sdf_op, world_to_screen, vector_scene.inner_mut())
    {
        return;
    }

    let Some(mut net) = cook_boolean_smoke(&p, op) else {
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

    draw_vector_network(vector_scene.inner_mut(), &net, &styles, world_to_screen);
}

/// Has the `(params, op)` key changed since the previous frame? Records `key`
/// for next frame. A change means a slider is being dragged → show the draft;
/// equal for two frames means settled → show the exact reconcile.
fn params_changed(key: (VectorGraphParams, i64)) -> bool {
    LAST_KEY.with(|cell| {
        let mut slot = cell.borrow_mut();
        let changed = slot.as_ref() != Some(&key);
        *slot = Some(key);
        changed
    })
}

/// The draft+reconcile decision: draft only while params are `changed` AND the
/// op has an SDF form. `None` → the caller shows the exact reconcile.
fn draft_op(changed: bool, op: i64) -> Option<SdfOp> {
    if changed { sdf_op_from_index(op) } else { None }
}

/// Map a `vector.boolean` op discriminant (`0..=8`, declaration order) to its
/// SDF-representable form, or `None` for the four topology-only ops
/// (`Divide`/`Trim`/`Merge`/`Crop`) and any out-of-range value — those have no
/// silhouette SDF and must always render the exact reconcile (ADR-0065 §2.1).
fn sdf_op_from_index(op: i64) -> Option<SdfOp> {
    match op {
        0 => Some(SdfOp::Union),
        1 => Some(SdfOp::Subtract),
        2 => Some(SdfOp::Intersect),
        3 => Some(SdfOp::Exclude),
        8 => Some(SdfOp::Outline {
            radius: DRAFT_OUTLINE_RADIUS,
        }),
        _ => None,
    }
}

/// Draw the SDF *draft* of `source(a) + source(b) ∘ op`: SDF each operand over a
/// co-located window, combine with the `min/max` kernel, march out the zero
/// contour, and stroke it as a thin preview outline. Returns `false` (so the
/// caller falls back to the exact reconcile) if a source fails to cook or the
/// grids don't co-locate.
fn draw_draft(p: &VectorGraphParams, op: SdfOp, world_to_screen: Affine, scene: &mut Scene) -> bool {
    let Some(net_a) = cook_source(p, 0.0) else {
        return false;
    };
    let Some(net_b) = cook_source(p, std::f32::consts::FRAC_PI_4) else {
        return false;
    };
    // Co-locate the sampling window over BOTH operands (same res + bounds) so
    // `boolean_sdf` can combine them cell-for-cell.
    let ba = Bounds::of_network(&net_a, DRAFT_PAD);
    let bb = Bounds::of_network(&net_b, DRAFT_PAD);
    let bounds = Bounds {
        min: ba.min.min(bb.min),
        max: ba.max.max(bb.max),
    };
    let sdf_a = network_sdf(&net_a, DRAFT_RES, bounds);
    let sdf_b = network_sdf(&net_b, DRAFT_RES, bounds);
    let Some(draft) = boolean_sdf(&sdf_a, &sdf_b, op) else {
        return false;
    };
    let segs = marching_contour(&draft);
    if segs.is_empty() {
        return false;
    }

    let mut path = BezPath::new();
    for s in &segs {
        path.move_to(Point::new(f64::from(s[0].x), f64::from(s[0].y)));
        path.line_to(Point::new(f64::from(s[1].x), f64::from(s[1].y)));
    }
    let color = Color::from_rgba8(DRAFT_RGB.0, DRAFT_RGB.1, DRAFT_RGB.2, DRAFT_ALPHA);
    scene.stroke(
        &Stroke::new(DRAFT_STROKE_WORLD),
        world_to_screen,
        &Brush::Solid(color),
        None,
        &path,
    );
    true
}

/// Cook a single `vector.source` node seeded from the panel params (rotation
/// offset by `rot`) → its `VectorNetwork`, for the SDF draft's per-operand
/// silhouette. Mirrors `cook_boolean_smoke`'s two source nodes.
fn cook_source(p: &VectorGraphParams, rot: f32) -> Option<VectorNetwork> {
    let mut reg = NodeRegistry::new();
    ph2d_node_vector_source::register(&mut reg).ok()?;
    let mut g = Graph::new();
    let node = add_source(&mut g, p, rot);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &reg, node, 0.0).ok()?;
    Some(out.first()?.as_any()?.downcast_ref::<VectorNetwork>()?.clone())
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

    #[test]
    fn only_five_ops_have_an_sdf_draft() {
        // Union/Subtract/Intersect/Exclude/Outline draft; the four topology-only
        // ops (Divide/Trim/Merge/Crop) and out-of-range values never do.
        for op in [0, 1, 2, 3, 8] {
            assert!(sdf_op_from_index(op).is_some(), "op {op} has an SDF form");
        }
        for op in [4, 5, 6, 7, 99, -1] {
            assert!(
                sdf_op_from_index(op).is_none(),
                "op {op} is topology-only / invalid → exact"
            );
        }
    }

    #[test]
    fn draft_gate_needs_both_changing_and_sdf() {
        // The decision the per-frame state machine drives.
        assert!(
            matches!(draft_op(true, 0), Some(SdfOp::Union)),
            "changing + SDF op → draft"
        );
        assert!(draft_op(false, 0).is_none(), "settled → exact even for an SDF op");
        assert!(
            draft_op(true, 4).is_none(),
            "changing + Divide (topology-only) → exact"
        );
        assert!(
            matches!(draft_op(true, 8), Some(SdfOp::Outline { .. })),
            "changing + Outline → draft"
        );
    }

    #[test]
    fn cook_source_yields_geometry() {
        let net = cook_source(&VectorGraphParams::default(), 0.0)
            .expect("a single vector.source must cook to a network");
        assert!(!net.vertices.is_empty(), "a source shape has vertices");
    }
}
