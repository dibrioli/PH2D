//! Easing presets — an internal copy of the classic Penner / `simple_easing`
//! equations.
//!
//! # Attribution
//!
//! The formulas below are **behaviourally ported** (re-implemented in Rust from
//! the published equations) from the `simple_easing` crate (MIT,
//! <https://crates.io/crates/simple_easing>) and the canonical Robert Penner
//! easing set (BSD/MIT). No third-party code is linked — this is an internal
//! copy per the PH2D stack-discipline rule (SKILL §5: no gratuitous crates.io
//! deps) and the DIRETIVA rule "port the reference algorithm before writing
//! your own".
//!
//! # HR-5 / determinism
//!
//! Animation is *presentation* and is therefore exempt from the HR-5
//! transcendental membrane (ADR-0030). Even so, the flag exists so a future
//! **gameplay** consumer can reject the non-deterministic presets, mirroring
//! `ph2d_expr`'s `Func::is_deterministic()`:
//!
//! - Polynomial families (`Linear`, `Quad`, `Cubic`, `Quart`, `Quint`, `Back`,
//!   `Bounce`) are transcendental-free → [`Easing::is_deterministic`] `== true`.
//! - Transcendental families (`Sine`, `Expo`, `Circ`, `Elastic`) use
//!   `sin`/`cos`/`exp2`/`sqrt` → [`Easing::is_deterministic`] `== false`.
//!
//! `Out` and `InOut` shapes are the mathematical reflections of the `In` form
//! (`out(u) = 1 - in(1 - u)`), so every preset satisfies `e(0) == 0` and
//! `e(1) == 1`.

use core::f64::consts::PI;

use serde::{Deserialize, Serialize};

/// The shape of an easing curve, independent of direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EasingFamily {
    /// Identity (`u`). Ignores the [`EasingMode`].
    Linear,
    /// Quadratic (`u²`).
    Quad,
    /// Cubic (`u³`).
    Cubic,
    /// Quartic (`u⁴`).
    Quart,
    /// Quintic (`u⁵`).
    Quint,
    /// Anticipation/overshoot (polynomial "back" ease).
    Back,
    /// Damped bounce (piecewise polynomial).
    Bounce,
    /// Sinusoidal — **transcendental** (`cos`).
    Sine,
    /// Exponential — **transcendental** (`exp2`).
    Expo,
    /// Circular — **transcendental** (`sqrt`).
    Circ,
    /// Elastic spring — **transcendental** (`exp2` + `sin`).
    Elastic,
}

impl EasingFamily {
    /// Every family, for iteration (preset pickers, property tests).
    pub const ALL: [EasingFamily; 11] = [
        EasingFamily::Linear,
        EasingFamily::Quad,
        EasingFamily::Cubic,
        EasingFamily::Quart,
        EasingFamily::Quint,
        EasingFamily::Back,
        EasingFamily::Bounce,
        EasingFamily::Sine,
        EasingFamily::Expo,
        EasingFamily::Circ,
        EasingFamily::Elastic,
    ];

    /// `true` if the family is transcendental-free (polynomial).
    #[must_use]
    pub const fn is_deterministic(self) -> bool {
        matches!(
            self,
            EasingFamily::Linear
                | EasingFamily::Quad
                | EasingFamily::Cubic
                | EasingFamily::Quart
                | EasingFamily::Quint
                | EasingFamily::Back
                | EasingFamily::Bounce
        )
    }
}

/// The direction an easing family is applied in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EasingMode {
    /// Accelerate from rest (the family's base form).
    In,
    /// Decelerate to rest (reflection of `In`).
    Out,
    /// Accelerate then decelerate (symmetric split).
    InOut,
}

impl EasingMode {
    /// Every mode, for iteration.
    pub const ALL: [EasingMode; 3] = [EasingMode::In, EasingMode::Out, EasingMode::InOut];
}

/// An easing preset: a [`EasingFamily`] plus an [`EasingMode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Easing {
    /// The curve shape.
    pub family: EasingFamily,
    /// The direction.
    pub mode: EasingMode,
}

impl Easing {
    /// The identity easing (`Linear`, `In`) — `eval(u) == u`.
    pub const LINEAR: Self = Self {
        family: EasingFamily::Linear,
        mode: EasingMode::In,
    };

    /// Construct a preset from a family and mode.
    #[must_use]
    pub const fn new(family: EasingFamily, mode: EasingMode) -> Self {
        Self { family, mode }
    }

    /// `true` if this preset is transcendental-free (polynomial).
    #[must_use]
    pub const fn is_deterministic(self) -> bool {
        self.family.is_deterministic()
    }

    /// Evaluate the eased fraction for a normalized input `u ∈ [0, 1]`.
    ///
    /// The input is clamped to `[0, 1]`. The output passes through `(0, 0)` and
    /// `(1, 1)`; `Back`/`Elastic` overshoot in between (output can leave
    /// `[0, 1]`), which is intentional.
    #[must_use]
    pub fn eval(self, u: f64) -> f64 {
        let u = u.clamp(0.0, 1.0);
        if matches!(self.family, EasingFamily::Linear) {
            return u;
        }
        match self.mode {
            EasingMode::In => base_in(self.family, u),
            EasingMode::Out => 1.0 - base_in(self.family, 1.0 - u),
            EasingMode::InOut => {
                if u < 0.5 {
                    base_in(self.family, 2.0 * u) / 2.0
                } else {
                    1.0 - base_in(self.family, 2.0 * (1.0 - u)) / 2.0
                }
            }
        }
    }
}

/// The `In` (base) form of each family for `u ∈ [0, 1]`.
///
/// `float_cmp` is allowed only for the exact-endpoint guards on the
/// transcendental families (`Expo`/`Elastic`), where snapping to `0`/`1` at the
/// boundary is the canonical, intended behaviour.
#[allow(clippy::float_cmp)]
fn base_in(family: EasingFamily, u: f64) -> f64 {
    match family {
        EasingFamily::Linear => u,
        EasingFamily::Quad => u * u,
        EasingFamily::Cubic => u * u * u,
        EasingFamily::Quart => u * u * u * u,
        EasingFamily::Quint => u * u * u * u * u,
        EasingFamily::Back => {
            const C1: f64 = 1.701_58;
            const C3: f64 = C1 + 1.0;
            C3 * u * u * u - C1 * u * u
        }
        EasingFamily::Bounce => 1.0 - bounce_out(1.0 - u),
        EasingFamily::Sine => 1.0 - (u * PI / 2.0).cos(),
        EasingFamily::Expo => {
            if u == 0.0 {
                0.0
            } else {
                (10.0 * u - 10.0).exp2()
            }
        }
        EasingFamily::Circ => 1.0 - (1.0 - u * u).max(0.0).sqrt(),
        EasingFamily::Elastic => {
            if u == 0.0 {
                0.0
            } else if u == 1.0 {
                1.0
            } else {
                const C4: f64 = (2.0 * PI) / 3.0;
                -(10.0 * u - 10.0).exp2() * ((u * 10.0 - 10.75) * C4).sin()
            }
        }
    }
}

/// The canonical `Bounce` `Out` form for `u ∈ [0, 1]` (the other bounce shapes
/// derive from this by reflection).
fn bounce_out(u: f64) -> f64 {
    const N1: f64 = 7.5625;
    const D1: f64 = 2.75;
    if u < 1.0 / D1 {
        N1 * u * u
    } else if u < 2.0 / D1 {
        let u = u - 1.5 / D1;
        N1 * u * u + 0.75
    } else if u < 2.5 / D1 {
        let u = u - 2.25 / D1;
        N1 * u * u + 0.9375
    } else {
        let u = u - 2.625 / D1;
        N1 * u * u + 0.984_375
    }
}
