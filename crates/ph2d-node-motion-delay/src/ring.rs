//! **The delay line — and it follows the ELEMENT, not the row.**
//!
//! The past `MAX_LAG` ticks of every element's position, carried as plain columns on the node's
//! own output (the sequential-node convention: the `pre` self-loop hands them back next tick).
//! `slot(k)` holds the position from **k ticks ago**, so a lookback of `k` is a column read, not a
//! search. Slot 0 does not exist: it is the live input.
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
/// allocate an unbounded ring.
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

/// The name of the slot holding the position `k` ticks ago (`1 ..= MAX_LAG`).
pub(crate) fn slot(k: usize) -> &'static str {
    SLOTS[k.clamp(1, MAX_LAG) - 1] // CLAMP-OK: the ring's own bounds, 1-based
}

/// Whether `name` is one of this node's own state columns — the caller strips them before
/// rewriting, so they never accumulate stale duplicates.
pub(crate) fn is_state(name: &str) -> bool {
    name == PREV_OUT || SLOTS.contains(&name)
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

/// One Vec2 column of the state, gathered onto the live rows. A row with no past (or a state
/// column that is missing / the wrong length) reads as its own LIVE value: the line seeds **flat**
/// rather than pairing unrelated elements, so the delay forms over the next `ticks` ticks instead
/// of snapping to nonsense.
fn gather(state: &Stream, name: &str, rows: &[Option<usize>], live: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let col = match state.get(name) {
        Some(Column::Vec2(v)) => Some(v),
        _ => None,
    };
    rows.iter()
        .enumerate()
        .map(|(i, r)| {
            match (col, r) {
                (Some(v), Some(j)) => v.get(*j).copied().unwrap_or(live[i]),
                _ => live[i], // newborn, or no such column yet: it starts where it is
            }
        })
        .collect()
}

/// The past positions, `[k - 1] = k ticks ago`, gathered onto the live rows.
pub(crate) fn past(
    state: &Stream,
    rows: &[Option<usize>],
    live: &[[f32; 2]],
) -> Vec<Vec<[f32; 2]>> {
    (1..=MAX_LAG)
        .map(|k| gather(state, slot(k), rows, live))
        .collect()
}

/// The node's previous output, gathered onto the live rows (a newborn's is its live position, so
/// the one-pole starts settled instead of easing in from a stranger's place).
pub(crate) fn prev_out(state: &Stream, rows: &[Option<usize>], live: &[[f32; 2]]) -> Vec<[f32; 2]> {
    gather(state, PREV_OUT, rows, live)
}

/// Advance the line one tick and write it (plus this tick's output) onto `out`: the live positions
/// become "1 tick ago" and every other slot shifts one deeper — the oldest falls off the end.
pub(crate) fn push(
    out: &mut Stream,
    past: Vec<Vec<[f32; 2]>>,
    live: &[[f32; 2]],
    emitted: &[[f32; 2]],
) {
    let mut shifted = live.to_vec();
    for (k, mut older) in past.into_iter().enumerate().take(MAX_LAG) {
        // `shifted` enters holding what belongs in slot k+1; swap it out and carry the slot's
        // previous contents into the next, deeper slot.
        std::mem::swap(&mut shifted, &mut older);
        out.set(slot(k + 1), Column::Vec2(older));
    }
    out.set(PREV_OUT, Column::Vec2(emitted.to_vec()));
}
