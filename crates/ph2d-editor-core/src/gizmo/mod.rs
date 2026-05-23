//! M14.7 B + C — sprite transform gizmo painter + state machine.
//!
//! Visual layer for the move/rotate/scale gizmo that paints over a
//! selected sprite on the canvas. The host computes the selection's
//! world-space bbox via `ph2d_render::selection_bbox_world` and pushes
//! it here through [`GizmoView`] — this module never sees a SimWorld
//! or PresentWorld directly (HR-8 / ADR-0021 keep the editor on the
//! consumer side).
//!
//! Wave 6+7 Phase 1.B split this from a single 1770-LOC file into
//! per-domain sub-modules: drag state, camera/modifiers/snap config,
//! transform math, hit ids, and paint.
//!
//! ## Layout
//!
//! - **Bbox**: stroke 1.5 px in `Selection` color, 4 px rounded
//!   corners.
//! - **8 handles** (12×12 px filled `Accent`, 1 px `BorderEmph`
//!   stroke): 4 corners = uniform scale; 4 edge midpoints = axis-only
//!   scale.
//! - **Rotate hover**: a 12-px ring just outside each corner. Painted
//!   only as hit rects in this module — actual cursor-change comes
//!   with M14.7 C.
//! - **Pivot dot**: 6-px filled `Accent` at the bbox center.
//! - **Bbox interior**: translate region. One large hit rect spanning
//!   the bbox minus the handle hits.
//!
//! Active handle (the one the user is currently dragging) is painted
//! with `AccentHover` once the state machine in M14.7 C lands; for now
//! every handle paints the same.

pub mod camera;
pub mod drag;
pub mod hit;
pub mod paint;
pub mod transform;

#[cfg(test)]
mod tests;

pub use camera::{GizmoCamera, GizmoModifiers, GizmoSnap};
pub use drag::{GizmoDragKind, GizmoDragState, GizmoHit, GizmoTarget, TransformSnapshot};
pub use hit::{gizmo_kind_for_id, ids, is_gizmo_handle_id};
pub use paint::{GizmoView, paint_gizmo_outline, paint_sprite_gizmo, paint_sprite_gizmo_keyed};
pub use transform::{
    anchor_pivot_world, compute_gizmo_transform, move_pivot_transform, pivot_snap_candidates,
};
