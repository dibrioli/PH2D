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

/// GPU compute kernel (ADR-0126) — the value domain's WRITE side on the device,
/// and the first consumer of [`ColumnAccess::ReadBroadcast`] in a chain that
/// actually reaches the screen.
///
/// **X/Y only, and the reason is structural rather than lazy.** `drive_channel`
/// writes a DIFFERENT column per channel — `P` for X/Y, `rot` for Rotation,
/// `size` for Size, `tint` for Opacity — and materialises the target from its
/// identity when the stream lacks it. `GpuKernel::bindings` is `&'static`, so a
/// kernel claiming every channel would have to bind all four, and then a drive on
/// X would emit `rot`/`size`/`tint` **that the CPU's output does not carry**.
/// That is not an ε: it is a different stream shape, and it would surface far
/// away as some later node branching on a column that should not exist
/// (the discipline `a_value_node_emits_a_bare_stream_not_the_instance_base` pins).
///
/// So the kernel binds exactly `{P, falloff}` and `applicable` claims the two
/// channels that write `P` — the same split, for the same reason, that
/// `motion.oscillator` already makes. Rotation/Size/Opacity recede to the CPU at
/// plan time: an explicit boundary, never a wrong answer. Covering them needs
/// **per-param bindings**, which is an engine feature and not a kernel edit.
///
/// ⚠️ **One mutation survives by design:** dropping the `clamp` on `falloff`
/// changes nothing measurable, because the CPU clamps identically and no node in
/// the library emits a falloff outside `[0,1]`. It is a defensive MIRROR, kept so
/// the two sides stay line-for-line comparable; the day something writes an
/// out-of-range falloff, both clamp and parity still holds.
///
/// `P` is `ReadWrite` and not `ReadWriteExisting` because the CPU materialises it
/// from `[0,0]` when absent (`base_vec2`) — dropping the write instead would make
/// the drive silently do nothing on a stream without `P`.
///
/// The value port BROADCASTS: length 1 is one number held across the field (the
/// `1 => vals[0]` arm of `value_at`), length N is per-element. `drive_round` is
/// round-half-away-from-zero to match Rust's `f32::round` — `channel` and `mode`
/// both pick BRANCHES ([[feedback_cpu_gpu_rounding_conventions_diverge]]).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_comp = i32(drive_round(params.channel));\n\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_p = read_in_P(i);\n\
        var dr_cur = dr_p.x;\n\
        if (dr_comp == 1) { dr_cur = dr_p.y; }\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        var dr_driven = dr_cur + dr_v;\n\
        if (dr_mode == 1) { dr_driven = dr_v; }\n\
        if (dr_mode == 2) { dr_driven = dr_cur * dr_v; }\n\
        // Lerp toward the original by falloff: 0 leaves the channel untouched.\n\
        // The clamp MIRRORS the CPU's; no node writes falloff outside [0,1]\n\
        // today, so it is defensive on both sides rather than load-bearing.\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_out = dr_cur + (dr_driven - dr_cur) * dr_f;\n\
        var dr_next = dr_p;\n\
        if (dr_comp == 1) { dr_next.y = dr_out; } else { dr_next.x = dr_out; }\n\
        write_P(i, dr_next);\n",
    wgsl_lib: "\
        fn drive_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
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
    ],
    params: &["channel", "scale", "mode"],
    count_law: None,
    applicable: Some(|param| {
        // Same rounding the CPU `eval` applies. Only X (0) / Y (1) write `P`.
        let channel = param("channel").round();
        channel == 0.0 || channel == 1.0
    }),
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
