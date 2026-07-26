#![forbid(unsafe_code)]
//! `value.pattern` — the value-domain STEP SEQUENCER: author an explicit list of
//! values that repeats across the instances by index (Motion Nodes M2, the value
//! domain — doc 12/78). Where `instance_field` Random/Ramp and `value.noise`
//! GENERATE a field procedurally, this one lets you TYPE the field — `[v₀, v₁,
//! …]` assigned to instance `0, 1, …` and cycled — the explicit-pattern producer
//! the procedural ones never gave. The step sequencer of a drum machine, the
//! Array of Cavalry, a Houdini detail array.
//!
//! **A PRODUCER** (like `instance_field`): its input is read for its COUNT only
//! (never passed through), and it writes a fresh `v` where `out[i] = pattern[i mod
//! steps]`. The pattern is up to **8 slots** (`v0…v7`) with `steps` saying how
//! many cycle — the common sequencer length, and the reason it rides the kernel's
//! own uniform (params, no device array channel): the values ARE the uniform, so
//! it is **device-resident** with no new infrastructure.
//!
//! - **`steps`** — how many slots cycle (`1…8`); `steps = 4` repeats `v0,v1,v2,v3`.
//! - **`v0…v7`** — the values, assigned to consecutive instances and cycled.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). `Pure` (no clock, no state); the
//! output length matches the input's count.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of the sibling value nodes; kept local so this stays a leaf drop-crate
/// — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The INPUT type — the instance geometry stream (`Vec2`), read for its COUNT only
/// (the same port `instance_field` reads the grid on). A producer keys on the
/// count, so it takes the geometry, not a value.
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The value column, out (this is a producer — it writes, never reads `v`).
const VALUE_COL: &str = "v";

/// The number of authored slots. The common step-sequencer length; the uniform
/// holds far more, so the cap is panel readability — a longer arbitrary pattern
/// wants a device ARRAY channel (a text-authored list), deferred.
const SLOTS: usize = 8;

/// The value for instance `i` given the authored `steps` (clamped to `[1, SLOTS]`,
/// so the cycle never indexes past the slots) and the `SLOTS` values. `out[i] =
/// vals[i mod steps]`.
fn pattern_value(i: usize, steps: usize, vals: &[f32; SLOTS]) -> f32 {
    let steps = steps.clamp(1, SLOTS);
    vals[i % steps]
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.pattern"),
    name: "value.pattern",
    // Read for its COUNT only (never passed through), like `instance_field` — the
    // instance geometry stream, so the grid connects straight in.
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "steps",
            default: 2.0,
        },
        ParamSpec {
            name: "v0",
            default: 0.0,
        },
        ParamSpec {
            name: "v1",
            default: 1.0,
        },
        ParamSpec {
            name: "v2",
            default: 0.0,
        },
        ParamSpec {
            name: "v3",
            default: 0.0,
        },
        ParamSpec {
            name: "v4",
            default: 0.0,
        },
        ParamSpec {
            name: "v5",
            default: 0.0,
        },
        ParamSpec {
            name: "v6",
            default: 0.0,
        },
        ParamSpec {
            name: "v7",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`pattern_value`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to
/// the CPU. VALUE out; the binding is `Write` (a producer — it never reads the
/// input `v`, only its count), so no `in_v` is declared for naga to strip. The
/// `steps` clamp keeps `vp_idx` inside the eight slots; the `switch` selects.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vp_steps = clamp(i32(vp_round(params.steps)), 1, 8);\n\
        let vp_idx = i32(i) % vp_steps;\n\
        var vp_v: f32;\n\
        switch (vp_idx) {\n\
            case 1: { vp_v = params.v1; }\n\
            case 2: { vp_v = params.v2; }\n\
            case 3: { vp_v = params.v3; }\n\
            case 4: { vp_v = params.v4; }\n\
            case 5: { vp_v = params.v5; }\n\
            case 6: { vp_v = params.v6; }\n\
            case 7: { vp_v = params.v7; }\n\
            default: { vp_v = params.v0; }\n\
        }\n\
        write_v(i, vp_v);\n",
    wgsl_lib: "\
        fn vp_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["steps", "v0", "v1", "v2", "v3", "v4", "v5", "v6", "v7"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValuePattern;

impl NodeOp for ValuePattern {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let steps = ctx.param("steps").round().max(1.0) as usize;
        let vals: [f32; SLOTS] = [
            ctx.param("v0"),
            ctx.param("v1"),
            ctx.param("v2"),
            ctx.param("v3"),
            ctx.param("v4"),
            ctx.param("v5"),
            ctx.param("v6"),
            ctx.param("v7"),
        ];
        // The input is read for its COUNT only (like `instance_field`); the output
        // is a fresh field of that length.
        let n = ctx.input(0).count();
        let out: Vec<f32> = (0..n).map(|i| pattern_value(i, steps, &vals)).collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValuePattern))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Pattern",
            // Source green: a value producer (authors a field), not a transformer.
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// A value slider for one slot, `0..1` (the value-domain normalised convention;
/// compose a `value.map_range` for another range).
const fn slot(param: &'static str, label: &'static str) -> ParamUiHint {
    ParamUiHint {
        param,
        label,
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    }
}

static PARAM_HINTS: &[ParamUiHint] = &[
    // How many slots cycle. The extra slots stay on the panel but are ignored
    // above `steps`; the doc says so.
    ParamUiHint {
        param: "steps",
        label: "Steps",
        min: 1.0,
        max: SLOTS as f32,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    slot("v0", "V0"),
    slot("v1", "V1"),
    slot("v2", "V2"),
    slot("v3", "V3"),
    slot("v4", "V4"),
    slot("v5", "V5"),
    slot("v6", "V6"),
    slot("v7", "V7"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **The pattern is assigned by index and cycles.** `steps = 3` over 7
    /// instances repeats `v0,v1,v2` — `[a,b,c,a,b,c,a]`.
    #[test]
    fn the_pattern_cycles_by_index() {
        let vals = [10.0, 20.0, 30.0, 40.0, 0.0, 0.0, 0.0, 0.0];
        let got: Vec<f32> = (0..7).map(|i| pattern_value(i, 3, &vals)).collect();
        assert_eq!(got, vec![10.0, 20.0, 30.0, 10.0, 20.0, 30.0, 10.0]);
    }

    /// **`steps` is clamped to `[1, SLOTS]`** — `0` collapses to a constant `v0`,
    /// and a value past the slots never indexes out of bounds.
    #[test]
    fn steps_is_clamped_to_the_slots() {
        let vals = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        // steps 0 -> clamps to 1 -> every element is v0.
        assert!((0..5).all(|i| pattern_value(i, 0, &vals) == 1.0), "0 steps is constant v0");
        // steps beyond SLOTS clamps to SLOTS (uses all eight, never out of bounds).
        assert_eq!(pattern_value(8, 20, &vals), 1.0, "index 8 wraps to slot 0");
        assert_eq!(pattern_value(7, 20, &vals), 8.0, "the last slot is used");
    }

    /// **Every one of the eight slots is reachable** — `steps = 8` reads `v0…v7`
    /// in order, so no slot is a dead param.
    #[test]
    fn all_eight_slots_are_reachable() {
        let vals = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let got: Vec<f32> = (0..SLOTS).map(|i| pattern_value(i, SLOTS, &vals)).collect();
        assert_eq!(got, vals.to_vec(), "all eight slots read in order");
    }

    /// The grid-like source: a count-only input, so `value.pattern` can produce a
    /// field of length N through a real cook (it reads the count, never a `v`).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.pattern.test.src"),
        name: "value.pattern.test.src",
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
    struct Src(usize);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            // A field of N zeros — only its LENGTH matters to `value.pattern`.
            ctx.emit(Stream::new(self.0).with(VALUE_COL, Column::Scalar(vec![0.0; self.0])));
        }
    }

    /// End-to-end through the cook: a length-5 input with `steps = 2`,
    /// `v0 = 0, v1 = 1` produces `[0, 1, 0, 1, 0]` — the pattern authored by
    /// param, keyed on the input's count.
    #[test]
    fn produces_the_pattern_through_the_cook() {
        struct Ops(usize);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => Some(Box::leak(Box::new(Src(self.0))) as &dyn NodeOp),
                    t if t == MANIFEST.id => Some(&ValuePattern),
                    _ => None,
                }
            }
        }
        let ops = Ops(5);
        let mut g = Graph::new();
        let src = g.add_node("value.pattern.test.src");
        let vp = g.add_node("value.pattern");
        g.set_param(vp, "steps", 2.0);
        g.set_param(vp, "v0", 0.0);
        g.set_param(vp, "v1", 1.0);
        g.connect(Edge {
            from: (src, 0),
            to: (vp, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vp, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v, &vec![0.0, 1.0, 0.0, 1.0, 0.0], "alternating pattern, length 5");
            }
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
