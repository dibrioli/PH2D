//! **The Motion cook's tick arithmetic** — split from `motion_bridge` for the shell LOC cap.
//! Declared there as a `#[path]` sibling, so `super` is `render_loop::motion_bridge`, which
//! re-exports [`ticks_owed`] (the per-frame cook loop and the GPU path both call it as
//! `super::ticks_owed`). Pure: no state, no `MotionState` — just the range of fixed ticks owed.

/// The ticks the cook still owes to reach `target`, given the last one it rendered.
///
/// **Forward: every tick, never a skip.** One cook + `pre`-advance per FIXED TICK,
/// never per frame (M2-dynamics). A sequential node's trajectory (`integrate`,
/// `spring`, `verlet_rope`) is the SUM of its steps, so a slow frame that produced
/// two fixed steps — or a playhead running at `rate 2` — must sim BOTH, or the
/// motion would depend on the frame rate (plan §1.4: dt fixo). The common frame
/// owes exactly one tick, which takes the pump's cheap forward path.
///
/// **Backwards or a jump: one call.** The playhead was scrubbed, sought, or wrapped
/// its loop. [`ph2d_eval_motion::MotionCookPump::advance_or_scrub_scoped`] restores
/// the newest checkpoint at or before the target and re-sims forward, bit-exact
/// (M2.N2) — walking there tick by tick would instead re-cook from the ring on every
/// step. Reading the marching future would show a spring mid-flight at a time it never
/// was in.
///
/// **Standing still: the same tick.** A paused playhead re-issues its current tick,
/// which the pump early-returns on unless a param drag / rewire dirtied the cook
/// (zero-alloc paused frame, M0.T12).
pub(crate) fn ticks_owed(last_cooked: Option<u64>, target: u64) -> std::ops::RangeInclusive<u64> {
    match last_cooked {
        Some(last) if target > last => last + 1..=target,
        _ => target..=target,
    }
}
