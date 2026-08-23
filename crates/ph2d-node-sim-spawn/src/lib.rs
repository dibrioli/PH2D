#![forbid(unsafe_code)]
//! **`sim.spawn`** — BIRTH inside a simulation zone (Motion Nodes O4, doc 49).
//!
//! ## The hole the zone left
//!
//! A zone gave the module *life* (`sim.step` over live state) and *death* (`motion.cull`, which
//! inside a zone stops being a filter and becomes a kill — what dies stays dead). It could not
//! give it *birth*, and a simulation that cannot be born is a population that can only shrink:
//! the rain demo thinned out and, given long enough, emptied.
//!
//! `motion.emitter` is no help here, and deliberately so: it is **stateless** — the live set is
//! a pure function of the playhead — so merging it into a zone's state every tick would merge
//! the same particles again and again. What a zone needs is a node that answers a much smaller
//! question: **who was born THIS tick?**
//!
//! ## One node, one question
//!
//! `sim.spawn` emits *only the newborns of this tick* — usually none, sometimes one, sometimes
//! three. It does not merge them into anything: **`motion.combine` does that**, and that is the
//! whole design.
//!
//! ```text
//!   zone ⊙─→ combine(in0) ─→ force.wind ─→ sim.step ─→ falloff ─→ cull ─→ state
//!   grid ──→ sim.spawn ────→ combine(in1)
//! ```
//!
//! Split that way, birth is composable with everything the library already has: the newborns
//! inherit **every column of the template element they are born from** (`P`, `vel`, `size`,
//! `tint`, …), so you aim, colour and size the birth with ordinary nodes upstream of the spawn,
//! and nothing about that vocabulary had to be invented here.
//!
//! ## Identity is the birth ordinal — so a scrub reproduces it exactly
//!
//! The `k`-th element ever born gets `id = k`, and `k` is derived from the CLOCK
//! (`floor(rate · t)`), not from a counter the node keeps. So it is a pure function of the
//! playhead: rewind, re-cook, and the same particles come back with the same ids, the same
//! template slots and the same jitter (`hash(seed, id, lane)` — the emitter's stateless
//! randomness, Jarzynski & Olano 2020; transcendental-free, HR-5).
//!
//! A counter in a state column would have been the obvious alternative, and it would have made
//! the ids depend on the *history* of the cook rather than on the clock — so a scrub would
//! renumber the world.
//!
//! ## Fractional rates do not leak
//!
//! Births come from the DIFFERENCE of two floors — `floor(rate·t) − floor(rate·(t−dt))` — so a
//! rate of 7/s at 60 fps emits nothing on most ticks and one particle on some, and over a second
//! it has emitted exactly 7. Rounding `rate·dt` per tick instead (the obvious way) would round
//! 0.116 to 0 every single tick and emit **nothing, forever**.
//!
//! ## A PULSE gives birth (the `pulse` port)
//!
//! Until 2026-08-10 the library had events (`pulse.*`) and a simulation, and **nothing was ever
//! triggered by a pulse** — the P0 the conference's sheet 12 opened against this family. The
//! `pulse` port closes it: a template row that FIRES gives birth to `burst` elements, born at
//! that row, on that tick. The whole Niagara *Event Handler* gesture — *when this happens, make
//! that* — with one port and one number.
//!
//! The port is **APPENDED and its default REDUCES**: unconnected it cooks to an empty stream, so
//! no row ever fires and not one extra element is born. The world before this port, byte for
//! byte, with no special case written anywhere.
//!
//! **A pulse-born element still has an id that is a pure function of the clock**, which is the
//! property the whole node is built on (a scrub must not renumber the world). It cannot be the
//! birth ordinal — *how many pulses have fired so far* is HISTORY, and a counter is exactly what
//! the section above refuses — so it is the **TICK ordinal** instead: the elements a tick's
//! pulses give birth to take ids `PULSE_ID_BASE + (tick·PULSE_IDS_PER_TICK + j)`, wrapped. Same
//! clock, same guarantee, and the two kinds of birth live in **disjoint halves** of the id space
//! so a rate-born and a pulse-born can never collide.
//!
//! ⚠️ **The half is carved only when a pulse is actually wired** (the `pulse` COLUMN is present
//! on the port). Without one, rate-born ids keep the full `ID_WRAP` period they have always had
//! — which is what makes "unconnected = the world before it" true of the *ids* too, and not just
//! of the count.
//!
//! ⚠️ **The pulse path is CPU**, and it costs nothing that anybody had: none of the six `pulse.*`
//! nodes has a GPU kernel (they are events per LINE, not maps per texel), so the chain feeding
//! this port is already a device boundary. The refusal is declared where the plan can see it —
//! a `ColumnAccess::RefuseIfPresent` binding on port 1 (ADR-0127 D3) — because a kernel that
//! quietly ignored the port would answer the artist's graph with every pulse-birth MISSING and
//! nothing on screen to say so.

mod hash;
mod kernel;
mod trig;

use kernel::GPU_KERNEL;
use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ID_WRAP, StreamOp};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The event type of the `pulse` port (mirror of `ph2d_node_pulse_beat::PULSE`; kept local so
/// this crate stays a leaf drop-crate — the shared vocabulary is the PORT, never a symbol).
const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);
/// The column a pulse travels in (mirror of the pulse family's `PULSE_COL`, same reason).
const PULSE_COL: &str = "pulse";

/// The most newborns one tick may emit. A rate of 10 000/s on a dropped frame would otherwise
/// try to build a stream of thousands in a single tick, and the frame that was already late is
/// the worst possible moment to do it. (The births are not lost — the ordinal is a function of
/// the clock, so the next tick simply starts from where the cap left off… by NOT emitting them:
/// they are skipped, and this is the one place the model is lossy. Said out loud, in the one
/// place it can bite.)
const MAX_PER_TICK: u32 = 256;

/// A cadência do relógio a que este nó é cozido, em hertz — o espelho local de
/// `ph2d_core::DEFAULT_HZ`.
///
/// ⚠️ **Dito aqui e não importado, pela MESMA razão que o [`PULSE_COL`] acima**: esta crate é
/// uma folha e o vocabulário partilhado é a PORTA, nunca um símbolo. O preço do espelho é
/// poder divergir, e é por isso que ele tem gate (`the_locally_spelled_cook_rate_is_the_houses`,
/// que importa o `ph2d-core` só como dev-dependency).
const COOK_HZ: f32 = 60.0;

/// **O maior `rate` que este nó HONRA, em nascimentos por segundo.**
///
/// [`born_in`] emite no máximo [`MAX_PER_TICK`] por cozedura, e o app cozinha uma vez por
/// tique de [`COOK_HZ`] ⇒ `256 × 60 = 15 360/s`. Acima disto os nascimentos devidos não são
/// adiados: são **saltados**. É este o número que o campo digitável aceita, e não o fim do
/// arrasto.
const MAX_RATE: f32 = MAX_PER_TICK as f32 * COOK_HZ;

/// The jitter lanes (independent draws off the same id).
const LANE_SLOT: u32 = 7;

/// Where the **pulse-born** half of the id space starts. The two kinds of birth are numbered by
/// two different clocks (the birth ordinal and the tick ordinal), so they must not share a range
/// — a rate-born and a pulse-born wearing the same id would be paired as ONE element by every
/// state node downstream (`motion.integrate`, `motion.spring`, `motion.delay` all key on `id`).
const PULSE_ID_BASE: u32 = ID_WRAP / 2;

/// A pista de hash da direção do ESTOURO — própria, para não correlacionar com o sorteio
/// de slot do `slot()` (dois sorteios da mesma pista dão o mesmo número, e irmãs que
/// partilhassem direção com a escolha de template seriam um padrão, não um estouro).
const LANE_BURST: u32 = 23;

/// A pista do sorteio de SOBREVIVÊNCIA, pelo mesmo motivo que a do estouro é própria: dois
/// sorteios da mesma pista devolvem o MESMO número, então partilhá-la com o `slot()` faria
/// todo sobrevivente sair da mesma faixa do template — um padrão, não um filtro.
const LANE_PROB: u32 = 11;

/// Ids reserved per tick for the pulse-born. It is [`MAX_PER_TICK`] because that is the same
/// number said once: a tick may not EMIT more than this, so a tick cannot NEED more than this.
///
/// ⚠️ **This buys the uniqueness window, and here is its number:** `PULSE_ID_BASE /
/// PULSE_IDS_PER_TICK` = 32 768 ticks = **546 s at 60 fps**. Two pulse-born elements collide
/// only if one outlives the other by more than nine minutes — the same argument [`ID_WRAP`]
/// already makes for the rate-born (identity is read as a DIFFERENCE inside one window, orders
/// of magnitude narrower than the period), one order tighter, said with the number.
const PULSE_IDS_PER_TICK: u64 = MAX_PER_TICK as u64;

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("sim.spawn"),
    name: "sim.spawn",
    inputs: &[
        PortSpec {
            name: "template",
            ty: INST_VEC2,
        },
        // APPENDED, and the default REDUCES: unconnected cooks to an empty stream, so no row
        // fires and not one extra element is born — the world before this port, byte for byte.
        PortSpec {
            name: "pulse",
            ty: PULSE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Reads the clock (it IS a clock: births are a function of the playhead). Holds no state.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // Newborns per second. Fractional rates are exact over time (see the module docs).
        ParamSpec {
            name: "rate",
            default: 12.0,
        },
        // Which element of the template a newborn is born from: 0 = round-robin (an orderly
        // march through the template), 1 = a hashed draw (a scatter).
        ParamSpec {
            name: "scatter",
            default: 1.0,
        },
        ParamSpec {
            name: "seed",
            default: 1.0,
        },
        // How many elements a PULSING template row gives birth to, on the tick it fires.
        // APPENDED; inert while the `pulse` port is unconnected, which is what makes a default
        // of 1 safe: an artist who wires the port and touches nothing gets ONE element per
        // event, which is the least surprising thing an event can mean.
        ParamSpec {
            name: "burst",
            default: 1.0,
        },
        // **A velocidade com que as irmãs de um estouro se SEPARAM** (o `Add Velocity in
        // Cone` do Niagara, o `Initial Speed` da Cavalry). APENDADO; `0` é o mundo de antes,
        // byte a byte.
        //
        // ⚠️ **Sem ele a capacidade não existe, e isso é um FATO medido, não uma questão de
        // afinação:** `burst` filhas de UMA linha de template nascem com `P` e `vel`
        // IDÊNTICOS, e toda força deste catálogo é função da posição — `curl(P)` dá a duas
        // partículas no mesmo `P` exatamente a mesma aceleração. Duas irmãs assim são o
        // mesmo elemento em tudo o que se observa menos o `id`: **nenhum campo consegue
        // separá-las, nunca** (medido na cena `=27`: as duas ficaram bit-idênticas em P e
        // vel por 150 tiques). A simetria tem de ser quebrada no NASCIMENTO, e a única coisa
        // que difere entre irmãs é o id — que é justamente o que este impulso usa.
        //
        // ⚠️ **E isto REFUTA um veredito da folha 13** (*"velocidade inicial: SIM, e o
        // desenho é melhor — o recém-nascido herda toda coluna do template, então ela é
        // autorada a montante"*): verdade para o nascimento por TAXA, onde cada filho pega
        // uma LINHA distinta do template, e falsa para o nascimento por PULSO, onde N filhos
        // saem da MESMA linha. A folha foi escrita antes de a porta `pulse` existir.
        ParamSpec {
            name: "burst_speed",
            default: 0.0,
        },
        // **A chance de um nascimento DEVIDO de fato acontecer** (o `Probability` do Spawn Rate
        // e do Spawn Burst do Niagara, o `Probability` do Particle Emitter da Cavalry).
        // APENDADO; `1` é o mundo de antes, byte a byte — e sem sequer avaliar o hash.
        //
        // ⚠️ **Ele é um FILTRO sobre a lista de devidos, nunca um multiplicador do `rate`, e a
        // diferença é medida:** `born_in` calcula `floor(rate·t) − floor(rate·(t−dt))` com o
        // `rate` de AGORA nos DOIS termos, então mexer no `rate` não deixa de emitir — ele
        // **re-deriva a história** (subir pula ids; descer faz `last < first` e o `.max(first)`
        // emite zero em silêncio até o relógio alcançar).
        //
        // ⚠️ **E a cadeia que o catálogo oferecia é TUDO-OU-NADA** (sonda
        // `measure_spawn_probability`): `sim.spawn → value.instance_field(Random) →
        // motion.drive(Falloff) → motion.cull` mede **0,000 · 0,000 · 0,000 · 1,000** em quatro
        // seeds onde o alvo era 0,5, porque todo sorteio por-elemento do domínio de VALOR é
        // chaveado no **ÍNDICE DA LINHA** (`value.instance_field`: `rand01(seed, i)`) e **nenhum
        // tique emite mais de um nascimento** enquanto `rate ≤ 60`, que é o próprio teto do
        // slider (medido: 481 tiques com 0 e 119 com 1, a rate 12; 201 e 399, a rate 40) ⇒ o
        // índice é sempre 0 e o sorteio é uma CONSTANTE por seed. Filtrado pelo **id**, o mesmo
        // corte dá 0,437-0,555.
        ParamSpec {
            name: "probability",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// How many elements have EVER been born by time `t` — the one number the whole node is built
/// on, so both ends of a tick read it the same way.
///
/// **The epsilon is not decoration.** The previous tick's clock is reconstructed as `t - dt`,
/// and in f64 that is the previous playhead to within an ulp, not exactly. Without the nudge,
/// `floor` sits right on a birth boundary and *goes back*: the tick recomputes "births before
/// me" as one fewer than the last tick actually emitted, and **the same particle is born
/// twice** (the guard caught 97 births where 90 were due). A nanosecond of slack costs nothing
/// and is perfectly deterministic.
fn births_upto(rate: f64, t: f64) -> u32 {
    if rate <= 0.0 || t <= 0.0 {
        return 0;
    }
    (rate * t + BIRTH_EPS).floor().max(0.0) as u32
}

/// Slack on the birth boundary, in births. Far above f64's noise on a playhead of any plausible
/// size, far below one particle.
const BIRTH_EPS: f64 = 1e-6;

/// The ids born in `(t - dt, t]` at `rate` births per second — the difference of two totals, so
/// a fractional rate is exact over time and never rounds itself away.
///
/// `dt == 0` (the first tick of a cook, or a paused clock) means no time passed: nothing is
/// born. A sim that spawned on a paused playhead would breed while you stared at it.
fn born_in(rate: f64, t: f64, dt: f64) -> std::ops::Range<u32> {
    if rate <= 0.0 || dt <= 0.0 || t <= 0.0 {
        return 0..0;
    }
    let first = births_upto(rate, t - dt);
    let last = births_upto(rate, t);
    first..last.max(first).min(first.saturating_add(MAX_PER_TICK))
}

/// **Este nascimento devido acontece?** — a porta ÚNICA da probabilidade, perguntada pelo
/// `eval` (que constrói a lista) e pela lei de contagem da GPU (que a conta antes de o kernel
/// existir). Duas cópias divergiriam num número que ninguém lê: a contagem da janela.
///
/// ⚠️ **O sorteio é do `id`, e do id JÁ ENVOLVIDO** (pós-`% span`) — é o número que o `slot` e o
/// carimbo veem, então os três concordam sobre quem este elemento é. Sortear pelo ordinal cru
/// faria o filtro discordar do slot na volta do `ID_WRAP`.
///
/// ⚠️ **`>= 1.0` não avalia o hash**, e é isso que torna o default byte-idêntico ao mundo antes
/// deste param em vez de meramente equivalente. Do outro lado, `<= 0.0` cai sozinho: `rand01`
/// devolve `[0,1)` e nada é `< 0`, então nenhum nascimento acontece — sem caso especial.
fn survives(id: u32, seed: u32, probability: f32) -> bool {
    probability >= 1.0 || hash::rand01(seed, id, LANE_PROB) < probability
}

/// The template row a newborn is born from.
fn slot(id: u32, n: usize, scatter: bool, seed: u32) -> usize {
    if n == 0 {
        return 0;
    }
    if scatter {
        (hash::rand01(seed, id, LANE_SLOT) * n as f32) as usize % n
    } else {
        id as usize % n
    }
}

/// The tick ordinal — the clock the **pulse-born** are numbered by.
///
/// `dt` is the root clock's step, so `t / dt` IS the tick index, and it is a pure function of the
/// playhead exactly like the birth ordinal is. The `round` is against f64's last ulp, not against
/// a variable step: a step that varied would be the engine changing frame rate mid-cook, and the
/// only thing it could cost is two ticks landing in one slot — inside the 546-second window, on
/// the frame the rate changed.
fn pulse_tick(t: f64, dt: f64) -> u64 {
    if dt <= 0.0 || t <= 0.0 {
        return 0;
    }
    (t / dt).round().max(0.0) as u64
}

/// The ids and template rows of the elements this tick's PULSES give birth to.
///
/// A row that fires gives birth to `burst` elements **at that row** — not at a hashed slot, which
/// is the whole point of an event: the birth happens WHERE the thing happened. Rows are read in
/// ascending order and the count is capped by [`MAX_PER_TICK`], the same ceiling and for the same
/// reason as the rate-born (a frame that is already late is the worst moment to build thousands).
fn pulse_born(pulse: &Stream, n: usize, burst: u32, t: f64, dt: f64) -> (Vec<u32>, Vec<usize>) {
    let (mut ids, mut rows) = (Vec::new(), Vec::new());
    if burst == 0 || n == 0 || dt <= 0.0 {
        return (ids, rows);
    }
    let Some(Column::Scalar(fired)) = pulse.get(PULSE_COL) else {
        return (ids, rows);
    };
    let base = pulse_tick(t, dt).wrapping_mul(PULSE_IDS_PER_TICK);
    let mut j: u64 = 0;
    for row in 0..n {
        // A shorter pulse stream than the template reads as "did not fire" — the neutral the
        // whole family already uses for a missing row, so a mismatched wiring is quiet and
        // harmless instead of a panic on someone's frame.
        if fired.get(row).copied().unwrap_or(0.0) <= 0.5 {
            continue;
        }
        for _ in 0..burst {
            if j >= PULSE_IDS_PER_TICK {
                return (ids, rows);
            }
            ids.push(PULSE_ID_BASE + ((base + j) % u64::from(PULSE_ID_BASE)) as u32);
            rows.push(row);
            j += 1;
        }
    }
    (ids, rows)
}

/// Gather `rows` of `template` into a stream, one newborn per row, stamped with its `id`.
///
/// The newborn takes EVERY column of its template row — position, velocity, size, tint, whatever
/// the artist authored upstream — because a birth node that invented its own vocabulary would be
/// a second, poorer copy of the library.
///
/// It is stamped with `id` and with NOTHING else: no clock (`sim_t`), so `sim.step` sees an
/// element it has never stepped and starts it at `dt = 0` instead of hurling it forward by the
/// whole life of the sim.
///
/// ⚠️ **The rows arrive already chosen, and that is deliberate:** the rate-born pick theirs from
/// the birth ordinal ([`slot`]) and the pulse-born from the row that FIRED, and the two answers
/// meet here at ONE gather. Two gathers would be two places for "what does a newborn inherit?"
/// to be answered, and the second one is where a column would go missing.
fn newborns(template: &Stream, ids: &[u32], rows: &[usize]) -> Stream {
    debug_assert_eq!(ids.len(), rows.len(), "one row per newborn");
    let mut out = Stream::new(ids.len());
    for (name, col) in template.columns() {
        // The newborn's identity is its birth ordinal, not the template's.
        // And **a newborn is age 0 BY DEFINITION** — `sim.step` already says so in as many
        // words (*"A row with no `age` is newborn: identity 0"*), so leaving the column
        // absent hands it the law that already exists instead of writing a second one.
        //
        // ⚠️ **This is inert for every template that has no clock on it** (a `source.*` grid
        // never carries an `age`), and load-bearing for the one that does: the corpses on
        // `sim.lifetime.died`. Without it a death-born child inherits an age that is, by
        // construction, past its own lifetime — it dies on the very next tick, and the whole
        // replicate chain looks built and does nothing.
        if name == "id" || name == "age" {
            continue;
        }
        out.set(name.clone(), gather(col, rows));
    }
    out.set(
        "id",
        Column::Scalar(ids.iter().map(|k| *k as f32).collect()),
    );
    out
}

/// **O IMPULSO DE NASCIMENTO das irmãs de um estouro** — some `speed` na velocidade de cada
/// pulse-born, numa direção sorteada da PRÓPRIA identidade dela.
///
/// ⚠️ **É ADITIVO à velocidade herdada, e isso é o modelo:** uma faísca lançada de um foguete
/// que subia continua subindo *e* se abre. Substituir a herdada apagaria o movimento do pai,
/// que é justamente o que o `died` foi buscar.
///
/// ⚠️ **Só os pulse-born**, que são a cauda da lista (`rate_n..`): um rate-born pega uma LINHA
/// distinta do template, então ele já nasce separado dos irmãos e não tem simetria a quebrar.
/// É também o que mantém esta lei fora do caminho de GPU — o nascimento por pulso já recusa o
/// dispositivo por declaração, então não há um segundo motor para divergir deste.
///
/// ⚠️ **E a coluna `vel` pode não existir no template** (uma fonte `motion.grid` não a
/// carrega). Materializá-la em zeros é o neutro que o `sim.step` já usa (*"a row with no
/// `vel`"*), e sem isso um estouro a partir de uma fonte parada não faria nada — em silêncio.
fn burst_kick(out: &mut Stream, pulse_ids: &[u32], rate_n: usize, speed: f32, seed: u32) {
    if speed == 0.0 || pulse_ids.is_empty() {
        return;
    }
    let n = out.count();
    let mut vel = match out.get("vel") {
        Some(Column::Vec2(v)) if v.len() == n => v.clone(),
        _ => vec![[0.0, 0.0]; n],
    };
    for (j, id) in pulse_ids.iter().enumerate() {
        let Some(slot) = vel.get_mut(rate_n + j) else {
            break;
        };
        // A fase é o sorteio da própria filha: irmãs têm ids distintos por construção
        // (`PULSE_ID_BASE + base + j`), logo direções distintas.
        let (cx, sy) = trig::cos_sin_cycles(hash::rand01(seed, *id, LANE_BURST));
        slot[0] += cx * speed;
        slot[1] += sy * speed;
    }
    out.set("vel".to_string(), Column::Vec2(vel));
}

fn gather(col: &Column, rows: &[usize]) -> Column {
    fn take<T: Copy + Default>(v: &[T], rows: &[usize]) -> Vec<T> {
        rows.iter()
            .map(|&i| v.get(i).copied().unwrap_or_default())
            .collect()
    }
    match col {
        Column::Scalar(v) => Column::Scalar(take(v, rows)),
        Column::Vec2(v) => Column::Vec2(take(v, rows)),
        Column::Vec3(v) => Column::Vec3(take(v, rows)),
        Column::Vec4(v) => Column::Vec4(take(v, rows)),
    }
}

struct SimSpawn;

impl NodeOp for SimSpawn {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let rate = ctx.param("rate") as f64;
        let scatter = ctx.param("scatter") >= 0.5;
        let seed = ctx.param("seed").max(0.0) as u32;
        // `dt` comes from the ENGINE (`EvalCtx::dt` — doc 49): this node holds no state, so it
        // has no clock column of its own to subtract from, and it must not invent one.
        //
        // The ordinal WRAPS at `ID_WRAP` (ADR-0136, the audit's C3): the id is
        // stored as `f32`, whose integers collapse past 2²⁴ — at the millions-per-
        // second a big sim runs, that is seconds, not days (the contract's own
        // arithmetic on `SourceWindow`). Wrapped HERE, at the single point the
        // ordinal becomes observable, so the slot draw, the stamped id and the GPU
        // kernel (told a wrapped `window_first`) all see the same number. The wrap
        // is invisible to consumers: identity is only ever read as a DIFFERENCE
        // inside one window, orders of magnitude narrower than the period.
        //
        // ⚠️ The period HALVES while a pulse is wired, because the pulse-born take the other
        // half (`PULSE_ID_BASE`). The question is asked of the `pulse` COLUMN — the same fact
        // the GPU plan refuses on, so one wiring cannot mean two things to the two cooks.
        let burst = ctx.param("burst").round().max(0.0) as u32;
        let template = ctx.input(0);
        let n = template.count();
        let pulsing = ctx.input(1).get(PULSE_COL).is_some();
        let span = if pulsing { PULSE_ID_BASE } else { ID_WRAP };
        let probability = ctx.param("probability");
        let mut ids: Vec<u32> = born_in(rate, ctx.playhead(), ctx.dt())
            .map(|k| k % span)
            .filter(|id| survives(*id, seed, probability))
            .collect();
        let mut rows: Vec<usize> = ids.iter().map(|id| slot(*id, n, scatter, seed)).collect();
        // Onde termina o nascimento por TAXA e começa o por PULSO. O índice, e não uma
        // faixa de id: o `PULSE_ID_BASE` só carva a metade **quando um pulso está fiado**,
        // então `id >= base` não distingue os dois no caso geral.
        let rate_n = ids.len();
        let (pulse_ids, pulse_rows) = pulse_born(ctx.input(1), n, burst, ctx.playhead(), ctx.dt());
        // ⚠️ **A probabilidade alcança as irmãs de um estouro também**, e não é generosidade: a
        // referência a põe na tríade do burst (`Spawn Count`·`Spawn Time`·`Probability`, Niagara
        // §C.11), e é o que faz *"de cada dez faíscas, três pegam"*. Cada irmã tem id próprio por
        // construção (`PULSE_ID_BASE + base + j`), então o sorteio as separa em vez de decidir o
        // estouro inteiro de uma vez. Filtrado AQUI, e não dentro do `pulse_born`, para aquela
        // função seguir respondendo só *quem pulsou* — uma lei, um lugar.
        for (id, row) in pulse_ids.into_iter().zip(pulse_rows) {
            if survives(id, seed, probability) {
                ids.push(id);
                rows.push(row);
            }
        }
        let mut out = newborns(ctx.input(0), &ids, &rows);
        burst_kick(
            &mut out,
            &ids[rate_n..],
            rate_n,
            ctx.param("burst_speed"),
            seed,
        );
        ctx.emit(out);
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SimSpawn))?;
    // ADR-0136: the kernel mints rows + ids; the SourceRows machinery gathers
    // the template's columns. NOT `register_dense_window`: a spawn's output ids
    // are this tick's window only — downstream state pairing is the zone's job.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_stream_op(MANIFEST.id, StreamOp::SourceRows { port: 0 });
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Spawn",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_hard_min(MANIFEST.id, PARAM_HARD_MIN);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

/// Param UI hints. `scatter` is a BOOLEAN (round-robin vs a hashed draw), so it is a
/// checkbox, not a 0..1 slider; `seed` is a Seed box + re-roll, because an artist wants
/// ANOTHER seed, never a BIGGER one.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rate",
        label: "Rate",
        min: 0.0,
        max: 60.0,
        step: 0.5,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "scatter",
        label: "Scatter",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    // The comfortable drag; the typed ceiling is `MAX_PER_TICK`, because that is what the node
    // HONOURS — a box that accepted 5 000 over a cap of 256 would accept and lie (doc 88, B2).
    ParamUiHint {
        param: "burst",
        label: "Burst",
        min: 0.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "burst_speed",
        label: "Burst Speed",
        min: 0.0,
        // A faixa CONFORTÁVEL do arrasto (doc 88 B2), não um teto: acima disto o estouro
        // ainda é legível, só sai de quadro depressa. Sem `ParamHardMax` de propósito — o
        // recurso não existe (é uma velocidade somada, e a sim já capa nada).
        max: 8.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "probability",
        label: "Probability",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// The typed ceiling: one tick may not emit more than [`MAX_PER_TICK`] elements, so a burst
/// larger than that is a number the node cannot honour.
static PARAM_HARD_MAX: &[ph2d_node_registry::ParamHardMax] = &[
    ph2d_node_registry::ParamHardMax {
        param: "burst",
        max: MAX_PER_TICK as f32,
    },
    // ⚠️ **O `rate` não tinha teto nenhum, e o irmão ao lado tinha.** Sem entrada aqui o campo
    // digita até ao fim do ARRASTO (`ui.rs:206`) — **60/s** —, enquanto a lei honra
    // [`MAX_PER_TICK`] por tique. Duzentos e cinquenta e seis vezes mais era inalcançável, e a
    // faixa que o artista via descrevia a comodidade do dedo, não o nó.
    //
    // **De que recurso é este teto: da LEI, não da precisão** (`CLAUDE.md` §0.0) — e é por isso
    // que ele não entra na lista `PRECISION_BOUND` do gate irmão. [`born_in`] grampeia a janela
    // em `first + MAX_PER_TICK`, então acima de [`MAX_RATE`] os nascimentos **são saltados**
    // (o único ponto lossy do modelo, dito em voz alta no doc-comment de [`MAX_PER_TICK`]) e um
    // campo que aceitasse mais aceitaria e mentiria — a mesma lei do `burst` uma linha acima.
    ph2d_node_registry::ParamHardMax {
        param: "rate",
        max: MAX_RATE,
    },
    // ⚠️ **Este teto não é de recurso, é de SIGNIFICADO** — e é por isso que ele existe: acima de
    // 1 a [`survives`] devolve `true` para tudo, então uma caixa que aceitasse `5` **aceitaria e
    // mentiria** (o teto digitável não pode passar do que a lei HONRA — a lição da varredura do
    // doc 88, onde `lattice` aceitava 5.000 sobre um clamp de 400).
    ph2d_node_registry::ParamHardMax {
        param: "probability",
        max: 1.0,
    },
];

/// **O que o número É** (doc 88): uma probabilidade é a `Ratio` do vocabulário — *"uma fração
/// 0..1 ou um multiplicador simples"* —, e é o que a distingue, para quem lê a declaração, de um
/// `seed` ou de um `burst`. `rate` e `burst_speed` ficam bares de propósito: o vocabulário não
/// tem *por segundo* nem *distância por segundo*, e **uma unidade errada é pior que uma ausente**.
static PARAM_UNITS: &[ph2d_node_registry::ParamUnitDecl] = &[ph2d_node_registry::ParamUnitDecl {
    param: "probability",
    unit: ph2d_node_registry::ParamUnit::Ratio,
}];

/// O piso, pela MESMA razão do teto: abaixo de zero nada mais deixa de nascer — `rand01` devolve
/// `[0,1)` e nada é `< 0`, então `-3` e `0` são o mesmo mundo, e uma caixa que os distinguisse
/// estaria oferecendo uma escolha que não existe.
static PARAM_HARD_MIN: &[ph2d_node_registry::ParamHardMin] = &[ph2d_node_registry::ParamHardMin {
    param: "probability",
    min: 0.0,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
