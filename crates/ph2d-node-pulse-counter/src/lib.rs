//! `pulse.counter` — the PURE reducer: a pulse train → a persistent integer
//! **value** (Motion Nodes M2, the value domain — doc 12). The name doc 09 §4.2
//! deliberately left free: `motion.step` is the visible *behaviour* (it pushes a
//! transform channel per beat); this is the abstract *reducer* the mature tools
//! ship (TouchDesigner **Count CHOP**, Max **counter**) — it emits a **value**
//! on its own socket and **never touches a channel**. Route its value into a
//! channel with `motion.drive`, so `beat → pulse.counter → motion.drive`
//! composes the same thing `motion.step` bundles — plus the value can fan out to
//! *several* drives (one count, many channels), which the bundled node cannot.
//!
//! **The value type** is the continuous per-instance field
//! `(Instances, Scalar, Frame)` carried on the `v` column — the same type the
//! `debug.const`/`debug.wave` chain uses, and the continuous dual of the
//! `(…, Event)` pulse. A value is ALWAYS a per-instance field; a "global" value
//! is just a length-1 field (the reference convergence — TD/Houdini/vvvv/Faust:
//! a constant is the degenerate field, so there is no separate global-scalar
//! type to double the node set). See doc 12.
//!
//! **The count math is `motion.step`'s, verbatim** (it was always the reducer at
//! heart): a monotonic per-instance `count_tick` rides the `pre` self-loop
//! (+1 only on the pulse's rising edge — edge-safe, TD `Off to On` vs `While
//! On`), and the *displayed* count is derived each tick from the tick + limit
//! mode: **Wrap** `tick mod N`, **Clamp** `min(tick, N-1)`, **Zigzag** a triangle
//! of period `2(N-1)`. Euclidean integer modulo (HR-5). What changed vs
//! `motion.step`: no `channel`/`step` params, no channel write — the displayed
//! count rides out as the `v` value column (plus `count_tick`/`count_prev` on the
//! `pre` loop). Positional per-instance (v1), matching the pulse family.
//!
//! **A porta `reset`** (folha 12 §P1; o `Reset` do TD Count CHOP) é o que torna o
//! estado ALCANÇÁVEL — sem ela, uma linha que deixa de receber pulsos (porque um
//! campo a montante se moveu, ou porque um param mudou em runtime) congela a
//! contagem para sempre, e nada consegue trazê-la de volta. Semântica de NÍVEL,
//! `reset_to` como destino, e o reset ganha de uma contagem simultânea (ver
//! [`tick_after_reset`]). ⚠️ **Desconectada, ela é o mundo anterior BYTE A BYTE** —
//! não há caso especial: um stream vazio vira zeros no `scalar_col` que já existia.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The pulse type — a discrete per-instance event `(Instances, Scalar, Event)`
/// (mirror of `ph2d_node_pulse_beat::PULSE`; kept local so this crate is a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);
/// The value type — a continuous per-instance scalar field `(Instances, Scalar,
/// Frame)`, carried on the `v` column (the `debug.*` convention). The continuous
/// dual of `PULSE`; its `Frame` clock keeps it from connecting to a pulse port.
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The pulse stream's fire column (`1.0` on a fired tick).
const PULSE_COL: &str = "pulse";
/// The value output column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// The monotonic tick carried on the `pre` self-loop (+1 per rising edge).
const TICK_COL: &str = "count_tick";
/// Last tick's pulse value, carried on the `pre` self-loop for edge detection.
const PREV_COL: &str = "count_prev";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("pulse.counter"),
    name: "pulse.counter",
    inputs: &[
        PortSpec {
            name: "pulse",
            ty: PULSE,
        },
        // Feedback: last tick's value output carries `count_tick`+`count_prev`.
        // Named `state` so the editor plumbs its `pre` self-loop on drop.
        PortSpec {
            name: "state",
            ty: VALUE,
        },
        // **A porta que torna o estado ALCANÇÁVEL** (folha 12 §P1; TD Count CHOP
        // `Reset`). Opcional: desconectada, `EvalCtx::input(2)` é um stream vazio,
        // `scalar_col` a preenche com `0.0`, e nada nunca passa de `0.5` ⇒ o mundo
        // de hoje BYTE A BYTE, sem caso especial nenhum.
        PortSpec {
            name: "reset",
            ty: PULSE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Pure: the tick enters the fingerprint through the consumed `pre` edge.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // Distinct counts in the cycle (the wrap/zigzag period). Clamped ≥1.
        ParamSpec {
            name: "count_max",
            default: 6.0,
        },
        // 0 Wrap · 1 Clamp · 2 Zigzag — the TD Count CHOP limit-mode vocabulary.
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // Para onde o `reset` leva o tique (TD Count CHOP `Reset Value`). `0` = o
        // começo, e é o default ⇒ quem não liga a porta não vê diferença nenhuma.
        ParamSpec {
            name: "reset_to",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// What happens to the count at the top of its range (TD Count CHOP limit modes).
#[derive(Copy, Clone, PartialEq, Eq)]
enum LimitMode {
    /// `tick mod N` — the staircase returns home (TD Loop Min/Max).
    Wrap,
    /// `min(tick, N-1)` — the staircase plateaus at the top (TD Clamp).
    Clamp,
    /// A triangle of period `2(N-1)` — up then down (TD Zigzag).
    Zigzag,
}

impl LimitMode {
    fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => LimitMode::Clamp,
            2 => LimitMode::Zigzag,
            _ => LimitMode::Wrap,
        }
    }
}

/// The monotonic tick after this frame: +1 only on the pulse's rising edge.
fn advance_tick(pulse: f32, prev_pulse: f32, prev_tick: f32) -> f32 {
    let rising = pulse > 0.5 && prev_pulse <= 0.5;
    if rising { prev_tick + 1.0 } else { prev_tick }
}

/// **A PORTA ÚNICA do reset** — o tique DEPOIS de considerar o `reset` desta linha.
///
/// Semântica de **NÍVEL**, não de borda (o `Reset` do TD Count CHOP: *enquanto* está alto,
/// a contagem é segurada no valor de reset). Duas consequências que são o desenho:
///
/// - **Nenhuma coluna de estado nova.** Uma borda exigiria lembrar o `reset` do tique
///   anterior no `pre`; o nível não exige nada — *menos estado para consertar um problema
///   de estado*. E um pulso dura um tique por contrato, então resetar-no-pulso continua
///   sendo o que o artista vê.
/// - **O reset GANHA de uma contagem simultânea.** É a decisão do TD, e é a única que
///   torna o estado alcançável de forma confiável: se a contagem ganhasse, uma linha que
///   recebe pulso e reset no mesmo tique ficaria em 1 para sempre, e o artista não teria
///   como zerá-la sem acertar um vão entre dois pulsos.
///
/// ⚠️ **Esta lei é a MESMA do `motion.step::advance_tick`, e isso não é coincidência: é o
/// ancestral.** O contador saiu dele (*"the count math is `motion.step`'s, verbatim"*) e
/// **não levou o reset junto** — o `motion.step` já shipava a porta, com o mesmo nível, o
/// mesmo `reset_to` e a mesma regra de quem ganha o tique. As duas cópias existem porque
/// cada nó é uma **leaf drop-crate** (o vocabulário compartilhado é a PORTA, nunca um
/// símbolo — a mesma razão de `PULSE` ser redeclarado aqui); **elas têm de se mover
/// juntas**, e é por isso que este parágrafo nomeia a outra.
fn tick_after_reset(reset: f32, reset_to: f32, counted: f32) -> f32 {
    if reset > 0.5 { reset_to } else { counted }
}

/// The displayed count for a monotonic `tick`, folded by the limit mode. Integer
/// arithmetic only (Euclidean modulo) — exact and transcendental-free (HR-5).
fn displayed(tick: i64, n: i64, mode: LimitMode) -> i64 {
    let n = n.max(1);
    match mode {
        LimitMode::Wrap => tick.rem_euclid(n),
        LimitMode::Clamp => tick.min(n - 1),
        LimitMode::Zigzag => {
            if n == 1 {
                return 0;
            }
            let period = 2 * (n - 1);
            let m = tick.rem_euclid(period);
            if m < n { m } else { period - m }
        }
    }
}

fn scalar_col(s: &Stream, name: &str, n: usize) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, 0.0);
    v
}

fn step(
    pulse: &Stream,
    state: &Stream,
    reset: &Stream,
    count_max: i64,
    mode: LimitMode,
    reset_to: f32,
) -> Stream {
    let n = pulse.count();
    let pulses = scalar_col(pulse, PULSE_COL, n);
    let prev_tick = scalar_col(state, TICK_COL, n);
    let prev_pulse = scalar_col(state, PREV_COL, n);
    // Porta desconectada ⇒ stream vazio ⇒ zeros ⇒ nenhum reset. O neutro cai do
    // helper que já existia; não há ramo "se conectado".
    let resets = scalar_col(reset, PULSE_COL, n);

    let mut tick = Vec::with_capacity(n);
    let mut value = Vec::with_capacity(n);
    for i in 0..n {
        let counted = advance_tick(pulses[i], prev_pulse[i], prev_tick[i]);
        let t = tick_after_reset(resets[i], reset_to, counted);
        tick.push(t);
        value.push(displayed(t as i64, count_max, mode) as f32);
    }
    Stream::new(n)
        .with(VALUE_COL, Column::Scalar(value))
        .with(TICK_COL, Column::Scalar(tick))
        // This tick's pulse becomes next tick's `prev` (the edge memory).
        .with(PREV_COL, Column::Scalar(pulses))
}

struct PulseCounter;

impl NodeOp for PulseCounter {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count_max = (ctx.param("count_max").round() as i64).max(1);
        let mode = LimitMode::from_param(ctx.param("mode"));
        // Uma contagem é uma contagem: o valor de reset é inteiro e não-negativo
        // (um `Clamp` sobre tique negativo mostraria uma contagem negativa).
        let reset_to = ctx.param("reset_to").round().max(0.0);
        let out = step(
            ctx.input(0),
            ctx.input(1),
            ctx.input(2),
            count_max,
            mode,
            reset_to,
        );
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(PulseCounter))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Counter",
            // Utility grey: a pulse→value reducer, plumbing (not a visible transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count_max",
        label: "Count",
        min: 1.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Wrap", "Clamp", "Zigzag"],
        },
    },
    ParamUiHint {
        param: "reset_to",
        label: "Reset To",
        min: 0.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    fn fire(v: f32) -> Stream {
        Stream::new(1).with(PULSE_COL, Column::Scalar(vec![v]))
    }
    /// Um tique SEM reset — o que todo gate anterior a esta wave exercita. A porta
    /// desconectada é um `Stream` vazio, exatamente o que o `EvalCtx` entrega.
    fn tick(pulse: &Stream, state: &Stream, count_max: i64, mode: LimitMode) -> Stream {
        step(pulse, state, &Stream::new(0), count_max, mode, 0.0)
    }

    fn value(s: &Stream) -> f32 {
        match s.get(VALUE_COL).unwrap() {
            Column::Scalar(v) => v[0],
            _ => panic!(),
        }
    }

    /// FALSIFICATION of edge-safety: a pulse HELD high advances the count ONCE
    /// (on the rising edge), not once per tick. Counting `pulse > 0.5` every tick
    /// would reach 5 after a 5-tick hold.
    #[test]
    fn a_held_pulse_counts_once_not_once_per_tick() {
        let mut state = Stream::new(1);
        for _ in 0..5 {
            state = tick(&fire(1.0), &state, 16, LimitMode::Wrap);
        }
        assert_eq!(value(&state), 1.0, "one rising edge = one count, not five");
        state = tick(&fire(0.0), &state, 16, LimitMode::Wrap);
        state = tick(&fire(1.0), &state, 16, LimitMode::Wrap);
        assert_eq!(value(&state), 2.0, "the next rising edge counts once more");
    }

    /// Um tique COM reset, para os gates da porta nova.
    fn tick_reset(
        pulse: &Stream,
        state: &Stream,
        reset: f32,
        count_max: i64,
        reset_to: f32,
    ) -> Stream {
        step(
            pulse,
            state,
            &fire(reset),
            count_max,
            LimitMode::Wrap,
            reset_to,
        )
    }

    /// **A porta desconectada é o mundo ANTERIOR, byte a byte.** O neutro não é uma
    /// promessa em prosa: a sequência com a porta vazia tem de bater, elemento a
    /// elemento, com a mesma corrida em que o reset nunca sobe.
    #[test]
    fn an_unconnected_reset_is_the_world_before_it() {
        let run = |with_port: bool| {
            let mut state = Stream::new(1);
            let mut seq = Vec::new();
            for k in 0..12 {
                let p = fire(if k % 2 == 0 { 1.0 } else { 0.0 });
                state = if with_port {
                    tick_reset(&p, &state, 0.0, 4, 0.0)
                } else {
                    tick(&p, &state, 4, LimitMode::Wrap)
                };
                seq.push(value(&state));
            }
            seq
        };
        assert_eq!(
            run(true),
            run(false),
            "porta desconectada e reset em zero descrevem a MESMA contagem"
        );
    }

    /// **O reset traz a contagem para casa** — a capacidade inteira da wave: uma linha
    /// que acumulou pode voltar ao começo sem que o documento seja reconstruído.
    #[test]
    fn the_reset_returns_the_count_home() {
        let mut state = Stream::new(1);
        for _ in 0..3 {
            state = tick_reset(&fire(1.0), &state, 0.0, 16, 0.0);
            state = tick_reset(&fire(0.0), &state, 0.0, 16, 0.0);
        }
        assert_eq!(value(&state), 3.0, "três bordas, três contagens");
        state = tick_reset(&fire(0.0), &state, 1.0, 16, 0.0);
        assert_eq!(value(&state), 0.0, "o reset devolve a contagem ao começo");
        // …e ela volta a contar dali, em vez de ficar presa em zero.
        state = tick_reset(&fire(1.0), &state, 0.0, 16, 0.0);
        assert_eq!(
            value(&state),
            1.0,
            "e a contagem segue viva depois do reset"
        );
    }

    /// **O reset GANHA de uma contagem simultânea** — a decisão do TD, pinada. Se a
    /// contagem ganhasse, uma linha que recebe pulso e reset no mesmo tique nunca
    /// poderia ser zerada por um sinal que chega junto com o metrônomo.
    #[test]
    fn the_reset_wins_a_simultaneous_count() {
        let mut state = Stream::new(1);
        for _ in 0..4 {
            state = tick_reset(&fire(1.0), &state, 0.0, 16, 0.0);
            state = tick_reset(&fire(0.0), &state, 0.0, 16, 0.0);
        }
        assert_eq!(value(&state), 4.0);
        // Pulso E reset no MESMO tique (a borda subiria de 4 para 5).
        state = tick_reset(&fire(1.0), &state, 1.0, 16, 0.0);
        assert_eq!(value(&state), 0.0, "o reset ganha da borda simultânea");
    }

    /// **`reset_to` é para ONDE o reset leva** (TD Count CHOP `Reset Value`), e ele
    /// atravessa o dobramento do modo como qualquer outro tique.
    #[test]
    fn the_reset_lands_on_the_authored_value() {
        let mut state = Stream::new(1);
        state = tick_reset(&fire(1.0), &state, 0.0, 6, 3.0);
        assert_eq!(value(&state), 1.0);
        state = tick_reset(&fire(0.0), &state, 1.0, 6, 3.0);
        assert_eq!(value(&state), 3.0, "o reset pousa no valor autorado");
    }

    /// The reducer emits a VALUE and NEVER a transform channel — the whole point
    /// of the pure split. The output stream carries `v` (+ the state columns) and
    /// no `P`/`rot`/`size`.
    #[test]
    fn it_emits_a_value_column_and_no_transform_channel() {
        let s = tick(&fire(1.0), &Stream::new(1), 6, LimitMode::Wrap);
        assert!(s.get(VALUE_COL).is_some(), "emits the value column");
        assert!(s.get("P").is_none(), "no position");
        assert!(
            s.get("rot").is_none() && s.get("size").is_none(),
            "no channel"
        );
    }

    /// Wrap / Clamp / Zigzag fold the same monotonic tick three ways (TD Count
    /// CHOP limit modes), as a VALUE sequence.
    #[test]
    fn the_three_limit_modes_fold_the_count() {
        let run = |count_max, mode| {
            let mut state = Stream::new(1);
            let mut seq = Vec::new();
            for _ in 0..8 {
                state = tick(&fire(1.0), &state, count_max, mode);
                seq.push(value(&state));
                state = tick(&fire(0.0), &state, count_max, mode);
            }
            seq
        };
        assert_eq!(
            run(4, LimitMode::Wrap),
            vec![1.0, 2.0, 3.0, 0.0, 1.0, 2.0, 3.0, 0.0],
            "wrap returns home"
        );
        assert_eq!(
            run(3, LimitMode::Clamp),
            vec![1.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0],
            "clamp plateaus at N-1"
        );
        assert_eq!(
            run(4, LimitMode::Zigzag),
            vec![1.0, 2.0, 3.0, 2.0, 1.0, 0.0, 1.0, 2.0],
            "zigzag ping-pongs"
        );
    }

    /// `count_max = 1` never divides by zero and stays home (value 0).
    #[test]
    fn count_max_one_stays_home_without_dividing_by_zero() {
        let mut state = Stream::new(1);
        for _ in 0..5 {
            state = tick(&fire(1.0), &state, 1, LimitMode::Zigzag);
            state = tick(&fire(0.0), &state, 1, LimitMode::Zigzag);
        }
        assert_eq!(value(&state), 0.0);
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
