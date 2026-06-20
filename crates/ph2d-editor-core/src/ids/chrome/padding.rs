//! Padding panel widget NodeIds (PAD_*).
use super::{NodeId, hash_node_id};

/// Padding panel widget NodeIds (typed `ph2d-panel-padding`, right-docked
/// in the Inspector slot while the `padding` tool is active). Four signed
/// per-edge rows — each a bipolar Slider (`PAD_*`) linked in real time to
/// a px-valued NumberInput chip (`PAD_*_NUM`) — plus a pivot-mode toggle
/// with Apply / Cancel. The directional-expand gizmo (v2) will add
/// canvas-edge handles, not panel ids.
pub const PAD_PANEL: NodeId = hash_node_id("pad_panel");
pub const PAD_TOP: NodeId = hash_node_id("pad_top");
pub const PAD_RIGHT: NodeId = hash_node_id("pad_right");
pub const PAD_BOTTOM: NodeId = hash_node_id("pad_bottom");
pub const PAD_LEFT: NodeId = hash_node_id("pad_left");
/// Per-edge px chips paired with the `PAD_*` sliders (real-time link).
pub const PAD_TOP_NUM: NodeId = hash_node_id("pad_top_num");
pub const PAD_RIGHT_NUM: NodeId = hash_node_id("pad_right_num");
pub const PAD_BOTTOM_NUM: NodeId = hash_node_id("pad_bottom_num");
pub const PAD_LEFT_NUM: NodeId = hash_node_id("pad_left_num");
/// Pivot-mode toggle: ON = recenter (recalculate translation so the
/// original content stays world-fixed); OFF = keep the pivot unchanged.
pub const PAD_PIVOT_RECENTER: NodeId = hash_node_id("pad_pivot_recenter");
pub const PAD_APPLY: NodeId = hash_node_id("pad_apply");
pub const PAD_CANCEL: NodeId = hash_node_id("pad_cancel");
/// Reset-all button — returns every per-edge padding back to 0 and
/// the pivot-recenter toggle to its default. Also fires from the
/// tool's `on_activate`.
pub const PAD_RESET: NodeId = hash_node_id("pad_reset");
