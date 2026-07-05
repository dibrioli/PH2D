//! Equal-power stereo panning.

use std::f32::consts::FRAC_PI_2;

/// Constant-power stereo pan law.
///
/// `pan` is `-1.0` (hard left) … `0.0` (center) … `+1.0` (hard right). Returns
/// `[left_gain, right_gain]`. The two gains satisfy `l² + r² == 1`, so the
/// perceived loudness is constant across the stereo field (center is −3 dB per
/// channel, not unity — the pro default). Presentation math (HR-5 exempt).
#[inline]
pub fn equal_power_pan(pan: f32) -> [f32; 2] {
    // Map [-1, 1] → [0, π/2]; cos falls left→right, sin rises.
    let angle = (pan.clamp(-1.0, 1.0) * 0.5 + 0.5) * FRAC_PI_2;
    [angle.cos(), angle.sin()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn power(g: [f32; 2]) -> f32 {
        g[0] * g[0] + g[1] * g[1]
    }

    #[test]
    fn center_is_balanced_and_power_preserving() {
        let c = equal_power_pan(0.0);
        assert!((c[0] - c[1]).abs() < 1e-6, "center must be balanced");
        assert!((power(c) - 1.0).abs() < 1e-5, "constant power across pan");
    }

    #[test]
    fn hard_sides_route_fully() {
        let l = equal_power_pan(-1.0);
        let r = equal_power_pan(1.0);
        assert!((l[0] - 1.0).abs() < 1e-6 && l[1].abs() < 1e-6);
        assert!(r[0].abs() < 1e-6 && (r[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn power_is_constant_everywhere() {
        for i in -10..=10 {
            let p = i as f32 / 10.0;
            assert!((power(equal_power_pan(p)) - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn clamps_out_of_range() {
        assert_eq!(equal_power_pan(-4.0), equal_power_pan(-1.0));
        assert_eq!(equal_power_pan(4.0), equal_power_pan(1.0));
    }
}
