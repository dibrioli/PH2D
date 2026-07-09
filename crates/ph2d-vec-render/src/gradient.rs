//! Gradient rendering + on-canvas handles for the vector scene (ADR-0108).
//!
//! Split from `lib.rs` (LOC cap): everything that turns a path's [`Paint`] gradient
//! into pixels or into editable handles lives here — the multi-point IDW rasterizer
//! + image-blit fill, and the [`GradHandle`] model with its pure world-space
//! hit-test / drag / draw helpers (the shell owns the camera + screen math).

use crate::color;
use ph2d_vec_scene::{GradientPoint, Paint, VecPath, VecPathId, VecScene};
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color, Fill, ImageQuality, Point, Stroke, VectorScene,
};
use std::sync::Arc;

/// A draggable handle of a path's gradient fill (on-canvas editing, Cavalry-style).
/// MultiPoint exposes one handle per point; Linear its two endpoints; Radial its
/// center + an edge handle (drags the radius). Both Linear and Radial also expose
/// their INTERIOR ramp stops ([`GradHandle::Stop`]) as markers along the ramp line —
/// the two END stops (offset 0 / 1) are pinned to the endpoint handles, which
/// recolour them. The screen hit-test / camera work lives in the shell;
/// [`hit_gradient_handle`] / [`drag_gradient_handle`] are the pure world-space
/// geometry helpers it drives.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GradHandle {
    /// A multi-point gradient point by index.
    Point(usize),
    /// A linear gradient's start endpoint.
    LinearStart,
    /// A linear gradient's end endpoint.
    LinearEnd,
    /// A radial gradient's center.
    RadialCenter,
    /// A radial gradient's edge (drags the radius).
    RadialEdge,
    /// An INTERIOR ramp stop (Linear/Radial) by index — dragged along the ramp to
    /// re-position its offset. The end stops (index 0 / last) are the endpoint
    /// handles, never a `Stop`.
    Stop(usize),
}

impl GradHandle {
    /// The MultiPoint point index this handle addresses (`None` otherwise).
    #[must_use]
    pub fn point(self) -> Option<usize> {
        match self {
            GradHandle::Point(i) => Some(i),
            _ => None,
        }
    }

    /// The ramp-stop index this handle addresses (`None` unless it is a [`Self::Stop`]).
    #[must_use]
    pub fn stop(self) -> Option<usize> {
        match self {
            GradHandle::Stop(i) => Some(i),
            _ => None,
        }
    }
}

/// The ramp line `(a, b)` of a Linear (`start`→`end`) or Radial (`center`→edge at
/// `center + (radius, 0)`) gradient — the axis stops sit on. `None` for others.
fn ramp_endpoints(fill: &Paint) -> Option<([f64; 2], [f64; 2])> {
    match fill {
        Paint::Linear { start, end, .. } => Some((*start, *end)),
        Paint::Radial { center, radius, .. } => Some((*center, [center[0] + radius, center[1]])),
        _ => None,
    }
}

/// The world-space point of ramp offset `t` along `(a, b)`.
#[inline]
fn ramp_point(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

/// The world-space anchor of `handle` on `path`'s fill (`None` if the handle
/// doesn't match the fill kind / index). The radial EDGE sits at `center + (r, 0)`.
fn handle_world_pos(path: &VecPath, handle: GradHandle) -> Option<[f64; 2]> {
    match (&path.fill, handle) {
        (Some(Paint::MultiPoint { points }), GradHandle::Point(i)) => {
            points.get(i).map(|gp| gp.pos)
        }
        (Some(Paint::Linear { start, .. }), GradHandle::LinearStart) => Some(*start),
        (Some(Paint::Linear { end, .. }), GradHandle::LinearEnd) => Some(*end),
        (Some(Paint::Radial { center, .. }), GradHandle::RadialCenter) => Some(*center),
        (Some(Paint::Radial { center, radius, .. }), GradHandle::RadialEdge) => {
            Some([center[0] + radius, center[1]])
        }
        (Some(fill), GradHandle::Stop(i)) => {
            let stops = match fill {
                Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => stops,
                _ => return None,
            };
            let (a, b) = ramp_endpoints(fill)?;
            stops.get(i).map(|s| ramp_point(a, b, s.offset))
        }
        _ => None,
    }
}

/// Hit-test all of `path`'s gradient handles at world point `(wx, wy)` within
/// `world_thresh` world-units, returning the closest match (`None` if none / no
/// gradient fill). Squared distance only.
#[must_use]
pub fn hit_gradient_handle(
    path: &VecPath,
    wx: f64,
    wy: f64,
    world_thresh: f64,
) -> Option<GradHandle> {
    let t2 = world_thresh * world_thresh;
    let mut best: Option<(GradHandle, f64)> = None;
    let consider = |best: &mut Option<(GradHandle, f64)>, h: GradHandle| {
        if let Some(p) = handle_world_pos(path, h) {
            let d2 = (wx - p[0]).powi(2) + (wy - p[1]).powi(2);
            if d2 <= t2 && best.is_none_or(|(_, b)| d2 < b) {
                *best = Some((h, d2));
            }
        }
    };
    match &path.fill {
        Some(Paint::MultiPoint { points }) => {
            for i in 0..points.len() {
                consider(&mut best, GradHandle::Point(i));
            }
        }
        Some(Paint::Linear { stops, .. }) => {
            consider(&mut best, GradHandle::LinearStart);
            consider(&mut best, GradHandle::LinearEnd);
            // Interior stops (the ends are the endpoint handles) — tried first so a
            // stop marker near an endpoint still wins by proximity.
            for i in interior_stop_indices(stops.len()) {
                consider(&mut best, GradHandle::Stop(i));
            }
        }
        Some(Paint::Radial { stops, .. }) => {
            consider(&mut best, GradHandle::RadialCenter);
            consider(&mut best, GradHandle::RadialEdge);
            for i in interior_stop_indices(stops.len()) {
                consider(&mut best, GradHandle::Stop(i));
            }
        }
        _ => {}
    }
    best.map(|(h, _)| h)
}

/// The interior ramp-stop indices (`1..len-1`) — the ones drawn/hit as draggable
/// markers (the two ends are the endpoint handles). Empty for a 2-stop ramp.
fn interior_stop_indices(len: usize) -> std::ops::Range<usize> {
    if len >= 3 { 1..len - 1 } else { 0..0 }
}

/// Move `handle` to world `(wx, wy)` on `path`'s fill (the radial edge sets the
/// radius = distance to the center). Returns `true` iff it applied. No-op if the
/// handle doesn't match the fill kind / index.
pub fn drag_gradient_handle(path: &mut VecPath, handle: GradHandle, wx: f64, wy: f64) -> bool {
    match (&mut path.fill, handle) {
        (Some(Paint::MultiPoint { points }), GradHandle::Point(i)) => {
            if let Some(gp) = points.get_mut(i) {
                gp.pos = [wx, wy];
                return true;
            }
        }
        (Some(Paint::Linear { start, .. }), GradHandle::LinearStart) => {
            *start = [wx, wy];
            return true;
        }
        (Some(Paint::Linear { end, .. }), GradHandle::LinearEnd) => {
            *end = [wx, wy];
            return true;
        }
        (Some(Paint::Radial { center, .. }), GradHandle::RadialCenter) => {
            *center = [wx, wy];
            return true;
        }
        (Some(Paint::Radial { center, radius, .. }), GradHandle::RadialEdge) => {
            *radius = ((wx - center[0]).powi(2) + (wy - center[1]).powi(2))
                .sqrt()
                .max(1e-6);
            return true;
        }
        (Some(fill), GradHandle::Stop(i)) => return drag_ramp_stop(fill, i, wx, wy),
        _ => {}
    }
    false
}

/// Re-position interior ramp stop `i` by projecting world `(wx, wy)` onto the ramp
/// line and setting its offset. Interior stops may **cross one another** freely; the
/// offset is only clamped to stay strictly INSIDE the two end stops (0 / 1), which
/// remain the ramp extremes. The Vec index is left stable (crossing doesn't reorder
/// it) — rendering sorts a copy by offset. No-op for a non-ramp fill / bad index.
fn drag_ramp_stop(fill: &mut Paint, i: usize, wx: f64, wy: f64) -> bool {
    let Some((a, b)) = ramp_endpoints(fill) else {
        return false;
    };
    let stops = match fill {
        Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => stops,
        _ => return false,
    };
    if i == 0 || i + 1 >= stops.len() {
        return false; // ends are pinned; guard interior-only
    }
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len2 = dx * dx + dy * dy;
    if len2 <= 1e-12 {
        return false;
    }
    let t = ((wx - a[0]) * dx + (wy - a[1]) * dy) / len2;
    // Keep interior stops a hair inside the two ends so the ends stay the extremes
    // (their offsets 0 / 1 keep bounding the ramp) and markers never sit on top of
    // the endpoint handles — but interior stops CAN pass each other.
    const EDGE: f64 = 1e-3;
    let t = t.clamp(EDGE, 1.0 - EDGE);
    if (stops[i].offset - t).abs() <= f64::EPSILON {
        return false;
    }
    stops[i].offset = t;
    true
}

/// Desenha os **handles do gradiente** do path `selected` em screen-space (bolinhas
/// roxas estilo Cavalry): um por ponto (MultiPoint), os dois extremos + a linha da
/// rampa (Linear), ou o centro + o anel de raio + o handle de borda (Radial). O
/// handle `active` ganha um anel branco. No-op se não houver gradiente selecionado.
pub fn draw_gradient_handles(
    scene: &VecScene,
    selected: Option<VecPathId>,
    active: Option<GradHandle>,
    transform: Affine,
    target: &mut VectorScene,
) {
    let Some(sel) = selected else { return };
    let Some(path) = scene.paths().iter().find(|p| p.id == sel) else {
        return;
    };
    let purple = Color::from_rgba8(180, 120, 235, 255); // Cavalry purple
    let white = Color::from_rgba8(255, 255, 255, 255);
    // A ringed colour-swatch dot; a white ring marks the active handle.
    let dot = |target: &mut VectorScene, c: Point, fill: Color, is_active: bool| {
        target.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(fill),
            None,
            &Circle::new(c, 5.5),
        );
        let ring = if is_active { white } else { purple };
        target.inner_mut().stroke(
            &Stroke::new(if is_active { 2.0 } else { 1.5 }),
            Affine::IDENTITY,
            &Brush::Solid(ring),
            None,
            &Circle::new(c, 5.5),
        );
    };
    match &path.fill {
        Some(Paint::MultiPoint { points }) => {
            for (i, gp) in points.iter().enumerate() {
                let c = transform * Point::new(gp.pos[0], gp.pos[1]);
                dot(
                    target,
                    c,
                    color(gp.color),
                    active == Some(GradHandle::Point(i)),
                );
            }
        }
        Some(Paint::Linear { start, end, stops }) => {
            let a = transform * Point::new(start[0], start[1]);
            let b = transform * Point::new(end[0], end[1]);
            let mut line = BezPath::new();
            line.move_to(a);
            line.line_to(b);
            target.inner_mut().stroke(
                &Stroke::new(1.5),
                Affine::IDENTITY,
                &Brush::Solid(purple),
                None,
                &line,
            );
            let c0 = stops.first().map_or(purple, |s| color(s.color));
            let c1 = stops.last().map_or(purple, |s| color(s.color));
            dot(target, a, c0, active == Some(GradHandle::LinearStart));
            dot(target, b, c1, active == Some(GradHandle::LinearEnd));
            // Interior stops as markers along the ramp line.
            for i in interior_stop_indices(stops.len()) {
                let s = &stops[i];
                dot(
                    target,
                    a.lerp(b, s.offset),
                    color(s.color),
                    active == Some(GradHandle::Stop(i)),
                );
            }
        }
        Some(Paint::Radial {
            center,
            radius,
            stops,
        }) => {
            let c = transform * Point::new(center[0], center[1]);
            let e = transform * Point::new(center[0] + radius, center[1]);
            let sr = ((e.x - c.x).powi(2) + (e.y - c.y).powi(2)).sqrt();
            target.inner_mut().stroke(
                &Stroke::new(1.5),
                Affine::IDENTITY,
                &Brush::Solid(purple),
                None,
                &Circle::new(c, sr),
            );
            let cc = stops.first().map_or(purple, |s| color(s.color));
            let ce = stops.last().map_or(purple, |s| color(s.color));
            dot(target, c, cc, active == Some(GradHandle::RadialCenter));
            dot(target, e, ce, active == Some(GradHandle::RadialEdge));
            // Interior stops as markers along the center→edge ramp.
            for i in interior_stop_indices(stops.len()) {
                let s = &stops[i];
                dot(
                    target,
                    c.lerp(e, s.offset),
                    color(s.color),
                    active == Some(GradHandle::Stop(i)),
                );
            }
        }
        _ => {}
    }
}

/// Raster resolution of the multi-point IDW blend (upscaled bilinearly by the image
/// brush, so a small buffer stays smooth).
const IDW_RES: u32 = 64;

/// World-space bbox of a path's CONTROL POINTS (anchor + both handles) — the
/// convex hull of every Bézier segment, so it FULLY contains the drawn curve
/// (the anchor-only bbox misses the parts that bulge past the anchors, leaving
/// the multi-point image unfilled there). Unit rect if empty.
fn control_point_bounds(path: &VecPath) -> ([f64; 2], [f64; 2]) {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for v in &path.verts {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            lo[0] = lo[0].min(p[0]);
            lo[1] = lo[1].min(p[1]);
            hi[0] = hi[0].max(p[0]);
            hi[1] = hi[1].max(p[1]);
        }
    }
    if lo[0] > hi[0] {
        ([0.0, 0.0], [1.0, 1.0])
    } else {
        (lo, hi)
    }
}

/// Deterministic per-texel white noise in `[0, 1)` from a 3-way integer key
/// (`px`, `py`, point index) via a splitmix64 finalizer — transcendental-free
/// (HR-5) and stable across frames/machines, so the jitter grain never crawls.
#[inline]
fn hash_noise(px: u32, py: u32, i: u32) -> f64 {
    let mut z = (u64::from(px) << 42)
        ^ (u64::from(py) << 21)
        ^ u64::from(i).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    // Top 53 bits → [0, 1).
    (z >> 11) as f64 / (1u64 << 53) as f64
}

/// Rasterize the Cavalry-style multi-point gradient into a straight-alpha RGBA8
/// buffer of `res × res`, covering the world bbox `(lo,hi)`. Each texel's colour is
/// the inverse-distance-weighted blend of the points:
/// `c(p) = Σ wᵢcᵢ / Σ wᵢ`, `wᵢ = influenceᵢ / (dist²(p,posᵢ) + ε)` — exact colour at
/// each point, smooth between. Squared distance only (no transcendentals). A point's
/// `jitter` (0..1) multiplies its weight by a deterministic per-texel noise factor
/// `1 ± jitter` (grain near that point); `jitter = 0` leaves the blend byte-identical.
fn rasterize_idw(points: &[GradientPoint], lo: [f64; 2], hi: [f64; 2], res: u32) -> Arc<Vec<u8>> {
    const EPS: f64 = 1e-6;
    let n = res as usize;
    let mut buf = vec![0u8; n * n * 4];
    let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
    for py in 0..n {
        for px in 0..n {
            // Texel center → world point.
            let wx = lo[0] + (px as f64 + 0.5) / res as f64 * w;
            let wy = lo[1] + (py as f64 + 0.5) / res as f64 * h;
            let (mut sr, mut sg, mut sb, mut sa, mut sw) = (0.0, 0.0, 0.0, 0.0, 0.0);
            for (i, gp) in points.iter().enumerate() {
                let (dx, dy) = (wx - gp.pos[0], wy - gp.pos[1]);
                let mut wgt = gp.influence.max(0.0) / (dx * dx + dy * dy + EPS);
                let jit = gp.jitter.clamp(0.0, 1.0);
                if jit > 0.0 {
                    let noise = hash_noise(px as u32, py as u32, i as u32);
                    // Factor ∈ [1−jit, 1+jit), floored at 0 so the weight stays ≥ 0.
                    wgt *= (1.0 + jit * (noise - 0.5) * 2.0).max(0.0);
                }
                sr += wgt * f64::from(gp.color.r);
                sg += wgt * f64::from(gp.color.g);
                sb += wgt * f64::from(gp.color.b);
                sa += wgt * f64::from(gp.color.a);
                sw += wgt;
            }
            let inv = if sw > 0.0 { 1.0 / sw } else { 0.0 };
            let o = (py * n + px) * 4;
            buf[o] = (sr * inv).round().clamp(0.0, 255.0) as u8;
            buf[o + 1] = (sg * inv).round().clamp(0.0, 255.0) as u8;
            buf[o + 2] = (sb * inv).round().clamp(0.0, 255.0) as u8;
            buf[o + 3] = (sa * inv).round().clamp(0.0, 255.0) as u8;
        }
    }
    Arc::new(buf)
}

/// Fill `bp` with a multi-point gradient: rasterize the IDW blend over the path's
/// world bbox, clip to the (screen-space) path, and blit the image scaled to the
/// bbox. Reuses the tested clip + image-blit path (mirror of the BgRemoval overlay).
pub(crate) fn fill_multipoint(
    target: &mut VectorScene,
    bp: &BezPath,
    path: &VecPath,
    points: &[GradientPoint],
    transform: Affine,
) {
    if points.is_empty() {
        return;
    }
    let (lo, hi) = control_point_bounds(path);
    let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let img = rasterize_idw(points, lo, hi, IDW_RES);
    // Clip to the path in screen space. `VectorScene::push_clip` hardcodes NonZero,
    // which would paint the gradient over a compound's hole — push the clip layer
    // with the path's OWN fill rule instead.
    let mut screen_bp = bp.clone();
    screen_bp.apply_affine(transform);
    target.push_clip_with_rule(&screen_bp, crate::fill_rule(path));
    // Image pixels (0..res) → world bbox → screen.
    let px_to_world = Affine::translate((lo[0], lo[1]))
        * Affine::scale_non_uniform(w / f64::from(IDW_RES), h / f64::from(IDW_RES));
    target.draw_image_rgba_transformed(
        &img,
        IDW_RES,
        IDW_RES,
        transform * px_to_world,
        ImageQuality::High,
    );
    target.pop_layer();
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::Rgba8;

    #[test]
    fn idw_raster_is_exact_at_points_and_blends_between() {
        let red = Rgba8::new(255, 0, 0, 255);
        let blue = Rgba8::new(0, 0, 255, 255);
        let pts = [
            GradientPoint::new([0.0, 0.5], red, 1.0),
            GradientPoint::new([1.0, 0.5], blue, 1.0),
        ];
        let res = 64u32;
        let buf = rasterize_idw(&pts, [0.0, 0.0], [1.0, 1.0], res);
        let texel = |px: u32, py: u32| {
            let o = ((py * res + px) * 4) as usize;
            [buf[o], buf[o + 1], buf[o + 2], buf[o + 3]]
        };
        // Near the left point → almost pure red; near the right → almost pure blue.
        let left = texel(0, res / 2);
        let right = texel(res - 1, res / 2);
        assert!(left[0] > 240 && left[2] < 15, "left ≈ red, got {left:?}");
        assert!(
            right[2] > 240 && right[0] < 15,
            "right ≈ blue, got {right:?}"
        );
        // Middle column blends both channels (neither is ~0 or ~255).
        let mid = texel(res / 2, res / 2);
        assert!(
            mid[0] > 40 && mid[0] < 215 && mid[2] > 40 && mid[2] < 215,
            "mid blends, got {mid:?}"
        );
        // Empty points → all-zero buffer (no divide-by-zero).
        let empty = rasterize_idw(&[], [0.0, 0.0], [1.0, 1.0], 4);
        assert!(empty.iter().all(|&b| b == 0));
    }

    #[test]
    fn jitter_perturbs_the_blend_but_zero_is_identical() {
        let red = Rgba8::new(255, 0, 0, 255);
        let blue = Rgba8::new(0, 0, 255, 255);
        let res = 32u32;
        let base = [
            GradientPoint::new([0.0, 0.5], red, 1.0),
            GradientPoint::new([1.0, 0.5], blue, 1.0),
        ];
        // jitter = 0 everywhere → byte-identical to the plain blend (default preserved).
        let plain = rasterize_idw(&base, [0.0, 0.0], [1.0, 1.0], res);
        let zero_jit = [
            GradientPoint::with_jitter([0.0, 0.5], red, 1.0, 0.0),
            GradientPoint::with_jitter([1.0, 0.5], blue, 1.0, 0.0),
        ];
        let same = rasterize_idw(&zero_jit, [0.0, 0.0], [1.0, 1.0], res);
        assert_eq!(*plain, *same, "jitter=0 must be byte-identical");
        // A non-zero jitter changes at least some texels (grain), but never the
        // exact colour at a point (its weight dominates → factor is irrelevant there).
        let jit = [
            GradientPoint::with_jitter([0.0, 0.5], red, 1.0, 0.8),
            GradientPoint::with_jitter([1.0, 0.5], blue, 1.0, 0.8),
        ];
        let noisy = rasterize_idw(&jit, [0.0, 0.0], [1.0, 1.0], res);
        assert_ne!(*plain, *noisy, "jitter>0 must perturb the blend");
        // Deterministic: same inputs → same buffer (no frame crawl).
        let noisy2 = rasterize_idw(&jit, [0.0, 0.0], [1.0, 1.0], res);
        assert_eq!(*noisy, *noisy2, "jitter must be deterministic");
    }

    #[test]
    fn interior_stops_hit_drag_and_cross_along_the_ramp() {
        use ph2d_vec_scene::{GradientStop, VecVertex};
        let mut path = VecPath {
            verts: vec![
                VecVertex::corner([0.0, 0.0]),
                VecVertex::corner([10.0, 0.0]),
            ],
            closed: true,
            fill: Some(Paint::Linear {
                stops: vec![
                    GradientStop::new(0.0, Rgba8::new(255, 0, 0, 255)), // end
                    GradientStop::new(0.3, Rgba8::new(0, 255, 0, 255)), // interior 1
                    GradientStop::new(0.6, Rgba8::new(255, 255, 0, 255)), // interior 2
                    GradientStop::new(1.0, Rgba8::new(0, 0, 255, 255)), // end
                ],
                start: [0.0, 0.0],
                end: [10.0, 0.0],
            }),
            ..VecPath::default()
        };
        // Interior stops are indices 1..len-1; the two ends aren't markers.
        assert_eq!(interior_stop_indices(4).collect::<Vec<_>>(), vec![1, 2]);
        // Interior stop 1 sits at offset 0.3 → world (3,0); hit-test there → Stop(1).
        assert_eq!(
            hit_gradient_handle(&path, 3.0, 0.0, 0.5),
            Some(GradHandle::Stop(1))
        );
        // Drag stop 1 to x=2 → offset 0.2 (strictly inside the ends).
        assert!(drag_gradient_handle(
            &mut path,
            GradHandle::Stop(1),
            2.0,
            0.0
        ));
        // Now drag stop 1 PAST stop 2 (to x=8 → offset 0.8): crossing is allowed —
        // stop 1's offset now exceeds stop 2's, and its Vec index is unchanged.
        assert!(drag_gradient_handle(
            &mut path,
            GradHandle::Stop(1),
            8.0,
            0.0
        ));
        if let Some(Paint::Linear { stops, .. }) = &path.fill {
            assert!((stops[1].offset - 0.8).abs() < 1e-6);
            assert!(
                stops[1].offset > stops[2].offset,
                "interior stops may cross"
            );
            // The ends stay pinned at 0 / 1 (interior can't reach them).
            assert_eq!(stops[0].offset, 0.0);
            assert_eq!(stops[3].offset, 1.0);
        } else {
            panic!("fill must stay Linear");
        }
    }
}
