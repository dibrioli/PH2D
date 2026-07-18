//! The **two invalidations** of a GPU simulation (ADR-0130 D7) — no device
//! needed, so they run on every CI lane.
//!
//! They answer different questions, and using the wrong one is what the emitter
//! smoke caught ("re-bake travado"):
//!
//! - [`GpuCook::forget_state`] — *"this state is invalid, RE-DERIVE it"*. The
//!   next `rewind_for` anchors the empty ring at tick 0, so the caller re-cooks
//!   `0..=target`. One honest bake for a discrete change (a document load, where
//!   the clock is rewound anyway); `O(current tick)` if it fires while a param is
//!   being *held and dragged*, which is a freeze, not a bake.
//! - [`GpuCook::reseed_from_next_tick`] — *"this state is invalid, RESTART the
//!   sim from now"*. The next `rewind_for` returns the target itself: ONE cook,
//!   which seeds (there is no prior state) and steps forward from there.
//!
//! `rewind_for` is pure state arithmetic, so the difference is observable with no
//! adapter at all — which is the whole reason this gate can exist.

use ph2d_gpu_cook::GpuCook;

/// Deliberately far from 0: at tick 1 the two policies agree by accident, and a
/// gate that could not tell "re-derive the history" from "restart from now" is
/// exactly the gate that would have let the freeze ship.
const DEEP_INTO_PLAYBACK: u64 = 600;

#[test]
fn a_full_forget_re_derives_the_history_from_the_seed() {
    // The scrub/document-change invalidation: nothing is known, so the march has
    // to start at the tick-0 seed and replay. That is the O(tick) cost, and it is
    // CORRECT here — it is only the wrong answer for a live drag.
    let mut gc = GpuCook::new();
    gc.forget_state();
    assert_eq!(
        gc.rewind_for(DEEP_INTO_PLAYBACK),
        0,
        "a full forget must re-derive from the seed"
    );
}

#[test]
fn a_live_edit_reseeds_at_the_target_instead_of_re_baking_the_history() {
    // THE fix for the smoke's "re-bake travado". The caller's march is
    // `rewind_for(target)..=target`, so returning the target itself is exactly
    // ONE cook — and with no prior state that cook is the seed. Holding a drag
    // therefore costs one cook per frame, like ordinary playback, instead of
    // re-simulating six hundred ticks per frame.
    let mut gc = GpuCook::new();
    gc.reseed_from_next_tick();
    assert_eq!(
        gc.rewind_for(DEEP_INTO_PLAYBACK),
        DEEP_INTO_PLAYBACK,
        "a live edit must seed AT the target — one cook, not `target` of them"
    );
    // …and it really is the SEED path: no state survived the edit, so the
    // kernel's `HAS_*` reads false and every element takes its seed branch.
    assert_eq!(
        gc.last_cooked_tick(),
        None,
        "no prior state survives a live-edit invalidation"
    );
}

#[test]
fn the_reseed_is_consumed_once_and_does_not_stick() {
    // A flag that survived its own use would pin the sim at the seed forever:
    // every later frame would "restart from now" and the fountain would never
    // accumulate. The second call must fall back to the ordinary forward march.
    let mut gc = GpuCook::new();
    gc.reseed_from_next_tick();
    assert_eq!(gc.rewind_for(10), 10, "the edit's own frame seeds at 10");
    // `rewind_for` alone does not cook, so `last_tick` is still None and the
    // fallback is the anchor — the point is that it is no longer the RESEED
    // branch, which would have answered 20.
    assert_eq!(
        gc.rewind_for(20),
        0,
        "the flag is spent: the next answer comes from the ordinary rule, not the reseed"
    );
}

#[test]
fn a_forget_cancels_a_pending_reseed() {
    // The two are not additive: a document load (`forget_state`) after a param
    // edit must re-derive, not inherit the edit's "restart from now" — otherwise
    // the freshly loaded document would skip its own history.
    let mut gc = GpuCook::new();
    gc.reseed_from_next_tick();
    gc.forget_state();
    assert_eq!(
        gc.rewind_for(DEEP_INTO_PLAYBACK),
        0,
        "the later, stronger invalidation wins"
    );
}
