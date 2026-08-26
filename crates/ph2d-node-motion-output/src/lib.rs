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
//! ⚠️ **The params are read by the LOWERING, not by `eval`.** On the device this
//! node is `GpuKernel::PASSTHROUGH` — the sequencer emits no pass for it — so a
//! column written here would never reach the device lowering. They travel as an
//! argument of both lowerings instead (`ph2d_eval_motion::sink_style` is the one
//! door), and the DEFAULTS are exactly what the two lowerings hardcoded before
//! any of them existed, i.e. byte-identical to every frame this app has drawn.
//!
//! **`pivot` · `filter` · `sort` — o resto do estilo do sink** (doc 89, folha 17,
//! e a mesma citação decidiu os quatro):
//!
//! - **`pivot_x`/`pivot_y`** — o *Pivot Offset* do Sprite Renderer do Niagara e a
//!   âncora por-cópia da Cavalry. ⚠️ **A unidade é a FRACÇÃO do tamanho do próprio
//!   elemento**, não metros: num stream cada linha tem o seu `size`, e um pivô em
//!   metros deslocaria as peças pequenas de outra maneira que as grandes. `0` =
//!   centrado. É em torno dele que o `rot` da linha gira.
//! - **`filter`** — o sampler que Niagara põe no Material e a Cavalry na camada.
//!   Importa em **pixel-art**: sem ele o único controlo é o default do projecto.
//! - **`sort`** — o `SortMode` do Sprite Renderer do Niagara. `Texture` (o de
//!   sempre) agrupa por textura e é o que forma runs de desenho; `Stream` diz que
//!   **a ordem das linhas é a ordem de desenho**, que é o que um `motion.sort` a
//!   montante autorou e que a mídia MISTA derrotava.

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

/// O pivô do elemento, em **fracção do tamanho dele**. Dois params porque a UI
/// desta casa pinta um número por linha; a porta que os junta num `[f32; 2]` é o
/// `sink_style`.
pub const PIVOT_X_PARAM: &str = "pivot_x";
/// O irmão em `y` de [`PIVOT_X_PARAM`].
pub const PIVOT_Y_PARAM: &str = "pivot_y";

/// Quão longe do centro o pivô pode ir, em fracções do tamanho.
///
/// ⚠️ **O recurso que este número nomeia é a MOLDURA DE CULL, não a aritmética.**
/// `±1` põe o `world_pos` numa aresta a UM tamanho inteiro de distância do quad —
/// já fora dele —, e o renderer decide o que desenhar pelo `world_pos`. Mais longe
/// que isso e a peça começa a poder desaparecer por estar «fora» num sítio onde se
/// vê. Medido: a `±1` a peça e o pivô ainda se tocam.
pub const PIVOT_LIMIT: f32 = 1.0;

/// O filtro de textura deste sink.
pub const FILTER_PARAM: &str = "filter";

/// Os modos de filtro, na ordem dos tags de `ph2d_ecs::FilterMode`.
///
/// ⚠️ **Copiados à mão, porque um nó é FOLHA e não alcança o `ph2d-ecs`** — a mesma
/// situação do `BLEND_LABELS`, e a mesma cura: um gate na shell (o único sítio que
/// vê os dois) conta os tags que o `from_tag` distingue e exige esta lista do
/// mesmo tamanho. Menos rótulos ⇒ um modo inalcançável; mais ⇒ um item de menu que
/// o `sink_style` clampa de volta.
pub const FILTER_LABELS: [&str; 7] = [
    "Project",
    "Nearest",
    "Linear",
    "Nearest Mip",
    "Linear Mip",
    "Nearest Aniso",
    "Linear Aniso",
];

/// Como este sink ordena as linhas para desenhar.
pub const SORT_PARAM: &str = "sort";

/// ⚠️ **`Texture` é o de sempre, e é o RÁPIDO.** `Stream` honra a ordem das linhas
/// e paga em draw calls — a conta é o próprio pedido (ver `SinkStyle::stream_order`).
pub const SORT_LABELS: [&str; 2] = ["Texture", "Stream"];

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
    params: &[
        ParamSpec {
            // The `BlendMode` tag this sink draws with. Default 0 = `Mix`, which is
            // the `flip_uv: 0` both lowerings hardcoded before this param existed.
            name: BLEND_PARAM,
            default: 0.0,
        },
        // ⚠️ Os quatro que se seguem sao APENDIDOS, e a ordem e' o contrato: um
        // `.ph2dproj` guarda overrides por NOME, mas o painel pinta por indice, e
        // reordenar esta lista trocaria as linhas de um documento ja' gravado.
        ParamSpec {
            name: PIVOT_X_PARAM,
            default: 0.0,
        },
        ParamSpec {
            name: PIVOT_Y_PARAM,
            default: 0.0,
        },
        ParamSpec {
            // `0` = `FilterMode::Inherit` = o `sampling: 0` que os dois lowerings
            // cravavam: o sampler default do projecto.
            name: FILTER_PARAM,
            default: 0.0,
        },
        ParamSpec {
            // `0` = `Texture` = o `sub_order: 0` de sempre.
            name: SORT_PARAM,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// The blend row. `Enum` (not a slider) because a tag is a NAME, not a quantity —
/// a slider between `Subtract` and `Multiply` has no midpoint to mean anything.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: BLEND_PARAM,
        label: "Blend",
        min: 0.0,
        max: (BLEND_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &BLEND_LABELS,
        },
    },
    // ⚠️ O pivo' e' um SLIDER e nao um enum: aqui o meio-caminho quer dizer
    // alguma coisa (um pivo' a 0,25 do centro), ao contrario de um tag.
    ParamUiHint {
        param: PIVOT_X_PARAM,
        label: "Pivot X",
        min: -PIVOT_LIMIT,
        max: PIVOT_LIMIT,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: PIVOT_Y_PARAM,
        label: "Pivot Y",
        min: -PIVOT_LIMIT,
        max: PIVOT_LIMIT,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: FILTER_PARAM,
        label: "Filter",
        min: 0.0,
        max: (FILTER_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &FILTER_LABELS,
        },
    },
    ParamUiHint {
        param: SORT_PARAM,
        label: "Sort",
        min: 0.0,
        max: (SORT_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &SORT_LABELS,
        },
    },
];

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
