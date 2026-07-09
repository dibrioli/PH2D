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
