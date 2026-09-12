//! Inspector panel container chrome NodeIds (INSP_*).
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/inspector.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Resize gripper at the Inspector's bottom-right corner. Registered
/// as `BlenderHit { parent: INSP_PANEL, kind: ResizeHandle }`.
pub const INSP_RESIZE_HANDLE: NodeId = hash_node_id("insp_resize_handle");

/// Resize gripper at the Inspector's bottom-LEFT corner. Mirror of
/// [`INSP_RESIZE_HANDLE`]. Registered as
/// `BlenderHit { parent: INSP_PANEL, kind: ResizeHandleBl }`.
pub const INSP_RESIZE_HANDLE_BL: NodeId = hash_node_id("insp_resize_handle_bl");
