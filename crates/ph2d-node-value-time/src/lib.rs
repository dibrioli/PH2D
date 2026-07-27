//! `value.time` — the value-domain PRODUCER of the raw animated clock: the
//! playhead as a plain value (Motion Nodes M2, the value domain — doc 12/80).
//! Every other value producer is spatial or random — `value.instance_field`
//! (Index/Ramp/Random), `value.noise`, `value.pattern` — and none of them is the
//! **clock**. `value.lfo`/`value.noise` bring time in but bake a WAVEFORM around
//! it; this emits the clock UN-shaped, so a value graph can build its own
//! function of time. It is Houdini's `@Time`/`$T`, TouchDesigner's Timer/Beat
//! CHOP, the `time` node of every node editor.
//!
//! **Monotonic, not periodic — that is the whole point.** `value.lfo(Saw)` folds
//! the clock back every period (`phase − floor(phase)`); this one keeps climbing:
//! `t · rate + offset`. That distinction is what it is FOR — endless rotation, an
//! ever-scrolling offset, an accumulating drift — and it is what makes it the
//! natural partner of `value.wrap`: `time → wrap(Repeat)` is a controllable
//! sawtooth clock, `time → wrap(Mirror)` a triangle one, each folded exactly where
//! you want, while the raw `time` still climbs for whatever wants it un-folded.
//!
//! **Cardinality follows the geometry** (the `value.lfo` pattern): the optional
//! `in` port is read for its **count only** — connected → a length-N field with a
//! per-instance `stagger` (a travelling ramp across the grid), **unconnected → a
//! length-1 global clock** held across every instance by `motion.drive`'s
//! broadcast rule (the common case: *the* clock). Nothing from the input rides
//! through — this mints a fresh value.
//!
//! Reads the playhead, holds no state → `Effect::Temporal` (pull-side, like the
//! LFO). Transcendental-free (HR-5): a multiply-add, bit-exact on the device.
//!
//! `value_i = playhead · rate + offset + i · stagger`.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, SourceWindow};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The instance stream type — read for its count only (the optional `in` port).
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of the sibling value nodes; kept local so this stays a leaf drop-crate
/// — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value output column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.time"),
    name: "value.time",
    // Optional: connected → count N + per-instance stagger; unconnected → one
    // global clock. Read for its count only; never passed through.
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Reads the playhead → pull-side; HR-5-exempt for the clock (the arithmetic
    // is nonetheless a plain multiply-add, bit-exact cross-platform).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // Clock speed — units of value per second. `1` = the raw playhead in
        // seconds; `2` runs twice as fast; negative runs backwards.
        ParamSpec {
            name: "rate",
            default: 1.0,
        },
        // A constant added to the clock — where the value starts at `t = 0`.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // Per-instance offset → a travelling ramp across the field (needs a
        // connected `in` for N > 1; 0 → every instance reads the same clock).
        ParamSpec {
            name: "stagger",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`ValueTime::eval`], **fully
/// device-resident**. VALUE out (mints a fresh `v`, does NOT ride the input
/// through — the sequencer derives that from the manifest, port 0 in-type vs
/// out-type differ, exactly like `value.lfo`). The clock is `params.playhead`,
/// the same uniform the LFO reads; the arithmetic is a bare multiply-add, so the
/// device result is bit-comparable to the CPU (the only divergence is an FMA the
/// driver may fuse, ε below the parity budget). No `applicable` — no CPU fallback.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vt_v = params.playhead * params.rate + params.offset\n\
        \x20   + f32(i) * params.stagger;\n\
        write_v(i, vt_v);\n",
    wgsl_lib: "",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["rate", "offset", "stagger"],
    count_law: Some(time_count),
    variant_by_param: None,
    applicable: None,
};

/// **How wide is the field?** — the same expression `eval` uses. Connected, it is
/// one clock per instance (a travelling ramp with `stagger`); **unconnected it is
/// ONE global clock**, held across every instance by `motion.drive`'s broadcast
/// rule. The engine's default law — "as wide as port 0" — gets the connected case
/// right and the unconnected one silently wrong: an empty port is `0`, a
/// zero-count stage is SKIPPED, and the node would be unreachable on the device
/// the moment something consumed the global clock (the `value.lfo` lesson).
fn time_count(c: &CountLawCtx<'_>) -> SourceWindow {
    SourceWindow::of_count(c.inputs.first().copied().unwrap_or(0).max(1) as usize)
}

struct ValueTime;

impl NodeOp for ValueTime {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let rate = ctx.param("rate");
        let offset = ctx.param("offset");
        let stagger = ctx.param("stagger");
        let t = ctx.playhead() as f32;
        // Cardinality follows the geometry: N from the (optional) input, else the
        // length-1 global clock (broadcast by `motion.drive`).
        let n = ctx.input(0).count().max(1);
        let value: Vec<f32> = (0..n)
            .map(|i| t * rate + offset + i as f32 * stagger)
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(value)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueTime))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Time",
            // Utility grey: a value SOURCE, plumbing (not a visible transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rate",
        label: "Rate",
        min: -8.0,
        max: 8.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "stagger",
        label: "Stagger",
        min: -2.0,
        max: 2.0,
        step: 0.02,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    // A grid source: `n` instances at the origin, so `value.time` can read a count.
    static GRID_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.time.test.grid"),
        name: "value.time.test.grid",
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
            ctx.emit(Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == GRID_MAN.id => Some(&Grid),
                t if t == MANIFEST.id => Some(&ValueTime),
                _ => None,
            }
        }
    }

    fn vals(s: &Stream) -> Vec<f32> {
        match s.get(VALUE_COL).unwrap() {
            Column::Scalar(v) => v.clone(),
            _ => panic!("v"),
        }
    }

    /// Cook `value.time` at `playhead`; `connect_grid` decides whether it reads a
    /// count from a source (length-N) or stands alone (length-1 global).
    fn time_at(
        playhead: f64,
        connect_grid: bool,
        setup: impl FnOnce(&mut Graph, NodeId),
    ) -> Vec<f32> {
        let mut g = Graph::new();
        let time = g.add_node("value.time");
        if connect_grid {
            let grid = g.add_node("value.time.test.grid");
            g.connect(Edge {
                from: (grid, 0),
                to: (time, 0),
                delayed: false,
            })
            .unwrap();
        }
        setup(&mut g, time);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, time, playhead).unwrap();
        vals(out[0].as_stream())
    }

    /// UNCONNECTED input → one global clock (length-1). This is the field that
    /// `motion.drive` broadcasts across every instance — *the* clock (doc 12).
    #[test]
    fn an_unconnected_time_emits_a_single_global_clock() {
        // Default rate 1 → the value IS the playhead.
        assert_eq!(time_at(0.0, false, |_, _| {}), vec![0.0], "t=0 -> 0");
        assert_eq!(time_at(2.5, false, |_, _| {}), vec![2.5], "t=2.5 -> 2.5");
        // Rate scales the clock; offset shifts its start.
        let v = time_at(2.0, false, |g, t| {
            g.set_param(t, "rate", 3.0);
            g.set_param(t, "offset", 1.0);
        });
        assert_eq!(v, vec![7.0], "2*3 + 1 = 7");
    }

    /// **Monotonic, not periodic** — the clock keeps CLIMBING past any period,
    /// where `value.lfo(Saw)` would fold back. Falsifiable: a wrapped clock would
    /// read the same at `t` and `t + 1/rate`.
    #[test]
    fn the_clock_climbs_and_never_folds() {
        let at = |t: f64| time_at(t, false, |g, node| g.set_param(node, "rate", 1.0))[0];
        let a = at(0.5);
        let b = at(10.5); // ten "periods" later — a Saw would read the same
        assert!(
            b > a + 9.5,
            "the clock climbed by ~10, did not fold: {a} -> {b}"
        );
    }

    /// CONNECTED input → the field's length follows the geometry (N=3), and
    /// `stagger` sends a travelling ramp across it: at t=0 with stagger 0.5,
    /// instance i reads `i·0.5`. As t advances the whole ramp shifts up.
    #[test]
    fn a_connected_time_emits_a_travelling_ramp() {
        let v = time_at(0.0, true, |g, t| g.set_param(t, "stagger", 0.5));
        assert_eq!(v, vec![0.0, 0.5, 1.0], "spatial ramp at t=0");
        // t=2 shifts every element up by 2 (rate 1) — the ramp travels.
        let v = time_at(2.0, true, |g, t| g.set_param(t, "stagger", 0.5));
        assert_eq!(v, vec![2.0, 2.5, 3.0], "the ramp travelled up by t");
    }

    /// A negative rate runs the clock BACKWARDS — the value decreases as the
    /// playhead advances (reverse scroll / counter-rotation).
    #[test]
    fn a_negative_rate_runs_backwards() {
        let v = time_at(3.0, false, |g, t| g.set_param(t, "rate", -2.0));
        assert_eq!(v, vec![-6.0], "3*(-2) = -6");
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
