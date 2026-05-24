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

use ph2d_editor_core::zones::Rect;
use ph2d_tool_color_equalization::algorithm::HistogramData;
use ph2d_tool_color_equalization::params::ColorEqualizationUiSnapshot;
use std::cell::{Cell, RefCell};

/// A dropdown chip that needs its popover painted AFTER the scroll
/// clip is popped (so the popover lays on top of every later section).
/// Set by `paint_dropdown_chip` mid-body; drained by the end of paint.
/// `slot` distinguishes the four dropdowns:
///   1 = LUT 1, 2 = LUT 2, 3 = Posterize, 4 = Quantize.
#[derive(Copy, Clone, Debug)]
pub(crate) struct PendingDropdownPopover {
    pub slot: u8,
    pub chip: Rect,
}

thread_local! {
    /// Live snapshot published by the host before each `paint`. `None`
    /// until the first push (panel paints `Default` — identity params).
    static CURRENT_SNAPSHOT: RefCell<Option<ColorEqualizationUiSnapshot>> = const { RefCell::new(None) };

    /// Live histogram of the current preview (Phase 2). `None` until the
    /// host publishes one — the panel overlay then falls back to drawing
    /// the chrome frame without bin bars. The shell computes via
    /// `ColorEqualizationTool::preview_histogram()` and pushes here once
    /// per frame.
    static CURRENT_HISTOGRAM: RefCell<Option<HistogramData>> = const { RefCell::new(None) };

    /// Last measured scrollable content height (set by `paint`, read by
    /// the orchestrator's content_h publish). Kept for parity with the
    /// other panels even though the Color EQ body fits without scroll.
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };

    /// Pending dropdown popover paints (LUT 1/2 + Posterize + Quantize).
    /// Set inside the scrollable body by `paint_dropdown_chip` when the
    /// dropdown is open; drained AFTER `pop_layer` so the popover floats
    /// above every later panel section (mirror of `paint_inspector`'s
    /// `take_pending_dropdown_chip`).
    static PENDING_POPOVERS: RefCell<Vec<PendingDropdownPopover>> = const { RefCell::new(Vec::new()) };
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

/// Publish the current preview histogram (Phase 2). Called by the shell
/// once per frame; pass `None` to clear (no preview / tool inactive).
pub fn set_current_histogram(hist: Option<HistogramData>) {
    CURRENT_HISTOGRAM.with(|c| *c.borrow_mut() = hist);
}

/// Borrow the current histogram for read-only paint. Returns `None`
/// (empty overlay frame painted) when the host hasn't published one.
pub(crate) fn with_current_histogram<R>(f: impl FnOnce(Option<&HistogramData>) -> R) -> R {
    CURRENT_HISTOGRAM.with(|c| f(c.borrow().as_ref()))
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

pub(crate) fn push_pending_popover(p: PendingDropdownPopover) {
    PENDING_POPOVERS.with(|c| c.borrow_mut().push(p));
}

pub(crate) fn take_pending_popovers() -> Vec<PendingDropdownPopover> {
    PENDING_POPOVERS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}
