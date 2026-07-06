//! Vector Style panel state + the shell→panel Style snapshot.
//!
//! Per-instance state is empty (ADR-0029 §4.3, mirror of Padding): the
//! authoritative Style lives on the shell-side `VectorTool`. Each frame the
//! shell publishes a [`VectorStyleSnapshot`] via [`set_current_vector_style`]
//! BEFORE the panel paints; `paint` reads it to seed the Width chip + the two
//! colour swatches. Edits flow back out over `EditorAction::ToolPanelEvent`
//! (Width slider, Fill-None) and the colour-picker read-back (Stroke / Fill
//! swatches), so the panel holds no authoritative state.

use ph2d_tool_vector::VectorStyleSnapshot;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Live snapshot published by the host before each `paint`. `None` until
    /// the first push (panel paints defaults).
    static CURRENT_SNAPSHOT: RefCell<Option<VectorStyleSnapshot>> = const { RefCell::new(None) };
    /// Last measured scrollable content height (set by `paint`).
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Retained per-instance state slot for `VectorPanel`. Intentionally empty —
/// the authoritative Style lives on the shell-side `VectorTool`; the panel
/// renders the per-frame snapshot. `Default` is required by the
/// `Panel::State: Default` bound.
#[derive(Clone, Debug, Default)]
pub struct VectorPanelState;

/// Publish the current Style snapshot. Called by the shell once per frame while
/// the `vector` tool is active; pass `None` to clear (tool inactive).
pub fn set_current_vector_style(snapshot: Option<VectorStyleSnapshot>) {
    CURRENT_SNAPSHOT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the snapshot the host published this frame, falling back to
/// [`VectorStyleSnapshot::default`] when none was pushed.
pub(crate) fn current_snapshot() -> VectorStyleSnapshot {
    CURRENT_SNAPSHOT.with(|c| c.borrow().unwrap_or_default())
}

#[must_use]
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

#[must_use]
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}
