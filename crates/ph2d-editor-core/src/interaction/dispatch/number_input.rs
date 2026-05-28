//! NumberInput / Slider-coupled-to-NumberInput dispatch helpers.
//!
//! Extracted from [`super`] (Track A4). Three responsibilities live
//! here:
//!
//! 1. **Slider drag → value** ([`update_drag_value`]) — convert a
//!    pointer position inside a slider's active rect into a `0..1`
//!    value, write it back to the slider's state, and mirror it
//!    into a linked NumberInput (if one is registered via
//!    [`super::super::WidgetStore::linked_number`]).
//! 2. **NumberInput stepper-arrow click** ([`apply_number_stepper_if_hit`])
//!    — when a click lands inside the up/down arrow rect of a
//!    NumberInput, apply one increment / decrement, mirror into a
//!    linked Slider, and arm the continuous-hold (so
//!    [`super::dispatch_tick`] can repeat the increment while the
//!    arrow stays pressed). Step heuristic: `0.01` if the buffer
//!    contains a `.`, else `1.0` (audit fix #6 — no buffer clone).
//! 3. **Char filter** ([`is_numeric_input_char`]) — gate text-input
//!    inserts so non-numeric characters can't put a NumberInput
//!    into an unparsable state.

use super::super::{InteractiveState, WidgetStore};
use crate::widget::{SliderOrientation, SliderState};
use crate::zones::Rect;
use ph2d_a11y::NodeId;

/// Recompute slider value from pointer position relative to its
/// active rect. Returns true iff the value actually changed (so
/// dispatcher can decide whether to emit `ValueChanged`). Mirrors
/// the new value into a linked NumberInput, if one is registered.
pub(super) fn update_drag_value(
    store: &mut WidgetStore,
    id: NodeId,
    rect: Rect,
    px: f32,
    py: f32,
) -> bool {
    let (changed, propagated) = {
        let Some(InteractiveState::Slider {
            state,
            value,
            orientation,
        }) = store.get_mut(id)
        else {
            return false;
        };
        let new_value = match *orientation {
            SliderOrientation::Horizontal => {
                if rect.w <= 0.0 {
                    0.0
                } else {
                    ((px - rect.x) / rect.w).clamp(0.0, 1.0)
                }
            }
            SliderOrientation::Vertical => {
                if rect.h <= 0.0 {
                    0.0
                } else {
                    (1.0 - (py - rect.y) / rect.h).clamp(0.0, 1.0)
                }
            }
        };
        let changed = (new_value - *value).abs() > f32::EPSILON;
        *value = new_value;
        *state = SliderState::Dragging;
        (changed, new_value as f64)
    };
    if changed && let Some(number_id) = store.linked_number(id) {
        // Forward-project storage → display so the chip's stored value
        // (and therefore the buffer shown on focus) matches what the
        // painter renders via `display_override`. Identity mapping
        // (the default) is `display = storage * 1 + 0`, equivalent to
        // the pre-mapping path. Non-identity mapping covers chips like
        // Grow ("±1" signed) and Min Px (integer count).
        let (scale, offset) = store.linked_slider_mapping(number_id);
        let display = (propagated as f32) * scale + offset;
        store.set_number_value(number_id, display as f64);
    }
    changed
}

/// Filter for chars allowed in a NumberInput buffer: digits, sign,
/// decimal point, and scientific-notation `e`/`E`/`+`. Anything else
/// (letters, spaces, control chars) is dropped silently.
pub(super) fn is_numeric_input_char(ch: char) -> bool {
    matches!(ch, '0'..='9' | '.' | '-' | '+' | 'e' | 'E')
}

/// If a pointer Down at `(click_x, click_y)` lands inside the up or
/// down stepper arrow of a NumberInput, apply one increment /
/// decrement (or no-op if the widget isn't a NumberInput / the click
/// missed both arrows) and arm the continuous-hold.
///
/// Step heuristic: `0.01` if `buffer.contains('.')` (fractional
/// value), `1.0` otherwise (integer). Dispatch has no access to the
/// widget's `step` field — that lives on the `NumberInput` struct,
/// not on the store's `InteractiveState`.
pub(super) fn apply_number_stepper_if_hit(
    store: &mut WidgetStore,
    id: NodeId,
    host: Rect,
    click_x: f32,
    click_y: f32,
    press_ns: u128,
) -> bool {
    use crate::widget::NumberInput;
    // Audit fix #6 (HIGH): peek value + step heuristic with an
    // immutable borrow — no `buffer.clone()` per click. Old path
    // allocated a fresh `String` every stepper-Down (every continuous-
    // hold tick), wasted work for a single-char membership test.
    let (current_value, step) = match store.get(id) {
        Some(InteractiveState::NumberInput { value, buffer, .. }) => (
            *value,
            if buffer.contains('.') {
                0.01_f64
            } else {
                1.0_f64
            },
        ),
        _ => return false,
    };
    let probe = NumberInput::new(id, "", current_value);
    let up = probe.up_rect(host);
    let down = probe.down_rect(host);
    let direction = if up.contains(click_x, click_y) {
        1.0_f64
    } else if down.contains(click_x, click_y) {
        -1.0_f64
    } else {
        return false;
    };
    let new_val = current_value + direction * step;
    // Shared mirror — writes chip, inverse-projects to slider with
    // clamp, re-syncs chip if clamped (so a step past the upper bound
    // of a mapped chip lands the visible buffer on the bound, not on
    // the unreachable raw value). Identity-linked / unlinked chips
    // pass through unchanged.
    let (final_val, _was_clamped) = super::apply_chip_value_with_mirror(store, id, new_val);
    if let Some(InteractiveState::NumberInput {
        last_committed, ..
    }) = store.get_mut(id)
    {
        *last_committed = final_val;
    }
    // M14.A: arm continuous-hold so dispatch_tick can repeat while
    // the user keeps the arrow pressed. The Down itself counts as the
    // first tick (already applied above) — `last_tick_ns == press_ns`
    // makes the initial-delay guard in `dispatch_tick` fire correctly.
    store.begin_number_stepper_hold(super::super::drag::NumberStepperHoldState {
        id,
        direction,
        step,
        press_ns,
        last_tick_ns: press_ns,
    });
    true
}
