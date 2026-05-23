//! Upscale panel state.
//!
//! The panel is a thin view over the `UpscaleTool` instance that lives
//! in the shell's `ToolRegistry` (unreachable from `HeroScreen`). Each
//! frame the shell publishes an [`UpscaleUiSnapshot`] via
//! [`set_current_upscale_snapshot`] BEFORE the panel paints; the paint
//! reads it to seed the algorithm-selected segment + the scale slider
//! value + the output-size readout. Edits flow back out over
//! `EditorAction::ToolPanelEvent` (see [`crate::event`]), so the panel
//! itself holds no authoritative state — [`UpscalePanelState`] is an
//! empty marker.
//!
//! The live preview RGBA is NOT painted inside the panel — it renders
//! shell-side as an on-canvas overlay (mirrors `ph2d-panel-bgremoval`,
//! which keeps its preview thumbnail buffer in the tool crate but
//! draws no image into the panel). The panel surfaces the active
//! algorithm and the projected scale factor via the algorithm
//! segmented button highlight + the slider chip's `1.50×` readout —
//! enough to read the operation at a glance.
//!
//! [`UpscaleUiSnapshot`]: ph2d_tool_upscale::params::UpscaleUiSnapshot

use ph2d_tool_upscale::params::UpscaleUiSnapshot;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Live snapshot published by the host before each `paint`. `None`
    /// until the first push (panel paints defaults — Lanczos3 @ 2×).
    static CURRENT_SNAPSHOT: RefCell<Option<UpscaleUiSnapshot>> = const { RefCell::new(None) };

    /// Last measured scrollable content height (set by `paint`, read by
    /// the orchestrator's content_h publish). Kept for parity with the
    /// other panels even though the Upscale body rarely overflows.
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Retained per-instance state slot for `UpscalePanel`. Intentionally
/// empty — the authoritative params live on the shell-side
/// `UpscaleTool`; the panel renders the per-frame snapshot. `Default`
/// is required by the `Panel::State: Default` bound.
#[derive(Clone, Debug, Default)]
pub struct UpscalePanelState;

/// Publish the current normalized param snapshot. Called by the shell
/// once per frame while the `upscale` tool is active; pass `None` to
/// clear (tool inactive).
pub fn set_current_upscale_snapshot(snapshot: Option<UpscaleUiSnapshot>) {
    CURRENT_SNAPSHOT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the snapshot the host published this frame, falling back to
/// [`UpscaleUiSnapshot::default`] when the host hasn't pushed yet.
pub(crate) fn current_snapshot() -> UpscaleUiSnapshot {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_round_trip_through_thread_local() {
        let s = UpscaleUiSnapshot {
            scale_factor: 3.0,
            ..UpscaleUiSnapshot::default()
        };
        set_current_upscale_snapshot(Some(s));
        let got = current_snapshot();
        assert!((got.scale_factor - 3.0).abs() < f32::EPSILON);
        set_current_upscale_snapshot(None);
        // Falls back to default.
        let got = current_snapshot();
        assert!((got.scale_factor - 2.0).abs() < f32::EPSILON);
    }
}
