//! Equalize Sizes panel state.
//!
//! The panel is a thin view over the `EqualizeSizesTool` instance that
//! lives in the shell's `ToolRegistry` (unreachable from `HeroScreen`).
//! Each frame the shell publishes a [`EqualizeSizesUiSnapshot`] via
//! [`set_current_equalize_sizes_snapshot`] BEFORE the panel paints; the
//! paint reads it to seed every control's value. Edits flow back out
//! over `EditorAction::ToolPanelEvent` (see [`crate::event`]), so the
//! panel itself holds no authoritative state —
//! [`EqualizeSizesPanelState`] is an empty marker (mirrors the
//! ph2d-panel-padding shape exactly).

use ph2d_tool_equalize_sizes::params::EqualizeSizesUiSnapshot;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Live snapshot published by the host before each `paint`. `None`
    /// until the first push (panel paints defaults — every control on
    /// its default values).
    static CURRENT_SNAPSHOT: RefCell<Option<EqualizeSizesUiSnapshot>> = const { RefCell::new(None) };

    /// Last measured scrollable content height (set by `paint`, read by
    /// the orchestrator's content_h publish).
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Retained per-instance state slot for `EqualizeSizesPanel`.
/// Intentionally empty — the authoritative spec lives on the shell-side
/// `EqualizeSizesTool`; the panel renders the per-frame snapshot.
/// `Default` is required by the `Panel::State: Default` bound.
#[derive(Clone, Debug, Default)]
pub struct EqualizeSizesPanelState;

/// Publish the current snapshot. Called by the shell once per frame
/// while the `equalize_sizes` tool is active; pass `None` to clear (tool
/// inactive).
pub fn set_current_equalize_sizes_snapshot(snapshot: Option<EqualizeSizesUiSnapshot>) {
    CURRENT_SNAPSHOT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the snapshot the host published this frame, falling back to
/// [`EqualizeSizesUiSnapshot::default`] when none was pushed.
pub(crate) fn current_snapshot() -> EqualizeSizesUiSnapshot {
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
