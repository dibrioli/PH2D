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

use super::super::{DEFORM_TEMPERAMENT_RESHAPE, DEFORM_TEMPERAMENT_TRANSFORM};
use super::transform_geom::{
    Affine2, Mat3, ROTATE_BAND, TransformFrame, affine_from_frames, drag_frame, hit_frame,
    homography_from_quads,
};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

pub(crate) use super::transform_float::FloatingPatch;
pub(crate) use super::transform_mesh::Mesh;

/// Transform sub-modes: `0` Uniform / `1` Free (affine box) · `2` Distort (free-corner homography) ·
/// `3` Warp (mesh of control points).
const MODE_DISTORT: u8 = 2;
const MODE_WARP: u8 = 3;
/// The Warp mesh lattice size (cells): a 3×3 grid → 4×4 = 16 control points.
const MESH_COLS: u32 = 3;
const MESH_ROWS: u32 = 3;

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
    /// A Warp-mesh control-point drag: `handle` is the point index in `Mesh::current`.
    pub is_mesh: bool,
}

/// The Transform session frame: the pristine box (`M = I` reference) + the current affine box, plus the free
/// quad `corners` when in Distort (canonical geometry in that mode; `None` for the affine sub-modes).
#[derive(Copy, Clone, Debug)]
pub(crate) struct Xform {
    pub pristine: TransformFrame,
    pub current: TransformFrame,
    pub corners: Option<[[f32; 2]; 4]>,
}

/// A gizmo pose (one gesture's worth of geometry) for the Transform's LOCAL undo stack — pixels aren't
/// stored (they re-composite from the pose), so it's cheap.
#[derive(Clone, Debug)]
pub(crate) struct PoseSnap {
    pub current: TransformFrame,
    pub corners: Option<[[f32; 2]; 4]>,
    pub mesh: Option<Mesh>,
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
    /// Warp mode: `(fine_cols, fine_rows, fine-points)` — the shell draws the SMOOTH (curved) grid lines
    /// through these Catmull-Rom-subdivided points.
    pub mesh: Option<(u32, u32, Vec<[f32; 2]>)>,
    /// Warp mode: the COARSE control points — the only draggable handles (drawn as squares).
    pub mesh_handles: Option<Vec<[f32; 2]>>,
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
    pub fn set_deform_temperament(&mut self, t: u8) {
        let t = t.min(DEFORM_TEMPERAMENT_TRANSFORM);
        let old = self.paint.deform.temperament;
        if t == old {
            return;
        }
        // Leaving Transform bakes + commits the outgoing transform (one undo entry).
        if old == DEFORM_TEMPERAMENT_TRANSFORM {
            self.end_transform(true);
        }
        self.paint.deform.temperament = t;
        if t == DEFORM_TEMPERAMENT_TRANSFORM {
            // Reshape and Transform don't share a session — drop any Reshape disp so nothing re-warps, then
            // LIFT the floating patch from the current selection (re-lifts fresh every time it's picked, so a
            // new selection always gets a new gizmo).
            self.end_deform_session();
            self.begin_transform();
        }
    }
    /// Back-compat toggle (Reshape ↔ Transform), used by tests + the legacy call sites.
    pub fn set_deform_transform_on(&mut self, on: bool) {
        self.set_deform_temperament(if on {
            DEFORM_TEMPERAMENT_TRANSFORM
        } else {
            DEFORM_TEMPERAMENT_RESHAPE
        });
    }

    /// Set the Transform sub-mode: `0` Uniform · `1` Free (affine box) · `2` Distort (free-corner
    /// perspective) · `3` Warp (mesh). Entering Distort/Warp seeds its geometry from the current affine box
    /// (continuous, byte-identical); leaving Distort/Warp resets to identity (a general quad/mesh can't map
    /// back to an affine box). Uniform↔Free is a pure relabel. Bake with Apply first to keep a warp.
    pub fn set_deform_transform_mode(&mut self, m: u8) {
        let m = m.min(MODE_WARP);
        let old = self.paint.deform.transform_mode;
        self.paint.deform.transform_mode = m;
        if m == old {
            return;
        }
        let Some(x) = self.paint.deform.xform else {
            return;
        };
        // Leaving Distort / Warp → reset the affine box to identity (drop the free geometry).
        if (old == MODE_DISTORT || old == MODE_WARP) && m != old {
            self.paint.deform.xform = Some(Xform {
                current: x.pristine,
                corners: None,
                ..x
            });
            self.paint.deform.xform_mesh = None;
            self.paint.deform.xform_grab = None;
            self.composite_transform(Mat3::from_affine(Affine2::IDENTITY));
        }
        // Enter the new free mode, seeding from the (now-current) affine box.
        let cur = self
            .paint
            .deform
            .xform
            .map(|x| x.current)
            .unwrap_or(x.current);
        if m == MODE_DISTORT {
            if let Some(xx) = self.paint.deform.xform {
                self.paint.deform.xform = Some(Xform {
                    corners: Some(box4(&cur)),
                    ..xx
                });
            }
        } else if m == MODE_WARP {
            self.paint.deform.xform_mesh = Some(Mesh::seed(box4(&cur), MESH_COLS, MESH_ROWS));
        }
    }

    /// Build the gizmo view for the shell overlay — `Some` only in Deform/Transform with a live floating
    /// patch + frame. Empty otherwise (Reshape draws no box).
    #[must_use]
    pub fn deform_gizmo(&self) -> Option<DeformGizmoView> {
        if !self.is_deform_mode()
            || self.paint.deform.temperament != DEFORM_TEMPERAMENT_TRANSFORM
            || self.paint.deform.xform_patch.is_none()
        {
            return None;
        }
        let x = self.paint.deform.xform?;
        let tol = self.paint.shape_grab_tol_px;
        if self.paint.deform.transform_mode == MODE_WARP {
            let mesh = self.paint.deform.xform_mesh.as_ref()?;
            let (fcols, frows, fine) = mesh.fine_current();
            let h = x.current.handles();
            return Some(DeformGizmoView {
                box_corners: [h[0], h[1], h[2], h[3]],
                scale_handles: h,
                center: x.current.center,
                scale_tol: tol,
                rotate_tol: tol,
                corner_only: false,
                mesh: Some((fcols, frows, fine)), // smooth curved grid lines
                mesh_handles: Some(mesh.current.clone()), // draggable control points
            });
        }
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
                mesh: None,
                mesh_handles: None,
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
            mesh: None,
            mesh_handles: None,
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
        // Hit-test the active sub-mode's handles. Warp / Distort grab ONLY a control point/corner within
        // `tol` — a click in open mesh area grabs nothing (it doesn't warp from anywhere).
        let grab = if self.paint.deform.transform_mode == MODE_WARP {
            self.paint
                .deform
                .xform_mesh
                .as_ref()
                .and_then(|mesh| mesh.nearest_point(pos, tol))
                .map(|i| TransformGrab {
                    handle: i as u8,
                    start: pos,
                    initial: x.current,
                    initial_corners: None,
                    is_mesh: true,
                })
        } else if self.paint.deform.transform_mode == MODE_DISTORT {
            let q = x.corners.unwrap_or_else(|| box4(&x.current));
            nearest_corner(&q, pos, tol).map(|i| TransformGrab {
                handle: i as u8,
                start: pos,
                initial: x.current,
                initial_corners: Some(q),
                is_mesh: false,
            })
        } else {
            hit_frame(&x.current, pos, tol).map(|handle| TransformGrab {
                handle,
                start: pos,
                initial: x.current,
                initial_corners: None,
                is_mesh: false,
            })
        };
        // Only a real grab pushes a pose onto the LOCAL undo stack (so a click in empty space leaves no junk).
        if let Some(g) = grab {
            if let Some(pose) = self.current_pose() {
                self.paint.deform.xform_undo.push(pose);
            }
            self.paint.deform.xform_moved = false;
            self.paint.deform.xform_grab = Some(g);
        }
        true // consume the canvas whether or not a handle was grabbed (Transform owns it)
    }

    fn deform_gizmo_move(&mut self, pos: [f32; 2]) -> bool {
        let Some(grab) = self.paint.deform.xform_grab else {
            return false;
        };
        let Some(x) = self.paint.deform.xform else {
            return false;
        };
        // Update the geometry for the grabbed handle, then re-composite.
        if grab.is_mesh {
            // Warp: move the grabbed control point directly (absolute → drift-free).
            if let Some(mesh) = self.paint.deform.xform_mesh.as_mut()
                && let Some(p) = mesh.current.get_mut(grab.handle as usize)
            {
                *p = pos;
            }
        } else if let Some(init_q) = grab.initial_corners {
            // Distort: move the grabbed corner freely.
            let mut q = init_q;
            q[grab.handle as usize] = pos;
            self.paint.deform.xform = Some(Xform {
                corners: Some(q),
                ..x
            });
        } else {
            // Affine (Uniform / Free): rebuild the frame drift-free.
            let uniform = self.paint.deform.transform_mode == 0;
            let new_current = drag_frame(&grab.initial, grab.handle, grab.start, pos, uniform);
            self.paint.deform.xform = Some(Xform {
                current: new_current,
                ..x
            });
        }
        self.recomposite_transform();
        self.paint.deform.xform_moved = true;
        true
    }

    fn deform_gizmo_up(&mut self) -> bool {
        let had = self.paint.deform.xform_grab.take().is_some();
        // A no-op gesture (down→up without a move) leaves nothing to undo — drop the pose pushed on Down.
        if had && !self.paint.deform.xform_moved {
            self.paint.deform.xform_undo.pop();
        }
        self.paint.deform.xform_moved = false;
        had
    }

    /// The current gizmo pose (for the local undo stack).
    fn current_pose(&self) -> Option<PoseSnap> {
        let x = self.paint.deform.xform?;
        Some(PoseSnap {
            current: x.current,
            corners: x.corners,
            mesh: self.paint.deform.xform_mesh.clone(),
        })
    }

    /// Re-composite the floating patch from the CURRENT pose, picking affine / homography / mesh by sub-mode.
    /// Shared by live dragging and the local undo step.
    pub(crate) fn recomposite_transform(&mut self) {
        let Some(x) = self.paint.deform.xform else {
            return;
        };
        if self.paint.deform.transform_mode == MODE_WARP {
            self.composite_mesh();
        } else if self.paint.deform.transform_mode == MODE_DISTORT {
            let q = x.corners.unwrap_or_else(|| box4(&x.current));
            if let Some(m) = homography_from_quads(&box4(&x.pristine), &q) {
                self.composite_transform(m);
            }
        } else if let Some(a) = affine_from_frames(&x.pristine, &x.current) {
            self.composite_transform(Mat3::from_affine(a));
        }
    }

    /// Step ONE gizmo pose back on the Transform's LOCAL undo stack (a live-transform undo), or — when the
    /// stack is empty — undo the lift itself (restore the pre-lift model: the selection marquee returns and
    /// the float is dropped). Returns `true` iff it handled the undo (so the caller skips the structural
    /// undo). No-op when no Transform is live.
    pub(crate) fn transform_undo_step(&mut self) -> bool {
        if self.paint.deform.xform_patch.is_none() {
            return false;
        }
        if let Some(pose) = self.paint.deform.xform_undo.pop() {
            if let Some(x) = self.paint.deform.xform {
                self.paint.deform.xform = Some(Xform {
                    current: pose.current,
                    corners: pose.corners,
                    ..x
                });
            }
            self.paint.deform.xform_mesh = pose.mesh;
            self.paint.deform.xform_grab = None;
            self.paint.deform.xform_moved = false;
            self.recomposite_transform();
            return true;
        }
        // Empty stack → the gizmo is at its lift pose; the next undo un-lifts (restore the pre-lift model).
        if let Some(before) = self.paint.deform.xform_before.take() {
            self.clear_transform_state();
            self.restore_model(before);
            return true;
        }
        false
    }

    /// `true` while a Transform float is lifted (a live transform is in progress).
    pub(crate) fn deform_transform_live(&self) -> bool {
        self.paint.deform.xform_patch.is_some()
    }

    /// Drop all live Transform state (float + poses) WITHOUT committing — used when the lift is undone.
    pub(crate) fn clear_transform_state(&mut self) {
        self.paint.deform.xform_patch = None;
        self.paint.deform.xform = None;
        self.paint.deform.xform_grab = None;
        self.paint.deform.xform_mesh = None;
        self.paint.deform.xform_before = None;
        self.paint.deform.xform_last_bbox = None;
        self.paint.deform.xform_undo.clear();
        self.paint.deform.xform_moved = false;
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
