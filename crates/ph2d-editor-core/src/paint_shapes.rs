//! Fill primitives that would be the natural siblings of [`crate::paint::fill_circle`]
//! but for one thing: `paint.rs` sits at its FROZEN LOC ceiling (884), so a new drawing
//! primitive is born here rather than growing a god-file. Same layer, same job — turn a
//! shape into a `VectorScene` fill — just a different file.
//!
//! ⚠️ This module is an APPEND-ONLY extension point, and that is not a style note: two
//! parallel lines reached for this address independently and for the same reason
//! (`fill_diamond` for the graph editor's sockets, `fill_polygon` for the timeline).
//! Because they added different functions rather than editing a shared one, the merge
//! was mechanical — keep both. A primitive arriving here should not rewrite the ones
//! already here.

use ph2d_vector::{Affine, BezPath, Color, Fill, VectorScene};

/// Fill a diamond (a square on its point) centered at `(cx, cy)` with half-diagonal `r`
/// — the value-vs-column socket glyph the graph editor draws next to `fill_circle`'s ○.
/// Four line segments, so it is exact and transcendental-free (no rotation matrix,
/// HR-5). `r <= 0` is a no-op.
pub fn fill_diamond(scene: &mut VectorScene, cx: f32, cy: f32, r: f32, color: Color) {
    if r <= 0.0 {
        return;
    }
    let (cx, cy, r) = (cx as f64, cy as f64, r as f64);
    let mut path = BezPath::new();
    path.move_to((cx, cy - r)); // top
    path.line_to((cx + r, cy)); // right
    path.line_to((cx, cy + r)); // bottom
    path.line_to((cx - r, cy)); // left
    path.close_path();
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &path);
}

/// Fill the closed polygon through `points`.
///
/// The sibling of [`crate::paint::stroke_polyline`]: same input, same non-zero fill rule
/// the rest of this module uses, and it closes the path for you. Fewer than three points
/// is a no-op — a polygon needs an inside.
///
/// ⚠️ Exists because a figure that must ROTATE cannot be assembled from circles and
/// axis-aligned rounded rects: `fill_rounded_rect` takes a `Rect`, which has no angle.
/// A caller that wants a shape to turn transforms its points and fills them.
pub fn fill_polygon(scene: &mut VectorScene, points: &[(f32, f32)], color: Color) {
    if points.len() < 3 {
        return;
    }
    let mut path = BezPath::new();
    path.move_to((points[0].0 as f64, points[0].1 as f64));
    for &(x, y) in &points[1..] {
        path.line_to((x as f64, y as f64));
    }
    path.close_path();
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &path);
}
