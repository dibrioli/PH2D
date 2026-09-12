//! Gradient-Map (W4 BATCH-2) fixed routing NodeIds (`PAINTER_GRADIENT_*`) — panel → tool stop
//! add/remove/drag/colour. The per-layer derive helpers (`painter_gradient_*_id`) stay in `painter`
//! (they derive through the runtime door `hash_node_id_runtime`); split out here for the file-LOC cap.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/painter_gradient.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Fixed routing id — panel → tool Gradient-Map stop drag (W4 BATCH-2). Payload
/// `"layer:index:offset"`; the tool calls `set_gradient_stop_offset`.
pub const PAINTER_GRADIENT_EDIT: NodeId = hash_node_id("painter_gradient_edit");

/// Fixed routing id — panel → tool "add a gradient stop". Payload `"layer"`.
pub const PAINTER_GRADIENT_ADD: NodeId = hash_node_id("painter_gradient_add");

/// Fixed routing id — panel → tool "remove a gradient stop". Payload `"layer:index"`.
pub const PAINTER_GRADIENT_REMOVE: NodeId = hash_node_id("painter_gradient_remove");

/// Fixed routing id — panel → tool selected-stop RGB edit (W4 BATCH-2). Payload
/// `"layer:stop:slot:value"`; the tool calls `set_gradient_stop_color`.
pub const PAINTER_GRADIENT_COLOR: NodeId = hash_node_id("painter_gradient_color");
