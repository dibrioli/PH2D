//! Force-node accumulation shared shape: copy the stream through and **add**
//! this force's per-instance contribution into the transient `accel` column
//! (created at zero when absent), gated by the multiplicative `falloff` column
//! (plan §1.6). Self-contained per node crate (drop-crate isolation — the same
//! ~40 lines live in each force, like `falloff_at` in the behaviours).

use ph2d_nodegraph::attr::{Column, Stream};

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
pub(crate) fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// A `Vec2` column element (absent / wrong-typed / short → `default`).
pub(crate) fn vec2_at(stream: &Stream, name: &str, i: usize, default: [f32; 2]) -> [f32; 2] {
    match stream.get(name) {
        Some(Column::Vec2(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}

/// Copy `input` through unchanged except `accel`, which becomes
/// `input.accel (or 0) + contrib` — chained forces accumulate (Houdini POP
/// convention: microsolvers add force, one solver integrates).
pub(crate) fn add_accel(input: &Stream, contrib: &[[f32; 2]]) -> Stream {
    let n = input.count();
    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != "accel" {
            out.set(name.clone(), col.clone());
        }
    }
    let mut accel = match input.get("accel") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    accel.resize(n, [0.0, 0.0]);
    for (a, c) in accel.iter_mut().zip(contrib) {
        a[0] += c[0];
        a[1] += c[1];
    }
    out.set("accel", Column::Vec2(accel));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contributions_accumulate_on_an_existing_accel() {
        let s = Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 1.0]]))
            .with("accel", Column::Vec2(vec![[1.0, 0.0], [0.0, 0.0]]));
        let out = add_accel(&s, &[[0.5, 0.5], [2.0, 0.0]]);
        match out.get("accel").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[1.5, 0.5], [2.0, 0.0]]),
            _ => panic!("accel"),
        }
        // Everything else passes through untouched.
        match out.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 0.0], [1.0, 1.0]]),
            _ => panic!("P"),
        }
    }
}
