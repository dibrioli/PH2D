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
/// **Motion Path** mode toggle (ADR-0141) — PER OBJECT: the SELECTED object animates
/// its position as one trajectory (`on`) or separate X/Y (`off`). Reflects the
/// selection; clicking converts that object (Convert to Motion Path / to Separate).
pub const TIMELINE_MOTION_PATH: NodeId = hash_node_id("timeline.motion_path");
/// Frame-snap toggle.
pub const TIMELINE_SNAP: NodeId = hash_node_id("timeline.snap");
/// **Ping-pong** loop toggle — the same loop as [`TIMELINE_LOOP`], played back
/// and forth. Mutually exclusive with it *by construction*: both read one value
/// (a range plus a mode), and no value is both.
pub const TIMELINE_PINGPONG: NodeId = hash_node_id("timeline.ping_pong");

/// **Physics** arm toggle (ADR-0131) — does Play drive the rigid simulation as
/// well as the curves? One transport, two consumers; this is which of them the
/// clock reaches. Off by default (`TimelineFlags::simulate_physics`).
pub const TIMELINE_PHYSICS: NodeId = hash_node_id("timeline.physics");

/// **Onion Settings** button (ADR-0142 W3b) — abre o card flutuante das contagens/opacidade/
/// cores. O botão VIVE na barra de transporte (panel), mas o card é chrome flutuante no shell
/// (não cabe na timeline); o clique é resolvido shell-side (tem `hero.store`), como o
/// [`TIMELINE_MOTION_PATH`].
pub const TIMELINE_ONION_SETTINGS: NodeId = hash_node_id("timeline.onion_settings");

// ── Onion settings modal (ADR-0142 W3b) — floating draggable card, hero chrome ──
/// Title band = the drag handle (Primary Down here starts a modal-move, shell-driven).
pub const TIMELINE_ONION_MODAL_HANDLE: NodeId = hash_node_id("timeline.onion_modal_handle");
/// Close (X) button in the title band.
pub const TIMELINE_ONION_MODAL_CLOSE: NodeId = hash_node_id("timeline.onion_modal_close");
/// Opacity slider (`0..1`) — the nearest ghost's alpha.
pub const TIMELINE_ONION_MODAL_OPACITY: NodeId = hash_node_id("timeline.onion_modal_opacity");
/// "Ghosts before" slider — `0..1` mapped to a count by the shell (the count↔slider mapping
/// lives ONLY there, so there is one copy of it — editor-core stays timeline-agnostic).
pub const TIMELINE_ONION_MODAL_BEFORE: NodeId = hash_node_id("timeline.onion_modal_before");
/// "Ghosts after" slider — same mapping.
pub const TIMELINE_ONION_MODAL_AFTER: NodeId = hash_node_id("timeline.onion_modal_after");
/// Past-ghost colour swatch (opens the shared OKLCH picker — `register_picker_swatch`).
pub const TIMELINE_ONION_MODAL_COLOR_BEFORE: NodeId =
    hash_node_id("timeline.onion_modal_color_before");
/// Future-ghost colour swatch.
pub const TIMELINE_ONION_MODAL_COLOR_AFTER: NodeId =
    hash_node_id("timeline.onion_modal_color_after");

/// The time ruler strip (scrub hit-target).
pub const TIMELINE_RULER: NodeId = hash_node_id("timeline.ruler");

/// "+M" button in the transport bar — drops a marker at the playhead (W4.T3).
pub const TIMELINE_ADD_MARKER: NodeId = hash_node_id("timeline.add_marker");
