#![forbid(unsafe_code)]
//! `force.wind` — a directional force with per-instance gusts. A Motion
//! **force**: a `Pure`-shaped node (marked `Temporal` — the gusts read the
//! playhead) that adds its contribution into the transient `accel` column
//! (× the multiplicative `falloff` field, plan §1.6); `motion.integrate` turns
//! the accumulated acceleration into motion.
//!
//! Reference behaviour (MiniCavalryV2 `forces/wind.js`, clean-room):
//! `strength · (1 + gust · noise(t·gust_freq, row_i))` along a fixed direction.
//! Each instance samples its own noise row, so gusts ripple organically across
//! the set instead of pumping it in lockstep. The noise is the deterministic
//! value-noise lattice (hash + smootherstep, HR-5), the direction comes from
//! the parabolic-sine trig — no transcendentals anywhere.
//!
//! With `gust = 0` this is a constant directional force — i.e. **gravity**:
//! `angle = 270°` (Y-up world) and the strength of your choice. A separate
//! gravity node would be this node minus two params, so it does not exist.
//!
//! `angle` is authored in **degrees** (the app's one authored-angle unit),
//! counter-clockwise from +X: `0°` blows +X, `90°` blows +Y (up), `270°` down.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::par_build;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod accum;
mod noise;
mod trig;
use accum::{add_accel, falloff_at};
use noise::value_noise_2d;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.wind"),
    name: "force.wind",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // The gusts sample the playhead-scrolled noise field.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "angle",
            default: 0.0,
        },
        ParamSpec {
            name: "strength",
            default: 3.0,
        },
        ParamSpec {
            name: "gust",
            default: 0.3,
        },
        ParamSpec {
            name: "gust_freq",
            default: 1.0,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (GPU/M5 **Fase 3**, ADR-0126 side channel): the exact
/// per-element map of the CPU `eval` — a constant direction, gusted by a noise
/// row per instance that scrolls with the playhead.
///
/// **HR-5:** the direction uses this node's OWN corrected parabolic sine (a port
/// of `trig.rs`), never WGSL's `sin`/`cos`, which carry no cross-vendor
/// guarantee; and the gust is the same integer-hash value noise as `noise.rs`
/// (`bitcast<u32>` == Rust's `as u32` — `u32(x)` is a VALUE cast and diverges on
/// negatives; WGSL u32 wraps mod 2^32 like `wrapping_*`). A single lattice cell
/// of divergence would not be ε — the noise would jump to an unrelated
/// pseudo-random value.
///
/// The CPU resolves the direction once per node rather than per element; the
/// arithmetic is identical either way, so recomputing it per invocation costs a
/// few ALU and keeps the body a pure function of `(i, params)`.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let wd_dir = wd_cos_sin_cycles(params.angle / 360.0);\n\
        let wd_var = params.gust\n\
        \x20   * wd_noise(params.playhead * params.gust_freq, f32(i) * 0.5 + params.seed);\n\
        let wd_mag = params.strength * (1.0 + wd_var) * read_falloff(i);\n\
        write_accel(i, read_accel(i) + vec2<f32>(wd_dir.x * wd_mag, wd_dir.y * wd_mag));\n",
    wgsl_lib: "\
        fn wd_sin_cycles(phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
            var p: f32;\n\
            if (f < 0.5) {\n\
                let u = f * 2.0;\n\
                p = 4.0 * u * (1.0 - u);\n\
            } else {\n\
                let u = (f - 0.5) * 2.0;\n\
                p = -4.0 * u * (1.0 - u);\n\
            }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n\
        fn wd_cos_sin_cycles(phase: f32) -> vec2<f32> {\n\
            return vec2<f32>(wd_sin_cycles(phase + 0.25), wd_sin_cycles(phase));\n\
        }\n\
        fn wd_hash2(ix: i32, iy: i32) -> f32 {\n\
            var h: u32 = bitcast<u32>(ix) * 0x27d4eb2du + bitcast<u32>(iy) * 0x165667b1u;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x2c1b3c6du;\n\
            h = h ^ (h >> 12u);\n\
            h = h * 0x297175f9u;\n\
            h = h ^ (h >> 15u);\n\
            return (f32(h) / f32(0xffffffffu)) * 2.0 - 1.0;\n\
        }\n\
        fn wd_fade(t: f32) -> f32 {\n\
            return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);\n\
        }\n\
        fn wd_noise(x: f32, y: f32) -> f32 {\n\
            let x0 = floor(x);\n\
            let y0 = floor(y);\n\
            let ix = i32(x0);\n\
            let iy = i32(y0);\n\
            let u = wd_fade(x - x0);\n\
            let v = wd_fade(y - y0);\n\
            let n00 = wd_hash2(ix, iy);\n\
            let n10 = wd_hash2(ix + 1, iy);\n\
            let n01 = wd_hash2(ix, iy + 1);\n\
            let n11 = wd_hash2(ix + 1, iy + 1);\n\
            let nx0 = n00 + u * (n10 - n00);\n\
            let nx1 = n01 + u * (n11 - n01);\n\
            return nx0 + v * (nx1 - nx0);\n\
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
    ],
    params: &["angle", "strength", "gust", "gust_freq", "seed"],
    source_count: None,
    applicable: None,
};

struct ForceWind;

impl NodeOp for ForceWind {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Degrees → cycles for the transcendental-free trig.
        let (dir_x, dir_y) = cos_sin_cycles(ctx.param("angle") / 360.0);
        let strength = ctx.param("strength");
        let gust = ctx.param("gust");
        let gust_freq = ctx.param("gust_freq");
        let seed = ctx.param("seed");
        let t = ctx.playhead() as f32;
        let out = {
            let input = ctx.input(0);
            // Pure per-instance map → parallel above the threshold
            // (bit-identical, no reduction). GPU/M5 Fase 0.
            let contrib: Vec<[f32; 2]> = par_build(input.count(), |i| {
                // Each instance = its own noise row (reference parity: the
                // per-instance seed picks the row; time scrolls along x).
                let variation = gust * value_noise_2d(t * gust_freq, i as f32 * 0.5 + seed);
                let mag = strength * (1.0 + variation) * falloff_at(input, i);
                [dir_x * mag, dir_y * mag]
            });
            add_accel(input, &contrib)
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ForceWind))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Wind",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "angle",
        label: "Angle",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "gust",
        label: "Gust",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "gust_freq",
        label: "Gust Frequency",
        min: 0.1,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
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
        id: NodeTypeId::of("force.wind.test.src"),
        name: "force.wind.test.src",
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
            ctx.emit(Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&ForceWind),
                _ => None,
            }
        }
    }

    fn accel_with(params: &[(&str, f32)], playhead: f64) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("force.wind.test.src");
        let w = g.add_node("force.wind");
        g.connect(Edge {
            from: (src, 0),
            to: (w, 0),
            delayed: false,
        })
        .unwrap();
        for (name, v) in params {
            g.set_param(w, *name, *v);
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, w, playhead).unwrap();
        match out[0].as_stream().get("accel").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("accel"),
        }
    }

    #[test]
    fn gustless_wind_is_a_constant_directional_force() {
        // gust = 0 → exactly strength along the angle. 0° = +X.
        let a = accel_with(&[("gust", 0.0), ("strength", 3.0)], 0.7);
        for ai in &a {
            assert!((ai[0] - 3.0).abs() < 1e-5, "blows +X at strength");
            assert!(ai[1].abs() < 1e-5);
        }
        // 270° = −Y (gravity in a Y-up world).
        let g = accel_with(&[("gust", 0.0), ("angle", 270.0)], 0.0);
        assert!(g[0][0].abs() < 1e-5);
        assert!((g[0][1] + 3.0).abs() < 1e-5, "270 deg pulls down (Y-up)");
    }

    #[test]
    fn gusts_vary_per_instance_and_over_time() {
        let a = accel_with(&[("gust", 0.5)], 0.4);
        assert!(
            a[0][0] != a[1][0] || a[1][0] != a[2][0],
            "each instance rides its own noise row"
        );
        let b = accel_with(&[("gust", 0.5)], 1.9);
        assert!((a[0][0] - b[0][0]).abs() > 1e-6, "gusts scroll with time");
    }

    #[test]
    fn is_deterministic_for_replay() {
        assert_eq!(
            accel_with(&[("gust", 0.7)], 0.9),
            accel_with(&[("gust", 0.7)], 0.9)
        );
    }

    /// The focus field gates the force (audit 2026-07-10: untested until now):
    /// gustless wind at strength 3 scales per-instance by the `falloff` column
    /// — 1.0 blows full, 0.25 a quarter, 0.0 not at all.
    #[test]
    fn falloff_scales_the_force_per_instance() {
        static FSRC_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("force.wind.test.fsrc"),
            name: "force.wind.test.fsrc",
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
        struct FSrc;
        impl NodeOp for FSrc {
            fn manifest(&self) -> &'static NodeManifest {
                &FSRC_MAN
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(3)
                        .with("P", Column::Vec2(vec![[0.0, 0.0]; 3]))
                        .with("falloff", Column::Scalar(vec![1.0, 0.25, 0.0])),
                );
            }
        }
        struct FOps;
        impl OpResolver for FOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == FSRC_MAN.id => Some(&FSrc),
                    t if t == MANIFEST.id => Some(&ForceWind),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("force.wind.test.fsrc");
        let w = g.add_node("force.wind");
        g.connect(Edge {
            from: (src, 0),
            to: (w, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(w, "gust", 0.0);
        g.set_param(w, "strength", 3.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &FOps, w, 0.0).unwrap();
        let a = match out[0].as_stream().get("accel").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("accel"),
        };
        assert!((a[0][0] - 3.0).abs() < 1e-5, "full falloff: full strength");
        assert!((a[1][0] - 0.75).abs() < 1e-5, "quarter falloff: a quarter");
        assert_eq!(a[2], [0.0, 0.0], "falloff 0: untouched");
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
