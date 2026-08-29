//! **The delay line — and it follows the ELEMENT, not the row.**
//!
//! The past `MAX_LAG` ticks of every element's *channel value*, carried as plain columns on the
//! node's own output (the sequential-node convention: the `pre` self-loop hands them back next
//! tick). `slot(k)` holds the value from **k ticks ago**, so a lookback of `k` is a column read,
//! not a search. Slot 0 does not exist: it is the live input.
//!
//! ## The line is a buffer of FLOATS, not of positions
//!
//! Everything here works on `dim` floats per element, because **the filter does not care what it
//! is lagging**: a position is two floats, an angle is one, a colour is four, and lerp / boxcar /
//! one-pole are the same arithmetic per component. The channel decides `dim` and which column the
//! caller reads and writes; the ring never learns the difference.
//!
//! ## Why this ring is not `motion.slit_scan`'s
//!
//! Slit-scan's ring pairs row *i* of the state with row *i* of the live set, and **re-seeds the
//! whole line whenever the count changes**. That is fine for a grid, and **useless inside a
//! simulation zone**: a particle system spawns and culls on almost every tick, so the count changes
//! constantly, the line re-seeds constantly, and the node quietly becomes a no-op — green, wired,
//! and doing nothing.
//!
//! So this one matches by **`id`** when the stream has one (every zone stream does — `sim.spawn`
//! mints it). A newborn has no past, so its whole line seeds flat at where it is: it starts
//! un-delayed rather than inheriting a stranger's history. An element that died simply stops being
//! asked about.
//!
//! With **no `id` column** (a plain grid, a distribution — a set whose rows *are* its identity) it
//! falls back to matching by row, with slit-scan's count-change re-seed. Both worlds, one node.

use ph2d_nodegraph::attr::{Column, Stream};
use std::collections::BTreeMap;

/// The deepest lookback, in ticks. At 60 Hz this is half a second, and it **bounds the state a
/// document can ask for** — the `ticks` param is clamped to it, so a hand-edited value cannot
/// allocate an unbounded ring. The line costs `MAX_LAG · dim` floats per element, so the widest
/// channel (Colour, `dim = 4`) is twice the cost of the default one.
pub(crate) const MAX_LAG: usize = 32;

/// The column names of the delay line, indexed by `k - 1`. Spelled out rather than formatted per
/// tick: the names are part of the stream's SHAPE, and a `format!` here would allocate a `String`
/// for every slot on every cook.
const SLOTS: [&str; MAX_LAG] = [
    "dl_1", "dl_2", "dl_3", "dl_4", "dl_5", "dl_6", "dl_7", "dl_8", "dl_9", "dl_10", "dl_11",
    "dl_12", "dl_13", "dl_14", "dl_15", "dl_16", "dl_17", "dl_18", "dl_19", "dl_20", "dl_21",
    "dl_22", "dl_23", "dl_24", "dl_25", "dl_26", "dl_27", "dl_28", "dl_29", "dl_30", "dl_31",
    "dl_32",
];

/// The node's own PREVIOUS OUTPUT, carried for the one-pole (Blend) mode.
///
/// A one-pole is a weighted sum of *all* past inputs, so it cannot be read off a 32-slot ring
/// without truncating it — and the truncation is not small: with a time constant of 32 ticks the
/// tail that falls off the end is still **36%** of the answer. Carrying the previous output makes
/// the recurrence exact and costs one column.
const PREV_OUT: &str = "dl_out";

/// **A VELOCIDADE que este nó emitiu no tique anterior** — o estado do teto de
/// aceleração, e só dele.
///
/// ⚠️ **Ela é GUARDADA e não derivada de duas saídas passadas.** Derivá-la
/// pediria uma segunda linha de `PREV_OUT`, e as duas responderiam *"a que
/// velocidade isto ia?"* — a segunda resposta divergiria no primeiro tique em
/// que alguém mexesse no teto, que é precisamente quando a pergunta importa. Ela
/// só é escrita com um teto ARMADO: sem ele a coluna não existe e o stream de
/// estado é o de sempre, byte a byte.
const PREV_VEL: &str = "dl_vel";

/// **Which channel this line is the history OF.**
///
/// A line is only history if it is history of the *same quantity*. Without this tag, switching the
/// channel would hand the old line to the new one and the type would not object — Position and
/// Size are both `Vec2`, X and Rotation are both `Scalar` — so the sprites would inherit a
/// position line as their scale and explode. Carried per element like every other column (it is a
/// property of the LINE, so any row answers; the first one is read).
const CHANNEL_TAG: &str = "dl_ch";

/// The name of the slot holding the value `k` ticks ago (`1 ..= MAX_LAG`).
pub(crate) fn slot(k: usize) -> &'static str {
    SLOTS[k.clamp(1, MAX_LAG) - 1] // CLAMP-OK: the ring's own bounds, 1-based
}

/// Whether `name` is one of this node's own state columns — the caller strips them before
/// rewriting, so they never accumulate stale duplicates.
pub(crate) fn is_state(name: &str) -> bool {
    name == PREV_OUT || name == PREV_VEL || name == CHANNEL_TAG || SLOTS.contains(&name)
}

/// Whether `state` is the history of `channel`. A line built for another channel is **not** old
/// history, it is someone else's — the caller treats it as absent and seeds flat.
pub(crate) fn is_for_channel(state: &Stream, channel: i32) -> bool {
    match state.get(CHANNEL_TAG) {
        Some(Column::Scalar(v)) => v.first().map(|c| c.round() as i32) == Some(channel),
        _ => false,
    }
}

/// A column flattened to `w` floats per element. The one place the ring meets a `Column`.
pub(crate) fn flat(col: &Column) -> (Vec<f32>, usize) {
    match col {
        Column::Scalar(v) => (v.clone(), 1),
        Column::Vec2(v) => (v.iter().flatten().copied().collect(), 2),
        Column::Vec3(v) => (v.iter().flatten().copied().collect(), 3),
        Column::Vec4(v) => (v.iter().flatten().copied().collect(), 4),
    }
}

/// The inverse of [`flat`]: `w` floats per element back into the column variant of that width.
pub(crate) fn unflat(v: Vec<f32>, w: usize) -> Column {
    match w {
        1 => Column::Scalar(v),
        2 => Column::Vec2(v.as_chunks::<2>().0.iter().map(|c| [c[0], c[1]]).collect()),
        3 => Column::Vec3(
            v.as_chunks::<3>()
                .0
                .iter()
                .map(|c| [c[0], c[1], c[2]])
                .collect(),
        ),
        _ => Column::Vec4(
            v.as_chunks::<4>()
                .0
                .iter()
                .map(|c| [c[0], c[1], c[2], c[3]])
                .collect(),
        ),
    }
}

/// Where each LIVE row's history lives in the state stream — `None` for an element the state has
/// never seen (a newborn).
///
/// By `id` when both sides have one; by row otherwise, and then only if the counts still agree
/// (a set that was rebuilt under an order-based ring has no honest pairing at all).
pub(crate) fn rows_of(state: &Stream, live: &Stream) -> Vec<Option<usize>> {
    let n = live.count();
    match (state.get("id"), live.get("id")) {
        (Some(Column::Scalar(prev)), Some(Column::Scalar(now))) => {
            // `f32` is not `Ord` (NaN), so key on the BITS — an id is an identity, never a
            // quantity, and two elements are the same one exactly when their ids are the same
            // number.
            let mut at: BTreeMap<u32, usize> = BTreeMap::new();
            for (i, d) in prev.iter().enumerate() {
                at.insert(d.to_bits(), i);
            }
            now.iter().map(|d| at.get(&d.to_bits()).copied()).collect()
        }
        _ if state.count() == n => (0..n).map(Some).collect(),
        _ => vec![None; n],
    }
}

/// One column of the state, gathered onto the live rows as `dim` floats each. A row with no past
/// (or a state column that is missing / the wrong width / short) reads as its own LIVE value: the
/// line seeds **flat** rather than pairing unrelated elements, so the delay forms over the next
/// `ticks` ticks instead of snapping to nonsense.
fn gather(
    state: &Stream,
    name: &str,
    rows: &[Option<usize>],
    live: &[f32],
    dim: usize,
) -> Vec<f32> {
    let col = match state.get(name).map(flat) {
        Some((v, w)) if w == dim => Some(v),
        _ => None,
    };
    let mut out = Vec::with_capacity(rows.len() * dim);
    for (i, r) in rows.iter().enumerate() {
        for k in 0..dim {
            let mine = live.get(i * dim + k).copied().unwrap_or(0.0);
            let past = match (&col, r) {
                (Some(v), Some(j)) => v.get(j * dim + k).copied().unwrap_or(mine),
                _ => mine, // newborn, or no such column yet: it starts where it is
            };
            out.push(past);
        }
    }
    out
}

/// The past values, `[k - 1] = k ticks ago`, gathered onto the live rows.
pub(crate) fn past(
    state: &Stream,
    rows: &[Option<usize>],
    live: &[f32],
    dim: usize,
) -> Vec<Vec<f32>> {
    (1..=MAX_LAG)
        .map(|k| gather(state, slot(k), rows, live, dim))
        .collect()
}

/// The node's previous output, gathered onto the live rows (a newborn's is its live value, so the
/// one-pole starts settled instead of easing in from a stranger's place).
pub(crate) fn prev_out(
    state: &Stream,
    rows: &[Option<usize>],
    live: &[f32],
    dim: usize,
) -> Vec<f32> {
    gather(state, PREV_OUT, rows, live, dim)
}

/// A velocidade que este nó emitiu no tique anterior, reunida nas linhas vivas.
///
/// ⚠️ **O recém-nascido entra com velocidade ZERO, não com a dele** — ao
/// contrário de [`prev_out`], cujo fallback é o valor vivo (um recém-nascido já
/// está onde está). Uma velocidade herdada faria um elemento novo arrancar com o
/// impulso de um estranho, que é exatamente o que um teto de aceleração existe
/// para impedir.
pub(crate) fn prev_vel(state: &Stream, rows: &[Option<usize>], len: usize, dim: usize) -> Vec<f32> {
    let zeros = vec![0.0; len];
    gather(state, PREV_VEL, rows, &zeros, dim)
}

/// Escreve a velocidade emitida — chamado **só com um teto armado**.
pub(crate) fn push_vel(out: &mut Stream, vel: Vec<f32>, dim: usize) {
    out.set(PREV_VEL, unflat(vel, dim));
}

/// Advance the line one tick and write it (plus this tick's output and the channel it belongs to)
/// onto `out`: the live values become "1 tick ago" and every other slot shifts one deeper — the
/// oldest falls off the end.
pub(crate) fn push(
    out: &mut Stream,
    past: Vec<Vec<f32>>,
    live: &[f32],
    emitted: &[f32],
    dim: usize,
    channel: i32,
) {
    let mut shifted = live.to_vec();
    for (k, mut older) in past.into_iter().enumerate().take(MAX_LAG) {
        // `shifted` enters holding what belongs in slot k+1; swap it out and carry the slot's
        // previous contents into the next, deeper slot.
        std::mem::swap(&mut shifted, &mut older);
        out.set(slot(k + 1), unflat(older, dim));
    }
    out.set(PREV_OUT, unflat(emitted.to_vec(), dim));
    // `dim` is 1, 2 or 4 — every arm of the channel table names one, so there is no zero to guard.
    out.set(
        CHANNEL_TAG,
        Column::Scalar(vec![channel as f32; live.len() / dim]),
    );
}
