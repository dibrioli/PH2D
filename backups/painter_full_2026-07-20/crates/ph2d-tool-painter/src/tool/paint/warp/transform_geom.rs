//! Pure geometry for the **Transform** temperament (Deform Wave 2) — the affine primitive, the gizmo frame,
//! and the drift-free handle drags. No `PainterTool`, no pixels: just math, so it's unit-testable in
//! isolation and keeps [`super::transform`] (the tool-side wiring) under the file-LOC cap. All rotation is
//! transcendental-free (a rotor `[cos, sin]` derived from drag vectors via dot/cross), so HR-5 holds.

/// Smallest half-extent a scale drag allows (image px) — mirrors the selection gizmo.
pub(super) const MIN_AXIS_PX: f32 = 1.0;
/// The rotate ring reaches this many grab-radii beyond a scale square (mirrors the selection gizmo).
pub(super) const ROTATE_BAND: f32 = 2.6;
// Unified handle ids: 0..=3 corners (TL,TR,BR,BL), 4..=7 edges (R,T,L,B), 8 rotate, 9 centre-move.
const H_SCALE_END: u8 = 8;
const H_ROTATE: u8 = 8;
const H_MOVE: u8 = 9;

/// A 2×3 affine `[[a, c, e], [b, d, f]]` mapping `p → (a·x + c·y + e, b·x + d·y + f)`. Row-major by
/// column: `a d` are the diagonal, `c b` the shear/rotation off-diagonal, `e f` the translation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct Affine2 {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Affine2 {
    /// The identity (no transform).
    #[allow(dead_code)] // primitive used by tests + kept for API completeness (built via `affine_from_frames`)
    pub(super) const IDENTITY: Affine2 = Affine2 {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    /// The affine mapping unit-frame coords `(s, t)` to `origin + s·ex + t·ey` (`ex`/`ey` are the frame's
    /// full axis vectors). Used to build `A0`/`A1` from the pristine/current frames.
    pub(super) fn from_basis(origin: [f32; 2], ex: [f32; 2], ey: [f32; 2]) -> Affine2 {
        Affine2 {
            a: ex[0],
            b: ex[1],
            c: ey[0],
            d: ey[1],
            e: origin[0],
            f: origin[1],
        }
    }

    /// Map a point. (Used by tests + as the reference for `Mat3::from_affine`; the live composite maps
    /// through `Mat3`.)
    #[allow(dead_code)]
    #[inline]
    pub(super) fn apply(&self, p: [f32; 2]) -> [f32; 2] {
        [
            self.a * p[0] + self.c * p[1] + self.e,
            self.b * p[0] + self.d * p[1] + self.f,
        ]
    }

    /// The inverse, or `None` when the linear part is singular (degenerate handle drag).
    pub(super) fn inverse(&self) -> Option<Affine2> {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() < 1e-9 {
            return None;
        }
        let inv = 1.0 / det;
        // Inverse linear 2×2, then the inverse translation `−A⁻¹·t`.
        let (ia, ib, ic, id) = (self.d * inv, -self.b * inv, -self.c * inv, self.a * inv);
        Some(Affine2 {
            a: ia,
            b: ib,
            c: ic,
            d: id,
            e: -(ia * self.e + ic * self.f),
            f: -(ib * self.e + id * self.f),
        })
    }

    /// `self ∘ rhs` — the affine that applies `rhs` first, then `self` (`(self∘rhs)(p) = self.apply(rhs.apply(p))`).
    pub(super) fn compose(&self, rhs: &Affine2) -> Affine2 {
        Affine2 {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            e: self.a * rhs.e + self.c * rhs.f + self.e,
            f: self.b * rhs.e + self.d * rhs.f + self.f,
        }
    }
}

/// The oriented editing frame of the Transform gizmo: `center`, unit axis `u` (local +x), and half-extents
/// `hx`/`hy` (always `> 0`). Its box maps unit coords `(s, t) ∈ [−1, 1]²` to `center + s·hx·u + t·hy·v`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct TransformFrame {
    pub center: [f32; 2],
    pub u: [f32; 2],
    pub hx: f32,
    pub hy: f32,
}

impl TransformFrame {
    fn v(&self) -> [f32; 2] {
        [-self.u[1], self.u[0]]
    }
    /// The unit→world basis affine of this frame (full axis vectors `hx·u`, `hy·v`).
    fn basis(&self) -> Affine2 {
        let v = self.v();
        Affine2::from_basis(
            self.center,
            [self.u[0] * self.hx, self.u[1] * self.hx],
            [v[0] * self.hy, v[1] * self.hy],
        )
    }
    /// `[TL, TR, BR, BL, R, T, L, B]` corners + edge mids in image space.
    pub(super) fn handles(&self) -> [[f32; 2]; 8] {
        let (c, u, v) = (self.center, self.u, self.v());
        let hxu = [u[0] * self.hx, u[1] * self.hx];
        let hyv = [v[0] * self.hy, v[1] * self.hy];
        let mk = |sx: f32, sy: f32| {
            [
                c[0] + sx * hxu[0] + sy * hyv[0],
                c[1] + sx * hxu[1] + sy * hyv[1],
            ]
        };
        [
            mk(-1.0, 1.0),  // 0 TL
            mk(1.0, 1.0),   // 1 TR
            mk(1.0, -1.0),  // 2 BR
            mk(-1.0, -1.0), // 3 BL
            mk(1.0, 0.0),   // 4 R
            mk(0.0, 1.0),   // 5 T
            mk(-1.0, 0.0),  // 6 L
            mk(0.0, -1.0),  // 7 B
        ]
    }
}

/// The affine that maps frame `f0`'s box onto frame `f1`'s box: `M = A1 ∘ A0⁻¹` (unit→world of `f1` after
/// world→unit of `f0`). Handles anisotropic scale + rotation + translation at once, transcendental-free.
/// Returns `None` only if `f0` is degenerate (guarded elsewhere: half-extents are clamped `> 0`).
pub(super) fn affine_from_frames(f0: &TransformFrame, f1: &TransformFrame) -> Option<Affine2> {
    let a0inv = f0.basis().inverse()?;
    Some(f1.basis().compose(&a0inv))
}

/// Hit-test the gizmo's handles at `pos` within `tol`; returns the grabbed handle id (scale square, rotate
/// ring, or centre-move) — mirrors the selection gizmo's `hit_shape`.
pub(super) fn hit_frame(f: &TransformFrame, pos: [f32; 2], tol: f32) -> Option<u8> {
    let handles = f.handles();
    let tol2 = tol * tol;
    // 1) ON a scale square → scale.
    let mut best = None;
    let mut bestd = tol2;
    for (i, h) in handles.iter().enumerate() {
        let d = dist2(*h, pos);
        if d <= bestd {
            bestd = d;
            best = Some(i as u8);
        }
    }
    if let Some(i) = best {
        return Some(i);
    }
    // 2) Centre-move square.
    if dist2(f.center, pos) <= tol2 {
        return Some(H_MOVE);
    }
    // 3) Rotate ring — the band just OUTSIDE a scale square (farther from the centre than it).
    let rot2 = (tol * ROTATE_BAND) * (tol * ROTATE_BAND);
    for h in &handles {
        let d = dist2(*h, pos);
        if d > tol2 && d <= rot2 && dist2(f.center, pos) > dist2(*h, f.center) {
            return Some(H_ROTATE);
        }
    }
    None
}

/// Apply a gizmo drag to the PRISTINE `initial` frame and return the new frame (drift-free). `uniform`
/// locks corner scales to preserve aspect (edge handles are always single-axis).
pub(super) fn drag_frame(
    initial: &TransformFrame,
    handle: u8,
    start: [f32; 2],
    pos: [f32; 2],
    uniform: bool,
) -> TransformFrame {
    let c = initial.center;
    if handle == H_MOVE {
        let d = [pos[0] - start[0], pos[1] - start[1]];
        return TransformFrame {
            center: [c[0] + d[0], c[1] + d[1]],
            ..*initial
        };
    }
    if handle == H_ROTATE {
        let v0 = unit_or([start[0] - c[0], start[1] - c[1]], [1.0, 0.0]);
        let v1 = unit_or([pos[0] - c[0], pos[1] - c[1]], [1.0, 0.0]);
        let cos = dot(v0, v1);
        let sin = v0[0] * v1[1] - v0[1] * v1[0];
        let nu = [
            initial.u[0] * cos - initial.u[1] * sin,
            initial.u[0] * sin + initial.u[1] * cos,
        ];
        return TransformFrame {
            u: unit_or(nu, initial.u),
            ..*initial
        };
    }
    if handle < H_SCALE_END {
        let v = initial.v();
        let rel = [pos[0] - c[0], pos[1] - c[1]];
        let du = dot(rel, initial.u).abs().max(MIN_AXIS_PX);
        let dv = dot(rel, v).abs().max(MIN_AXIS_PX);
        // Corners (0..3) scale both axes; edge R/L (4/6) scale hx; edge T/B (5/7) scale hy.
        let (mut nhx, mut nhy) = match handle {
            0..=3 => (du, dv),
            4 | 6 => (du, initial.hy),
            _ => (initial.hx, dv),
        };
        if uniform && handle <= 3 {
            // Aspect-locked: drive both axes by the pointer's distance ratio to the grabbed corner.
            let grabbed = initial.handles()[handle as usize];
            let d0 = dist(grabbed, c).max(MIN_AXIS_PX);
            let d1 = dist(pos, c).max(MIN_AXIS_PX);
            let s = d1 / d0;
            nhx = (initial.hx * s).max(MIN_AXIS_PX);
            nhy = (initial.hy * s).max(MIN_AXIS_PX);
        }
        return TransformFrame {
            hx: nhx,
            hy: nhy,
            ..*initial
        };
    }
    *initial
}

fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}
fn dist2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}
fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    dist2(a, b).sqrt()
}
/// Unit vector of `a`, or `fallback` when `a` is ~zero-length.
fn unit_or(a: [f32; 2], fallback: [f32; 2]) -> [f32; 2] {
    let m = (a[0] * a[0] + a[1] * a[1]).sqrt();
    if m > 1e-6 {
        [a[0] / m, a[1] / m]
    } else {
        fallback
    }
}

// ── Projective 3×3 (Distort homography) ──────────────────────────────────────────────────────────────

/// A 3×3 projective matrix (row-major `[m00..m22]`) mapping `(x, y, 1) → (X, Y, W)` with a perspective
/// divide. Generalizes [`Affine2`] to the free-corner Distort warp (a quadrilateral, not a parallelogram).
/// Transcendental-free — built by linear solves only (HR-5).
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct Mat3(pub [f32; 9]);

impl Mat3 {
    /// Embed an affine (Uniform / Free transform) as a projective matrix (bottom row `0 0 1`).
    pub(super) fn from_affine(a: Affine2) -> Mat3 {
        Mat3([a.a, a.c, a.e, a.b, a.d, a.f, 0.0, 0.0, 1.0])
    }

    /// Project a point through the matrix (perspective divide); the point at infinity maps to itself-ish
    /// (guarded: a ~zero `w` returns the un-divided numerator so the caller's clamp still yields a finite px).
    pub(super) fn apply(&self, p: [f32; 2]) -> [f32; 2] {
        let m = &self.0;
        let x = m[0] * p[0] + m[1] * p[1] + m[2];
        let y = m[3] * p[0] + m[4] * p[1] + m[5];
        let w = m[6] * p[0] + m[7] * p[1] + m[8];
        if w.abs() < 1e-12 {
            return [x, y];
        }
        [x / w, y / w]
    }

    /// Matrix product `self · rhs`.
    fn mul(&self, rhs: &Mat3) -> Mat3 {
        let a = &self.0;
        let b = &rhs.0;
        let mut o = [0.0f32; 9];
        for r in 0..3 {
            for c in 0..3 {
                o[r * 3 + c] = a[r * 3] * b[c] + a[r * 3 + 1] * b[3 + c] + a[r * 3 + 2] * b[6 + c];
            }
        }
        Mat3(o)
    }

    /// The inverse, or `None` when singular (degenerate quad).
    pub(super) fn inverse(&self) -> Option<Mat3> {
        let m = &self.0;
        let c00 = m[4] * m[8] - m[5] * m[7];
        let c01 = m[5] * m[6] - m[3] * m[8];
        let c02 = m[3] * m[7] - m[4] * m[6];
        let det = m[0] * c00 + m[1] * c01 + m[2] * c02;
        if det.abs() < 1e-12 {
            return None;
        }
        let inv = 1.0 / det;
        // Adjugate (transpose of cofactors) × 1/det.
        Some(Mat3([
            c00 * inv,
            (m[2] * m[7] - m[1] * m[8]) * inv,
            (m[1] * m[5] - m[2] * m[4]) * inv,
            c01 * inv,
            (m[0] * m[8] - m[2] * m[6]) * inv,
            (m[2] * m[3] - m[0] * m[5]) * inv,
            c02 * inv,
            (m[1] * m[6] - m[0] * m[7]) * inv,
            (m[0] * m[4] - m[1] * m[3]) * inv,
        ]))
    }
}

/// The homography mapping the UNIT SQUARE corners `(0,0),(1,0),(1,1),(0,1)` onto quad `q` (`[TL,TR,BR,BL]`
/// order) — Heckbert's closed form. `None` for a degenerate quad.
fn square_to_quad(q: &[[f32; 2]; 4]) -> Option<Mat3> {
    let (x0, y0) = (q[0][0], q[0][1]);
    let (x1, y1) = (q[1][0], q[1][1]);
    let (x2, y2) = (q[2][0], q[2][1]);
    let (x3, y3) = (q[3][0], q[3][1]);
    let dx1 = x1 - x2;
    let dx2 = x3 - x2;
    let dx3 = x0 - x1 + x2 - x3;
    let dy1 = y1 - y2;
    let dy2 = y3 - y2;
    let dy3 = y0 - y1 + y2 - y3;
    let (a, b, c, d, e, f, g, h);
    if dx3.abs() < 1e-9 && dy3.abs() < 1e-9 {
        // Affine (parallelogram).
        a = x1 - x0;
        b = x2 - x1;
        d = y1 - y0;
        e = y2 - y1;
        g = 0.0;
        h = 0.0;
    } else {
        let den = dx1 * dy2 - dx2 * dy1;
        if den.abs() < 1e-12 {
            return None;
        }
        g = (dx3 * dy2 - dx2 * dy3) / den;
        h = (dx1 * dy3 - dx3 * dy1) / den;
        a = x1 - x0 + g * x1;
        b = x3 - x0 + h * x3;
        d = y1 - y0 + g * y1;
        e = y3 - y0 + h * y3;
    }
    c = x0;
    f = y0;
    Some(Mat3([a, b, c, d, e, f, g, h, 1.0]))
}

/// The homography mapping source quad `src` onto destination quad `dst` (both `[TL,TR,BR,BL]`). `None` when
/// either quad is degenerate. Used by Distort: `src` = the pristine box corners, `dst` = the dragged corners.
pub(super) fn homography_from_quads(src: &[[f32; 2]; 4], dst: &[[f32; 2]; 4]) -> Option<Mat3> {
    let s = square_to_quad(src)?;
    let d = square_to_quad(dst)?;
    Some(d.mul(&s.inverse()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(cx: f32, cy: f32, hx: f32, hy: f32) -> TransformFrame {
        TransformFrame {
            center: [cx, cy],
            u: [1.0, 0.0],
            hx,
            hy,
        }
    }

    #[test]
    fn identity_inverse_and_apply() {
        let m = Affine2::IDENTITY;
        assert_eq!(m.apply([3.0, 7.0]), [3.0, 7.0]);
        assert_eq!(m.inverse(), Some(Affine2::IDENTITY));
    }

    #[test]
    fn inverse_round_trips() {
        let m = Affine2 {
            a: 2.0,
            b: 0.3,
            c: -0.4,
            d: 0.5,
            e: 10.0,
            f: -5.0,
        };
        let inv = m.inverse().expect("non-singular");
        for &p in &[[0.0, 0.0], [12.0, 8.0], [-3.0, 20.0]] {
            let round = inv.apply(m.apply(p));
            assert!((round[0] - p[0]).abs() < 1e-3 && (round[1] - p[1]).abs() < 1e-3);
        }
    }

    #[test]
    fn singular_affine_has_no_inverse() {
        let m = Affine2 {
            a: 1.0,
            b: 2.0,
            c: 2.0,
            d: 4.0,
            e: 0.0,
            f: 0.0,
        }; // det = 0
        assert_eq!(m.inverse(), None);
    }

    #[test]
    fn compose_matches_sequential_apply() {
        let s = Affine2 {
            a: 1.5,
            b: 0.2,
            c: -0.3,
            d: 0.9,
            e: 4.0,
            f: -2.0,
        };
        let r = Affine2 {
            a: 0.7,
            b: -0.1,
            c: 0.4,
            d: 1.2,
            e: -3.0,
            f: 5.0,
        };
        let sr = s.compose(&r);
        for &p in &[[1.0, 2.0], [10.0, -4.0], [0.0, 7.0]] {
            let a = sr.apply(p);
            let b = s.apply(r.apply(p));
            assert!((a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3);
        }
    }

    #[test]
    fn identical_frames_give_the_identity_affine() {
        // The whole point of the byte-identical guarantee: F == F0 ⇒ M = I.
        let f = frame(50.0, 40.0, 30.0, 20.0);
        let m = affine_from_frames(&f, &f).expect("non-degenerate");
        for &p in &[[0.0, 0.0], [50.0, 40.0], [123.0, -9.0]] {
            let q = m.apply(p);
            assert!((q[0] - p[0]).abs() < 1e-3 && (q[1] - p[1]).abs() < 1e-3);
        }
    }

    #[test]
    fn frame_translation_maps_pristine_onto_current() {
        // Move the frame by (+10, −5): M must translate every point by the same.
        let f0 = frame(50.0, 40.0, 30.0, 20.0);
        let f1 = TransformFrame {
            center: [60.0, 35.0],
            ..f0
        };
        let m = affine_from_frames(&f0, &f1).unwrap();
        let q = m.apply([50.0, 40.0]);
        assert!((q[0] - 60.0).abs() < 1e-3 && (q[1] - 35.0).abs() < 1e-3);
    }

    #[test]
    fn frame_scale_doubles_extent_about_center() {
        // Double hx about center (50,40): a point on the right edge (80,40) maps to (110,40).
        let f0 = frame(50.0, 40.0, 30.0, 20.0);
        let f1 = frame(50.0, 40.0, 60.0, 20.0);
        let m = affine_from_frames(&f0, &f1).unwrap();
        assert!(
            (m.apply([50.0, 40.0])[0] - 50.0).abs() < 1e-3,
            "center fixed"
        );
        let q = m.apply([80.0, 40.0]);
        assert!((q[0] - 110.0).abs() < 1e-3 && (q[1] - 40.0).abs() < 1e-3);
    }

    #[test]
    fn uniform_corner_drag_preserves_aspect() {
        // Drag corner 1 (TR) uniformly to 2× its distance: both half-extents double (aspect kept).
        let f0 = frame(0.0, 0.0, 10.0, 4.0);
        let tr = f0.handles()[1]; // (10, 4)
        let far = [tr[0] * 2.0, tr[1] * 2.0];
        let f1 = drag_frame(&f0, 1, tr, far, true);
        let sx = f1.hx / f0.hx;
        let sy = f1.hy / f0.hy;
        assert!((sx - sy).abs() < 1e-3, "aspect preserved: sx={sx} sy={sy}");
        assert!((sx - 2.0).abs() < 0.05, "≈2×: {sx}");
    }

    #[test]
    fn free_edge_drag_scales_one_axis_only() {
        // Free mode, drag the R edge (handle 4) out along +x: hx grows, hy unchanged.
        let f0 = frame(0.0, 0.0, 10.0, 4.0);
        let f1 = drag_frame(&f0, 4, [10.0, 0.0], [20.0, 0.0], false);
        assert!((f1.hx - 20.0).abs() < 1e-3, "hx follows pointer");
        assert!((f1.hy - 4.0).abs() < 1e-3, "hy untouched");
    }

    #[test]
    fn rotate_handle_spins_the_axis() {
        // Grab the R handle and swing it 90° CCW about center → u rotates to ~(0,1).
        let f0 = frame(0.0, 0.0, 10.0, 10.0);
        let f1 = drag_frame(&f0, H_ROTATE, [10.0, 0.0], [0.0, 10.0], false);
        assert!(
            f1.u[0].abs() < 1e-2 && (f1.u[1] - 1.0).abs() < 1e-2,
            "u≈(0,1): {:?}",
            f1.u
        );
    }

    #[test]
    fn mat3_from_affine_matches_affine_apply() {
        let a = Affine2 {
            a: 1.3,
            b: 0.2,
            c: -0.5,
            d: 0.8,
            e: 7.0,
            f: -4.0,
        };
        let m = Mat3::from_affine(a);
        for &p in &[[0.0, 0.0], [10.0, -3.0], [5.0, 12.0]] {
            let x = m.apply(p);
            let y = a.apply(p);
            assert!((x[0] - y[0]).abs() < 1e-3 && (x[1] - y[1]).abs() < 1e-3);
        }
    }

    #[test]
    fn mat3_inverse_round_trips() {
        // A genuine perspective matrix (non-zero bottom row).
        let m = Mat3([1.2, 0.1, 3.0, -0.2, 0.9, -5.0, 0.001, 0.002, 1.0]);
        let inv = m.inverse().expect("non-singular");
        for &p in &[[4.0, 9.0], [-6.0, 2.0], [20.0, 20.0]] {
            let round = inv.apply(m.apply(p));
            assert!(
                (round[0] - p[0]).abs() < 1e-2 && (round[1] - p[1]).abs() < 1e-2,
                "round={round:?} p={p:?}"
            );
        }
    }

    #[test]
    fn homography_maps_src_quad_corners_onto_dst_quad() {
        // A square [0,10]² distorted into a trapezoid: each source corner must land on its dst corner.
        let src = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let dst = [[2.0, 1.0], [9.0, 0.0], [12.0, 11.0], [-1.0, 9.0]];
        let m = homography_from_quads(&src, &dst).expect("non-degenerate");
        for i in 0..4 {
            let q = m.apply(src[i]);
            assert!(
                (q[0] - dst[i][0]).abs() < 1e-2 && (q[1] - dst[i][1]).abs() < 1e-2,
                "corner {i}: got {q:?} want {:?}",
                dst[i]
            );
        }
    }

    #[test]
    fn homography_of_identical_quads_is_identity() {
        let q = [[3.0, 4.0], [13.0, 4.0], [13.0, 14.0], [3.0, 14.0]];
        let m = homography_from_quads(&q, &q).expect("non-degenerate");
        for &p in &[[3.0, 4.0], [8.0, 9.0], [13.0, 14.0]] {
            let r = m.apply(p);
            assert!(
                (r[0] - p[0]).abs() < 1e-2 && (r[1] - p[1]).abs() < 1e-2,
                "r={r:?} p={p:?}"
            );
        }
    }
}
