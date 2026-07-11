#![forbid(unsafe_code)]
//! `motion.wiggle` — a Motion **behaviour**: adds a smooth pseudo-random offset
//! to a chosen channel, **added** to the existing value and scaled per-instance
//! by the multiplicative `falloff` column (§1.2; absent → `1.0`). The AE-style
//! *wiggle* — each instance samples its own row of a 2D value-noise field scrolled
//! by the playhead, so they jitter organically and independently. Reads the
//! playhead but holds no state → `Effect::Temporal` (pull-side). Every other
//! column passes through unchanged (count preserved).
//!
//! The noise is deterministic / transcendental-free (HR-5): an integer-hash
//! lattice + smootherstep fade + bilinear lerp (see [`noise`]).
//!
//! Params (read via `ctx.param`):
//! - `channel` (1): target — `0` X, `1` Y, `2` Rotation, `3` Size.
//! - `amplitude` (1): peak of the jitter (channel-native units).
//! - `frequency` (0.5): how fast the noise scrolls in playhead-seconds.
//! - `seed` (0): shifts the noise field so several Wiggles differ.
//!
//! `delta_i = value_noise(t·frequency, i + seed) · amplitude · falloff_i`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
mod noise;
use channel::{apply_channel_delta, falloff_at};
use noise::value_noise_2d;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.wiggle"),
    name: "motion.wiggle",
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
        ParamSpec {
            name: "channel",
            default: 1.0,
        },
        ParamSpec {
            name: "amplitude",
            default: 1.0,
        },
        ParamSpec {
            name: "frequency",
            default: 0.5,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct MotionWiggle;

impl NodeOp for MotionWiggle {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let amplitude = ctx.param("amplitude");
        let frequency = ctx.param("frequency");
        let seed = ctx.param("seed");
        let t = ctx.playhead() as f32;
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            let deltas: Vec<f32> = (0..n)
                .map(|i| {
                    // Each instance = a distinct noise row (`i + seed`), scrolled
                    // by time on the x-axis → independent organic wiggle.
                    let nx = t * frequency;
                    let ny = i as f32 + seed;
                    value_noise_2d(nx, ny) * amplitude * falloff_at(input, i)
                })
                .collect();
            apply_channel_delta(input, channel, &deltas)
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionWiggle))?;
    // M1.R1 — UI metadata. Behaviours modify transform channels → Transform
    // (blue) for now; a dedicated Behaviour category (cyan) is a follow-up.
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Wiggle",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1). `channel` is a named selector; `seed` a seed widget.
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
        param: "frequency",
        label: "Frequency",
        min: 0.0,
        max: 8.0,
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

    // Source: 4 instances at the origin (so the wiggle IS the output).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.wiggle.test.src"),
        name: "motion.wiggle.test.src",
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
            ctx.emit(Stream::new(4).with("P", Column::Vec2(vec![[0.0, 0.0]; 4])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionWiggle),
                _ => None,
            }
        }
    }

    fn wiggle_y_at(playhead: f64) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("motion.wiggle.test.src");
        let w = g.add_node("motion.wiggle");
        g.connect(Edge {
            from: (src, 0),
            to: (w, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(w, "channel", 1.0); // Y
        g.set_param(w, "amplitude", 2.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, w, playhead).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    #[test]
    fn instances_wiggle_independently_within_amplitude() {
        // At a non-zero time the four instances have different Y offsets (each its
        // own noise row), all bounded by ±amplitude, X untouched.
        let p = wiggle_y_at(0.7);
        for q in &p {
            assert_eq!(q[0], 0.0, "X untouched");
            assert!(q[1].abs() <= 2.0, "within ±amplitude");
        }
        assert!(
            p[0][1] != p[1][1] || p[1][1] != p[2][1],
            "instances jitter independently"
        );
    }

    #[test]
    fn the_wiggle_changes_over_time() {
        // Same instance, two different playheads → different offset (it scrolls).
        let a = wiggle_y_at(0.0);
        let b = wiggle_y_at(1.5);
        assert!((a[2][1] - b[2][1]).abs() > 1e-5, "wiggle scrolls with time");
    }

    #[test]
    fn is_deterministic_for_replay() {
        // The whole point of the transcendental-free noise: identical playhead →
        // identical output, every time (HR-5 replay).
        assert_eq!(wiggle_y_at(0.9), wiggle_y_at(0.9));
    }

    /// The focus field gates the jitter (audit 2026-07-10: untested until now):
    /// with `falloff` [1, 0] the second instance sits perfectly still while the
    /// first wiggles — a wiggle ignoring the mask would shake both.
    #[test]
    fn falloff_zero_pins_the_masked_instance() {
        static FSRC_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.wiggle.test.fsrc"),
            name: "motion.wiggle.test.fsrc",
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
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[0.0, 0.0]; 2]))
                        .with("falloff", Column::Scalar(vec![1.0, 0.0])),
                );
            }
        }
        struct FOps;
        impl OpResolver for FOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == FSRC_MAN.id => Some(&FSrc),
                    t if t == MANIFEST.id => Some(&MotionWiggle),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("motion.wiggle.test.fsrc");
        let w = g.add_node("motion.wiggle");
        g.connect(Edge {
            from: (src, 0),
            to: (w, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(w, "channel", 1.0); // Y
        g.set_param(w, "amplitude", 2.0);
        // A playhead where the focused row's noise is visibly non-zero.
        let mut cook = Cook::new();
        let out = cook.cook(&g, &FOps, w, 0.7).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => {
                assert!(v[0][1].abs() > 1e-4, "focused instance wiggles: {:?}", v[0]);
                assert_eq!(v[1], [0.0, 0.0], "masked instance is pinned");
            }
            _ => panic!("P"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
