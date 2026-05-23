//! Background-Removal panel state.
//!
//! The panel is a thin view over the `BgRemovalTool` instance that lives
//! in the shell's `ToolRegistry` (unreachable from `HeroScreen`). Each
//! frame the shell publishes a normalized [`BgRemovalUiSnapshot`] via
//! [`set_current_bgremoval_snapshot`] BEFORE the panel paints; the paint
//! reads it to position the sliders + highlight the active mode. Panel
//! events flow back out over `EditorAction::ToolPanelEvent` (see
//! [`crate::event`]), so the panel itself holds no authoritative param
//! state — [`BgRemovalPanelState`] is an empty marker.

use ph2d_tool_bgremoval::params::BgRemovalUiSnapshot;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Live normalized snapshot published by the host before each
    /// `paint`. `None` until the first push (panel paints defaults).
    static CURRENT_SNAPSHOT: RefCell<Option<BgRemovalUiSnapshot>> = const { RefCell::new(None) };

    /// Last measured scrollable content height (set by `paint`, read by
    /// the orchestrator's content_h publish). Kept for parity with the
    /// other panels even though the Bg Removal body rarely overflows.
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Retained per-instance state slot for `BgRemovalPanel`. Intentionally
/// empty — the authoritative params live on the shell-side
/// `BgRemovalTool`; the panel renders the per-frame snapshot. `Default`
/// is required by the `Panel::State: Default` bound.
#[derive(Clone, Debug, Default)]
pub struct BgRemovalPanelState;

/// Publish the current normalized param snapshot. Called by the shell
/// once per frame while the `bgremoval` tool is active; pass `None` to
/// clear (tool inactive).
pub fn set_current_bgremoval_snapshot(snapshot: Option<BgRemovalUiSnapshot>) {
    CURRENT_SNAPSHOT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the snapshot the host published this frame, falling back to
/// [`BgRemovalUiSnapshot::default`] when the host hasn't pushed yet.
pub(crate) fn current_snapshot() -> BgRemovalUiSnapshot {
    CURRENT_SNAPSHOT.with(|c| c.borrow().clone().unwrap_or_default())
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
