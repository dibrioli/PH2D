//! `value.send_signal` — **Send signal** (GEF 08 MI-8): o grafo publica um sinal NOMEADO quando um
//! valor cruza um nível (Rises above · Falls below) ou a cada disparo de um pulso (Pulse).
//!
//! O `eval` é Pure: passa o `value` adiante e escreve nele o que a ponte precisa (`when`, `level`,
//! quantos pulsos dispararam). A BORDA não mora aqui — o cook re-roda num scrub e não tem memória
//! de quadro: mora em [`Envios`], estado da ponte de CPU (`ph2d-app-motion`,
//! `motion_bridge_signals::collect_signals`), que lê a tomada deste nó tique a tique, a mesma do
//! `pulse.signal`. Quem publica no `SignalOutbox` é o shell, só com o relógio a tocar para a frente.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, ParamGate, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::NodeId;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::collections::BTreeMap;

const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);
const VALUE_COL: &str = "v";
const PULSE_COL: &str = "pulse";
const WHEN_COL: &str = "send.when";
const LEVEL_COL: &str = "send.level";
const FIRED_COL: &str = "send.pulses";

/// O nome do sinal — a MESMA chave de texto do `pulse.signal`, para a ponte ler os dois igual.
pub const NAME_KEY: &str = "name";
pub const WHEN: &str = "when";
pub const LEVEL: &str = "level";
pub const WHEN_PULSE: i32 = 0;
pub const WHEN_RISES: i32 = 1;
pub const WHEN_FALLS: i32 = 2;

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.send_signal"),
    name: "value.send_signal",
    inputs: &[
        PortSpec {
            name: "value",
            ty: VALUE,
        },
        PortSpec {
            name: "pulse",
            ty: PULSE,
        },
    ],
    // Passa o valor adiante, como o `pulse.signal` passa o pulso: entra no meio do fio.
    outputs: &[PortSpec {
        name: "value",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: WHEN,
            default: 0.0,
        },
        ParamSpec {
            name: LEVEL,
            default: 0.5,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Quantas linhas de um stream de pulso dispararam (`> 0.5`, a soleira da família `pulse.*`).
#[must_use]
pub fn pulsos(stream: &Stream) -> usize {
    match stream.get(PULSE_COL) {
        Some(Column::Scalar(v)) => v.iter().filter(|&&x| x > 0.5).count(),
        _ => 0,
    }
}

/// A saída: o `value` de entrada (uma linha a zero sem fio), com as três colunas da ponte.
#[must_use]
pub fn saida(value: &Stream, pulse: &Stream, when: f32, level: f32) -> Stream {
    let base = if value.count() > 0 {
        value.clone()
    } else {
        Stream::new(1).with(VALUE_COL, Column::Scalar(vec![0.0]))
    };
    let n = base.count();
    let todos = |x: f32| Column::Scalar(vec![x; n]);
    base.with(WHEN_COL, todos(when))
        .with(LEVEL_COL, todos(level))
        .with(FIRED_COL, todos(pulsos(pulse) as f32))
}

/// O que a ponte lê da tomada deste nó num tique.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Leitura {
    pub v: f32,
    pub when: i32,
    pub level: f32,
    pub pulsos: usize,
}

/// A leitura da tomada; `None` quando o stream não é de um Send signal (o `pulse.signal`).
#[must_use]
pub fn ler(stream: &Stream) -> Option<Leitura> {
    let primeiro = |c: &str| match stream.get(c) {
        Some(Column::Scalar(v)) => v.first().copied(),
        _ => None,
    };
    Some(Leitura {
        when: primeiro(WHEN_COL)?.round() as i32,
        level: primeiro(LEVEL_COL)?,
        pulsos: primeiro(FIRED_COL)? as usize,
        v: primeiro(VALUE_COL).unwrap_or(0.0),
    })
}

/// ⭐ O estado da PONTE: o valor do tique anterior por nó (a borda) e quantas vezes cada um enviou
/// (o «Sent <signal> N times» do cartão). Fora do documento, do save e do undo.
#[derive(Debug, Default, Clone)]
pub struct Envios {
    antes: BTreeMap<NodeId, (u64, f32)>,
    vezes: BTreeMap<NodeId, u64>,
}

impl Envios {
    /// Um tique do livro-razão. Devolve quantas linhas o sinal leva (`0` = não envia).
    ///
    /// A borda só vale entre dois tiques SEGUIDOS: o primeiro tique, um salto ou o recomeço depois
    /// de parar apenas guardam o valor — nunca uma borda falsa.
    pub fn passo(&mut self, node: NodeId, tick: u64, l: &Leitura) -> usize {
        let antes = self
            .antes
            .insert(node, (tick, l.v))
            .filter(|&(t, _)| t + 1 == tick)
            .map(|(_, v)| v);
        let linhas = match l.when {
            WHEN_RISES => usize::from(antes.is_some_and(|a| a < l.level && l.level <= l.v)),
            WHEN_FALLS => usize::from(antes.is_some_and(|a| a > l.level && l.level >= l.v)),
            _ => l.pulsos,
        };
        if linhas > 0 {
            *self.vezes.entry(node).or_default() += 1;
        }
        linhas
    }

    /// Quantas vezes este nó enviou o sinal nesta sessão.
    #[must_use]
    pub fn vezes(&self, node: NodeId) -> u64 {
        self.vezes.get(&node).copied().unwrap_or(0)
    }

    /// Esquece os nós que saíram do grafo.
    pub fn retem(&mut self, vivos: &[NodeId]) {
        self.antes.retain(|n, _| vivos.contains(n));
        self.vezes.retain(|n, _| vivos.contains(n));
    }
}

struct SendSignal;

impl NodeOp for SendSignal {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Lido para entrar na impressão digital do cook (renomear invalida o memo).
        let _ = ctx.text_param(NAME_KEY);
        let (when, level) = (ctx.param(WHEN).round(), ctx.param(LEVEL));
        let out = saida(ctx.input(0), ctx.input(1), when, level);
        ctx.emit(out);
    }
}

/// Registra o tipo no registry.
///
/// # Errors
/// Devolve [`RegistryError`] se o tipo já estiver registrado.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SendSignal))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_key: "node.value.send_signal.name",
            category: ph2d_node_registry::NodeUiCategory::Output,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    Ok(())
}

static PARAM_GATES: &[ParamGate] = &[ParamGate {
    param: LEVEL,
    when: WHEN,
    values: &[WHEN_RISES, WHEN_FALLS],
}];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: NAME_KEY,
        label: "node.value.send_signal.param.name",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: WHEN,
        label: "node.value.send_signal.param.when",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.value.send_signal.param.when.0",
                "node.value.send_signal.param.when.1",
                "node.value.send_signal.param.when.2",
            ],
        },
    },
    ParamUiHint {
        param: LEVEL,
        label: "node.value.send_signal.param.level",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
