//! Brush-along-path — the RASTER half of W8 pattern-along-path.
//!
//! Strokes a brush along a vector path: walk the path by arc length, drop a
//! [`Stamp`] every `spacing` pixels (rotated to the tangent when asked), and
//! optionally rasterize them with the painter's CPU stamp kernel. This is the
//! satellite bridge the vector W8 handoff deferred — it READS the two frozen
//! contracts ([`VectorNetwork`] = the path, [`Stamp`] = the dab) and touches
//! neither crate, so it stays an isolated drop-crate (no node roster / cap
//! change, no new `Domain::Raster` foundational surface). A future raster
//! graph-node or a Painter stroke-along-path op can both build on it.
//!
//! ## Following the real curve (mirrors `ph2d-node-vector-pattern-along-path`)
//!
//! The path's segments are cubic (`out_at_start`/`in_at_end` handles, the
//! `c1 = start + out`, `c2 = end + in` convention `primitives::ellipse` uses).
//! We order the segments into one chain, flatten each cubic to a dense polyline,
//! and sample frames by ARC LENGTH — so dabs sit *on* the curve, not its chords.
//! Unlike the shape-placer (which spaces `count` copies), a brush stroke wants
//! CONTINUOUS coverage, so frames are spaced by DISTANCE (`size · spacing_ratio`).

use std::collections::BTreeMap;

use glam::Vec2;
use ph2d_painter_brush::Stamp;
use ph2d_vector_doc::{VectorNetwork, VertexId};

/// Cubic-flattening samples per path segment — matches the vector pattern-along-
/// path engine so the two follow the same polyline.
const FLATTEN_STEPS: usize = 24;

/// Smallest stamp spacing (px). Guards a degenerate `size · ratio` from emitting
/// an unbounded number of dabs along a long path.
const MIN_SPACING_PX: f32 = 0.5;

/// Hard ceiling on emitted stamps — bounds OOM from a tiny spacing over a long
/// path (a pathological input), well above any real stroke.
pub const MAX_STAMPS: usize = 1 << 16;

/// How to stroke a brush along a path. The dab fields mirror [`Stamp`]'s
/// user-facing knobs; `spacing_ratio` sets the dab pitch as a fraction of the
/// brush diameter (≈0.25 reads as a continuous stroke).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BrushAlongPathParams {
    /// Brush diameter in canvas pixels (the dab `size_px`).
    pub size_px: f32,
    /// Dab pitch = `size_px · spacing_ratio` along the arc (clamped ≥ 0.5 px).
    pub spacing_ratio: f32,
    /// Dab colour, OKLab straight `[L, a, b, α]` (the `Stamp.color_oklab` space).
    pub color_oklab: [f32; 4],
    /// Per-dab opacity `[0, 1]`.
    pub opacity: f32,
    /// Per-dab flow `[0, 1]`.
    pub flow: f32,
    /// Brush shape-atlas slot: 0 = round_hard, 1 = round_soft, 2 = square_hard,
    /// 3 = oval_hard (the 4 procedural slots shipped, ADR-0044).
    pub shape_layer: u32,
    /// `RenderingMode` discriminant `0..=5`.
    pub rendering_mode: u32,
    /// Rotate each dab to the path tangent. Only visible for non-radial shapes
    /// (square/oval); a round brush is rotation-invariant.
    pub align_to_tangent: bool,
}

impl Default for BrushAlongPathParams {
    fn default() -> Self {
        Self {
            size_px: 20.0,
            spacing_ratio: 0.25,
            color_oklab: [0.0, 0.0, 0.0, 1.0], // opaque black (OKLab L=0)
            opacity: 1.0,
            flow: 1.0,
            shape_layer: 0,
            rendering_mode: 0,
            align_to_tangent: false,
        }
    }
}

/// A placement frame: a point on the path and the unit tangent there.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Frame {
    pos: Vec2,
    dir: Vec2,
}

/// Build the dabs that stroke `params`'s brush along `path`, spaced by arc length
/// (the first dab at the path start, then every `size_px · spacing_ratio` px, the
/// last ≤ the total length). Empty / point-only / zero-size inputs → no dabs.
#[must_use]
pub fn stamps_along_path(path: &VectorNetwork, params: &BrushAlongPathParams) -> Vec<Stamp> {
    if params.size_px <= 0.0 {
        return Vec::new();
    }
    let spacing = (params.size_px * params.spacing_ratio).max(MIN_SPACING_PX);
    let frames = frames_by_spacing(path, spacing);
    frames
        .iter()
        .map(|f| {
            let mut s = Stamp::zeroed(); // grain_layer = u32::MAX, _pad = 0
            s.position_world = [f.pos.x, f.pos.y];
            s.size_px = params.size_px;
            // `rotation_rad` is an ANGLE field, so align needs atan2 (sub-pixel
            // ULP across platforms is irrelevant for a raster dab; the painter's
            // det-mode snaps stamps itself when byte-identity is required).
            s.rotation_rad = if params.align_to_tangent {
                f.dir.y.atan2(f.dir.x)
            } else {
                0.0
            };
            s.pressure = 1.0;
            s.color_oklab = params.color_oklab;
            s.opacity = params.opacity;
            s.flow = params.flow;
            s.shape_layer = params.shape_layer;
            s.rendering_mode = params.rendering_mode;
            s
        })
        .collect()
}

/// Convenience: stroke the brush along `path` into a straight-RGBA8 `canvas`
/// (`width · height · 4` bytes) via the painter's CPU stamp kernel. The bridge's
/// end-to-end: path → dabs → pixels.
pub fn rasterize_along_path(
    path: &VectorNetwork,
    params: &BrushAlongPathParams,
    canvas: &mut [u8],
    width: u32,
    height: u32,
) {
    let stamps = stamps_along_path(path, params);
    ph2d_painter_brush::apply_stamps(canvas, width, height, &stamps);
}

// ─────────────────────────── path → arc-length frames ───────────────────────

/// Frames spaced every `spacing` px by arc length (inclusive of the start). A
/// continuous-stroke pitch — distinct from the shape-placer's `count` spacing.
fn frames_by_spacing(path: &VectorNetwork, spacing: f32) -> Vec<Frame> {
    let poly = flatten_path(path);
    if poly.len() < 2 {
        return Vec::new();
    }
    let mut cum = vec![0.0_f32; poly.len()];
    for i in 1..poly.len() {
        cum[i] = cum[i - 1] + (poly[i] - poly[i - 1]).length();
    }
    let total = cum[poly.len() - 1];
    if total <= f32::EPSILON {
        return Vec::new();
    }
    let spacing = spacing.max(MIN_SPACING_PX);
    let n = ((total / spacing).floor() as usize + 1).min(MAX_STAMPS);
    (0..n)
        .map(|i| sample_arc(&poly, &cum, i as f32 * spacing))
        .collect()
}

/// Point + unit tangent at arc length `s` along the polyline.
fn sample_arc(poly: &[Vec2], cum: &[f32], s: f32) -> Frame {
    let s = s.clamp(0.0, cum[cum.len() - 1]);
    let mut j = 0;
    while j + 2 < poly.len() && cum[j + 1] < s {
        j += 1;
    }
    let seg_len = (cum[j + 1] - cum[j]).max(f32::EPSILON);
    let t = ((s - cum[j]) / seg_len).clamp(0.0, 1.0);
    let pos = poly[j].lerp(poly[j + 1], t);
    let mut dir = (poly[j + 1] - poly[j]).normalize_or_zero();
    if dir == Vec2::ZERO {
        dir = Vec2::X;
    }
    Frame { pos, dir }
}

/// Flatten the path's (ordered) cubic segments into one dense polyline.
fn flatten_path(path: &VectorNetwork) -> Vec<Vec2> {
    let ordered = ordered_segments(path);
    if ordered.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<Vec2> = Vec::new();
    for (idx, fwd) in ordered {
        let s = &path.segments[idx];
        let p0 = vertex_pos(path, s.start);
        let p3 = vertex_pos(path, s.end);
        let c1 = p0 + s.out_at_start; // primitives::ellipse convention
        let c2 = p3 + s.in_at_end;
        let (a0, a1, a2, a3) = if fwd {
            (p0, c1, c2, p3)
        } else {
            (p3, c2, c1, p0)
        };
        let first_k = if out.is_empty() { 0 } else { 1 }; // skip the join point
        for k in first_k..=FLATTEN_STEPS {
            let t = k as f32 / FLATTEN_STEPS as f32;
            out.push(cubic_eval(a0, a1, a2, a3, t));
        }
    }
    out
}

/// Order the path's segments into one connected chain (start at an open end, else
/// the smallest vertex id), each tagged whether traversed forward. Only the
/// component containing the start is walked. Deterministic via [`BTreeMap`].
fn ordered_segments(net: &VectorNetwork) -> Vec<(usize, bool)> {
    if net.segments.is_empty() {
        return Vec::new();
    }
    let mut adj: BTreeMap<VertexId, Vec<(usize, bool)>> = BTreeMap::new();
    for (i, s) in net.segments.iter().enumerate() {
        adj.entry(s.start).or_default().push((i, true));
        adj.entry(s.end).or_default().push((i, false));
    }
    let start = adj
        .iter()
        .find(|(_, e)| e.len() == 1)
        .map(|(v, _)| *v)
        .or_else(|| adj.keys().next().copied());
    let Some(mut cur) = start else {
        return Vec::new();
    };
    let mut visited = vec![false; net.segments.len()];
    let mut out = Vec::new();
    loop {
        let next = adj
            .get(&cur)
            .and_then(|edges| edges.iter().find(|(i, _)| !visited[*i]).copied());
        let Some((i, at_start)) = next else {
            break;
        };
        visited[i] = true;
        out.push((i, at_start));
        cur = if at_start {
            net.segments[i].end
        } else {
            net.segments[i].start
        };
    }
    out
}

fn vertex_pos(net: &VectorNetwork, id: VertexId) -> Vec2 {
    net.vertices
        .iter()
        .find(|v| v.id == id)
        .map_or(Vec2::ZERO, |v| v.pos)
}

/// Cubic Bézier point at `t`.
fn cubic_eval(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector_doc::{Segment, Vertex};

    /// A straight horizontal path from x=0 to x=len on y=0.
    fn straight_path(len: f32) -> VectorNetwork {
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(len, 0.0)));
        net.segments.push(Segment::straight(0, 0, 1));
        net
    }

    #[test]
    fn dabs_spaced_by_arc_length_along_straight_path() {
        // len 100, size 20, ratio 0.5 → spacing 10 → dabs at 0,10,…,100 = 11.
        let p = BrushAlongPathParams {
            size_px: 20.0,
            spacing_ratio: 0.5,
            ..Default::default()
        };
        let stamps = stamps_along_path(&straight_path(100.0), &p);
        assert_eq!(stamps.len(), 11, "0,10,…,100");
        // Positions march along x; y stays on the path.
        for (i, s) in stamps.iter().enumerate() {
            assert!(
                (s.position_world[0] - i as f32 * 10.0).abs() < 0.5,
                "dab {i} x"
            );
            assert!(s.position_world[1].abs() < 0.5, "dab {i} y on path");
            assert_eq!(s.size_px, 20.0);
            assert_eq!(s.grain_layer, u32::MAX, "no grain (zeroed ctor)");
            s.assert_pad_zero();
        }
    }

    #[test]
    fn align_rotates_dab_to_tangent() {
        // Vertical path → tangent (0,1) → rotation_rad = atan2(1,0) = π/2.
        let mut vpath = VectorNetwork::empty();
        vpath.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        vpath.vertices.push(Vertex::auto(1, Vec2::new(0.0, 100.0)));
        vpath.segments.push(Segment::straight(0, 0, 1));
        let p = BrushAlongPathParams {
            size_px: 10.0,
            spacing_ratio: 1.0,
            align_to_tangent: true,
            ..Default::default()
        };
        let stamps = stamps_along_path(&vpath, &p);
        assert!(!stamps.is_empty());
        for s in &stamps {
            assert!(
                (s.rotation_rad - std::f32::consts::FRAC_PI_2).abs() < 1e-4,
                "tangent rotation π/2, got {}",
                s.rotation_rad
            );
        }
        // Without align, rotation stays 0.
        let p0 = BrushAlongPathParams {
            align_to_tangent: false,
            ..p
        };
        assert!(
            stamps_along_path(&vpath, &p0)
                .iter()
                .all(|s| s.rotation_rad == 0.0)
        );
    }

    #[test]
    fn follows_cubic_curve_not_chord() {
        // One cubic that bulges up: a mid dab must sit above the straight chord.
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(100.0, 0.0)));
        let mut seg = Segment::straight(0, 0, 1);
        seg.out_at_start = Vec2::new(0.0, 80.0); // c1 = (0,80)
        seg.in_at_end = Vec2::new(0.0, 80.0); // c2 = (100,80)
        net.segments.push(seg);
        let p = BrushAlongPathParams {
            size_px: 10.0,
            spacing_ratio: 0.5,
            ..Default::default()
        };
        let stamps = stamps_along_path(&net, &p);
        let max_y = stamps
            .iter()
            .map(|s| s.position_world[1])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max_y > 30.0,
            "dabs follow the bulging cubic, peak y = {max_y}"
        );
    }

    #[test]
    fn rasterize_inks_along_path_only() {
        // Horizontal stroke through the middle of a 64×64 canvas.
        let (w, h) = (64u32, 64u32);
        let mut canvas = vec![0u8; (w * h * 4) as usize];
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(8.0, 32.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(56.0, 32.0)));
        net.segments.push(Segment::straight(0, 0, 1));
        let p = BrushAlongPathParams {
            size_px: 8.0,
            spacing_ratio: 0.25,
            color_oklab: [0.7, 0.1, 0.05, 1.0],
            ..Default::default()
        };
        rasterize_along_path(&net, &p, &mut canvas, w, h);
        let alpha = |x: u32, y: u32| canvas[((y * w + x) * 4 + 3) as usize];
        // The stroke centre line is inked…
        assert!(alpha(32, 32) > 0, "stroke centre should be inked");
        // …and a far corner stays clear.
        assert_eq!(alpha(2, 2), 0, "off-path corner stays clear");
    }

    #[test]
    fn degenerate_inputs_emit_no_dabs() {
        let p = BrushAlongPathParams::default();
        assert!(stamps_along_path(&VectorNetwork::empty(), &p).is_empty());
        // Zero size → nothing.
        let zero = BrushAlongPathParams { size_px: 0.0, ..p };
        assert!(stamps_along_path(&straight_path(100.0), &zero).is_empty());
    }

    #[test]
    fn reproducible_on_the_same_inputs() {
        let p = BrushAlongPathParams {
            size_px: 12.0,
            align_to_tangent: true,
            ..Default::default()
        };
        let a = stamps_along_path(&straight_path(77.0), &p);
        let b = stamps_along_path(&straight_path(77.0), &p);
        assert_eq!(a.len(), b.len());
        assert!(
            a.iter()
                .zip(&b)
                .all(|(x, y)| x.position_world == y.position_world
                    && x.rotation_rad == y.rotation_rad),
            "same inputs → same dabs"
        );
    }
}
