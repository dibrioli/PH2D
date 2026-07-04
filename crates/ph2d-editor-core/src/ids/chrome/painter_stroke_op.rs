//! Stroke **Operation** (multi-shape, Enio 2026-07-04) NodeIds — the boolean mode a NEW shape is created
//! with, mirroring the Selection Operation but with Overlay (no boolean) replacing New. Fixed-id,
//! tool-global widgets registered in the painter-layers `populate` and routed over the `PanelEvent` channel.
//! Split from [`super::painter`] for the workspace file-LOC cap.
use super::{NodeId, hash_node_id};

/// Stroke Operation segmented GROUP. `Click` on a child → `set_stroke_op_mode`.
pub const PAINTER_STROKE_OP: NodeId = hash_node_id("painter_brush.stroke_op"); // group
/// Overlay op (no boolean — the shape paints independently; the default). `Click` → op `0`.
pub const PAINTER_STROKE_OP_OVERLAY: NodeId = hash_node_id("painter_brush.stroke_op_overlay");
/// Add op (boolean union with overlapping Add/Remove shapes). `Click` → op `1`.
pub const PAINTER_STROKE_OP_ADD: NodeId = hash_node_id("painter_brush.stroke_op_add");
/// Remove op (boolean subtract). `Click` → op `2`.
pub const PAINTER_STROKE_OP_REMOVE: NodeId = hash_node_id("painter_brush.stroke_op_remove");
/// The three Stroke Operation children, index-aligned with the op wire value (`0`=Overlay `1`=Add `2`=Remove).
pub const PAINTER_STROKE_OP_IDS: [NodeId; 3] = [
    PAINTER_STROKE_OP_OVERLAY,
    PAINTER_STROKE_OP_ADD,
    PAINTER_STROKE_OP_REMOVE,
];
/// The framed "OPERATION" card wrapping the stroke Operation group.
pub const PAINTER_STROKE_OP_CARD: NodeId = hash_node_id("painter_brush.stroke_op_card");
