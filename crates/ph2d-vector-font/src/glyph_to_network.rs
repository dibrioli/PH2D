//! Glyph outline → [`VectorNetwork`] (ADR-0066 §2.2): a glyph **is** a vector
//! network, one [`Region`] per closed contour, `NonZero` winding (the OT/SVG
//! canon that resolves holes — `O`, `B`, `8`…).
//!
//! The input is a flat [`GlyphOutline`] of [`PathCommand`]s — exactly what a font
//! outliner (skrifa's `OutlinePen`) emits — so this conversion is **decoupled
//! from skrifa** and golden-testable with hand-built outlines.
//!
//! ## Coordinate convention
//!
//! Font design units are **y-up**; the vector module is **y-down** (screen, as
//! `primitives::ellipse`). We map `(x, y) → (x·scale, −y·scale)`. The Y flip
//! reverses contour orientation, but `NonZero` only cares that holes wind
//! *opposite* the outer contour — true before and after the flip — so holes
//! still subtract.
//!
//! ## Quadratics → cubics
//!
//! TrueType outlines are quadratic. A quad `S–Q–E` elevates exactly to a cubic
//! with handles `out = ⅔(Q−S)`, `in = ⅔(Q−E)` (the substrate's
//! `out_at_start`/`in_at_end` tangent vectors).

use glam::Vec2;
use ph2d_vector_doc::{Region, SegmentId, Segment, VectorNetwork, Vertex, VertexId, WindingRule};

/// One command of a glyph outline, in font design units (y-up). Mirrors a font
/// outliner's pen calls.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PathCommand {
    /// Start a new contour at this point.
    MoveTo(Vec2),
    /// Straight line to this point.
    LineTo(Vec2),
    /// Quadratic Bézier (one control) to `to`.
    QuadTo { ctrl: Vec2, to: Vec2 },
    /// Cubic Bézier (two controls) to `to`.
    CurveTo { c1: Vec2, c2: Vec2, to: Vec2 },
    /// Close the current contour.
    Close,
}

/// A glyph's outline: pen commands plus the font's units-per-em (the scale
/// reference). Produced by [`crate::skrifa_bridge`]; consumed by
/// [`outline_to_network`].
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphOutline {
    pub commands: Vec<PathCommand>,
    pub units_per_em: u16,
}

impl GlyphOutline {
    pub fn new(commands: Vec<PathCommand>, units_per_em: u16) -> Self {
        Self {
            commands,
            units_per_em,
        }
    }
}

/// Convert an outline to a [`VectorNetwork`] in **em space**: coordinates scaled
/// by `1 / units_per_em` (so the em square is `[0,1]` wide, y-flipped). This is
/// the canonical, size-independent form a downstream transform then places.
pub fn outline_to_network_em(outline: &GlyphOutline) -> VectorNetwork {
    let upem = outline.units_per_em.max(1) as f32;
    outline_to_network(outline, 1.0 / upem)
}

/// Convert an outline to a [`VectorNetwork`], scaling font units by `scale` and
/// flipping Y (font y-up → screen y-down). Each closed contour becomes one
/// `NonZero` [`Region`].
pub fn outline_to_network(outline: &GlyphOutline, scale: f32) -> VectorNetwork {
    let mut b = Builder::new(scale);
    for cmd in &outline.commands {
        match *cmd {
            PathCommand::MoveTo(p) => b.move_to(p),
            PathCommand::LineTo(p) => b.line_to(p),
            PathCommand::QuadTo { ctrl, to } => b.quad_to(ctrl, to),
            PathCommand::CurveTo { c1, c2, to } => b.curve_to(c1, c2, to),
            PathCommand::Close => b.close(),
        }
    }
    b.finish()
}

/// State for one contour being built.
struct Contour {
    start_vid: VertexId,
    start_pos: Vec2,
    cur_vid: VertexId,
    cur_pos: Vec2,
    seg_ids: Vec<SegmentId>,
}

struct Builder {
    scale: f32,
    /// Squared "same point" tolerance (¼ font unit) — distinct integer anchors
    /// are ≥ 1 unit apart, so this only ever fuses a contour's closing point.
    eps_sq: f32,
    net: VectorNetwork,
    next_v: VertexId,
    next_s: SegmentId,
    next_r: u32,
    contour: Option<Contour>,
}

impl Builder {
    fn new(scale: f32) -> Self {
        let e = scale * 0.25;
        Self {
            scale,
            eps_sq: e * e,
            net: VectorNetwork::empty(),
            next_v: 0,
            next_s: 0,
            next_r: 0,
            contour: None,
        }
    }

    /// Font units (y-up) → vector space (scaled, y-down).
    fn tf(&self, p: Vec2) -> Vec2 {
        Vec2::new(p.x * self.scale, -p.y * self.scale)
    }

    fn new_vertex(&mut self, pos: Vec2) -> VertexId {
        let id = self.next_v;
        self.next_v += 1;
        self.net.vertices.push(Vertex::auto(id, pos));
        id
    }

    fn move_to(&mut self, p: Vec2) {
        self.close(); // finalize any open contour first
        let pos = self.tf(p);
        let vid = self.new_vertex(pos);
        self.contour = Some(Contour {
            start_vid: vid,
            start_pos: pos,
            cur_vid: vid,
            cur_pos: pos,
            seg_ids: Vec::new(),
        });
    }

    /// The vertex to end a segment at `to`: the contour's start vertex when `to`
    /// closes the loop (fusing the duplicate endpoint), else a fresh vertex.
    fn endpoint_vertex(&mut self, to: Vec2) -> VertexId {
        if let Some(c) = &self.contour
            && (to - c.start_pos).length_squared() <= self.eps_sq
            && !c.seg_ids.is_empty()
        {
            return c.start_vid;
        }
        self.new_vertex(to)
    }

    fn push_segment(&mut self, end_vid: VertexId, end_pos: Vec2, out_at_start: Vec2, in_at_end: Vec2) {
        let Some(start_vid) = self.contour.as_ref().map(|c| c.cur_vid) else {
            return; // a draw command with no MoveTo — ignore (malformed)
        };
        let sid = self.next_s;
        self.next_s += 1;
        let mut seg = Segment::straight(sid, start_vid, end_vid);
        seg.out_at_start = out_at_start;
        seg.in_at_end = in_at_end;
        self.net.segments.push(seg);
        if let Some(c) = &mut self.contour {
            c.seg_ids.push(sid);
            c.cur_vid = end_vid;
            c.cur_pos = end_pos;
        }
    }

    fn line_to(&mut self, p: Vec2) {
        if self.contour.is_none() {
            return;
        }
        let to = self.tf(p);
        let end = self.endpoint_vertex(to);
        self.push_segment(end, to, Vec2::ZERO, Vec2::ZERO);
    }

    fn quad_to(&mut self, ctrl: Vec2, to: Vec2) {
        let Some(start) = self.contour.as_ref().map(|c| c.cur_pos) else {
            return;
        };
        let q = self.tf(ctrl);
        let e = self.tf(to);
        // Exact quad → cubic elevation.
        let out_at_start = (q - start) * (2.0 / 3.0);
        let in_at_end = (q - e) * (2.0 / 3.0);
        let end = self.endpoint_vertex(e);
        self.push_segment(end, e, out_at_start, in_at_end);
    }

    fn curve_to(&mut self, c1: Vec2, c2: Vec2, to: Vec2) {
        let Some(start) = self.contour.as_ref().map(|c| c.cur_pos) else {
            return;
        };
        let p1 = self.tf(c1);
        let p2 = self.tf(c2);
        let e = self.tf(to);
        let out_at_start = p1 - start;
        let in_at_end = p2 - e;
        let end = self.endpoint_vertex(e);
        self.push_segment(end, e, out_at_start, in_at_end);
    }

    fn close(&mut self) {
        let Some(c) = self.contour.take() else {
            return;
        };
        let mut seg_ids = c.seg_ids;
        // Connect back to the start if the pen didn't already land there.
        if c.cur_vid != c.start_vid {
            let sid = self.next_s;
            self.next_s += 1;
            self.net
                .segments
                .push(Segment::straight(sid, c.cur_vid, c.start_vid));
            seg_ids.push(sid);
        }
        if seg_ids.len() < 2 {
            return; // a point / single edge — no fillable region
        }
        let rid = self.next_r;
        self.next_r += 1;
        let mut region = Region::new(rid, WindingRule::NonZero);
        region.segments = seg_ids.into_iter().map(|s| (s, true)).collect();
        self.net.regions.push(region);
    }

    fn finish(mut self) -> VectorNetwork {
        self.close(); // an unterminated contour still closes (OT implies it)
        self.net
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upem(commands: Vec<PathCommand>) -> GlyphOutline {
        GlyphOutline::new(commands, 1000)
    }

    #[test]
    fn triangle_is_one_region_three_segments() {
        // Closed triangle in font units (y-up).
        let net = outline_to_network_em(&upem(vec![
            PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
            PathCommand::LineTo(Vec2::new(600.0, 0.0)),
            PathCommand::LineTo(Vec2::new(300.0, 600.0)),
            PathCommand::Close,
        ]));
        assert!(net.validate().is_ok());
        assert_eq!(net.regions.len(), 1);
        assert_eq!(net.vertices.len(), 3, "closing fused the loop, no 4th vertex");
        assert_eq!(net.segments.len(), 3);
        assert_eq!(net.regions[0].winding, WindingRule::NonZero);
    }

    #[test]
    fn y_is_flipped_to_screen_space() {
        let net = outline_to_network_em(&upem(vec![
            PathCommand::MoveTo(Vec2::new(0.0, 1000.0)), // top in font space
            PathCommand::LineTo(Vec2::new(1000.0, 1000.0)),
            PathCommand::LineTo(Vec2::new(0.0, 0.0)),
            PathCommand::Close,
        ]));
        // y=1000 font (top) → y=-1.0 screen (up is negative).
        assert!(net.vertices.iter().any(|v| (v.pos.y + 1.0).abs() < 1e-5));
        assert!(net.vertices.iter().any(|v| v.pos.y.abs() < 1e-5));
    }

    #[test]
    fn quadratic_elevates_to_cubic_handles() {
        // One quad from (0,0) via ctrl (500,1000) to (1000,0).
        let net = outline_to_network_em(&upem(vec![
            PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
            PathCommand::QuadTo {
                ctrl: Vec2::new(500.0, 1000.0),
                to: Vec2::new(1000.0, 0.0),
            },
            PathCommand::LineTo(Vec2::new(0.0, 0.0)),
            PathCommand::Close,
        ]));
        let quad_seg = &net.segments[0];
        // out = ⅔(Q−S): Q=(0.5,-1.0) S=(0,0) → (⅓, −⅔).
        assert!((quad_seg.out_at_start.x - 1.0 / 3.0).abs() < 1e-5);
        assert!((quad_seg.out_at_start.y + 2.0 / 3.0).abs() < 1e-5);
        // in = ⅔(Q−E): E=(1.0,0) → (−⅓, −⅔).
        assert!((quad_seg.in_at_end.x + 1.0 / 3.0).abs() < 1e-5);
        assert!((quad_seg.in_at_end.y + 2.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn glyph_with_hole_is_two_regions() {
        // "O" topology: outer contour + inner contour (opposite winding).
        let net = outline_to_network_em(&upem(vec![
            // outer (CCW in font space)
            PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
            PathCommand::LineTo(Vec2::new(800.0, 0.0)),
            PathCommand::LineTo(Vec2::new(800.0, 800.0)),
            PathCommand::LineTo(Vec2::new(0.0, 800.0)),
            PathCommand::Close,
            // inner hole (CW)
            PathCommand::MoveTo(Vec2::new(200.0, 200.0)),
            PathCommand::LineTo(Vec2::new(200.0, 600.0)),
            PathCommand::LineTo(Vec2::new(600.0, 600.0)),
            PathCommand::LineTo(Vec2::new(600.0, 200.0)),
            PathCommand::Close,
        ]));
        assert!(net.validate().is_ok());
        assert_eq!(net.regions.len(), 2, "outer + hole");
        assert_eq!(net.vertices.len(), 8);
        assert!(net.regions.iter().all(|r| r.winding == WindingRule::NonZero));
    }

    #[test]
    fn malformed_draw_without_moveto_is_ignored() {
        let net = outline_to_network_em(&upem(vec![
            PathCommand::LineTo(Vec2::new(10.0, 10.0)),
            PathCommand::Close,
        ]));
        assert!(net.validate().is_ok());
        assert!(net.regions.is_empty());
        assert!(net.segments.is_empty());
    }

    #[test]
    fn cubic_handles_are_relative_vectors() {
        let net = outline_to_network_em(&upem(vec![
            PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
            PathCommand::CurveTo {
                c1: Vec2::new(300.0, 0.0),
                c2: Vec2::new(700.0, 0.0),
                to: Vec2::new(1000.0, 0.0),
            },
            PathCommand::LineTo(Vec2::new(0.0, 0.0)),
            PathCommand::Close,
        ]));
        let seg = &net.segments[0];
        // out = c1 − S = (0.3, 0); in = c2 − E = (0.7−1.0, 0) = (−0.3, 0).
        assert!((seg.out_at_start.x - 0.3).abs() < 1e-5);
        assert!((seg.in_at_end.x + 0.3).abs() < 1e-5);
    }
}
