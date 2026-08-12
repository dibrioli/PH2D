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
//! `Index`/`Count` are renumbered **continuous across everything stamped** so a
//! downstream index-driven effect (a colour ramp, a stagger) flows over the whole
//! set rather than restarting per copy — exactly as `motion.clone` does.
//!
//! ## The variant mode (doc 89 folha 08 — the P0)
//!
//! `pick` chooses **which shape lands on a point**: `Off` is the product above,
//! `Cycle` deals the shapes around the points (`id mod shapes`) and `Random`
//! scatters them by a seeded hash. In the two variant modes the output is one
//! stamp **per point**, not the product.
//!
//! ⚠️ The pick is a property of the POINT — its `Index` column, falling back to
//! its position — which is Blender's `id` default and Houdini's `Piece Attribute`
//! in one sentence: run the points through a `motion.sort` and each keeps the
//! shape it had, instead of the shapes sliding along the reordered list.
//!
//! ⚠️ **The two modes differ in ONE place**, the `(shape, point)` pair list; the
//! `P`/`rot` sums, the renumbering and the appearance spread all read that list
//! and cannot tell which mode built it. A second stamping loop is how *"Random
//! forgot to renumber"* is born six months from now.
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

use ph2d_node_registry::{NodeRegistry, ParamGate, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
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
    params: &[
        // 0 Off (the cartesian product) · 1 Cycle · 2 Random. See [`Pick`].
        ParamSpec {
            name: "pick",
            default: 0.0,
        },
        // Which random assignment, in Random mode. Gated to it: in Off/Cycle the
        // cook never reads it, and a knob the cook never reads is a dead control.
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
    ],
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

/// **Which shape lands on a point** (doc 89 folha 08 — the P0).
///
/// The cartesian product is the right DEFAULT (it is Houdini's), and what was
/// missing is the variant mode: four tools ship one and we shipped none —
/// Blender's *Instance on Points* has `Pick Instances` + `Instance Index`
/// (defaulting to `id`, wrapping in both directions), Houdini's *Copy to Points*
/// a `Piece Attribute`, Cavalry's Duplicator `Auto Id`/`Shape Id`, C4D `Modify
/// Clone`. With it off nothing about this node moves.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Pick {
    /// Every shape at every point — the product, and the default.
    Off,
    /// Point `i` takes shape `i mod shapes` — the reference default in all four
    /// tools, and the one that reads as *"deal these three around the grid"*.
    Cycle,
    /// Point `i` takes a shape chosen by a stable hash of its id and the seed.
    ///
    /// ⚠️ This one is not decoration: our substrate has **no way to write a named
    /// column**, so an artist cannot build the index field the references let you
    /// wire into `Instance Index`. Without a Random mode the variant is only ever
    /// a deterministic repeat and there is no escape at all.
    Random,
}

impl Pick {
    /// From the `pick` param. An out-of-range index falls back to `Off` — the
    /// product is the honest answer to a mode nobody asked for.
    #[must_use]
    pub fn of(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Cycle,
            2 => Self::Random,
            _ => Self::Off,
        }
    }
}

/// A stable per-instance hash in `[0, 1)` from an id + seed — integer ops only
/// (`wrapping_mul`/`xor`/`shift`), so it is bit-identical anywhere: `x >> 8` is
/// `<= 2^24 - 1` and exactly representable, and `/ 2^24` is exact.
///
/// ⚠️ **Local on purpose, and not a second door.** This is the leaf-crate mirror
/// idiom the repo already runs on — twenty-eight files carry their own copy of
/// this constant, each an INDEPENDENT random that nothing requires to agree with
/// another. Two nodes sharing one stream of numbers would be the surprising
/// thing, not the reverse; consolidating them is a repo-wide call, not this
/// node's to make silently.
fn hash01(id: u32, seed: u32) -> f32 {
    let mut x = id
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(seed.wrapping_mul(0x85EB_CA6B));
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    #[expect(clippy::cast_precision_loss, reason = "24 bits, exactly representable")]
    let f = (x >> 8) as f32 / 16_777_216.0;
    f
}

/// The **id** of point `pi` — its `Index` column when it has one, else its
/// position in the list.
///
/// ⚠️ Reading the column rather than the position is what makes the assignment a
/// property of the POINT instead of of its row, which is Blender's `id` default
/// and Houdini's `Piece Attribute` in one sentence: run the points through a
/// `motion.sort` and each keeps the shape it had, instead of the shapes sliding
/// along the reordered list.
fn point_id(points: &Stream, pi: usize) -> i64 {
    match points.get(INDEX) {
        #[expect(clippy::cast_possible_truncation, reason = "an instance index")]
        Some(Column::Scalar(v)) => v.get(pi).copied().unwrap_or(pi as f32).round() as i64,
        _ => pi as i64,
    }
}

/// Which shape point `pi` stamps, given `ns` shapes. `ns >= 1` is the caller's
/// job (with no shapes there is nothing to pick).
///
/// ⚠️ `rem_euclid`, never `%`: the reference wraps in **both** directions, and a
/// bare remainder on a negative id gives a negative index that would panic on the
/// very stream an upstream node could hand us.
fn pick_shape(mode: Pick, points: &Stream, pi: usize, ns: usize, seed: u32) -> usize {
    #[expect(
        clippy::cast_possible_wrap,
        reason = "a shape count, far below i64::MAX"
    )]
    let n = ns as i64;
    match mode {
        Pick::Off => 0,
        Pick::Cycle => point_id(points, pi).rem_euclid(n) as usize,
        Pick::Random => {
            #[expect(clippy::cast_sign_loss, reason = "hashed as a bit pattern")]
            let id = point_id(points, pi) as u32;
            #[expect(clippy::cast_precision_loss, reason = "a shape count")]
            let f = hash01(id, seed) * ns as f32;
            // ⚠️ `hash01` is strictly `< 1` (it divides by `2^24` a value capped at
            // `2^24 - 1`), so `f < ns` and the floor is in range by construction.
            // The `min` is defence in depth and **provably unreachable** — swept,
            // no `ns` from 3 to 1e8 makes `floor(max_hash · ns)` reach `ns` — so it
            // is documented rather than gated: a gate on it could not fail, and a
            // gate that cannot fail is worse than none. What IS gated is the
            // premise it rests on, next door: that `hash01` never returns 1.
            #[expect(clippy::cast_possible_truncation, reason = "floored, in range")]
            let k = f as usize;
            k.min(ns - 1)
        }
    }
}

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

/// Spread a **shape** column over the stamps: stamp `k` takes the shape's value
/// at `pairs[k].0`. This is how the appearance replicates — the same tile / size
/// / tint on every stamp that chose it.
fn spread(col: &Column, pairs: &[(usize, usize)]) -> Column {
    fn go<T: Copy + Default>(v: &[T], pairs: &[(usize, usize)]) -> Vec<T> {
        pairs
            .iter()
            .map(|&(si, _)| v.get(si).copied().unwrap_or_default())
            .collect()
    }
    match col {
        Column::Scalar(v) => Column::Scalar(go(v, pairs)),
        Column::Vec2(v) => Column::Vec2(go(v, pairs)),
        Column::Vec3(v) => Column::Vec3(go(v, pairs)),
        Column::Vec4(v) => Column::Vec4(go(v, pairs)),
    }
}

/// **Which shape meets which point** — the one place the two modes differ, and
/// the reason there is no second column policy to drift.
///
/// `Off` is the product in shape-major order (shape outer, point inner), which is
/// exactly the order the node always emitted; the variant modes are one stamp per
/// POINT, so the output is `np` long instead of `ns·np`. Everything downstream —
/// the `P`/`rot` sums, the `Index`/`Count` renumbering, the appearance spread —
/// reads this list and cannot tell which mode built it.
fn pairs_for(
    mode: Pick,
    shape: &Stream,
    points: &Stream,
    np: usize,
    seed: u32,
) -> Vec<(usize, usize)> {
    let ns = shape.count();
    if mode == Pick::Off {
        let mut v = Vec::with_capacity(ns * np);
        for si in 0..ns {
            for pi in 0..np {
                v.push((si, pi));
            }
        }
        return v;
    }
    (0..np)
        .map(|pi| (pick_shape(mode, points, pi, ns, seed), pi))
        .collect()
}

/// Clamp the point count so the stamped total stays within `max` (and so the
/// multiplication can never overflow the allocation). Never grows the request.
///
/// ⚠️ The divisor is the MODE's, not a constant: `Off` stamps `ns·np` and the
/// budget has to be shared, while a variant mode stamps exactly `np` — capping it
/// by `max / ns` there would throw away points for a product that is never built.
fn points_within_budget(mode: Pick, ns: usize, np: usize, max: usize) -> usize {
    if ns == 0 {
        return 0;
    }
    match mode {
        Pick::Off => np.min(max / ns.max(1)),
        _ => np.min(max),
    }
}

/// Stamp `shape` at every `point`: for each shape × point, take the shape's
/// appearance columns unchanged and add the point's `P`/`rot`. Pure and
/// isolated so the product order, the offset sum, and the `Index`/`Count`
/// renumbering are unit-tested directly. `np` must already be budget-clamped.
fn duplicate(shape: &Stream, points: &Stream, np: usize, mode: Pick, seed: u32) -> Stream {
    // No points: the duplicator has nowhere to stamp → pass the shapes through
    // (the reference's `points`-less behaviour). Cloning is refcount, not copy.
    if np == 0 {
        return shape.clone();
    }
    let pairs = pairs_for(mode, shape, points, np, seed);
    let total = pairs.len();
    let shape_p = shape.get(P);
    let point_p = points.get(P);
    let shape_rot = shape.get(ROT);
    let point_rot = points.get(ROT);
    let has_rot = shape_rot.is_some() || point_rot.is_some();

    let mut out = Stream::new(total);

    // `P` always exists on an Instances stream (the port type guarantees Vec2);
    // sum shape + point in shape-major order.
    let mut pos = Vec::with_capacity(total);
    for &(si, pi) in &pairs {
        let sp = p_at(shape_p, si);
        let pp = p_at(point_p, pi);
        pos.push([sp[0] + pp[0], sp[1] + pp[1]]);
    }
    out.set(P, Column::Vec2(pos));

    // `rot` only if either side authored one (else the lowering's default 0 is
    // the right answer, and an empty column would be noise).
    if has_rot {
        let mut rot = Vec::with_capacity(total);
        for &(si, pi) in &pairs {
            rot.push(rot_at(shape_rot, si) + rot_at(point_rot, pi));
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
        out.set(name.clone(), spread(col, &pairs));
    }
    out
}

struct MotionDuplicator;

impl NodeOp for MotionDuplicator {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = Pick::of(ctx.param("pick"));
        #[expect(clippy::cast_sign_loss, reason = "a seed is a bit pattern")]
        let seed = ctx.param("seed").max(0.0).round() as u32;
        let shape = ctx.input(0);
        let points = ctx.input(1);
        let np = points_within_budget(
            mode,
            shape.count(),
            points.count(),
            RECOMMENDED_MAX_ELEMENTS,
        );
        let out = duplicate(shape, points, np, mode, seed);
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
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    // Both inputs are REQUIRED (ADR-0155): with no `shape` there is nothing to copy, with
    // no `points` there is nowhere to put it — either empty and the node is a silent no-op.
    // A PORT requirement, not a column one, so it is declared (unlike `motion.integrate`,
    // whose `forces` port is optional — a static integration).
    reg.register_required_inputs(MANIFEST.id, &["shape", "points"]);
    Ok(())
}

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "pick",
        label: "Pick",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Off", "Cycle", "Random"],
        },
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
];

/// The seed belongs to the mode that reads it. In Off/Cycle the assignment is
/// fully determined, so a seed row there would be a knob the cook never opens.
static PARAM_GATES: &[ParamGate] = &[ParamGate {
    param: "seed",
    when: "pick",
    values: &[2],
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "pick_tests.rs"]
mod pick_tests;
