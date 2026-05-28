//! Per-frame tick for NumberInput continuous-hold (M14.A stepper).
//!
//! Extracted from [`super`] (Track A12). One public entry point —
//! [`dispatch_tick`] — that the shell calls once per frame with the
//! current host timestamp. The function drives the repeat interval
//! on an in-flight stepper hold (initial 250 ms delay, then 30 ms
//! between ticks), applying one +/- step to the held NumberInput
//! (and its linked Slider, if any). Zero allocation on the no-hold
//! fast path.

use super::super::drag::{STEPPER_HOLD_INITIAL_DELAY_NS, STEPPER_REPEAT_INTERVAL_NS};
use super::super::{InteractiveState, WidgetEvent, WidgetStore};
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;

/// M14.A: drive the continuous-hold repeat on a NumberInput stepper
/// arrow. The shell calls this once per frame with the current host
/// timestamp. After the initial 250 ms delay since Down, the function
/// fires one increment / decrement every 30 ms while the hold stays
/// active. Returns the slice of `WidgetEvent::ValueChanged` events
/// that fired this tick (zero-allocation via the bumpalo arena).
///
/// The Down event itself counts as the first tick (`apply_number_stepper_if_hit`
/// already applied the increment); `dispatch_tick` only handles the
/// repeats after the initial delay. The hold is cleared on Up
/// (see `PointerKind::Up` in `dispatch_pointer`).
pub fn dispatch_tick<'frame>(
    arena: &'frame Bump,
    store: &mut WidgetStore,
    now_ns: u128,
) -> &'frame [WidgetEvent] {
    let mut events = BumpVec::new_in(arena);
    let hold = match store.number_stepper_hold() {
        Some(h) => h,
        None => return events.into_bump_slice(),
    };
    // Initial delay: wait `STEPPER_HOLD_INITIAL_DELAY_NS` after the
    // press before the first repeat tick fires (matches macOS Aqua).
    if now_ns.saturating_sub(hold.press_ns) < STEPPER_HOLD_INITIAL_DELAY_NS {
        return events.into_bump_slice();
    }
    // After the initial delay, gate by the repeat interval.
    if now_ns.saturating_sub(hold.last_tick_ns) < STEPPER_REPEAT_INTERVAL_NS {
        return events.into_bump_slice();
    }
    let new_value = match store.get(hold.id) {
        Some(InteractiveState::NumberInput { value, .. }) => *value + hold.direction * hold.step,
        _ => {
            // Widget vanished mid-hold (e.g. selection switched and
            // the field was force-rewritten). Clear the hold so we
            // stop ticking against a non-existent target.
            store.end_number_stepper_hold();
            return events.into_bump_slice();
        }
    };
    // Shared mirror — symmetric with stepper-Down + commit-Enter so
    // the continuous-hold path can't drift from the single-tick path.
    let (final_val, _was_clamped) = super::apply_chip_value_with_mirror(store, hold.id, new_value);
    if let Some(InteractiveState::NumberInput {
        last_committed, ..
    }) = store.get_mut(hold.id)
    {
        *last_committed = final_val;
    }
    store.record_number_stepper_tick(now_ns);
    events.push(WidgetEvent::ValueChanged(hold.id));
    // `apply_chip_value_with_mirror` also writes a linked slider —
    // emit its ValueChanged so panel handlers keyed off the slider id
    // (canonical pattern post-mapped-link) see the change.
    if let Some(slider_id) = store.linked_slider(hold.id) {
        events.push(WidgetEvent::ValueChanged(slider_id));
    }
    events.into_bump_slice()
}
