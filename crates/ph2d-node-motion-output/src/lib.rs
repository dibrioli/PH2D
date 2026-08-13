#![forbid(unsafe_code)]
//! `motion.output` — the Motion **render sink**: the terminal node whose incoming
//! stream is what gets drawn. It is a pure pass-through (emits its input stream
//! unchanged), so cooking it lowers whatever feeds it to instances. The shell
//! bridge auto-selects the Output node in the graph as the cook's sink, so the
//! rendered result *follows the graph* — wire a chain into an Output node and it
//! shows on canvas (a Material-Output / render node, not a hidden toggle).
//!
//! One input, one output (the pass-through) so the cook lowers it like any node;
//! conventionally terminal (you don't chain past it). An **unconnected** Output
//! emits an empty stream → nothing renders. Pure.
//!
//! **`blend` — how this sink composites** (doc 89, folha 17). Niagara puts blend on
//! the Sprite Renderer's material, Cavalry on the layer/shader, AE/Stardust on the
//! layer: every reference makes it a property of the RENDERER, not of a particle,
//! and this node IS the renderer. The tag is `ph2d_ecs::BlendMode::tag()` (0 Mix ·
//! 1 Add · 2 Subtract · 3 Multiply · 4 Screen · 5 PremultAlpha) — the SAME encoding
//! a sprite's blend rides in, so the renderer keys its draw runs on it with no ABI
//! cost (`RenderInstance::pack_blend_bits`, `flip_uv` bits 5-7).
//!
//! ⚠️ **The param is read by the LOWERING, not by `eval`.** On the device this node
//! is `GpuKernel::PASSTHROUGH` — the sequencer emits no pass for it — so a column
//! written here would never reach the device lowering. The tag travels as an
//! argument of both lowerings instead (`ph2d_eval_motion::sink_blend_tag` is the
//! one door), and `Mix` (the default) is `flip_uv = 0`, i.e. byte-identical to
//! every frame this app has ever drawn.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The canonical type name the shell bridge scans for to pick the render sink.
pub const TYPE_NAME: &str = "motion.output";

/// The blend param's name. It is the SAME string `ph2d_eval_motion` looks up when
/// it lowers a sink (`SINK_BLEND_PARAM`) — the two crates are leaves and neither
/// may depend on the other, so a gate in the shell (which sees both) pins that
/// they agree rather than a shared symbol nobody could host.
pub const BLEND_PARAM: &str = "blend";

/// The blend modes this sink offers, in tag order — the artist-facing names for
/// `ph2d_ecs::BlendMode::ALL`. Kept as one list so the UI hint and any future
/// reader share it (the labels are what a segmented row paints).
pub const BLEND_LABELS: [&str; 6] = [
    "Normal",
    "Add",
    "Subtract",
    "Multiply",
    "Screen",
    "Premultiplied",
];

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.output"),
    name: "motion.output",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        // The `BlendMode` tag this sink draws with. Default 0 = `Mix`, which is
        // the `flip_uv: 0` both lowerings hardcoded before this param existed.
        name: BLEND_PARAM,
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// The blend row. `Enum` (not a slider) because a tag is a NAME, not a quantity —
/// a slider between `Subtract` and `Multiply` has no midpoint to mean anything.
static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: BLEND_PARAM,
    label: "Blend",
    min: 0.0,
    max: (BLEND_LABELS.len() - 1) as f32,
    step: 1.0,
    widget: ParamWidget::Enum {
        labels: &BLEND_LABELS,
    },
}];

struct MotionOutput;

impl NodeOp for MotionOutput {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Pass the input stream through unchanged (count + every column). An
        // absent input → an empty stream (nothing to render).
        let out = {
            let input = ctx.input(0);
            let mut out = Stream::new(input.count());
            for (name, col) in input.columns() {
                out.set(name.clone(), col.clone());
            }
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionOutput))?;
    // M1.R1 — UI metadata (the render sink → red Output, circle terminal).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Output",
            category: ph2d_node_registry::NodeUiCategory::Output,
            silhouette: ph2d_node_registry::NodeSilhouette::Circle,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // GPU/M5 Fase 1 (ADR-0126): the render sink is a pure copy, so on the GPU
    // it is the PASSTHROUGH kernel — the sequencer emits no pass and the
    // upstream stream flows straight into the lowering.
    reg.register_gpu_kernel(MANIFEST.id, ph2d_nodegraph::gpu::GpuKernel::PASSTHROUGH);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.output.test.src"),
        name: "motion.output.test.src",
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
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]))
                    .with("rot", Column::Scalar(vec![0.5, 0.5])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionOutput),
                _ => None,
            }
        }
    }

    #[test]
    fn output_passes_its_input_through_unchanged() {
        let mut g = Graph::new();
        let src = g.add_node("motion.output.test.src");
        let out = g.add_node("motion.output");
        g.connect(Edge {
            from: (src, 0),
            to: (out, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let cooked = cook.cook(&g, &Ops, out, 0.0).unwrap();
        let stream = cooked[0].as_stream();
        assert_eq!(stream.count(), 2);
        match stream.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[1.0, 2.0], [3.0, 4.0]]),
            _ => panic!("P"),
        }
        match stream.get("rot").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.5, 0.5]),
            _ => panic!("rot"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
