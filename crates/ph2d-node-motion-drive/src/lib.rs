//! `motion.drive` — the value-domain CONSUMER: route a **value** field onto a
//! transform channel (Motion Nodes M2, the value domain — doc 12). This is the
//! single write-side node that replaces every behaviour that used to bundle its
//! own value math: instead of `motion.step` computing a count AND pushing X, you
//! wire `pulse.counter → motion.drive(channel = X)` — and the same value can fan
//! out to *several* drives (one count → X and Rotation at once), which no bundled
//! node can. It is the Cavalry "connect this value to that attribute" made a
//! first-class node.
//!
//! **The value type** is the continuous per-instance field `(Instances, Scalar,
//! Frame)` on the `v` column — the continuous dual of the pulse (doc 12).
//!
//! **The one broadcast rule (the load-bearing decision, doc 12):** a value field
//! of length 1 is HELD (broadcast) across every instance; a length-N field is
//! applied element-wise; anything else is a mismatch. This is TouchDesigner's
//! "held constant" / Houdini's "detail→point", restricted to `1→N` only so the
//! strict substrate never silently fits a 3-field to a 7-stream. It lives in
//! `channel::value_at`, and is what lets a single global LFO/counter drive many
//! instances without a scalar-vs-field node explosion (the reference
//! convergence — TD/Houdini/vvvv/Faust: a constant is the degenerate field).
//!
//! Params: `channel` (X/Y/Rotation/Size), `scale` (multiplies the value before
//! it hits the channel — the "count · step" that used to live in `motion.step`),
//! and `mode` (Add / Set / Multiply against the existing channel). Falloff-masked
//! like every behaviour. `Pure` (no clock, no state — a straight combinator).

#![forbid(unsafe_code)]

use channel::CH_OPACITY;
use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
use channel::{Combine, drive_channel};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value stream's column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.drive"),
    name: "motion.drive",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "value",
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
        // 0 X · 1 Y · 2 Rotation · 3 Size · 4 Opacity — the shared channel vocabulary.
        ParamSpec {
            name: "channel",
            default: 0.0,
        },
        // Multiplies the value before it hits the channel (the ex-`step`).
        ParamSpec {
            name: "scale",
            default: 1.0,
        },
        // 0 Add · 1 Set · 2 Multiply — how the value combines with the channel.
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The params every variant declares, in one order — the uniform layout is
/// per-variant, and keeping them identical means a reader never has to ask which
/// variant a `params.scale` belongs to.
const DRIVE_PARAMS: &[&str] = &["channel", "scale", "mode"];

/// The shared prologue: resolve the mode, the scaled value and the falloff mask.
/// Every variant pastes this and then writes ITS column.
///
/// `drive_round` is round-half-away-from-zero to match Rust's `f32::round` —
/// `mode` picks a BRANCH ([[feedback_cpu_gpu_rounding_conventions_diverge]]).
/// The falloff clamp MIRRORS the CPU's; no node writes a falloff outside `[0,1]`
/// today, so it is defensive on both sides rather than load-bearing.
const DRIVE_LIB: &str = "\
    fn drive_round(x: f32) -> f32 {\n\
        // Rust f32::round = half away from zero (WGSL round is half-even).\n\
        return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
    }\n\
    fn drive_combine(cur: f32, v: f32, mode: i32) -> f32 {\n\
        if (mode == 1) { return v; }\n\
        if (mode == 2) { return cur * v; }\n\
        return cur + v;\n\
    }\n";

/// `falloff` and the value port, bound identically by every variant.
macro_rules! drive_common {
    () => {
        [
            ColumnBinding {
                column: "falloff",
                dim: Dim::Scalar,
                access: ColumnAccess::Read,
                // Absent falloff = full effect, the CPU's `falloff_at` fallback.
                identity: [1.0, 0.0, 0.0, 0.0],
                port: 0,
            },
            ColumnBinding {
                column: VALUE_COL,
                dim: Dim::Scalar,
                access: ColumnAccess::ReadBroadcast,
                // Absent value = 0.0, the `0 =>` arm of `value_at`.
                identity: [0.0; 4],
                port: 1,
            },
        ]
    };
}

/// **X / Y** — writes one component of `P`.
const DRIVE_P: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_comp = i32(drive_round(params.channel));\n\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_p = read_in_P(i);\n\
        var dr_cur = dr_p.x;\n\
        if (dr_comp == 1) { dr_cur = dr_p.y; }\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_out = dr_cur + (drive_combine(dr_cur, dr_v, dr_mode) - dr_cur) * dr_f;\n\
        var dr_next = dr_p;\n\
        if (dr_comp == 1) { dr_next.y = dr_out; } else { dr_next.x = dr_out; }\n\
        write_P(i, dr_next);\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Rotation** — writes `rot`, in degrees like the CPU's.
const DRIVE_ROT: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_cur = read_in_rot(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        write_rot(i, dr_cur + (drive_combine(dr_cur, dr_v, dr_mode) - dr_cur) * dr_f);\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Size** — drives BOTH components uniformly, from the unit identity.
const DRIVE_SIZE: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_s = read_in_size(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_x = dr_s.x + (drive_combine(dr_s.x, dr_v, dr_mode) - dr_s.x) * dr_f;\n\
        let dr_y = dr_s.y + (drive_combine(dr_s.y, dr_v, dr_mode) - dr_s.y) * dr_f;\n\
        write_size(i, vec2<f32>(dr_x, dr_y));\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            // An element with no size starts UNIT, not zero (`base_vec2`).
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Opacity** — the ALPHA of `tint`, clamped to `[0,1]`. An element with no
/// tint starts from opaque white, so driving the opacity of an uncoloured stream
/// does what it says instead of silently nothing (doc 51).
const DRIVE_TINT: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_t = read_in_tint(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_a = dr_t.w + (drive_combine(dr_t.w, dr_v, dr_mode) - dr_t.w) * dr_f;\n\
        write_tint(i, vec4<f32>(dr_t.x, dr_t.y, dr_t.z, clamp(dr_a, 0.0, 1.0)));\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "tint",
            dim: Dim::Vec4,
            access: ColumnAccess::ReadWrite,
            identity: [1.0, 1.0, 1.0, 1.0],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// GPU compute kernel (ADR-0126) — the value domain's WRITE side on the device,
/// and the node that named [`GpuKernel::variant_by_param`].
///
/// `drive_channel` writes a DIFFERENT column per channel — `P` for X/Y, `rot`
/// for Rotation, `size` for Size, `tint` for Opacity — and materialises the
/// target from its identity when the stream lacks it. One static shape could not
/// express that: binding all four would emit columns **the CPU's output does not
/// carry** (a different stream SHAPE, not an ε), and binding one meant claiming
/// only the two channels that write `P`. So the node ships four variants and the
/// engine picks by `channel` — the SAME mapping `channel_column` uses, including
/// its `_ => size` catch-all for an out-of-range value.
///
/// The value port BROADCASTS: length 1 is one number held across the field (the
/// `1 => vals[0]` arm of `value_at`), length N is per-element.
const GPU_KERNEL: GpuKernel = GpuKernel {
    // The top-level shape IS the X/Y variant, so a caller that never resolves
    // still sees a real kernel rather than the empty (pass-through) one.
    wgsl: DRIVE_P.wgsl,
    wgsl_lib: DRIVE_P.wgsl_lib,
    bindings: DRIVE_P.bindings,
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: Some(|param| {
        // The same rounding and the same mapping as `channel_column`.
        match param("channel").round() as i32 {
            2 => &DRIVE_ROT,
            CH_OPACITY => &DRIVE_TINT,
            0 | 1 => &DRIVE_P,
            _ => &DRIVE_SIZE,
        }
    }),
    applicable: None,
};

struct MotionDrive;

impl NodeOp for MotionDrive {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let scale = ctx.param("scale");
        let mode = Combine::from_param(ctx.param("mode"));
        let vals: Vec<f32> = match ctx.input(1).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let out = drive_channel(ctx.input(0), channel, &vals, scale, mode);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionDrive))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Drive",
            // Transform blue: it writes a transform channel — a visible behaviour.
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
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["X", "Y", "Rotation", "Size", "Opacity"],
        },
    },
    ParamUiHint {
        param: "scale",
        label: "Scale",
        min: -4.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Add", "Set", "Multiply"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::Stream;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    // A source: 2 instances at the origin, plus a value node emitting one value.
    static GRID_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.drive.test.grid"),
        name: "motion.drive.test.grid",
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
    struct Grid;
    impl NodeOp for Grid {
        fn manifest(&self) -> &'static NodeManifest {
            &GRID_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]])));
        }
    }
    static VAL_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.drive.test.val"),
        name: "motion.drive.test.val",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: VALUE,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Val;
    impl NodeOp for Val {
        fn manifest(&self) -> &'static NodeManifest {
            &VAL_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(1).with(VALUE_COL, Column::Scalar(vec![3.0])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == GRID_MAN.id => Some(&Grid),
                t if t == VAL_MAN.id => Some(&Val),
                t if t == MANIFEST.id => Some(&MotionDrive),
                _ => None,
            }
        }
    }

    fn drive_graph(setup: impl FnOnce(&mut Graph, NodeId)) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let grid = g.add_node("motion.drive.test.grid");
        let val = g.add_node("motion.drive.test.val");
        let drive = g.add_node("motion.drive");
        g.connect(Edge {
            from: (grid, 0),
            to: (drive, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (val, 0),
            to: (drive, 1),
            delayed: false,
        })
        .unwrap();
        setup(&mut g, drive);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, drive, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    /// The end-to-end value path through the cook: a length-1 value from a
    /// separate node drives a channel of the instance stream, broadcast to all
    /// instances. This is what proves the value domain is wired (value produced
    /// by one node, consumed by another, made visible).
    #[test]
    fn a_value_node_drives_the_grid_channel_through_the_cook() {
        // scale 0.5, add → each instance X += 3 · 0.5 = 1.5.
        let p = drive_graph(|g, d| {
            g.set_param(d, "channel", 0.0); // X
            g.set_param(d, "scale", 0.5);
        });
        assert_eq!(
            p,
            vec![[1.5, 0.0], [1.5, 0.0]],
            "value broadcast to both, scaled"
        );
    }

    /// The value-domain WIN a bundled node can't do: ONE value node fans out to
    /// TWO drives — X and Rotation — off the same value. `motion.step` (reduce +
    /// apply in one node) can only touch one channel; the split lets a single
    /// count animate several. Proves the value is a first-class thing that flows,
    /// not a private computation.
    #[test]
    fn one_value_fans_out_to_two_channels() {
        let mut g = Graph::new();
        let grid = g.add_node("motion.drive.test.grid");
        let val = g.add_node("motion.drive.test.val"); // emits 3.0
        let drive_x = g.add_node("motion.drive");
        let drive_r = g.add_node("motion.drive");
        // grid → drive_x.in → drive_r.in ; val → both drives' value port.
        for (from, to) in [((grid, 0), (drive_x, 0)), ((drive_x, 0), (drive_r, 0))] {
            g.connect(Edge {
                from,
                to,
                delayed: false,
            })
            .unwrap();
        }
        for d in [drive_x, drive_r] {
            g.connect(Edge {
                from: (val, 0),
                to: (d, 1),
                delayed: false,
            })
            .unwrap();
        }
        g.set_param(drive_x, "channel", 0.0); // X += 3
        g.set_param(drive_r, "channel", 2.0); // Rotation += 3
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, drive_r, 0.0).unwrap();
        let s = out[0].as_stream();
        match s.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [3.0, 0.0], "X driven by the value"),
            _ => panic!("P"),
        }
        match s.get("rot").unwrap() {
            Column::Scalar(v) => assert_eq!(v[0], 3.0, "Rotation driven by the SAME value"),
            _ => panic!("rot"),
        }
    }

    /// FALSIFICATION: with the value input UNCONNECTED (empty value field), the
    /// drive is a no-op — the channel passes through untouched. A drive that
    /// invented a value would move the grid off an empty input.
    #[test]
    fn an_unconnected_value_leaves_the_channel_untouched() {
        let mut g = Graph::new();
        let grid = g.add_node("motion.drive.test.grid");
        let drive = g.add_node("motion.drive");
        g.connect(Edge {
            from: (grid, 0),
            to: (drive, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, drive, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 0.0], [0.0, 0.0]], "no value → no move"),
            _ => panic!("P"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
    /// **Opacity is a channel** (doc 51): the drive writes the ALPHA of the tint, so a particle
    /// can FADE — which is what "fades away" means, and what the library could not do at all.
    ///
    /// An uncoloured stream starts from opaque white, so driving the opacity of a stream nobody
    /// tinted does exactly what it says instead of silently doing nothing.
    #[test]
    fn the_opacity_channel_fades_the_tint_and_starts_from_opaque_white() {
        let plain = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0]; 2]));
        let out = channel::drive_channel(
            &plain,
            channel::CH_OPACITY,
            &[0.25, 0.75],
            1.0,
            Combine::Set,
        );
        match out.get("tint") {
            Some(Column::Vec4(v)) => {
                assert_eq!(v[0], [1.0, 1.0, 1.0, 0.25], "white, a quarter opaque");
                assert_eq!(v[1][3], 0.75);
            }
            _ => panic!("the opacity drive minted a tint"),
        }

        // Multiply against an existing colour: the hue survives, the alpha bleeds.
        let red = Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 0.0]]))
            .with("tint", Column::Vec4(vec![[1.0, 0.0, 0.0, 1.0]]));
        let faded =
            channel::drive_channel(&red, channel::CH_OPACITY, &[0.5], 1.0, Combine::Multiply);
        match faded.get("tint") {
            Some(Column::Vec4(v)) => assert_eq!(v[0], [1.0, 0.0, 0.0, 0.5]),
            _ => panic!("tint"),
        }

        // An alpha the renderer cannot use is not a brighter particle — it is a bug. Clamped.
        let over =
            channel::drive_channel(&plain, channel::CH_OPACITY, &[4.0, -2.0], 1.0, Combine::Set);
        match over.get("tint") {
            Some(Column::Vec4(v)) => assert_eq!((v[0][3], v[1][3]), (1.0, 0.0)),
            _ => panic!("tint"),
        }
    }
}
