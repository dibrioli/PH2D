#![forbid(unsafe_code)]
//! `motion.duplicator` — **stamp a shape at every point** (doc 86 §2, the
//! reference's `Duplicator`).
//!
//! Two inputs, `shape` and `points`, and the output is their **cartesian
//! product**: one copy of every shape at every point. It is Houdini's *Copy to
//! Points* and Cavalry's *Duplicator* — the node that turns *what to draw* (a
//! `source.object`'s sprite tile) and *where to draw it* (a `motion.grid`,
//! `scatter`, `distribute-*` — any stream of positions) into the instanced set
//! the render sink draws.
//!
//! ## What comes from which input, and why
//!
//! The **shape carries the appearance** — `texture_id` (which texture), `uv_rect`
//! (its atlas cell), `size`, `tint`, and everything else it holds. The **point
//! carries the placement** — its `P` is ADDED to the shape's `P`, and its `rot`
//! to the shape's `rot`. So a `source.object` emitting one sprite tile at the
//! origin, crossed with a grid of `N` points, is that sprite stamped `N` times,
//! one per grid cell. Point columns other than `P`/`rot` are the placement's
//! business, not the appearance's, so they do not override the shape (a
//! per-point tint would be a future extension — the shape is the template).
//!
//! `Index`/`Count` are renumbered **continuous across the whole product** (`0..
//! shapes·points`) so a downstream index-driven effect (a colour ramp, a
//! stagger) flows over the entire stamped set rather than restarting per copy —
//! exactly as `motion.clone` does for its multiplied copies.
//!
//! ## Degenerate inputs
//!
//! - **No points** → pass the shapes through unchanged (a duplicator with
//!   nowhere to stamp is a passthrough, matching the reference). An empty
//!   `points` input is the same as an unconnected one.
//! - **No shapes** → empty (there is nothing to stamp).
//!
//! `Effect::Pure` (a drop-crate, contract untouched): the product is a pure
//! function of the two input streams. **NOT** `motion.clone` — the clone is a
//! polar multiplier of ONE stream; this crosses TWO. CPU-only (it changes the
//! element count, which is structural, not a per-element map).

use ph2d_node_registry::{NodeRegistry, ParamUiHint, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec, RECOMMENDED_MAX_ELEMENTS,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.duplicator"),
    name: "motion.duplicator",
    inputs: &[
        PortSpec {
            name: "shape",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "points",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    // Changes the element count (shapes → shapes·points), which is structural,
    // not a per-element `ph2d-expr` map an `eval_column` could lower; and no
    // Instances-domain WGSL runtime exists. CPU-only, like `motion.clone`.
    lowerings: &[LoweringKind::Cpu],
};

/// The names `P` and `rot` sum both inputs; every other shape column replicates.
/// `Index`/`Count` are rebuilt continuous. Anything left is appearance.
const P: &str = "P";
const ROT: &str = "rot";
const INDEX: &str = "Index";
const COUNT: &str = "Count";

/// Vec2 column values at index `i`, or `[0, 0]` if absent/short — the neutral
/// element for the `P` sum, so a shape or a point without a `P` contributes
/// nothing to the offset (an unconnected input reads as the origin).
fn p_at(c: Option<&Column>, i: usize) -> [f32; 2] {
    match c {
        Some(Column::Vec2(v)) => v.get(i).copied().unwrap_or([0.0, 0.0]),
        _ => [0.0, 0.0],
    }
}

/// Scalar column value at `i`, or `0.0` — the neutral element for the `rot` sum.
fn rot_at(c: Option<&Column>, i: usize) -> f32 {
    match c {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Spread a **shape** column across the product: element `(si, pi)` (shape-major
/// order — shape outer, point inner) takes the shape's value at `si`. This is
/// how the appearance replicates — the same tile/size/tint on every stamp.
fn spread(col: &Column, ns: usize, np: usize) -> Column {
    fn go<T: Copy>(v: &[T], ns: usize, np: usize) -> Vec<T> {
        let mut out = Vec::with_capacity(ns * np);
        for &val in v.iter().take(ns) {
            for _ in 0..np {
                out.push(val);
            }
        }
        out
    }
    match col {
        Column::Scalar(v) => Column::Scalar(go(v, ns, np)),
        Column::Vec2(v) => Column::Vec2(go(v, ns, np)),
        Column::Vec3(v) => Column::Vec3(go(v, ns, np)),
        Column::Vec4(v) => Column::Vec4(go(v, ns, np)),
    }
}

/// Clamp the point count so the product `shapes·points` stays within `max` (and
/// so the multiplication can never overflow the allocation). Never grows the
/// request; with `ns == 0` the product is empty anyway.
fn points_within_budget(ns: usize, np: usize, max: usize) -> usize {
    if ns == 0 {
        return 0;
    }
    np.min(max / ns.max(1))
}

/// Stamp `shape` at every `point`: for each shape × point, take the shape's
/// appearance columns unchanged and add the point's `P`/`rot`. Pure and
/// isolated so the product order, the offset sum, and the `Index`/`Count`
/// renumbering are unit-tested directly. `np` must already be budget-clamped.
fn duplicate(shape: &Stream, points: &Stream, np: usize) -> Stream {
    let ns = shape.count();
    // No points: the duplicator has nowhere to stamp → pass the shapes through
    // (the reference's `points`-less behaviour). Cloning is refcount, not copy.
    if np == 0 {
        return shape.clone();
    }
    let total = ns * np;
    let shape_p = shape.get(P);
    let point_p = points.get(P);
    let shape_rot = shape.get(ROT);
    let point_rot = points.get(ROT);
    let has_rot = shape_rot.is_some() || point_rot.is_some();

    let mut out = Stream::new(total);

    // `P` always exists on an Instances stream (the port type guarantees Vec2);
    // sum shape + point in shape-major order.
    let mut pos = Vec::with_capacity(total);
    for si in 0..ns {
        let sp = p_at(shape_p, si);
        for pi in 0..np {
            let pp = p_at(point_p, pi);
            pos.push([sp[0] + pp[0], sp[1] + pp[1]]);
        }
    }
    out.set(P, Column::Vec2(pos));

    // `rot` only if either side authored one (else the lowering's default 0 is
    // the right answer, and an empty column would be noise).
    if has_rot {
        let mut rot = Vec::with_capacity(total);
        for si in 0..ns {
            let sr = rot_at(shape_rot, si);
            for pi in 0..np {
                rot.push(sr + rot_at(point_rot, pi));
            }
        }
        out.set(ROT, Column::Scalar(rot));
    }

    // Continuous global index/count so a downstream ramp reads one 0..total run.
    out.set(
        INDEX,
        Column::Scalar((0..total).map(|i| i as f32).collect()),
    );
    out.set(COUNT, Column::Scalar(vec![total as f32; total]));

    // Every OTHER shape column is appearance — replicate it across the points.
    for (name, col) in shape.columns() {
        if matches!(name.as_str(), P | ROT | INDEX | COUNT) {
            continue;
        }
        out.set(name.clone(), spread(col, ns, np));
    }
    out
}

struct MotionDuplicator;

impl NodeOp for MotionDuplicator {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let shape = ctx.input(0);
        let points = ctx.input(1);
        let np = points_within_budget(shape.count(), points.count(), RECOMMENDED_MAX_ELEMENTS);
        let out = duplicate(shape, points, np);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionDuplicator))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Duplicator",
            category: ph2d_node_registry::NodeUiCategory::Distribute,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // Both inputs are REQUIRED (ADR-0155): with no `shape` there is nothing to copy, with
    // no `points` there is nowhere to put it — either empty and the node is a silent no-op.
    // A PORT requirement, not a column one, so it is declared (unlike `motion.integrate`,
    // whose `forces` port is optional — a static integration).
    reg.register_required_inputs(MANIFEST.id, &["shape", "points"]);
    Ok(())
}

/// No params (the count is `shapes·points`, automatic) — an empty hint list
/// keeps the registration shape identical to every other node.
static PARAM_HINTS: &[ParamUiHint] = &[];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
