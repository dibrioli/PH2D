//! Fill primitives that would be the natural siblings of [`crate::paint::fill_circle`]
//! but for one thing: `paint.rs` sits at its FROZEN LOC ceiling (884), so a new drawing
//! primitive is born here rather than growing a god-file. Same layer, same job — turn a
//! shape into a `VectorScene` fill — just a different file.
//!
//! ⚠️ APPEND-ONLY, and that is not a style note: two parallel lines reached for this
//! same address independently, for this same reason. They collided cleanly only because
//! each ADDED a function instead of editing a shared one — and when the second line
//! later withdrew its feature, dropping its primitive did not disturb this one. A
//! primitive that arrives here should not rewrite, or delete, the ones already here.

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
