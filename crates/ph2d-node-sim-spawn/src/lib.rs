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

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{
    ColumnAccess, ColumnBinding, GpuKernel, ID_WRAP, ROWS_COL, SourceWindow, StreamOp,
};
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

/// The jitter lanes (independent draws off the same id).
const LANE_SLOT: u32 = 7;

/// Where the **pulse-born** half of the id space starts. The two kinds of birth are numbered by
/// two different clocks (the birth ordinal and the tick ordinal), so they must not share a range
/// — a rate-born and a pulse-born wearing the same id would be paired as ONE element by every
/// state node downstream (`motion.integrate`, `motion.spring`, `motion.delay` all key on `id`).
const PULSE_ID_BASE: u32 = ID_WRAP / 2;

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
        if name == "id" {
            continue; // the newborn's identity is its birth ordinal, not the template's
        }
        out.set(name.clone(), gather(col, rows));
    }
    out.set(
        "id",
        Column::Scalar(ids.iter().map(|k| *k as f32).collect()),
    );
    out
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

/// The GPU kernel (ADR-0136, `StreamOp::SourceRows`): output element `i` IS
/// newborn `window_first + i`. The kernel writes the newborn's identity and the
/// TEMPLATE ROW it is born from ([`ROWS_COL`]); the sequencer then gathers every
/// other template column at those rows — the newborn inherits the whole
/// vocabulary without this kernel enumerating a single column, exactly like the
/// CPU's [`newborns`].
///
/// The id wraps at [`ID_WRAP`] — and so does the CPU's, at the SAME single
/// point (`eval` wraps the ordinal before slotting and stamping), so both sides
/// hash and write the same number. `window_first` arrives already wrapped (the
/// count law's `f64` arithmetic, the emitter's pattern — ADR-0130).
///
/// [`slot`]'s expressions, verbatim: the scatter draw is the same avalanche
/// hash (`sp_hash3`, bit-exact in u32), `u32(draw · n) % n` is Rust's
/// `as usize % n`, round-robin is `id % n`. `rate` is NOT a kernel param — only
/// the count law (host-side, `f64`) reads it.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let sp_id = (params.window_first + i) % 16777216u;\n\
        var sp_row: u32 = 0u;\n\
        if (params.window_src_n > 0u) {\n\
        \x20   if (params.scatter >= 0.5) {\n\
        \x20       let sp_draw = sp_rand01(u32(max(params.seed, 0.0)), sp_id, 7u);\n\
        \x20       sp_row = u32(sp_draw * f32(params.window_src_n)) % params.window_src_n;\n\
        \x20   } else {\n\
        \x20       sp_row = sp_id % params.window_src_n;\n\
        \x20   }\n\
        }\n\
        write_cp_rows(i, f32(sp_row));\n\
        write_id(i, f32(sp_id));\n",
    wgsl_lib: "\
        fn sp_hash3(a: u32, b: u32, lane: u32) -> f32 {\n\
            var h: u32 = a * 0x9e3779b9u + b * 0x85ebca6bu + lane * 0xc2b2ae35u;\n\
            h = h ^ (h >> 16u);\n\
            h = h * 0x7feb352du;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x846ca68bu;\n\
            h = h ^ (h >> 16u);\n\
            return f32(h >> 8u) / f32(16777216u);\n\
        }\n\
        fn sp_rand01(seed: u32, id: u32, lane: u32) -> f32 {\n\
            return sp_hash3(seed, id, lane);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: ROWS_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "id",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        // **The pulse port's refusal** (ADR-0127 D3). This kernel has no lane for an event
        // column, and a pulse-born element is born at the row that FIRED — arithmetic the
        // device cannot reach without a prefix scan it was never given. Absent column ⇒ the
        // plan claims the node exactly as it always has (the device path this wave ships is
        // byte-identical); present ⇒ the frame recedes to the CPU, which is where the pulse
        // family already lives. The alternative is a device answer with every pulse-birth
        // silently MISSING, and nothing on screen to say so.
        ColumnBinding {
            column: PULSE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::RefuseIfPresent,
            identity: [0.0; 4],
            port: 1,
        },
    ],
    params: &["scatter", "seed"],
    // The SAME `born_in` the CPU `eval` runs, in the same `f64` — this is why
    // `CountLawCtx` carries `dt` (ADR-0136). The window's `first` is wrapped
    // here, once, in integer arithmetic (the emitter's rule: the kernel is TOLD
    // the window, never re-derives it).
    count_law: Some(|c| {
        let rate = (c.param)("rate") as f64;
        let born = born_in(rate, c.playhead, c.dt);
        SourceWindow {
            count: born.len(),
            first: born.start % ID_WRAP,
            age_first: 0.0,
        }
    }),
    variant_by_param: None,
    applicable: None,
};

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
        let mut ids: Vec<u32> = born_in(rate, ctx.playhead(), ctx.dt())
            .map(|k| k % span)
            .collect();
        let mut rows: Vec<usize> = ids.iter().map(|id| slot(*id, n, scatter, seed)).collect();
        let (pulse_ids, pulse_rows) =
            pulse_born(ctx.input(1), n, burst, ctx.playhead(), ctx.dt());
        ids.extend(pulse_ids);
        rows.extend(pulse_rows);
        let out = newborns(ctx.input(0), &ids, &rows);
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
];

/// The typed ceiling: one tick may not emit more than [`MAX_PER_TICK`] elements, so a burst
/// larger than that is a number the node cannot honour.
static PARAM_HARD_MAX: &[ph2d_node_registry::ParamHardMax] = &[ph2d_node_registry::ParamHardMax {
    param: "burst",
    max: MAX_PER_TICK as f32,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
