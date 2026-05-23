//! Color Equalization panel `populate` — pre-registers the panel's widget
//! slots in the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Mirrors the canonical "slider Speed" setup from the widget gallery
//! (`pre_populate.rs`): chip + slider share the same `0..1` value space,
//! `link_slider_number` registers the bidirectional mirror, and the
//! dispatch handles drag / clamp / sync automatically. Natural-unit
//! formatting ("2.00" clip limit, "+0.30" brightness etc.) is paint-only
//! via `display_override` and lives outside the storage.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_color_equalization::params::{
    BRIGHTNESS_DEFAULT, CLIP_LIMIT_DEFAULT, CONTRAST_DEFAULT, SATURATION_DEFAULT,
    TILE_GRID_DEFAULT, brightness_to_slider, clip_limit_to_slider, contrast_to_slider,
    saturation_to_slider, tile_grid_to_slider,
};

pub fn populate(store: &mut WidgetStore) {
    for id in [ids::CEQ_CANCEL, ids::CEQ_APPLY, ids::CEQ_AUTO_WB] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }

    let rows: [(_, _, f32); 5] = [
        (
            ids::CEQ_CLIP_LIMIT,
            ids::CEQ_CLIP_LIMIT_NUM,
            clip_limit_to_slider(CLIP_LIMIT_DEFAULT),
        ),
        (
            ids::CEQ_TILE_GRID,
            ids::CEQ_TILE_GRID_NUM,
            tile_grid_to_slider(TILE_GRID_DEFAULT),
        ),
        (
            ids::CEQ_BRIGHTNESS,
            ids::CEQ_BRIGHTNESS_NUM,
            brightness_to_slider(BRIGHTNESS_DEFAULT),
        ),
        (
            ids::CEQ_CONTRAST,
            ids::CEQ_CONTRAST_NUM,
            contrast_to_slider(CONTRAST_DEFAULT),
        ),
        (
            ids::CEQ_SATURATION,
            ids::CEQ_SATURATION_NUM,
            saturation_to_slider(SATURATION_DEFAULT),
        ),
    ];
    for (slider_id, chip_id, track) in rows {
        store.register(
            slider_id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: track,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            chip_id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: track as f64,
                buffer: format_number(track as f64),
                caret: 0,
                last_committed: track as f64,
                selection_anchor: None,
            },
        );
        store.link_slider_number(slider_id, chip_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_registers_all_controls() {
        let mut store = WidgetStore::with_capacity(16);
        populate(&mut store);
        for id in [ids::CEQ_CANCEL, ids::CEQ_APPLY, ids::CEQ_AUTO_WB] {
            assert!(store.button_state(id).is_some(), "button {id:?} missing");
        }
        for id in [
            ids::CEQ_CLIP_LIMIT,
            ids::CEQ_TILE_GRID,
            ids::CEQ_BRIGHTNESS,
            ids::CEQ_CONTRAST,
            ids::CEQ_SATURATION,
        ] {
            assert!(store.slider(id).is_some(), "slider {id:?} missing");
        }
        for id in [
            ids::CEQ_CLIP_LIMIT_NUM,
            ids::CEQ_TILE_GRID_NUM,
            ids::CEQ_BRIGHTNESS_NUM,
            ids::CEQ_CONTRAST_NUM,
            ids::CEQ_SATURATION_NUM,
        ] {
            assert!(store.number_value(id).is_some(), "chip {id:?} missing");
        }
    }

    #[test]
    fn populate_links_each_slider_to_its_chip() {
        let mut store = WidgetStore::with_capacity(16);
        populate(&mut store);
        for (slider, chip) in [
            (ids::CEQ_CLIP_LIMIT, ids::CEQ_CLIP_LIMIT_NUM),
            (ids::CEQ_TILE_GRID, ids::CEQ_TILE_GRID_NUM),
            (ids::CEQ_BRIGHTNESS, ids::CEQ_BRIGHTNESS_NUM),
            (ids::CEQ_CONTRAST, ids::CEQ_CONTRAST_NUM),
            (ids::CEQ_SATURATION, ids::CEQ_SATURATION_NUM),
        ] {
            assert_eq!(store.linked_number(slider), Some(chip));
            assert_eq!(store.linked_slider(chip), Some(slider));
        }
    }
}
