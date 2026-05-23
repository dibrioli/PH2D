//! Color Equalization panel `populate` — pre-registers the panel's widget
//! slots in the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Five horizontal sliders (`CEQ_*`, normalized `0..1`) each paired with a
//! chip (`CEQ_*_NUM`) that displays the natural unit (clip limit, tile
//! grid count, brightness etc.), plus an Auto-WB toggle and Cancel /
//! Apply buttons. The slider↔chip pair is NOT wired with
//! `link_slider_number` — that helper couples both widgets in the same
//! `0..1` space, but our chips show natural units. Instead [`crate::event`]
//! mirrors the values manually (slider drag → chip value; chip type →
//! slider track), and the host overwrites both from the live snapshot
//! every frame.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_color_equalization::params::{
    BRIGHTNESS_DEFAULT, BRIGHTNESS_MAX, BRIGHTNESS_MIN, CLIP_LIMIT_DEFAULT, CLIP_LIMIT_MAX,
    CLIP_LIMIT_MIN, CONTRAST_DEFAULT, CONTRAST_MAX, CONTRAST_MIN, SATURATION_DEFAULT,
    SATURATION_MAX, SATURATION_MIN, TILE_GRID_DEFAULT, TILE_GRID_MAX, TILE_GRID_MIN,
    brightness_to_slider, clip_limit_to_slider, contrast_to_slider, saturation_to_slider,
    tile_grid_to_slider,
};

pub fn populate(store: &mut WidgetStore) {
    // Cancel / Apply / Auto-WB buttons.
    for id in [ids::CEQ_CANCEL, ids::CEQ_APPLY, ids::CEQ_AUTO_WB] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }

    // Five slider+chip rows, each seeded from the identity defaults so the
    // first frame paints sane values even before the host pushes a
    // snapshot.
    let rows: [(_, _, f32, f64, &str); 5] = [
        (
            ids::CEQ_CLIP_LIMIT,
            ids::CEQ_CLIP_LIMIT_NUM,
            clip_limit_to_slider(CLIP_LIMIT_DEFAULT),
            CLIP_LIMIT_DEFAULT as f64,
            "2.0",
        ),
        (
            ids::CEQ_TILE_GRID,
            ids::CEQ_TILE_GRID_NUM,
            tile_grid_to_slider(TILE_GRID_DEFAULT),
            TILE_GRID_DEFAULT as f64,
            "8",
        ),
        (
            ids::CEQ_BRIGHTNESS,
            ids::CEQ_BRIGHTNESS_NUM,
            brightness_to_slider(BRIGHTNESS_DEFAULT),
            BRIGHTNESS_DEFAULT as f64,
            "0.00",
        ),
        (
            ids::CEQ_CONTRAST,
            ids::CEQ_CONTRAST_NUM,
            contrast_to_slider(CONTRAST_DEFAULT),
            CONTRAST_DEFAULT as f64,
            "1.00",
        ),
        (
            ids::CEQ_SATURATION,
            ids::CEQ_SATURATION_NUM,
            saturation_to_slider(SATURATION_DEFAULT),
            SATURATION_DEFAULT as f64,
            "0.00",
        ),
    ];
    for (slider_id, chip_id, track, chip_v, chip_buf) in rows {
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
                value: chip_v,
                buffer: chip_buf.to_string(),
                caret: 0,
                last_committed: chip_v,
                selection_anchor: None,
            },
        );
    }

    // Per-chip drag step + bounds — the buffer-inferred heuristic
    // ("." → 0.01 step, else 1.0) was tuned for chips with wide
    // natural ranges (Inspector position 0..N×1000 px). CEQ chips have
    // narrow ranges (brightness ±1, contrast 0.5..2.0, etc.) so the
    // heuristic's `DRAG_RATE_X × step = 50 × 0.01 = 0.5 units/px` would
    // cover the full range in 2-6 pixels — unusable. Targets here aim
    // for "full range over ~200 px of drag" so the chip scrub matches
    // the slider's feel.
    let steps_and_bounds: [(_, f64, f64, f64); 5] = [
        // (chip_id, step, min, max) — `delta = dom_dx * 50.0 * step`.
        (
            ids::CEQ_CLIP_LIMIT_NUM,
            (CLIP_LIMIT_MAX - CLIP_LIMIT_MIN) as f64 / (200.0 * 50.0),
            CLIP_LIMIT_MIN as f64,
            CLIP_LIMIT_MAX as f64,
        ),
        (
            ids::CEQ_TILE_GRID_NUM,
            (TILE_GRID_MAX - TILE_GRID_MIN) as f64 / (200.0 * 50.0),
            TILE_GRID_MIN as f64,
            TILE_GRID_MAX as f64,
        ),
        (
            ids::CEQ_BRIGHTNESS_NUM,
            (BRIGHTNESS_MAX - BRIGHTNESS_MIN) as f64 / (200.0 * 50.0),
            BRIGHTNESS_MIN as f64,
            BRIGHTNESS_MAX as f64,
        ),
        (
            ids::CEQ_CONTRAST_NUM,
            (CONTRAST_MAX - CONTRAST_MIN) as f64 / (200.0 * 50.0),
            CONTRAST_MIN as f64,
            CONTRAST_MAX as f64,
        ),
        (
            ids::CEQ_SATURATION_NUM,
            (SATURATION_MAX - SATURATION_MIN) as f64 / (200.0 * 50.0),
            SATURATION_MIN as f64,
            SATURATION_MAX as f64,
        ),
    ];
    for (chip_id, step, min, max) in steps_and_bounds {
        store.register_chip_drag_step(chip_id, step);
        store.register_chip_drag_bounds(chip_id, min, max);
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
}
