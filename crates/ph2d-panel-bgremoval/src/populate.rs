//! Background-Removal panel `populate` — pre-registers the panel's
//! widget slots in the `WidgetStore` at host boot (once, via
//! `Panel::populate`). Initial slider values are placeholders; the host
//! overwrites them every frame from the live `BgRemovalUiSnapshot` (the
//! paint reads the snapshot, not the stored slider value, for track
//! position — the stored value is what dispatch mutates on drag and what
//! [`crate::event`] reads on `ValueChanged`).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::tools::bgremoval::BgRemovalUiSnapshot;
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

pub fn populate(store: &mut WidgetStore) {
    for id in [
        ids::BGR_APPLY,
        ids::BGR_CANCEL,
        // Eyedropper toggle. Swatches need NO store entry — they're
        // paint-time hit registrations from a fixed id pool.
        ids::BGR_EYEDROPPER,
        // Protection-brush toggle + its Clear button.
        ids::BGR_PROTECT,
        ids::BGR_PROTECT_CLEAR,
        // Show-mask toggle + the 4-way falloff segmented buttons.
        ids::BGR_SHOW_MASK,
        ids::BGR_FALLOFF_SMOOTH,
        ids::BGR_FALLOFF_SPHERE,
        ids::BGR_FALLOFF_SHARP,
        ids::BGR_FALLOFF_CONSTANT,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Seed slider positions from the canonical default snapshot so the
    // boot UI can never drift from `BgRemovalParams::default()` (single
    // source of truth — the tuned defaults live there, not here).
    let d = BgRemovalUiSnapshot::default();
    for (slider_id, chip_id, value) in [
        (ids::BGR_TOLERANCE, ids::BGR_TOLERANCE_NUM, d.tolerance01),
        (ids::BGR_FEATHER, ids::BGR_FEATHER_NUM, d.feather01),
        (ids::BGR_REFINE, ids::BGR_REFINE_NUM, d.refine01),
        // Grow/Shrink is bipolar (0.5 = neutral); default is a slight erode.
        (ids::BGR_GROW, ids::BGR_GROW_NUM, d.grow01),
        // Protection-brush size (source-px radius, normalized).
        (ids::BGR_BRUSH_SIZE, ids::BGR_BRUSH_SIZE_NUM, d.brush_size01),
    ] {
        store.register(
            slider_id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value,
                orientation: SliderOrientation::Horizontal,
            },
        );
        // Editable numeric chip paired with the slider — keyboard +
        // drag-scrub via the canonical NumberInput dispatch (same
        // behaviour as the Inspector / color-picker chips).
        let v = value as f64;
        store.register(
            chip_id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: v,
                buffer: format!("{v:.3}"),
                caret: 0,
                last_committed: v,
                selection_anchor: None,
            },
        );
        // Bidirectional slider↔chip link — the SINGLE source of truth
        // for this behaviour. With it, the canonical dispatch (a) clamps
        // chip keyboard/drag-scrub edits to the slider's 0..1 range and
        // (b) mirrors the value back onto the slider track live. Exactly
        // how the Widget Gallery wires `INSP_SAMPLE_SLIDER` to its chip
        // (see `screens/hero/pre_populate.rs`). Without it the chip is
        // orphaned: edits don't move the slider and aren't range-bounded.
        store.link_slider_number(slider_id, chip_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_registers_all_controls() {
        let mut store = WidgetStore::with_capacity(8);
        populate(&mut store);
        // Buttons.
        for id in [ids::BGR_APPLY, ids::BGR_CANCEL, ids::BGR_PROTECT] {
            assert!(store.button_state(id).is_some(), "button {id:?} missing");
        }
        // Sliders seeded from the default snapshot (tuned defaults:
        // tolerance 0.6, feather 0.9, refine 0.01).
        let d = BgRemovalUiSnapshot::default();
        for (id, expect) in [
            (ids::BGR_TOLERANCE, d.tolerance01),
            (ids::BGR_FEATHER, d.feather01),
            (ids::BGR_REFINE, d.refine01),
        ] {
            let (_, v) = store.slider(id).expect("slider registered");
            assert!((v - expect).abs() < 1e-5, "slider {id:?}: {v} vs {expect}");
        }
    }
}
