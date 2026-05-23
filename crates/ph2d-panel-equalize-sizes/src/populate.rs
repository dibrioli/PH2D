//! Equalize Sizes panel `populate` — pre-registers every widget slot in
//! the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Layout:
//! - **3 mode buttons** (Max / Fixed / Grid), **3 algorithm buttons**
//!   (Lanczos / Nearest / xBR), **3 toggle buttons**
//!   (Align-to-grid / upscale-if-smaller / rasterize-after),
//!   **Cancel / Apply** — all plain buttons.
//! - **Fixed-mode W/H chips** (`EQS_FIXED_W`, `EQS_FIXED_H`) — standalone
//!   `NumberInput` widgets storing pixels (natural unit). Explicitly
//!   marked `mark_chip_no_stepper` so the dispatch's default phantom-
//!   stepper carve doesn't fire on their right edge.
//!
//! **No grid-unit slider / chip:** Grid-mode reads its cell size from
//! `GridSnapState::square_cfg.cell_size`. The shell bridge syncs
//! `params.grid_unit` (px) from the snap state every frame; the panel
//! paints the live value as read-only info text in Grid mode, alongside
//! the **Align position to grid** toggle.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, TextInputState};
use ph2d_tool_equalize_sizes::params::EqualizeSizesUiSnapshot;

pub fn populate(store: &mut WidgetStore) {
    // Every plain button (modes, algorithm, toggles incl. Align,
    // Cancel/Apply).
    for id in [
        ids::EQS_MODE_MAX,
        ids::EQS_MODE_FIXED,
        ids::EQS_MODE_GRID,
        ids::EQS_ALIGN_TO_GRID,
        ids::EQS_UPSCALE_IF_SMALLER,
        ids::EQS_RASTERIZE_AFTER,
        ids::EQS_ALG_LANCZOS,
        ids::EQS_ALG_NEAREST,
        ids::EQS_ALG_XBR,
        ids::EQS_CANCEL,
        ids::EQS_APPLY,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }

    // Fixed-mode W/H chips — standalone (no slider pair). Storage in
    // pixels (natural unit). Explicit `mark_chip_no_stepper` because
    // there's no `link_slider_number` here to auto-mark.
    let defaults = EqualizeSizesUiSnapshot::default();
    for (chip_id, default_px) in [
        (ids::EQS_FIXED_W, defaults.fixed_w),
        (ids::EQS_FIXED_H, defaults.fixed_h),
    ] {
        store.register(
            chip_id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: default_px as f64,
                buffer: default_px.to_string(),
                caret: 0,
                last_committed: default_px as f64,
                selection_anchor: None,
            },
        );
        store.mark_chip_no_stepper(chip_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_registers_all_controls() {
        let mut store = WidgetStore::with_capacity(32);
        populate(&mut store);
        for id in [
            ids::EQS_MODE_MAX,
            ids::EQS_MODE_FIXED,
            ids::EQS_MODE_GRID,
            ids::EQS_ALIGN_TO_GRID,
            ids::EQS_UPSCALE_IF_SMALLER,
            ids::EQS_RASTERIZE_AFTER,
            ids::EQS_ALG_LANCZOS,
            ids::EQS_ALG_NEAREST,
            ids::EQS_ALG_XBR,
            ids::EQS_CANCEL,
            ids::EQS_APPLY,
        ] {
            assert!(store.button_state(id).is_some(), "button {id:?} missing");
        }
        for id in [ids::EQS_FIXED_W, ids::EQS_FIXED_H] {
            assert!(
                store.number_value(id).is_some(),
                "number chip {id:?} missing"
            );
        }
    }

    #[test]
    fn fixed_chips_seed_in_pixels() {
        let mut store = WidgetStore::with_capacity(32);
        populate(&mut store);
        let defaults = EqualizeSizesUiSnapshot::default();
        assert_eq!(
            store.number_value(ids::EQS_FIXED_W),
            Some(defaults.fixed_w as f64)
        );
        assert_eq!(
            store.number_value(ids::EQS_FIXED_H),
            Some(defaults.fixed_h as f64)
        );
    }
}
