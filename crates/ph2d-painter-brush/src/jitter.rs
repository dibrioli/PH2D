//! Per-dab **randomization** — the deterministic RNG plus the three jitter quantities the panel
//! exposes: **Jitter Scale** (radius), **Jitter Rotate** (texture rotation) and **Randomize Color**
//! (per-dab HSV offset, a clean-room model of Blender's brush "Color → Randomize").
//!
//! **Determinism (HR-5).** Everything here is transcendental-free (`+ - * / floor abs sqrt` and the
//! baked 1° rotation step in [`crate::texture`]). The RNG is the dep-free splitmix64 used across the
//! engine; the per-dab draws happen in a **fixed order** — scale → rotate → H → S → V — and *only*
//! for the features that are active, so an all-off brush draws nothing and is byte-identical to the
//! no-jitter baseline (the position jitter, drawn afterwards in [`crate::stroke`], keeps its slot).

use crate::spec::BrushSpec;

/// Dep-free deterministic `[0, 1)` RNG (splitmix64 → top 24 bits). The single RNG for the whole
/// engine — the stroke's position jitter, the texture's Random rotation/offset, and the per-dab
/// jitter below all advance one `u64` through this so a replay is bit-reproducible.
pub(crate) fn next_f32(state: &mut u64) -> f32 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    // Top 24 bits → [0, 1).
    ((z >> 40) as f32) / ((1u32 << 24) as f32)
}

/// Uniform sample inside the unit disc by rejection (matches Blender's jitter sampling).
pub(crate) fn disc_sample(rng: &mut u64) -> (f32, f32) {
    loop {
        let x = next_f32(rng) * 2.0 - 1.0;
        let y = next_f32(rng) * 2.0 - 1.0;
        if x * x + y * y <= 1.0 {
            return (x, y);
        }
    }
}

/// Resolve the per-dab jitter for `spec`: the radius **scale multiplier**, the extra **rotation**
/// unit vector for the texture frame (identity `[1, 0]` = none), the jittered **colour** and the
/// **HSV offset** that was drawn for it (`[0; 3]` = none — the offset a texture layer applies to its
/// own per-pixel colour, Bug #51).
///
/// Draw order is fixed (scale → rotate → H → S → V); each block draws from `rng` *only* when its
/// feature would actually change the dab, so toggling one feature never perturbs another's stream
/// while it is off, and an untouched brush consumes no randomness at all. The caller gates the whole
/// thing behind [`crate::StrokeMethod::allows_jitter`] (Drag Dot / Anchored place one deliberate dab,
/// so they opt out — like the position jitter).
pub(crate) fn per_dab(rng: &mut u64, spec: &BrushSpec) -> (f32, [f32; 2], [f32; 3], [f32; 3]) {
    let scale = if spec.jitter_scale > 0.0 {
        // ±amount around 1.0; never let the radius collapse — clamped by the caller.
        1.0 + (next_f32(rng) * 2.0 - 1.0) * spec.jitter_scale.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let rotation = if spec.jitter_rotate > 0.0 {
        rotation_unit(rng, spec.jitter_rotate)
    } else {
        [1.0, 0.0]
    };
    // Color randomization is driven SOLELY by the Hue/Sat/Value amounts (any > 0 → active).
    // The legacy `color_jitter_enabled` flag is no longer gated on — the brush panel dropped its
    // redundant enable toggle (the "Randomize Color" section activates on amount > 0, Enio 2026-06-24).
    let (color, shift) = colour_draw(rng, spec);
    (scale, rotation, color, shift)
}

/// The Randomize-Color draw for `spec` — `(jittered colour, HSV offset)` — or the plain colour and a
/// zero offset when every amount is zero (no draw). The ONE place the H → S → V draws happen, shared by
/// the per-dab jitter and the per-stroke draw in [`crate::Stroke::new`].
pub(crate) fn colour_draw(rng: &mut u64, spec: &BrushSpec) -> ([f32; 3], [f32; 3]) {
    if !spec.has_colour_jitter_amount() {
        return (spec.color, [0.0; 3]);
    }
    let shift = [
        (next_f32(rng) * 2.0 - 1.0) * spec.color_jitter_hue.clamp(0.0, 1.0),
        (next_f32(rng) * 2.0 - 1.0) * spec.color_jitter_sat.clamp(0.0, 1.0),
        (next_f32(rng) * 2.0 - 1.0) * spec.color_jitter_val.clamp(0.0, 1.0),
    ];
    (shift_hsv(spec.color, shift), shift)
}

/// **When** Randomize Color draws: a new offset for every dab (the classic scatter) or ONE offset per
/// stroke (the whole stroke in one colour, the next in another) — Procreate's *Color dynamics* keeps
/// the two as separate controls. The per-stroke draw comes from its own stream seeded by the stroke's
/// seed, and the per-dab draws still happen, so every other jitter is byte-identical in both modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ColorJitterScope {
    /// A new colour every dab (the default — what every preset and project before this field paints).
    #[default]
    PerDab,
    /// One colour for the whole stroke.
    PerStroke,
}

impl ColorJitterScope {
    /// Wire discriminant (`PerDab` = 0, `PerStroke` = 1).
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::PerDab => 0,
            Self::PerStroke => 1,
        }
    }

    /// Decode a wire discriminant; unknown values fall back to [`ColorJitterScope::PerDab`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::PerStroke,
            _ => Self::PerDab,
        }
    }
}

/// Per-gap **Jitter Spacing** multiplier: `1 ± amount` around the base spacing, floored at `0.05` so a
/// gap never collapses to zero (the walk re-floors the absolute px to ≥ 1 anyway). One `rng` draw;
/// drawn only when the amount is non-zero (gated by the caller) so an even-spacing brush stays
/// byte-identical to the no-jitter baseline.
pub(crate) fn spacing_mult(rng: &mut u64, amount: f32) -> f32 {
    let signed = next_f32(rng) * 2.0 - 1.0; // [-1, 1)
    (1.0 + signed * amount.clamp(0.0, 1.0)).max(0.05)
}

/// A random rotation unit vector in `±(amount · 180°)`, built transcendental-free from the baked 1°
/// step in [`crate::texture`] (so it composes with the texture's base/Rake/Random rotation exactly
/// like that module's own angles do).
fn rotation_unit(rng: &mut u64, amount: f32) -> [f32; 2] {
    let signed = next_f32(rng) * 2.0 - 1.0; // [-1, 1)
    let deg = (signed * amount.clamp(0.0, 1.0) * 180.0).round() as i32; // [-180, 180]
    let wrapped = (((deg % 360) + 360) % 360) as u16; // [0, 360)
    crate::texture::rotate_by_degrees(wrapped)
}

/// Offset `base` in HSV by the drawn `shift = [dh, ds, dv]`: hue wraps the colour wheel, saturation /
/// value clamp to the unit range. Identity (to the bit) for a zero `shift`.
#[must_use]
pub fn apply_color_shift(base: [f32; 3], shift: [f32; 3]) -> [f32; 3] {
    if shift == [0.0; 3] {
        return base;
    }
    shift_hsv(base, shift)
}

/// [`apply_color_shift`] without the zero short-cut: the per-dab draw always ran the HSV round trip,
/// and its bytes are the contract.
fn shift_hsv(base: [f32; 3], shift: [f32; 3]) -> [f32; 3] {
    let mut hsv = rgb_to_hsv(base);
    hsv[0] = (hsv[0] + shift[0]).rem_euclid(1.0);
    hsv[1] = (hsv[1] + shift[1]).clamp(0.0, 1.0);
    hsv[2] = (hsv[2] + shift[2]).clamp(0.0, 1.0);
    hsv_to_rgb(hsv)
}

/// RGB → HSV, all channels `0..1`. Pure min/max/division (no transcendentals). Hue is normalised to
/// `[0, 1)`; a greyscale colour gets hue `0`, saturation `0`.
pub(crate) fn rgb_to_hsv(rgb: [f32; 3]) -> [f32; 3] {
    let (r, g, b) = (rgb[0], rgb[1], rgb[2]);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let s = if max <= 0.0 { 0.0 } else { delta / max };
    let h6 = if delta <= 0.0 {
        0.0
    } else if max == r {
        (g - b) / delta
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    [(h6 / 6.0).rem_euclid(1.0), s, max]
}

/// HSV → RGB, inverse of [`rgb_to_hsv`]. Pure (`floor` + arithmetic).
pub(crate) fn hsv_to_rgb(hsv: [f32; 3]) -> [f32; 3] {
    let h = hsv[0].rem_euclid(1.0);
    let s = hsv[1].clamp(0.0, 1.0);
    let v = hsv[2].clamp(0.0, 1.0);
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match (i as i32).rem_euclid(6) {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_roundtrip_is_identity() {
        // A grid of colours survives RGB → HSV → RGB within 8-bit precision.
        for ri in 0..=4 {
            for gi in 0..=4 {
                for bi in 0..=4 {
                    let rgb = [ri as f32 / 4.0, gi as f32 / 4.0, bi as f32 / 4.0];
                    let back = hsv_to_rgb(rgb_to_hsv(rgb));
                    for c in 0..3 {
                        assert!(
                            (rgb[c] - back[c]).abs() < 1.0 / 255.0,
                            "roundtrip {rgb:?} -> {back:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn all_off_draws_no_randomness_and_keeps_colour() {
        let spec = BrushSpec {
            color: [0.2, 0.6, 0.9],
            ..Default::default()
        };
        let mut rng = 12345u64;
        let before = rng;
        let (scale, rot, color, shift) = per_dab(&mut rng, &spec);
        assert_eq!(rng, before, "an all-off brush must not advance the RNG");
        assert_eq!(scale, 1.0);
        assert_eq!(rot, [1.0, 0.0]);
        assert_eq!(color, spec.color);
        assert_eq!(shift, [0.0; 3]);
    }

    #[test]
    fn active_jitter_is_deterministic_and_varies() {
        let spec = BrushSpec {
            color: [0.5, 0.5, 0.5],
            jitter_scale: 0.5,
            jitter_rotate: 1.0,
            color_jitter_enabled: true,
            color_jitter_hue: 0.5,
            color_jitter_sat: 0.5,
            color_jitter_val: 0.5,
            ..Default::default()
        };
        // Same seed → identical result (replayable).
        let mut a = 7u64;
        let mut b = 7u64;
        assert_eq!(per_dab(&mut a, &spec), per_dab(&mut b, &spec));
        // Successive dabs differ (it actually jitters).
        let mut rng = 7u64;
        let d0 = per_dab(&mut rng, &spec);
        let d1 = per_dab(&mut rng, &spec);
        assert_ne!(d0.0, d1.0, "scale should vary dab to dab");
        assert_ne!(d0.2, d1.2, "colour should vary dab to dab");
    }
}
