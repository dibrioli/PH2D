//! The floating BlenderColorPicker's pre-paint widget registrations — split out of
//! [`super::pre_populate`] (HR-18 file-LOC cap) as the picker grew its palette CRUD (tabs, New/
//! Delete, Import/Export, inline rename). One cohesive `impl`-free fn: every sub-control's
//! [`crate::interaction::InteractiveState`] entry + the hex / palette-name `TextInput` links.

use crate::ids;
use crate::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use crate::widget::{ChannelMode, InterpolationMode, TextInputState};
use ph2d_tokens::ColorValue;

/// Register every BlenderColorPicker sub-control slot. Called once from
/// [`super::pre_populate::populate_shared`].
pub(crate) fn populate_blender_picker(store: &mut WidgetStore) {
    store.register(
        ids::INSP_BLENDER_PICKER,
        InteractiveState::BlenderPicker {
            value: ColorValue::from_rgba8(0, 0, 0, 255),
            channel_mode: ChannelMode::Rgb,
            interpolation: InterpolationMode::Perceptual,
            active_palette: 0,
            hsv_h: 0.0,
            hsv_s: 0.0,
            harmony: crate::widget::Harmony::None,
        },
    );
    store.init_blender_palette(
        ids::INSP_BLENDER_PICKER,
        crate::widget::default_palette().swatches.clone(),
    );
    for (id, kind) in [
        (ids::BLENDER_ADD_SWATCH, BlenderHitKind::AddSwatch),
        (ids::BLENDER_IMPORT_PALETTE, BlenderHitKind::ImportPalette),
        (ids::BLENDER_EXPORT_PALETTE, BlenderHitKind::ExportPalette),
        (
            ids::BLENDER_PALETTE_DROPDOWN,
            BlenderHitKind::PaletteDropdown,
        ),
        (ids::BLENDER_NEW_PALETTE, BlenderHitKind::NewPalette),
        (ids::BLENDER_RENAME_PALETTE, BlenderHitKind::RenamePalette),
        (ids::BLENDER_DELETE_PALETTE, BlenderHitKind::DeletePalette),
        (ids::BLENDER_CLOSE, BlenderHitKind::Close),
        (ids::BLENDER_EYEDROPPER, BlenderHitKind::Eyedropper),
        (ids::BLENDER_DRAG_HANDLE, BlenderHitKind::DragHandle),
        (ids::BLENDER_WHEEL, BlenderHitKind::Wheel),
        (ids::BLENDER_VALUE_SLIDER, BlenderHitKind::ValueSlider),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind,
            },
        );
    }
    for (id, idx) in [
        (ids::BLENDER_CHANNEL_0, 0u8),
        (ids::BLENDER_CHANNEL_1, 1),
        (ids::BLENDER_CHANNEL_2, 2),
        (ids::BLENDER_CHANNEL_3, 3),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind: BlenderHitKind::ChannelSlider(idx),
            },
        );
    }
    // Palette tab strip (named-palette select): 8 pre-registered slots, index = palette position.
    for (idx, id) in ids::BLENDER_PALETTE_TABS.into_iter().enumerate() {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind: BlenderHitKind::PaletteTab(idx as u8),
            },
        );
    }
    // Color Harmonies: the 7 scheme segments + up-to-4 partner swatches + the "add all" button.
    for (idx, id) in ids::BLENDER_HARMONY_SCHEMES.into_iter().enumerate() {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind: BlenderHitKind::HarmonyScheme(idx as u8),
            },
        );
    }
    for (idx, id) in ids::BLENDER_HARMONY_SWATCHES.into_iter().enumerate() {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind: BlenderHitKind::HarmonySwatch(idx as u8),
            },
        );
    }
    store.register(
        ids::BLENDER_HARMONY_ADD,
        InteractiveState::BlenderHit {
            parent: ids::INSP_BLENDER_PICKER,
            kind: BlenderHitKind::HarmonyAdd,
        },
    );
    for (id, idx) in [
        (ids::BLENDER_NUM_0, 0u8),
        (ids::BLENDER_NUM_1, 1),
        (ids::BLENDER_NUM_2, 2),
        (ids::BLENDER_NUM_3, 3),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::new(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_blender_channel(ids::INSP_BLENDER_PICKER, id, idx);
    }
    store.register(
        ids::BLENDER_HEX,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: "#E7E7E7FF".to_string(), // LITERAL-COLOR-OK: default value shown in the picker's hex input
            caret: 9,
            selection_anchor: None,
        },
    );
    store.link_blender_hex(ids::INSP_BLENDER_PICKER, ids::BLENDER_HEX);
    store.register(
        ids::BLENDER_PALETTE_NAME,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: "Palette".to_string(),
            caret: 7,
            selection_anchor: None,
        },
    );
    store.link_blender_palette_name(ids::INSP_BLENDER_PICKER, ids::BLENDER_PALETTE_NAME);
    for (id, kind) in [
        (
            ids::BLENDER_INTERP_LINEAR,
            BlenderHitKind::InterpolationLinear,
        ),
        (
            ids::BLENDER_INTERP_PERCEPTUAL,
            BlenderHitKind::InterpolationPerceptual,
        ),
        (ids::BLENDER_CHANNEL_RGB, BlenderHitKind::ChannelRgb),
        (ids::BLENDER_CHANNEL_HSV, BlenderHitKind::ChannelHsv),
        (ids::BLENDER_CHANNEL_OKLCH, BlenderHitKind::ChannelOklch),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind,
            },
        );
    }
    for (id, swatch_idx) in [
        (ids::BLENDER_SWATCH_0, 0u8),
        (ids::BLENDER_SWATCH_1, 1),
        (ids::BLENDER_SWATCH_2, 2),
        (ids::BLENDER_SWATCH_3, 3),
        (ids::BLENDER_SWATCH_4, 4),
        (ids::BLENDER_SWATCH_5, 5),
        (ids::BLENDER_SWATCH_6, 6),
        (ids::BLENDER_SWATCH_7, 7),
        (ids::BLENDER_SWATCH_8, 8),
        (ids::BLENDER_SWATCH_9, 9),
        (ids::BLENDER_SWATCH_10, 10),
        (ids::BLENDER_SWATCH_11, 11),
        (ids::BLENDER_SWATCH_12, 12),
        (ids::BLENDER_SWATCH_13, 13),
        (ids::BLENDER_SWATCH_14, 14),
        (ids::BLENDER_SWATCH_15, 15),
        (ids::BLENDER_SWATCH_16, 16),
        (ids::BLENDER_SWATCH_17, 17),
        (ids::BLENDER_SWATCH_18, 18),
        (ids::BLENDER_SWATCH_19, 19),
        (ids::BLENDER_SWATCH_20, 20),
        (ids::BLENDER_SWATCH_21, 21),
        (ids::BLENDER_SWATCH_22, 22),
        (ids::BLENDER_SWATCH_23, 23),
        (ids::BLENDER_SWATCH_24, 24),
        (ids::BLENDER_SWATCH_25, 25),
        (ids::BLENDER_SWATCH_26, 26),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind: BlenderHitKind::PaletteSwatch(swatch_idx),
            },
        );
    }
}
