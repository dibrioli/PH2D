//! Color Equalization panel state.
//!
//! The panel is a thin view over the `ColorEqualizationTool` instance in
//! the shell's `ToolRegistry` (unreachable from `HeroScreen`). Each frame
//! the shell publishes a [`ColorEqualizationUiSnapshot`] via
//! [`set_current_snapshot`] BEFORE the panel paints; the paint reads it to
//! seed the slider tracks + chip values. Edits flow back out over
//! `EditorAction::ToolPanelEvent` (see [`crate::event`]); the panel itself
//! holds no authoritative state — `ColorEqualizationPanelState` is an
//! empty marker.

use ph2d_tool_color_equalization::params::ColorEqualizationUiSnapshot;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Live snapshot published by the host before each `paint`. `None`
    /// until the first push (panel paints `Default` — identity params).
    static CURRENT_SNAPSHOT: RefCell<Option<ColorEqualizationUiSnapshot>> = const { RefCell::new(None) };

    /// Last measured scrollable content height (set by `paint`, read by
    /// the orchestrator's content_h publish). Kept for parity with the
    /// other panels even though the Color EQ body fits without scroll.
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Publish the current snapshot. Called by the shell once per frame while
/// the `color_equalization` tool is active; pass `None` to clear (tool
/// inactive).
pub fn set_current_snapshot(snapshot: Option<ColorEqualizationUiSnapshot>) {
    CURRENT_SNAPSHOT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the snapshot the host published this frame, falling back to
/// `ColorEqualizationUiSnapshot::default` (identity params) when none was
/// pushed.
pub(crate) fn current_snapshot() -> ColorEqualizationUiSnapshot {
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
