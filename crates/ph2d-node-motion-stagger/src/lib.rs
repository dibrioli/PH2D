#![forbid(unsafe_code)]
//! `motion.stagger` — a Motion **behaviour**: writes a per-index ramp `min→max`
//! (shaped by an easing curve) into a chosen channel of the stream, **added** to
//! the existing value and scaled per-instance by the multiplicative `falloff`
//! column (§1.2; absent → `1.0`). The ramp uses the instance's position in the
//! stream (`i / (n-1)`), so the first instance takes ≈`min·falloff` and the last
//! ≈`max·falloff` (`reverse` flips the direction). Index-based, not time-based →
//! `Pure`. Every other column passes through unchanged (count preserved).
//!
//! Params (read via `ctx.param`):
//! - `channel` (1): target channel — `0` X, `1` Y, `2` Rotation, `3` Size.
//!   Position is world units, Rotation is **degrees** (the `rot` column's unit),
//!   Size a scale delta on the unit identity `[1,1]`.
//! - `min` (-1), `max` (1): the ramp endpoints (channel-native units).
//! - `ease_curve` (0): curve family — `0` Linear, `1` Quad, `2` Cubic, `3` Quart,
//!   `4` Quint, `5` Circ, `6` Back, `7` Bounce. All polynomial except Circ,
//!   which uses IEEE `sqrt` — correctly rounded, so still deterministic (HR-5;
//!   no transcendentals anywhere).
//! - `ease_dir` (0): the family's direction — `0` In, `1` Out, `2` In-Out.
//! - `reverse` (0/1): mirror the ramp (last instance takes `min`).
//!
//! `delta_i = (min + ease(raw_i)·(max−min)) · falloff_i`, added to the channel.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
mod ease;
use channel::{apply_channel_delta, falloff_at};
use ease::ease;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.stagger"),
    name: "motion.stagger",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Index-based (no playhead, no state) → freely cacheable.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "channel",
            default: 1.0,
        },
        ParamSpec {
            name: "min",
            default: -1.0,
        },
        ParamSpec {
            name: "max",
            default: 1.0,
        },
        // Easing = a curve family shaped by a direction (see `ease`).
        ParamSpec {
            name: "ease_curve",
            default: 0.0,
        },
        ParamSpec {
            name: "ease_dir",
            default: 0.0,
        },
        ParamSpec {
            name: "reverse",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The per-instance stagger delta before it's added to the channel: the eased
/// ramp `min→max` at position `raw ∈ [0,1]`, scaled by `falloff`. Easing is the
/// `curve` family shaped by `dir` (In / Out / In-Out) — see [`ease`].
fn stagger_delta(min: f32, max: f32, curve: i32, dir: i32, raw: f32, falloff: f32) -> f32 {
    (min + ease(curve, dir, raw) * (max - min)) * falloff
}

/// GPU compute kernel (ADR-0126) — the WGSL port of [`ease`] + [`stagger_delta`].
///
/// The ramp is the element's **position in the stream**, `i/(n−1)`, so this kernel
/// is one of the few that genuinely needs `params.count` (the engine's own uniform)
/// rather than only its own columns. A single-element stream reads 0, matching the
/// CPU's `n <= 1` guard — and dividing by `n−1` without it is a divide by zero.
///
/// The whole easing table ports verbatim: 8 curve families × 3 directions, all
/// polynomial or `sqrt` (HR-5 — no `pow`, no transcendental), with In-Out built by
/// reflecting the In base exactly as the CPU does. `sg_round` is half-away-from-zero
/// because `channel`/`ease_curve`/`ease_dir` all pick BRANCHES: a half-even
/// disagreement would select a different curve, not shift a value by an ε.
///
/// **Covers the X/Y channels only** (`applicable`, the `motion.oscillator`
/// precedent): Rotation/Size write a different column.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        var sg_raw = 0.0;\n\
        if (params.count > 1u) {\n\
        \x20   sg_raw = f32(i) / (f32(params.count) - 1.0);\n\
        }\n\
        if (params.reverse >= 0.5) { sg_raw = 1.0 - sg_raw; }\n\
        let sg_e = sg_ease(i32(sg_round(params.ease_curve)),\n\
        \x20   i32(sg_round(params.ease_dir)), sg_raw);\n\
        let sg_d = (params.min + sg_e * (params.max - params.min)) * read_falloff(i);\n\
        var sg_p = read_P(i);\n\
        if (params.channel < 0.5) {\n\
        \x20   sg_p.x = sg_p.x + sg_d;\n\
        } else {\n\
        \x20   sg_p.y = sg_p.y + sg_d;\n\
        }\n\
        write_P(i, sg_p);\n",
    wgsl_lib: "\
        fn sg_round(x: f32) -> f32 {\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn sg_bounce_out(t: f32) -> f32 {\n\
            let n1 = 7.5625;\n\
            let d1 = 2.75;\n\
            if (t < 1.0 / d1) { return n1 * t * t; }\n\
            if (t < 2.0 / d1) {\n\
                let u = t - 1.5 / d1;\n\
                return n1 * u * u + 0.75;\n\
            }\n\
            if (t < 2.5 / d1) {\n\
                let u = t - 2.25 / d1;\n\
                return n1 * u * u + 0.9375;\n\
            }\n\
            let u = t - 2.625 / d1;\n\
            return n1 * u * u + 0.984375;\n\
        }\n\
        fn sg_ease_in(curve: i32, t: f32) -> f32 {\n\
            if (curve == 1) { return t * t; }\n\
            if (curve == 2) { return t * t * t; }\n\
            if (curve == 3) { return t * t * t * t; }\n\
            if (curve == 4) { return t * t * t * t * t; }\n\
            if (curve == 5) { return 1.0 - sqrt(max(1.0 - t * t, 0.0)); }\n\
            if (curve == 6) {\n\
                let c1 = 1.70158;\n\
                let c3 = c1 + 1.0;\n\
                return c3 * t * t * t - c1 * t * t;\n\
            }\n\
            if (curve == 7) { return 1.0 - sg_bounce_out(1.0 - t); }\n\
            return t;\n\
        }\n\
        fn sg_ease(curve: i32, dir: i32, t: f32) -> f32 {\n\
            if (curve == 0) { return t; }\n\
            if (dir == 1) { return 1.0 - sg_ease_in(curve, 1.0 - t); }\n\
            if (dir == 2) {\n\
                if (t < 0.5) { return sg_ease_in(curve, 2.0 * t) * 0.5; }\n\
                return 1.0 - sg_ease_in(curve, 2.0 - 2.0 * t) * 0.5;\n\
            }\n\
            return sg_ease_in(curve, t);\n\
        }\n",
    bindings: &[
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
    params: &["channel", "min", "max", "ease_curve", "ease_dir", "reverse"],
    count_law: None,
    variant_by_param: None,
    applicable: Some(|param| {
        let channel = param("channel").round();
        channel == 0.0 || channel == 1.0
    }),
};

struct MotionStagger;

impl NodeOp for MotionStagger {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let min = ctx.param("min");
        let max = ctx.param("max");
        let curve = ctx.param("ease_curve").round() as i32;
        let dir = ctx.param("ease_dir").round() as i32;
        let reverse = ctx.param("reverse") >= 0.5;
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            let deltas: Vec<f32> = (0..n)
                .map(|i| {
                    // Position in the stream, `0..1` (a single instance → 0).
                    let raw = if n <= 1 {
                        0.0
                    } else {
                        i as f32 / (n as f32 - 1.0)
                    };
                    let raw = if reverse { 1.0 - raw } else { raw };
                    stagger_delta(min, max, curve, dir, raw, falloff_at(input, i))
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
    reg.register(Box::new(MotionStagger))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // M1.R1 — UI metadata. Behaviours modify transform channels → Transform
    // (blue) for now; a dedicated Behaviour category (cyan) is a follow-up.
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Stagger",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1). `channel` / `ease_curve` / `ease_dir` are **named**
/// selectors (segmented buttons), `reverse` a checkbox — never number sliders.
/// Easing is a curve family × direction (the Penner set minus the transcendental
/// ones); the enum option index IS the param value.
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
        param: "min",
        label: "Min",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "max",
        label: "Max",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "ease_curve",
        label: "Ease",
        min: 0.0,
        max: 7.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "Linear", "Quad", "Cubic", "Quart", "Quint", "Circ", "Back", "Bounce",
            ],
        },
    },
    ParamUiHint {
        param: "ease_dir",
        label: "Direction",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["In", "Out", "In-Out"],
        },
    },
    ParamUiHint {
        param: "reverse",
        label: "Reverse",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 3 instances on a line (x = 0,1,2) with an existing rot [0,0,0].
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.stagger.test.src"),
        name: "motion.stagger.test.src",
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
            ctx.emit(
                Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionStagger),
                _ => None,
            }
        }
    }

    fn staggered_y(setup: impl FnOnce(&mut Graph, ph2d_nodegraph::graph::NodeId)) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("motion.stagger.test.src");
        let st = g.add_node("motion.stagger");
        g.connect(Edge {
            from: (src, 0),
            to: (st, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(st, "channel", 1.0); // Y
        setup(&mut g, st);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, st, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    #[test]
    fn linear_ramp_adds_min_to_max_across_the_stream() {
        // channel=Y, min=0, max=2, linear: raw = 0, 0.5, 1 → Δy = 0, 1, 2 added.
        let p = staggered_y(|g, st| {
            g.set_param(st, "min", 0.0);
            g.set_param(st, "max", 2.0);
        });
        assert_eq!(p, vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]]);
    }

    #[test]
    fn reverse_flips_the_ramp_endpoints() {
        // reverse: raw = 1, 0.5, 0 → Δy = 2, 1, 0. X passes through unchanged.
        let p = staggered_y(|g, st| {
            g.set_param(st, "min", 0.0);
            g.set_param(st, "max", 2.0);
            g.set_param(st, "reverse", 1.0);
        });
        assert_eq!(p, vec![[0.0, 2.0], [1.0, 1.0], [2.0, 0.0]]);
    }

    #[test]
    fn easing_applies_a_non_linear_curve_to_the_ramp() {
        // Quad-In (curve 1, dir 0) bends the ramp below the diagonal at the
        // midpoint: min=0,max=4 over 3 instances → mid Δy = 4·ease(1,0,0.5) = 1
        // (vs 2 for linear). Proves ease_curve/ease_dir flow into the delta.
        let p = staggered_y(|g, st| {
            g.set_param(st, "min", 0.0);
            g.set_param(st, "max", 4.0);
            g.set_param(st, "ease_curve", 1.0); // Quad
            g.set_param(st, "ease_dir", 0.0); // In
        });
        assert_eq!(p[0][1], 0.0); // endpoint
        assert_eq!(p[1][1], 1.0); // eased midpoint (below linear's 2)
        assert_eq!(p[2][1], 4.0); // endpoint
    }

    #[test]
    fn single_instance_takes_min() {
        // n == 1 → raw = 0 → delta = min (no divide-by-zero on `n-1`).
        assert_eq!(stagger_delta(3.0, 9.0, 0, 0, 0.0, 1.0), 3.0);
    }

    /// The focus field gates the ramp (audit 2026-07-10: untested until now):
    /// a source that carries `falloff` [1, 0.5, 0] sees the linear 0→2 ramp
    /// scaled per instance — the masked tail doesn't move at all.
    #[test]
    fn falloff_scales_the_ramp_per_instance() {
        static FSRC_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.stagger.test.fsrc"),
            name: "motion.stagger.test.fsrc",
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
                        .with("falloff", Column::Scalar(vec![1.0, 0.5, 0.0])),
                );
            }
        }
        struct FOps;
        impl OpResolver for FOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == FSRC_MAN.id => Some(&FSrc),
                    t if t == MANIFEST.id => Some(&MotionStagger),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("motion.stagger.test.fsrc");
        let st = g.add_node("motion.stagger");
        g.connect(Edge {
            from: (src, 0),
            to: (st, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(st, "channel", 1.0); // Y
        g.set_param(st, "min", 0.0);
        g.set_param(st, "max", 2.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &FOps, st, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            // Linear raw = 0, 0.5, 1 → Δy = 0, 1, 2; × falloff [1, 0.5, 0]
            // → 0, 0.5, 0. The masked LAST instance stays put even though the
            // raw ramp peaks there.
            Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 0.0], [0.0, 0.5], [0.0, 0.0]]),
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
