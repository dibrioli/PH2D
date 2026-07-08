//! Vector Style panel state + the shell→panel Style snapshot.
//!
//! Per-instance state is empty (ADR-0029 §4.3, mirror of Padding): the
//! authoritative Style lives on the shell-side `VectorTool`. Each frame the
//! shell publishes a [`VectorStyleSnapshot`] via [`set_current_vector_style`]
//! BEFORE the panel paints; `paint` reads it to seed the Width chip + the two
//! colour swatches. Edits flow back out over `EditorAction::ToolPanelEvent`
//! (Width slider, Fill-None) and the colour-picker read-back (Stroke / Fill
//! swatches), so the panel holds no authoritative state.

use ph2d_tool_vector::{VectorStyleSnapshot, VertexType};
use std::cell::{Cell, RefCell};

thread_local! {
    /// Live snapshot published by the host before each `paint`. `None` until
    /// the first push (panel paints defaults).
    static CURRENT_SNAPSHOT: RefCell<Option<VectorStyleSnapshot>> = const { RefCell::new(None) };
    /// Type of the currently-selected vertex (published by the shell each frame
    /// from the Pen). `None` = no vertex selected → the Vertex section hides.
    static CURRENT_VERTEX_TYPE: RefCell<Option<VertexType>> = const { RefCell::new(None) };
    /// Selected path's anchor bbox `[x, y, w, h]` (world), published each frame.
    /// `None` = no path selected → the Transform section hides.
    static CURRENT_TRANSFORM: Cell<Option<[f64; 4]>> = const { Cell::new(None) };
    /// Rotation-field accumulator: the angle (degrees) the Angle chip last
    /// reported THIS gesture. `event` emits the DELTA `(current − this)` so the
    /// shell rotates incrementally; reset to 0 by `paint` whenever the field is
    /// unfocused (gesture ended), so the shell stays stateless.
    static ROT_LAST: Cell<f64> = const { Cell::new(0.0) };
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

/// Publish the selected vertex's type (or `None` when no vertex is selected).
/// Called by the shell each frame while the `vector` tool is active.
pub fn set_selected_vertex_type(kind: Option<VertexType>) {
    CURRENT_VERTEX_TYPE.with(|c| *c.borrow_mut() = kind);
}

/// The selected vertex's type this frame (`None` ⇒ hide the Vertex section).
pub(crate) fn current_vertex_type() -> Option<VertexType> {
    CURRENT_VERTEX_TYPE.with(|c| *c.borrow())
}

/// Publish the selected path's anchor bbox `[x, y, w, h]` (world), or `None`.
/// Called by the shell each frame while the `vector` tool is active.
pub fn set_current_transform(bbox: Option<[f64; 4]>) {
    CURRENT_TRANSFORM.with(|c| c.set(bbox));
}

/// The selected path's bbox this frame (`None` ⇒ hide the Transform section).
pub(crate) fn current_transform() -> Option<[f64; 4]> {
    CURRENT_TRANSFORM.with(|c| c.get())
}

/// The angle the Angle chip last reported this gesture (for the delta emit).
pub(crate) fn rot_last() -> f64 {
    ROT_LAST.with(Cell::get)
}

/// Record the Angle chip's current value as the gesture baseline (or reset to 0
/// between gestures).
pub(crate) fn set_rot_last(v: f64) {
    ROT_LAST.with(|c| c.set(v));
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
