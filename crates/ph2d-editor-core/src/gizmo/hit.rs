//! Gizmo hit-test: id classification + handle id constants.
//!
//! Extracted from monolithic `gizmo.rs` in Wave 6+7 Phase 1.B.

use super::drag::GizmoDragKind;
use ph2d_a11y::NodeId;

/// Resolve a hit-test NodeId to the gizmo drag kind it should begin.
/// Returns `None` for ids that aren't part of the gizmo's hit set.
/// The host calls this on Mouse Down to decide whether the click
/// starts a drag or falls through to the regular widget pipeline.
pub fn gizmo_kind_for_id(id: NodeId) -> Option<GizmoDragKind> {
    use ids::*;
    let kind = if id == GIZMO_BBOX_INTERIOR {
        GizmoDragKind::Translate
    } else if id == GIZMO_HANDLE_TL {
        GizmoDragKind::ScaleCorner {
            dx_sign: -1.0,
            dy_sign: 1.0,
        }
    } else if id == GIZMO_HANDLE_TR {
        GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: 1.0,
        }
    } else if id == GIZMO_HANDLE_BL {
        GizmoDragKind::ScaleCorner {
            dx_sign: -1.0,
            dy_sign: -1.0,
        }
    } else if id == GIZMO_HANDLE_BR {
        GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: -1.0,
        }
    } else if id == GIZMO_HANDLE_L {
        GizmoDragKind::ScaleEdge {
            axis: 0,
            sign: -1.0,
        }
    } else if id == GIZMO_HANDLE_R {
        GizmoDragKind::ScaleEdge { axis: 0, sign: 1.0 }
    } else if id == GIZMO_HANDLE_T {
        GizmoDragKind::ScaleEdge { axis: 1, sign: 1.0 }
    } else if id == GIZMO_HANDLE_B {
        GizmoDragKind::ScaleEdge {
            axis: 1,
            sign: -1.0,
        }
    } else if id == GIZMO_ROTATE_TL
        || id == GIZMO_ROTATE_TR
        || id == GIZMO_ROTATE_BL
        || id == GIZMO_ROTATE_BR
    {
        GizmoDragKind::Rotate
    } else {
        return None;
    };
    Some(kind)
}

/// True iff `id` is one of the gizmo's 14 hit zones. Convenience for
/// the host's canvas-click guard: when a gizmo handle was hit, the
/// host suppresses the canvas-pick fallback (which would otherwise
/// re-select the same sprite needlessly).
///
/// Note it is the 13 zones that BEGIN A DRAG — [`ids::GIZMO_PIVOT`] is deliberately not one of them
/// (the pivot dot is the Pivot tool's, not a drag kind). Ask [`is_gizmo_id`] for *"is this id the
/// gizmo's at all"*.
pub fn is_gizmo_handle_id(id: NodeId) -> bool {
    gizmo_kind_for_id(id).is_some()
}

/// True iff `id` belongs to the gizmo **at all** — any of its reserved hit zones, the pivot dot
/// included.
///
/// This is the question *"is this thing CHROME, or is it drawn on the artwork?"*, answered by the module
/// that owns the ids. The gizmo is **not chrome**: it is painted over the very sprite the artist is
/// painting, so a host guard that treats "the hit index claims this pixel" as "the editor's UI owns this
/// pixel" would carve dead zones out of the picture exactly at the sprite's corners and edges.
///
/// [`is_gizmo_handle_id`] cannot answer it: the pivot dot is a hit zone and is not a drag kind, so it
/// reports `false` for `GIZMO_PIVOT`. Every caller that wanted *"the gizmo's"* had to remember to add
/// `|| id == GIZMO_PIVOT` by hand — which is a list, and lists rot. This is the door.
pub fn is_gizmo_id(id: NodeId) -> bool {
    is_gizmo_handle_id(id) || id == ids::GIZMO_PIVOT
}

/// Reserved [`NodeId`]s for the gizmo's 13 hit zones (8 handles +
/// 4 rotation regions + 1 bbox-interior + 1 pivot dot). Range
/// 950-963 — past the M14.6 F context menu range (940-948) and
/// well below the bridge's `BASE_NODE_ID` (100_000).
pub mod ids {
    use super::NodeId;
    pub const GIZMO_BBOX_INTERIOR: NodeId = NodeId(950);
    pub const GIZMO_PIVOT: NodeId = NodeId(951);
    // 4 corner scale handles.
    pub const GIZMO_HANDLE_TL: NodeId = NodeId(952);
    pub const GIZMO_HANDLE_TR: NodeId = NodeId(953);
    pub const GIZMO_HANDLE_BL: NodeId = NodeId(954);
    pub const GIZMO_HANDLE_BR: NodeId = NodeId(955);
    // 4 edge axis-scale handles.
    pub const GIZMO_HANDLE_T: NodeId = NodeId(956);
    pub const GIZMO_HANDLE_R: NodeId = NodeId(957);
    pub const GIZMO_HANDLE_B: NodeId = NodeId(958);
    pub const GIZMO_HANDLE_L: NodeId = NodeId(959);
    // 4 rotate-hover regions just outside each corner. Hit by the
    // outer edges of `corner_outer_rect` minus the corner handle.
    pub const GIZMO_ROTATE_TL: NodeId = NodeId(960);
    pub const GIZMO_ROTATE_TR: NodeId = NodeId(961);
    pub const GIZMO_ROTATE_BL: NodeId = NodeId(962);
    pub const GIZMO_ROTATE_BR: NodeId = NodeId(963);
}
