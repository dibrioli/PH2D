//! **`fx.lens` — A LENTE DA CÂMERA** (Motion ciclo 20, [doc 129](../../../docs/Motion%20Nodes/129_ciclo_20_o_acabamento.md)
//! §3.2): aberração cromática · vinheta · grade (+ LUT `.cube`) · grão, sobre a CENA inteira, na
//! ordem fixa da física.
//!
//! Nó passthrough (`out == in`, `Effect::Pure`) com a forma do `fx.glow`: ele DECLARA a lente, e
//! o passe da placa (`ph2d_render::LensPass`, chamado por `ph2d-app-motion`) corre-a sobre o
//! acumulador do mundo. Sem o nó no grafo o quadro é byte-idêntico. Um passe de tela é UM: o
//! [`from_graph`] lê o primeiro nó, e o diagnóstico marca os outros.
//!
//! ⭐ A vinheta nasce LIGADA (`0,4`), para o artista ver a lente ao largar o nó; o resto nasce
//! neutro e as secções dele fechadas.

#![forbid(unsafe_code)]

use ph2d_lens::{Aberracao, Grade, Grao, Lente, Vinheta};
use ph2d_node_registry::{
    FileKind, NodeRegistry, NodeSilhouette, NodeUiCategory, NodeUiManifest, ParamGate,
    ParamGateText, ParamGroup, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget, RegistryError,
};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::collections::BTreeMap;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// O tipo canónico — quem varre o grafo procura este nome.
pub const TYPE_NAME: &str = "fx.lens";

/// O param de TEXTO com o caminho do `.cube`.
pub const LUT: &str = "lut";

/// Os nomes dos params (as chaves do grafo).
pub mod param {
    pub const AB: &str = "aberration";
    pub const AB_MODE: &str = "aberration_mode";
    pub const AB_ANGLE: &str = "aberration_angle";
    pub const VIN: &str = "vignette";
    pub const VIN_MID: &str = "vignette_midpoint";
    pub const VIN_ROUND: &str = "vignette_roundness";
    pub const VIN_FEATHER: &str = "vignette_feather";
    pub const VIN_CX: &str = "vignette_center_x";
    pub const VIN_CY: &str = "vignette_center_y";
    pub const VIN_COR: [&str; 4] = ["vignette_r", "vignette_g", "vignette_b", "vignette_a"];
    pub const EXPOSURE: &str = "exposure";
    pub const TEMPERATURE: &str = "temperature";
    pub const TINT: &str = "tint";
    pub const CONTRAST: &str = "contrast";
    pub const SATURATION: &str = "saturation";
    pub const LUT_MIX: &str = "lut_mix";
    pub const GRAIN: &str = "grain";
    pub const GRAIN_SIZE: &str = "grain_size";
    pub const GRAIN_RESPONSE: &str = "grain_response";
    pub const GRAIN_COLOR: &str = "grain_color";
    pub const GRAIN_ANIMATED: &str = "grain_animated";
    pub const GRAIN_SEED: &str = "grain_seed";
}

const fn p(name: &'static str, default: f32) -> ParamSpec {
    ParamSpec { name, default }
}

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("fx.lens"),
    name: "fx.lens",
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
        p(param::AB, 0.0),
        p(param::AB_MODE, 0.0),
        p(param::AB_ANGLE, 0.0),
        p(param::VIN, 0.4),
        p(param::VIN_MID, 1.0),
        p(param::VIN_ROUND, 0.0),
        p(param::VIN_FEATHER, 0.5),
        p(param::VIN_CX, 0.0),
        p(param::VIN_CY, 0.0),
        p(param::VIN_COR[0], 0.0),
        p(param::VIN_COR[1], 0.0),
        p(param::VIN_COR[2], 0.0),
        p(param::VIN_COR[3], 1.0),
        p(param::EXPOSURE, 0.0),
        p(param::TEMPERATURE, 0.0),
        p(param::TINT, 0.0),
        p(param::CONTRAST, 0.0),
        p(param::SATURATION, 0.0),
        p(param::LUT_MIX, 1.0),
        p(param::GRAIN, 0.0),
        p(param::GRAIN_SIZE, 1.5),
        p(param::GRAIN_RESPONSE, 0.5),
        p(param::GRAIN_COLOR, 0.0),
        p(param::GRAIN_ANIMATED, 1.0),
        p(param::GRAIN_SEED, 0.0),
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Os modos da aberração (o índice é o valor do param).
pub static AB_MODE_LABELS: [&str; 2] = [
    "node.opts.node_fx_lens.aberration_mode_labels.0",
    "node.opts.node_fx_lens.aberration_mode_labels.1",
];

/// O nó da lente no grafo, lido.
#[derive(Clone, Debug, PartialEq)]
pub struct NoDaLente {
    pub no: NodeId,
    pub lente: Lente,
    /// O caminho do `.cube` (`None` = sem LUT).
    pub lut: Option<String>,
}

fn default_of(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|p| p.name == name)
        .map_or(0.0, |p| p.default)
}

fn read(ov: Option<&BTreeMap<String, f32>>, name: &str) -> f32 {
    ov.and_then(|m| m.get(name))
        .copied()
        .unwrap_or_else(|| default_of(name))
}

/// **A lente do grafo** — a do PRIMEIRO `fx.lens` (override-ou-default). `None` sem o nó: o passe
/// não corre e o quadro é byte-idêntico.
#[must_use]
pub fn from_graph(graph: &Graph) -> Option<NoDaLente> {
    let no = graph.nodes().iter().find(|n| n.type_name == TYPE_NAME)?.id;
    let ov = graph.node_param_overrides(no);
    let r = |n: &str| read(ov, n);
    let lut = graph
        .node_text_param_overrides(no)
        .and_then(|m| m.get(LUT))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a semente é um inteiro guardado num f32"
    )]
    let semente = r(param::GRAIN_SEED).round().max(0.0) as u32;
    Some(NoDaLente {
        no,
        lente: Lente {
            aberracao: Aberracao {
                amount: r(param::AB),
                linear: r(param::AB_MODE).round() >= 1.0,
                // O cartão fala em GRAUS; a lei, em voltas.
                angle: r(param::AB_ANGLE) / 360.0,
            },
            vinheta: Vinheta {
                amount: r(param::VIN),
                midpoint: r(param::VIN_MID),
                roundness: r(param::VIN_ROUND),
                feather: r(param::VIN_FEATHER),
                center: [r(param::VIN_CX), r(param::VIN_CY)],
                cor: [
                    r(param::VIN_COR[0]),
                    r(param::VIN_COR[1]),
                    r(param::VIN_COR[2]),
                ],
            },
            grade: Grade {
                exposure: r(param::EXPOSURE),
                temperatura: r(param::TEMPERATURE),
                matiz: r(param::TINT),
                contraste: r(param::CONTRAST),
                saturacao: r(param::SATURATION),
                lut_mix: r(param::LUT_MIX),
            },
            grao: Grao {
                amount: r(param::GRAIN),
                size: r(param::GRAIN_SIZE),
                response: r(param::GRAIN_RESPONSE),
                colorido: r(param::GRAIN_COLOR) >= 0.5,
                animado: r(param::GRAIN_ANIMATED) >= 0.5,
                semente,
            },
        },
        lut,
    })
}

struct FxLens;

impl NodeOp for FxLens {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Passthrough: o nó DECLARA a lente; o fio passa intacto.
        let out = ctx.input(0).clone();
        ctx.emit(out);
    }
}

/// Regista o nó (chamado por `ph2d-node-registry-init`).
///
/// # Errors
/// O [`RegistryError`] do registo.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FxLens))?;
    // O kernel é o PASSTHROUGH (ADR-0126): sem ele a cadeia inteira cairia na CPU (o mesmo
    // porquê do `fx.glow`).
    reg.register_gpu_kernel(MANIFEST.id, ph2d_nodegraph::gpu::GpuKernel::PASSTHROUGH);
    reg.register_ui(
        MANIFEST.id,
        NodeUiManifest {
            display_key: "node.fx.lens.name",
            category: NodeUiCategory::Fx,
            silhouette: NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_gates_text(MANIFEST.id, PARAM_GATES_TEXT);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

/// As quatro secções, na ordem fixa da lente. Só a vinheta (a que nasce ligada) nasce aberta.
static PARAM_GROUPS: &[ParamGroup] = &[
    ParamGroup::new(param::AB, "node.group.aberration").folded(),
    ParamGroup::new(param::AB_MODE, "node.group.aberration").folded(),
    ParamGroup::new(param::AB_ANGLE, "node.group.aberration").folded(),
    ParamGroup::new(param::VIN, "node.group.vignette"),
    ParamGroup::new(param::VIN_MID, "node.group.vignette"),
    ParamGroup::new(param::VIN_ROUND, "node.group.vignette"),
    ParamGroup::new(param::VIN_FEATHER, "node.group.vignette"),
    ParamGroup::new(param::VIN_CX, "node.group.vignette"),
    ParamGroup::new(param::VIN_CY, "node.group.vignette"),
    ParamGroup::new(param::VIN_COR[0], "node.group.vignette"),
    ParamGroup::new(param::EXPOSURE, "node.group.grade").folded(),
    ParamGroup::new(param::TEMPERATURE, "node.group.grade").folded(),
    ParamGroup::new(param::TINT, "node.group.grade").folded(),
    ParamGroup::new(param::CONTRAST, "node.group.grade").folded(),
    ParamGroup::new(param::SATURATION, "node.group.grade").folded(),
    ParamGroup::new(LUT, "node.group.grade").folded(),
    ParamGroup::new(param::LUT_MIX, "node.group.grade").folded(),
    ParamGroup::new(param::GRAIN, "node.group.grain").folded(),
    ParamGroup::new(param::GRAIN_SIZE, "node.group.grain").folded(),
    ParamGroup::new(param::GRAIN_RESPONSE, "node.group.grain").folded(),
    ParamGroup::new(param::GRAIN_COLOR, "node.group.grain").folded(),
    ParamGroup::new(param::GRAIN_ANIMATED, "node.group.grain").folded(),
    ParamGroup::new(param::GRAIN_SEED, "node.group.grain").folded(),
];

const fn slider(
    param: &'static str,
    label: &'static str,
    min: f32,
    max: f32,
    step: f32,
) -> ParamUiHint {
    ParamUiHint {
        param,
        label,
        min,
        max,
        step,
        widget: ParamWidget::Slider,
    }
}

const fn toggle(param: &'static str, label: &'static str) -> ParamUiHint {
    ParamUiHint {
        param,
        label,
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    }
}

static PARAM_HINTS: &[ParamUiHint] = &[
    // A aberração em fração da meia-diagonal: `0,05` já separa 3 % das bordas.
    slider(param::AB, "node.fx.lens.param.aberration", 0.0, 0.1, 0.001),
    ParamUiHint {
        param: param::AB_MODE,
        label: "node.fx.lens.param.aberration_mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &AB_MODE_LABELS,
        },
    },
    ParamUiHint {
        param: param::AB_ANGLE,
        label: "node.fx.lens.param.aberration_angle",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    slider(param::VIN, "node.fx.lens.param.vignette", -1.0, 1.0, 0.01),
    slider(
        param::VIN_MID,
        "node.fx.lens.param.vignette_midpoint",
        0.0,
        2.0,
        0.01,
    ),
    slider(
        param::VIN_ROUND,
        "node.fx.lens.param.vignette_roundness",
        -1.0,
        1.0,
        0.01,
    ),
    slider(
        param::VIN_FEATHER,
        "node.fx.lens.param.vignette_feather",
        0.0,
        1.0,
        0.01,
    ),
    slider(
        param::VIN_CX,
        "node.fx.lens.param.vignette_center_x",
        -1.0,
        1.0,
        0.01,
    ),
    slider(
        param::VIN_CY,
        "node.fx.lens.param.vignette_center_y",
        -1.0,
        1.0,
        0.01,
    ),
    ParamUiHint {
        param: param::VIN_COR[0],
        label: "node.fx.lens.param.vignette_r",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: param::VIN_COR,
        },
    },
    slider(
        param::EXPOSURE,
        "node.fx.lens.param.exposure",
        -4.0,
        4.0,
        0.05,
    ),
    slider(
        param::TEMPERATURE,
        "node.fx.lens.param.temperature",
        -1.0,
        1.0,
        0.01,
    ),
    slider(param::TINT, "node.fx.lens.param.tint", -1.0, 1.0, 0.01),
    slider(
        param::CONTRAST,
        "node.fx.lens.param.contrast",
        -1.0,
        1.0,
        0.01,
    ),
    slider(
        param::SATURATION,
        "node.fx.lens.param.saturation",
        -1.0,
        1.0,
        0.01,
    ),
    // ⚠️ Um param de TEXTO precisa de hint para EXISTIR no cartão (a lição do `fx.glow`).
    ParamUiHint {
        param: LUT,
        label: "node.fx.lens.param.lut",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::File {
            kind: FileKind::Lut,
        },
    },
    slider(param::LUT_MIX, "node.fx.lens.param.lut_mix", 0.0, 1.0, 0.01),
    slider(param::GRAIN, "node.fx.lens.param.grain", 0.0, 1.0, 0.01),
    slider(
        param::GRAIN_SIZE,
        "node.fx.lens.param.grain_size",
        1.0,
        8.0,
        0.1,
    ),
    slider(
        param::GRAIN_RESPONSE,
        "node.fx.lens.param.grain_response",
        0.0,
        1.0,
        0.01,
    ),
    toggle(param::GRAIN_COLOR, "node.fx.lens.param.grain_color"),
    toggle(param::GRAIN_ANIMATED, "node.fx.lens.param.grain_animated"),
    ParamUiHint {
        param: param::GRAIN_SEED,
        label: "node.fx.lens.param.grain_seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
];

/// A direção só existe no modo Linear (no Radial o canal escala a partir do centro).
static PARAM_GATES: &[ParamGate] = &[ParamGate {
    param: param::AB_ANGLE,
    when: param::AB_MODE,
    values: &[1],
}];

/// O `LUT Mix` só aparece com uma LUT escolhida.
static PARAM_GATES_TEXT: &[ParamGateText] = &[ParamGateText {
    param: param::LUT_MIX,
    when_text: LUT,
    when_present: true,
}];

static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: param::AB_ANGLE,
    unit: ParamUnit::Angle,
}];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
