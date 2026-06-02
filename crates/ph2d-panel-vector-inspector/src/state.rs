//! Vector inspector panel state + the shell→panel fill-color snapshot.
//!
//! Per-instance state is empty (ADR-0029 §4.3, mirror of BgRemoval): the
//! canonical fill color lives on the shell `App.vector_fill_color`. The shell
//! publishes it each frame via [`set_current_fill`]; `paint` reads it to draw
//! the swatch fill.

use std::cell::Cell;

thread_local! {
    /// The current fill color (sRGB8) the shell published this frame — drawn as
    /// the swatch fill. Default mid-grey until the shell publishes.
    static CURRENT_FILL: Cell<[u8; 4]> = const { Cell::new([0x88, 0x88, 0x88, 0xFF]) };
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// State per-instance retained by the panel. Intentionally empty — canonical
/// state is the shell-owned `App.vector_fill_color`.
#[derive(Clone, Debug, Default)]
pub struct VectorInspectorPanelState;

/// Shell publishes the current fill color (sRGB8) before paint.
pub fn set_current_fill(rgba: [u8; 4]) {
    CURRENT_FILL.with(|c| c.set(rgba));
}

pub(crate) fn current_fill() -> [u8; 4] {
    CURRENT_FILL.with(|c| c.get())
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
