//! Deform-section widget registration (Deform Wave 1) — split from [`crate::populate`] for the panel
//! file-LOC cap. Registers the 5 brush sliders (with linked editable chips) + the mode segmented + the
//! Freeze/Invert toggles + the Reset/Apply/Apply&Keep action buttons, so panel-wiring-parity + the
//! dispatch see them (mirror of the Selection block in `populate`).

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

pub(crate) fn register_deform_widgets(store: &mut WidgetStore) {
    for id in [
        ph2d_editor_core::ids::PAINTER_DEFORM_SIZE_SLIDER,
        ph2d_editor_core::ids::PAINTER_DEFORM_PRESSURE_SLIDER,
        ph2d_editor_core::ids::PAINTER_DEFORM_DISTORTION_SLIDER,
        ph2d_editor_core::ids::PAINTER_DEFORM_MOMENTUM_SLIDER,
        ph2d_editor_core::ids::PAINTER_DEFORM_STRENGTH_SLIDER,
    ] {
        store.register(
            id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
    }
    for (slider, chip) in [
        (
            ph2d_editor_core::ids::PAINTER_DEFORM_SIZE_SLIDER,
            ph2d_editor_core::ids::PAINTER_DEFORM_SIZE_CHIP,
        ),
        (
            ph2d_editor_core::ids::PAINTER_DEFORM_PRESSURE_SLIDER,
            ph2d_editor_core::ids::PAINTER_DEFORM_PRESSURE_CHIP,
        ),
        (
            ph2d_editor_core::ids::PAINTER_DEFORM_DISTORTION_SLIDER,
            ph2d_editor_core::ids::PAINTER_DEFORM_DISTORTION_CHIP,
        ),
        (
            ph2d_editor_core::ids::PAINTER_DEFORM_MOMENTUM_SLIDER,
            ph2d_editor_core::ids::PAINTER_DEFORM_MOMENTUM_CHIP,
        ),
        (
            ph2d_editor_core::ids::PAINTER_DEFORM_STRENGTH_SLIDER,
            ph2d_editor_core::ids::PAINTER_DEFORM_STRENGTH_CHIP,
        ),
    ] {
        store.register(
            chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::new(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number(slider, chip);
        store.set_number_range(chip, 0.0, 1.0, 0.01); // LITERAL-PX-OK: chip 0..1 track step (behaviour value)
    }
    for id in ph2d_editor_core::ids::PAINTER_DEFORM_MODE_IDS
        .iter()
        .chain(ph2d_editor_core::ids::PAINTER_DEFORM_ACTION_IDS.iter())
        .chain(std::iter::once(
            &ph2d_editor_core::ids::PAINTER_DEFORM_FREEZE,
        ))
        .chain(std::iter::once(
            &ph2d_editor_core::ids::PAINTER_DEFORM_FREEZE_INVERT,
        ))
        .copied()
    {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}
