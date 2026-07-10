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
/// Draggable splitter between the track-name column and the time area.
pub const TIMELINE_LABEL_SPLIT: NodeId = hash_node_id("timeline.label_split");
/// Padlock on the Summary channel — toggles the column lock (see
/// [`crate::interaction::TimelineHitKind::SummaryLock`]).
pub const TIMELINE_SUMMARY_LOCK: NodeId = hash_node_id("timeline.summary_lock");

// ── Resize grippers (4 edges + 4 corners) ────────────────────────────────────
/// Left edge.
pub const TIMELINE_RESIZE_L: NodeId = hash_node_id("timeline.resize.l");
/// Right edge.
pub const TIMELINE_RESIZE_R: NodeId = hash_node_id("timeline.resize.r");
/// Top edge (drag up to grow the dock taller).
pub const TIMELINE_RESIZE_T: NodeId = hash_node_id("timeline.resize.t");
/// Bottom edge.
pub const TIMELINE_RESIZE_B: NodeId = hash_node_id("timeline.resize.b");
/// Top-left corner.
pub const TIMELINE_RESIZE_TL: NodeId = hash_node_id("timeline.resize.tl");
/// Top-right corner.
pub const TIMELINE_RESIZE_TR: NodeId = hash_node_id("timeline.resize.tr");
/// Bottom-left corner.
pub const TIMELINE_RESIZE_BL: NodeId = hash_node_id("timeline.resize.bl");
/// Bottom-right corner.
pub const TIMELINE_RESIZE_BR: NodeId = hash_node_id("timeline.resize.br");

/// A stable `NodeId` for one key-diamond hit target, keyed by the track's
/// `AnimTarget` and the key's `KeyId` (both raw u64s). Stable across frames so a
/// dope-sheet drag's active capture keeps resolving to the same key even as
/// keys move — the hit is keyed by *identity*, not row/column position. Seeded
/// from the `timeline.key` domain so it never collides with the `timeline.*`
/// slug consts. The key set is dynamic, so these can't be consts.
#[must_use]
pub fn timeline_key_hit_id(target: u64, key: u64) -> NodeId {
    dynamic_id("timeline.key", &[target, key])
}

/// A stable `NodeId` for one track row's expand/collapse twirl (W3.E1), keyed by
/// the track's `AnimTarget`.
#[must_use]
pub fn timeline_twirl_id(target: u64) -> NodeId {
    dynamic_id("timeline.twirl", &[target])
}

/// A stable `NodeId` for one expanded row's graph-height grip, keyed by the
/// track's `AnimTarget` (every grip drags the same shared height).
#[must_use]
pub fn timeline_graph_resize_id(target: u64) -> NodeId {
    dynamic_id("timeline.graph_resize", &[target])
}

/// A stable `NodeId` for one key's anchor dot in the graph editor (W3.E5),
/// keyed by the track's `AnimTarget` and the key's `KeyId`. A separate domain
/// from [`timeline_key_hit_id`]: the same key owns a diamond in the dope-sheet
/// strip AND an anchor in the band below it, and they must not share an id.
#[must_use]
pub fn timeline_anchor_hit_id(target: u64, key: u64) -> NodeId {
    dynamic_id("timeline.anchor", &[target, key])
}

/// A stable `NodeId` for one Summary-channel column, keyed by its time in
/// seconds as `f64::to_bits`. Its own domain: the summary diamond at `t` and a
/// track's key at `t` are different targets.
#[must_use]
pub fn timeline_summary_hit_id(t_bits: u64) -> NodeId {
    dynamic_id("timeline.summary", &[t_bits])
}

/// A stable `NodeId` for a loop-range brace on the ruler, keyed by `edge`
/// (`0` = start, `1` = end, `2` = the band between them).
#[must_use]
pub fn timeline_loop_brace_id(edge: u8) -> NodeId {
    dynamic_id("timeline.loop_brace", &[u64::from(edge)])
}

/// A stable `NodeId` for one bézier handle in the graph editor (W3.E3), keyed by
/// the segment's owning `(target, key)` and `which` (`0` = out handle `P1`,
/// `1` = in handle `P2`).
#[must_use]
pub fn timeline_handle_hit_id(target: u64, key: u64, which: u8) -> NodeId {
    dynamic_id("timeline.handle", &[target, key, u64::from(which)])
}

/// FNV-1a over `parts`' little-endian bytes, seeded from `domain`'s slug hash.
/// Each dynamic id family gets its own domain, so a twirl for target `7` and a
/// key hit for target `7` can never land on the same `NodeId` — nor on any
/// `timeline.*` const.
fn dynamic_id(domain: &'static str, parts: &[u64]) -> NodeId {
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = hash_node_id(domain).0;
    for b in parts.iter().flat_map(|p| p.to_le_bytes()) {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    NodeId(h)
}
