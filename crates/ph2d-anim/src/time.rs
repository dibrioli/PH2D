//! [`RationalTime`] — drift-free time storage (OpenTimelineIO pattern).
//!
//! Time is stored as an integer numerator over an integer denominator so that
//! frame times are represented *exactly* (`from_frame(1, 24)` is `1/24`, not a
//! rounded `f64`) and multi-hour timelines never accumulate float drift. `f64`
//! appears only at the boundary, via [`RationalTime::to_seconds`], where a
//! sampler needs a scalar.
//!
//! This is deliberately **minimal** — only what [`crate::Track`] / [`crate::Clip`]
//! need (construct, compare, convert to seconds). It is *not* a re-implementation
//! of OTIO's full rational arithmetic.

use core::cmp::Ordering;
use core::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

/// Denominator used by [`RationalTime::from_seconds`]: microsecond resolution.
///
/// `from_seconds` is documented as *approximate* — it snaps a real number to
/// the nearest microsecond. Frame-accurate times must be built with
/// [`RationalTime::from_frame`], which is exact.
const SECONDS_DENOM: u32 = 1_000_000;

/// A point in time as an exact rational number of seconds (`num / den`).
///
/// The denominator is kept `>= 1` (a zero denominator is meaningless and is
/// clamped to `1` at construction). Equality and ordering are by *normalized
/// value*: `2/4` equals `1/2`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RationalTime {
    num: i64,
    den: u32,
}

impl RationalTime {
    /// The zero instant (`0/1`).
    pub const ZERO: Self = Self { num: 0, den: 1 };

    /// Construct from an explicit numerator and denominator.
    ///
    /// A `den` of `0` is clamped to `1` (documented; callers should not pass 0).
    #[must_use]
    pub const fn new(num: i64, den: u32) -> Self {
        Self {
            num,
            den: if den == 0 { 1 } else { den },
        }
    }

    /// Construct an **exact** time from a frame index at a given frame rate.
    ///
    /// `from_frame(f, fps)` is exactly `f / fps` seconds — no rounding.
    #[must_use]
    pub fn from_frame(frame: i64, fps: u32) -> Self {
        debug_assert!(fps > 0, "fps must be positive");
        Self::new(frame, fps)
    }

    /// Construct an **approximate** time from a floating-point number of seconds.
    ///
    /// The value is snapped to the nearest microsecond (see [`SECONDS_DENOM`]).
    /// For frame-accurate work use [`RationalTime::from_frame`].
    #[must_use]
    pub fn from_seconds(secs: f64) -> Self {
        let num = (secs * f64::from(SECONDS_DENOM)).round() as i64;
        Self {
            num,
            den: SECONDS_DENOM,
        }
    }

    /// Convert to seconds as an `f64`. This is the single boundary where the
    /// rational representation becomes an approximate scalar.
    #[must_use]
    pub fn to_seconds(self) -> f64 {
        self.num as f64 / f64::from(self.den)
    }

    /// The stored numerator.
    #[must_use]
    pub const fn num(self) -> i64 {
        self.num
    }

    /// The stored denominator (always `>= 1`).
    #[must_use]
    pub const fn den(self) -> u32 {
        self.den
    }

    /// Shared body of the `Add` / `Sub` impls: put both times over their lowest
    /// common denominator, combine the numerators, then reduce.
    fn combine(self, rhs: Self, negate_rhs: bool) -> Self {
        let (da, db) = (u128::from(self.den), u128::from(rhs.den));
        let lcm = da / gcd(da, db) * db;
        // Negate in i128 so `i64::MIN` cannot overflow on the way in.
        let rhs_num = if negate_rhs {
            -i128::from(rhs.num)
        } else {
            i128::from(rhs.num)
        };
        // `lcm / d` is a small integer; the products stay far inside i128.
        let num = i128::from(self.num) * (lcm / da) as i128 + rhs_num * (lcm / db) as i128;
        reduce(num, lcm)
    }
}

/// Greatest common divisor (Euclid). `gcd(x, 0) == x`; never returns `0`.
fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

/// Reduce `num / den` to lowest terms and fit it back into `(i64, u32)`.
///
/// Denominators reachable from [`RationalTime::from_frame`] (frame rates) and
/// [`RationalTime::from_seconds`] (`1_000_000`) always fit after reduction; the
/// fallback exists so a pathological pair degrades to the documented microsecond
/// approximation instead of panicking.
fn reduce(num: i128, den: u128) -> RationalTime {
    if num == 0 {
        return RationalTime::ZERO;
    }
    let g = gcd(num.unsigned_abs(), den);
    let (mag, small_den) = (num.unsigned_abs() / g, den / g);
    match (i64::try_from(mag), u32::try_from(small_den)) {
        (Ok(mag), Ok(den)) => RationalTime::new(if num < 0 { -mag } else { mag }, den),
        _ => RationalTime::from_seconds(num as f64 / den as f64),
    }
}

/// Exact sum: `from_frame(1, 24) + from_frame(1, 24) == from_frame(2, 24)`.
///
/// Going through [`RationalTime::to_seconds`] instead would round the result to
/// the nearest microsecond, so a key offset by two frames at 24 fps would land
/// at `83333 µs` rather than exactly `2/24` — close enough to look right, far
/// enough to fail an equality test against a frame-exact key. That is the whole
/// reason this type is rational.
impl Add for RationalTime {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.combine(rhs, false)
    }
}

/// Exact difference (see the [`Add`] impl).
impl Sub for RationalTime {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.combine(rhs, true)
    }
}

impl PartialEq for RationalTime {
    fn eq(&self, other: &Self) -> bool {
        // Cross-multiply in i128 (denominators are always positive).
        i128::from(self.num) * i128::from(other.den) == i128::from(other.num) * i128::from(self.den)
    }
}

impl Eq for RationalTime {}

impl PartialOrd for RationalTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RationalTime {
    fn cmp(&self, other: &Self) -> Ordering {
        // Denominators are positive, so cross-multiplication preserves sign.
        (i128::from(self.num) * i128::from(other.den))
            .cmp(&(i128::from(other.num) * i128::from(self.den)))
    }
}

impl Default for RationalTime {
    fn default() -> Self {
        Self::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_frames_stays_frame_exact() {
        let two_frames = RationalTime::from_frame(2, 24);
        let sum = RationalTime::from_frame(0, 24) + two_frames;
        assert_eq!(sum, two_frames);
        // The lossy route lands 1/3 of a microsecond away — enough to miss.
        assert_ne!(
            RationalTime::from_seconds(two_frames.to_seconds()),
            two_frames,
            "from_seconds() quantizes to microseconds; add() must not"
        );
    }

    #[test]
    fn add_and_sub_are_exact_across_denominators() {
        // 1/24 + 1/30 = 5/120 + 4/120 = 9/120 = 3/40.
        let sum = RationalTime::from_frame(1, 24) + RationalTime::from_frame(1, 30);
        assert_eq!(sum, RationalTime::new(3, 40));
        assert_eq!((sum.num(), sum.den()), (3, 40), "reduced to lowest terms");
        assert_eq!(
            sum - RationalTime::from_frame(1, 30),
            RationalTime::from_frame(1, 24),
            "sub undoes add"
        );
    }

    #[test]
    fn sub_of_equal_times_is_exactly_zero() {
        let t = RationalTime::from_frame(7, 30);
        assert_eq!(t - t, RationalTime::ZERO);
        assert_eq!((t - t).num(), 0);
        // Mixed representations of the same instant, too.
        let half = RationalTime::from_frame(12, 24);
        assert_eq!(
            half.sub(RationalTime::from_seconds(0.5)),
            RationalTime::ZERO
        );
    }

    #[test]
    fn sub_can_go_negative() {
        let d = RationalTime::from_frame(1, 24) - RationalTime::from_frame(3, 24);
        assert_eq!(d, RationalTime::new(-1, 12));
        assert!(d.to_seconds() < 0.0);
    }

    #[test]
    fn adding_a_negative_delta_walks_back_to_the_start() {
        let t = RationalTime::from_frame(5, 60);
        let back = t + RationalTime::from_frame(-5, 60);
        assert_eq!(back, RationalTime::ZERO);
    }
}
