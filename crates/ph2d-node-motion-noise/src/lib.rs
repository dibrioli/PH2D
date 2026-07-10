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
