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
        }
    }

    /// Create a playhead whose per-tick step matches a [`FixedStep`] clock.
    #[must_use]
    pub fn from_fixed_step(step: &FixedStep) -> Self {
        Self::new(step.fixed_dt())
    }

    /// Advance the position by one fixed tick (`fixed_dt * rate`) if playing.
    /// Call this once per simulation tick. The position never goes below `0`.
    pub fn advance(&mut self) {
        if self.playing {
            self.time = (self.time + self.fixed_dt * self.rate).max(0.0);
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
}
