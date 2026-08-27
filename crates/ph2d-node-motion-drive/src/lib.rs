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

use ph2d_node_registry::{NodeRegistry, ParamChannelRange, RegistryError};
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
/// Os kernels WGSL — cortados no teto de LOC (HR-18); ver o cabeçalho deles.
mod kernel;
mod trig;
use kernel::GPU_KERNEL;
/// A aritmética que junta o valor conduzido ao canal — os sete modos.
mod combine;
pub use channel::DRIVE_COL_KEY;
use channel::{CH_CUSTOM, Combine, drive_channel, drive_named};

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
        // **EM QUE EIXO** o deslocamento cai — `0` mundo (o de sempre), `1` o do próprio
        // elemento. Só os canais `X`/`Y` o lêem; ver [`channel::local_axis`].
        //
        // ⚠️ **É capacidade e não ergonomia, e foi MEDIDO:** o `value.math` tem dezassete
        // operações e **nenhuma trigonométrica**, então não existe cadeia capaz de virar a
        // coluna `rot` numa direcção. Sem este param o espaço do elemento é inexprimível.
        ParamSpec {
            name: "space",
            default: 0.0,
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
        // ⚠️ O canal CUSTOM pega o alvo do text param; os nove do enum pegam-no do
        // `channel_column`. Uma porta por pergunta, e a LEI (blend · broadcast ·
        // falloff) é a mesma nas duas.
        let out = if channel == CH_CUSTOM {
            let name = ctx.text_param(DRIVE_COL_KEY).unwrap_or("").to_string();
            drive_named(ctx.input(0), &name, &vals, scale, mode)
        } else {
            drive_channel(
                ctx.input(0),
                channel,
                &vals,
                scale,
                mode,
                ctx.param("space") >= 0.5,
            )
        };
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
    // ⚠️ A linha do nome só aparece no canal que a LÊ — um campo de texto
    // visível sob "Rotation" seria o controle morto que esta casa recusa.
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_channel_range(MANIFEST.id, PARAM_CHANNEL_RANGE);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_hard_min(MANIFEST.id, PARAM_HARD_MIN);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamUiHint, ParamWidget};

/// A linha do NOME pertence ao canal Custom e a mais nenhum.
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: DRIVE_COL_KEY,
        when: "channel",
        values: &[CH_CUSTOM],
    },
    // ⚠️ **Só `X` e `Y` têm eixo.** Uma rotação, um tamanho ou um matiz não apontam para
    // lado nenhum, e um selector de espaço pintado ali seria um knob que não muda um
    // quadro — o defeito que a caça do doc 90 desta linha existiu para apagar.
    ParamGate {
        param: "space",
        when: "channel",
        values: &[0, 1],
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "channel",
        label: "Channel",
        min: 0.0,
        max: 11.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            // ⚠️ **Apendados**, nunca inseridos: o índice é o que o grafo guarda, então uma
            // ordem "mais bonita" (as três cores ao lado do Opacity) trocaria o canal de todo
            // documento já autorado — em silêncio, porque o param é um `f32` sem versão.
            labels: &[
                "X",
                "Y",
                "Rotation",
                "Size",
                "Opacity",
                "Falloff",
                "Hue",
                "Saturation",
                "Value",
                "Custom…",
                "Size X",
                "Size Y",
            ],
        },
    },
    // ⚠️ **O NOME da coluna é um TEXT param**, não um `ParamSpec`: o manifesto é
    // f32-only por contrato congelado (ADR-0039), e o canal de texto do `Graph` é
    // o padrão canônico para param não-f32 — o mesmo assento que a fórmula do
    // `motion.expression` e a tabela do `value.pattern` ocupam.
    ParamUiHint {
        param: DRIVE_COL_KEY,
        label: "Column",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: "space",
        label: "Space",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["World", "Element"],
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
        max: 7.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            // ⚠️ Os CINCO últimos são APENDADOS (folha 06 linhas 40 e 41): `0..2` ficam
            // onde estavam, então todo documento já autorado lê o mesmo modo.
            //
            // ⚠️ **`Remap` não é um `Set` com outro nome**, e o que os separa é o que a
            // MÁSCARA faz: em `Set` um `falloff = 0` protege o valor de origem, em `Remap`
            // ele leva o canal a ZERO. É a diferença entre *"pinte por cima onde a máscara
            // deixar"* e *"esta máscara É o valor"* — ver [`combine::Combine::Remap`].
            labels: &[
                "Add", "Set", "Multiply", "Subtract", "Divide", "Min", "Max", "Remap",
            ],
        },
    },
];
/// **A faixa que estas magnitudes querem quando o canal é ANGULAR** — graus, não
/// unidades de mundo. Uma volta para cada lado, discada em graus inteiros.
///
/// ⚠️ Ela mora AQUI e não numa tabela do shell porque a tabela apodreceu: medida,
/// ela cobria três dos seis nós que precisavam dela, e cada um dos três ausentes
/// esperava o próprio report do artista.
const TURN: f32 = 360.0;
static PARAM_CHANNEL_RANGE: &[ParamChannelRange] = &[ParamChannelRange {
    param: "scale",
    min: -TURN,
    max: TURN,
    step: 1.0,
}];

/// **O teto DIGITÁVEL do `scale`, e o recurso é a PRECISÃO DE REPRESENTAÇÃO.**
///
/// ⚠️ O `scale` é uma multiplicação PURA — nem a CPU nem o WGSL o clampam —, então
/// o teto do slider não era um teto do kernel: medido, o canal Rotation honra
/// `1e7` graus com erro `0.000e0`. Era um número de UI sem recurso nenhum atrás,
/// e a §0 do CLAUDE.md chama isso pelo nome.
///
/// O limite real é o `f32`: **`2^24 = 16 777 216` é o primeiro número em que
/// `x + 1.0 == x`** (medido, não escolhido). Acima dele um passo de UM grau não
/// move o número — o controle deixa de controlar, que é onde *o disfuncional
/// começa*. Abaixo, tudo é honrado exatamente.
const F32_UNIT_STEP_CEILING: f32 = 16_777_216.0;
static PARAM_HARD_MAX: &[ph2d_node_registry::ParamHardMax] = &[ph2d_node_registry::ParamHardMax {
    param: "scale",
    max: F32_UNIT_STEP_CEILING,
}];
/// O piso, espelhado: um `scale` negativo INVERTE o drive, e o slider já o oferece.
static PARAM_HARD_MIN: &[ph2d_node_registry::ParamHardMin] = &[ph2d_node_registry::ParamHardMin {
    param: "scale",
    min: -F32_UNIT_STEP_CEILING,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "space_tests.rs"]
mod space_tests;

#[cfg(test)]
#[path = "custom_tests.rs"]
mod custom_tests;
