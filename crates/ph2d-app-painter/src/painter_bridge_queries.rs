//! Painter bridge — tool-concrete DOWNCAST query helpers, split out of
//! `painter_bridge.rs` to keep it under the HR-18 file-LOC cap.
//!
//! The helper needs the `PainterTool` concrete type, so it lives in an
//! allowlisted bridge file to keep the central dispatch free of tool-concrete
//! downcasts per the `architecture_no_downcast_to_concrete_tool_in_shell` gate.

use ph2d_editor_core::ToolRegistry;

/// **O Painter do registo, activo ou não** — a porta da shell para o `PainterTool` concreto por id.
pub fn o_painter(tools: &mut ToolRegistry) -> Option<&mut ph2d_tool_painter::PainterTool> {
    tools
        .tool_by_id_mut(&ph2d_editor_core::ToolId::new("painter"))?
        .as_any_mut()
        .downcast_mut()
}

/// **A imagem que um verbo SÓ-LEITURA deve levar da sprite `entity`** (RGBA8 straight) enquanto o
/// Painter a pinta sem a ter assado: a tinta vive no tool e a textura da sprite ainda é a de antes
/// (Enio 24/10: o «Export Image…» de um gato pintado saiu BRANCO; `BUGS_painter_fechados_41_a_46.md` #53). `None` = a sprite é a verdade
/// (outro documento, ou nada por assar — o leitor dela guarda os 16 bits).
pub fn imagem_por_assar(tools: &mut ToolRegistry, entity: u64) -> Option<(Vec<u8>, u32, u32)> {
    imagem_por_assar_do(o_painter(tools)?, entity)
}

/// [`imagem_por_assar`] sobre o tool já na mão.
#[must_use]
pub fn imagem_por_assar_do(
    p: &ph2d_tool_painter::PainterTool,
    entity: u64,
) -> Option<(Vec<u8>, u32, u32)> {
    (p.bound_document() == Some(entity) && p.has_unbaked_edits())
        .then(|| p.imagem_aplicada())
        .flatten()
}

/// Apply a Painter layers drag-reparent (W3 T3.8) emitted by the dispatch on Up
/// of an active layer-row drag. The active `PainterTool` reverses
/// `NodeId`→`LayerId` and applies `move_into_group` / reorder. Mirror of
/// [`painter_has_unflushed_strokes`]. No-op if the active tool is not the Painter.
/// `true` while the active Painter's **Deform Transform** gizmo is live (any of the 4 sub-modes —
/// Uniform / Free / Distort / Warp, with a lifted floating patch). The render loop then SUPPRESSES the
/// sprite gizmo for the frame: on a whole-image transform both gizmos occupy the same screen corners,
/// and a near-corner Down grabbed the SPRITE's scale handle instead of the deform's (Enio 2026-07-04,
/// "Transform uniform/free interferem no gizmo da sprite — inative o gizmo da sprite").
pub fn deform_transform_gizmo_active(tools: &mut ToolRegistry) -> bool {
    tools
        .active_mut()
        .and_then(|t| {
            t.as_any_mut()
                .downcast_mut::<ph2d_tool_painter::PainterTool>()
        })
        .is_some_and(|p| p.deform_gizmo_grab_margin_px() > 0.0)
}

pub fn apply_layer_reparent(
    tools: &mut ToolRegistry,
    dragged: ph2d_editor_core::NodeId,
    drop: ph2d_editor_core::interaction::PanelRowDrop,
) {
    if let Some(painter) = tools.active_mut().and_then(|t| {
        t.as_any_mut()
            .downcast_mut::<ph2d_tool_painter::PainterTool>()
    }) {
        painter.handle_layer_reparent(dragged, drop);
    }
}
