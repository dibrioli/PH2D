//! **Transform** temperament (Deform Wave 2) — the gizmo-driven half of Deform. Where Reshape ([`super::field`])
//! pushes pixels with a brush, Transform LIFTS the selected pixels into a floating patch and moves/scales/
//! rotates that patch freely over the layer (Procreate model — see [`super::transform_float`] for the lift +
//! composite). This module holds the tool-side wiring: temperament state, the gizmo frame + pointer, and the
//! affine built from the drag. The affine primitive + gizmo geometry live in [`super::transform_geom`] (pure
//! math, HR-5 transcendental-free).
//!
//! **Model.** Turning Transform on lifts the patch and frames the gizmo on it (`F0`). Each gizmo drag rebuilds
//! the CURRENT frame `F` from the pristine-at-grab frame (drift-free, like the selection gizmo) and
//! re-composites the patch under the affine that maps `F0`'s box onto `F`'s (`affine_from_frames`). `F == F0`
//! ⇒ `M = I` ⇒ patch sits on its origin ⇒ **byte-identical**. The whole transform is ONE undo entry,
//! committed when it ends (temperament switch / mode change / Apply).

use super::transform_geom::{
    ROTATE_BAND, TransformFrame, affine_from_frames, drag_frame, hit_frame,
};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

pub(crate) use super::transform_float::FloatingPatch;

/// The active Transform gizmo grab — carries the PRISTINE current-frame at grab + the pointer position, so
/// every drag is computed drift-free from the untouched frame (mirrors `SelectionGrab`).
#[derive(Copy, Clone, Debug)]
pub(crate) struct TransformGrab {
    pub handle: u8,
    pub start: [f32; 2],
    pub initial: TransformFrame,
}

/// The Transform session frame: the pristine frame (`M = I` reference) + the current (dragged) frame.
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
    /// Set the Transform sub-mode: `0` Uniform (aspect-locked corners), `1` Free (independent axes).
    pub fn set_deform_transform_mode(&mut self, m: u8) {
        self.paint.deform.transform_mode = m.min(1);
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

    /// Route a canvas pointer to the Transform gizmo. Down grabs a handle, Move drags it (re-compositing the
    /// patch live), Up releases (no per-drag undo — the whole transform is one entry). Returns `true` iff
    /// handled.
    pub(crate) fn deform_gizmo_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.deform_gizmo_down(ev.pos),
            PointerPhase::Move => self.deform_gizmo_move(ev.pos),
            PointerPhase::Up => self.deform_gizmo_up(),
            PointerPhase::Hover => false,
        }
    }

    fn deform_gizmo_down(&mut self, pos: [f32; 2]) -> bool {
        // Lift on demand if the temperament was set without a patch yet (e.g. a selection made afterwards).
        if self.paint.deform.xform_patch.is_none() {
            self.begin_transform();
        }
        let Some(x) = self.paint.deform.xform else {
            return false;
        };
        let tol = self.paint.shape_grab_tol_px;
        let Some(handle) = hit_frame(&x.current, pos, tol) else {
            // A Down away from every handle doesn't grab — but still consumes (Transform owns the canvas).
            return true;
        };
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
        let Some(x) = self.paint.deform.xform else {
            return false;
        };
        self.paint.deform.xform = Some(Xform {
            pristine: x.pristine,
            current: new_current,
        });
        if let Some(m) = affine_from_frames(&x.pristine, &new_current) {
            self.composite_transform(m);
        }
        true
    }

    fn deform_gizmo_up(&mut self) -> bool {
        self.paint.deform.xform_grab.take().is_some()
    }
}
