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
use ph2d_nodegraph::attr::par_build;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
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

/// GPU compute kernel (GPU/M5 **Fase 3**, ADR-0126 side channel): the exact
/// per-element map of the CPU `eval` — the divergence-free curl of an fBm
/// potential (Bridson 2007), drifting with the playhead.
///
/// **HR-5:** the potential is this node's OWN integer-hash value noise (a port
/// of `noise.rs`, the same mix `motion.wiggle` proved bit-exact on the RTX) —
/// `bitcast<u32>` is Rust's `as u32`, `u32(x)` would be a value cast and diverge
/// on negatives. The divergence-free property is a *cancellation* of mixed
/// differences at exactly step `EPS`, so the stencil must be the CPU's to the
/// letter: a different `h` would measure the noise's curvature instead.
///
/// The octave count is a rounded param (half-AWAY, like Rust) and bounds a real
/// loop — 4 octaves × 4 psi samples × 4 lattice hashes is the heaviest kernel of
/// the five, and still a pure per-element map.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let cl_p = read_P(i);\n\
        let cl_oct = i32(min(max(cl_round(params.octaves), 1.0), CL_MAX_OCTAVES));\n\
        let cl_v = cl_curl(\n\
        \x20   cl_p.x * params.scale,\n\
        \x20   cl_p.y * params.scale,\n\
        \x20   params.playhead * params.speed,\n\
        \x20   params.seed,\n\
        \x20   cl_oct);\n\
        let cl_w = params.strength * read_falloff(i);\n\
        write_accel(i, read_accel(i) + vec2<f32>(cl_v.x * cl_w, cl_v.y * cl_w));\n",
    wgsl_lib: "\
        const CL_EPS: f32 = 0.01;\n\
        const CL_MAX_OCTAVES: f32 = 4.0;\n\
        fn cl_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn cl_hash2(ix: i32, iy: i32) -> f32 {\n\
            var h: u32 = bitcast<u32>(ix) * 0x27d4eb2du + bitcast<u32>(iy) * 0x165667b1u;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x2c1b3c6du;\n\
            h = h ^ (h >> 12u);\n\
            h = h * 0x297175f9u;\n\
            h = h ^ (h >> 15u);\n\
            return (f32(h) / f32(0xffffffffu)) * 2.0 - 1.0;\n\
        }\n\
        fn cl_fade(t: f32) -> f32 {\n\
            return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);\n\
        }\n\
        fn cl_noise(x: f32, y: f32) -> f32 {\n\
            let x0 = floor(x);\n\
            let y0 = floor(y);\n\
            let ix = i32(x0);\n\
            let iy = i32(y0);\n\
            let u = cl_fade(x - x0);\n\
            let v = cl_fade(y - y0);\n\
            let n00 = cl_hash2(ix, iy);\n\
            let n10 = cl_hash2(ix + 1, iy);\n\
            let n01 = cl_hash2(ix, iy + 1);\n\
            let n11 = cl_hash2(ix + 1, iy + 1);\n\
            let nx0 = n00 + u * (n10 - n00);\n\
            let nx1 = n01 + u * (n11 - n01);\n\
            return nx0 + v * (nx1 - nx0);\n\
        }\n\
        fn cl_fbm(x: f32, y: f32, octaves: i32) -> f32 {\n\
            var freq = 1.0;\n\
            var amp = 1.0;\n\
            var sum = 0.0;\n\
            var norm = 0.0;\n\
            let n = max(octaves, 1);\n\
            for (var k = 0; k < n; k = k + 1) {\n\
                sum = sum + cl_noise(x * freq, y * freq) * amp;\n\
                norm = norm + amp;\n\
                freq = freq * 2.0;\n\
                amp = amp * 0.5;\n\
            }\n\
            return sum / norm;\n\
        }\n\
        fn cl_psi(x: f32, y: f32, drift: f32, seed: f32, octaves: i32) -> f32 {\n\
            return cl_fbm(x + drift + seed, y, octaves);\n\
        }\n\
        fn cl_curl(x: f32, y: f32, drift: f32, seed: f32, octaves: i32) -> vec2<f32> {\n\
            let dpsi_dx =\n\
                cl_psi(x + CL_EPS, y, drift, seed, octaves)\n\
                - cl_psi(x - CL_EPS, y, drift, seed, octaves);\n\
            let dpsi_dy =\n\
                cl_psi(x, y + CL_EPS, drift, seed, octaves)\n\
                - cl_psi(x, y - CL_EPS, drift, seed, octaves);\n\
            let inv = 1.0 / (2.0 * CL_EPS);\n\
            return vec2<f32>(dpsi_dy * inv, -dpsi_dx * inv);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "accel",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["strength", "scale", "speed", "octaves", "seed"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

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
            // Pure per-instance map → parallel above the threshold
            // (bit-identical, no reduction). GPU/M5 Fase 0.
            let contrib: Vec<[f32; 2]> = par_build(input.count(), |i| {
                let p = vec2_at(input, "P", i, [0.0, 0.0]);
                let v = curl(p[0] * scale, p[1] * scale, drift, seed, octaves);
                let w = strength * falloff_at(input, i);
                [v[0] * w, v[1] * w]
            });
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
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // ADR-0130: per-element force: accumulates accel, identity preserved.
    reg.register_dense_window(MANIFEST.id);
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
