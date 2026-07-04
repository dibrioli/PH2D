//! **Transform** temperament (Deform Wave 2) — the gizmo-driven half of Deform. Where Reshape ([`super::field`])
//! pushes pixels with a brush, Transform warps a whole region by a matrix `M` set from bounding-box handles:
//! Uniform / Free (affine) here, Distort (homography) + Warp (mesh) in later steps. It feeds the SAME
//! inverse-warp sink as Reshape — the session `disp` map — by writing `D(p) = p − M⁻¹·p`, so `apply.rs`'s
//! single-resample render stays intact. The affine is built transcendental-free (rotation from a rotor
//! `[cos, sin]`, never a runtime angle) so HR-5 holds like the rest of the module.

use super::apply::bilinear_clamped;
use super::super::Region;
use crate::tool::PainterTool;
use std::sync::Arc;

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
    #[allow(dead_code)] // used by the Transform gizmo (Deform Wave 2, next step) + tests
    pub(super) const IDENTITY: Affine2 = Affine2 {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

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

    /// Translate → rotate (by rotor `[cos, sin]`) → scale about the pivot `c`, all transcendental-free
    /// (the caller derives the rotor from drag vectors via dot/cross, never a runtime `sin`/`cos`). Builds
    /// the affine that a Free-transform gizmo produces: `M(p) = R·S·(p − c) + c + t`.
    #[allow(dead_code)] // wired by the Transform gizmo (Deform Wave 2, next step)
    pub(super) fn trs_about(pivot: [f32; 2], rotor: [f32; 2], scale: [f32; 2], t: [f32; 2]) -> Affine2 {
        let (co, si) = (rotor[0], rotor[1]);
        let (sx, sy) = (scale[0], scale[1]);
        // Linear part L = R · S.
        let (a, b, c, d) = (co * sx, si * sx, -si * sy, co * sy);
        // Translation so the pivot maps to pivot + t: e/f = pivot + t − L·pivot.
        Affine2 {
            a,
            b,
            c,
            d,
            e: pivot[0] + t[0] - (a * pivot[0] + c * pivot[1]),
            f: pivot[1] + t[1] - (b * pivot[0] + d * pivot[1]),
        }
    }
}

impl PainterTool {
    /// Set the session displacement to the affine warp `D(p) = p − M⁻¹·p` over the whole canvas, then render
    /// from `pre`. Unlike Reshape (which ACCUMULATES dabs), a Transform is ABSOLUTE — the gizmo's current
    /// matrix defines the whole displacement, so this REPLACES `disp` each gizmo update. Identity `M` ⇒
    /// `disp = 0` ⇒ byte-identical. No-op before a session exists or when `M` is singular. Freeze holds the
    /// protected texels at their pristine spot (displacement forced to zero there).
    #[allow(dead_code)] // wired by the Transform gizmo (Deform Wave 2, next step)
    pub(super) fn apply_affine_transform(&mut self, m: Affine2) {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if self.paint.deform.pre.len() != n * 4 || self.paint.deform.disp.len() != n {
            return;
        }
        let Some(minv) = m.inverse() else {
            return;
        };
        let freeze = self.deform_freeze_effective();
        let inverted = self.deform_freeze_inverted();
        // Coverage snapshot for Freeze (immutable borrow) before the mutable passes.
        let protect: Vec<f32> = if freeze {
            let mut v = vec![0.0f32; n];
            for (i, slot) in v.iter_mut().enumerate() {
                let (x, y) = ((i as u32 % w), (i as u32 / w));
                let mut keep = f32::from(self.selection_coverage_at(x, y)) / 255.0;
                if inverted {
                    keep = 1.0 - keep;
                }
                *slot = keep;
            }
            v
        } else {
            Vec::new()
        };
        let src = Arc::clone(&self.paint.deform.pre);
        let disp = Arc::make_mut(&mut self.paint.deform.disp);
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        for i in 0..n {
            let (x, y) = ((i as u32 % w) as f32, (i as u32 / w) as f32);
            let src_pt = minv.apply([x, y]);
            let mut d = [x - src_pt[0], y - src_pt[1]];
            if freeze {
                let allow = 1.0 - protect[i];
                d[0] *= allow;
                d[1] *= allow;
            }
            disp[i] = d;
            let px = bilinear_clamped(&src, w, h, x - d[0], y - d[1]);
            let b = i * 4;
            buf[b..b + 4].copy_from_slice(&px);
        }
        self.mark_dirty(Region { x: 0, y: 0, w, h });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_inverse_and_apply() {
        let m = Affine2::IDENTITY;
        assert_eq!(m.apply([3.0, 7.0]), [3.0, 7.0]);
        assert_eq!(m.inverse(), Some(Affine2::IDENTITY));
    }

    #[test]
    fn inverse_round_trips() {
        // A non-trivial affine: scale (2, 0.5), a shear, and a translation.
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
    fn trs_uniform_scale_about_pivot_keeps_the_pivot_fixed() {
        // Scale ×2 about (50, 50) with no rotation/translation: the pivot maps to itself, a point 10 to the
        // right maps to 20 to the right.
        let m = Affine2::trs_about([50.0, 50.0], [1.0, 0.0], [2.0, 2.0], [0.0, 0.0]);
        assert_eq!(m.apply([50.0, 50.0]), [50.0, 50.0]);
        assert_eq!(m.apply([60.0, 50.0]), [70.0, 50.0]);
    }

    #[test]
    fn trs_quarter_turn_rotor_rotates_about_pivot() {
        // 90° rotor (cos=0, sin=1) about (0,0): (1,0) → (0,1).
        let m = Affine2::trs_about([0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [0.0, 0.0]);
        let r = m.apply([1.0, 0.0]);
        assert!((r[0]).abs() < 1e-4 && (r[1] - 1.0).abs() < 1e-4);
    }
}
