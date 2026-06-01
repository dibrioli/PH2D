//! Painter layers `populate` — pre-registers widget IDs no `WidgetStore`
//! ao boot do host (uma vez via `Panel::populate`). Mirror do sidebar.
//!
//! **SCAFFOLD:** só o close (X) button é registrado (é o único widget
//! pintado pelo chrome). O Implementador registra aqui os widgets das
//! layer rows (visibility toggle, opacity slider+chip, blend dropdown,
//! row-select) quando preencher `paint`. Y-7 audit: registrar sem paint
//! deixa roteamento morto — registre só o que `paint` pinta.

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::interaction::InteractiveState;

pub fn populate(store: &mut WidgetStore) {
    // Close (X) button.
    store.register(
        ph2d_editor_core::ids::PAINTER_LAYERS_CLOSE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );

    // TODO(impl W3.T3.4): register per-layer-row widgets (visibility
    // toggle Button, opacity Slider + NumberInput chip with the affine
    // link, blend-mode dropdown Button) — one set per visible row, mirror
    // do register_slider_chip_pair do sidebar.
}
