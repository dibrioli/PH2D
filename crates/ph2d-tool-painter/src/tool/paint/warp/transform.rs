//! **Transform** temperament (Deform Wave 2) — the gizmo-driven half of Deform. Where Reshape ([`super::field`])
//! pushes pixels with a brush, Transform LIFTS the selected pixels into a floating patch and moves/scales/
//! rotates/distorts that patch over the layer (Procreate model — see [`super::transform_float`] for the lift +
//! composite). This module holds the tool-side wiring: temperament + sub-mode state, the gizmo frame/quad +
//! pointer, and the projective map built from the drag. The math primitives live in [`super::transform_geom`]
//! (affine + homography, transcendental-free).
//!
//! **Sub-modes.** Uniform / Free build an AFFINE from an oriented box (8 handles: scale + rotate + move).
//! Distort frees the 4 corners independently → a projective HOMOGRAPHY (perspective). All feed the same
//! [`FloatingPatch`] composite via a [`Mat3`]. `M = I` ⇒ patch on its origin ⇒ **byte-identical**. The whole
//! transform is ONE undo entry, committed when it ends.

use super::transform_geom::{
    Affine2, Mat3, ROTATE_BAND, TransformFrame, affine_from_frames, drag_frame, hit_frame,
    homography_from_quads,
};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

pub(crate) use super::transform_float::FloatingPatch;

/// Transform sub-mode: Distort (free-corner homography); `0` Uniform / `1` Free are affine.
const MODE_DISTORT: u8 = 2;

/// The active Transform gizmo grab — carries the PRISTINE state at grab + the pointer position, so every
/// drag is computed drift-free. Affine drags carry the pristine frame; Distort drags carry the corner index
/// + the pristine quad.
#[derive(Copy, Clone, Debug)]
pub(crate) struct TransformGrab {
    pub handle: u8,
    pub start: [f32; 2],
    pub initial: TransformFrame,
    /// `Some` for a Distort corner drag: the 4 corners at grab (the dragged one is `handle`, `0..4`).
    pub initial_corners: Option<[[f32; 2]; 4]>,
}

/// The Transform session frame: the pristine box (`M = I` reference) + the current affine box, plus the free
/// quad `corners` when in Distort (canonical geometry in that mode; `None` for the affine sub-modes).
#[derive(Copy, Clone, Debug)]
pub(crate) struct Xform {
    pub pristine: TransformFrame,
    pub current: TransformFrame,
    pub corners: Option<[[f32; 2]; 4]>,
}

/// A drawable Transform gizmo (image-space px) for the shell overlay. Affine sub-modes show the oriented box
/// with 8 scale squares (corners + edge mids, a square reads as a circle in its rotate ring) + a centre-move
/// square. Distort shows ONLY the 4 corner squares (each drags freely). `corner_only` selects which.
pub struct DeformGizmoView {
    pub box_corners: [[f32; 2]; 4],
    pub scale_handles: [[f32; 2]; 8],
    pub center: [f32; 2],
    pub scale_tol: f32,
    pub rotate_tol: f32,
    /// Distort mode: draw only the 4 `box_corners` as handles (no edges / rotate / centre).
    pub corner_only: bool,
}

/// The box's 4 corners `[TL, TR, BR, BL]` (the first four gizmo handles).
fn box4(f: &TransformFrame) -> [[f32; 2]; 4] {
    let h = f.handles();
    [h[0], h[1], h[2], h[3]]
}

impl PainterTool {
    // ── Temperament + sub-mode setters (single clamp source; routed from `route_deform_event`) ──

    /// Switch the Deform temperament: `false` = Reshape (brush), `true` = Transform (gizmo). Turning
    /// Transform on LIFTS the floating patch (consuming the selection marquee); turning it off bakes the
    /// patch + commits the whole transform as one undo entry.
    pub fn set_deform_transform_on(&mut self, on: bool) {
        if self.paint.deform.transform_on == on {
            return;
        }
        self.paint.deform.transform_on = on;
        if on {
            // Reshape and Transform don't share a session — drop any Reshape disp so nothing re-warps.
            self.end_deform_session();
            self.begin_transform();
        } else {
            self.end_transform(true);
        }
    }

    /// Set the Transform sub-mode: `0` Uniform (aspect-locked corners) · `1` Free (independent axes) ·
    /// `2` Distort (free-corner perspective). Entering Distort seeds the free quad from the current affine
    /// box (continuous); leaving Distort resets the transform to identity (the quad can't map back to an
    /// affine box); Uniform↔Free is a pure relabel. Bake with Apply first to keep a distortion.
    pub fn set_deform_transform_mode(&mut self, m: u8) {
        let m = m.min(MODE_DISTORT);
        let old = self.paint.deform.transform_mode;
        self.paint.deform.transform_mode = m;
        if m == old {
            return;
        }
        let Some(x) = self.paint.deform.xform else {
            return;
        };
        if m == MODE_DISTORT {
            // Seed the free quad from the current affine box → continuous.
            self.paint.deform.xform = Some(Xform {
                corners: Some(box4(&x.current)),
                ..x
            });
        } else if old == MODE_DISTORT {
            // Leaving Distort → reset to identity (no affine box represents a general quad).
            self.paint.deform.xform = Some(Xform {
                current: x.pristine,
                corners: None,
                ..x
            });
            self.paint.deform.xform_grab = None;
            self.composite_transform(Mat3::from_affine(Affine2::IDENTITY));
        }
    }

    /// Build the gizmo view for the shell overlay — `Some` only in Deform/Transform with a live floating
    /// patch + frame. Empty otherwise (Reshape draws no box).
    #[must_use]
    pub fn deform_gizmo(&self) -> Option<DeformGizmoView> {
        if !self.is_deform_mode()
            || !self.paint.deform.transform_on
            || self.paint.deform.xform_patch.is_none()
        {
            return None;
        }
        let x = self.paint.deform.xform?;
        let tol = self.paint.shape_grab_tol_px;
        if self.paint.deform.transform_mode == MODE_DISTORT {
            let q = x.corners.unwrap_or_else(|| box4(&x.current));
            let center = [
                (q[0][0] + q[1][0] + q[2][0] + q[3][0]) * 0.25,
                (q[0][1] + q[1][1] + q[2][1] + q[3][1]) * 0.25,
            ];
            return Some(DeformGizmoView {
                box_corners: q,
                scale_handles: [q[0], q[1], q[2], q[3], q[0], q[1], q[2], q[3]],
                center,
                scale_tol: tol,
                rotate_tol: tol,
                corner_only: true,
            });
        }
        let h = x.current.handles();
        Some(DeformGizmoView {
            box_corners: [h[0], h[1], h[2], h[3]],
            scale_handles: h,
            center: x.current.center,
            scale_tol: tol,
            rotate_tol: tol * ROTATE_BAND,
            corner_only: false,
        })
    }

    // ── Gizmo pointer lifecycle (routed from `canvas_pointer.rs` when Transform is active) ──

    /// Route a canvas pointer to the Transform gizmo. Down grabs a handle/corner, Move drags it (re-
    /// compositing the patch live), Up releases (no per-drag undo — the whole transform is one entry).
    pub(crate) fn deform_gizmo_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.deform_gizmo_down(ev.pos),
            PointerPhase::Move => self.deform_gizmo_move(ev.pos),
            PointerPhase::Up => self.deform_gizmo_up(),
            PointerPhase::Hover => false,
        }
    }

    fn deform_gizmo_down(&mut self, pos: [f32; 2]) -> bool {
        if self.paint.deform.xform_patch.is_none() {
            self.begin_transform();
        }
        let Some(x) = self.paint.deform.xform else {
            return false;
        };
        let tol = self.paint.shape_grab_tol_px;
        if self.paint.deform.transform_mode == MODE_DISTORT {
            let q = x.corners.unwrap_or_else(|| box4(&x.current));
            let Some(i) = nearest_corner(&q, pos, tol) else {
                return true; // Down away from a corner still consumes (Transform owns the canvas).
            };
            self.paint.deform.xform_grab = Some(TransformGrab {
                handle: i as u8,
                start: pos,
                initial: x.current,
                initial_corners: Some(q),
            });
            return true;
        }
        let Some(handle) = hit_frame(&x.current, pos, tol) else {
            return true;
        };
        self.paint.deform.xform_grab = Some(TransformGrab {
            handle,
            start: pos,
            initial: x.current,
            initial_corners: None,
        });
        true
    }

    fn deform_gizmo_move(&mut self, pos: [f32; 2]) -> bool {
        let Some(grab) = self.paint.deform.xform_grab else {
            return false;
        };
        let Some(x) = self.paint.deform.xform else {
            return false;
        };
        if let Some(init_q) = grab.initial_corners {
            // Distort: move the grabbed corner freely → homography from the pristine box corners.
            let mut q = init_q;
            q[grab.handle as usize] = pos;
            self.paint.deform.xform = Some(Xform {
                corners: Some(q),
                ..x
            });
            if let Some(m) = homography_from_quads(&box4(&x.pristine), &q) {
                self.composite_transform(m);
            }
            return true;
        }
        // Affine (Uniform / Free): rebuild the frame drift-free → affine box→box.
        let uniform = self.paint.deform.transform_mode == 0;
        let new_current = drag_frame(&grab.initial, grab.handle, grab.start, pos, uniform);
        self.paint.deform.xform = Some(Xform {
            current: new_current,
            ..x
        });
        if let Some(a) = affine_from_frames(&x.pristine, &new_current) {
            self.composite_transform(Mat3::from_affine(a));
        }
        true
    }

    fn deform_gizmo_up(&mut self) -> bool {
        self.paint.deform.xform_grab.take().is_some()
    }
}

/// The index of the quad corner within `tol` of `pos` (nearest), or `None`.
fn nearest_corner(q: &[[f32; 2]; 4], pos: [f32; 2], tol: f32) -> Option<usize> {
    let tol2 = tol * tol;
    let mut best = None;
    let mut bestd = tol2;
    for (i, c) in q.iter().enumerate() {
        let dx = c[0] - pos[0];
        let dy = c[1] - pos[1];
        let d = dx * dx + dy * dy;
        if d <= bestd {
            bestd = d;
            best = Some(i);
        }
    }
    best
}
