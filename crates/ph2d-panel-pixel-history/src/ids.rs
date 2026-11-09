//! **Os ids da janela UNDO HISTORY.** ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

pub const PANEL: NodeId = ph2d_editor_core::ids::PIXEL_HISTORY_PANEL;
pub const HISTORY_DRAG: NodeId = hash_node_id("pixel_history.drag");
pub const HISTORY_EDGE: NodeId = hash_node_id("pixel_history.edge");
pub const HISTORY_SEC_LIST: NodeId = hash_node_id("pixel_history.sec.list");

/// A linha `i` da lista (0 = `Initial State`).
#[must_use]
pub fn row(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel_history.row.{i}"))
}
