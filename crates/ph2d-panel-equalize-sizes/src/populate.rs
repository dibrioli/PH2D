//! Equalize Sizes panel `populate` — pre-registers every widget slot in
//! the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Layout:
//! - **3 mode buttons** (Max / Fixed / Grid), **3 algorithm buttons**
//!   (Lanczos / Nearest / xBR), **3 toggle buttons**
//!   (Arrange-on-grid / upscale-if-smaller / rasterize-after),
//!   **Cancel / Apply** — all plain buttons.
//! - **Fixed-mode W/H chips** (`EQS_FIXED_W`, `EQS_FIXED_H`) — standalone
//!   `NumberInput` widgets storing pixels (natural unit). Paint via
//!   canonical `paint_number_chip` (arrows + click-drag h/v + step-on-
//!   click; chip canon post-2026-05-24).
//! - **Grid-mode "Offset" slider + chip** (`EQS_GRID_OFFSET`,
//!   `EQS_GRID_OFFSET_NUM`) — Widget Gallery §4.2 pair: both store the
//!   raw offset in PIXELS, paired via `link_slider_number`. The slider
//!   track maps to `0..1` linearly the same way every other paired
//!   slider does — but since the chip's range here IS pixels (small
//!   numbers like 0..32 typically), we store pixels too. The display
//!   override in paint shows "N px"; the dispatch clamp + mirror works
//!   in the shared px space.
//!
//! Grid cell size has no slider/chip — it's read from
//! `GridSnapState::square_cfg.cell_size` and pushed in by the bridge
//! every frame; the panel paints the live `snapshot.grid_unit` as
//! read-only info text in Grid mode.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_equalize_sizes::params::EqualizeSizesUiSnapshot;

pub fn populate(store: &mut WidgetStore) {
    // Every plain button (modes, algorithm, toggles incl. Arrange,
    // Cancel/Apply).
    for id in [
        ids::EQS_MODE_MAX,
        ids::EQS_MODE_FIXED,
        ids::EQS_MODE_GRID,
        ids::EQS_ARRANGE_ON_GRID,
        ids::EQS_UPSCALE_IF_SMALLER,
        ids::EQS_RASTERIZE_AFTER,
        ids::EQS_ALG_LANCZOS,
        ids::EQS_ALG_NEAREST,
        ids::EQS_ALG_XBR,
        ids::EQS_CANCEL,
        ids::EQS_APPLY,
        ids::EQS_RESET,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }

    // Fixed-mode W/H chips — standalone (no slider pair). Storage in
    // pixels (natural unit). Post-2026-05-24: chips paint arrows; the
    // stepper click→step affordance is the canon for every chip.
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
    }

    // Grid-mode Offset slider + chip — slider stores TRACK `0..1`, chip
    // stores PIXELS (natural unit, since the user expects to type "4 px"
    // not "0.25").
    //
    // Intentionally NOT migrated to `link_slider_number_mapped` (the
    // 2026-05-27 canonical pattern used by bgremoval / color-eq /
    // padding / upscale) because the projection scale is **dynamic**:
    // `max_offset = snapshot.grid_unit / 2` changes per frame as the
    // user edits Grid Unit on the side. The affine `(scale, offset)`
    // registered at populate time cannot model a per-frame variable
    // bound — migrating would either freeze `max_offset` to a
    // constant (UX regression: slider sweep covers an invalid range)
    // or require a closure-based mapping API (premature abstraction
    // for a single call site, vide CLAUDE.md "Don't design for
    // hypothetical future requirements").
    //
    // [`crate::event`] mirrors the two manually each frame the user
    // edits one. The shared `apply_chip_value_with_mirror` re-sync
    // invariant is NOT in effect here — clamping is handled in the
    // event handler against the live `snapshot.grid_unit / 2`.
    let default_offset = defaults.grid_offset as f64;
    store.register(
        ids::EQS_GRID_OFFSET,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.0,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::EQS_GRID_OFFSET_NUM,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: default_offset,
            buffer: format_number(default_offset),
            caret: 0,
            last_committed: default_offset,
            selection_anchor: None,
        },
    );
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
            ids::EQS_ARRANGE_ON_GRID,
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
        for id in [ids::EQS_FIXED_W, ids::EQS_FIXED_H, ids::EQS_GRID_OFFSET_NUM] {
            assert!(
                store.number_value(id).is_some(),
                "number chip {id:?} missing"
            );
        }
        assert!(store.slider(ids::EQS_GRID_OFFSET).is_some());
    }

    #[test]
    fn offset_chip_paints_steppers_like_every_other_chip() {
        // Post-2026-05-24 chip canon: every chip paints arrows and the
        // dispatch's stepper hit-test is the canon click→step
        // affordance — no chip is "no-stepper" anymore. Asserts the
        // opposite of the old `offset_chip_marked_no_stepper` test.
        let mut store = WidgetStore::with_capacity(32);
        populate(&mut store);
        assert!(!store.is_chip_no_stepper(ids::EQS_GRID_OFFSET_NUM));
    }
}
