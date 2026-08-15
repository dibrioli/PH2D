//! **The Motion cook's tick arithmetic** — split from `motion_bridge` for the shell LOC cap.
//! Também a derivação tique↔SEGUNDOS e a trava de entrada da ferramenta: o mesmo assunto,
//! e o pai estava a uma linha do teto.
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

/// The fixed tick the playhead is standing on — the Motion cook's clock, DERIVED
/// (W4.T7). Motion keeps no transport of its own: it used to, and two clocks that
/// each advance themselves are two clocks that drift.
///
/// The cook is tick-based on purpose (a fixed `dt` is what makes a spring
/// deterministic, plan §1.4) while the playhead is a continuous position in
/// seconds — so the seam rounds. At `rate == 1` the playhead's time is an exact
/// multiple of `fixed_dt` and the rounding is a no-op; a seek to mid-tick lands on
/// the nearest one.
pub(crate) fn motion_tick(playhead: &ph2d_core::Playhead, fixed_dt: f64) -> u64 {
    if fixed_dt <= 0.0 {
        return 0;
    }
    // `Playhead::time` is `>= 0` by construction; the clamp is belt-and-braces
    // against a NaN reaching the cast, which would saturate to 0 silently.
    let t = playhead.time() / fixed_dt;
    if t.is_finite() && t > 0.0 {
        t.round() as u64
    } else {
        0
    }
}

/// The COOK's time in seconds — the tick of [`motion_tick`] measured back out in seconds.
///
/// This is what a readout (probe, flow marching) must be sampled at, and it is NOT the same
/// as `Playhead::time()`: the playhead is continuous and a scrub lands mid-tick, while the
/// pump's memo only ever holds the tick it cooked. Reading the panel at the raw playhead time
/// would ask the memo for an instant that was never simulated — the derived-coordinate trap
/// (the seed must match the sample), and it has bitten this codebase before.
pub(super) fn motion_time(playhead: &ph2d_core::Playhead, fixed_dt: f64) -> f64 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "um tique cabe num f64 com folga"
    )]
    let t = motion_tick(playhead, fixed_dt) as f64;
    t * fixed_dt
}

/// The tool-activation latch: `true` while the Motion tool is the active tool.
///
/// The auto-play (and the scene/graph split) is EDGE-triggered on entry, so a document that
/// changes underneath an already-open tool never sees that edge.
pub(super) static LAST_ACTIVE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// **Forget that the tool was ever entered** — so the next frame re-fires the entry edge.
///
/// Loading a project is a NEW document arriving under an open tool. Without this, the Motion
/// tool's *"auto-play on entry so time-driven behaviours animate live"* never re-fires (the tool
/// did not change; the document did), and — since a load now rewinds AND pauses the editor's one
/// clock — the artist would open a project and watch the graph sit frozen at t=0 until they
/// pressed Space. Re-entering is what the loaded document deserves: it is being opened for the
/// first time.
pub(crate) fn forget_tool_transition() {
    LAST_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
}
