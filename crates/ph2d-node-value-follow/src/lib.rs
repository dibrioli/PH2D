//! `value.follow` — **o suavizador TEMPORAL de valores**: a saída persegue a entrada no tempo
//! (ciclo 19, [doc 128 §3.5](../../../docs/Motion%20Nodes/128_ciclo_19_o_jogo_manda.md)).
//!
//! É o que faz um crossfade de estados sem máquina nova: `value.input("estado") → value.follow →
//! mixer.weight_k`. Faltava — o `value.smooth` é uma média ESPACIAL (sobre as instâncias) e o
//! `motion.spring` só age em canais de transformação. É o *Lag* do TouchDesigner e o `dampTime`
//! do `Animator.SetFloat` da Unity, aqui como NÓ (legível no grafo, não escondido na escrita).
//!
//! Três modos, um `duration`:
//!
//! | modo | lei | `duration` |
//! |---|---|---|
//! | Linear | anda a `1/duration` por segundo até ao alvo | o tempo de um salto de `0` a `1` (o `xfade_time` do Godot — o oráculo, doc 128 §5.2) |
//! | Smooth | `v += (alvo − v)·(1 − 2^(−dt/duration))` | a MEIA-VIDA |
//! | Spring | a lei do `motion.spring` (`law::passo`), modo `Time` | o período; `bounce` o salto |
//!
//! ⭐ **Uma mudança conta a partir do tique em que CHEGA** — o passo de cada tique anda na direcção
//! do alvo do tique ANTERIOR (guardado no estado). É a lei do `AnimationNodeTransition` do Godot,
//! CORRIDO como oráculo (doc 128 §5.2): no quadro do pedido o peso ainda é o de antes, e sobe
//! `dt/xfade` por quadro a partir do seguinte. Andar já no quadro do pedido poria a transição um
//! quadro à frente.
//!
//! **Sequencial** pelo laço `pre` (`out --pre--> state`, a convenção da casa): o estado viaja na
//! saída (`v` e `follow_vel`). No primeiro tique, ou quando o número de linhas muda, nasce NO
//! alvo (sem salto). `dt` é o do relógio de raiz, encostado a `MAX_DT` da mola.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, ParamGate, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";
const VEL_COL: &str = "follow_vel";
/// O alvo do tique ANTERIOR — o passo anda na direcção dele (ver o cabeçalho).
const ALVO_COL: &str = "follow_target";

pub const MODE: &str = "mode";
pub const MODE_LINEAR: i32 = 0;
pub const MODE_SMOOTH: i32 = 1;
pub const MODE_SPRING: i32 = 2;
pub const DURATION: &str = "duration";
pub const BOUNCE: &str = "bounce";
/// O piso da duração: abaixo dele cada modo é o alvo no próprio tique (um passo a 240 Hz).
pub const MIN_DURATION: f32 = 1.0 / 240.0;

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.follow"),
    name: "value.follow",
    inputs: &[
        PortSpec {
            name: "in",
            ty: VALUE,
        },
        PortSpec {
            name: "state",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Temporal: o estado anda com o relógio (o irmão `motion.spring` declara o mesmo).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: MODE,
            default: 0.0,
        },
        ParamSpec {
            name: DURATION,
            default: 0.3,
        },
        ParamSpec {
            name: BOUNCE,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Um passo de um elemento: `(valor, velocidade)` depois de `dt`.
#[must_use]
pub fn passo(
    mode: i32,
    duration: f32,
    bounce: f32,
    valor: f32,
    vel: f32,
    alvo: f32,
    dt: f32,
) -> (f32, f32) {
    if duration <= MIN_DURATION {
        return (alvo, 0.0);
    }
    match mode {
        MODE_SMOOTH => {
            let k = 1.0 - ph2d_ease::exp2(-dt / duration);
            (valor + (alvo - valor) * k, 0.0)
        }
        MODE_SPRING => {
            let (tension, friction) = ph2d_node_motion_spring::law::physics_of(
                ph2d_node_motion_spring::law::MODE_TIME,
                0.0,
                0.0,
                duration,
                bounce,
            );
            ph2d_node_motion_spring::law::passo(valor, vel, alvo, tension, friction, dt)
        }
        _ => {
            let maximo = dt / duration;
            let d = alvo - valor;
            if d.abs() <= maximo {
                (alvo, 0.0)
            } else {
                (valor + maximo.copysign(d), 0.0)
            }
        }
    }
}

fn coluna(s: &Stream, nome: &str) -> Option<Vec<f32>> {
    match s.get(nome) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

struct ValueFollow;

impl NodeOp for ValueFollow {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let alvos = coluna(ctx.input(0), VALUE_COL).unwrap_or_else(|| vec![0.0]);
        let n = alvos.len();
        let mode = ctx.param(MODE).round() as i32;
        let duration = ctx.param(DURATION);
        let bounce = ctx.param(BOUNCE);
        let dt = (ctx.dt() as f32).clamp(0.0, ph2d_node_motion_spring::MAX_DT);
        let estado = ctx.input(1);
        let antes = match (
            coluna(estado, VALUE_COL),
            coluna(estado, VEL_COL),
            coluna(estado, ALVO_COL),
        ) {
            (Some(v), Some(w), Some(a)) if v.len() == n && w.len() == n && a.len() == n => {
                Some((v, w, a))
            }
            _ => None,
        };
        let (valores, vels): (Vec<f32>, Vec<f32>) = match antes {
            None => (alvos.clone(), vec![0.0; n]),
            Some((v, w, a)) => (0..n)
                .map(|i| passo(mode, duration, bounce, v[i], w[i], a[i], dt))
                .unzip(),
        };
        ctx.emit(
            Stream::new(n)
                .with(VALUE_COL, Column::Scalar(valores))
                .with(VEL_COL, Column::Scalar(vels))
                .with(ALVO_COL, Column::Scalar(alvos)),
        );
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueFollow))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_key: "node.value.follow.name",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamUnit, ParamUnitDecl};

static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: DURATION,
    unit: ParamUnit::Seconds,
}];

static PARAM_GATES: &[ParamGate] = &[ParamGate {
    param: BOUNCE,
    when: MODE,
    values: &[MODE_SPRING],
}];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: MODE,
        label: "node.value.follow.param.mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.value.follow.param.mode.0",
                "node.value.follow.param.mode.1",
                "node.value.follow.param.mode.2",
            ],
        },
    },
    ParamUiHint {
        param: DURATION,
        label: "node.value.follow.param.duration",
        min: 0.0,
        max: 3.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: BOUNCE,
        label: "node.value.follow.param.bounce",
        min: -0.9,
        max: 0.9,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
