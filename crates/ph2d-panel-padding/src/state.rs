//! Padding panel state.
//!
//! The panel is a thin view over the `PaddingTool` instance that lives
//! in the shell's `ToolRegistry` (unreachable from `HeroScreen`). Each
//! frame the shell publishes a [`PaddingUiSnapshot`] via
//! [`set_current_padding_snapshot`] BEFORE the panel paints; the paint
//! reads it to seed the four field values. Edits flow back out over
//! `EditorAction::PaddingUiEdit` (see [`crate::event`]), so the panel
//! itself holds no authoritative state — [`PaddingPanelState`] is an
//! empty marker.

use ph2d_editor_core::tools::padding::PaddingUiSnapshot;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Live snapshot published by the host before each `paint`. `None`
    /// until the first push (panel paints defaults — all edges 0).
    static CURRENT_SNAPSHOT: RefCell<Option<PaddingUiSnapshot>> = const { RefCell::new(None) };

    /// Last measured scrollable content height (set by `paint`, read by
    /// the orchestrator's content_h publish). Kept for parity with the
    /// other panels even though the Padding body never overflows.
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Retained per-instance state slot for `PaddingPanel`. Intentionally
/// empty — the authoritative spec lives on the shell-side `PaddingTool`;
/// the panel renders the per-frame snapshot. `Default` is required by
/// the `Panel::State: Default` bound.
#[derive(Clone, Debug, Default)]
pub struct PaddingPanelState;

/// Publish the current per-edge snapshot. Called by the shell once per
/// frame while the `padding` tool is active; pass `None` to clear (tool
/// inactive).
pub fn set_current_padding_snapshot(snapshot: Option<PaddingUiSnapshot>) {
    CURRENT_SNAPSHOT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the snapshot the host published this frame, falling back to
/// [`PaddingUiSnapshot::default`] (all edges 0) when none was pushed.
pub(crate) fn current_snapshot() -> PaddingUiSnapshot {
    CURRENT_SNAPSHOT.with(|c| c.borrow().unwrap_or_default())
}

pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}
