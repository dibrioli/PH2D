#![forbid(unsafe_code)]
//! `motion.pin_constraint` — **nail elements down**: mark part of a stream as
//! immovable so the simulation flows around it while the pinned elements stay
//! driven by the upstream animation (Motion Nodes M3, simulation — doc 01 §3 /
//! doc 34). The hanging cloth's top corners, the anchor a rope swings from, the
//! obstacles a crowd has to avoid.
//!
//! **The primitive is inverse mass, not a boolean.** The gold standard is
//! Position Based Dynamics (Müller et al., 2007): every particle carries
//! `w = 1/m`, and a constraint's correction is distributed in proportion to the
//! `w`s of the particles it touches. `w = 0` is infinite mass — no force, no
//! contact and no constraint can move it. It is the same primitive as Houdini
//! Vellum's `pintoanimation`, Blender's cloth **pin group** (whose vertex weight
//! is likewise a *partial* pin) and Bullet's `invMass`; a bool would give the
//! hard pin only, and would not let a heavier-than-its-neighbours element merely
//! resist. So this node writes a per-element [`INV_MASS`] column and the
//! solvers read it.
//!
//! ```text
//! grid ── pin_constraint ──> integrate ──> output      (the pinned run holds still;
//!           (one row's worth)    ^                      the free ones fall/blow away)
//!                                └── force.wind
//! ```
//!
//! **Mind which row you are naming.** The index range is exact but it is not geometry:
//! `motion.grid` is row-major from the LOWEST y up, so its *first* `cols` elements are
//! the row at the BOTTOM of the screen and a curtain hangs from the LAST ones. To select
//! by shape instead of by index — the safer habit — put a `motion.falloff` upstream and
//! pin the region it covers.
//!
//! **Who reads it:** `motion.integrate` (the force chain's one integrator),
//! `motion.spring` and `motion.collide` — every node that takes an instance
//! stream and moves it. A missing column reads as `1.0` (free), so every graph
//! authored before this node behaves exactly as it did.
//! `motion.verlet_rope` / `motion.soft_body` / `motion.boids` are *generators*
//! (they mint their own points from params and carry state; no instance stream
//! flows in), so their intrinsic pins — the rope's head/tail, the body's top row
//! — stay where they are: an upstream pin has no wire to reach them through.
//!
//! **Selection** is the index range `[first, first + count)` **times** the
//! multiplicative `falloff` field the module's falloff nodes write (so a
//! `motion.falloff` upstream pins a *region* — the classic "pin what the circle
//! covers"), times `strength` (a partial pin: a heavy, sluggish element rather
//! than an immovable one). `count = 0` selects nothing and the node is the
//! identity.
//!
//! Pins **compose**: the node multiplies into whatever `inv_mass` is already on
//! the stream, exactly like the falloffs multiply into `falloff`, so two pin
//! nodes stack instead of the second erasing the first.
//!
//! `Effect::Pure`, no clock, no state — the weights are a pure function of the
//! params and the incoming field. HR-5: arithmetic only.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The **inverse-mass** column — PBD's `w = 1/m`, per element. `1` = free (the
/// default when the column is absent), `0` = pinned (infinite mass: no force,
/// contact or constraint may move it), in between = heavy. The solvers scale
/// their correction by it.
///
/// The name is the one piece of this node other crates must agree on, so it
/// lives here as a `pub const` and the readers refer to it rather than
/// re-spelling the string.
pub const INV_MASS: &str = "inv_mass";

/// The module's multiplicative selection field (written by the `motion.falloff`
/// family). Absent reads as `1` — the whole stream is inside the field.
const FALLOFF: &str = "falloff";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.pin_constraint"),
    name: "motion.pin_constraint",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // The first element of the pinned run. Rounded and clamped at eval.
        ParamSpec {
            name: "first",
            default: 0.0,
        },
        // How many consecutive elements from `first` are pinned. A dropped node
        // must SHOW something, so it lands pinning the first element (the rope
        // anchor). `0` selects nothing and the node is the identity.
        ParamSpec {
            name: "count",
            default: 1.0,
        },
        // How hard: 1 = immovable (w = 0), 0.5 = twice as heavy as its
        // neighbours, 0 = free. Blender's partial pin weight.
        ParamSpec {
            name: "strength",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// A param as a non-negative element index/count: non-finite reads as 0.
fn as_index(v: f32) -> usize {
    if v.is_finite() && v >= 0.0 {
        v.round() as usize
    } else {
        0
    }
}

/// `x` clamped to `[0, 1]`; a non-finite value (a NaN param from a hand-edited
/// document) reads as 0 — no pin — rather than poisoning the weights.
fn clamp01(x: f32) -> f32 {
    if x.is_finite() {
        x.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// The scalar column `name`, widened to `n` elements, `fallback` where absent.
fn scalar_or(s: &Stream, name: &str, n: usize, fallback: f32) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) if v.len() == n => v.clone(),
        _ => vec![fallback; n],
    }
}

/// The whole node: multiply the pinned run's inverse mass down toward zero.
/// Every other column (P included — the node moves nothing) rides through.
fn pin(input: &Stream, first: usize, count: usize, strength: f32) -> Stream {
    let n = input.count();
    let falloff = scalar_or(input, FALLOFF, n, 1.0);
    let prev = scalar_or(input, INV_MASS, n, 1.0);
    // `first + count` cannot wrap: both are element counts (saturating on the
    // absurd param that a loaded document may carry).
    let last = first.saturating_add(count);
    let w: Vec<f32> = (0..n)
        .map(|i| {
            let selected = i >= first && i < last;
            // The pin AMOUNT (1 = nailed): the range mask times the field times
            // the strength. Its complement is the inverse mass, multiplied into
            // whatever an upstream pin already wrote (pins compose).
            let amount = if selected {
                clamp01(strength * falloff[i])
            } else {
                0.0
            };
            prev[i] * (1.0 - amount)
        })
        .collect();

    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != INV_MASS {
            out.set(name.clone(), col.clone());
        }
    }
    out.set(INV_MASS, Column::Scalar(w));
    out
}

struct MotionPinConstraint;

impl NodeOp for MotionPinConstraint {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let first = as_index(ctx.param("first"));
        let count = as_index(ctx.param("count"));
        let strength = ctx.param("strength");
        let out = pin(ctx.input(0), first, count, strength);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionPinConstraint))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Pin Constraint",
            // It authors a weight FIELD the sims read — the falloff family's
            // category, not a transform (it moves nothing itself).
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "first",
        label: "First",
        min: 0.0,
        max: 4096.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 0.0,
        max: 4096.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// A stream of `n` elements at the origin, with the given optional fields.
    fn stream(n: usize, falloff: Option<Vec<f32>>, inv_mass: Option<Vec<f32>>) -> Stream {
        let mut s = Stream::new(n).with("P", Column::Vec2(vec![[0.0, 0.0]; n]));
        if let Some(f) = falloff {
            s.set(FALLOFF, Column::Scalar(f));
        }
        if let Some(w) = inv_mass {
            s.set(INV_MASS, Column::Scalar(w));
        }
        s
    }

    fn weights(s: &Stream) -> Vec<f32> {
        match s.get(INV_MASS) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("no inv_mass column"),
        }
    }

    /// The range is what gets pinned: inside it the inverse mass is 0 (infinite
    /// mass), outside it stays 1 (free). FALSIFIED if the node pinned the whole
    /// stream (the bug that would freeze every sim downstream).
    #[test]
    fn the_index_range_is_what_gets_pinned() {
        let out = pin(&stream(5, None, None), 1, 2, 1.0);
        assert_eq!(weights(&out), vec![1.0, 0.0, 0.0, 1.0, 1.0]);
    }

    /// `strength` is a PARTIAL pin (Blender's pin weight): half strength leaves
    /// half the inverse mass, i.e. an element twice as heavy as its neighbours.
    #[test]
    fn strength_is_a_partial_pin() {
        let out = pin(&stream(2, None, None), 0, 1, 0.25);
        assert_eq!(weights(&out), vec![0.75, 1.0]);
    }

    /// The `falloff` field scales the pin, so an upstream falloff pins a REGION:
    /// full field = nailed, half = heavy, zero = untouched.
    #[test]
    fn the_falloff_field_scales_the_pin() {
        let out = pin(&stream(3, Some(vec![1.0, 0.5, 0.0]), None), 0, 3, 1.0);
        assert_eq!(weights(&out), vec![0.0, 0.5, 1.0]);
    }

    /// Two pins COMPOSE (multiply) instead of the second erasing the first —
    /// the falloff family's rule. Two half-pins on the same element leave a
    /// quarter of the inverse mass.
    #[test]
    fn pins_compose_multiplicatively() {
        let once = pin(&stream(1, None, None), 0, 1, 0.5);
        let twice = pin(&once, 0, 1, 0.5);
        assert_eq!(weights(&twice), vec![0.25]);
    }

    /// `count = 0` (or a zero strength) selects nothing: every element stays
    /// free, and an upstream weight rides through untouched.
    #[test]
    fn an_empty_selection_is_the_identity() {
        assert_eq!(
            weights(&pin(&stream(3, None, None), 0, 0, 1.0)),
            vec![1.0; 3]
        );
        assert_eq!(
            weights(&pin(&stream(3, None, None), 0, 3, 0.0)),
            vec![1.0; 3]
        );
        let carried = stream(2, None, Some(vec![0.0, 0.5]));
        assert_eq!(weights(&pin(&carried, 0, 0, 1.0)), vec![0.0, 0.5]);
    }

    /// A non-finite param never poisons the weights (a hand-edited document can
    /// carry any `f32`): the element stays free rather than going NaN.
    #[test]
    fn a_non_finite_strength_reads_as_free() {
        let out = pin(&stream(1, None, None), 0, 1, f32::NAN);
        assert_eq!(weights(&out), vec![1.0]);
    }

    /// Cooks through the registry: the weights land on the stream and every
    /// other column (the positions the node must NOT touch) passes through.
    #[test]
    fn registers_and_cooks_the_weight_column() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.pin.test.src"),
            name: "motion.pin.test.src",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(3)
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionPinConstraint),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.pin.test.src");
        let p = g.add_node("motion.pin_constraint");
        g.set_param(p, "count", 2.0);
        g.connect(Edge {
            from: (src, 0),
            to: (p, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, p, 0.0).unwrap();
        let s = out[0].as_stream();
        assert_eq!(s.count(), 3, "count preserved");
        assert_eq!(weights(s), vec![0.0, 0.0, 1.0], "the first two are pinned");
        match s.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v[1], [1.0, 0.0], "positions ride through"),
            _ => panic!("P"),
        }
    }
}
