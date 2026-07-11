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

use ph2d_node_registry::{NodeRegistry, RegistryError};
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
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The per-ring point counts for `count` points over `rings` rings — split as evenly
/// as possible (the first `count % rings` rings get one extra).
fn ring_counts(count: usize, rings: usize) -> Vec<usize> {
    let base = count / rings;
    let rem = count % rings;
    (0..rings).map(|r| base + usize::from(r < rem)).collect()
}

/// Lay out the radial array: `count` points across `rings` rings (radii `inner`..
/// `radius`), each ring equally spaced in angle, offset by `spin` cycles.
fn radial(count: usize, rings: usize, radius: f32, inner: f32, spin_cycles: f32) -> Vec<[f32; 2]> {
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
            let cycles = k as f32 / n_ring.max(1) as f32 + spin_cycles;
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
        let positions = radial(count, rings.min(count), radius, inner, spin / 360.0);
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
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 1.0,
        max: 2000.0,
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
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn radius_of(p: [f32; 2]) -> f32 {
        (p[0] * p[0] + p[1] * p[1]).sqrt()
    }

    /// A single ring: every point sits on the outer radius, equally spaced. FALSIFIED
    /// if they landed at mixed radii (that would be a spiral, not a ring).
    #[test]
    fn a_single_ring_is_evenly_spaced_on_the_radius() {
        let pts = radial(8, 1, 3.0, 0.6, 0.0);
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
        let pts = radial(60, 3, 3.0, 1.0, 0.0);
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
        assert_eq!(radial(61, 4, 3.0, 0.5, 0.0).len(), 61);
        assert_eq!(ring_counts(61, 4), vec![16, 15, 15, 15]);
    }

    /// `spin` rotates the whole array: a quarter-turn spin moves a point that was on
    /// +x onto +y. FALSIFIED by a dead spin (the point stays on +x).
    #[test]
    fn spin_rotates_the_array() {
        let base = radial(4, 1, 2.0, 0.0, 0.0); // points at 0°, 90°, 180°, 270°
        let spun = radial(4, 1, 2.0, 0.0, 0.25); // +90°
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
