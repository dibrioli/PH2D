//! Painter sidebar `populate` — pre-registers widget IDs no `WidgetStore`
//! ao boot do host (uma vez via `Panel::populate`).
//!
//! Initial values são placeholders; host overwrites cada frame do live
//! `PainterUiSnapshot` (paint lê snapshot pra track position; valor
//! stored é o que dispatch muta no drag e o que event.rs lê em
//! `ValueChanged`).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState};

pub fn populate(store: &mut WidgetStore) {
    // Buttons: undo/redo + modifier square (modifier square é tappable).
    for id in [ids::UNDO_BUTTON, ids::REDO_BUTTON, ids::MODIFIER_SQUARE] {
        store.set_button(id, ButtonState::default());
    }

    // Sliders: size + opacity (ambos normalizados 0..1; display via
    // chip override pra px / %).
    for id in [ids::SIZE_SLIDER, ids::OPACITY_SLIDER] {
        store.set_slider(
            id,
            SliderState {
                orientation: SliderOrientation::Horizontal,
                value: 0.5,
                state: InteractiveState::default(),
            },
        );
    }
    // Chips numéricos via `link_slider_number` (DIRETRIZ v7.0 §5.2 regra 1)
    // ficam para T2.1 Day-7 functional wire — depende de chip API canon
    // confirmada pós-Inspector/BgRemoval.
}
