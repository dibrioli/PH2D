//! Channel routing shared by the Motion behaviours: add a per-instance `delta`
//! to one selected transform channel of a stream (`0` X, `1` Y, `2` Rotation,
//! `3` Size), passing every other column through unchanged. Self-contained per
//! node crate (drop-crate isolation — the same ~40 lines live in each behaviour,
//! like `falloff_at`).
//!
//! An absent target column starts from its **identity**, so an additive behaviour
//! is well-defined on a bare generator that only emits `P`. The identity is per
//! column, NOT a blanket zero: `P` and `rot` are `0`, but `size`'s identity is
//! **unit scale `[1,1]`** (matching `motion.scale`'s absent-column base). Basing
//! `size` at `[0,0]` would make a bipolar behaviour drive the size *through zero
//! and negative* — sprites collapsing and inverting instead of pulsing.

use ph2d_nodegraph::attr::{Column, Stream};

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
pub(crate) fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The stream column a channel index writes to: X/Y → `P`, Rotation → `rot`,
/// Size (or any out-of-range value) → `size`.
fn channel_column(channel: i32) -> &'static str {
    match channel {
        0 | 1 => "P",
        2 => "rot",
        _ => "size",
    }
}

/// A `Vec2` column read to length `n`, with `identity` filling an absent /
/// wrong-typed / short column. `P`'s identity is `[0,0]`; `size`'s is `[1,1]`.
fn base_vec2(input: &Stream, name: &str, n: usize, identity: [f32; 2]) -> Vec<[f32; 2]> {
    let mut v = match input.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, identity);
    v
}

/// A `Scalar` column read to length `n` (absent / wrong-typed → all `0`).
fn base_scalar(input: &Stream, name: &str, n: usize) -> Vec<f32> {
    let mut v = match input.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, 0.0);
    v
}

/// Add `deltas[i]` to instance `i`'s value on the selected `channel`, returning a
/// new stream with that column rewritten and all others copied through. Size adds
/// the delta to **both** components (uniform).
pub(crate) fn apply_channel_delta(input: &Stream, channel: i32, deltas: &[f32]) -> Stream {
    let n = input.count();
    let target = channel_column(channel);
    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != target {
            out.set(name.clone(), col.clone());
        }
    }
    match channel {
        0 | 1 => {
            let comp = channel as usize; // 0 = X, 1 = Y
            let mut p = base_vec2(input, "P", n, [0.0, 0.0]);
            for (pi, &d) in p.iter_mut().zip(deltas) {
                pi[comp] += d;
            }
            out.set("P", Column::Vec2(p));
        }
        2 => {
            let mut r = base_scalar(input, "rot", n);
            for (ri, &d) in r.iter_mut().zip(deltas) {
                *ri += d;
            }
            out.set("rot", Column::Scalar(r));
        }
        _ => {
            // Unit scale is the identity of `size` — never `[0,0]`.
            let mut s = base_vec2(input, "size", n, [1.0, 1.0]);
            for (si, &d) in s.iter_mut().zip(deltas) {
                si[0] += d;
                si[1] += d;
            }
            out.set("size", Column::Vec2(s));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_channel_writes_both_components_from_the_unit_identity() {
        // A bare stream (only P) + a Size delta → a fresh size column = UNIT + delta
        // on both axes (uniform), P untouched. The identity is `[1,1]`, not `[0,0]`.
        let input = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 1.0]]));
        let out = apply_channel_delta(&input, 3, &[0.5, 2.0]);
        match out.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[1.5, 1.5], [3.0, 3.0]]),
            _ => panic!("size"),
        }
        // P passes through unchanged.
        match out.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 0.0], [1.0, 1.0]]),
            _ => panic!("P"),
        }
    }

    #[test]
    fn a_bipolar_size_delta_never_collapses_or_inverts_the_sprite() {
        // The defect this identity guards: an oscillator on the Size channel emits
        // a bipolar delta (±amplitude). Based at `[0,0]` the size would sweep
        // through 0 and go NEGATIVE every cycle; based at the unit identity it
        // stays positive for any |delta| < 1.
        let input =
            Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]));
        let out = apply_channel_delta(&input, 3, &[-0.3, 0.0, 0.3]);
        match out.get("size").unwrap() {
            Column::Vec2(v) => {
                assert_eq!(v, &vec![[0.7, 0.7], [1.0, 1.0], [1.3, 1.3]]);
                assert!(v.iter().all(|s| s[0] > 0.0), "size never reaches zero");
            }
            _ => panic!("size"),
        }
    }

    #[test]
    fn rotation_channel_adds_to_existing_rot() {
        let input = Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]]))
            .with("rot", Column::Scalar(vec![1.0, 2.0]));
        let out = apply_channel_delta(&input, 2, &[0.5, 0.5]);
        match out.get("rot").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![1.5, 2.5]),
            _ => panic!("rot"),
        }
    }

    #[test]
    fn x_channel_touches_only_x() {
        let input = Stream::new(1).with("P", Column::Vec2(vec![[1.0, 5.0]]));
        let out = apply_channel_delta(&input, 0, &[2.0]);
        match out.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[3.0, 5.0]]),
            _ => panic!("P"),
        }
    }
}
