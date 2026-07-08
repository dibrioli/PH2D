//! Easing curves for `motion.stagger` — the standard Penner set **minus** the
//! families that need `sin`/`exp`/`pow` (Sine/Expo/Elastic are omitted so the
//! whole table stays **transcendental-free**, HR-5). A curve **family** is shaped
//! by a **direction** (In / Out / In-Out); every family/direction is endpoint-exact
//! (`0→0`, `1→1`).
//!
//! Family index (`ease_curve`): 0 Linear · 1 Quad · 2 Cubic · 3 Quart · 4 Quint ·
//! 5 Circ (sqrt) · 6 Back (overshoot) · 7 Bounce. Direction (`ease_dir`): 0 In ·
//! 1 Out · 2 In-Out. Linear ignores the direction.

/// The `easeIn` base shape for a family on `t ∈ [0,1]` (monotonic, accelerating).
/// Out / In-Out are derived from this by reflection in [`ease`].
fn ease_in(curve: i32, t: f32) -> f32 {
    match curve {
        1 => t * t,                               // Quad
        2 => t * t * t,                           // Cubic
        3 => t * t * t * t,                       // Quart
        4 => t * t * t * t * t,                   // Quint
        5 => 1.0 - (1.0 - t * t).max(0.0).sqrt(), // Circ (sqrt is HR-5-safe)
        6 => {
            // Back (overshoot) — the standard c1/c3 cubic.
            const C1: f32 = 1.70158;
            const C3: f32 = C1 + 1.0;
            C3 * t * t * t - C1 * t * t
        }
        7 => 1.0 - bounce_out(1.0 - t), // Bounce (base derived from easeOutBounce)
        _ => t,                         // Linear (family 0 / unknown)
    }
}

/// Canonical `easeOutBounce` — four piecewise parabolas (transcendental-free).
fn bounce_out(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / D1;
        N1 * t * t + 0.984375
    }
}

/// A full easing: the `curve` family shaped by `dir` (In / Out / In-Out) on
/// `t ∈ [0,1]`. Endpoint-exact for every family/direction; Linear ignores `dir`.
pub(crate) fn ease(curve: i32, dir: i32, t: f32) -> f32 {
    if curve == 0 {
        return t; // Linear
    }
    match dir {
        1 => 1.0 - ease_in(curve, 1.0 - t), // Out = reflect the In base
        2 => {
            // In-Out = In on the first half, reflected Out on the second.
            if t < 0.5 {
                ease_in(curve, 2.0 * t) * 0.5
            } else {
                1.0 - ease_in(curve, 2.0 - 2.0 * t) * 0.5
            }
        }
        _ => ease_in(curve, t), // In (0 / unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_family_and_direction_is_endpoint_exact() {
        for curve in 0..=7 {
            for dir in 0..=2 {
                assert_eq!(ease(curve, dir, 0.0), 0.0, "ease({curve},{dir},0)");
                assert_eq!(ease(curve, dir, 1.0), 1.0, "ease({curve},{dir},1)");
            }
        }
    }

    #[test]
    fn in_out_is_symmetric_at_the_midpoint() {
        // Every In-Out family crosses 0.5 exactly at t = 0.5.
        for curve in 1..=7 {
            assert!(
                (ease(curve, 2, 0.5) - 0.5).abs() < 1e-6,
                "midpoint curve {curve}"
            );
        }
    }

    #[test]
    fn back_overshoots_below_zero_on_ease_in() {
        // Back's signature: it dips below 0 early (anticipation) before rising.
        assert!(ease(6, 0, 0.2) < 0.0, "Back In anticipates below 0");
    }

    #[test]
    fn quad_in_matches_the_reference_t_squared() {
        // Numerically identical to the reference's easeIn (t²) — a fidelity pin.
        assert!((ease(1, 0, 0.5) - 0.25).abs() < 1e-6);
        assert!((ease(1, 1, 0.5) - 0.75).abs() < 1e-6); // Out = t(2-t) at 0.5
    }
}
