//! Column plumbing for the trail: gather rows, concatenate two streams, and
//! know what an ABSENT column means for each reserved name.
//!
//! The identity of a column is not always zero — a `size` filled with `[0,0]`
//! collapses every echo to nothing, and a `tint` filled with `[0,0,0,0]` makes
//! them invisible. The reserved names carry their own identity (see
//! [`default_for`]); everything else is genuinely additive and defaults to 0.

use ph2d_nodegraph::attr::{Column, Stream};
use std::collections::BTreeSet;

/// Scalar identity for a reserved column name. `size` and `tint` are
/// multiplicative (1.0), everything else additive (0.0).
fn scalar_identity(name: &str) -> f32 {
    match name {
        "size" | "tint" => 1.0,
        _ => 0.0,
    }
}

/// The value an ABSENT column contributes for one row: the reserved names carry
/// their own identity, so filling a gap never darkens or shrinks an echo.
/// `uv_rect` is the whole-atlas rect, matching the lowering's own fallback.
pub(crate) fn default_for(name: &str, dim: &Column) -> Column {
    let s = scalar_identity(name);
    match dim {
        Column::Scalar(_) => Column::Scalar(vec![s]),
        Column::Vec2(_) => Column::Vec2(vec![[s; 2]]),
        Column::Vec3(_) => Column::Vec3(vec![[s; 3]]),
        Column::Vec4(_) => {
            if name == "uv_rect" {
                Column::Vec4(vec![[0.0, 0.0, 1.0, 1.0]])
            } else {
                Column::Vec4(vec![[s; 4]])
            }
        }
    }
}

/// Repeat a one-row column `n` times.
fn repeat(one: &Column, n: usize) -> Column {
    match one {
        Column::Scalar(v) => Column::Scalar(vec![v[0]; n]),
        Column::Vec2(v) => Column::Vec2(vec![v[0]; n]),
        Column::Vec3(v) => Column::Vec3(vec![v[0]; n]),
        Column::Vec4(v) => Column::Vec4(vec![v[0]; n]),
    }
}

/// Rows `idx` of `s`, in order. Columns keep their types.
pub(crate) fn gather(s: &Stream, idx: &[usize]) -> Stream {
    let mut out = Stream::new(idx.len());
    for (name, col) in s.columns() {
        let picked = match col {
            Column::Scalar(v) => Column::Scalar(idx.iter().map(|&i| v[i]).collect()),
            Column::Vec2(v) => Column::Vec2(idx.iter().map(|&i| v[i]).collect()),
            Column::Vec3(v) => Column::Vec3(idx.iter().map(|&i| v[i]).collect()),
            Column::Vec4(v) => Column::Vec4(idx.iter().map(|&i| v[i]).collect()),
        };
        out.set(name.clone(), picked);
    }
    out
}

/// Append two columns of the same variant. `None` on a variant mismatch (the
/// caller then fills that side with the column's identity instead of silently
/// reinterpreting the bits).
fn append(a: &Column, b: &Column) -> Option<Column> {
    Some(match (a, b) {
        (Column::Scalar(x), Column::Scalar(y)) => {
            Column::Scalar(x.iter().chain(y).copied().collect())
        }
        (Column::Vec2(x), Column::Vec2(y)) => Column::Vec2(x.iter().chain(y).copied().collect()),
        (Column::Vec3(x), Column::Vec3(y)) => Column::Vec3(x.iter().chain(y).copied().collect()),
        (Column::Vec4(x), Column::Vec4(y)) => Column::Vec4(x.iter().chain(y).copied().collect()),
        _ => return None,
    })
}

/// `a` then `b`, over the UNION of their columns. A column present on one side
/// only (the newest generation has no `trail_age` yet; last tick's echoes have
/// no column the upstream just started emitting) is filled with its identity on
/// the other, so no row is silently zeroed.
pub(crate) fn concat(a: &Stream, b: &Stream) -> Stream {
    let mut out = Stream::new(a.count() + b.count());
    let names: BTreeSet<&str> = a
        .columns()
        .chain(b.columns())
        .map(|(n, _)| n.as_str())
        .collect();
    for name in names {
        let col = match (a.get(name), b.get(name)) {
            (Some(x), Some(y)) => append(x, y).unwrap_or_else(|| {
                // Same name, different dimension on the two sides: keep `a`'s
                // shape and fill `b`'s rows with the identity. Only reachable
                // when the upstream changed a column's type mid-run.
                append(x, &repeat(&default_for(name, x), b.count())).expect("same variant")
            }),
            (Some(x), None) => {
                append(x, &repeat(&default_for(name, x), b.count())).expect("same variant")
            }
            (None, Some(y)) => {
                append(&repeat(&default_for(name, y), a.count()), y).expect("same variant")
            }
            (None, None) => unreachable!("name came from one of the two"),
        };
        out.set(name.to_string(), col);
    }
    out
}

/// Multiply every component of a `Vec2` column (the echo's `shrink`).
pub(crate) fn scale_vec2(s: &mut Stream, name: &str, k: f32) {
    if let Some(Column::Vec2(v)) = s.get(name) {
        let scaled = v.iter().map(|p| [p[0] * k, p[1] * k]).collect();
        s.set(name, Column::Vec2(scaled));
    }
}

/// Multiply the ALPHA (`w`) of a `Vec4` column (the echo's `fade`), leaving the
/// hue alone — a faded echo is the same colour, more transparent.
pub(crate) fn fade_alpha(s: &mut Stream, name: &str, k: f32) {
    if let Some(Column::Vec4(v)) = s.get(name) {
        let faded = v.iter().map(|c| [c[0], c[1], c[2], c[3] * k]).collect();
        s.set(name, Column::Vec4(faded));
    }
}

/// The `trail_age` column, or zeros when absent (the freshest generation).
pub(crate) fn ages(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => vec![0.0; s.count()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_size_and_tint_fill_with_their_identity_not_zero() {
        // The bug this guards: a `[0,0]` size collapses every echo to a point,
        // and a `[0,0,0,0]` tint makes them invisible — both look like "the
        // trail is broken" rather than "a column was missing".
        let live = Stream::new(1).with("P", Column::Vec2(vec![[1.0, 1.0]]));
        let echo = Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 0.0]]))
            .with("size", Column::Vec2(vec![[0.5, 0.5]]))
            .with("tint", Column::Vec4(vec![[1.0, 0.0, 0.0, 0.5]]));
        let out = concat(&live, &echo);
        assert_eq!(out.count(), 2);
        match out.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [1.0, 1.0], "live keeps unit size"),
            _ => panic!(),
        }
        match out.get("tint").unwrap() {
            Column::Vec4(v) => assert_eq!(v[0], [1.0; 4], "live keeps opaque white"),
            _ => panic!(),
        }
    }

    #[test]
    fn concat_is_order_preserving_and_gather_picks_rows() {
        let a = Stream::new(2).with("v", Column::Scalar(vec![1.0, 2.0]));
        let b = Stream::new(1).with("v", Column::Scalar(vec![3.0]));
        match concat(&a, &b).get("v").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![1.0, 2.0, 3.0]),
            _ => panic!(),
        }
        match gather(&a, &[1, 0, 1]).get("v").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![2.0, 1.0, 2.0]),
            _ => panic!(),
        }
    }

    #[test]
    fn fade_touches_alpha_only_and_shrink_scales_both_axes() {
        let mut s = Stream::new(1)
            .with("tint", Column::Vec4(vec![[0.2, 0.4, 0.6, 1.0]]))
            .with("size", Column::Vec2(vec![[2.0, 4.0]]));
        fade_alpha(&mut s, "tint", 0.5);
        scale_vec2(&mut s, "size", 0.5);
        match s.get("tint").unwrap() {
            Column::Vec4(v) => assert_eq!(v[0], [0.2, 0.4, 0.6, 0.5], "hue untouched"),
            _ => panic!(),
        }
        match s.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [1.0, 2.0]),
            _ => panic!(),
        }
    }
}
