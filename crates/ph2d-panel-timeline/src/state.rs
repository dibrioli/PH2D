//! Timeline panel state + the shell→panel view snapshot.
//!
//! The authoritative timeline document lives in the shell (`AppGfx.timeline`);
//! each frame the shell publishes a [`TimelineViewSnapshot`] via
//! [`set_current_timeline`] BEFORE the panel paints, and `paint` reads it. Edits
//! flow back as `TimelineIntent`s the shell drains (mirror of the vector/motion
//! panels). Per-instance state holds only view transform (pan/zoom of the time
//! axis), which is panel-local and not undoable.

use ph2d_timeline::TimelineViewSnapshot;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Live snapshot published by the host before each `paint`. `None` until the
    /// first push (panel paints an empty timeline).
    static CURRENT_SNAPSHOT: RefCell<Option<TimelineViewSnapshot>> = const { RefCell::new(None) };
    /// Last measured scrollable content height (set by `paint`).
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus header + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Retained per-instance state for `TimelinePanel`: the horizontal view of the
/// time axis (pan + zoom). Wired in E6; `Default` satisfies the
/// `Panel::State: Default` bound.
#[derive(Clone, Debug)]
pub struct TimelinePanelState {
    /// Seconds at the left edge of the lanes area (pan).
    pub view_start_s: f64,
    /// Pixels per second (zoom). `> 0`.
    pub px_per_s: f64,
    /// Seconds currently visible across the ruler width. Written by `paint`
    /// (from the ruler pixel width ÷ zoom) and read by `event` to map a ruler
    /// scrub value `0..1` back to an absolute time.
    pub view_span_s: f64,
}

impl Default for TimelinePanelState {
    fn default() -> Self {
        Self {
            view_start_s: 0.0,
            px_per_s: DEFAULT_PX_PER_S,
            view_span_s: 0.0,
        }
    }
}

/// Default zoom: pixels per second of timeline.
pub const DEFAULT_PX_PER_S: f64 = 120.0; // LITERAL-PX-OK: default time-axis zoom (px per second), a functional view scale, not a design spacing token

/// Publish the current timeline view snapshot. Called by the shell once per
/// frame; pass `None` to clear.
pub fn set_current_timeline(snapshot: Option<TimelineViewSnapshot>) {
    CURRENT_SNAPSHOT.with(|c| *c.borrow_mut() = snapshot);
}

/// The snapshot the host published this frame, or a default empty one.
pub(crate) fn current_snapshot() -> TimelineViewSnapshot {
    CURRENT_SNAPSHOT.with(|c| c.borrow().clone().unwrap_or_default())
}

/// Last scrollable content height measured by `paint`.
#[must_use]
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(Cell::get)
}

/// Last visible body height measured by `paint`.
#[must_use]
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(Cell::get)
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}
