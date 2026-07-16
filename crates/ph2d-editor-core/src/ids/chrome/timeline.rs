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
/// Performing (record-during-play, W5) arm toggle.
pub const TIMELINE_RECORD: NodeId = hash_node_id("timeline.record");
/// Frame-snap toggle.
pub const TIMELINE_SNAP: NodeId = hash_node_id("timeline.snap");
/// **Ping-pong** loop toggle — the same loop as [`TIMELINE_LOOP`], played back
/// and forth. Mutually exclusive with it *by construction*: both read one value
/// (a range plus a mode), and no value is both.
pub const TIMELINE_PINGPONG: NodeId = hash_node_id("timeline.ping_pong");
/// Speed-graph view toggle (W5) — flips every expanded graph band between the
/// VALUE curve and the VELOCITY (speed) curve. Panel-local view state.
pub const TIMELINE_SPEED: NodeId = hash_node_id("timeline.speed");

// ── View tabs (ADR-0115 R8, amended) ─────────────────────────────────────────
/// The tab strip itself (the `TabList` a11y parent).
pub const TIMELINE_TABS: NodeId = hash_node_id("timeline.tabs");
/// **Keys** tab — the active clip's dope sheet + graph editor, ruled by the
/// CLIP's clock.
pub const TIMELINE_TAB_KEYS: NodeId = hash_node_id("timeline.tab_keys");
/// **Arrange** tab — the clip stack (lanes + strips), ruled by the TIMELINE's
/// clock.
///
/// A tab is a VIEW, not a mode: it changes what the panel shows and what its
/// ruler measures, never what an edit means. The clip dropdown still decides
/// which clip's keys the Keys tab holds — which is why ADR-0115 R8's rejection of
/// Blender's *tweak mode* survives this amendment intact.
pub const TIMELINE_TAB_ARRANGE: NodeId = hash_node_id("timeline.tab_arrange");

// ── Clip selector (W5 / NLA step 1) ──────────────────────────────────────────
/// The clip dropdown chip — shows the active clip's name, opens the clip list.
pub const TIMELINE_CLIP_DD: NodeId = hash_node_id("timeline.clip_dd");
/// "+" — append a new, empty clip and make it active.
pub const TIMELINE_ADD_CLIP: NodeId = hash_node_id("timeline.clip_add");
/// Pencil — open the inline rename field for the ACTIVE clip.
pub const TIMELINE_RENAME_CLIP: NodeId = hash_node_id("timeline.clip_rename");
/// Trash — delete the ACTIVE clip. Never painted (nor hit-registered) while the
/// document holds only one: a document must always have a clip to edit.
pub const TIMELINE_DELETE_CLIP: NodeId = hash_node_id("timeline.clip_delete");
/// The inline clip-rename text field (mirrors [`TIMELINE_MARKER_RENAME_INPUT`]).
pub const TIMELINE_CLIP_RENAME_INPUT: NodeId = hash_node_id("timeline.clip_rename_input");

/// "+ Lane" — append a lane to the clip stack (ADR-0115).
pub const TIMELINE_ADD_LANE: NodeId = hash_node_id("timeline.add_lane");

/// Per-lane mute toggles. **This array IS the lane cap** — same rule as
/// [`TIMELINE_CLIP_OPT`]: the chrome cannot mint a hit id at runtime, so a
/// document that accepted a 9th lane would paint a header nothing could click.
/// `ph2d_timeline::MAX_LANES` must equal this length, and a gate holds them
/// together.
pub const TIMELINE_LANE_MUTE: [NodeId; 8] = [
    hash_node_id("timeline.lane_mute_0"),
    hash_node_id("timeline.lane_mute_1"),
    hash_node_id("timeline.lane_mute_2"),
    hash_node_id("timeline.lane_mute_3"),
    hash_node_id("timeline.lane_mute_4"),
    hash_node_id("timeline.lane_mute_5"),
    hash_node_id("timeline.lane_mute_6"),
    hash_node_id("timeline.lane_mute_7"),
];

/// Per-lane "drop the active clip here" buttons. Same cap, same reason.
pub const TIMELINE_LANE_ADD_STRIP: [NodeId; 8] = [
    hash_node_id("timeline.lane_add_strip_0"),
    hash_node_id("timeline.lane_add_strip_1"),
    hash_node_id("timeline.lane_add_strip_2"),
    hash_node_id("timeline.lane_add_strip_3"),
    hash_node_id("timeline.lane_add_strip_4"),
    hash_node_id("timeline.lane_add_strip_5"),
    hash_node_id("timeline.lane_add_strip_6"),
    hash_node_id("timeline.lane_add_strip_7"),
];

/// Per-lane weight fields (`[0, 1]`). Same cap, same reason.
pub const TIMELINE_LANE_WEIGHT: [NodeId; 8] = [
    hash_node_id("timeline.lane_weight_0"),
    hash_node_id("timeline.lane_weight_1"),
    hash_node_id("timeline.lane_weight_2"),
    hash_node_id("timeline.lane_weight_3"),
    hash_node_id("timeline.lane_weight_4"),
    hash_node_id("timeline.lane_weight_5"),
    hash_node_id("timeline.lane_weight_6"),
    hash_node_id("timeline.lane_weight_7"),
];

/// Per-lane label strips — the R-click surface that opens the lane menu (mode,
/// Delete Lane). Same cap, same reason.
pub const TIMELINE_LANE_ROW: [NodeId; 8] = [
    hash_node_id("timeline.lane_row_0"),
    hash_node_id("timeline.lane_row_1"),
    hash_node_id("timeline.lane_row_2"),
    hash_node_id("timeline.lane_row_3"),
    hash_node_id("timeline.lane_row_4"),
    hash_node_id("timeline.lane_row_5"),
    hash_node_id("timeline.lane_row_6"),
    hash_node_id("timeline.lane_row_7"),
];

/// Clip dropdown option ids — one per clip slot, index = clip index.
///
/// **This array IS the clip cap.** The chrome cannot mint a hit id at runtime, so
/// the number of clips a document may hold is exactly the number of ids here, and
/// `ph2d_timeline::MAX_CLIPS` must equal `TIMELINE_CLIP_OPT.len()` — a gate holds
/// the two together, because a doc that accepted a 17th clip would paint an option
/// nothing could click.
pub const TIMELINE_CLIP_OPT: [NodeId; 16] = [
    hash_node_id("timeline.clip_opt_0"),
    hash_node_id("timeline.clip_opt_1"),
    hash_node_id("timeline.clip_opt_2"),
    hash_node_id("timeline.clip_opt_3"),
    hash_node_id("timeline.clip_opt_4"),
    hash_node_id("timeline.clip_opt_5"),
    hash_node_id("timeline.clip_opt_6"),
    hash_node_id("timeline.clip_opt_7"),
    hash_node_id("timeline.clip_opt_8"),
    hash_node_id("timeline.clip_opt_9"),
    hash_node_id("timeline.clip_opt_10"),
    hash_node_id("timeline.clip_opt_11"),
    hash_node_id("timeline.clip_opt_12"),
    hash_node_id("timeline.clip_opt_13"),
    hash_node_id("timeline.clip_opt_14"),
    hash_node_id("timeline.clip_opt_15"),
];

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
/// "+Track" for **Time Remap** (W5): binds the entity's per-object clock — a
/// keyed curve retiming every other track it has (AE time remap).
pub const TIMELINE_ADDPROP_TIME: NodeId = hash_node_id("timeline.addprop.time");
/// The time ruler strip (scrub hit-target).
pub const TIMELINE_RULER: NodeId = hash_node_id("timeline.ruler");
/// The dope-sheet lanes background (box-select / deselect hit-target).
pub const TIMELINE_LANES: NodeId = hash_node_id("timeline.lanes");
/// Vertical scrollbar for the track list.
pub const TIMELINE_SCROLLBAR: NodeId = hash_node_id("timeline.scrollbar");
/// Draggable splitter between the track-name column and the time area.
pub const TIMELINE_LABEL_SPLIT: NodeId = hash_node_id("timeline.label_split");
/// "+M" button in the transport bar — drops a marker at the playhead (W4.T3).
pub const TIMELINE_ADD_MARKER: NodeId = hash_node_id("timeline.add_marker");
/// Padlock on the Summary channel — toggles the column lock (see
/// [`crate::interaction::TimelineHitKind::SummaryLock`]).
pub const TIMELINE_SUMMARY_LOCK: NodeId = hash_node_id("timeline.summary_lock");
/// Inline single-line `TextInput` for renaming a marker (W4.T3), opened by a
/// double-click on the marker's pennant. One shared field (only one rename is
/// open at a time — the panel tracks which marker index).
pub const TIMELINE_MARKER_RENAME_INPUT: NodeId = hash_node_id("timeline.marker_rename_input");

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

/// A stable `NodeId` for one track row's LABEL (the name column slice after the
/// twirl), keyed by the track's `AnimTarget`. Right-clicking it opens the
/// track-row context menu ([`crate::ids::TIMELINE_TRACK_MENU`]).
#[must_use]
pub fn timeline_row_id(target: u64) -> NodeId {
    dynamic_id("timeline.row", &[target])
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

/// A stable `NodeId` for a marker pennant on the ruler, keyed by its storage
/// index.
#[must_use]
pub fn timeline_marker_hit_id(index: usize) -> NodeId {
    dynamic_id("timeline.marker", &[index as u64])
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
/// Hit id for one strip's grab region: the body (`edge` 2) or either edge
/// (`0` = start, `1` = end). Dynamic: strips are created and destroyed at
/// runtime, so unlike a lane header they cannot come from a fixed array.
#[must_use]
pub fn timeline_strip_hit_id(lane: u64, strip: u64, edge: u8) -> NodeId {
    dynamic_id("timeline.strip", &[lane, strip, u64::from(edge)])
}

fn dynamic_id(domain: &'static str, parts: &[u64]) -> NodeId {
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = hash_node_id(domain).0;
    for b in parts.iter().flat_map(|p| p.to_le_bytes()) {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    NodeId(h)
}
