#![forbid(unsafe_code)]
//! `motion.noise` — a Motion **behaviour**: a coherent Perlin **gradient**-noise
//! FIELD that displaces a chosen channel, added to the existing value and scaled
//! per-instance by the multiplicative `falloff` column (§1.2; absent → `1.0`).
//! Reads the playhead but holds no state → `Effect::Temporal`.
//!
//! **Field, not jitter — the distinction from `motion.wiggle`.** Wiggle samples
//! `noise(time, instance_index)`: each element jitters on its own row, so they
//! move INDEPENDENTLY (nervous jitter). Noise samples `noise(position·scale,
//! time)`: neighbouring elements read nearby points of one continuous field, so
//! they flow TOGETHER — coherent turbulence (smoke, current, drift). And it is
//! **gradient** noise, not the **value** noise wiggle uses: gradient noise is
//! zero at every lattice point, so it has none of value noise's grid artifacts
//! (see [`noise`] and docs/Motion Nodes/07).
//!
//! Gold standard (doc 07): Improved Perlin 2002 (quintic fade, 8 isotropic
//! gradients) + fBm, transcendental-free (HR-5). Param surface is the
//! cross-tool intersection (Cavalry/AE/Houdini/Blender): scale, octaves,
//! roughness, type, speed, seed.
//!
//! `delta_i = fbm(P_i·scale, seed, octaves, roughness, type @ t·speed) ·
//! amplitude · falloff_i`, added to the chosen channel.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
mod noise;
use channel::{apply_channel_delta, falloff_at};
use noise::{NoiseType, fbm_2d};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Hard ceiling on octaves — an untrusted `f32` param drives the fBm loop count.
/// 8 is past the point of visible return (each octave halves the feature size).
const MAX_OCTAVES: u32 = 8;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.noise"),
    name: "motion.noise",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Reads the playhead → pull-side; the noise is nonetheless deterministic.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // 0 X · 1 Y · 2 Rotation · 3 Size — the shared channel vocabulary.
        ParamSpec {
            name: "channel",
            default: 1.0,
        },
        // Peak of the displacement (channel-native units), before falloff.
        ParamSpec {
            name: "amplitude",
            default: 1.0,
        },
        // Spatial frequency: feature size of the field. World units are metres
        // (~single digits), so a smaller scale than a pixel tool's — 0.4 gives
        // features a couple of metres across.
        ParamSpec {
            name: "scale",
            default: 0.4,
        },
        // fBm octaves (= AE "Complexity" / Blender "Detail").
        ParamSpec {
            name: "octaves",
            default: 3.0,
        },
        // Per-octave amplitude falloff (= Houdini/Blender "Roughness", the
        // gain/persistence). 0.5 is the universal default.
        ParamSpec {
            name: "roughness",
            default: 0.5,
        },
        // 0 fBm · 1 Turbulence · 2 Ridged.
        ParamSpec {
            name: "type",
            default: 0.0,
        },
        // Temporal scroll speed (= AE "Evolution" / Cavalry "Time Scale"): the
        // field drifts through the elements over playhead-seconds.
        ParamSpec {
            name: "speed",
            default: 0.4,
        },
        // Decorrelates several Noise nodes.
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126): the WGSL port of [`noise::fbm_2d`] over each
/// element's own `P`, element for element.
///
/// **Gradient noise, ported as gradient noise.** `force.curl` already ships an
/// fBm in WGSL and it is the wrong one to reuse: that is VALUE noise (it lerps a
/// per-corner random value), while this node is Perlin 2002 GRADIENT noise (it
/// lerps the dot of a per-corner gradient with the distance vector). The whole
/// reason this node exists rather than `motion.wiggle` is that gradient noise is
/// exactly zero at every lattice point and so has no grid artifacts — reusing the
/// curl's lib would have silently made the GPU draw a DIFFERENT field from the CPU,
/// and the parity gate would have been the only thing standing between that and a
/// shipped visual divergence.
///
/// **The discrete params are rounded the way Rust rounds.** `octaves`, `type` and
/// `seed` all pick a BRANCH (or a hash), so a half-even/half-away disagreement is
/// not an ε — it is a different field. `ns_round` is round-half-away-from-zero to
/// match `f32::round`, the same guard `motion.oscillator`'s `osc_round` carries
/// ([[feedback_cpu_gpu_rounding_conventions_diverge]]).
///
/// **`params.type_`** — `type` is a WGSL reserved word, so the generated uniform
/// field takes a trailing underscore (`codegen::wgsl_field`). The param keeps its
/// artist-facing name.
///
/// **Covers the X/Y channels only** (`applicable`, the `motion.oscillator`
/// precedent): Rotation/Size write a different column, which a static binding set
/// cannot switch on, so those recede to the CPU rather than to a wrong answer.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let ns_p = read_P(i);\n\
        let ns_seed = i32(ns_round(params.seed));\n\
        let ns_oct = min(max(i32(ns_round(params.octaves)), 1), 8);\n\
        let ns_ty = i32(ns_round(params.type_));\n\
        let ns_s = ns_fbm(\n\
        \x20   ns_p.x * params.scale,\n\
        \x20   ns_p.y * params.scale + params.playhead * params.speed,\n\
        \x20   ns_seed, ns_oct, params.roughness, ns_ty);\n\
        let ns_d = ns_s * params.amplitude * read_falloff(i);\n\
        var ns_out = ns_p;\n\
        if (params.channel < 0.5) {\n\
        \x20   ns_out.x = ns_out.x + ns_d;\n\
        } else {\n\
        \x20   ns_out.y = ns_out.y + ns_d;\n\
        }\n\
        write_P(i, ns_out);\n",
    wgsl_lib: "\
        const NS_NORM: f32 = 1.0 / 1.5;\n\
        const NS_LACUNARITY: f32 = 2.0;\n\
        fn ns_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn ns_hash(ix: i32, iy: i32, seed: i32) -> u32 {\n\
            var h: u32 = bitcast<u32>(ix) * 0x27d4eb2du\n\
                + bitcast<u32>(iy) * 0x165667b1u\n\
                + bitcast<u32>(seed) * 0x01934f07u;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x2c1b3c6du;\n\
            h = h ^ (h >> 12u);\n\
            h = h * 0x297175f9u;\n\
            h = h ^ (h >> 15u);\n\
            return h;\n\
        }\n\
        fn ns_dot_grad(h: u32, dx: f32, dy: f32) -> f32 {\n\
            // The eight 2002 gradients (+-1,+-2)/(+-2,+-1), as +-u +- 2v.\n\
            let g = h & 7u;\n\
            var u = dx;\n\
            var v = dy;\n\
            if (g >= 4u) { u = dy; v = dx; }\n\
            let a = select(u, -u, (g & 1u) != 0u);\n\
            let b = select(2.0 * v, -2.0 * v, (g & 2u) != 0u);\n\
            return a + b;\n\
        }\n\
        fn ns_fade(t: f32) -> f32 {\n\
            return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);\n\
        }\n\
        fn ns_grad_noise(x: f32, y: f32, seed: i32) -> f32 {\n\
            let x0 = floor(x);\n\
            let y0 = floor(y);\n\
            let ix = i32(x0);\n\
            let iy = i32(y0);\n\
            let fx = x - x0;\n\
            let fy = y - y0;\n\
            let u = ns_fade(fx);\n\
            let v = ns_fade(fy);\n\
            let n00 = ns_dot_grad(ns_hash(ix, iy, seed), fx, fy);\n\
            let n10 = ns_dot_grad(ns_hash(ix + 1, iy, seed), fx - 1.0, fy);\n\
            let n01 = ns_dot_grad(ns_hash(ix, iy + 1, seed), fx, fy - 1.0);\n\
            let n11 = ns_dot_grad(ns_hash(ix + 1, iy + 1, seed), fx - 1.0, fy - 1.0);\n\
            let nx0 = n00 + u * (n10 - n00);\n\
            let nx1 = n01 + u * (n11 - n01);\n\
            return (nx0 + v * (nx1 - nx0)) * NS_NORM;\n\
        }\n\
        fn ns_fbm(x0: f32, y0: f32, seed: i32, octaves: i32, roughness: f32, ty: i32) -> f32 {\n\
            let gain = clamp(roughness, 0.0, 1.0);\n\
            var x = x0;\n\
            var y = y0;\n\
            var amp = 1.0;\n\
            var sum = 0.0;\n\
            var total = 0.0;\n\
            for (var o = 0; o < octaves; o = o + 1) {\n\
                // Per-octave seed offset: octaves must be independent fields,\n\
                // not scaled copies of one (which would beat visibly).\n\
                let n = ns_grad_noise(x, y, seed + o * 1013);\n\
                var shaped = n;\n\
                if (ty == 1) {\n\
                    shaped = abs(n);\n\
                } else if (ty == 2) {\n\
                    let r = 1.0 - abs(n);\n\
                    shaped = r * r;\n\
                }\n\
                sum = sum + amp * shaped;\n\
                total = total + amp;\n\
                amp = amp * gain;\n\
                x = x * NS_LACUNARITY;\n\
                y = y * NS_LACUNARITY;\n\
            }\n\
            return sum / total;\n\
        }\n",
    bindings: &[
        // The target channel is materialized from its identity when absent, the
        // same as the CPU's `apply_channel_delta` (`base_vec2`). `P` is also the
        // SAMPLE point, and each invocation reads only its own element, so the
        // read-then-write is not a race.
        ColumnBinding {
            column: "P",
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
    params: &[
        "channel",
        "amplitude",
        "scale",
        "octaves",
        "roughness",
        "type",
        "speed",
        "seed",
    ],
    source_window: None,
    applicable: Some(|param| {
        // The same rounding the CPU `eval` applies. Only X (0) / Y (1) write `P`.
        let channel = param("channel").round();
        channel == 0.0 || channel == 1.0
    }),
};

struct MotionNoise;

impl NodeOp for MotionNoise {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let amplitude = ctx.param("amplitude");
        let scale = ctx.param("scale");
        let octaves = (ctx.param("octaves").round().max(1.0) as u32).min(MAX_OCTAVES);
        let roughness = ctx.param("roughness");
        let ty = NoiseType::from_index(ctx.param("type"));
        let speed = ctx.param("speed");
        let seed = ctx.param("seed").round() as i32;
        let t = ctx.playhead() as f32;

        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Each element's own world position is the sample point, so the field
            // is spatially coherent; the playhead scrolls it along Y (the field
            // "flows" through the elements).
            let pos = positions(input, n);
            let deltas: Vec<f32> = (0..n)
                .map(|i| {
                    let (px, py) = pos[i];
                    let s = fbm_2d(
                        px * scale,
                        py * scale + t * speed,
                        seed,
                        octaves,
                        roughness,
                        ty,
                    );
                    s * amplitude * falloff_at(input, i)
                })
                .collect();
            apply_channel_delta(input, channel, &deltas)
        };
        ctx.emit(out);
    }
}

/// Each element's `P` (absent → origin), the field's sample points.
fn positions(input: &Stream, n: usize) -> Vec<(f32, f32)> {
    match input.get("P") {
        Some(Column::Vec2(v)) => {
            let mut out: Vec<(f32, f32)> = v.iter().map(|p| (p[0], p[1])).collect();
            out.resize(n, (0.0, 0.0));
            out
        }
        _ => vec![(0.0, 0.0); n],
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionNoise))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Noise",
            // Transform blue: a spatial behaviour that moves elements.
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "channel",
        label: "Channel",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["X", "Y", "Rotation", "Size"],
        },
    },
    ParamUiHint {
        param: "amplitude",
        label: "Amplitude",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "scale",
        label: "Scale",
        min: 0.02,
        max: 2.0,
        step: 0.02,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "octaves",
        label: "Octaves",
        min: 1.0,
        max: MAX_OCTAVES as f32,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "roughness",
        label: "Roughness",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "type",
        label: "Type",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["fBm", "Turbulence", "Ridged"],
        },
    },
    ParamUiHint {
        param: "speed",
        label: "Speed",
        min: 0.0,
        max: 3.0,
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
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Graph, NodeId as GNodeId};

    struct Reg;
    impl OpResolver for Reg {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            // A tiny grid source + this node.
            static SRC: NodeManifest = NodeManifest {
                id: NodeTypeId::of("test.grid"),
                name: "test.grid",
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
                    // A 4-across row so neighbours are spatially close.
                    let p: Vec<[f32; 2]> = (0..8).map(|i| [i as f32, 0.0]).collect();
                    ctx.emit(Stream::new(8).with("P", Column::Vec2(p)));
                }
            }
            match ty {
                t if t == MANIFEST.id => {
                    static N: MotionNoise = MotionNoise;
                    Some(&N)
                }
                t if t == SRC.id => {
                    static S: Src = Src;
                    Some(&S)
                }
                _ => None,
            }
        }
    }

    fn cook_y(graph: &Graph, node: GNodeId, t: f64) -> Vec<f32> {
        let mut cook = Cook::new();
        let out = cook.cook(graph, &Reg, node, t).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.iter().map(|p| p[1]).collect(),
            _ => panic!("P"),
        }
    }

    /// Cooked through the substrate: the noise displaces Y, and the field is
    /// COHERENT — neighbouring elements move by similar amounts (a smooth field),
    /// unlike a per-element jitter which would be uncorrelated. This is the whole
    /// point of a spatial noise field vs `motion.wiggle`.
    #[test]
    fn the_field_displaces_y_coherently_across_neighbours() {
        let mut g = Graph::new();
        let src = g.add_node("test.grid");
        let noise = g.add_node("motion.noise");
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (src, 0),
            to: (noise, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(noise, "amplitude", 1.0);
        g.set_param(noise, "scale", 0.25); // large features → strong neighbour correlation
        g.set_param(noise, "octaves", 1.0);

        let ys = cook_y(&g, noise, 0.5);
        // Something moved (the field is not flat).
        assert!(
            ys.iter().any(|&y| y.abs() > 0.01),
            "the field displaced nothing"
        );
        // Coherence: at a large feature size, adjacent elements differ far less
        // than the amplitude — they belong to the same swell, not independent
        // random draws. Max neighbour step is a fraction of the peak.
        let max_step = ys
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0_f32, f32::max);
        let peak = ys.iter().map(|y| y.abs()).fold(0.0_f32, f32::max);
        assert!(
            max_step < peak * 0.9,
            "neighbours should move coherently: step {max_step} vs peak {peak}"
        );
    }

    /// The playhead scrolls the field (Temporal): the same elements read a
    /// different slice of the field at a later time.
    #[test]
    fn the_field_evolves_with_the_playhead() {
        let mut g = Graph::new();
        let src = g.add_node("test.grid");
        let noise = g.add_node("motion.noise");
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (src, 0),
            to: (noise, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(noise, "speed", 1.0);
        assert_ne!(cook_y(&g, noise, 0.0), cook_y(&g, noise, 1.0));
    }
}
