//! Math + color-space helpers behind [`super::ColorRamp`]: scalar / cubic interpolation, the
//! Catmull–Rom & B-spline bases, hue-arc interpolation, and the RGB↔HSV / RGB↔HSL conversions
//! (operating on linear RGB floats, hue in `[0, 1)`).

use super::RampHue;

pub(super) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub(super) fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
        lerp(a[3], b[3], t),
    ]
}

pub(super) fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Weighted sum of four scalars, clamped to `[0, 1]`.
pub(super) fn cub(w: [f32; 4], a: f32, b: f32, c: f32, d: f32) -> f32 {
    (w[0] * a + w[1] * b + w[2] * c + w[3] * d).clamp(0.0, 1.0)
}

/// Catmull–Rom basis weights for the four control points around parameter `t ∈ [0, 1]`.
pub(super) fn catmull_rom_weights(t: f32) -> [f32; 4] {
    let (t2, t3) = (t * t, t * t * t);
    [
        -0.5 * t3 + t2 - 0.5 * t,
        1.5 * t3 - 2.5 * t2 + 1.0,
        -1.5 * t3 + 2.0 * t2 + 0.5 * t,
        0.5 * t3 - 0.5 * t2,
    ]
}

/// Uniform cubic B-spline basis weights for `t ∈ [0, 1]`.
pub(super) fn bspline_weights(t: f32) -> [f32; 4] {
    let (t2, t3) = (t * t, t * t * t);
    let s = 1.0 / 6.0;
    [
        s * (-t3 + 3.0 * t2 - 3.0 * t + 1.0),
        s * (3.0 * t3 - 6.0 * t2 + 4.0),
        s * (-3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0),
        s * t3,
    ]
}

/// Interpolate hue `h1→h2` (both `[0,1)`) along the chosen [`RampHue`] arc; result wrapped to `[0,1)`.
pub(super) fn lerp_hue(h1: f32, h2: f32, fac: f32, mode: RampHue) -> f32 {
    let mut d = h2 - h1; // raw difference in (-1, 1)
    match mode {
        RampHue::Near => {
            if d > 0.5 {
                d -= 1.0;
            } else if d < -0.5 {
                d += 1.0;
            }
        }
        RampHue::Far => {
            if (0.0..0.5).contains(&d) {
                d -= 1.0;
            } else if (-0.5..=0.0).contains(&d) {
                d += 1.0;
            }
        }
        RampHue::Ccw => {
            if d < 0.0 {
                d += 1.0;
            }
        }
        RampHue::Cw => {
            if d > 0.0 {
                d -= 1.0;
            }
        }
    }
    (h1 + d * fac).rem_euclid(1.0)
}

/// Unwrap four hues so consecutive values stay within half a turn of the previous (Near path), so a
/// cubic blend across them doesn't jump across the `1.0→0.0` seam.
pub(super) fn unwrap_hues(mut h: [f32; 4]) -> [f32; 4] {
    for i in 1..4 {
        let mut d = h[i] - h[i - 1];
        if d > 0.5 {
            d -= 1.0;
        } else if d < -0.5 {
            d += 1.0;
        }
        h[i] = h[i - 1] + d;
    }
    h
}

/// `(max, min, max-min)` of the RGB channels — the chroma helper for both HSV and HSL.
fn rgb_range(c: [f32; 4]) -> (f32, f32, f32) {
    let max = c[0].max(c[1]).max(c[2]);
    let min = c[0].min(c[1]).min(c[2]);
    (max, min, max - min)
}

/// Hue of an RGB color in `[0,1)` (0 for greys).
fn rgb_hue(c: [f32; 4], max: f32, delta: f32) -> f32 {
    if delta <= 0.0 {
        return 0.0;
    }
    let (r, g, b) = (c[0], c[1], c[2]);
    let h = if max == r {
        (g - b) / delta + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    h / 6.0
}

pub(super) fn rgb_to_hsv(c: [f32; 4]) -> (f32, f32, f32) {
    let (max, _min, delta) = rgb_range(c);
    let s = if max <= 0.0 { 0.0 } else { delta / max };
    (rgb_hue(c, max, delta), s, max)
}

pub(super) fn hsv_to_rgba(h: f32, s: f32, v: f32, a: f32) -> [f32; 4] {
    let h = h.rem_euclid(1.0) * 6.0;
    let i = h.floor();
    let f = h - i;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    let (r, g, b) = match i as i32 % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    [r, g, b, a]
}

pub(super) fn rgb_to_hsl(c: [f32; 4]) -> (f32, f32, f32) {
    let (max, min, delta) = rgb_range(c);
    let l = (max + min) * 0.5;
    let s = if delta <= 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };
    (rgb_hue(c, max, delta), s.clamp(0.0, 1.0), l)
}

pub(super) fn hsl_to_rgba(h: f32, s: f32, l: f32, a: f32) -> [f32; 4] {
    if s <= 0.0 {
        return [l, l, l, a];
    }
    let chroma = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = h.rem_euclid(1.0) * 6.0;
    let x = chroma * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = match hp as i32 % 6 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let m = l - chroma * 0.5;
    [r1 + m, g1 + m, b1 + m, a]
}
