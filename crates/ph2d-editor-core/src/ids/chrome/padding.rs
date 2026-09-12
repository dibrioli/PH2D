//! Padding panel widget NodeIds (PAD_*).
use super::{NodeId, hash_node_id};

/// Padding panel widget NodeIds (typed `ph2d-panel-padding`, right-docked
/// in the Inspector slot while the `padding` tool is active). Four signed
/// per-edge rows — each a bipolar Slider (`PAD_*`) linked in real time to
/// a px-valued NumberInput chip (`PAD_*_NUM`) — plus a pivot-mode toggle
/// with Apply / Cancel. The directional-expand gizmo (v2) will add
/// canvas-edge handles, not panel ids.
pub const PAD_PANEL: NodeId = hash_node_id("pad_panel");
