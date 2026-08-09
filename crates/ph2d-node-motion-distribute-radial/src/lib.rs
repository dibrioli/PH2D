#![forbid(unsafe_code)]
//! `motion.distribute_radial` — a **radial array**: `count` points spread evenly
//! around `rings` concentric circles, a sunburst / clock-face / radial cloner
//! (Motion Nodes M3, distributions — doc 01 §3 / doc 25). The polar counterpart to
//! the rectangular `motion.grid` and the organic `motion.fibonacci`: rings and spokes.
//!
//! **Algorithm — the regular polar array.** The `count` points are split as evenly as
//! possible across `rings` rings (radii from `inner` to `radius`); within each ring
//! the points are equally spaced in angle, offset by a global `spin`. A `spin` **value**
//! input (degrees) rotates the whole array, so a `value.lfo` swings it round.
//!
//! A **Source** node (no stream input, mints `P`). Transcendental-free (HR-5): the
//! angles use `cos_sin_cycles` — the parabolic sine copied from `motion.orbit` — so no
//! `sin`/`cos` calls. `Effect::Pure` (no clock — the spin animation arrives through the
//! value input).
//!
//! ## `start_angle` / `end_angle` — the WEDGE
//!
//! A radial array that can only be a whole circle cannot draw a fan, a speedometer dial or a
//! pie slice. The reference has had the pair since forever (C4D *Radial mode:* `Count · Radius ·
//! Plane · Align · **Start/End Angle** · Offset`), and our own sibling field — `field.radial_sweep`
//! — already exposes exactly this window.
//!
//! ⚠️ **The wedge PACKS the points; it does not CULL them.** The composition that existed before
//! this param (`distribute_radial → field.radial_sweep → motion.cull`) *removes* whatever falls
//! outside the window: the count drops and holes appear where the wedge cuts. That is the opposite
//! of what the reference does and of what the artist asked for — they asked for *these N things,
//! arranged over that wedge*. So `count` is honoured whole and the spacing follows the wedge.
//!
//! ⚠️ **A closed wedge and an open one are different questions, and the node asks the GEOMETRY
//! rather than offering a mode.** A ring has no ends, so its `n` points step by `1/n` and the last
//! does not sit on the first. A fan HAS ends, so its `n` points step by `1/(n-1)` and the first and
//! last sit exactly on `start` and `end` — otherwise the artist types `end = 180`, watches the last
//! clone land at 144°, and concludes the param is broken. The two laws meet where the two ends land
//! on the same angle, which is a fact the node can read off the numbers.
//!
//! `spin` still rotates everything, wedge and all: it is a *rotation*, the wedge is an *extent*,
//! and they compose. **Default `0 .. 360`** is the closed circle that always shipped, byte for byte.
//!
//! ## `align` — the clone turns to face outward
//!
//! Placing a point is half of a radial array; the other half is **which way each clone looks**,
//! and C4D's Radial Cloner has shipped the pair forever (`Count · Radius · Plane · **Align** ·
//! Start/End Angle · Offset`). Nothing downstream could supply it: `motion.look_at` aims at a
//! *target point*, so pointing every clone away from the centre would need a target *per clone*.
//!
//! ⚠️ **The heading is not derived — it IS the layout parameter.** The sibling that walks a curve
//! (`motion.distribute_curve`, and `motion.path` before it) has to differentiate the geometry and
//! then read the angle back with an `atan2` approximation. Here the angle is the number the layout
//! already computed to place the point: `cycles`. Asking `atan2(y, x)` for it would re-derive a
//! known quantity through an approximation — spending error to learn what we just typed — and
//! would break at the one place it matters most (see the centre, below).
//!
//! ⚠️ **The angle is NOT wrapped, and that is deliberate.** An `atan2` hands back `[-180, 180]`
//! because a direction has no history; this node knows the winding, because `spin` is an animatable
//! input and the wedge may sweep past a full turn. Emitting the wrapped value would throw away
//! information only this node has, and would plant a ±180 seam exactly where a downstream filter
//! blows up (the `motion.delay` lesson: *an angle is a circle, so the line keeps the UNWRAPPED
//! value*). The renderer builds a basis with `sin`/`cos`, which are periodic, so magnitude is free.
//!
//! ⚠️ **A clone at the centre still has a heading.** With `inner = 0` the innermost ring sits at
//! radius zero and "outward" is the direction of a zero-length vector — undefined. The layout angle
//! is not: it is what spaced that ring, so that is what the clone reports. An `atan2(0, 0)` would
//! have answered `0` for every one of them, silently collapsing a ring's worth of orientation.
//!
//! **Default off** (`align = 0`): a graph that never heard of this param emits `P` and nothing
//! else, byte for byte, exactly as it did before.

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod trig;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `spin` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Max rings (a bound on the layout loop).
const MAX_RINGS: i64 = 256;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.distribute_radial"),
    name: "motion.distribute_radial",
    inputs: &[
        // Global rotation of the whole array, in degrees (animatable). Optional:
        // unconnected reads as 0.
        PortSpec {
            name: "spin",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "count",
            default: 60.0,
        },
        ParamSpec {
            name: "rings",
            default: 3.0,
        },
        // Outer ring radius (world units).
        ParamSpec {
            name: "radius",
            default: 3.0,
        },
        // Inner ring radius (0 = a point at the centre ring).
        ParamSpec {
            name: "inner",
            default: 0.6,
        },
        // The WEDGE, in degrees. APPENDED, and `0 .. 360` is the closed circle that always
        // shipped: a saved graph reads them as absent and lays out byte-identically.
        ParamSpec {
            name: "start_angle",
            default: 0.0,
        },
        ParamSpec {
            name: "end_angle",
            default: 360.0,
        },
        // APPENDED, and `0` is off: a saved graph reads it as absent, takes the default and
        // emits `P` alone — the stream it always emitted, byte for byte.
        ParamSpec {
            name: "align",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The angular window a ring's points are packed into, in **turns**.
///
/// `wraps` is not a mode the artist picks — it is read off the numbers: the two ends land on the
/// same angle exactly when the sweep is a whole number of turns, and that is what decides whether
/// the ring has ends to sit on.
#[derive(Copy, Clone)]
struct Wedge {
    start: f32,
    sweep: f32,
    wraps: bool,
}

impl Wedge {
    /// The closed circle — the only wedge that existed before the params did.
    ///
    /// ⚠️ **`cfg(test)`, and that is the honest place for it:** the product never builds this,
    /// it builds [`Self::from_degrees`] from two params. It exists to be the NEUTRAL that
    /// `from_degrees(0, 360)` is asserted to reduce to — a `const` reachable from the product
    /// with no caller would be a second answer waiting for one.
    #[cfg(test)]
    const FULL: Self = Self {
        start: 0.0,
        sweep: 1.0,
        wraps: true,
    };

    fn from_degrees(start_deg: f32, end_deg: f32) -> Self {
        let sweep = (end_deg - start_deg) / 360.0;
        Self {
            start: start_deg / 360.0,
            sweep,
            // ⚠️ `fract()`, never a tolerance: 359.9° is an OPEN fan and 360° is a ring, and a
            // window that called them the same would make the ring's seam double up.
            wraps: sweep.abs().fract() == 0.0,
        }
    }

    /// Where point `k` of `n` sits inside the wedge, as a fraction of the sweep.
    fn frac(self, k: usize, n: usize) -> f32 {
        if n <= 1 {
            return 0.0;
        }
        if self.wraps {
            k as f32 / n as f32
        } else {
            k as f32 / (n - 1) as f32
        }
    }
}

/// The per-ring point counts for `count` points over `rings` rings — split as evenly
/// as possible (the first `count % rings` rings get one extra).
fn ring_counts(count: usize, rings: usize) -> Vec<usize> {
    let base = count / rings;
    let rem = count % rings;
    (0..rings).map(|r| base + usize::from(r < rem)).collect()
}

/// Lay out the radial array: `count` points across `rings` rings (radii `inner`..
/// `radius`), each ring equally spaced **inside `wedge`**, offset by `spin` cycles.
///
/// With the neutral wedge (`0 .. 360`) the angle expression reduces to the one that always shipped
/// (`0 + frac·1 + spin`), term for term — which is why the default is byte-identical rather
/// than merely close.
///
/// With `align`, each point also reports the **outward heading there**, in degrees. It is the very
/// `cycles` that placed the point, scaled to degrees — not an `atan2` of the position, which would
/// re-derive through an approximation what this loop already knows exactly (module docs).
fn radial(
    count: usize,
    rings: usize,
    radius: f32,
    inner: f32,
    spin_cycles: f32,
    wedge: Wedge,
    align: bool,
) -> (Vec<[f32; 2]>, Vec<f32>) {
    let mut out = Vec::with_capacity(count);
    let mut rot = Vec::with_capacity(if align { count } else { 0 });
    let per = ring_counts(count, rings);
    for (r, &n_ring) in per.iter().enumerate() {
        // Ring radius: `inner` at r=0 up to `radius` at the last ring (a lone ring
        // sits at `radius`).
        let rr = if rings > 1 {
            inner + (radius - inner) * r as f32 / (rings as f32 - 1.0)
        } else {
            radius
        };
        for k in 0..n_ring {
            let frac = wedge.frac(k, n_ring.max(1));
            let cycles = wedge.start + frac * wedge.sweep + spin_cycles;
            let (c, s) = cos_sin_cycles(cycles);
            out.push([rr * c, rr * s]);
            if align {
                rot.push(cycles * 360.0);
            }
        }
    }
    (out, rot)
}

struct MotionDistributeRadial;

impl NodeOp for MotionDistributeRadial {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS).max(1);
        let rings = (ctx.param("rings").round() as i64).clamp(1, MAX_RINGS) as usize;
        let radius = ctx.param("radius");
        let inner = ctx.param("inner");
        let spin = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.0),
            _ => 0.0,
        };
        let wedge = Wedge::from_degrees(ctx.param("start_angle"), ctx.param("end_angle"));
        let align = ctx.param("align") >= 0.5;
        let (positions, rot) = radial(
            count,
            rings.min(count),
            radius,
            inner,
            spin / 360.0,
            wedge,
            align,
        );
        let mut out = Stream::new(positions.len()).with("P", Column::Vec2(positions));
        if align {
            out = out.with("rot", Column::Scalar(rot));
        }
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionDistributeRadial))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Radial Array",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};
/// **O teto DURO de `count` — MEDIDO** (doc 88 A1 · §0); o slider fica nos **600** onde a mão
/// trabalha (default 60 ⇒ ~4 instâncias por pixel de arrasto), e o teto é **1.667×** ele.
/// A distribuição é um laço linear, e o cook mediu pela porta do produto (`rings = 1`, para o eixo
/// ser o que a linha nomeia):
///
/// | instâncias | cook |
/// |---|---|
/// | 100.000 | 0,461 ms |
/// | 400.000 | 1,858 ms |
/// | **1.000.000** | **4,584 ms** |
///
/// ⚠️ Freio ERGONÔMICO por eixo: as instâncias são `count × rings`, e um cap sobre um fator não
/// exprime um limite sobre o produto (o precedente do `rate` do emitter).
pub(crate) static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "count",
        max: 1_000_000.0,
    },
    ParamHardMax {
        param: "inner",
        max: 20.0,
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 1.0,
        max: 600.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "rings",
        label: "Rings",
        min: 1.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.1,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "inner",
        label: "Inner",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // The wedge. `-360..720` on both so a fan can start behind zero and a sweep can go the long
    // way round or backwards — `end < start` is a legal wedge that runs clockwise.
    ParamUiHint {
        param: "start_angle",
        label: "Start Angle",
        min: -360.0,
        max: 720.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "end_angle",
        label: "End Angle",
        min: -360.0,
        max: 720.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    // The label says what the clone DOES, not which column it writes: "Align" alone is the
    // reference's word but leaves *to what?* open, and this array has two plausible answers
    // (the radius, or the ring's tangent). It picks the radius, so it says so.
    ParamUiHint {
        param: "align",
        label: "Align To Radius",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "radius",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "inner",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "start_angle",
        unit: ParamUnit::Angle,
    },
    ParamUnitDecl {
        param: "end_angle",
        unit: ParamUnit::Angle,
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
