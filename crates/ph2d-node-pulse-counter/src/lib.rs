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
    outputs: &[
        PortSpec {
            name: "out",
            ty: VALUE,
        },
        // **O CARRY** (folha 12 §P1) — APENDADO, e é a saída que faz contadores
        // COMPOREM: `beat → counter(4) → carry → counter(4)` é um divisor de
        // relógio, e sem ela um contador é um beco sem saída. Um pulso, então o
        // domínio é o mesmo do `pulse`, não o do valor.
        PortSpec {
            name: "carry",
            ty: PULSE,
        },
    ],
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
        // Quanto uma batida avança a contagem (o `Increment` do TD Count CHOP).
        // APENDADO, default **1** ⇒ o mundo de hoje byte a byte. ⚠️ **Inteiro, e
        // pode ser NEGATIVO** — uma contagem de 2,5 não é uma contagem (o doc do
        // módulo diz *"a persistent INTEGER value"*), e contar PARA TRÁS é a
        // capacidade de verdade que este param compra.
        ParamSpec {
            name: "step",
            default: 1.0,
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

/// Did the pulse RISE this tick? The one place the edge is decided — o carry e o
/// avanço perguntam à mesma função, senão *"a contagem andou"* teria duas respostas.
fn rising(pulse: f32, prev_pulse: f32) -> bool {
    pulse > 0.5 && prev_pulse <= 0.5
}

/// The monotonic tick after this frame: `+step` only on the pulse's rising edge.
///
/// ⚠️ `step` é inteiro e pode ser negativo; com o default `1.0` esta é a expressão
/// que sempre shipou, **ao bit**.
fn advance_tick(pulse: f32, prev_pulse: f32, prev_tick: f32, step: f32) -> f32 {
    if rising(pulse, prev_pulse) {
        prev_tick + step
    } else {
        prev_tick
    }
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
        // ⚠️ O piso do `Clamp` é NOVO e **byte-idêntico hoje**: antes do `step` o tique
        // só sabia crescer a partir de zero (o avanço era `+1` e o `reset_to` já era
        // clampado em `>= 0`), então `clamp(0, n-1)` e `min(n-1)` concordam em TODO
        // tique alcançável. Ele existe porque um `step` negativo torna o tique negativo
        // alcançável, e um `Clamp` sem piso mostraria uma contagem NEGATIVA — a mesma
        // razão que o comentário do `reset_to` já nomeava. A representação apaga o caso
        // especial em vez de o gatear.
        LimitMode::Clamp => tick.clamp(0, n - 1),
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

/// O PERÍODO do ciclo deste modo — quantos tiques crus levam a contagem exibida de volta ao
/// ponto de partida. `None` quando o modo **não tem ciclo**.
///
/// ⚠️ **O `Clamp` não tem, e isso é a verdade e não uma lacuna:** ele PARA. Uma contagem que
/// para nunca completa uma volta, então nunca há o que carregar — e encadear um Clamp como
/// divisor é justamente o que não funciona. Dizer `None` aqui é o nó recusando um sinal que
/// mentiria, em vez de inventar um.
fn cycle_period(n: i64, mode: LimitMode) -> Option<i64> {
    let n = n.max(1);
    match mode {
        LimitMode::Wrap => Some(n),
        LimitMode::Clamp => None,
        // A ida E a volta: o zigzag só volta ao ponto de partida depois das duas pernas.
        LimitMode::Zigzag => (n > 1).then(|| 2 * (n - 1)),
    }
}

/// **A LEI DO CARRY, numa frase:** num tique em que a contagem AVANÇOU, o carry dispara se a
/// trajetória crua **cruzou uma fronteira de CICLO** — `div_euclid` do tique mudou.
///
/// É o carry clássico, e é ele que faz `beat → counter(4) → carry → counter(4)` dividir por
/// dezasseis. Cai certo em cada modo sem um `if` por modo: **Wrap** dispara a cada `n`
/// batidas · **Zigzag** a cada volta COMPLETA (ida e volta — a dobra do meio é uma dobra, não
/// um ciclo) · **Clamp** nunca, porque não tem ciclo.
///
/// ⚠️ **A primeira versão desta lei era OUTRA e estava ERRADA:** *"o carry dispara se a
/// contagem exibida não andou os `step`"*. Ela lê bem, dá a resposta certa no Wrap, e no
/// **Zigzag dispara em TODA batida da perna descendente** — porque descer é a trajetória
/// legítima daquele modo, não uma intervenção. O gate do zigzag foi escrito com a expectativa
/// certa e a lei errada, e foi ele que a derrubou. *Uma lei que "cai certo em cada modo" tem
/// de ser conferida CONTRA cada modo, não deduzida de um.*
///
/// ⚠️ **A cláusula do RESET é a única que carrega peso, e a outra foi REMOVIDA por medição.**
/// A primeira versão também exigia *"a contagem avançou"*, e a mutação que a apaga
/// **sobreviveu à suíte inteira** — corretamente: sob a lei do ciclo um tique quieto tem
/// `tick == prev_tick`, logo o `div_euclid` não muda e o carry já sai mudo pela aritmética. Um
/// guarda que a álgebra torna inalcançável é uma **segunda resposta** a *"a contagem andou?"*,
/// não uma defesa — e o doc-comment que o chamava de *"não decoração"* estava a afirmar o
/// contrário do que a mutação media. O **reset** é outra história: ele salta a contagem para
/// um lugar arbitrário, e um salto ATRAVESSA fronteiras de ciclo — sem esta cláusula um reset
/// que cruze uma delas acende o carry por acidente.
fn carry_fired(reset: bool, prev_tick: i64, tick: i64, n: i64, mode: LimitMode) -> bool {
    !reset && cycle_period(n, mode).is_some_and(|p| tick.div_euclid(p) != prev_tick.div_euclid(p))
}

/// A saída do nó: o valor exibido e o carry, os dois alinhados linha a linha.
struct Counted {
    value: Stream,
    carry: Stream,
}

fn step(
    pulse: &Stream,
    state: &Stream,
    reset: &Stream,
    count_max: i64,
    mode: LimitMode,
    reset_to: f32,
    step_by: f32,
) -> Counted {
    let n = pulse.count();
    let pulses = scalar_col(pulse, PULSE_COL, n);
    let prev_tick = scalar_col(state, TICK_COL, n);
    let prev_pulse = scalar_col(state, PREV_COL, n);
    // Porta desconectada ⇒ stream vazio ⇒ zeros ⇒ nenhum reset. O neutro cai do
    // helper que já existia; não há ramo "se conectado".
    let resets = scalar_col(reset, PULSE_COL, n);

    let mut tick = Vec::with_capacity(n);
    let mut value = Vec::with_capacity(n);
    let mut carry = Vec::with_capacity(n);
    for i in 0..n {
        let counted = advance_tick(pulses[i], prev_pulse[i], prev_tick[i], step_by);
        let did_reset = resets[i] > 0.5;
        let t = tick_after_reset(resets[i], reset_to, counted);
        let shown = displayed(t as i64, count_max, mode);
        tick.push(t);
        value.push(shown as f32);
        carry.push(f32::from(u8::from(carry_fired(
            did_reset,
            prev_tick[i] as i64,
            t as i64,
            count_max,
            mode,
        ))));
    }
    Counted {
        value: Stream::new(n)
            .with(VALUE_COL, Column::Scalar(value))
            .with(TICK_COL, Column::Scalar(tick))
            // This tick's pulse becomes next tick's `prev` (the edge memory).
            .with(PREV_COL, Column::Scalar(pulses)),
        carry: Stream::new(n).with(PULSE_COL, Column::Scalar(carry)),
    }
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
        // Inteiro, mas SEM piso: contar para trás é a capacidade, e o `Clamp` ganhou o
        // piso próprio para que um tique negativo tenha resposta em vez de caso especial.
        let step_by = ctx.param("step").round();
        let out = step(
            ctx.input(0),
            ctx.input(1),
            ctx.input(2),
            count_max,
            mode,
            reset_to,
            step_by,
        );
        // DUAS saídas, na ordem do manifesto (`EvalCtx::emit`: *"call once per output
        // port, in order"*). ⚠️ Este é o PRIMEIRO nó do repo com duas — o motor sempre
        // as suportou (`Cook::cur_output` indexa a porta de origem) e o painel sempre
        // desenhou `outputs.len()` sockets; o que não existia era um nó a exercitá-las.
        ctx.emit(out.value);
        ctx.emit(out.carry);
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
    // ⚠️ O teto DIGITÁVEL da contagem — outra grandeza que o curso da mão.
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// **O TETO DIGITÁVEL da contagem — e ele existe porque o slider NÃO é um limite.**
///
/// ⚠️ **Isto é um defeito, não uma feature** (folha 12 linha 45, §0 do `CLAUDE.md`):
/// o slider parava em `32` e **não havia `ParamHardMax`**, então o bridge caía no
/// `hint.max` e uma contagem maior que 32 era **indigitável** — num nó cuja
/// referência (TD/Max `counter`) conta sem teto nenhum. O `32` nunca teve medição
/// atrás: ele era o curso confortável da mão, promovido a limite por omissão.
///
/// O teto real é o do TIPO: a contagem vive num `i64` no `displayed` e num `f32` no
/// stream, e `1e6` está muito abaixo dos `2²⁴` onde um `f32` deixa de contar
/// inteiros um a um. O curso do slider **não muda**.
static PARAM_HARD_MAX: &[ph2d_node_registry::ParamHardMax] = &[ph2d_node_registry::ParamHardMax {
    param: "count_max",
    max: 1_000_000.0,
}];

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
    // A faixa confortável do arrasto vai dos dois sentidos: o lado negativo É metade da
    // capacidade que este param compra, e um slider que só sobe a esconderia.
    ParamUiHint {
        param: "step",
        label: "Increment",
        min: -8.0,
        max: 8.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
