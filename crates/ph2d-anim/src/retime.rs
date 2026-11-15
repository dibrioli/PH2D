//! [`Retime`] — the RHYTHM of one segment, apart from its easing (plano 18, dono 15/11).
//!
//! A segment `k0 → k1` plays `lerp(v0, v1, E(u))`, `E` the key's [`crate::Interp`]. A retime is a
//! warp `W: [0,1] → [0,1]` applied BEFORE the easing: `lerp(v0, v1, E(W(u)))`. Both keys, both
//! instants and the whole drawing of `E` (every bounce of a Bounce) stay; only WHEN each part of it
//! happens moves. `W` is a CSS cubic bézier from `(0,0)` to `(1,1)` with all four control
//! coordinates in `[0,1]` — which makes it monotone (non-decreasing): with `a = y1`, `c = 1 − y2`,
//! `W'` is a Bernstein quadratic whose middle coefficient `y2 − y1 = 1 − a − c ≥ −ac ≥ −√(ac)`, so
//! it never goes negative. Time never runs backwards inside a retimed segment.
//!
//! It lives on the [`crate::Key`] (`Option<Retime>`, `None` = the identity, never evaluated: a
//! segment without a retime samples byte-identical to the engine before it), not inside the
//! `Interp`, which is `Copy` and at its 216 B ceiling.

use serde::{Deserialize, Serialize};

use crate::curve_bezier::{bezier_slope, solve_cubic_bezier};

/// The two control points of a segment's time warp `W`, in the unit square. See the module docs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Retime {
    /// Out handle, time axis.
    pub x1: f32,
    /// Out handle, warped-time axis.
    pub y1: f32,
    /// In handle, time axis.
    pub x2: f32,
    /// In handle, warped-time axis.
    pub y2: f32,
}

impl Retime {
    /// The control points of `W(u) = u` — where a segment with no retime draws its rhythm handles.
    pub const LINEAR: Retime = Retime {
        x1: 1.0 / 3.0,
        y1: 1.0 / 3.0,
        x2: 2.0 / 3.0,
        y2: 2.0 / 3.0,
    };

    /// A warp with every coordinate clamped into `[0,1]` (the monotonicity condition) and a
    /// non-finite one replaced by its [`Self::LINEAR`] counterpart.
    #[must_use]
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let l = Self::LINEAR;
        let c = |v: f32, d: f32| if v.is_finite() { v.clamp(0.0, 1.0) } else { d }; // CLAMP-OK: unit square, NaN handled
        Self {
            x1: c(x1, l.x1),
            y1: c(y1, l.y1),
            x2: c(x2, l.x2),
            y2: c(y2, l.y2),
        }
    }

    /// `W(u)`, `u` clamped to `[0,1]`. Polynomial (the same Newton/bisection solve as
    /// [`crate::Interp::Bezier`]), so deterministic.
    #[must_use]
    pub fn eval(self, u: f64) -> f64 {
        let [x1, y1, x2, y2] = self.f64s();
        solve_cubic_bezier(x1, y1, x2, y2, u.clamp(0.0, 1.0)) // CLAMP-OK: normalized u
    }

    /// `W'(u)` — the rate the easing is played at. `≥ 0`; `+∞` only at a vertical end tangent.
    #[must_use]
    pub fn slope(self, u: f64) -> f64 {
        let [x1, y1, x2, y2] = self.f64s();
        bezier_slope(x1, y1, x2, y2, u.clamp(0.0, 1.0)) // CLAMP-OK: normalized u
    }

    /// The warp of the same segment played backwards: `W_rev(u) = 1 − W(1 − u)`, the reflection
    /// through `(½, ½)` with the handles swapped (as [`crate::Interp::reversed`] does a bézier).
    #[must_use]
    pub fn reversed(self) -> Self {
        Self {
            x1: 1.0 - self.x2,
            y1: 1.0 - self.y2,
            x2: 1.0 - self.x1,
            y2: 1.0 - self.y1,
        }
    }

    fn f64s(self) -> [f64; 4] {
        [self.x1, self.y1, self.x2, self.y2].map(f64::from)
    }
}

/// `W(u)` for an optional warp — `u` itself, untouched, when there is none.
#[inline]
#[must_use]
pub fn warp(retime: Option<Retime>, u: f64) -> f64 {
    match retime {
        Some(w) => w.eval(u),
        None => u,
    }
}

/// `W'(u)` for an optional warp — exactly `1` when there is none.
#[inline]
#[must_use]
pub fn warp_slope(retime: Option<Retime>, u: f64) -> f64 {
    match retime {
        Some(w) => w.slope(u),
        None => 1.0,
    }
}
