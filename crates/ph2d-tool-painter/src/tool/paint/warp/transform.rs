//! **Transform** temperament (Deform Wave 2) — the gizmo-driven half of Deform. Where Reshape ([`super::field`])
//! pushes pixels with a brush, Transform warps a whole region by a matrix `M` set from bounding-box handles:
//! Uniform / Free (affine) here, Distort (homography) + Warp (mesh) in later steps. It feeds the SAME
//! inverse-warp sink as Reshape — the session `disp` map — by writing `D(p) = p − M⁻¹·p`, so `apply.rs`'s
//! single-resample render stays intact. The affine primitive + gizmo geometry live in [`super::transform_geom`]
//! (pure math, HR-5 transcendental-free); this module is the tool-side wiring (session state, pointer,
//! kernel, undo glue).
//!
//! **Model.** A session holds a PRISTINE frame `F0` (the selection bbox, or the content bbox when nothing is
//! selected) and a CURRENT frame `F` (`F0` at first, then dragged). The warp is the affine that maps `F0`'s
//! box onto `F`'s box (`affine_from_frames`) — so `F == F0` ⇒ `M = I` ⇒ `disp = 0` ⇒ **byte-identical**.
//! Each gizmo drag rebuilds `F` from the PRISTINE-at-grab frame (drift-free, like the selection gizmo) and
//! re-applies the whole affine from `pre` (absolute, not accumulated → no compound blur). The current frame
//! is captured in the undo snapshot next to `disp`, so undo rolls the gizmo box back in lock-step with the
//! pixels.

use super::super::Region;
use super::apply::bilinear_clamped;
use super::transform_geom::{
    Affine2, ROTATE_BAND, TransformFrame, affine_from_frames, drag_frame, hit_frame,
};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use std::sync::Arc;

/// Smallest half-extent used when boxing the content/selection (image px) — keeps the basis invertible.
const MIN_AXIS_PX: f32 = 1.0;

/// The active Transform gizmo grab — carries the PRISTINE current-frame at grab + the pointer position, so
/// every drag is computed drift-free from the untouched frame (mirrors `SelectionGrab`).
#[derive(Copy, Clone, Debug)]
pub(crate) struct TransformGrab {
    pub handle: u8,
    pub start: [f32; 2],
    pub initial: TransformFrame,
}

/// The Transform session: the pristine frame (`M = I` reference) + the current (dragged) frame.
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
    /// Transform on starts a session (captures `pre`) and initializes the gizmo frame to the selection /
    /// content bbox so the box is visible immediately.
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

    /// Ensure the gizmo frame exists, initializing `pristine == current` to the target bbox. No-op once
    /// initialized.
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

    /// The axis-aligned frame the gizmo starts on: the **selection** bbox when a selection confines the warp,
    /// else the session's opaque **content** bbox (from `pre`), else the whole canvas. Half-extents are
    /// clamped `> 0` so the basis is invertible.
    fn deform_content_frame(&self) -> TransformFrame {
        let (w, h) = self.source_size;
        let full = TransformFrame {
            center: [w as f32 * 0.5, h as f32 * 0.5],
            u: [1.0, 0.0],
            hx: (w as f32 * 0.5).max(MIN_AXIS_PX),
            hy: (h as f32 * 0.5).max(MIN_AXIS_PX),
        };
        let n = (w as usize) * (h as usize);
        if n == 0 {
            return full;
        }
        // A selection confines the transform → box the selection; otherwise box the opaque content.
        let restrict = self.deform_restricts_to_selection();
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        let mut any = false;
        if restrict {
            for y in 0..h {
                for x in 0..w {
                    if self.selection_coverage_at(x, y) > 0 {
                        any = true;
                        x0 = x0.min(x);
                        y0 = y0.min(y);
                        x1 = x1.max(x);
                        y1 = y1.max(y);
                    }
                }
            }
        } else if self.paint.deform.pre.len() == n * 4 {
            let px = &self.paint.deform.pre;
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

    /// Reinstate the gizmo frame from an undo snapshot. The pristine frame is re-derived from the selection /
    /// `pre` (stable within a session); `current` is restored so the box + pixels roll back together.
    pub(crate) fn restore_deform_xform(&mut self, current: Option<[f32; 6]>) {
        self.paint.deform.xform_grab = None;
        self.paint.deform.xform = current.map(|a| Xform {
            pristine: self.deform_content_frame(),
            current: TransformFrame::from_array(a),
        });
    }

    /// Set the session displacement to the affine warp `D(p) = p − M⁻¹·p`, then render from `pre`. Unlike
    /// Reshape (which ACCUMULATES dabs), a Transform is ABSOLUTE — the gizmo's current matrix defines the
    /// whole displacement, so this REPLACES `disp` each gizmo update. Identity `M` ⇒ `disp = 0` ⇒
    /// byte-identical. No-op before a session exists or when `M` is singular. **Selection-confined:** with an
    /// active selection the warp only moves the selected texels (the rest stay pristine); with no selection
    /// it transforms the whole sprite.
    pub(super) fn apply_affine_transform(&mut self, m: Affine2) {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if self.paint.deform.pre.len() != n * 4 || self.paint.deform.disp.len() != n {
            return;
        }
        let Some(minv) = m.inverse() else {
            return;
        };
        let restrict = self.deform_restricts_to_selection();
        // Selection coverage snapshot (immutable borrow) before the mutable passes — the per-texel fraction
        // of the warp that applies (`1` inside the selection, `0` outside; feather rides between).
        let cover: Vec<f32> = if restrict {
            let mut v = vec![0.0f32; n];
            for (i, slot) in v.iter_mut().enumerate() {
                let (x, y) = ((i as u32 % w), (i as u32 / w));
                *slot = f32::from(self.selection_coverage_at(x, y)) / 255.0;
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
            if restrict {
                let allow = cover[i];
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
