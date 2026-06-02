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
    /// `true` while the `vector_shape` tool is active — the bridge publishes it
    /// so `paint` knows to add the Shape-kind row (shape-specific, so hidden for
    /// Select/Direct).
    static SHAPE_ACTIVE: Cell<bool> = const { Cell::new(false) };
    /// Index (0..5) of the active tool's current `ShapeKind` — drives which
    /// option paints selected. Published by the bridge each frame.
    static SHAPE_INDEX: Cell<u8> = const { Cell::new(0) };
    /// A pending option index set by `apply_event` on a Shape-option `Click`;
    /// the bridge drains it (`take_pending_shape_selection`) and forwards it to
    /// `VectorShapeTool::set_kind`. `None` ⇒ no pending change.
    static PENDING_SHAPE: Cell<Option<u8>> = const { Cell::new(None) };
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

/// Shell publishes whether the Shape tool is active + its current kind index
/// (0..5, `ShapeKind::ALL` order) before paint.
pub fn set_current_shape(active: bool, kind_index: u8) {
    SHAPE_ACTIVE.with(|c| c.set(active));
    SHAPE_INDEX.with(|c| c.set(kind_index));
}

pub(crate) fn shape_active() -> bool {
    SHAPE_ACTIVE.with(|c| c.get())
}

pub(crate) fn shape_index() -> u8 {
    SHAPE_INDEX.with(|c| c.get())
}

/// `apply_event` records a clicked Shape option here; the bridge drains it.
pub(crate) fn set_pending_shape(index: u8) {
    PENDING_SHAPE.with(|c| c.set(Some(index)));
}

/// Shell drains a pending Shape-option selection (clears it). `None` ⇒ none.
pub fn take_pending_shape_selection() -> Option<u8> {
    PENDING_SHAPE.with(|c| c.take())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_shape_round_trips_and_drains() {
        // `take` is single-shot: the bridge drains exactly one pick per frame.
        assert_eq!(take_pending_shape_selection(), None);
        set_pending_shape(3);
        assert_eq!(take_pending_shape_selection(), Some(3));
        assert_eq!(
            take_pending_shape_selection(),
            None,
            "take clears the pending"
        );
    }

    #[test]
    fn shape_publish_round_trips() {
        set_current_shape(true, 2);
        assert!(shape_active());
        assert_eq!(shape_index(), 2);
        set_current_shape(false, 0);
        assert!(!shape_active());
    }
}
