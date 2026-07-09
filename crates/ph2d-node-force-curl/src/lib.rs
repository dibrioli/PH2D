#![forbid(unsafe_code)]
//! `force.curl` — **curl noise**: a divergence-free turbulent flow field
//! (Bridson, Houriham & Nordenstam, *Curl-Noise for Procedural Fluid Flow*,
//! SIGGRAPH 2007). A Motion **force**: a node that adds its per-instance
//! contribution into the transient `accel` column (× the multiplicative
//! `falloff` field, plan §1.6); `motion.integrate` turns it into motion.
//!
//! ## Why the curl, and not just "noise pushed at the particle"
//!
//! Sampling a noise vector directly gives a field with sources and sinks:
//! particles pile up in the sinks and the cloud clumps and dies. The curl of a
//! scalar potential is **divergence-free by construction** — in 2D,
//! `v = (∂ψ/∂y, −∂ψ/∂x)` satisfies `∇·v = ψ_yx − ψ_xy = 0` — so the flow
//! swirls forever and never compresses. That is the whole idea of the paper.
//!
//! `ψ` is fBm value noise ([`noise::fbm`]), evaluated with central differences.
//! Everything is integer-hash + polynomial: **transcendental-free and
//! deterministic** (HR-5), so the same particle at the same playhead always
//! feels the same eddy — CPU today, GPU lane tomorrow.
//!
//! Params: `strength` (accel scale), `scale` (spatial frequency of the eddies),
//! `speed` (how fast the field itself drifts, in playhead-seconds), `octaves`
//! (1..=4 scales of swirl), `seed`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod accum;
mod noise;
use accum::{add_accel, falloff_at, vec2_at};
use noise::fbm;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Central-difference step, in noise space. Small enough to approximate the
/// derivative, large enough that `ψ(x+ε) − ψ(x−ε)` keeps its significant digits
/// in `f32` (the noise lattice has unit period, so `ψ` changes by ~`ε` here — a
/// far smaller step would subtract two nearly equal numbers and keep only noise).
const EPS: f32 = 0.01;
/// Ceiling on `octaves` — the field is sampled `4 × octaves` times per instance
/// per tick, and past four the extra octaves are below a pixel of motion.
const MAX_OCTAVES: u32 = 4;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.curl"),
    name: "force.curl",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // The field drifts with the playhead (`speed`).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "strength",
            default: 6.0,
        },
        ParamSpec {
            name: "scale",
            default: 0.35,
        },
        ParamSpec {
            name: "speed",
            default: 0.2,
        },
        ParamSpec {
            name: "octaves",
            default: 2.0,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The scalar potential `ψ` at a world point, for a given field configuration.
/// Time drifts the field along its own x-axis (the noise is 2D; a third lattice
/// axis is the follow-up when the noise grows one).
fn psi(x: f32, y: f32, drift: f32, seed: f32, octaves: u32) -> f32 {
    fbm(x + drift + seed, y, octaves)
}

/// `curl ψ` at a world point: `(∂ψ/∂y, −∂ψ/∂x)` by central differences.
/// Divergence-free (see the module docs).
fn curl(x: f32, y: f32, drift: f32, seed: f32, octaves: u32) -> [f32; 2] {
    let dpsi_dx = psi(x + EPS, y, drift, seed, octaves) - psi(x - EPS, y, drift, seed, octaves);
    let dpsi_dy = psi(x, y + EPS, drift, seed, octaves) - psi(x, y - EPS, drift, seed, octaves);
    let inv = 1.0 / (2.0 * EPS);
    [dpsi_dy * inv, -dpsi_dx * inv]
}

struct ForceCurl;

impl NodeOp for ForceCurl {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let strength = ctx.param("strength");
        let scale = ctx.param("scale");
        let drift = ctx.playhead() as f32 * ctx.param("speed");
        let octaves = (ctx.param("octaves").round().max(1.0) as u32).min(MAX_OCTAVES);
        let seed = ctx.param("seed");
        let out = {
            let input = ctx.input(0);
            let contrib: Vec<[f32; 2]> = (0..input.count())
                .map(|i| {
                    let p = vec2_at(input, "P", i, [0.0, 0.0]);
                    let v = curl(p[0] * scale, p[1] * scale, drift, seed, octaves);
                    let w = strength * falloff_at(input, i);
                    [v[0] * w, v[1] * w]
                })
                .collect();
            add_accel(input, &contrib)
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ForceCurl))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Curl Noise",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "scale",
        label: "Scale",
        min: 0.01,
        max: 3.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "speed",
        label: "Speed",
        min: 0.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "octaves",
        label: "Octaves",
        min: 1.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 100.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("force.curl.test.src"),
        name: "force.curl.test.src",
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
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(3).with(
                "P",
                Column::Vec2(vec![[0.7, 1.3], [-2.1, 0.4], [3.3, -1.8]]),
            ));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&ForceCurl),
                _ => None,
            }
        }
    }

    fn accel_at(playhead: f64) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("force.curl.test.src");
        let c = g.add_node("force.curl");
        g.connect(Edge {
            from: (src, 0),
            to: (c, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, c, playhead).unwrap();
        match out[0].as_stream().get("accel").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("accel"),
        }
    }

    /// Divergence `∇·v` of a field, sampled with the SAME central-difference
    /// step the field itself uses. Matching the stencil matters: the discrete
    /// curl's mixed differences cancel exactly at step `EPS`, so any residue is
    /// float rounding, not a modelling error. Measuring with a different `h`
    /// would report the noise's own curvature, not the field's divergence.
    fn divergence(field: impl Fn(f32, f32) -> [f32; 2], x: f32, y: f32) -> f32 {
        let inv = 1.0 / (2.0 * EPS);
        (field(x + EPS, y)[0] - field(x - EPS, y)[0]) * inv
            + (field(x, y + EPS)[1] - field(x, y - EPS)[1]) * inv
    }

    /// The raw gradient `(∂ψ/∂x, ∂ψ/∂y)` — the naive "push the particle along
    /// the noise" field the curl exists to replace.
    fn gradient(x: f32, y: f32) -> [f32; 2] {
        let inv = 1.0 / (2.0 * EPS);
        [
            (psi(x + EPS, y, 0.0, 0.0, 2) - psi(x - EPS, y, 0.0, 0.0, 2)) * inv,
            (psi(x, y + EPS, 0.0, 0.0, 2) - psi(x, y - EPS, 0.0, 0.0, 2)) * inv,
        ]
    }

    /// The property the whole node exists for (Bridson 2007): the field has
    /// **zero divergence**, so particles swirl forever and never pile into a
    /// sink. The same measurement on the raw gradient — the field you get by
    /// sampling noise directly — shows divergence orders of magnitude larger,
    /// which is exactly why curl noise is the published answer.
    #[test]
    fn the_curl_is_divergence_free_and_the_raw_gradient_is_not() {
        let (mut worst_curl, mut worst_grad) = (0.0f32, 0.0f32);
        for k in 0..64 {
            let (x, y) = (k as f32 * 0.37 - 7.0, k as f32 * 0.21 - 4.0);
            // Scale-relative, so a quiet patch of the field cannot flatter us.
            let mag = curl(x, y, 0.0, 0.0, 2)
                .iter()
                .map(|c| c.abs())
                .sum::<f32>()
                .max(1e-3);
            let d_curl = divergence(|a, b| curl(a, b, 0.0, 0.0, 2), x, y).abs() / mag;
            let d_grad = divergence(gradient, x, y).abs() / mag;
            worst_curl = worst_curl.max(d_curl);
            worst_grad = worst_grad.max(d_grad);
        }
        assert!(
            worst_curl < 1e-2,
            "curl divergence must vanish, worst = {worst_curl}"
        );
        assert!(
            worst_grad > 1.0,
            "the raw gradient DOES diverge (that is the point), worst = {worst_grad}"
        );
    }

    #[test]
    fn instances_feel_different_eddies_and_the_field_drifts() {
        let a = accel_at(0.0);
        assert!(
            a[0] != a[1] || a[1] != a[2],
            "distinct positions sample distinct swirl"
        );
        let b = accel_at(2.0);
        assert!(
            (a[0][0] - b[0][0]).abs() > 1e-6,
            "the field drifts with the playhead"
        );
    }

    #[test]
    fn is_deterministic_for_replay() {
        assert_eq!(accel_at(0.8), accel_at(0.8));
    }

    #[test]
    fn falloff_gates_the_force() {
        // A stream with falloff 0 on the middle instance: it feels nothing.
        static MASK_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("force.curl.test.mask"),
            name: "force.curl.test.mask",
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
        struct Mask;
        impl NodeOp for Mask {
            fn manifest(&self) -> &'static NodeManifest {
                &MASK_MAN
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[0.7, 1.3], [0.7, 1.3]]))
                        .with("falloff", Column::Scalar(vec![1.0, 0.0])),
                );
            }
        }
        struct MaskOps;
        impl OpResolver for MaskOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == MASK_MAN.id => Some(&Mask),
                    t if t == MANIFEST.id => Some(&ForceCurl),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("force.curl.test.mask");
        let c = g.add_node("force.curl");
        g.connect(Edge {
            from: (src, 0),
            to: (c, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &MaskOps, c, 0.0).unwrap();
        match out[0].as_stream().get("accel").unwrap() {
            Column::Vec2(v) => {
                assert!(v[0] != [0.0, 0.0], "unmasked instance swirls");
                assert_eq!(v[1], [0.0, 0.0], "falloff 0 → no force");
            }
            _ => panic!("accel"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
