#![forbid(unsafe_code)]
//! `value.mix` — the value-domain SOFT BLEND: crossfade between two **value**
//! fields by a factor, `mix = a + t·(b − a)` (Motion Nodes M2, the value domain —
//! doc 12/70). It completes the COMBINE trilogy the value vocabulary converged on:
//! `value.math` does arithmetic (`+ − × ÷ min max`), `value.switch` does a HARD
//! select (pick `a` OR `b`), and `value.mix` does the SOFT blend between them —
//! the crossfader every mature graph ships: Blender's **Mix** (Float), Nuke's
//! **Merge(mix)**, TouchDesigner's **Cross CHOP**, the fundamental "combine two
//! behaviours" node.
//!
//! **The factor is a VALUE, not (only) a param** — so an LFO, a `value.noise`, or
//! any field can DRIVE the crossfade and animate it (the domain philosophy
//! `value.switch` set: controls are values). But a bare node still wants a knob,
//! so `t` is a PORT with a `factor` PARAM fallback: **`t` connected overrides
//! `factor`; `t` unconnected reads `factor`** — exactly Blender's Mix, whose
//! Factor socket carries an inline default a wire overrides. The GPU kernel reads
//! the generated `HAS_t_v` presence const to make the SAME choice on the device.
//!
//! **`clamp`** (Blender's Clamp Factor, default On) holds `t` in `[0,1]` so the
//! blend stays between `a` and `b`; Off lets `t` past the ends OVERSHOOT (`t > 1`
//! past `b`) / UNDERSHOOT (`t < 0` before `a`) — an authored extreme, not a bug.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). **Per-element AND broadcast** (the
//! one `1→N` rule, doc 12): a length-1 field is HELD at every index; the output is
//! `max` of all connected input lengths (the same law `value.math`/`value.switch`
//! use). `Pure` (no clock, no state). Transcendental-free (HR-5): `+ − ×` and
//! `clamp` only. The kernel is the WGSL port of the same blend, so the node is
//! **device-resident** — it cooks on the GPU, no CPU fallback.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, SourceWindow};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_value_math::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, inputs and output (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.mix"),
    name: "value.mix",
    inputs: &[
        PortSpec { name: "a", ty: VALUE },
        PortSpec { name: "b", ty: VALUE },
        PortSpec { name: "t", ty: VALUE },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "factor",
            default: 0.5,
        },
        ParamSpec {
            name: "clamp",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The sample of value field `v` at index `i` under the `1→N` broadcast rule: a
/// length-1 field is held at every index; a length-N field is read element-wise;
/// a missing field reads as `0.0`. Mirror of `value.math`'s `field_at`.
fn field_at(v: &[f32], i: usize) -> f32 {
    match v.len() {
        0 => 0.0,
        1 => v[0], // broadcast: one value → every index (the 1→N rule)
        _ => v.get(i).copied().unwrap_or(0.0),
    }
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Crossfade `a` and `b` by the factor. `t` is the field from the port if
/// connected (`t_connected`), else the constant `factor`; `clamp` holds it in
/// `[0,1]`. Output length is `max` of the input lengths under the broadcast rule;
/// a length that is neither 1 nor the max is read leniently (element-wise, `0.0`
/// past the end).
fn blend(a: &[f32], b: &[f32], t: &[f32], t_connected: bool, factor: f32, clamp: bool) -> Vec<f32> {
    let n = a.len().max(b.len()).max(t.len());
    (0..n)
        .map(|i| {
            let mut tt = if t_connected { field_at(t, i) } else { factor };
            if clamp {
                tt = tt.clamp(0.0, 1.0);
            }
            let va = field_at(a, i);
            let vb = field_at(b, i);
            va + tt * (vb - va)
        })
        .collect()
}

/// GPU compute kernel (ADR-0126) — the WGSL port of [`blend`], **fully device-
/// resident**. `read_a_v`/`read_b_v`/`read_t_v` apply the broadcast rule per
/// dispatch; `HAS_t_v` is the generated presence const that answers *"is `t`
/// wired?"* — `select(factor, port, HAS_t_v)` makes the SAME port-overrides-param
/// choice the CPU makes. No `applicable` gate — the sequencer never falls back to
/// the CPU (the "maximize GPU" north).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        var vx_t = select(params.factor, read_t_v(i), HAS_t_v);\n\
        if (params.clamp >= 0.5) { vx_t = clamp(vx_t, 0.0, 1.0); }\n\
        let vx_a = read_a_v(i);\n\
        let vx_b = read_b_v(i);\n\
        write_v(i, vx_a + vx_t * (vx_b - vx_a));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 2,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["factor", "clamp"],
    count_law: Some(mix_count),
    variant_by_param: None,
    applicable: None,
};

/// **How wide is the output?** — `max` over every input port (the same law
/// `value.math`/`value.switch` use), written as a `max` over all ports rather than
/// named ones so it stays correct rather than merely true. An unconnected `t`
/// contributes 0 (the `factor` fallback carries no length), so a bare-factor
/// crossfade of two length-N fields is length N.
fn mix_count(c: &CountLawCtx<'_>) -> SourceWindow {
    SourceWindow::of_count(c.inputs.iter().copied().max().unwrap_or(0) as usize)
}

struct ValueMix;

impl NodeOp for ValueMix {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let factor = ctx.param("factor");
        let clamp = ctx.param("clamp") >= 0.5;
        let a = scalar_col(ctx.input(0), VALUE_COL);
        let b = scalar_col(ctx.input(1), VALUE_COL);
        let t = scalar_col(ctx.input(2), VALUE_COL);
        // `t` connected == a non-empty field on port 2 (matches `HAS_t_v` on GPU).
        let out = blend(&a, &b, &t, !t.is_empty(), factor, clamp);
        ctx.emit(Stream::new(out.len()).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueMix))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Mix",
            // Utility grey: a value→value combiner, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    // The constant blend, used when the `t` port is unconnected (a wire overrides
    // it). `[0,1]` is the crossfade range; overshoot is authored by driving `t`
    // with `clamp` off.
    ParamUiHint {
        param: "factor",
        label: "Factor",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // Hold `t` in `[0,1]` (On) or let the ends overshoot/undershoot (Off) — the
    // same Off/On enum `value.map_range`'s clamp uses.
    ParamUiHint {
        param: "clamp",
        label: "Clamp",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Off", "On"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **The bare node crossfades by the `factor` param** (t unconnected). At
    /// factor 0 it is all `a`; at 1 all `b`; at 0.5 the midpoint. A regression that
    /// ignored the factor (always 0, or always the port identity) would fail.
    #[test]
    fn the_factor_param_crossfades_when_t_is_unconnected() {
        let a = [2.0];
        let b = [10.0];
        let mix = |f: f32| blend(&a, &b, &[], false, f, true)[0];
        assert_eq!(mix(0.0), 2.0, "factor 0 = all a");
        assert_eq!(mix(1.0), 10.0, "factor 1 = all b");
        assert_eq!(mix(0.5), 6.0, "factor 0.5 = midpoint");
        assert_eq!(mix(0.25), 4.0, "quarter of the way from a to b");
    }

    /// **A connected `t` port OVERRIDES the factor** — the driver takes over. The
    /// factor here is a decoy 0.9; the port's per-element `t` is what lands, so a
    /// regression that read the param instead of the port would produce 9.2, not
    /// the port's answers.
    #[test]
    fn a_connected_t_port_overrides_the_factor() {
        let a = [0.0, 0.0, 0.0];
        let b = [100.0, 100.0, 100.0];
        let t = [0.0, 0.5, 1.0];
        let out = blend(&a, &b, &t, true, 0.9, true);
        assert_eq!(out, vec![0.0, 50.0, 100.0], "the port drives the blend");
    }

    /// **`clamp` holds `t` in `[0,1]`; Off lets it overshoot.** With clamp on,
    /// `t = 1.5` is pinned to `b`; with clamp off it extrapolates PAST `b`.
    #[test]
    fn clamp_pins_the_ends_and_off_overshoots() {
        let a = [0.0];
        let b = [10.0];
        // t = 1.5 (past b) and t = -0.5 (before a).
        assert_eq!(blend(&a, &b, &[1.5], true, 0.0, true)[0], 10.0, "clamped to b");
        assert_eq!(blend(&a, &b, &[-0.5], true, 0.0, true)[0], 0.0, "clamped to a");
        assert_eq!(
            blend(&a, &b, &[1.5], true, 0.0, false)[0],
            15.0,
            "unclamped overshoots past b"
        );
        assert_eq!(
            blend(&a, &b, &[-0.5], true, 0.0, false)[0],
            -5.0,
            "unclamped undershoots before a"
        );
    }

    /// **The `1→N` broadcast rule** (doc 12): a length-1 `a`/`b` is HELD across a
    /// length-N `t`, so a per-element factor blends between two constants. Output
    /// length is the `max` of the inputs.
    #[test]
    fn a_length_one_field_is_held_across_a_length_n_factor() {
        let a = [0.0]; // one constant, broadcast
        let b = [8.0]; // one constant, broadcast
        let t = [0.0, 0.25, 0.5, 0.75, 1.0];
        let out = blend(&a, &b, &t, true, 0.5, true);
        assert_eq!(out.len(), 5, "output is as wide as the widest input");
        assert_eq!(out, vec![0.0, 2.0, 4.0, 6.0, 8.0], "the ramp blends a→b");
    }

    /// Three DISTINCT value-source types, so the cook can feed `a`, `b`, and `t`
    /// different fields (the `OpResolver` keys on the node TYPE, so three nodes of
    /// one type would all emit the same field).
    macro_rules! src {
        ($man:ident, $ty:ident, $id:literal, $field:expr) => {
            static $man: NodeManifest = NodeManifest {
                id: NodeTypeId::of($id),
                name: $id,
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
            struct $ty;
            impl NodeOp for $ty {
                fn manifest(&self) -> &'static NodeManifest {
                    &$man
                }
                fn eval(&self, ctx: &mut EvalCtx<'_>) {
                    let f: Vec<f32> = $field;
                    ctx.emit(Stream::new(f.len()).with(VALUE_COL, Column::Scalar(f)));
                }
            }
        };
    }
    src!(SRC_A_MAN, SrcA, "value.mix.test.a", vec![0.0, 0.0, 0.0]);
    src!(SRC_B_MAN, SrcB, "value.mix.test.b", vec![10.0, 20.0, 30.0]);
    src!(SRC_T_MAN, SrcT, "value.mix.test.t", vec![0.0, 0.5, 1.0]);

    /// End-to-end through the cook: `a = [0,0,0]`, `b = [10,20,30]`, `t = [0,0.5,1]`
    /// blends to `[0, 10, 30]` — the factor reaches the output element-wise, the
    /// port overrides the (decoy) factor param, and the length is preserved.
    #[test]
    fn blends_two_fields_through_the_cook() {
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == MANIFEST.id => Some(&ValueMix),
                    t if t == SRC_A_MAN.id => Some(&SrcA),
                    t if t == SRC_B_MAN.id => Some(&SrcB),
                    t if t == SRC_T_MAN.id => Some(&SrcT),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let sa = g.add_node("value.mix.test.a");
        let sb = g.add_node("value.mix.test.b");
        let st = g.add_node("value.mix.test.t");
        let mix = g.add_node("value.mix");
        g.set_param(mix, "factor", 0.9); // a decoy — the connected `t` must win
        for (from, port) in [(sa, 0u16), (sb, 1), (st, 2)] {
            g.connect(Edge {
                from: (from, 0),
                to: (mix, port),
                delayed: false,
            })
            .unwrap();
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, mix, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.0, 10.0, 30.0], "blended per element"),
            _ => panic!("v"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
