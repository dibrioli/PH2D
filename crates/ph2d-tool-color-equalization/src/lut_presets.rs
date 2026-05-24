//! 15 procedural 3D-LUT presets organised in 4 groups (Cinematic,
//! Atmosphere, Vintage, Stylized). Mirrors the legacy engine's
//! `lut-presets.ts` — each preset is a `(r, g, b) → (r, g, b)` color
//! function evaluated on a regular sRGB grid by
//! [`generate_preset_lut`], then sampled trilinearly by
//! [`crate::lut::apply_lut3d`].
//!
//! All presets operate on **normalised sRGB** `[0, 1]` (no linear-light
//! conversion). The looks are deliberately matched to the legacy
//! reference so existing creative pipelines port over unchanged.

use crate::color_utils::{
    clamp01, hsl_to_rgb, lerp_f32, lift_blacks, luma_srgb, rgb_to_hsl, s_curve,
};
use crate::lut::{DEFAULT_LUT_SIZE, LUT3D};

/// 15 procedural presets + an explicit `None` slot, in canonical
/// dropdown order. `next` / `prev` cycle through the list, wrapping at
/// the ends, so the panel's cycle button can advance / step back
/// without holding state outside the enum itself.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum LutPreset {
    #[default]
    None,
    Cinematic,
    Blockbuster,
    FilmNoir,
    Warm,
    Cool,
    GoldenHour,
    Moonlight,
    Vintage,
    Sepia,
    FadedFilm,
    Polaroid,
    Vibrant,
    Matte,
    BleachBypass,
    CrossProcess,
}

impl LutPreset {
    /// Full ordered list, `None` first. Used by `next` / `prev`.
    pub const ALL: [Self; 16] = [
        Self::None,
        Self::Cinematic,
        Self::Blockbuster,
        Self::FilmNoir,
        Self::Warm,
        Self::Cool,
        Self::GoldenHour,
        Self::Moonlight,
        Self::Vintage,
        Self::Sepia,
        Self::FadedFilm,
        Self::Polaroid,
        Self::Vibrant,
        Self::Matte,
        Self::BleachBypass,
        Self::CrossProcess,
    ];

    /// Human-readable label for the panel chip.
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Cinematic => "Cinematic",
            Self::Blockbuster => "Blockbuster",
            Self::FilmNoir => "Film Noir",
            Self::Warm => "Warm",
            Self::Cool => "Cool",
            Self::GoldenHour => "Golden Hour",
            Self::Moonlight => "Moonlight",
            Self::Vintage => "Vintage",
            Self::Sepia => "Sepia",
            Self::FadedFilm => "Faded Film",
            Self::Polaroid => "Polaroid",
            Self::Vibrant => "Vibrant",
            Self::Matte => "Matte",
            Self::BleachBypass => "Bleach Bypass",
            Self::CrossProcess => "Cross Process",
        }
    }

    /// Kebab-case slug (matches the legacy preset name).
    pub fn slug(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Cinematic => "cinematic",
            Self::Blockbuster => "blockbuster",
            Self::FilmNoir => "film-noir",
            Self::Warm => "warm",
            Self::Cool => "cool",
            Self::GoldenHour => "golden-hour",
            Self::Moonlight => "moonlight",
            Self::Vintage => "vintage",
            Self::Sepia => "sepia",
            Self::FadedFilm => "faded-film",
            Self::Polaroid => "polaroid",
            Self::Vibrant => "vibrant",
            Self::Matte => "matte",
            Self::BleachBypass => "bleach-bypass",
            Self::CrossProcess => "cross-process",
        }
    }

    /// Group header for the panel dropdown grouping.
    pub fn group(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Cinematic | Self::Blockbuster | Self::FilmNoir => "Cinematic",
            Self::Warm | Self::Cool | Self::GoldenHour | Self::Moonlight => "Atmosphere",
            Self::Vintage | Self::Sepia | Self::FadedFilm | Self::Polaroid => "Vintage",
            Self::Vibrant | Self::Matte | Self::BleachBypass | Self::CrossProcess => "Stylized",
        }
    }

    /// Index in [`Self::ALL`].
    fn index(self) -> usize {
        Self::ALL.iter().position(|p| *p == self).unwrap_or(0)
    }

    /// Cycle one step forward (wraps `CrossProcess → None`).
    pub fn next(self) -> Self {
        let i = self.index();
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    /// Cycle one step backward (wraps `None → CrossProcess`).
    pub fn prev(self) -> Self {
        let i = self.index();
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

// ── Preset color functions (port of legacy lut-presets.ts) ───────

type PresetFn = fn(f32, f32, f32) -> [f32; 3];

fn cinematic(r: f32, g: f32, b: f32) -> [f32; 3] {
    // Teal shadows + orange highlights, mild S-curve.
    let [h, s, l] = rgb_to_hsl(r, g, b);
    let mut new_h = h;
    let mut new_s = s;
    if l < 0.4 {
        new_h = lerp_f32(h, 0.5, 0.3 * (1.0 - l / 0.4));
        new_s = (s * 1.2).min(1.0);
    } else if l > 0.6 {
        new_h = lerp_f32(h, 0.08, 0.2 * ((l - 0.6) / 0.4));
    }
    let new_l = s_curve(l, 1.5);
    let out = hsl_to_rgb(new_h, new_s, new_l);
    [clamp01(out[0]), clamp01(out[1]), clamp01(out[2])]
}

fn blockbuster(r: f32, g: f32, b: f32) -> [f32; 3] {
    // High contrast, desaturated midtones, teal-orange split.
    let [h, s, l] = rgb_to_hsl(r, g, b);
    let mut new_s = s;
    let mut new_h = h;
    if l > 0.2 && l < 0.8 {
        let mid = 1.0 - (l - 0.5).abs() / 0.3;
        new_s = s * (1.0 - mid * 0.4);
    }
    if l < 0.35 {
        new_h = lerp_f32(h, 0.52, 0.4);
    } else if l > 0.65 {
        new_h = lerp_f32(h, 0.06, 0.3);
    }
    let new_l = s_curve(l, 2.0);
    let out = hsl_to_rgb(new_h, new_s, new_l);
    [clamp01(out[0]), clamp01(out[1]), clamp01(out[2])]
}

fn film_noir(r: f32, g: f32, b: f32) -> [f32; 3] {
    // Nearly monochrome, high contrast, slight warm tint in highlights.
    let l = luma_srgb(r, g, b);
    let curved = s_curve(l, 2.5);
    [
        clamp01(curved * 1.05),
        clamp01(curved),
        clamp01(curved * 0.92),
    ]
}

fn warm(r: f32, g: f32, b: f32) -> [f32; 3] {
    [
        clamp01(r * 1.1 + 0.02),
        clamp01(g * 1.02),
        clamp01(b * 0.88),
    ]
}

fn cool(r: f32, g: f32, b: f32) -> [f32; 3] {
    [
        clamp01(r * 0.9),
        clamp01(g * 0.98),
        clamp01(b * 1.12 + 0.02),
    ]
}

fn golden_hour(r: f32, g: f32, b: f32) -> [f32; 3] {
    // Warm golden cast, lifted shadows, reduced blues.
    let [h, s, l] = rgb_to_hsl(r, g, b);
    let new_h = lerp_f32(h, 0.1, 0.15);
    let new_l = lift_blacks(l, 0.03);
    let new_s = (s * 1.15).min(1.0);
    let out = hsl_to_rgb(new_h, new_s, new_l);
    [
        clamp01(out[0] * 1.08),
        clamp01(out[1] * 1.02),
        clamp01(out[2] * 0.85),
    ]
}

fn moonlight(r: f32, g: f32, b: f32) -> [f32; 3] {
    // Cool blue tint, desaturated, lifted shadows.
    let [h, s, l] = rgb_to_hsl(r, g, b);
    let new_h = lerp_f32(h, 0.6, 0.25);
    let new_s = s * 0.6;
    let new_l = lift_blacks(l, 0.05);
    let out = hsl_to_rgb(new_h, new_s, new_l);
    [clamp01(out[0]), clamp01(out[1]), clamp01(out[2])]
}

fn vintage(r: f32, g: f32, b: f32) -> [f32; 3] {
    // Faded, warm cast, lifted blacks, reduced saturation.
    let [h, s, l] = rgb_to_hsl(r, g, b);
    let new_s = s * 0.7;
    let new_l = lift_blacks(l, 0.06);
    let out = hsl_to_rgb(h, new_s, new_l);
    [
        clamp01(out[0] * 1.05 + 0.02),
        clamp01(out[1] * 0.98),
        clamp01(out[2] * 0.88),
    ]
}

fn sepia(r: f32, g: f32, b: f32) -> [f32; 3] {
    // Classic sepia tone — luma-driven warm cast.
    let l = luma_srgb(r, g, b);
    [
        clamp01(l * 1.12 + 0.04),
        clamp01(l * 0.95 + 0.02),
        clamp01(l * 0.74),
    ]
}

fn faded_film(r: f32, g: f32, b: f32) -> [f32; 3] {
    // Lifted blacks, faded look, slight color cast.
    let [h, s, l] = rgb_to_hsl(r, g, b);
    let new_l = lift_blacks(l, 0.08);
    let new_s = s * 0.65;
    let comp_l = if new_l > 0.85 {
        0.85 + (new_l - 0.85) * 0.5
    } else {
        new_l
    };
    let out = hsl_to_rgb(h, new_s, comp_l);
    [
        clamp01(out[0] + 0.01),
        clamp01(out[1]),
        clamp01(out[2] + 0.02),
    ]
}

fn polaroid(r: f32, g: f32, b: f32) -> [f32; 3] {
    // Warm highlights, slightly green midtones, lifted blacks.
    let [h, s, l] = rgb_to_hsl(r, g, b);
    let new_h = if l > 0.5 {
        lerp_f32(h, 0.1, 0.1)
    } else {
        lerp_f32(h, 0.35, 0.08)
    };
    let new_l = lift_blacks(l, 0.04);
    let new_s = (s * 1.1).min(1.0);
    let out = hsl_to_rgb(new_h, new_s, new_l);
    [clamp01(out[0]), clamp01(out[1]), clamp01(out[2])]
}

fn vibrant(r: f32, g: f32, b: f32) -> [f32; 3] {
    // Boost saturation and contrast.
    let [h, s, l] = rgb_to_hsl(r, g, b);
    let new_s = (s * 1.4).min(1.0);
    let new_l = s_curve(l, 1.3);
    let out = hsl_to_rgb(h, new_s, new_l);
    [clamp01(out[0]), clamp01(out[1]), clamp01(out[2])]
}

fn matte(r: f32, g: f32, b: f32) -> [f32; 3] {
    // Lifted blacks, compressed highlights, reduced saturation.
    let [h, s, l] = rgb_to_hsl(r, g, b);
    let new_l = lift_blacks(l, 0.1);
    let comp_l = if new_l > 0.9 {
        0.9 + (new_l - 0.9) * 0.3
    } else {
        new_l
    };
    let new_s = s * 0.8;
    let out = hsl_to_rgb(h, new_s, comp_l);
    [clamp01(out[0]), clamp01(out[1]), clamp01(out[2])]
}

fn bleach_bypass(r: f32, g: f32, b: f32) -> [f32; 3] {
    // High contrast, desaturated — silver-retention film look.
    let l = luma_srgb(r, g, b);
    let curved = s_curve(l, 2.0);
    let mix = 0.6;
    [
        clamp01(r * (1.0 - mix) + curved * mix),
        clamp01(g * (1.0 - mix) + curved * mix),
        clamp01(b * (1.0 - mix) + curved * mix),
    ]
}

fn cross_process(r: f32, g: f32, b: f32) -> [f32; 3] {
    // Green/yellow shadows, magenta highlights, boosted saturation.
    let [h, s, l] = rgb_to_hsl(r, g, b);
    let new_h = if l < 0.4 {
        lerp_f32(h, 0.22, 0.35)
    } else if l > 0.6 {
        lerp_f32(h, 0.85, 0.2)
    } else {
        h
    };
    let new_s = (s * 1.3).min(1.0);
    let new_l = s_curve(l, 1.2);
    let out = hsl_to_rgb(new_h, new_s, new_l);
    [clamp01(out[0]), clamp01(out[1]), clamp01(out[2])]
}

fn preset_fn(preset: LutPreset) -> Option<PresetFn> {
    Some(match preset {
        LutPreset::None => return None,
        LutPreset::Cinematic => cinematic,
        LutPreset::Blockbuster => blockbuster,
        LutPreset::FilmNoir => film_noir,
        LutPreset::Warm => warm,
        LutPreset::Cool => cool,
        LutPreset::GoldenHour => golden_hour,
        LutPreset::Moonlight => moonlight,
        LutPreset::Vintage => vintage,
        LutPreset::Sepia => sepia,
        LutPreset::FadedFilm => faded_film,
        LutPreset::Polaroid => polaroid,
        LutPreset::Vibrant => vibrant,
        LutPreset::Matte => matte,
        LutPreset::BleachBypass => bleach_bypass,
        LutPreset::CrossProcess => cross_process,
    })
}

/// Materialise a procedural preset as a [`LUT3D`] of the given `size`.
/// Returns `None` for `LutPreset::None` (so callers can short-circuit
/// the apply pass). Uses [`DEFAULT_LUT_SIZE`] when `size = 0`.
pub fn generate_preset_lut(preset: LutPreset, size: u32) -> Option<LUT3D> {
    let func = preset_fn(preset)?;
    let size = if size == 0 { DEFAULT_LUT_SIZE } else { size }.max(2);
    let n = size as usize;
    let mut data = vec![0.0_f32; LUT3D::expected_len(size)];
    let denom = (size - 1) as f32;
    for bi in 0..n {
        for gi in 0..n {
            for ri in 0..n {
                let r = ri as f32 / denom;
                let g = gi as f32 / denom;
                let b = bi as f32 / denom;
                let out = func(r, g, b);
                let idx = ((bi * n + gi) * n + ri) * 3;
                data[idx] = out[0];
                data[idx + 1] = out[1];
                data[idx + 2] = out[2];
            }
        }
    }
    Some(LUT3D { size, data })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_wraps_at_both_ends() {
        assert_eq!(LutPreset::None.prev(), LutPreset::CrossProcess);
        assert_eq!(LutPreset::CrossProcess.next(), LutPreset::None);
        assert_eq!(LutPreset::Cinematic.next(), LutPreset::Blockbuster);
        assert_eq!(LutPreset::Blockbuster.prev(), LutPreset::Cinematic);
    }

    #[test]
    fn all_variants_have_labels_and_slugs() {
        for p in LutPreset::ALL {
            assert!(!p.label().is_empty());
            assert!(!p.slug().is_empty());
            assert!(!p.group().is_empty());
        }
    }

    #[test]
    fn slugs_match_legacy_kebab_case() {
        // Spot check a few against the legacy LUTPresetName strings.
        assert_eq!(LutPreset::Cinematic.slug(), "cinematic");
        assert_eq!(LutPreset::FilmNoir.slug(), "film-noir");
        assert_eq!(LutPreset::GoldenHour.slug(), "golden-hour");
        assert_eq!(LutPreset::FadedFilm.slug(), "faded-film");
        assert_eq!(LutPreset::BleachBypass.slug(), "bleach-bypass");
        assert_eq!(LutPreset::CrossProcess.slug(), "cross-process");
    }

    #[test]
    fn generate_preset_lut_returns_none_for_none() {
        assert!(generate_preset_lut(LutPreset::None, DEFAULT_LUT_SIZE).is_none());
    }

    #[test]
    fn generate_preset_lut_for_warm_lifts_red_drops_blue_in_grey_cell() {
        // The middle grid cell at (0.5, 0.5, 0.5) under `warm` should
        // bump R up (mul 1.1 + 0.02 = 0.57) and pull B down
        // (mul 0.88 = 0.44).
        let lut = generate_preset_lut(LutPreset::Warm, 9).expect("warm LUT");
        // Cell (4, 4, 4) of a 9³ LUT = exact midpoint.
        let idx = ((4_usize * 9 + 4) * 9 + 4) * 3;
        assert!(lut.data[idx] > 0.5, "warm should lift R: {}", lut.data[idx]);
        assert!(
            lut.data[idx + 2] < 0.5,
            "warm should drop B: {}",
            lut.data[idx + 2]
        );
    }

    #[test]
    fn generate_preset_lut_for_sepia_outputs_warm_grey_for_grey_input() {
        // Sepia maps luma to a warm cast — for input mid-grey (0.5),
        // R should be highest, then G, then B.
        let lut = generate_preset_lut(LutPreset::Sepia, 9).expect("sepia LUT");
        let idx = ((4_usize * 9 + 4) * 9 + 4) * 3;
        let r = lut.data[idx];
        let g = lut.data[idx + 1];
        let b = lut.data[idx + 2];
        assert!(r > g, "sepia R should be > G ({r} vs {g})");
        assert!(g > b, "sepia G should be > B ({g} vs {b})");
    }

    #[test]
    fn generate_preset_lut_for_film_noir_collapses_to_near_grey() {
        // film_noir is luma-driven with a very mild warm tint.
        // For a saturated input (red), the output R/G/B should be much
        // closer together than the input (which is 1,0,0).
        let lut = generate_preset_lut(LutPreset::FilmNoir, 9).expect("film noir LUT");
        // Cell (rI=8, gI=0, bI=0) = (R=1, G=0, B=0). `bI*81 + gI*9 + rI`
        // with bI=gI=0 simplifies to just rI, so the explicit zeros
        // would only clutter the index math.
        let idx = 8_usize * 3;
        let r = lut.data[idx];
        let g = lut.data[idx + 1];
        let b = lut.data[idx + 2];
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        // Saturated red has range 1.0; film noir should compress to
        // less than 0.3 across channels (curved luma at 0.2126).
        assert!(
            (max - min) < 0.3,
            "film noir did not desaturate ({r}, {g}, {b})"
        );
    }

    #[test]
    fn all_presets_generate_well_formed_luts() {
        for p in LutPreset::ALL {
            if p == LutPreset::None {
                continue;
            }
            let lut = generate_preset_lut(p, DEFAULT_LUT_SIZE).expect("non-None LUT");
            assert_eq!(lut.size, DEFAULT_LUT_SIZE);
            assert_eq!(lut.data.len(), LUT3D::expected_len(DEFAULT_LUT_SIZE));
            // Every cell value is finite and in `[0, 1]`.
            for v in &lut.data {
                assert!(v.is_finite(), "{} produced non-finite cell", p.slug());
                assert!(
                    *v >= 0.0 && *v <= 1.0,
                    "{} cell out of range: {}",
                    p.slug(),
                    v
                );
            }
        }
    }
}
