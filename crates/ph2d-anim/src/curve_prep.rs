//! **Channel-aware sample preparation** — everything the F-curve fit
//! ([`crate::curve_fit`]) must know about a recorded channel BEYOND its raw
//! numbers, applied to the dense samples before they are fitted.
//!
//! A record-during-play session hands the fit a column of `(time, value)` pairs
//! and nothing else. That is enough for a plain scalar, but two kinds of channel
//! carry structure the numbers alone do not show, and a fit that ignores it
//! produces a curve that is wrong in a way no tolerance can catch:
//!
//! 1. **Angles wrap.** The rotate gizmo writes
//!    `start + (atan2(now) − atan2(start))` (`ph2d-editor-core`'s
//!    `gizmo::transform`), and both `atan2` results live in `(−π, π]` — so
//!    `Transform.rotation` jumps by 2π the instant the cursor crosses the branch
//!    cut. On screen that is invisible (rotation is mod 2π), but the recorded
//!    track is a SAWTOOTH, and a fit through it reconstructs a two-turn spin as a
//!    net rotation of ~zero. [`unwrap_angles`] adds back the 2π steps, which
//!    recovers the true continuous spin exactly: the jump is exactly 2π and a
//!    hand gesture never turns half a revolution between two frames.
//! 2. **Some channels are bounded.** Opacity lives in `[0, 1]`; a least-squares
//!    cubic through samples that settle ON the bound happily overshoots past it.
//!    The bound travels with the channel ([`FitChannel::bounds`]) and the fit
//!    clamps its control points to it (convex hull ⇒ the whole curve obeys).
//!
//! # Not here: a corner pre-pass
//!
//! The third thing the fit could use — keeping a genuine CUSP (a bounce, an
//! impact) out of the tremor low-pass, which currently rounds it and costs the
//! apex ~2.8% of the recording's range — is **deliberately absent**, and the
//! reason is measured rather than assumed. Four detectors were built and all four
//! fabricate corners in the middle of SMOOTH gestures once the input carries
//! realistic hand tremor (~2% of range): over 200 noise seeds the best of them
//! fired on 100% of smooth recordings, about a dozen phantom cusps each. The cause
//! is not tuning — at the sample scale a fast smooth gesture and a cusp differ
//! only in ways tremor masks, and the noise estimate that would unmask them is
//! itself inflated by fast motion.
//!
//! A phantom corner pins a key and breaks a tangent inside a smooth curve; the
//! rounding it would prevent sits INSIDE the ±1–3% fidelity the fit already
//! declares (and that was signed off). Shipping it would trade an accepted
//! approximation for a visible regression, so it does not ship. See the line's
//! handoff for the data and the two approaches worth trying next.
//!
//! Transcendental-free (HR-5): the 2π constants only — no `sin`/`cos` to unwrap.

use crate::curve_fit::smooth_values;
use std::f64::consts::{PI, TAU};

/// What kind of quantity a recorded channel carries — the structure the raw
/// `(time, value)` samples do not show. The channel's owner supplies it (in this
/// workspace: `ph2d_timeline::PropKind::fit_channel`), so the fit stays a pure
/// numeric routine that knows nothing about sprites.
///
/// Extend by adding a FIELD (append-only, defaults preserve today's behaviour) —
/// never by adding a variant a caller must match on.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FitChannel {
    /// The value is an angle in radians: unwrap the ±2π sawtooth before fitting
    /// ([`unwrap_angles`]).
    pub angular: bool,
    /// Hard bounds the fitted curve may not leave, in the channel's own units.
    /// `None` = unbounded (translation, scale, a raw scalar).
    pub bounds: Option<(f64, f64)>,
}

impl FitChannel {
    /// A plain unbounded scalar — translation, scale. No preparation.
    pub const LINEAR: Self = Self {
        angular: false,
        bounds: None,
    };

    /// An angle in radians — rotation. Unwrapped before the fit.
    pub const ANGLE: Self = Self {
        angular: true,
        bounds: None,
    };

    /// A scalar the fitted curve may not take outside `[lo, hi]` — opacity is
    /// `bounded(0.0, 1.0)`.
    #[must_use]
    pub const fn bounded(lo: f64, hi: f64) -> Self {
        Self {
            angular: false,
            bounds: Some((lo, hi)),
        }
    }
}

/// Prepare `samples` (sorted by time) for the fit, per the channel's semantics:
/// unwrap the angle sawtooth, then low-pass the tremor out
/// ([`crate::smooth_values`]).
///
/// The unwrap runs FIRST and it matters: on the wrapped signal every 2π jump looks
/// like a colossal spike, and the low-pass would smear each one across the frames
/// around it, corrupting the very samples the unwrap needs to read.
pub fn prepare(samples: &mut [(f64, f64)], channel: FitChannel, smooth_passes: usize) {
    if channel.angular {
        unwrap_angles(samples);
    }
    smooth_values(samples, smooth_passes);
}

/// Undo the ±2π wrap of an angle channel, in place: add the 2π steps back so a
/// multi-turn spin reads as the continuous angle it really was
/// (`… 3.10, 3.14, −3.14, −3.10 …` → `… 3.10, 3.14, 3.18, 3.22 …`).
///
/// The reconstruction is EXACT for any gesture that turns less than half a
/// revolution between two samples — at 60 fps that is 30 revolutions per second,
/// far past what a hand does. Faster than that, no unwrap can tell a forward turn
/// from a backward one (the sampling is below Nyquist) and the shorter path wins,
/// which is the standard, documented behaviour of every unwrap.
///
/// Values stay unwrapped afterwards — a key may hold 12.57 rad (two turns), and
/// the scene reads it mod 2π. That is the point: it is what makes the spin
/// replay as a spin instead of snapping back.
pub fn unwrap_angles(samples: &mut [(f64, f64)]) {
    let mut offset = 0.0f64;
    for i in 1..samples.len() {
        let prev = samples[i - 1].1; // already unwrapped
        let step = samples[i].1 + offset - prev;
        if step > PI {
            offset -= TAU;
        } else if step < -PI {
            offset += TAU;
        }
        samples[i].1 += offset;
    }
}

#[cfg(test)]
#[path = "curve_prep_tests.rs"]
mod tests;
