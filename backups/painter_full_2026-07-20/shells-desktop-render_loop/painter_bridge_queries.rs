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
/// `true` while the active Painter's **Deform Transform** gizmo is live (any of the 4 sub-modes —
/// Uniform / Free / Distort / Warp, with a lifted floating patch). The render loop then SUPPRESSES the
/// sprite gizmo for the frame: on a whole-image transform both gizmos occupy the same screen corners,
/// and a near-corner Down grabbed the SPRITE's scale handle instead of the deform's (Enio 2026-07-04,
/// "Transform uniform/free interferem no gizmo da sprite — inative o gizmo da sprite").
pub(crate) fn deform_transform_gizmo_active(tools: &mut ToolRegistry) -> bool {
    tools
        .active_mut()
        .and_then(|t| {
            t.as_any_mut()
                .downcast_mut::<ph2d_tool_painter::PainterTool>()
        })
        .is_some_and(|p| p.deform_gizmo_grab_margin_px() > 0.0)
}

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
