//! [`Playhead`] — the engine-wide time cursor + transport.
//!
//! The Playhead is the single source of "where are we on the timeline right
//! now". It is what every animatable system (sprites, vectors, painter, node
//! params, …) reads to sample its animation for the current frame — the live
//! scalar side of the general timeline.
//!
//! # Time model
//!
//! The playhead position is an `f64` in **seconds**. This is deliberate: the
//! *storage* of keyframes is drift-free rational time (`ph2d-anim`'s
//! `RationalTime`), and `f64` seconds is exactly the boundary type the samplers
//! take (`AttributeEvaluator::sample(t: f64)`). Keeping the playhead in
//! `ph2d-core` (which sits below `ph2d-anim`) also keeps this primitive free of
//! any animation-data dependency.
//!
//! # Advancing
//!
//! [`Playhead::advance`] is called **once per fixed simulation tick** (driven by
//! [`crate::FixedStep`]); it moves the position by `fixed_dt * rate` while
//! playing. Because it is a fixed, in-order sequence of `f64` additions (no FMA,
//! no reordering — HR-5), the same sequence of `advance`/`seek`/transport calls
//! reproduces bit-identical positions across runs and platforms.
//!
//! Presentation animation is exempt from the HR-5 determinism membrane
//! (ADR-0030), but the transport is deterministic anyway so a future
//! gameplay-motion consumer can rely on it.

use crate::time::FixedStep;

/// How many times one `advance` may reflect off the loop's ends. A step longer
/// than the span (a huge `rate` over a tiny loop) would bounce more than once;
/// this bounds the fold so a pathological rate cannot spin here forever. Two
/// bounces cover any step up to twice the span, which no real transport reaches.
const MAX_PING_BOUNCES: usize = 8;

/// How the transport behaves at the end of its [loop range](Playhead::set_loop).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopMode {
    /// Jump back to the start — a cycle (the classic loop).
    #[default]
    Wrap,
    /// Turn around and play backwards to the start, then forwards again — a
    /// **ping-pong**. What a walk cycle authored one way needs to read both ways,
    /// and what AE calls a "boomerang".
    PingPong,
}

/// The engine-wide timeline cursor and its transport state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Playhead {
    /// Current position in seconds (`>= 0`).
    time: f64,
    /// Seconds advanced per fixed tick at `rate == 1.0` (from [`FixedStep`]).
    fixed_dt: f64,
    /// Playback-speed multiplier (`1.0` = real time, `0.5` = half speed,
    /// negative = reverse). Applied per [`Playhead::advance`].
    rate: f64,
    /// Whether [`Playhead::advance`] moves the position.
    playing: bool,
    /// Optional `[start, end)` loop; while set, [`Playhead::advance`] wraps the
    /// position back into the range (deterministically, via `rem_euclid`).
    loop_range: Option<(f64, f64)>,
    /// What happens at the end of the loop — wrap, or turn around.
    loop_mode: LoopMode,
    /// Which way a [`LoopMode::PingPong`] is currently travelling: `+1` forwards,
    /// `-1` back. Meaningless (and left at `+1`) in [`LoopMode::Wrap`].
    ping_dir: f64,
}

impl Playhead {
    /// Create a playhead at `t = 0` with the given per-tick step, playing at
    /// `rate = 1.0`. `fixed_dt` is clamped to be strictly positive.
    #[must_use]
    pub fn new(fixed_dt: f64) -> Self {
        Self {
            time: 0.0,
            fixed_dt: if fixed_dt.is_finite() && fixed_dt > 0.0 {
                fixed_dt
            } else {
                1.0 / crate::time::DEFAULT_HZ
            },
            rate: 1.0,
            playing: true,
            loop_range: None,
            loop_mode: LoopMode::Wrap,
            ping_dir: 1.0,
        }
    }

    /// Create a playhead whose per-tick step matches a [`FixedStep`] clock.
    #[must_use]
    pub fn from_fixed_step(step: &FixedStep) -> Self {
        Self::new(step.fixed_dt())
    }

    /// Advance the position by one fixed tick (`fixed_dt * rate`) if playing.
    /// Call this once per simulation tick. The position never goes below `0`; if
    /// a [loop range](Playhead::set_loop) is set, the position wraps back into
    /// `[start, end)` deterministically.
    pub fn advance(&mut self) {
        if !self.playing {
            return;
        }
        let step = self.fixed_dt * self.rate;
        let Some((start, end)) = self.loop_range.filter(|&(a, b)| b > a) else {
            self.time = (self.time + step).max(0.0);
            return;
        };
        match self.loop_mode {
            LoopMode::Wrap => {
                self.time = (self.time + step).max(0.0);
                self.time = start + (self.time - start).rem_euclid(end - start);
            }
            // REFLECT off each end rather than jumping back. The reflection is
            // applied to the OVERSHOOT, not to the raw position, so a step that
            // lands past the end resumes exactly as far inside as it went beyond —
            // no frame is dropped and no time is invented, which is what keeps a
            // ping-pong bit-exact under any `rate`.
            LoopMode::PingPong => {
                let mut t = self.time + step * self.ping_dir;
                // A rate so large it overshoots the whole span would bounce more
                // than once; fold repeatedly rather than clamp, so it stays exact.
                for _ in 0..MAX_PING_BOUNCES {
                    if t > end {
                        t = end - (t - end);
                        self.ping_dir = -1.0;
                    } else if t < start {
                        t = start + (start - t);
                        self.ping_dir = 1.0;
                    } else {
                        break;
                    }
                }
                self.time = t.clamp(start, end); // CLAMP-OK: measured bounds, end>start
            }
        }
    }

    /// Advance by `n` fixed ticks (convenience for a frame that ran multiple
    /// substeps). Equivalent to calling [`Playhead::advance`] `n` times.
    pub fn advance_ticks(&mut self, n: u32) {
        for _ in 0..n {
            self.advance();
        }
    }

    /// The current position in seconds.
    #[must_use]
    pub fn time(&self) -> f64 {
        self.time
    }

    /// The current frame index at a given frame rate (`round(time * fps)`).
    #[must_use]
    pub fn frame(&self, fps: f64) -> i64 {
        (self.time * fps).round() as i64
    }

    /// Jump the playhead to an absolute time in seconds (scrub). Clamped to
    /// `>= 0`. Does not change play/pause state.
    pub fn seek(&mut self, seconds: f64) {
        self.time = if seconds.is_finite() {
            seconds.max(0.0)
        } else {
            0.0
        };
    }

    /// Jump to a frame index at a given frame rate.
    pub fn seek_frame(&mut self, frame: i64, fps: f64) {
        let fps = if fps.is_finite() && fps > 0.0 {
            fps
        } else {
            crate::time::DEFAULT_HZ
        };
        self.seek(frame as f64 / fps);
    }

    /// Reset to `t = 0` (keeps rate + play state).
    pub fn rewind(&mut self) {
        self.time = 0.0;
    }

    /// Set the playback loop to `[start, end)` (seconds). `start` is clamped to
    /// `>= 0`; a non-finite range or `end <= start` clears the loop instead.
    /// While set, [`Playhead::advance`] wraps the position back into the range.
    pub fn set_loop(&mut self, start: f64, end: f64) {
        if start.is_finite() && end.is_finite() && end > start.max(0.0) {
            self.loop_range = Some((start.max(0.0), end));
        } else {
            self.loop_range = None;
        }
    }

    /// Clear the playback loop.
    pub fn clear_loop(&mut self) {
        self.loop_range = None;
    }

    /// Choose what happens at the end of the loop — [`LoopMode::Wrap`] (cycle) or
    /// [`LoopMode::PingPong`] (turn around). Resets the ping-pong to travelling
    /// FORWARDS, so arming it never starts the animation running backwards.
    pub fn set_loop_mode(&mut self, mode: LoopMode) {
        self.loop_mode = mode;
        self.reset_ping_dir();
    }

    /// Send a ping-pong back to travelling FORWARDS — the derived state
    /// [`Self::set_loop_mode`] must reconcile, so arming the mode never starts the
    /// animation running backwards from wherever the last one happened to stop.
    fn reset_ping_dir(&mut self) {
        self.ping_dir = 1.0;
    }

    /// The current loop mode.
    #[must_use]
    pub fn loop_mode(&self) -> LoopMode {
        self.loop_mode
    }

    /// The current loop range `[start, end)` in seconds, if any.
    #[must_use]
    pub fn loop_range(&self) -> Option<(f64, f64)> {
        self.loop_range
    }

    /// Whether the armed loop travels back and forth ([`LoopMode::PingPong`]) — what a
    /// transport toggle mirrors when it reads THIS clock rather than a document.
    #[must_use]
    pub fn is_ping_pong(&self) -> bool {
        matches!(self.loop_mode, LoopMode::PingPong)
    }

    /// Start advancing.
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Stop advancing (position holds).
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Toggle play/pause; returns the new playing state.
    pub fn toggle_play(&mut self) -> bool {
        self.playing = !self.playing;
        self.playing
    }

    /// Whether the playhead is currently advancing.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Whether the play is currently moving the position FORWARD in time — playing,
    /// with an effective positive step (the `rate` sign times the ping-pong
    /// direction). A normal forward loop is forward even on the tick its position
    /// wraps back to the start; a ping-pong REVERSE leg and reverse playback
    /// (`rate < 0`) are not. Timeline signals fire only on forward motion
    /// (ADR-0143 §3): a signal is a forward-time event, like a footstep.
    #[must_use]
    pub fn is_advancing_forward(&self) -> bool {
        if !self.playing {
            return false;
        }
        let dir = if matches!(self.loop_mode, LoopMode::PingPong) {
            self.rate * self.ping_dir
        } else {
            self.rate
        };
        dir > 0.0
    }

    /// The playback-speed multiplier.
    #[must_use]
    pub fn rate(&self) -> f64 {
        self.rate
    }

    /// Set the playback-speed multiplier (finite values only; non-finite
    /// ignored).
    pub fn set_rate(&mut self, rate: f64) {
        if rate.is_finite() {
            self.rate = rate;
        }
    }

    /// The per-tick step in seconds at `rate == 1.0`.
    #[must_use]
    pub fn fixed_dt(&self) -> f64 {
        self.fixed_dt
    }
}

impl Default for Playhead {
    /// 60 Hz step, playing, at `t = 0`.
    fn default() -> Self {
        Self::new(1.0 / crate::time::DEFAULT_HZ)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f64 = 1.0 / 60.0;

    #[test]
    fn advances_by_fixed_dt_while_playing() {
        let mut p = Playhead::new(DT);
        p.advance();
        assert!((p.time() - DT).abs() < 1e-12);
        p.advance();
        assert!((p.time() - 2.0 * DT).abs() < 1e-12);
    }

    #[test]
    fn pause_freezes_position() {
        let mut p = Playhead::new(DT);
        p.advance();
        let held = p.time();
        p.pause();
        for _ in 0..10 {
            p.advance();
        }
        assert_eq!(p.time(), held);
        p.play();
        p.advance();
        assert!(p.time() > held);
    }

    #[test]
    fn seek_and_frame_roundtrip() {
        let mut p = Playhead::new(DT);
        p.seek_frame(24, 24.0);
        assert!((p.time() - 1.0).abs() < 1e-12);
        assert_eq!(p.frame(24.0), 24);
        p.seek(-5.0); // clamped to 0
        assert_eq!(p.time(), 0.0);
    }

    #[test]
    fn rate_scales_advance() {
        let mut p = Playhead::new(DT);
        p.set_rate(0.5);
        p.advance();
        assert!((p.time() - 0.5 * DT).abs() < 1e-12);
    }

    #[test]
    fn advance_is_deterministic_and_reproducible() {
        // Same sequence of transport ops → bit-identical position.
        let script = |p: &mut Playhead| {
            for i in 0..1000 {
                if i % 7 == 0 {
                    p.toggle_play();
                }
                if i % 13 == 0 {
                    p.set_rate(if i % 2 == 0 { 1.0 } else { 0.5 });
                }
                p.advance();
            }
        };
        let mut a = Playhead::new(DT);
        let mut b = Playhead::new(DT);
        script(&mut a);
        script(&mut b);
        assert_eq!(a.time().to_bits(), b.time().to_bits());
    }

    #[test]
    fn advance_ticks_matches_repeated_advance() {
        let mut a = Playhead::new(DT);
        let mut b = Playhead::new(DT);
        a.advance_ticks(5);
        for _ in 0..5 {
            b.advance();
        }
        assert_eq!(a.time().to_bits(), b.time().to_bits());
    }

    // ── W0.T3: loop range ────────────────────────────────────────────────────
    #[test]
    fn advance_wraps_within_loop() {
        let mut p = Playhead::new(DT);
        p.set_loop(0.0, 0.05); // ~3 ticks
        for _ in 0..200 {
            p.advance();
        }
        let t = p.time();
        assert!((0.0..0.05).contains(&t), "looped time {t} left [0, 0.05)");
    }

    #[test]
    fn loop_advance_is_deterministic() {
        let mut a = Playhead::new(DT);
        let mut b = Playhead::new(DT);
        a.set_loop(0.0, 0.05);
        b.set_loop(0.0, 0.05);
        for _ in 0..137 {
            a.advance();
            b.advance();
        }
        assert_eq!(a.time().to_bits(), b.time().to_bits());
    }

    #[test]
    fn set_loop_validates_and_clears() {
        let mut p = Playhead::new(DT);
        p.set_loop(1.0, 0.5); // end <= start → no loop
        assert_eq!(p.loop_range(), None);
        p.set_loop(-1.0, 2.0); // start clamped to 0
        assert_eq!(p.loop_range(), Some((0.0, 2.0)));
        p.clear_loop();
        assert_eq!(p.loop_range(), None);
    }

    #[test]
    fn loop_pulls_an_outside_position_into_range() {
        let mut p = Playhead::new(DT);
        p.seek(10.0);
        p.set_loop(1.0, 2.0);
        for _ in 0..600 {
            p.advance();
        }
        let t = p.time();
        assert!((1.0..2.0).contains(&t), "t={t} not in [1, 2)");
    }
}

#[cfg(test)]
mod ping_pong_tests {
    use super::*;

    /// Play `n` ticks and collect the position after each.
    fn run(ph: &mut Playhead, n: usize) -> Vec<f64> {
        (0..n)
            .map(|_| {
                ph.advance();
                ph.time()
            })
            .collect()
    }

    #[test]
    fn a_wrapping_loop_jumps_back_to_the_start() {
        let mut ph = Playhead::new(0.25);
        ph.set_loop(0.0, 1.0);
        ph.set_loop_mode(LoopMode::Wrap);
        // …0.75, then over the end -> back to 0.0.
        assert_eq!(run(&mut ph, 5), vec![0.25, 0.5, 0.75, 0.0, 0.25]);
    }

    #[test]
    fn a_ping_pong_turns_around_instead_of_jumping() {
        // THE difference: at the end it REVERSES, and the reflection is applied to
        // the OVERSHOOT — so it resumes exactly as far inside as it went beyond and
        // no frame is dropped.
        let mut ph = Playhead::new(0.25);
        ph.set_loop(0.0, 1.0);
        ph.set_loop_mode(LoopMode::PingPong);
        assert_eq!(
            run(&mut ph, 9),
            vec![0.25, 0.5, 0.75, 1.0, 0.75, 0.5, 0.25, 0.0, 0.25],
            "out to the end, back to the start, and out again"
        );
    }

    #[test]
    fn arming_a_ping_pong_always_starts_it_forwards() {
        // The direction is DERIVED state. Left over from a previous ping-pong, it
        // would start the animation running backwards the moment the toggle is
        // armed — which is why `set_loop_mode` reconciles it.
        let mut ph = Playhead::new(0.25);
        ph.set_loop(0.0, 1.0);
        ph.set_loop_mode(LoopMode::PingPong);
        run(&mut ph, 5); // …now travelling BACK (0.75)
        ph.set_loop_mode(LoopMode::PingPong); // re-arm
        assert_eq!(run(&mut ph, 1), vec![1.0], "forwards again, not backwards");
    }

    #[test]
    fn a_ping_pong_with_no_loop_range_just_plays_on() {
        let mut ph = Playhead::new(0.25);
        ph.set_loop_mode(LoopMode::PingPong);
        assert_eq!(
            run(&mut ph, 3),
            vec![0.25, 0.5, 0.75],
            "nothing to bounce off"
        );
    }
}
