//! Channel access for the spring: read one transform channel of a stream
//! (`0` X, `1` Y, `2` Rotation, `3` Size) and write it back, all other columns
//! untouched. Self-contained per node crate (drop-crate isolation — the same
//! shape as the behaviours' `channel.rs`).
//!
//! Identities per column (the lying-widget lesson): `P`/`rot` are `0`, but
//! `size`'s identity is **unit scale** — reading Size off a bare generator
//! yields `1.0`, so a spring on Size settles at unit scale instead of
//! collapsing the sprite through zero.

use ph2d_nodegraph::attr::{Column, Stream};

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
pub(crate) fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The inverse mass (PBD's `w = 1/m`) that `motion.pin_constraint` writes:
/// `1` = free (absent, the default), `0` = pinned. It multiplies the falloff, so
/// a pinned element's spring is fully transparent — it tracks its animated
/// target rigidly, with no lag, overshoot or settle, which is exactly what a
/// nailed-down element does.
pub(crate) fn inv_mass_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("inv_mass") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// Per-element identity keys — the `id` column when present, else `None`
/// (positional identity). Mirrors `motion.integrate`'s reading of the same
/// convention, so a spring downstream of the particle emitter tracks each
/// particle instead of resetting as the set churns.
pub(crate) fn ids_of(s: &Stream, n: usize) -> Option<Vec<u32>> {
    match s.get("id") {
        Some(Column::Scalar(v)) => {
            let mut out: Vec<u32> = v.iter().map(|f| f.max(0.0) as u32).collect();
            out.resize(n, 0);
            Some(out)
        }
        _ => None,
    }
}

/// Instance `i`'s value on `channel` (absent column → the channel's identity).
/// Size reads the X component (the spring is uniform, like the reference).
pub(crate) fn channel_get(stream: &Stream, channel: i32, i: usize) -> f32 {
    match channel {
        0 | 1 => match stream.get("P") {
            Some(Column::Vec2(v)) => v.get(i).map(|p| p[channel as usize]).unwrap_or(0.0),
            _ => 0.0,
        },
        2 => match stream.get("rot") {
            Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(0.0),
            _ => 0.0,
        },
        _ => match stream.get("size") {
            Some(Column::Vec2(v)) => v.get(i).map(|s| s[0]).unwrap_or(1.0),
            _ => 1.0,
        },
    }
}

/// Rewrite `channel` with `values` (one per instance), copying every other
/// column through. Size writes both components (uniform).
pub(crate) fn channel_set(input: &Stream, channel: i32, values: &[f32]) -> Stream {
    let n = input.count();
    let target = match channel {
        0 | 1 => "P",
        2 => "rot",
        _ => "size",
    };
    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != target {
            out.set(name.clone(), col.clone());
        }
    }
    match channel {
        0 | 1 => {
            let comp = channel as usize;
            let mut p = match input.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => Vec::new(),
            };
            p.resize(n, [0.0, 0.0]);
            for (pi, &v) in p.iter_mut().zip(values) {
                pi[comp] = v;
            }
            out.set("P", Column::Vec2(p));
        }
        2 => {
            out.set("rot", Column::Scalar(values.to_vec()));
        }
        _ => {
            out.set(
                "size",
                Column::Vec2(values.iter().map(|&v| [v, v]).collect()),
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_identity_is_unit_not_zero() {
        // A bare stream (only P): Size reads 1.0, and writing it back yields a
        // uniform size column — never a collapse through zero.
        let s = Stream::new(1).with("P", Column::Vec2(vec![[3.0, 4.0]]));
        assert_eq!(channel_get(&s, 3, 0), 1.0);
        let out = channel_set(&s, 3, &[1.25]);
        match out.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [1.25, 1.25]),
            _ => panic!("size"),
        }
        // P untouched by a Size write.
        match out.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [3.0, 4.0]),
            _ => panic!("P"),
        }
    }

    #[test]
    fn y_write_keeps_x() {
        let s = Stream::new(1).with("P", Column::Vec2(vec![[3.0, 4.0]]));
        assert_eq!(channel_get(&s, 1, 0), 4.0);
        let out = channel_set(&s, 1, &[9.0]);
        match out.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [3.0, 9.0]),
            _ => panic!("P"),
        }
    }
}
