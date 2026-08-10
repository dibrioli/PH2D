#![forbid(unsafe_code)]
//! `pulse.adsr` — **um disparo vira uma CURVA** (doc 89, folha 12, §2.4 · doc 63 §2.4).
//!
//! O TouchDesigner descreve o **Trigger CHOP** como *"starts an audio-style ADSR envelope
//! to all trigger pulses"*, e era a última coisa da família que não existia aqui: o
//! `motion.strobe` **é** um envelope, mas escreve canal de **transform**, então usá-lo como
//! valor seria exatamente o *clock hack* que o doc 09 matou (`strobe → value.attribute`).
//! Este nó vive no **domínio de VALOR** desde o tipo da porta.
//!
//! ## O gatilho não tem *note off*, e é isso que decide o desenho
//!
//! Num sintetizador o envelope é dirigido por um **portão**: a tecla desce (A→D→S) e a
//! tecla sobe (R). O nosso pulso é um **impulso** — `1.0` por um tique, sem duração — e a
//! diferença não é cosmética: um ADSR dirigido por portão precisaria de um nível
//! sustentado, e **`pulse.level` é momentâneo e sem estado por decisão** (folha 12, a P0),
//! então não existe no catálogo nada que segure o portão aberto. ⇒ o envelope é um
//! **one-shot**: o pulso o começa e o param **`hold`** diz quanto tempo o sustain dura
//! antes de soltar. É o AHDSR de caixa de ritmo, e é a única forma que **compõe** com o que
//! a família já produz (`pulse.beat`, `pulse.threshold`, o `carry` do `pulse.counter`).
//!
//! Um segundo porto *release* daria o modelo de teclado — e seria uma **segunda** resposta
//! para *"quando este envelope termina?"*, com dois caminhos capazes de discordar. Fica
//! nomeado aqui em vez de construído.
//!
//! ## A forma das rampas é ALGÉBRICA (HR-5)
//!
//! `attack_shape` / `release_shape` são o *bias* de Schlick — `u / ((1/b − 2)(1 − u) + 1)`
//! —, quatro operações e nenhuma transcendental. Em **`0.5` ele reduz LITERALMENTE à
//! identidade** (`(2 − 2)(1 − u) + 1 == 1`), então uma curva linear não é uma aproximação de
//! linear: é a mesma aritmética. Abaixo de 0,5 a rampa sai rápida e assenta (côncava);
//! acima, sai lenta e acelera (convexa).
//!
//! ⚠️ **Dois shapes, não três.** O TD tem um por segmento; aqui o `release_shape` governa
//! **decay E release**, porque os dois são o mesmo gesto — uma queda — e um terceiro knob
//! compraria a diferença entre *cair para o sustain* e *cair para zero*, que nenhum uso
//! deste catálogo distingue. Se um dia distinguir, o param entra apendado e o default
//! preserva o mundo.
//!
//! `Pure`: o tique entra na impressão digital pela aresta `pre` consumida.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// O tipo do pulso — evento discreto por instância (espelho local: esta é uma
/// folha drop-in, e o vocabulário compartilhado é a PORTA, nunca um símbolo).
pub const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);
/// O tipo do valor — campo escalar contínuo por instância, na coluna `v`.
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// A coluna de disparo de um stream de pulso.
const PULSE_COL: &str = "pulse";
/// A coluna canônica do domínio de valor.
const VALUE_COL: &str = "v";
/// Segundos desde o disparo, no `pre` self-loop.
const AGE_COL: &str = "adsr_age";
/// `1.0` enquanto um envelope está correndo, no `pre` self-loop. ⚠️ **Ele é
/// necessário e não é redundante com a idade:** `age = 0` parado e `age = 0`
/// recém-disparado emitem o mesmo número e divergem no tique seguinte. Com os
/// dois em zero — que é o que uma coluna AUSENTE lê — o estado é *parado*, então
/// um grafo recém-montado não dispara envelope nenhum.
const ON_COL: &str = "adsr_on";

/// O contrato estático deste tipo de nó (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("pulse.adsr"),
    name: "pulse.adsr",
    inputs: &[
        PortSpec {
            name: "pulse",
            ty: PULSE,
        },
        // Feedback: a saída do tique anterior carrega `adsr_age`+`adsr_on`.
        // Chamado `state` para o editor plumbar o `pre` self-loop ao soltar o nó.
        PortSpec {
            name: "state",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Pure: o tique entra na impressão digital pela aresta `pre` consumida.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // O atraso do TD entre o disparo e o começo do ataque.
        ParamSpec {
            name: "delay",
            default: 0.0,
        },
        ParamSpec {
            name: "attack",
            default: 0.05,
        },
        ParamSpec {
            name: "decay",
            default: 0.10,
        },
        // O NÍVEL do sustain (0..1), não uma duração — o único param da lista que
        // não é tempo, e a razão de o `hold` existir ao lado dele.
        ParamSpec {
            name: "sustain",
            default: 0.70,
        },
        // Quanto o sustain DURA. Um gatilho não tem *note off* (ver o cabeçalho),
        // então é este número que fecha o envelope.
        ParamSpec {
            name: "hold",
            default: 0.20,
        },
        ParamSpec {
            name: "release",
            default: 0.30,
        },
        // 0.5 = reta EXATA. O *bias* de Schlick, algébrico (HR-5).
        ParamSpec {
            name: "attack_shape",
            default: 0.5,
        },
        // Governa decay E release — as duas quedas (ver o cabeçalho).
        ParamSpec {
            name: "release_shape",
            default: 0.5,
        },
        // O `Retrigger` do TD: um pulso no meio do envelope o reinicia (1) ou é
        // ignorado (0). Reiniciar é o default musical.
        ParamSpec {
            name: "retrigger",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Tudo o que o artista autorou, em segundos e níveis.
#[derive(Copy, Clone)]
struct Envelope {
    delay: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    hold: f32,
    release: f32,
    attack_shape: f32,
    release_shape: f32,
    retrigger: bool,
}

impl Envelope {
    /// Quanto tempo o envelope inteiro dura. Depois disto ele está parado.
    fn total(&self) -> f32 {
        self.delay + self.attack + self.decay + self.hold + self.release
    }

    /// O nível em `age` segundos desde o disparo. Função **PURA** da idade: é isto
    /// que torna o scrub exato e o envelope idêntico em qualquer taxa de quadros.
    fn level(&self, age: f32) -> f32 {
        let mut t = age - self.delay;
        if t < 0.0 {
            return 0.0;
        }
        // ATAQUE: 0 → 1.
        if t < self.attack {
            return bias(t / self.attack, self.attack_shape);
        }
        t -= self.attack;
        // DECAY: 1 → sustain.
        if t < self.decay {
            let u = bias(t / self.decay, self.release_shape);
            return 1.0 + (self.sustain - 1.0) * u;
        }
        t -= self.decay;
        // SUSTAIN: o patamar, pelo tempo que o `hold` disser.
        if t < self.hold {
            return self.sustain;
        }
        t -= self.hold;
        // RELEASE: sustain → 0.
        if t < self.release {
            let u = bias(t / self.release, self.release_shape);
            return self.sustain * (1.0 - u);
        }
        0.0
    }
}

/// O *bias* de Schlick: `u / ((1/b − 2)(1 − u) + 1)`, com `b = 0.5` reduzindo à
/// IDENTIDADE por aritmética (`(2 − 2)(1 − u) + 1 == 1`), não por aproximação.
///
/// ⚠️ `b` é preso longe de `0` e `1` porque `1/b` explode num extremo e o
/// denominador vai a zero no outro — os dois produziriam `inf`/`NaN` num número
/// que sai direto para a tela.
fn bias(u: f32, b: f32) -> f32 {
    // ⚠️ `f32::clamp` devolve **NaN** quando o próprio valor é NaN — ele prende a
    // FAIXA, não a sanidade —, então um param ilegível atravessaria os dois clamps
    // e sairia como NaN direto para a tela. O não-finito cai no NEUTRO, a mesma lei
    // do fallback de `direction` no `pulse.on_change`.
    let u = if u.is_finite() {
        u.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let b = if b.is_finite() {
        b.clamp(0.001, 0.999)
    } else {
        0.5
    };
    u / ((1.0 / b - 2.0) * (1.0 - u) + 1.0)
}

fn scalar_col(s: &Stream, name: &str, n: usize) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, 0.0);
    v
}

/// Um tique do envelope, por instância. O comprimento espelha o do pulso — cada
/// linha carrega o próprio envelope, então um campo de N dispara N deles.
fn step(pulse: &Stream, state: &Stream, env: Envelope, dt: f32) -> Stream {
    let n = pulse.count();
    let fires = scalar_col(pulse, PULSE_COL, n);
    let prev_age = scalar_col(state, AGE_COL, n);
    let prev_on = scalar_col(state, ON_COL, n);
    let total = env.total();

    let mut out = Vec::with_capacity(n);
    let mut age = Vec::with_capacity(n);
    let mut on = Vec::with_capacity(n);
    for i in 0..n {
        let running = prev_on[i] > 0.5;
        let triggered = fires[i] > 0.5;
        // O disparo zera a idade. No meio de um envelope isso só acontece com
        // `retrigger` — senão o pulso é ignorado e o envelope corrente termina.
        let (mut a, mut r) = if triggered && (!running || env.retrigger) {
            (0.0, true)
        } else if running {
            (prev_age[i] + dt, true)
        } else {
            (0.0, false)
        };
        if r && a >= total {
            // Acabou: volta ao estado parado, que é o mesmo par de zeros de um
            // grafo recém-montado.
            a = 0.0;
            r = false;
        }
        out.push(if r { env.level(a) } else { 0.0 });
        age.push(a);
        on.push(if r { 1.0 } else { 0.0 });
    }
    Stream::new(n)
        .with(VALUE_COL, Column::Scalar(out))
        .with(AGE_COL, Column::Scalar(age))
        .with(ON_COL, Column::Scalar(on))
}

struct PulseAdsr;

impl NodeOp for PulseAdsr {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let env = Envelope {
            delay: ctx.param("delay").max(0.0),
            attack: ctx.param("attack").max(0.0),
            decay: ctx.param("decay").max(0.0),
            sustain: ctx.param("sustain").clamp(0.0, 1.0),
            hold: ctx.param("hold").max(0.0),
            release: ctx.param("release").max(0.0),
            attack_shape: ctx.param("attack_shape"),
            release_shape: ctx.param("release_shape"),
            retrigger: ctx.param("retrigger") > 0.5,
        };
        let out = step(ctx.input(0), ctx.input(1), env, ctx.dt() as f32);
        ctx.emit(out);
    }
}

/// Registra este nó no registry de runtime. Chamado (por codegen) do
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(PulseAdsr))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "ADSR",
            // Cinza de utilidade: um adaptador pulso→valor, não uma transformação
            // visível — a mesma categoria dos irmãos que cruzam a membrana.
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    Ok(())
}

use ph2d_node_registry::{
    ParamGroup, ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
};

/// Três perguntas, em vez de uma parede de nove sliders (doc 88): *que forma tem o
/// envelope no tempo* · *que curva têm as rampas* · *o que um disparo faz com um
/// envelope que já corre*.
static PARAM_GROUPS: &[ParamGroup] = &[
    ParamGroup::new("delay", "Envelope"),
    ParamGroup::new("attack", "Envelope"),
    ParamGroup::new("decay", "Envelope"),
    ParamGroup::new("sustain", "Envelope"),
    ParamGroup::new("hold", "Envelope"),
    ParamGroup::new("release", "Envelope"),
    ParamGroup::new("attack_shape", "Shape"),
    ParamGroup::new("release_shape", "Shape"),
    ParamGroup::new("retrigger", "Trigger"),
];

static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "delay",
        unit: ParamUnit::Seconds,
    },
    ParamUnitDecl {
        param: "attack",
        unit: ParamUnit::Seconds,
    },
    ParamUnitDecl {
        param: "decay",
        unit: ParamUnit::Seconds,
    },
    ParamUnitDecl {
        param: "sustain",
        unit: ParamUnit::Ratio,
    },
    ParamUnitDecl {
        param: "hold",
        unit: ParamUnit::Seconds,
    },
    ParamUnitDecl {
        param: "release",
        unit: ParamUnit::Seconds,
    },
];

/// O teto DIGITÁVEL dos tempos. O recurso é o mesmo do `debounce` do
/// `pulse.threshold` — **precisão de representação**: a idade vive em `f32` e é
/// incrementada por `dt`, então acima de certa magnitude `age + dt == age` e o
/// envelope **para de andar**, congelado no nível em que estava.
///
/// ⚠️ **E o número NÃO é o do `debounce`, embora a grandeza e o relógio sejam os
/// mesmos** — copiá-lo teria shipado um teto quebrado. Aquele conta para BAIXO e
/// este para CIMA, e numa potência de dois os vizinhos de baixo estão **duas vezes
/// mais juntos** que os de cima: em `2^17` a subtração de um tique de 240 fps ainda
/// morde e a soma **já não morde**. Medido, o último que ainda AVANÇA a 240 fps é
/// **65536 s** (18 horas), uma oitava abaixo do teto do irmão, e o gate afirma as
/// duas metades (avança · o dobro não).
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "delay",
        max: 65_536.0,
    },
    ParamHardMax {
        param: "attack",
        max: 65_536.0,
    },
    ParamHardMax {
        param: "decay",
        max: 65_536.0,
    },
    ParamHardMax {
        param: "hold",
        max: 65_536.0,
    },
    ParamHardMax {
        param: "release",
        max: 65_536.0,
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "delay",
        label: "Delay",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "attack",
        label: "Attack",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "decay",
        label: "Decay",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "sustain",
        label: "Sustain",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "hold",
        label: "Hold",
        min: 0.0,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "release",
        label: "Release",
        min: 0.0,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "attack_shape",
        label: "Attack Shape",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "release_shape",
        label: "Release Shape",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "retrigger",
        label: "Retrigger",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
