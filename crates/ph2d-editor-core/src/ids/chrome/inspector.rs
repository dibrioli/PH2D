//! Inspector panel container chrome NodeIds (INSP_*).
use super::{NodeId, hash_node_id};

/// Inspector panel container — used as the wheel-scroll key.
pub const INSP_PANEL: NodeId = hash_node_id("insp_panel");
/// Close (X) button at the top-right of the Inspector — toggles
/// `panel_visibility["inspector"]` same as the left-rail Inspector
/// pill. UI canon post-2026-05-24: every floating panel except
/// Hierarchy carries a close X.
pub const INSP_CLOSE: NodeId = hash_node_id("insp_close");
/// Drag handle at the top of the Inspector — click+drag moves the
/// panel. Registered as `BlenderHit { parent: INSP_PANEL, kind:
/// DragHandle }` so the existing picker-drag dispatch infra
/// (panel-agnostic on parent NodeId) drives it.
pub const INSP_DRAG_HANDLE: NodeId = hash_node_id("insp_drag_handle");
/// Resize gripper at the Inspector's bottom-right corner. Registered
/// as `BlenderHit { parent: INSP_PANEL, kind: ResizeHandle }`.
pub const INSP_RESIZE_HANDLE: NodeId = hash_node_id("insp_resize_handle");
/// Resize gripper at the Inspector's bottom-LEFT corner. Mirror of
/// [`INSP_RESIZE_HANDLE`]. Registered as
/// `BlenderHit { parent: INSP_PANEL, kind: ResizeHandleBl }`.
pub const INSP_RESIZE_HANDLE_BL: NodeId = hash_node_id("insp_resize_handle_bl");
