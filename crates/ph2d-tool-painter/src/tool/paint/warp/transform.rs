//! **Transform** temperament (Deform Wave 2) — the gizmo-driven half of Deform. Where Reshape ([`super::field`])
//! pushes pixels with a brush, Transform warps a whole region by a matrix `M` set from bounding-box handles:
//! Uniform / Free (affine) here, Distort (homography) + Warp (mesh) in later steps. It feeds the SAME
//! inverse-warp sink as Reshape — the session `disp` map — by writing `D(p) = p − M⁻¹·p`, so `apply.rs`'s
//! single-resample render stays intact. The affine is built transcendental-free (rotation from a rotor
//! `[cos, sin]` derived from the drag via dot/cross, never a runtime angle) so HR-5 holds like the rest of
//! the module.
//!
//! **Model.** A session holds a PRISTINE frame `F0` (the content's oriented bbox at Transform start) and a
//! CURRENT frame `F` (`F0` at first, then dragged). The warp is the affine that maps `F0`'s box onto `F`'s
//! box (`affine_from_frames`) — so `F == F0` ⇒ `M = I` ⇒ `disp = 0` ⇒ **byte-identical**. Each gizmo drag
//! rebuilds `F` from the PRISTINE-at-grab frame (drift-free, like the selection gizmo) and re-applies the
//! whole affine from `pre` (absolute, not accumulated → no compound blur). The current frame is captured in
//! the undo snapshot next to `disp`, so undo rolls the gizmo box back in lock-step with the pixels.

use super::super::Region;
use super::apply::bilinear_clamped;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use std::sync::Arc;

/// Smallest half-extent a scale drag allows (image px) — mirrors the selection gizmo.
const MIN_AXIS_PX: f32 = 1.0;
/// The rotate ring reaches this many grab-radii beyond a scale square (mirrors the selection gizmo).
const ROTATE_BAND: f32 = 2.6;
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
    fn handles(&self) -> [[f32; 2]; 8] {
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
    /// Serialize as `[cx, cy, ux, uy, hx, hy]` for the undo snapshot.
    pub(crate) fn to_array(self) -> [f32; 6] {
        [self.center[0], self.center[1], self.u[0], self.u[1], self.hx, self.hy]
    }
    /// Deserialize from the undo snapshot.
    pub(crate) fn from_array(a: [f32; 6]) -> TransformFrame {
        TransformFrame {
            center: [a[0], a[1]],
            u: unit_or([a[2], a[3]], [1.0, 0.0]),
            hx: a[4].max(MIN_AXIS_PX),
            hy: a[5].max(MIN_AXIS_PX),
        }
    }
}

/// The affine that maps frame `f0`'s box onto frame `f1`'s box: `M = A1 ∘ A0⁻¹` (unit→world of `f1` after
/// world→unit of `f0`). Handles anisotropic scale + rotation + translation at once, transcendental-free.
/// Returns `None` only if `f0` is degenerate (guarded elsewhere: half-extents are clamped `> 0`).
pub(super) fn affine_from_frames(f0: &TransformFrame, f1: &TransformFrame) -> Option<Affine2> {
    let a0inv = f0.basis().inverse()?;
    Some(f1.basis().compose(&a0inv))
}

/// The active Transform gizmo grab — carries the PRISTINE current-frame at grab + the pointer position, so
/// every drag is computed drift-free from the untouched frame (mirrors `SelectionGrab`).
#[derive(Copy, Clone, Debug)]
pub(crate) struct TransformGrab {
    pub handle: u8,
    pub start: [f32; 2],
    pub initial: TransformFrame,
}

/// The Transform session: the pristine content frame (`M = I` reference) + the current (dragged) frame.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Xform {
    pub pristine: TransformFrame,
    pub current: TransformFrame,
}

/// A drawable Transform gizmo (image-space px) for the shell overlay — an oriented box with 8 scale squares
/// (corners + edge mids, a square reads as a circle in its rotate ring) + a centre-move square.
pub struct DeformGizmoView {
    pub box_corners: [[f32; 2]; 4],
    pub scale_handles: [[f32; 2]; 8],
    pub center: [f32; 2],
    pub scale_tol: f32,
    pub rotate_tol: f32,
}

impl PainterTool {
    // ── Temperament + sub-mode setters (single clamp source; routed from `route_deform_event`) ──

    /// Switch the Deform temperament: `false` = Reshape (brush), `true` = Transform (gizmo). Turning
    /// Transform on starts a session (captures `pre`) and initializes the gizmo frame to the content bbox so
    /// the box is visible immediately.
    pub fn set_deform_transform_on(&mut self, on: bool) {
        self.paint.deform.transform_on = on;
        self.paint.deform.xform_grab = None;
        if on {
            self.ensure_deform_session();
            self.ensure_xform();
        }
    }
    /// Set the Transform sub-mode: `0` Uniform (aspect-locked corners), `1` Free (independent axes).
    pub fn set_deform_transform_mode(&mut self, m: u8) {
        self.paint.deform.transform_mode = m.min(1);
    }

    /// Ensure the gizmo frame exists, initializing `pristine == current` to the session content's oriented
    /// bbox (axis-aligned) — or the whole canvas when nothing is opaque. No-op once initialized.
    pub(super) fn ensure_xform(&mut self) {
        if self.paint.deform.xform.is_some() {
            return;
        }
        let f = self.deform_content_frame();
        self.paint.deform.xform = Some(Xform {
            pristine: f,
            current: f,
        });
    }

    /// The axis-aligned bounding frame of the session's opaque content (from `pre`), or the whole canvas
    /// when nothing is opaque / no session. Half-extents are clamped `> 0` so the basis is invertible.
    fn deform_content_frame(&self) -> TransformFrame {
        let (w, h) = self.source_size;
        let full = TransformFrame {
            center: [w as f32 * 0.5, h as f32 * 0.5],
            u: [1.0, 0.0],
            hx: (w as f32 * 0.5).max(MIN_AXIS_PX),
            hy: (h as f32 * 0.5).max(MIN_AXIS_PX),
        };
        let n = (w as usize) * (h as usize);
        if self.paint.deform.pre.len() != n * 4 || n == 0 {
            return full;
        }
        let px = &self.paint.deform.pre;
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        let mut any = false;
        for y in 0..h {
            for x in 0..w {
                if px[((y * w + x) as usize) * 4 + 3] > 0 {
                    any = true;
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        if !any {
            return full;
        }
        // Inclusive bbox → +1 to span the far pixel edge.
        let (fx0, fy0, fx1, fy1) = (x0 as f32, y0 as f32, x1 as f32 + 1.0, y1 as f32 + 1.0);
        TransformFrame {
            center: [(fx0 + fx1) * 0.5, (fy0 + fy1) * 0.5],
            u: [1.0, 0.0],
            hx: ((fx1 - fx0) * 0.5).max(MIN_AXIS_PX),
            hy: ((fy1 - fy0) * 0.5).max(MIN_AXIS_PX),
        }
    }

    /// Build the gizmo view for the shell overlay — `Some` only in Deform/Transform with a live session +
    /// initialized frame. Empty otherwise (Reshape draws no box).
    #[must_use]
    pub fn deform_gizmo(&self) -> Option<DeformGizmoView> {
        if !self.is_deform_mode() || !self.paint.deform.transform_on || !self.paint.deform.active {
            return None;
        }
        let f = self.paint.deform.xform?.current;
        let h = f.handles();
        let tol = self.paint.shape_grab_tol_px;
        Some(DeformGizmoView {
            box_corners: [h[0], h[1], h[2], h[3]],
            scale_handles: h,
            center: f.center,
            scale_tol: tol,
            rotate_tol: tol * ROTATE_BAND,
        })
    }

    // ── Gizmo pointer lifecycle (routed from `canvas_pointer.rs` when Transform is active) ──

    /// Route a canvas pointer to the Transform gizmo. Down grabs a handle (+ opens one structural undo
    /// entry), Move drags it (rebuilding the affine live), Up commits. Returns `true` iff handled.
    pub(crate) fn deform_gizmo_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.deform_gizmo_down(ev.pos),
            PointerPhase::Move => self.deform_gizmo_move(ev.pos),
            PointerPhase::Up => self.deform_gizmo_up(),
            PointerPhase::Hover => false,
        }
    }

    fn deform_gizmo_down(&mut self, pos: [f32; 2]) -> bool {
        self.ensure_deform_session();
        self.ensure_xform();
        let Some(x) = self.paint.deform.xform else {
            return false;
        };
        let tol = self.paint.shape_grab_tol_px;
        let Some(handle) = hit_frame(&x.current, pos, tol) else {
            // A Down away from every handle doesn't grab — but still consumes (Transform owns the canvas).
            return true;
        };
        let before = self.snapshot_model();
        self.paint.stroke_undo = Some(before);
        self.paint.deform.xform_grab = Some(TransformGrab {
            handle,
            start: pos,
            initial: x.current,
        });
        true
    }

    fn deform_gizmo_move(&mut self, pos: [f32; 2]) -> bool {
        let Some(grab) = self.paint.deform.xform_grab else {
            return false;
        };
        let uniform = self.paint.deform.transform_mode == 0;
        let new_current = drag_frame(&grab.initial, grab.handle, grab.start, pos, uniform);
        let pristine = match self.paint.deform.xform {
            Some(x) => x.pristine,
            None => return false,
        };
        self.paint.deform.xform = Some(Xform {
            pristine,
            current: new_current,
        });
        if let Some(m) = affine_from_frames(&pristine, &new_current) {
            self.apply_affine_transform(m);
        }
        true
    }

    fn deform_gizmo_up(&mut self) -> bool {
        let had = self.paint.deform.xform_grab.take().is_some();
        if had && let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
        had
    }

    // ── Undo glue (the gizmo frame rides the snapshot next to `disp`) ──

    /// The current gizmo frame for the undo snapshot (serialized), or `None` when no Transform frame exists.
    pub(crate) fn deform_xform_for_snapshot(&self) -> Option<[f32; 6]> {
        self.paint.deform.xform.map(|x| x.current.to_array())
    }

    /// Reinstate the gizmo frame from an undo snapshot. The pristine frame is re-derived from `pre` (stable
    /// within a session); `current` is restored so the box + pixels roll back together.
    pub(crate) fn restore_deform_xform(&mut self, current: Option<[f32; 6]>) {
        self.paint.deform.xform_grab = None;
        self.paint.deform.xform = current.map(|a| Xform {
            pristine: self.deform_content_frame(),
            current: TransformFrame::from_array(a),
        });
    }

    /// Set the session displacement to the affine warp `D(p) = p − M⁻¹·p` over the whole canvas, then render
    /// from `pre`. Unlike Reshape (which ACCUMULATES dabs), a Transform is ABSOLUTE — the gizmo's current
    /// matrix defines the whole displacement, so this REPLACES `disp` each gizmo update. Identity `M` ⇒
    /// `disp = 0` ⇒ byte-identical. No-op before a session exists or when `M` is singular. Freeze holds the
    /// protected texels at their pristine spot (displacement forced to zero there).
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

/// Hit-test the gizmo's handles at `pos` within `tol`; returns the grabbed handle id (scale square, rotate
/// ring, or centre-move) — mirrors the selection gizmo's `hit_shape`.
fn hit_frame(f: &TransformFrame, pos: [f32; 2], tol: f32) -> Option<u8> {
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
fn drag_frame(
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

    #[test]
    fn frame_round_trips_through_array() {
        let f = TransformFrame {
            center: [12.0, -3.0],
            u: unit_or([1.0, 1.0], [1.0, 0.0]),
            hx: 7.0,
            hy: 9.0,
        };
        let g = TransformFrame::from_array(f.to_array());
        assert!((g.center[0] - f.center[0]).abs() < 1e-4);
        assert!((g.u[0] - f.u[0]).abs() < 1e-4 && (g.u[1] - f.u[1]).abs() < 1e-4);
        assert!((g.hx - f.hx).abs() < 1e-4 && (g.hy - f.hy).abs() < 1e-4);
    }
}
