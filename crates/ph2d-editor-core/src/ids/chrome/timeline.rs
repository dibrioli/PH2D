//! General timeline panel chrome NodeIds (`TIMELINE_*`).
//!
//! The bottom-docked `ph2d-panel-timeline` (plan `docs/Timeline/`): a transport
//! bar + ruler + dope-sheet. Distinct slug family (`timeline.*`) from the Motion
//! graph's `motion.*` ids. Populated incrementally across W2 (transport/ruler
//! first; lane/key ids are dynamic fnv64, not consts).
use super::{NodeId, hash_node_id};

/// Timeline panel outer rect (for `z_order` + hit-barrier).
pub const TIMELINE_PANEL: NodeId = hash_node_id("timeline.panel");

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
