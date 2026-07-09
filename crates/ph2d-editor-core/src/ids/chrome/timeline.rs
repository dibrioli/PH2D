//! General timeline panel chrome NodeIds (`TIMELINE_*`).
//!
//! The bottom-docked `ph2d-panel-timeline` (plan `docs/Timeline/`): a transport
//! bar + ruler + dope-sheet. Distinct slug family (`timeline.*`) from the Motion
//! graph's `motion.*` ids. Populated incrementally across W2 (transport/ruler
//! first; lane/key ids are dynamic fnv64, not consts).
use super::{NodeId, hash_node_id};

/// Timeline panel outer rect (for `z_order` + hit-barrier).
pub const TIMELINE_PANEL: NodeId = hash_node_id("timeline.panel");
/// Timeline panel close (X) button.
pub const TIMELINE_CLOSE: NodeId = hash_node_id("timeline.close");

// ── Transport bar ────────────────────────────────────────────────────────────
/// Play / Pause toggle.
pub const TIMELINE_PLAY: NodeId = hash_node_id("timeline.play");
/// Jump to start (frame 0).
pub const TIMELINE_GO_START: NodeId = hash_node_id("timeline.go_start");
/// Jump to end (clip duration).
pub const TIMELINE_GO_END: NodeId = hash_node_id("timeline.go_end");
/// Step one frame back.
pub const TIMELINE_PREV_FRAME: NodeId = hash_node_id("timeline.prev_frame");
/// Step one frame forward.
pub const TIMELINE_NEXT_FRAME: NodeId = hash_node_id("timeline.next_frame");
/// Editable seconds chip (seek).
pub const TIMELINE_TIME_NUM: NodeId = hash_node_id("timeline.time_num");
/// Editable frame chip (seek).
pub const TIMELINE_FRAME_NUM: NodeId = hash_node_id("timeline.frame_num");
/// Loop-range toggle.
pub const TIMELINE_LOOP: NodeId = hash_node_id("timeline.loop");
/// Auto-key arm toggle.
pub const TIMELINE_AUTOKEY: NodeId = hash_node_id("timeline.autokey");
/// Frame-snap toggle.
pub const TIMELINE_SNAP: NodeId = hash_node_id("timeline.snap");

// ── Track list + ruler ───────────────────────────────────────────────────────
/// "+ Track" button (adds a binding for the selected object's property).
pub const TIMELINE_ADD_TRACK: NodeId = hash_node_id("timeline.add_track");
/// The six per-property "+Track" buttons — bind the selected sprite's
/// Translation X/Y · Rotation · Scale X/Y · Opacity (order matches
/// `PropKind::ALL`).
pub const TIMELINE_ADDPROP_TX: NodeId = hash_node_id("timeline.addprop.tx");
pub const TIMELINE_ADDPROP_TY: NodeId = hash_node_id("timeline.addprop.ty");
pub const TIMELINE_ADDPROP_ROT: NodeId = hash_node_id("timeline.addprop.rot");
pub const TIMELINE_ADDPROP_SX: NodeId = hash_node_id("timeline.addprop.sx");
pub const TIMELINE_ADDPROP_SY: NodeId = hash_node_id("timeline.addprop.sy");
pub const TIMELINE_ADDPROP_OPACITY: NodeId = hash_node_id("timeline.addprop.opacity");
/// The time ruler strip (scrub hit-target).
pub const TIMELINE_RULER: NodeId = hash_node_id("timeline.ruler");
/// The dope-sheet lanes background (box-select / deselect hit-target).
pub const TIMELINE_LANES: NodeId = hash_node_id("timeline.lanes");
/// Vertical scrollbar for the track list.
pub const TIMELINE_SCROLLBAR: NodeId = hash_node_id("timeline.scrollbar");
