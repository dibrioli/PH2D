//! `atan2` without a transcendental — the Rajan rational approximation of `atan` on `[0,1]`, folded
//! across the eight octants (~0.0015 rad error, multiply/add/compare only, HR-5). A leaf-local copy
//! of `motion.look_at`'s: the shared vocabulary is the BEHAVIOUR, not a shared symbol.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

/// `atan2(y, x)` in radians. Returns 0 at the origin.
pub(crate) fn atan2_approx(y: f32, x: f32) -> f32 {
    let (ax, ay) = (x.abs(), y.abs());
    let hi = ax.max(ay);
    if hi == 0.0 {
        return 0.0;
    }
    let a = ax.min(ay) / hi;
    let mut r = FRAC_PI_4 * a - a * (a - 1.0) * (0.2447 + 0.0663 * a);
    if ay > ax {
        r = FRAC_PI_2 - r;
    }
    if x < 0.0 {
        r = PI - r;
    }
    if y < 0.0 {
        r = -r;
    }
    r
}

/// Radians → degrees. The app authors angles in **degrees** — the one authored-angle unit — and the
/// `rot` column is in them.
pub(crate) fn deg(rad: f32) -> f32 {
    rad * (180.0 / PI)
}
