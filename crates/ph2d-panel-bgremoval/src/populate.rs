//! Background-Removal panel `populate` — pre-registers the panel's
//! widget slots in the `WidgetStore` at host boot (once, via
//! `Panel::populate`). Initial slider values are placeholders; the host
//! overwrites them every frame from the live `BgRemovalUiSnapshot` (the
//! paint reads the snapshot, not the stored slider value, for track
//! position — the stored value is what dispatch mutates on drag and what
//! [`crate::event`] reads on `ValueChanged`).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_bgremoval::params::{BgRemovalUiSnapshot, MIN_ISLAND_PIXELS_FULL_SCALE};

pub fn populate(store: &mut WidgetStore) {
    for id in [
        ids::BGR_APPLY,
        ids::BGR_CANCEL,
        ids::BGR_RESET,
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
        // "Separate Islands" toggle — legacy parity. When on, an Apply
        // pass also splits the result into one sprite per connected
        // component (see `ph2d_tool_bgremoval::algorithm::islands`).
        ids::BGR_SEPARATE_ISLANDS,
        // "Detect subject" toggle — edge-aware silhouette upgrade
        // (Enio 2026-05-26). Without this register the dispatcher
        // doesn't recognise the id as a clickable button and the
        // click is silently dropped.
        ids::BGR_AUTO_PROTECT_SUBJECT,
        // "Add area" toggle + its Clear button (Enio 2026-05-26) —
        // symmetric to the eyedropper: arm → single click → flood-
        // fill writes the connected same-colour region into the
        // force-remove mask. Shown in the eyedropper-row slot when
        // Detect Subject is on. Same dispatcher-register requirement
        // as every other clickable id (see BGR_AUTO_PROTECT_SUBJECT
        // comment above).
        ids::BGR_ADD_AREA,
        ids::BGR_ADD_AREA_CLEAR,
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
    //
    // The chip seed is the **display-space** value (what the user sees
    // and types), even when that differs from the slider's `0..1`
    // storage. The dispatch projects between the two via the mapping
    // registered with `link_slider_number_mapped`. This is the
    // 2026-05-27 fix for the "type 0.2 see -0.6" bug — the chip's
    // stored value now matches its painter (`display_override`) so
    // round-trip is invariant.
    let d = BgRemovalUiSnapshot::default();
    // Identity-mapped chips (display = storage). Refine / Feather /
    // Tolerance / Brush Size all paint the raw 0..1 slider value.
    for (slider_id, chip_id, value) in [
        (ids::BGR_TOLERANCE, ids::BGR_TOLERANCE_NUM, d.tolerance01),
        (ids::BGR_FEATHER, ids::BGR_FEATHER_NUM, d.feather01),
        (ids::BGR_REFINE, ids::BGR_REFINE_NUM, d.refine01),
        (ids::BGR_BRUSH_SIZE, ids::BGR_BRUSH_SIZE_NUM, d.brush_size01),
    ] {
        register_slider_chip_pair(store, slider_id, chip_id, value, value as f64);
        store.link_slider_number(slider_id, chip_id);
    }
    // Grow/Shrink — bipolar `±1` display: `signed = (storage-0.5)*2`,
    // i.e. `display = storage*2 + (-1)`. Mapping `(scale=2, offset=-1)`.
    {
        let grow_display = (d.grow01 - 0.5) * 2.0;
        register_slider_chip_pair(
            store,
            ids::BGR_GROW,
            ids::BGR_GROW_NUM,
            d.grow01,
            grow_display as f64,
        );
        store.link_slider_number_mapped(ids::BGR_GROW, ids::BGR_GROW_NUM, 2.0, -1.0);
    }
    // Min island pixels — integer count `[1..FULL_SCALE]`:
    // `count = storage*(FULL_SCALE-1) + 1`, i.e. mapping
    // `(scale=FULL_SCALE-1, offset=1)`.
    {
        let min_scale = MIN_ISLAND_PIXELS_FULL_SCALE - 1.0;
        let min_display = (d.min_island_pixels01 * min_scale) + 1.0;
        register_slider_chip_pair(
            store,
            ids::BGR_MIN_ISLAND_PX,
            ids::BGR_MIN_ISLAND_PX_NUM,
            d.min_island_pixels01,
            min_display as f64,
        );
        // Integer-domain snap (audit finding #3, 2026-05-28):
        // chip stores `1..FULL_SCALE` (whole pixels). Without the
        // _integer variant, typing "50.5" left the chip at 50.5 while
        // the painter rounded to "50" — split-brain. The variant
        // rounds the typed display before persisting.
        store.link_slider_number_mapped_integer(
            ids::BGR_MIN_ISLAND_PX,
            ids::BGR_MIN_ISLAND_PX_NUM,
            min_scale,
            1.0,
        );
    }
}

fn register_slider_chip_pair(
    store: &mut WidgetStore,
    slider_id: ph2d_a11y::NodeId,
    chip_id: ph2d_a11y::NodeId,
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
