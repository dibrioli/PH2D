//! Painter layers `populate` — pre-registers the panel's **fixed-id** widget
//! slots in the `WidgetStore` at host boot (once via `Panel::populate`).
//!
//! Per-row widgets (eye toggle, opacity slider+chip, blend chip, row select)
//! have *dynamic* ids derived from the live layer ids, so they are registered
//! in `paint` via `register_if_absent` (the panel owns `store_mut` there) —
//! they can't be known at boot. Only the chrome buttons live here.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, DropdownState, SliderOrientation, SliderState};

pub fn populate(store: &mut WidgetStore) {
    let buttons = [
        ph2d_editor_core::ids::PAINTER_LAYERS_CLOSE,
        // Action toolbar (below the header): New layer / Group / Duplicate /
        // Delete. MUST be registered here or the dispatcher drops the click
        // (paint + hit_index alone is not enough — feedback-panel-populate-register).
        ph2d_editor_core::ids::PAINTER_LAYERS_ADD,
        ph2d_editor_core::ids::PAINTER_LAYERS_GROUP,
        ph2d_editor_core::ids::PAINTER_LAYERS_DUPLICATE,
        ph2d_editor_core::ids::PAINTER_LAYERS_DELETE,
        // Modifier toolbar (acts on the active layer): Mask / Clip / Lock / Ref.
        ph2d_editor_core::ids::PAINTER_LAYERS_MASK,
        ph2d_editor_core::ids::PAINTER_LAYERS_CLIP,
        ph2d_editor_core::ids::PAINTER_LAYERS_ALPHA_LOCK,
        ph2d_editor_core::ids::PAINTER_LAYERS_REFERENCE,
        // Dock-mode toggle ("Brush") in the header (mode C).
        ph2d_editor_core::ids::PAINTER_LAYERS_TOGGLE_DOCK,
        // "Apply" CTA — commits the composite to the sprite (shared id with the
        // sidebar panel).
        ph2d_editor_core::ids::PAINTER_APPLY,
    ];
    for id in buttons {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // "+ Adj" is a Dropdown, not a plain Button (W4 T4.15): clicking it toggles
    // the 24-kind picker popover open/closed via the generic Dropdown dispatch
    // (which emits no Click — the panel observes `open` via the store), exactly
    // like the per-row blend chip. Painted as an icon button; the open popover
    // is a deferred pass in `paint`.
    store.register(
        ph2d_editor_core::ids::PAINTER_LAYERS_ADD_ADJUSTMENT,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );

    // Brush-properties view (fixed-id, so registered here, not per-frame): the
    // Size slider. The panel paints it off the published `BrushSettings`
    // snapshot; the value here is a placeholder overwritten by the drag / snapshot.
    store.register(
        ph2d_editor_core::ids::PAINTER_BRUSH_SIZE_SLIDER,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.0,
            orientation: SliderOrientation::Horizontal,
        },
    );
    // Colour swatch — a Button; clicking it toggles the shared Blender picker.
    // MUST be registered here or the dispatcher drops the click.
    store.register(
        ph2d_editor_core::ids::PAINTER_COLOR_THUMB,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // Blend chip is a Dropdown (generic open/close dispatch, like the per-row
    // blend chip + the "+ Adj" picker); its 24 options are deferred-popover buttons.
    store.register(
        ph2d_editor_core::ids::PAINTER_BRUSH_BLEND,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(0),
        },
    );
}
