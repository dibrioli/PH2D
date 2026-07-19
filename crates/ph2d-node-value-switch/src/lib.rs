//! `value.switch` — the value-domain ROUTER: pick one of N **value** fields by a
//! `select` value (Motion Nodes M2, the value domain — doc 12/17). This is the
//! multiplexer every mature node graph ships: TouchDesigner's **Switch CHOP**,
//! Houdini's **Switch VOP**, Nuke's **Switch** (`which`), Max's `selector~`. It is
//! what lets a value graph BRANCH — route a different source onto the same wire as
//! a selector animates — the last routing primitive the value vocabulary was
//! missing (docs 12–16 gave produce / combine / sample / compare / remap / drive).
//!
//! **`select` is a value, not a param** — so a `pulse.counter`, a `value.lfo`, or
//! any field can drive the selection and animate it. The chosen input index is
//! `clamp(round(select), 0, N-1)`; `select` unconnected reads as `0` (→ `in0`, the
//! sensible default).
//!
//! **Per-element AND broadcast (doc 12).** Because `select` is itself a field, the
//! switch is per-element by construction: element `i` reads
//! `in[round(select_i)][i]`. A length-1 `select` broadcasts (the whole grid
//! switches together — the common case); a length-N `select` picks a possibly
//! different input for each element (a Houdini-style per-point mux). Every input
//! obeys the `1→N` hold, so a length-1 source is held across the grid. The output
//! length is the `max` of all connected inputs. `Pure` (no clock, no state).
//! Transcendental-free (HR-5): `round` / `clamp` / index only.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, SourceWindow};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, inputs and output (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// The number of routed data inputs (`in0..in3`). Four covers the common mux
/// without a variable-arity port surface.
const N_INPUTS: usize = 4;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.switch"),
    name: "value.switch",
    inputs: &[
        PortSpec {
            name: "select",
            ty: VALUE,
        },
        PortSpec {
            name: "in0",
            ty: VALUE,
        },
        PortSpec {
            name: "in1",
            ty: VALUE,
        },
        PortSpec {
            name: "in2",
            ty: VALUE,
        },
        PortSpec {
            name: "in3",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// The sample of value field `v` at index `i`, applying the `1→N` broadcast rule
/// (mirror of `motion.drive`/`value.math`): a length-1 field is held at every
/// index; a length-N field is read element-wise; a missing field reads as `0.0`.
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

/// Route the `ins` fields by the `select` field. Output length is the `max` of
/// all inputs; element `i` reads `ins[clamp(round(select_i), 0, N-1)][i]` under
/// the broadcast rule.
fn switch(select: &[f32], ins: &[Vec<f32>]) -> Vec<f32> {
    let n = ins
        .iter()
        .map(|c| c.len())
        .chain(std::iter::once(select.len()))
        .max()
        .unwrap_or(0);
    let last = ins.len().saturating_sub(1) as i32;
    (0..n)
        .map(|i| {
            // round-to-nearest, then clamp into the connected input range.
            let idx = (field_at(select, i).round() as i32).clamp(0, last) as usize;
            field_at(&ins[idx], i)
        })
        .collect()
}

/// GPU compute kernel (ADR-0126) — the WGSL port of [`switch`].
///
/// **A dynamic index over a STATIC set of readers.** The generated module names
/// one accessor per bound port (`read_in0_v` … `read_in3_v`), so the routing is
/// a branch over four constants rather than an indexed lookup — which is what a
/// GPU wants anyway, and which is why `N_INPUTS` being a fixed 4 is a
/// convenience here rather than a limitation to route around.
///
/// **The clamp bound is 3, unconditionally, and that is the CPU's rule**: `eval`
/// always builds `N_INPUTS` vectors regardless of what is connected, so `last`
/// is always `3` and a disconnected input is the empty field. Clamping to *the
/// number of connected inputs* instead would look tidier and would be a
/// different function — `select = 2` with only `in0` wired reads `0.0` on the
/// CPU, not `in0`.
///
/// Every port is [`ColumnAccess::ReadBroadcast`], including `select`: a length-1
/// selector switches the whole grid together (the common case) and a length-N
/// one picks per element, and both are the same compiled module — the broadcast
/// is a uniform bit, not a pipeline variant.
///
/// `vs_round` is round-half-AWAY-from-zero to match Rust's `f32::round`.
/// `select` picks a BRANCH, so at `select = 0.5` a half-even disagreement routes
/// a *different input* — the loudest possible way for a rounding convention to
/// be wrong.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vs_i = clamp(i32(vs_round(read_select_v(i))), 0, 3);\n\
        var vs_r = read_in0_v(i);\n\
        if (vs_i == 1) {\n\
        \x20   vs_r = read_in1_v(i);\n\
        } else if (vs_i == 2) {\n\
        \x20   vs_r = read_in2_v(i);\n\
        } else if (vs_i == 3) {\n\
        \x20   vs_r = read_in3_v(i);\n\
        }\n\
        write_v(i, vs_r);\n",
    wgsl_lib: "\
        fn vs_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
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
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 3,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 4,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &[],
    count_law: Some(switch_count),
    variant_by_param: None,
    applicable: None,
};

/// **How wide is the output?** — the `max` over every port, which is the same
/// expression `switch` computes, **`select` included**. Leaving the selector out
/// would be the tempting simplification and it is wrong: an animated length-N
/// selector against length-1 sources is exactly the per-element mux this node
/// advertises, and the output has to be as long as the selection.
fn switch_count(ctx: &CountLawCtx<'_>) -> SourceWindow {
    SourceWindow::of_count(ctx.inputs.iter().copied().max().unwrap_or(0) as usize)
}

struct ValueSwitch;

impl NodeOp for ValueSwitch {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let select = scalar_col(ctx.input(0), VALUE_COL);
        let ins: Vec<Vec<f32>> = (0..N_INPUTS)
            .map(|k| scalar_col(ctx.input(k + 1), VALUE_COL))
            .collect();
        let out = switch(&select, &ins);
        ctx.emit(Stream::new(out.len()).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueSwitch))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Switch",
            // Utility grey: a value→value router, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    // No params — the selection is a value input, so it can be animated.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    /// A length-1 `select` broadcasts: the WHOLE field switches together. `0`
    /// picks `in0`, `1` picks `in1` — the common global-switch case.
    #[test]
    fn a_global_select_routes_the_whole_field() {
        let ins = vec![vec![10.0, 11.0], vec![20.0, 21.0], vec![], vec![]];
        assert_eq!(switch(&[0.0], &ins), vec![10.0, 11.0], "select 0 → in0");
        assert_eq!(switch(&[1.0], &ins), vec![20.0, 21.0], "select 1 → in1");
    }

    /// `select` rounds to the nearest input index (0.4 → 0, 0.6 → 1) and clamps
    /// past the connected range (a huge/negative select never indexes out).
    #[test]
    fn select_rounds_to_nearest_and_clamps() {
        let ins = vec![vec![10.0], vec![20.0], vec![], vec![]];
        assert_eq!(switch(&[0.4], &ins), vec![10.0], "0.4 rounds down to in0");
        assert_eq!(switch(&[0.6], &ins), vec![20.0], "0.6 rounds up to in1");
        // clamp: N_INPUTS is 4, so index 3 is the top; 9.0 clamps to in3.
        let four = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0]];
        assert_eq!(
            switch(&[9.0], &four),
            vec![4.0],
            "9 clamps to the top input"
        );
        assert_eq!(switch(&[-5.0], &four), vec![1.0], "negative clamps to in0");
    }

    /// FALSIFICATION of per-element routing: a length-N `select` picks a possibly
    /// DIFFERENT input for each element — the Houdini per-point mux. Element 0
    /// reads in0, element 1 reads in1.
    #[test]
    fn a_per_element_select_routes_each_element_independently() {
        let ins = vec![vec![10.0, 11.0], vec![20.0, 21.0], vec![], vec![]];
        // select [0, 1] → element 0 from in0 (10), element 1 from in1 (21).
        assert_eq!(
            switch(&[0.0, 1.0], &ins),
            vec![10.0, 21.0],
            "each element its own input"
        );
    }

    /// A length-1 source is HELD (broadcast) across a longer field — the `1→N`
    /// rule reaches the routed inputs too.
    #[test]
    fn a_length_one_source_broadcasts_through_the_switch() {
        // in1 is a single global constant; select 1 over a 3-long select → held.
        let ins = vec![vec![], vec![7.0], vec![], vec![]];
        assert_eq!(switch(&[1.0, 1.0, 1.0], &ins), vec![7.0, 7.0, 7.0]);
    }

    /// End-to-end through the cook: two source fields and an animated select
    /// (its own value node) route through the registry.
    #[test]
    fn routes_two_sources_through_the_cook() {
        use ph2d_nodegraph::cook::Cook;
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("value.switch.test.src"),
            name: "value.switch.test.src",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: VALUE,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[ph2d_nodegraph::node::ParamSpec {
                name: "v",
                default: 0.0,
            }],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                let v = ctx.param("v");
                ctx.emit(Stream::new(1).with(VALUE_COL, Column::Scalar(vec![v])));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&ValueSwitch),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let sel = g.add_node("value.switch.test.src");
        let a = g.add_node("value.switch.test.src");
        let b = g.add_node("value.switch.test.src");
        let sw = g.add_node("value.switch");
        g.set_param(sel, "v", 1.0); // select in1
        g.set_param(a, "v", 100.0);
        g.set_param(b, "v", 200.0);
        for (from, port) in [(sel, 0), (a, 1), (b, 2)] {
            g.connect(Edge {
                from: (from, 0),
                to: (sw, port),
                delayed: false,
            })
            .unwrap();
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, sw, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![200.0], "select 1 routed in1 (b)"),
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
