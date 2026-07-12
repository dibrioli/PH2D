//! **Batched paint primitives** — many little shapes in ONE draw call. Split from `paint` for the
//! LOC cap; `super` is `paint`.
//!
//! Vello's cost is per **draw object**, not per vertex. A panel that draws a thousand dots as a
//! thousand fills hands the rasterizer a thousand paths to bound, bin and tile — and pays for it
//! every single frame. In one path it is one. These helpers exist so a caller that already HAS a
//! set of shapes never has to learn that the hard way (doc 53: the motion graph's postage stamps
//! did, at ~4 000 draw objects a frame, and took the frame rate with them).

use ph2d_vector::{Affine, BezPath, Color, Fill, Stroke, VectorScene};

/// **Many little dots, ONE draw call.**
///
/// Vello's cost is per **draw object**, not per vertex: a scatter of 96 dots drawn as 96 fills is
/// 96 paths to bound, bin and rasterize. The motion graph's postage stamps (a preview of what each
/// node emits, on every card) turned that into ~4 000 draw objects a frame and the editor's frame
/// rate fell off a cliff. In one path they are ~40 — one per card — and the pixels are identical.
///
/// The dots are **squares**, and that is not a compromise: at the radius a stamp uses (1-2 px) a
/// circle and a square cover the same pixels, and four straight lines flatten for free where four
/// cubics do not (measured: 2.9 ms → 1.5 ms of encoding for 4 032 dots).
pub fn fill_dots(scene: &mut VectorScene, centers: &[(f32, f32)], r: f32, color: Color) {
    if centers.is_empty() || r <= 0.0 {
        return;
    }
    let mut path = BezPath::new();
    for (cx, cy) in centers {
        let (x, y, r) = (*cx as f64, *cy as f64, r as f64);
        path.move_to((x - r, y - r));
        path.line_to((x + r, y - r));
        path.line_to((x + r, y + r));
        path.line_to((x - r, y + r));
        path.close_path();
    }
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &path);
}

/// **Many disjoint polylines, ONE draw call** — the dashes of a marching wire, the ticks of a
/// ruler, any set of strokes that share a width and a colour.
///
/// Same reason as [`fill_dots`]: a wire whose flow animation is thirteen little strokes costs
/// thirteen draw objects, every frame, on every wire. As one path with thirteen subpaths it costs
/// one.
pub fn stroke_subpaths(
    scene: &mut VectorScene,
    subpaths: &[Vec<(f32, f32)>],
    width: f32,
    color: Color,
) {
    let mut path = BezPath::new();
    let mut any = false;
    for pts in subpaths.iter().filter(|p| p.len() >= 2) {
        any = true;
        path.move_to((pts[0].0 as f64, pts[0].1 as f64));
        for &(x, y) in &pts[1..] {
            path.line_to((x as f64, y as f64));
        }
    }
    if !any {
        return;
    }
    let stroke = Stroke::new(width as f64);
    scene
        .inner_mut()
        .stroke(&stroke, Affine::IDENTITY, color, None, &path);
}
