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
#[derive(Debug, Clone, Copy)]
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
