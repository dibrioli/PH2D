//! Fill (Bucket) "Fill adjust" floating-modal NodeIds — the compact draggable card that appears at
//! the ColorDrop release point so the artist can tune the flood-fill threshold LIVE and confirm/cancel
//! (Procreate ColorDrop-Threshold). Fixed ids (the modal is tool-global, one open at a time),
//! registered by `WidgetStore::open_fill_modal`. The slider + buttons forward over the frozen
//! `PanelEvent` channel to `PainterTool::{set_fill_threshold, fill_commit, fill_cancel}`.
//!
//! Split from `painter.rs` (which is at the 600-LOC file cap) — same public path via `chrome::*`.
use super::{NodeId, hash_node_id};

/// The modal's title band — the drag handle. A Primary Down here starts a modal-move (the shell tracks
/// the drag); the card must NOT close while moving.
pub const PAINTER_FILL_MODAL_HANDLE: NodeId = hash_node_id("painter.fill_modal_handle");
/// The threshold slider (`0..1` track). Its `ValueChanged` is forwarded as `PanelEvent::SetValue` →
/// `PainterTool::set_fill_threshold` (re-fills live from the pre-fill snapshot).
pub const PAINTER_FILL_MODAL_SLIDER: NodeId = hash_node_id("painter.fill_modal_slider");
/// "Done" CTA — `Click` → `PainterTool::fill_commit` (keep the fill, one undo entry).
pub const PAINTER_FILL_MODAL_DONE: NodeId = hash_node_id("painter.fill_modal_done");
/// "Cancel" — `Click` → `PainterTool::fill_cancel` (revert the layer, no undo entry).
pub const PAINTER_FILL_MODAL_CANCEL: NodeId = hash_node_id("painter.fill_modal_cancel");
