//! Brush Studio panel state — mirrors the sidebar's snapshot plumbing.
//!
//! `BrushStudioPanelState` is empty: the canonical `Brush` lives on the
//! shell-side `PainterTool`. The shell publishes a [`BrushStudioSnapshot`] via
//! [`set_current_brush_studio_snapshot`] BEFORE `paint`; the paint reads it and
//! renders the sections. Events go out via `EditorAction::ToolPanelEvent`
//! (ADR-0040 TG-B) — the shell routes them to `PainterTool::handle_panel_event`
//! which dispatches `PainterUiEdit::SetBrushParam`.

use ph2d_tool_painter::BrushStudioSnapshot;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Snapshot published by the host before each `paint`. `None` until the
    /// first push (the panel paints `BrushStudioSnapshot::default()` — Round Hard).
    static CURRENT_SNAPSHOT: RefCell<Option<BrushStudioSnapshot>> = const { RefCell::new(None) };

    /// Last measured scrollable content height (set by paint, read by the
    /// orchestrator's content_h publish — parity with sidebar / layers).
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus header + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Per-instance retained state of the `BrushStudioPanel`. Empty intentionally —
/// brush params live on the shell-side `PainterTool`; the panel renders a
/// per-frame snapshot. `Default` required by the `Panel::State: Default` bound.
#[derive(Clone, Debug, Default)]
pub struct BrushStudioPanelState;

/// Publish the current Brush Studio snapshot. Called by the shell once per
/// frame while the `painter` tool is active; pass `None` to clear.
pub fn set_current_brush_studio_snapshot(snapshot: Option<BrushStudioSnapshot>) {
    CURRENT_SNAPSHOT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the snapshot published this frame, falling back to the Round-Hard
/// default before the host's first push.
pub(crate) fn current_snapshot() -> BrushStudioSnapshot {
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
