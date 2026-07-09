//! Column readers shared by the integrator: read a named column to exactly `n`
//! elements, filling absences with a per-column identity. Self-contained per
//! node crate (drop-crate isolation — the same ~30 lines live in each sibling,
//! like `falloff_at` in the behaviours).

use ph2d_nodegraph::attr::{Column, Stream};

/// A `Vec2` column read to length `n`, `identity` filling an absent /
/// wrong-typed / short column.
pub(crate) fn vec2_to_n(s: &Stream, name: &str, n: usize, identity: [f32; 2]) -> Vec<[f32; 2]> {
    let mut v = match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, identity);
    v
}

/// A `Scalar` column read to length `n` (absent / wrong-typed → `identity`).
pub(crate) fn scalar_to_n(s: &Stream, name: &str, n: usize, identity: f32) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, identity);
    v
}

/// Per-element **identity keys** of a stream: the `id` column when present
/// (the particle emitter stamps one, stable across a birth/death churn), else
/// the element's position. `None` when the stream has no `id` column at all —
/// the caller then knows identity is positional and a count change means the
/// whole set was rebuilt (a re-seed), not that some elements died.
pub(crate) fn ids_of(s: &Stream, n: usize) -> Option<Vec<u32>> {
    match s.get("id") {
        // Ids are authored as exact small integers in an f32 (`u32 as f32` is
        // lossless below 2^24 — ~9 hours of spawning at 500/s), so the cast back
        // is exact. A negative / non-finite id is a corrupt stream: key it to 0.
        Some(Column::Scalar(v)) => {
            let mut out: Vec<u32> = v.iter().map(|f| f.max(0.0) as u32).collect();
            out.resize(n, 0);
            Some(out)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_and_short_columns_fill_with_identity() {
        let s = Stream::new(3).with(
            "vel",
            Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0], [0.0, 0.0]]),
        );
        assert_eq!(
            vec2_to_n(&s, "vel", 3, [9.0, 9.0]),
            vec![[1.0, 2.0], [3.0, 4.0], [0.0, 0.0]]
        );
        assert_eq!(vec2_to_n(&s, "nope", 2, [7.0, 7.0]), vec![[7.0, 7.0]; 2]);
        assert_eq!(scalar_to_n(&s, "nope", 2, 0.5), vec![0.5, 0.5]);
    }
}
