//! `value.input` — **uma ENTRADA PUBLICADA do grafo**: um número que o jogo escreve e o grafo lê
//! (ciclo 19, [doc 128 §3.1](../../../docs/Motion%20Nodes/128_ciclo_19_o_jogo_manda.md)).
//!
//! O nome (o text param `name`) é a morada: o componente `Motion Input`, o Luau
//! (`motion.set("vida", 0.3)`) e os sinais do runtime escrevem em
//! [`external::input_of`](ph2d_nodegraph::external::input_of)`(nome)`, e este nó lê-o. **Sem
//! publicação, vale o número do PRÓPRIO cartão** — o botão de teste do artista, e o valor que um
//! grafo mostra no editor sem o jogo a correr.
//!
//! Quatro tipos (o vocabulário do Rive — Number · Boolean · Trigger — mais a cor do Data Binding):
//!
//! | `kind` | `out` | `r g b a` |
//! |---|---|---|
//! | Number | o número | o cinza dele, opaco |
//! | Toggle | `0` / `1` | idem |
//! | Trigger | `1` no tique do disparo, `0` nos outros | idem |
//! | Color | a luminosidade OKLab (`L`) | a cor |
//!
//! E `pulse`: a coluna do `out` no tipo da família `pulse.*` — um Trigger liga direto ao
//! `pulse.adsr` / `pulse.counter`.
//!
//! ⚠️ **O gatilho é um pulso decidido por quem ESCREVE** (doc 128 §3.2): o escritor conta os
//! disparos e publica `1` no tique em que a conta mudou. Uma borda detectada AQUI precisaria do
//! laço `pre`, e um nó sequencial é proibido debaixo de um `time_remap`. O `fire` do cartão é o
//! disparo de teste: cada clique muda-o, e quem publica (a ponte do editor) vê a mudança.
//!
//! **Vários escritores com o mesmo nome** ⇒ uma linha por escritor (o campo de comprimento N); um
//! param dirigido lê a primeira (`driven_value`).
//!
//! `Effect::Pure` e CPU: o valor vem de um external, que um kernel não alcança (o irmão
//! `value.cursor` pela mesma razão); a placa recebe-o como o número de um param dirigido.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, ParamGate, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::external::{INPUT_COLOR, INPUT_VALUE, input_of};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
/// O PULSO (o tipo da família `pulse.*`): um gatilho liga direto a um `pulse.adsr`/`counter`.
const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);
const VALUE_COL: &str = "v";
const PULSE_COL: &str = "pulse";

/// O text param com o NOME da entrada.
pub const NAME_KEY: &str = "name";
/// O nome com que um nó largado nasce.
pub const FACTORY_NAME: &str = "input";

pub const KIND: &str = "kind";
pub const KIND_NUMBER: i32 = 0;
pub const KIND_TOGGLE: i32 = 1;
pub const KIND_TRIGGER: i32 = 2;
pub const KIND_COLOR: i32 = 3;
pub const VALUE_PARAM: &str = "value";
pub const STATE: &str = "state";
/// O disparo de TESTE do cartão: cada mudança é um disparo (ver o cabeçalho).
pub const FIRE: &str = "fire";
pub const COLOR: [&str; 4] = ["r", "g", "b", "a"];

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.input"),
    name: "value.input",
    inputs: &[],
    outputs: &[
        PortSpec {
            name: "out",
            ty: VALUE,
        },
        PortSpec {
            name: "r",
            ty: VALUE,
        },
        PortSpec {
            name: "g",
            ty: VALUE,
        },
        PortSpec {
            name: "b",
            ty: VALUE,
        },
        PortSpec {
            name: "a",
            ty: VALUE,
        },
        // A mesma coluna do `out`, no tipo do PULSO (num Trigger, o disparo).
        PortSpec {
            name: "pulse",
            ty: PULSE,
        },
    ],
    // Pure: o memo guarda a revisão de cada external lido, então isto volta a cozer exactamente
    // quando a entrada muda.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: KIND,
            default: 0.0,
        },
        ParamSpec {
            name: VALUE_PARAM,
            default: 0.0,
        },
        ParamSpec {
            name: STATE,
            default: 0.0,
        },
        ParamSpec {
            name: FIRE,
            default: 0.0,
        },
        ParamSpec {
            name: COLOR[0],
            default: 1.0,
        },
        ParamSpec {
            name: COLOR[1],
            default: 1.0,
        },
        ParamSpec {
            name: COLOR[2],
            default: 1.0,
        },
        ParamSpec {
            name: COLOR[3],
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// O que o nó emite, por linha: `(out, [r, g, b, a])`.
#[must_use]
pub fn ler(
    kind: i32,
    publicado: Option<&Stream>,
    cartao: impl Fn(&str) -> f32,
) -> Vec<(f32, [f32; 4])> {
    let cinza = |x: f32| (x, [x, x, x, 1.0]);
    let numeros = publicado.and_then(|s| match s.get(INPUT_VALUE) {
        Some(Column::Scalar(v)) if !v.is_empty() => Some(v.as_slice()),
        _ => None,
    });
    match kind {
        KIND_COLOR => {
            let cores: Vec<[f32; 4]> = match publicado.and_then(|s| s.get(INPUT_COLOR)) {
                Some(Column::Vec4(c)) if !c.is_empty() => c.clone(),
                _ => match numeros {
                    Some(v) => v.iter().map(|&x| [x, x, x, 1.0]).collect(),
                    None => vec![COLOR.map(&cartao)],
                },
            };
            cores
                .into_iter()
                .map(|c| (ph2d_color::perceptual::to_oklab([c[0], c[1], c[2]])[0], c))
                .collect()
        }
        KIND_TOGGLE => match numeros {
            Some(v) => v.iter().map(|&x| cinza(f32::from(x >= 0.5))).collect(),
            None => vec![cinza(f32::from(cartao(STATE) >= 0.5))],
        },
        KIND_TRIGGER => match numeros {
            Some(v) => v.iter().map(|&x| cinza(x)).collect(),
            None => vec![cinza(0.0)],
        },
        _ => match numeros {
            Some(v) => v.iter().map(|&x| cinza(x)).collect(),
            None => vec![cinza(cartao(VALUE_PARAM))],
        },
    }
}

struct ValueInput;

impl NodeOp for ValueInput {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let kind = ctx.param(KIND).round() as i32;
        let nome = ctx
            .text_param(NAME_KEY)
            .unwrap_or(FACTORY_NAME)
            .trim()
            .to_string();
        let publicado = if nome.is_empty() {
            None
        } else {
            Some(ctx.external(&input_of(&nome)))
        };
        let publicado = publicado.filter(|s| s.count() > 0);
        let linhas = ler(kind, publicado, |p| ctx.param(p));
        let n = linhas.len();
        let col = |v: Vec<f32>| Stream::new(n).with(VALUE_COL, Column::Scalar(v));
        // Na ORDEM das saídas do manifesto (o contrato do `emit`).
        ctx.emit(col(linhas.iter().map(|l| l.0).collect()));
        for k in 0..4 {
            ctx.emit(col(linhas.iter().map(|l| l.1[k]).collect()));
        }
        // O pulso viaja na coluna da família `pulse.*`.
        let pulso = linhas.iter().map(|l| l.0).collect();
        ctx.emit(Stream::new(n).with(PULSE_COL, Column::Scalar(pulso)));
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueInput))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_key: "node.value.input.name",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_text_defaults(MANIFEST.id, FACTORY_TEXT);
    Ok(())
}

static FACTORY_TEXT: &[(&str, &str)] = &[(NAME_KEY, FACTORY_NAME)];

static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: VALUE_PARAM,
        when: KIND,
        values: &[KIND_NUMBER],
    },
    ParamGate {
        param: STATE,
        when: KIND,
        values: &[KIND_TOGGLE],
    },
    ParamGate {
        param: FIRE,
        when: KIND,
        values: &[KIND_TRIGGER],
    },
    ParamGate {
        param: COLOR[0],
        when: KIND,
        values: &[KIND_COLOR],
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: NAME_KEY,
        label: "node.value.input.param.name",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: KIND,
        label: "node.value.input.param.kind",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.value.input.param.kind.0",
                "node.value.input.param.kind.1",
                "node.value.input.param.kind.2",
                "node.value.input.param.kind.3",
            ],
        },
    },
    ParamUiHint {
        param: VALUE_PARAM,
        label: "node.value.input.param.value",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: STATE,
        label: "node.value.input.param.state",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: FIRE,
        label: "node.value.input.param.fire",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: COLOR[0],
        label: "node.value.input.param.r",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color { channels: COLOR },
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
