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

    /// Map a point.
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
        assert!((m.apply([50.0, 40.0])[0] - 50.0).abs() < 1e-3, "center fixed");
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
        assert!(f1.u[0].abs() < 1e-2 && (f1.u[1] - 1.0).abs() < 1e-2, "u≈(0,1): {:?}", f1.u);
    }
}
