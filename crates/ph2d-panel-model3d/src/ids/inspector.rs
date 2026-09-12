//! Inspector panel container chrome NodeIds (INSP_*).
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/inspector.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Drag handle at the top of the Inspector — click+drag moves the
/// panel. Registered as `BlenderHit { parent: INSP_PANEL, kind:
/// DragHandle }` so the existing picker-drag dispatch infra
/// (panel-agnostic on parent NodeId) drives it.
pub const INSP_DRAG_HANDLE: NodeId = hash_node_id("insp_drag_handle");
