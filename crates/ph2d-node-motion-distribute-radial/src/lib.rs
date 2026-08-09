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
fn radial(
    count: usize,
    rings: usize,
    radius: f32,
    inner: f32,
    spin_cycles: f32,
    wedge: Wedge,
) -> Vec<[f32; 2]> {
    let mut out = Vec::with_capacity(count);
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
        }
    }
    out
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
        let positions = radial(count, rings.min(count), radius, inner, spin / 360.0, wedge);
        ctx.emit(Stream::new(positions.len()).with("P", Column::Vec2(positions)));
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
mod tests {
    use super::*;

    fn radius_of(p: [f32; 2]) -> f32 {
        (p[0] * p[0] + p[1] * p[1]).sqrt()
    }

    /// Degrees CCW from +x, folded to `[0, 360)` — the standard-library `atan2`, so the oracle
    /// shares nothing with the parabolic `cos_sin_cycles` it judges.
    fn angle_of(p: [f32; 2]) -> f32 {
        p[1].atan2(p[0]).to_degrees().rem_euclid(360.0)
    }

    /// **The layout as it stood before the wedge existed**, frozen verbatim under `cfg(test)`.
    /// A `pub` copy with no caller would be a second answer waiting for someone to call it; this
    /// one exists only to be disagreed with.
    fn radial_before_the_wedge(
        count: usize,
        rings: usize,
        radius: f32,
        inner: f32,
        spin_cycles: f32,
    ) -> Vec<[f32; 2]> {
        let mut out = Vec::with_capacity(count);
        let per = ring_counts(count, rings);
        for (r, &n_ring) in per.iter().enumerate() {
            let rr = if rings > 1 {
                inner + (radius - inner) * r as f32 / (rings as f32 - 1.0)
            } else {
                radius
            };
            for k in 0..n_ring {
                let cycles = k as f32 / n_ring.max(1) as f32 + spin_cycles;
                let (c, s) = cos_sin_cycles(cycles);
                out.push([rr * c, rr * s]);
            }
        }
        out
    }

    /// **The circle that always shipped is BYTE-identical** — every point, every bit, across a
    /// spread of counts, rings and spins. This is what makes `0 .. 360` a default that costs
    /// nothing rather than a default that is merely close.
    #[test]
    fn the_full_circle_is_byte_identical_to_the_law_that_shipped() {
        for &(count, rings, spin) in &[
            (60usize, 3usize, 0.0f32),
            (8, 1, 0.0),
            (61, 4, 0.137),
            (1, 1, -0.4),
            (255, 7, 1.75),
        ] {
            let before = radial_before_the_wedge(count, rings, 3.0, 0.6, spin);
            let now = radial(count, rings, 3.0, 0.6, spin, Wedge::FULL);
            assert_eq!(before, now, "count {count} rings {rings} spin {spin}");
            let wired = radial(
                count,
                rings,
                3.0,
                0.6,
                spin,
                Wedge::from_degrees(0.0, 360.0),
            );
            assert_eq!(before, wired, "0..360 is FULL: {count}/{rings}");
        }
    }

    /// **The wedge PACKS, it does not CULL** — the distinction the whole param exists for. The
    /// composition it replaces (`field.radial_sweep → motion.cull`) would have returned FEWER
    /// than `count` points; this returns all of them, inside the window.
    #[test]
    fn the_wedge_packs_the_points_it_does_not_cull_them() {
        let pts = radial(8, 1, 2.0, 0.0, 0.0, Wedge::from_degrees(0.0, 180.0));
        assert_eq!(pts.len(), 8, "every point asked for is placed");
        for p in &pts {
            let a = angle_of(*p);
            assert!((-0.2..=180.2).contains(&a), "inside the wedge: {a} deg");
        }
    }

    /// **An open wedge lands on both of its ends** — first on `start`, last on `end`. This is the
    /// half that separates a fan from a ring, and the half an artist checks first.
    #[test]
    fn an_open_wedge_lands_on_both_of_its_ends() {
        let pts = radial(5, 1, 2.0, 0.0, 0.0, Wedge::from_degrees(20.0, 200.0));
        assert!((angle_of(pts[0]) - 20.0).abs() < 0.3, "{:?}", pts[0]);
        assert!((angle_of(pts[4]) - 200.0).abs() < 0.3, "{:?}", pts[4]);
        // And evenly, in between: 45 deg of wedge per step.
        for k in 0..4 {
            let step = angle_of(pts[k + 1]) - angle_of(pts[k]);
            assert!((step - 45.0).abs() < 0.3, "even step {step}");
        }
    }

    /// **A closed wedge does not double up its seam** — the price of the inclusive law, and the
    /// reason the node reads `wraps` off the geometry instead of offering it as a mode. Eight
    /// points over a full turn step by 45 deg and the last does NOT sit on the first.
    #[test]
    fn a_closed_wedge_does_not_double_up_its_seam() {
        let pts = radial(8, 1, 2.0, 0.0, 0.0, Wedge::from_degrees(0.0, 360.0));
        assert!((angle_of(pts[7]) - 315.0).abs() < 0.3, "{:?}", pts[7]);
        let (dx, dy) = (pts[7][0] - pts[0][0], pts[7][1] - pts[0][1]);
        assert!(dx * dx + dy * dy > 1.0, "the seam is not a stack");
    }

    /// `spin` carries the wedge with it: the fan is an EXTENT and the spin is a ROTATION, so
    /// they compose instead of being two doors onto the same number.
    #[test]
    fn the_spin_carries_the_wedge_with_it() {
        let wedge = Wedge::from_degrees(0.0, 90.0);
        let still = radial(4, 1, 2.0, 0.0, 0.0, wedge);
        let spun = radial(4, 1, 2.0, 0.0, 0.25, wedge);
        for (a, b) in still.iter().zip(&spun) {
            let d = (angle_of(*b) - angle_of(*a)).rem_euclid(360.0);
            assert!(
                (d - 90.0).abs() < 0.3,
                "the whole fan turned 90 deg, got {d}"
            );
        }
    }

    /// **A graph that never heard of the wedge draws the circle it always drew.** It does not
    /// name the defaults — it cooks through the registry without touching them.
    #[test]
    fn a_graph_that_never_heard_of_the_wedge_draws_the_circle() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::Graph;

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionDistributeRadial as &dyn NodeOp)
            }
        }
        let mut g = Graph::new();
        let n = g.add_node("motion.distribute_radial");
        g.set_param(n, "count", 24.0);
        g.set_param(n, "rings", 2.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        let Some(Column::Vec2(got)) = out[0].as_stream().get("P") else {
            panic!("P")
        };
        assert_eq!(*got, radial_before_the_wedge(24, 2, 3.0, 0.6, 0.0));
    }

    /// **The authored wedge REACHES the layout.** Every other wedge gate calls `radial` directly,
    /// so all of them stay green with the two params unread — this is the one that walks the seam
    /// from `set_param` to a placed point.
    #[test]
    fn the_authored_wedge_reaches_the_layout() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::Graph;

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionDistributeRadial as &dyn NodeOp)
            }
        }
        let mut g = Graph::new();
        let n = g.add_node("motion.distribute_radial");
        g.set_param(n, "count", 8.0);
        g.set_param(n, "rings", 1.0);
        g.set_param(n, "start_angle", 0.0);
        g.set_param(n, "end_angle", 90.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        let Some(Column::Vec2(got)) = out[0].as_stream().get("P") else {
            panic!("P")
        };
        assert_eq!(got.len(), 8, "the count survives the wedge");
        for p in got {
            let a = angle_of(*p);
            assert!((-0.2..=90.2).contains(&a), "inside the quarter: {a} deg");
        }
    }

    /// A single ring: every point sits on the outer radius, equally spaced. FALSIFIED
    /// if they landed at mixed radii (that would be a spiral, not a ring).
    #[test]
    fn a_single_ring_is_evenly_spaced_on_the_radius() {
        let pts = radial(8, 1, 3.0, 0.6, 0.0, Wedge::FULL);
        assert_eq!(pts.len(), 8);
        for p in &pts {
            // ~1e-2 tolerance: the parabolic cos_sin_cycles is ~0.09% off unit.
            assert!((radius_of(*p) - 3.0).abs() < 1e-2, "on the radius: {p:?}");
        }
        // Equal spacing: consecutive points differ by 1/8 turn — the same chord length.
        let chord = |a: [f32; 2], b: [f32; 2]| {
            let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
            (dx * dx + dy * dy).sqrt()
        };
        let d0 = chord(pts[0], pts[1]);
        for k in 1..8 {
            assert!(
                (chord(pts[k], pts[(k + 1) % 8]) - d0).abs() < 2e-2,
                "equal chords"
            );
        }
    }

    /// `rings` concentric rings: every point lands between `inner` and `radius`, and
    /// more than one distinct radius appears.
    #[test]
    fn rings_are_concentric_between_inner_and_radius() {
        let pts = radial(60, 3, 3.0, 1.0, 0.0, Wedge::FULL);
        assert_eq!(pts.len(), 60);
        let mut radii: Vec<f32> = pts.iter().map(|p| radius_of(*p)).collect();
        for r in &radii {
            assert!(
                *r >= 1.0 - 1e-2 && *r <= 3.0 + 1e-2,
                "within [inner, radius]: {r}"
            );
        }
        radii.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!(radii[59] - radii[0] > 1.0, "spans multiple rings");
    }

    /// The exact count is honoured even when it doesn't divide evenly across rings.
    #[test]
    fn count_is_exact_across_uneven_rings() {
        assert_eq!(radial(61, 4, 3.0, 0.5, 0.0, Wedge::FULL).len(), 61);
        assert_eq!(ring_counts(61, 4), vec![16, 15, 15, 15]);
    }

    /// `spin` rotates the whole array: a quarter-turn spin moves a point that was on
    /// +x onto +y. FALSIFIED by a dead spin (the point stays on +x).
    #[test]
    fn spin_rotates_the_array() {
        let base = radial(4, 1, 2.0, 0.0, 0.0, Wedge::FULL); // points at 0°, 90°, 180°, 270°
        let spun = radial(4, 1, 2.0, 0.0, 0.25, Wedge::FULL); // +90°
        assert!(
            base[0][0] > 1.9 && base[0][1].abs() < 1e-3,
            "base point on +x"
        );
        assert!(
            spun[0][1] > 1.9 && spun[0][0].abs() < 1e-3,
            "spun point on +y: {:?}",
            spun[0]
        );
    }

    /// Deterministic + cooks through the registry, emitting `P` at the exact count.
    #[test]
    fn registers_and_cooks() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::Graph;

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionDistributeRadial as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let n = g.add_node("motion.distribute_radial");
        g.set_param(n, "count", 24.0);
        g.set_param(n, "rings", 2.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 24),
            _ => panic!("P"),
        }
    }
}
