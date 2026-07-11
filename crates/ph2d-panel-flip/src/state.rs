//! Flip Style panel state + the shell→panel snapshots.
//!
//! Per-instance state is empty (mirror of Vector/Padding): the authoritative
//! Style lives on the shell-side `FlipTool` and the layer model lives on the
//! shell's `FlipDoc`. Each frame the shell publishes a [`FlipStyleSnapshot`]
//! (via [`set_current_flip_style`]) + a [`FlipLayersSnapshot`] (via
//! [`set_current_flip_layers`]) BEFORE the panel paints; `paint` reads them.
//! Edits flow back out over `EditorAction::ToolPanelEvent`.

use ph2d_tool_flip::FlipStyleSnapshot;
use std::cell::{Cell, RefCell};

/// A single layer as the panel paints it — a panel-local mirror of the shell's
/// `FlipLayer` (kept panel-local so the panel needn't depend on the doc model).
/// Published in the object's z-order (index 0 = back/bottom); the panel paints
/// the list TOP-first (reversed), like every layers panel.
#[derive(Clone, Debug, PartialEq)]
pub struct FlipLayerRow {
    /// `LayerId.0` — the stable id woven into the per-row widget ids.
    pub id: u64,
    /// Display name.
    pub name: String,
    /// Blend-mode wire discriminant (`ph2d_painter_effects::BlendMode as u8`).
    pub blend: u8,
    /// Opacity `0..1`.
    pub opacity: f32,
    /// The eye.
    pub visible: bool,
    /// The padlock.
    pub locked: bool,
}

/// The layer list the panel paints, published by the shell each frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlipLayersSnapshot {
    /// Layers in the object's z-order (index 0 = back). Empty ⇒ no object / no
    /// layers → the Layers section shows only the Add button.
    pub rows: Vec<FlipLayerRow>,
    /// The active layer's id (`LayerId.0`) — highlighted + the target of Delete
    /// / the drawn stroke. `None` ⇒ nothing active.
    pub active: Option<u64>,
}

thread_local! {
    /// Live style snapshot published by the host before each `paint`. `None`
    /// until the first push (panel paints defaults).
    static CURRENT_STYLE: RefCell<Option<FlipStyleSnapshot>> = const { RefCell::new(None) };
    /// Live layer list published by the host before each `paint`.
    static CURRENT_LAYERS: RefCell<FlipLayersSnapshot> =
        const { RefCell::new(FlipLayersSnapshot { rows: Vec::new(), active: None }) };
    /// Last measured scrollable content height (set by `paint`).
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Retained per-instance state slot for `FlipPanel`. Intentionally empty — the
/// authoritative Style lives on the shell-side `FlipTool`, the layer model on
/// the shell's `FlipDoc`. `Default` is required by `Panel::State: Default`.
#[derive(Clone, Debug, Default)]
pub struct FlipPanelState;

/// Publish the current Style snapshot. Called by the shell once per frame while
/// the `flip` tool is active; pass `None` to clear (tool inactive).
pub fn set_current_flip_style(snapshot: Option<FlipStyleSnapshot>) {
    CURRENT_STYLE.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the snapshot the host published this frame, falling back to
/// [`FlipStyleSnapshot::default`] when none was pushed.
pub(crate) fn current_style() -> FlipStyleSnapshot {
    CURRENT_STYLE.with(|c| c.borrow().unwrap_or_default())
}

/// Publish the layer list. Called by the shell once per frame while the `flip`
/// tool is active (empty snapshot when there is no active object).
pub fn set_current_flip_layers(layers: FlipLayersSnapshot) {
    CURRENT_LAYERS.with(|c| *c.borrow_mut() = layers);
}

/// The layer list this frame (cloned — a handful of layers, per-frame; fine).
pub(crate) fn current_layers() -> FlipLayersSnapshot {
    CURRENT_LAYERS.with(|c| c.borrow().clone())
}

#[must_use]
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(Cell::get)
}

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
