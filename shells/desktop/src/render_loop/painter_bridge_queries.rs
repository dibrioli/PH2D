//! Painter bridge — tool-concrete DOWNCAST query helpers, split out of
//! `painter_bridge.rs` to keep it under the HR-18 file-LOC cap.
//!
//! The helper needs the `PainterTool` concrete type, so it lives in an
//! allowlisted bridge file to keep the central dispatch free of tool-concrete
//! downcasts per the `architecture_no_downcast_to_concrete_tool_in_shell` gate.

use ph2d_editor::ToolRegistry;

/// Apply a Painter layers drag-reparent (W3 T3.8) emitted by the dispatch on Up
/// of an active layer-row drag. The active `PainterTool` reverses
/// `NodeId`→`LayerId` and applies `move_into_group` / reorder. Mirror of
/// [`painter_has_unflushed_strokes`]. No-op if the active tool is not the Painter.
pub(crate) fn apply_layer_reparent(
    tools: &mut ToolRegistry,
    dragged: ph2d_editor::NodeId,
    drop: ph2d_editor::interaction::PainterLayerDrop,
) {
    if let Some(painter) = tools.active_mut().and_then(|t| {
        t.as_any_mut()
            .downcast_mut::<ph2d_tool_painter::PainterTool>()
    }) {
        painter.handle_layer_reparent(dragged, drop);
    }
}
