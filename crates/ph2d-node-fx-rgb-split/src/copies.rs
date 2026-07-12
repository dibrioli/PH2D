//! Column plumbing for a **ghost-copy** FX: read the columns a copy needs, and
//! repeat every other column once per copy.
//!
//! A copied leaf (60 lines), not a new foundational crate — the rule of the line
//! for a helper with two consumers ([[project_brush_along_path_satellite_not_node]]).
//! `fx.drop_shadow` carries the same file.

use ph2d_nodegraph::attr::{Column, Stream};

/// Every element's position (an absent `P` → the origin, so a stream that carries
/// only a value still copies rather than panicking).
pub(crate) fn positions(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) if v.len() == s.count() => v.clone(),
        _ => vec![[0.0, 0.0]; s.count()],
    }
}

/// Every element's tint. **Absent → opaque white**, which is the identity of a
/// multiplicative tint (and the same fallback the lowering uses) — filling the
/// gap with zeros would make every copy black and invisible.
pub(crate) fn tints(s: &Stream) -> Vec<[f32; 4]> {
    match s.get("tint") {
        Some(Column::Vec4(v)) if v.len() == s.count() => v.clone(),
        _ => vec![[1.0, 1.0, 1.0, 1.0]; s.count()],
    }
}

/// The multiplicative `falloff` weight of element `i` (absent → `1.0`) — the
/// module's convention for "which elements does this node act on".
pub(crate) fn falloff_at(s: &Stream, i: usize) -> f32 {
    match s.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// `col` repeated `k` times back-to-back — the copies inherit every column the
/// source had (`id`, `size`, `rot`, `uv_rect`, …), so a ghost is the same element,
/// only displaced and recoloured. The caller overwrites `P` / `tint` afterwards.
///
/// **Block order, not interleaved:** all of copy 1, then all of copy 2, then the
/// originals. The stream's order IS the draw order (the lowering walks it), so a
/// block layout puts every ghost behind every element — the whole-layer shadow of
/// Photoshop, not a per-element one that would fall on top of its neighbour.
pub(crate) fn tile(col: &Column, k: usize) -> Column {
    fn rep<T: Copy>(v: &[T], k: usize) -> Vec<T> {
        let mut out = Vec::with_capacity(v.len() * k);
        for _ in 0..k {
            out.extend_from_slice(v);
        }
        out
    }
    match col {
        Column::Scalar(v) => Column::Scalar(rep(v, k)),
        Column::Vec2(v) => Column::Vec2(rep(v, k)),
        Column::Vec3(v) => Column::Vec3(rep(v, k)),
        Column::Vec4(v) => Column::Vec4(rep(v, k)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_absent_tint_reads_as_opaque_white_not_black() {
        // The bug this guards: filling a missing tint with zeros makes every ghost
        // invisible, which reads as "the FX is broken" rather than "a column was
        // missing".
        let s = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]));
        assert_eq!(tints(&s), vec![[1.0; 4]; 2]);
        assert_eq!(positions(&s), vec![[0.0, 0.0], [1.0, 0.0]]);
        assert_eq!(falloff_at(&s, 1), 1.0);
    }

    #[test]
    fn tile_repeats_in_blocks_so_the_ghosts_stay_behind() {
        let col = Column::Scalar(vec![1.0, 2.0]);
        match tile(&col, 3) {
            Column::Scalar(v) => assert_eq!(v, vec![1.0, 2.0, 1.0, 2.0, 1.0, 2.0]),
            _ => panic!("variant"),
        }
    }
}
