//! Vector Geometry-Graph smoke bridge (W3 T3.1 source -> T3.3 boolean) — the
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
//!   (Divide/Trim/Merge/Crop) have no SDF -> always exact. The silhouette SDF runs
//!   on the GPU (`ph2d_vector_sdf::gpu::GpuSdf`, pipeline cached); the `min/max`
//!   combine + marching-squares stay on the CPU.

use ph2d_color::OklchColor;
use ph2d_color::srgb::linear_to_srgb_byte;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_panel_vector_graph::VectorGraphParams;
use ph2d_render::Camera2d;
use ph2d_vector::{
    Affine, BezPath, Brush, Color, Point, ProceduralFillImage, Scene, Stroke, VectorScene,
    draw_vector_network, draw_vector_network_with_fills,
};
use ph2d_vector_doc::{FillSolid, ProceduralFill, StyleTable, VectorNetwork};
use ph2d_vector_fill::ubo::FillParamsUbo;
use ph2d_vector_fill::{
    DiffusionCurve, DiffusionCurveSet, FieldStore, FillGraph, FillNode, NodeId as FillNodeId,
    Resolution, eval_color_with_fields, solve_color_field,
};
use ph2d_vector_sdf::gpu::GpuSdf;
use ph2d_vector_sdf::{Bounds, SdfOp, boolean_sdf, marching_contour};
use std::cell::RefCell;
use std::sync::{Arc, OnceLock};

/// Multiplier on the result's bounding box for the auto-framing camera height —
/// 1.5 = 50% headroom so the shape sits comfortably inside the viewport.
const FRAME_MARGIN: f32 = 1.5;
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

/// A rasterized procedural-fill smoke image: `(straight-sRGB8 rgba, width, height)`.
type SmokeImage = (Arc<Vec<u8>>, u32, u32);

thread_local! {
    /// Last `(params, op)` `dispatch` saw, so it can tell an active slider drag
    /// (params changing frame-to-frame) from a settled state. The render loop is
    /// single-threaded; this is the bridge's only per-frame state (handoff §2.B).
    static LAST_KEY: RefCell<Option<(VectorGraphParams, i64)>> = const { RefCell::new(None) };

    /// ADR-0065 Phase 3: the GPU SDF compute pipeline, built once on the first
    /// draft frame and reused (pipeline creation is cheap but not free). Single
    /// render thread -> thread-local. Holds wgpu handles (Arc-backed).
    static GPU_SDF: RefCell<Option<GpuSdf>> = const { RefCell::new(None) };

    /// W7 diffusion-mesh-gradient smoke image (`PH2D_VECTOR_FILL_SMOKE=1`):
    /// the canonical diffusion scene solved + encoded to straight-sRGB8 once.
    /// `(rgba, width, height)`.
    static DIFFUSION_SMOKE: RefCell<Option<SmokeImage>> = const { RefCell::new(None) };
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
    gpu: &GpuContext,
) {
    if !visible {
        return;
    }
    let p = ph2d_panel_vector_graph::current_graph_params();

    // W4 transform-node smoke: `PH2D_VECTOR_NODE=<slug>` (e.g. `roughen`, `mirror`,
    // `corner-round`) cooks source(sliders) -> `vector.<slug>` -> render, so each of
    // the 11 unary geometry nodes is visible on screen (no node-graph editor yet).
    // They have no boolean draft and cook cheaply (~0.05 ms), so render the exact
    // output every frame, auto-framed. When the flag is set this owns the frame.
    if let Some(slug) = node_slug() {
        if let Some(mut net) = cook_transform_smoke(&p, slug) {
            let to_screen = framing_affine(&net, &net, camera, window_size);
            render_filled(&mut net, to_screen, vector_scene.inner_mut());
        }
        return;
    }

    let op = bool_op_from_env();
    let op_key = op.round() as i64;

    // Cook the two smoke sources up front: needed for the auto-framing transform
    // and, on the draft path, for the per-operand SDF. The sources are ~100 world
    // units but the default camera shows only ~10 (`Camera2d::height_world`), so we
    // frame the *result* into view rather than trusting the live camera — otherwise
    // the shape fills the whole screen as a flat blob (the prior smoke's gotcha).
    let Some((net_a, net_b)) = smoke_sources(&p) else {
        return;
    };
    let to_screen = framing_affine(&net_a, &net_b, camera, window_size);

    // Draft+reconcile gate (ADR-0065 §2.2): while params are CHANGING and the op
    // has an SDF form, show the cheap silhouette draft; once they settle (≥1
    // stable frame), fall through to the exact reconcile below. The op (from env)
    // is constant per run, so `changed` tracks slider drags. Both paths share
    // `to_screen` so the shape doesn't jump when the drag settles.
    if let Some(sdf_op) = draft_op(params_changed((p, op_key)), op_key)
        && draw_draft(
            &net_a,
            &net_b,
            sdf_op,
            to_screen,
            vector_scene.inner_mut(),
            gpu,
        )
    {
        return;
    }

    let Some(mut net) = cook_boolean_smoke(&p, op) else {
        return;
    };
    render_filled(&mut net, to_screen, vector_scene.inner_mut());
}

/// Give every region a visible default fill and draw the network under
/// `to_screen`. The cooked geometry carries fill refs that resolve in an asset's
/// StyleTable — there's no asset in the smoke, so a default fill makes the
/// silhouette visible (styling is a follow-up). Open results (no region) draw
/// nothing. Shared by the boolean + transform smoke paths.
///
/// `PH2D_VECTOR_FILL_SMOKE=1` swaps the flat fill for the W7 diffusion-curve mesh
/// gradient (ADR-0056-amendment-3): the region resolves to a `ProceduralFill`,
/// the diffusion field is solved + rendered as an image clipped to the region.
fn render_filled(net: &mut VectorNetwork, to_screen: Affine, scene: &mut Scene) {
    if procedural_fill_smoke_enabled() {
        render_filled_diffusion(net, to_screen, scene);
        return;
    }
    let mut styles = StyleTable::default();
    let fref = styles.insert_fill(FillSolid::default());
    for region in &mut net.regions {
        region.fill = Some(fref);
    }
    draw_vector_network(scene, net, &styles, to_screen);
}

/// `true` if the W7 diffusion-fill smoke is enabled (`PH2D_VECTOR_FILL_SMOKE=1`).
fn procedural_fill_smoke_enabled() -> bool {
    std::env::var("PH2D_VECTOR_FILL_SMOKE").is_ok_and(|v| v == "1")
}

/// Fill every region with the W7 diffusion mesh gradient (procedural fill path):
/// resolves through `StyleTable::procedural` → renders the solved `ColorField` as
/// an image clipped to the region. Proves the contract → resolve → render chain.
fn render_filled_diffusion(net: &mut VectorNetwork, to_screen: Affine, scene: &mut Scene) {
    let mut styles = StyleTable::default();
    // Fallback (no-GPU / unresolved) is a neutral grey; the image is the real fill.
    let fref = styles.insert_procedural(ProceduralFill::diffusion(
        0,
        OklchColor::opaque(0.5, 0.0, 0.0),
    ));
    for region in &mut net.regions {
        region.fill = Some(fref);
    }
    let (rgba, width, height) = diffusion_smoke_rgba();
    draw_vector_network_with_fills(scene, net, &styles, to_screen, move |fr| {
        (fr == fref).then(|| ProceduralFillImage {
            rgba: rgba.clone(),
            width,
            height,
        })
    });
}

/// The diffusion smoke image, solved + encoded once (cached thread-local).
fn diffusion_smoke_rgba() -> SmokeImage {
    DIFFUSION_SMOKE.with(|cell| {
        cell.borrow_mut()
            .get_or_insert_with(build_diffusion_smoke)
            .clone()
    })
}

/// Build the smoke image through the **W6 procedural-fill graph** (the general
/// path the Vector impl asked the host to wire): solve the canonical diffusion
/// scene into a `ColorField`, register it in a [`FieldStore`] (the host's
/// `FieldResolver`), then eval a `FillGraph` = `[MeshGradient { gradient_id: 0 }]`
/// per pixel — `eval_color_with_fields` samples the field via the store. Encodes
/// straight-sRGB8 OPAQUE (the region clip provides coverage). The visual is the
/// diffusion gradient; the *path* is now `FillGraph → FieldResolver → render`, so
/// any FillGraph (gradient/noise/mix/mesh) renders the same way.
fn build_diffusion_smoke() -> SmokeImage {
    use glam::Vec2;
    let red = OklchColor::opaque(0.63, 0.26, 29.0);
    let blue = OklchColor::opaque(0.45, 0.31, 264.0);
    let green = OklchColor::opaque(0.70, 0.20, 142.0);
    let amber = OklchColor::opaque(0.82, 0.16, 75.0);
    let set = DiffusionCurveSet::from_curves([
        DiffusionCurve::straight(Vec2::new(0.5, 0.0), Vec2::new(0.5, 1.0), red, blue),
        DiffusionCurve::straight(Vec2::new(0.1, 0.22), Vec2::new(0.9, 0.22), green, green),
        DiffusionCurve::straight(Vec2::new(0.55, 0.95), Vec2::new(0.95, 0.55), amber, amber),
    ]);
    let field = solve_color_field(&set, Resolution::square(129).expect("129 = 2^7 + 1"));

    // Host-side FieldResolver: gradient_id 0 → the solved field.
    let mut store = FieldStore::new();
    store.insert(0, field);

    // The minimal W6 graph: a single MeshGradient sampling gradient_id 0.
    let graph = FillGraph {
        nodes: vec![FillNode::MeshGradient { gradient_id: 0 }].into(),
        connections: Vec::new().into(),
        output_node_id: FillNodeId(0),
    };
    let params = FillParamsUbo::from_graph(&graph);

    // Eval the graph per pixel over a 129² grid (uv ∈ [0,1]²) → straight sRGB8.
    let (w, h) = (129u32, 129u32);
    let mut rgba = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for y in 0..h {
        for x in 0..w {
            let uv = Vec2::new(x as f32 / (w - 1) as f32, y as f32 / (h - 1) as f32);
            let c =
                eval_color_with_fields(&graph, uv, &params, &store).unwrap_or([0.0, 0.0, 0.0, 0.0]);
            rgba.push(linear_to_srgb_byte(c[0]));
            rgba.push(linear_to_srgb_byte(c[1]));
            rgba.push(linear_to_srgb_byte(c[2]));
            rgba.push(255); // opaque; the region path provides coverage via the clip
        }
    }
    (Arc::new(rgba), w, h)
}

/// Has the `(params, op)` key changed since the previous frame? Records `key`
/// for next frame. A change means a slider is being dragged -> show the draft;
/// equal for two frames means settled -> show the exact reconcile.
fn params_changed(key: (VectorGraphParams, i64)) -> bool {
    LAST_KEY.with(|cell| {
        let mut slot = cell.borrow_mut();
        let changed = slot.as_ref() != Some(&key);
        *slot = Some(key);
        changed
    })
}

/// The draft+reconcile decision: draft only while params are `changed` AND the
/// op has an SDF form. `None` -> the caller shows the exact reconcile.
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

/// Draw the SDF *draft* of `source(a) ∘ source(b)`: SDF each operand on the GPU
/// over a co-located window (ADR-0065 Phase 3 — the real-time path), combine with
/// the `min/max` kernel on the CPU, march out the zero contour, and stroke it as
/// a thin preview outline (under `to_screen`). Returns `false` (so the caller
/// falls back to the exact reconcile) if the grids don't co-locate or the contour
/// is empty.
fn draw_draft(
    net_a: &VectorNetwork,
    net_b: &VectorNetwork,
    op: SdfOp,
    to_screen: Affine,
    scene: &mut Scene,
    gpu: &GpuContext,
) -> bool {
    // Co-locate the sampling window over BOTH operands (same res + bounds) so
    // `boolean_sdf` can combine them cell-for-cell.
    let bounds_a = Bounds::of_network(net_a, DRAFT_PAD);
    let bounds_b = Bounds::of_network(net_b, DRAFT_PAD);
    let bounds = Bounds {
        min: bounds_a.min.min(bounds_b.min),
        max: bounds_a.max.max(bounds_b.max),
    };
    // GPU silhouette per operand, reusing the cached compute pipeline. The trivial
    // `min/max` combine + marching stay on the CPU (GpuSdf handles device-loss by
    // returning a flat field, so this never panics on a bad frame).
    let (sdf_a, sdf_b) = GPU_SDF.with(|cell| {
        let mut slot = cell.borrow_mut();
        let pipe = slot.get_or_insert_with(|| GpuSdf::new(gpu));
        (
            pipe.network_sdf(gpu, net_a, DRAFT_RES, bounds),
            pipe.network_sdf(gpu, net_b, DRAFT_RES, bounds),
        )
    });
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
        to_screen,
        &Brush::Solid(color),
        None,
        &path,
    );
    true
}

/// Cook the two smoke operands — `source(a)` and `source(b = a rotated 45°)` —
/// once per frame, shared by the auto-framing transform and the draft SDF.
fn smoke_sources(p: &VectorGraphParams) -> Option<(VectorNetwork, VectorNetwork)> {
    Some((
        cook_source(p, 0.0)?,
        cook_source(p, std::f32::consts::FRAC_PI_4)?,
    ))
}

/// A world->screen transform that frames the two operands' combined bounds into
/// the viewport, centered with [`FRAME_MARGIN`] headroom. Sidesteps the live
/// camera (default `height_world` 10 ≪ the ~100-unit smoke shapes) so the result
/// is always visible at a sane size, whatever the sliders say.
fn framing_affine(
    net_a: &VectorNetwork,
    net_b: &VectorNetwork,
    camera: &Camera2d,
    window_size: WindowSize,
) -> Affine {
    let bounds_a = Bounds::of_network(net_a, 0.0);
    let bounds_b = Bounds::of_network(net_b, 0.0);
    let min = bounds_a.min.min(bounds_b.min);
    let max = bounds_a.max.max(bounds_b.max);
    let center = (min + max) * 0.5;
    let span = max - min;
    let mut cam = *camera;
    cam.center = [center.x, center.y];
    // Bypass `Camera2d::zoom`'s clamp (≤100) by setting the field directly — the
    // smoke shapes routinely exceed it and this transform never touches the real
    // camera.
    cam.height_world = (span.x.max(span.y) * FRAME_MARGIN).max(1.0);
    cam.world_to_screen_affine(window_size)
}

/// Cook a single `vector.source` node seeded from the panel params (rotation
/// offset by `rot`) -> its `VectorNetwork`, for the SDF draft's per-operand
/// silhouette. Mirrors `cook_boolean_smoke`'s two source nodes.
fn cook_source(p: &VectorGraphParams, rot: f32) -> Option<VectorNetwork> {
    let mut reg = NodeRegistry::new();
    ph2d_node_vector_source::register(&mut reg).ok()?;
    let mut g = Graph::new();
    let node = add_source(&mut g, p, rot);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &reg, node, 0.0).ok()?;
    Some(
        out.first()?
            .as_any()?
            .downcast_ref::<VectorNetwork>()?
            .clone(),
    )
}

/// Cook `source(sliders) -> vector.<slug>` (a W4 unary geometry node) with the
/// node's default params, returning its output network. Uses the all-nodes
/// registry so any node type-id resolves without a per-node dep; an unknown slug
/// fails the cook -> `None` (caller draws nothing). The transform uses MANIFEST
/// defaults — moving the source sliders drives the input shape so the effect is
/// visible live.
fn cook_transform_smoke(p: &VectorGraphParams, slug: &str) -> Option<VectorNetwork> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).ok()?;
    let mut g = Graph::new();
    let source = add_source(&mut g, p, 0.0);
    let node = g.add_node(format!("vector.{slug}"));
    g.connect(Edge {
        from: (source, 0),
        to: (node, 0),
        delayed: false,
    })
    .ok()?;
    let mut cook = Cook::new();
    let out = cook.cook(&g, &reg, node, 0.0).ok()?;
    Some(
        out.first()?
            .as_any()?
            .downcast_ref::<VectorNetwork>()?
            .clone(),
    )
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
    // a -> boolean input 0, b -> boolean input 1 (MANIFEST input order).
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

/// W4 transform-smoke slug from `PH2D_VECTOR_NODE` (e.g. `roughen`, `mirror`,
/// `corner-round`, `bend-path`, `outline-stroke`, `width-profile`, `twist`,
/// `scatter`, `hatch`, `warp`, `recolor`). Read once. `None` -> boolean smoke.
fn node_slug() -> Option<&'static str> {
    static SLUG: OnceLock<Option<String>> = OnceLock::new();
    SLUG.get_or_init(|| {
        std::env::var("PH2D_VECTOR_NODE")
            .ok()
            .filter(|s| !s.is_empty())
    })
    .as_deref()
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
                "op {op} is topology-only / invalid -> exact"
            );
        }
    }

    #[test]
    fn draft_gate_needs_both_changing_and_sdf() {
        // The decision the per-frame state machine drives.
        assert!(
            matches!(draft_op(true, 0), Some(SdfOp::Union)),
            "changing + SDF op -> draft"
        );
        assert!(
            draft_op(false, 0).is_none(),
            "settled -> exact even for an SDF op"
        );
        assert!(
            draft_op(true, 4).is_none(),
            "changing + Divide (topology-only) -> exact"
        );
        assert!(
            matches!(draft_op(true, 8), Some(SdfOp::Outline { .. })),
            "changing + Outline -> draft"
        );
    }

    #[test]
    fn cook_source_yields_geometry() {
        let net = cook_source(&VectorGraphParams::default(), 0.0)
            .expect("a single vector.source must cook to a network");
        assert!(!net.vertices.is_empty(), "a source shape has vertices");
    }
}
