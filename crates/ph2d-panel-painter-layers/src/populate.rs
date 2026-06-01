//! Painter layers `populate` — pre-registers the panel's **fixed-id** widget
//! slots in the `WidgetStore` at host boot (once via `Panel::populate`).
//!
//! Per-row widgets (eye toggle, opacity slider+chip, blend chip, row select)
//! have *dynamic* ids derived from the live layer ids, so they are registered
//! in `paint` via `register_if_absent` (the panel owns `store_mut` there) —
//! they can't be known at boot. Only the chrome buttons live here.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

pub fn populate(store: &mut WidgetStore) {
    let buttons = [
        ph2d_editor_core::ids::PAINTER_LAYERS_CLOSE,
        // "+ Layer" footer button.
        ph2d_editor_core::ids::PAINTER_LAYERS_ADD,
        // Dock-mode toggle ("Brush") in the header (mode C).
        ph2d_editor_core::ids::PAINTER_LAYERS_TOGGLE_DOCK,
    ];
    for id in buttons {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}
