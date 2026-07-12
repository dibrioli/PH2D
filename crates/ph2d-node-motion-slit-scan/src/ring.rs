//! The delay line: the past `MAX_LAG` ticks of every element's position, carried
//! as plain columns on the node's own output (the sequential-node convention —
//! the `pre` self-loop hands them back next tick).
//!
//! `slot(k)` holds the live position from **k ticks ago**, so a lookback of `k`
//! is a column read, not a search. Slot 0 does not exist: it is the live input.

use ph2d_nodegraph::attr::{Column, Stream};

/// The deepest lookback, in ticks — the delay line's length. At 60 Hz this is
/// half a second of shear across the set, and it bounds the state a document can
/// ask for (the `lag` param is clamped to it, so a hand-edited value cannot
/// allocate an unbounded ring).
pub(crate) const MAX_LAG: usize = 32;

/// The column names of the delay line, indexed by `k - 1`. Spelled out rather
/// than formatted per tick: the names are part of the stream's shape, and a
/// `format!` here would allocate a `String` for every slot on every cook.
const SLOTS: [&str; MAX_LAG] = [
    "ss_1", "ss_2", "ss_3", "ss_4", "ss_5", "ss_6", "ss_7", "ss_8", "ss_9", "ss_10", "ss_11",
    "ss_12", "ss_13", "ss_14", "ss_15", "ss_16", "ss_17", "ss_18", "ss_19", "ss_20", "ss_21",
    "ss_22", "ss_23", "ss_24", "ss_25", "ss_26", "ss_27", "ss_28", "ss_29", "ss_30", "ss_31",
    "ss_32",
];

/// The name of the slot holding the position `k` ticks ago (`1 ..= MAX_LAG`).
pub(crate) fn slot(k: usize) -> &'static str {
    SLOTS[k.clamp(1, MAX_LAG) - 1]
}

/// Whether `name` is one of the delay-line's own columns — the caller strips
/// them before rewriting, so they never accumulate stale duplicates.
pub(crate) fn is_slot(name: &str) -> bool {
    SLOTS.contains(&name)
}

/// The past positions the state carries, `[k - 1] = k ticks ago`. A slot that is
/// missing or whose length no longer matches the live element count (the set was
/// rebuilt — an emitter churned, a grid resized) reads as the LIVE positions:
/// the delay line re-seeds flat instead of pairing unrelated elements, so the
/// scan re-forms over the next `lag` ticks rather than snapping to nonsense.
pub(crate) fn past(state: &Stream, live: &[[f32; 2]]) -> Vec<Vec<[f32; 2]>> {
    (1..=MAX_LAG)
        .map(|k| match state.get(slot(k)) {
            Some(Column::Vec2(v)) if v.len() == live.len() => v.clone(),
            _ => live.to_vec(),
        })
        .collect()
}

/// Advance the delay line one tick and write it onto `out`: the live positions
/// become "1 tick ago", and every other slot shifts one deeper (the oldest falls
/// off the end).
pub(crate) fn push(out: &mut Stream, past: Vec<Vec<[f32; 2]>>, live: &[[f32; 2]]) {
    let mut shifted = live.to_vec();
    for (k, mut older) in past.into_iter().enumerate().take(MAX_LAG) {
        // `shifted` enters holding what belongs in slot k+1; swap it out and
        // carry the slot's previous contents into the next, deeper slot.
        std::mem::swap(&mut shifted, &mut older);
        out.set(slot(k + 1), Column::Vec2(older));
    }
}
