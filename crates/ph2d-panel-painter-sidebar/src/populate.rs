//! Painter sidebar `populate` — pre-registers widget IDs no `WidgetStore`
//! ao boot do host (uma vez via `Panel::populate`).
//!
//! Initial values são placeholders; host overwrites cada frame do live
//! `PainterUiSnapshot`. Pattern espelha BgRemoval/Padding.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

pub fn populate(store: &mut WidgetStore) {
    // Buttons: undo/redo + modifier square (tappable).
    for id in [ids::UNDO_BUTTON, ids::REDO_BUTTON, ids::MODIFIER_SQUARE] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Sliders + chips pair (size + opacity). Initial values seed do
    // PainterParams::default(): size_px=32 → 0.0151, opacity=1.0 → 1.0.
    let size01 = (32.0_f32 - 1.0) / (2048.0 - 1.0);
    register_slider_chip_pair(store, ids::SIZE_SLIDER, ids::SIZE_CHIP, size01, 32.0);
    register_slider_chip_pair(store, ids::OPACITY_SLIDER, ids::OPACITY_CHIP, 1.0, 100.0);
    // Link slider ↔ chip (DIRETRIZ v7.0 §5.2 regra 1).
    store.link_slider_number(ids::SIZE_SLIDER, ids::SIZE_CHIP);
    store.link_slider_number(ids::OPACITY_SLIDER, ids::OPACITY_CHIP);
}

fn register_slider_chip_pair(
    store: &mut WidgetStore,
    slider_id: NodeId,
    chip_id: NodeId,
    slider_value: f32,
    chip_display_value: f64,
) {
    store.register(
        slider_id,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: slider_value,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        chip_id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: chip_display_value,
            buffer: format!("{chip_display_value:.3}"),
            caret: 0,
            last_committed: chip_display_value,
            selection_anchor: None,
        },
    );
}
