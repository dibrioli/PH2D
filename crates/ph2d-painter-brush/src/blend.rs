//! Brush blend modes — the 24 modes of Blender's texture-paint brush.
//!
//! Behavioural reference (clean-room, no code copied): the mode set is Blender's `IMB_BLEND_*`
//! dispatched in `imbuf/intern/rectop.cc::IMB_blend_color_byte/float`. Blender's 8-bit texture
//! paint blends in the image's **native (encoded, straight-alpha) space** — it does *not*
//! linearize — so this engine blends the dab in the layer's stored space and lets the host's
//! compositor do the sRGB→linear decode at composite time. The per-mode arithmetic is the
//! standard separable / W3C non-separable blend math.
//!
//! All values are straight-alpha RGBA in `[0, 1]`. `a` is the per-pixel coverage (falloff × flow ×
//! pressure) the dab has already computed.

/// A brush blend mode. The set and ordering mirror Blender's `IMB_BLEND_*` brush modes (the single
/// reference). Discriminants are PH2D-stable (used by the serialized `BrushSpec`); they need not
/// match Blender's numeric values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum BrushBlend {
    /// Alpha-over (the default "Mix"). Changes both colour and alpha.
    #[default]
    Mix = 0,
    Add = 1,
    Subtract = 2,
    Multiply = 3,
    Lighten = 4,
    Darken = 5,
    /// Reduce destination alpha by the coverage (eraser). Colour untouched.
    EraseAlpha = 6,
    /// Increase destination alpha by the coverage. Colour untouched.
    AddAlpha = 7,
    Overlay = 8,
    HardLight = 9,
    ColorBurn = 10,
    LinearBurn = 11,
    ColorDodge = 12,
    Screen = 13,
    SoftLight = 14,
    PinLight = 15,
    LinearLight = 16,
    VividLight = 17,
    Difference = 18,
    Exclusion = 19,
    Color = 20,
    Hue = 21,
    Saturation = 22,
    Luminosity = 23,
}

/// Number of brush blend modes (matches Blender's brush mode set).
pub const MAX_BRUSH_BLEND_MODES: u8 = 24;

impl BrushBlend {
    /// Every mode, in wire-discriminant order. The brush-settings dropdown
    /// iterates this; the index is the stable `u8` discriminant.
    pub const ALL: [BrushBlend; MAX_BRUSH_BLEND_MODES as usize] = [
        Self::Mix,
        Self::Add,
        Self::Subtract,
        Self::Multiply,
        Self::Lighten,
        Self::Darken,
        Self::EraseAlpha,
        Self::AddAlpha,
        Self::Overlay,
        Self::HardLight,
        Self::ColorBurn,
        Self::LinearBurn,
        Self::ColorDodge,
        Self::Screen,
        Self::SoftLight,
        Self::PinLight,
        Self::LinearLight,
        Self::VividLight,
        Self::Difference,
        Self::Exclusion,
        Self::Color,
        Self::Hue,
        Self::Saturation,
        Self::Luminosity,
    ];

    /// The stable wire discriminant (`0..MAX_BRUSH_BLEND_MODES`).
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Mode for a wire discriminant; out-of-range falls back to [`Self::Mix`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        Self::ALL.get(v as usize).copied().unwrap_or(Self::Mix)
    }

    /// Human display name for the brush-settings dropdown (English UI per HR-15).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Mix => "Mix",
            Self::Add => "Add",
            Self::Subtract => "Subtract",
            Self::Multiply => "Multiply",
            Self::Lighten => "Lighten",
            Self::Darken => "Darken",
            Self::EraseAlpha => "Erase Alpha",
            Self::AddAlpha => "Add Alpha",
            Self::Overlay => "Overlay",
            Self::HardLight => "Hard Light",
            Self::ColorBurn => "Color Burn",
            Self::LinearBurn => "Linear Burn",
            Self::ColorDodge => "Color Dodge",
            Self::Screen => "Screen",
            Self::SoftLight => "Soft Light",
            Self::PinLight => "Pin Light",
            Self::LinearLight => "Linear Light",
            Self::VividLight => "Vivid Light",
            Self::Difference => "Difference",
            Self::Exclusion => "Exclusion",
            Self::Color => "Color",
            Self::Hue => "Hue",
            Self::Saturation => "Saturation",
            Self::Luminosity => "Luminosity",
        }
    }
}

/// Blend a single brush sample of colour `color` and coverage `a` over destination pixel `dst`.
///
/// All inputs/outputs are straight-alpha RGBA in `[0, 1]` in the layer's native space. Colour
/// blend modes keep the destination alpha (you cannot paint colour where there is no coverage and
/// no alpha); `Mix` composites alpha via source-over; `EraseAlpha`/`AddAlpha` only move alpha.
#[must_use]
pub fn blend_over(mode: BrushBlend, dst: [f32; 4], color: [f32; 3], a: f32) -> [f32; 4] {
    let a = a.clamp(0.0, 1.0);
    let cb = [dst[0], dst[1], dst[2]];
    let ab = dst[3];

    match mode {
        BrushBlend::Mix => {
            // Straight-alpha Porter–Duff source-over; source alpha = a.
            let ao = a + ab * (1.0 - a);
            if ao <= 0.0 {
                return [0.0, 0.0, 0.0, 0.0];
            }
            let co = |b: f32, s: f32| (s * a + b * ab * (1.0 - a)) / ao;
            [co(cb[0], color[0]), co(cb[1], color[1]), co(cb[2], color[2]), ao]
        }
        BrushBlend::EraseAlpha => [cb[0], cb[1], cb[2], (ab - a).clamp(0.0, 1.0)],
        BrushBlend::AddAlpha => [cb[0], cb[1], cb[2], (ab + a).clamp(0.0, 1.0)],
        _ => {
            // Colour modes: B(background, source) per the mode, then mix toward it by coverage `a`.
            // Destination alpha is preserved.
            let blended = blend_rgb(mode, cb, color);
            [
                lerp(cb[0], blended[0], a),
                lerp(cb[1], blended[1], a),
                lerp(cb[2], blended[2], a),
                ab,
            ]
        }
    }
}

#[inline]
fn lerp(b: f32, s: f32, t: f32) -> f32 {
    b + (s - b) * t
}

/// Per-channel / non-separable colour blend `B(b, s)` for the colour modes.
fn blend_rgb(mode: BrushBlend, b: [f32; 3], s: [f32; 3]) -> [f32; 3] {
    let sep = |f: fn(f32, f32) -> f32| [f(b[0], s[0]), f(b[1], s[1]), f(b[2], s[2])];
    match mode {
        BrushBlend::Add => sep(|b, s| (b + s).min(1.0)),
        BrushBlend::Subtract => sep(|b, s| (b - s).max(0.0)),
        BrushBlend::Multiply => sep(|b, s| b * s),
        BrushBlend::Lighten => sep(|b, s| b.max(s)),
        BrushBlend::Darken => sep(|b, s| b.min(s)),
        BrushBlend::Screen => sep(screen),
        BrushBlend::Overlay => sep(|b, s| hard_light(s, b)), // overlay(b,s) = hardlight(s,b)
        BrushBlend::HardLight => sep(hard_light),
        BrushBlend::ColorDodge => sep(color_dodge),
        BrushBlend::ColorBurn => sep(color_burn),
        BrushBlend::LinearBurn => sep(|b, s| (b + s - 1.0).clamp(0.0, 1.0)),
        BrushBlend::LinearLight => sep(|b, s| (b + 2.0 * s - 1.0).clamp(0.0, 1.0)),
        BrushBlend::VividLight => sep(vivid_light),
        BrushBlend::PinLight => sep(pin_light),
        BrushBlend::Difference => sep(|b, s| (b - s).abs()),
        BrushBlend::Exclusion => sep(|b, s| b + s - 2.0 * b * s),
        BrushBlend::SoftLight => sep(soft_light),
        BrushBlend::Color => set_lum(s, lum(b)),
        BrushBlend::Hue => set_lum(set_sat(s, sat(b)), lum(b)),
        BrushBlend::Saturation => set_lum(set_sat(b, sat(s)), lum(b)),
        BrushBlend::Luminosity => set_lum(b, lum(s)),
        // Mix / EraseAlpha / AddAlpha are handled in `blend_over`.
        BrushBlend::Mix | BrushBlend::EraseAlpha | BrushBlend::AddAlpha => s,
    }
}

#[inline]
fn screen(b: f32, s: f32) -> f32 {
    1.0 - (1.0 - b) * (1.0 - s)
}

#[inline]
fn hard_light(b: f32, s: f32) -> f32 {
    if s <= 0.5 {
        2.0 * b * s
    } else {
        1.0 - 2.0 * (1.0 - b) * (1.0 - s)
    }
}

#[inline]
fn color_dodge(b: f32, s: f32) -> f32 {
    if b <= 0.0 {
        0.0
    } else if s >= 1.0 {
        1.0
    } else {
        (b / (1.0 - s)).min(1.0)
    }
}

#[inline]
fn color_burn(b: f32, s: f32) -> f32 {
    if b >= 1.0 {
        1.0
    } else if s <= 0.0 {
        0.0
    } else {
        1.0 - ((1.0 - b) / s).min(1.0)
    }
}

#[inline]
fn vivid_light(b: f32, s: f32) -> f32 {
    if s <= 0.5 {
        color_burn(b, (2.0 * s).min(1.0))
    } else {
        color_dodge(b, (2.0 * (s - 0.5)).min(1.0))
    }
}

#[inline]
fn pin_light(b: f32, s: f32) -> f32 {
    if s < 0.5 {
        b.min(2.0 * s)
    } else {
        b.max(2.0 * s - 1.0)
    }
}

#[inline]
fn soft_light(b: f32, s: f32) -> f32 {
    // W3C soft-light.
    if s <= 0.5 {
        b - (1.0 - 2.0 * s) * b * (1.0 - b)
    } else {
        let d = if b <= 0.25 {
            ((16.0 * b - 12.0) * b + 4.0) * b
        } else {
            b.sqrt()
        };
        b + (2.0 * s - 1.0) * (d - b)
    }
}

// --- Non-separable (HSL) helpers — W3C/Blender luma weights 0.3 / 0.59 / 0.11. ---

#[inline]
fn lum(c: [f32; 3]) -> f32 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

#[inline]
fn sat(c: [f32; 3]) -> f32 {
    c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
}

fn clip_color(c: [f32; 3]) -> [f32; 3] {
    let l = lum(c);
    let n = c[0].min(c[1]).min(c[2]);
    let x = c[0].max(c[1]).max(c[2]);
    let mut out = c;
    if n < 0.0 && (l - n).abs() > f32::EPSILON {
        for (o, &ci) in out.iter_mut().zip(c.iter()) {
            *o = l + (ci - l) * l / (l - n);
        }
    }
    if x > 1.0 && (x - l).abs() > f32::EPSILON {
        for o in &mut out {
            *o = l + (*o - l) * (1.0 - l) / (x - l);
        }
    }
    out
}

fn set_lum(c: [f32; 3], l: f32) -> [f32; 3] {
    let d = l - lum(c);
    clip_color([c[0] + d, c[1] + d, c[2] + d])
}

fn set_sat(c: [f32; 3], s: f32) -> [f32; 3] {
    // Order channel indices by value: min, mid, max (total_cmp = deterministic, NaN-safe).
    let mut idx = [0usize, 1, 2];
    idx.sort_by(|&a, &b| c[a].total_cmp(&c[b]));
    let (i_min, i_mid, i_max) = (idx[0], idx[1], idx[2]);
    let mut out = [0.0f32; 3];
    if c[i_max] > c[i_min] {
        out[i_mid] = (c[i_mid] - c[i_min]) * s / (c[i_max] - c[i_min]);
        out[i_max] = s;
    }
    out[i_min] = 0.0;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_full_coverage_replaces() {
        // Mix, a=1, opaque bg ⟹ exactly the source colour, opaque. (Source-over identity.)
        let out = blend_over(BrushBlend::Mix, [0.2, 0.4, 0.6, 1.0], [0.9, 0.1, 0.5], 1.0);
        assert!((out[0] - 0.9).abs() < 1e-6);
        assert!((out[1] - 0.1).abs() < 1e-6);
        assert!((out[2] - 0.5).abs() < 1e-6);
        assert!((out[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn mix_zero_coverage_is_noop() {
        let dst = [0.2, 0.4, 0.6, 0.8];
        let out = blend_over(BrushBlend::Mix, dst, [0.9, 0.1, 0.5], 0.0);
        for i in 0..4 {
            assert!((out[i] - dst[i]).abs() < 1e-6, "channel {i}");
        }
    }

    #[test]
    fn mix_over_transparent_takes_source() {
        // Painting onto fully transparent ⟹ source colour at coverage alpha.
        let out = blend_over(BrushBlend::Mix, [0.0, 0.0, 0.0, 0.0], [0.8, 0.2, 0.4], 0.5);
        assert!((out[3] - 0.5).abs() < 1e-6);
        assert!((out[0] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn multiply_halves() {
        let out = blend_over(BrushBlend::Multiply, [0.5, 0.5, 0.5, 1.0], [0.5, 0.5, 0.5], 1.0);
        assert!((out[0] - 0.25).abs() < 1e-6);
        assert!((out[3] - 1.0).abs() < 1e-6, "colour mode keeps bg alpha");
    }

    #[test]
    fn screen_is_inverse_multiply() {
        let out = blend_over(BrushBlend::Screen, [0.5, 0.5, 0.5, 1.0], [0.5, 0.5, 0.5], 1.0);
        assert!((out[0] - 0.75).abs() < 1e-6);
    }

    #[test]
    fn erase_and_add_alpha_move_only_alpha() {
        let e = blend_over(BrushBlend::EraseAlpha, [0.3, 0.3, 0.3, 0.8], [1.0, 1.0, 1.0], 0.5);
        assert!((e[3] - 0.3).abs() < 1e-6);
        assert!((e[0] - 0.3).abs() < 1e-6, "erase keeps colour");
        let a = blend_over(BrushBlend::AddAlpha, [0.3, 0.3, 0.3, 0.4], [1.0, 1.0, 1.0], 0.5);
        assert!((a[3] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn overlay_is_hardlight_with_swapped_roles() {
        let b = [0.3, 0.6, 0.2];
        let s = [0.7, 0.4, 0.9];
        let ov = blend_rgb(BrushBlend::Overlay, b, s);
        let hl = blend_rgb(BrushBlend::HardLight, s, b);
        for i in 0..3 {
            assert!((ov[i] - hl[i]).abs() < 1e-6, "channel {i}");
        }
    }

    #[test]
    fn luminosity_matches_source_luma_keeps_bg_chroma() {
        // Luminosity: take bg chroma, source luma ⟹ output luma == source luma.
        let out = blend_rgb(BrushBlend::Luminosity, [0.2, 0.5, 0.8], [0.6, 0.6, 0.6]);
        assert!((lum(out) - lum([0.6, 0.6, 0.6])).abs() < 1e-5);
    }

    #[test]
    fn color_mode_outputs_stay_in_range() {
        // HSL modes can push out of [0,1]; clip_color must keep results valid.
        for &mode in &[BrushBlend::Color, BrushBlend::Hue, BrushBlend::Saturation] {
            let out = blend_rgb(mode, [0.05, 0.9, 0.5], [0.95, 0.1, 0.2]);
            for (i, v) in out.iter().enumerate() {
                assert!((-1e-4..=1.0001).contains(v), "{mode:?} channel {i} = {v} out of range");
            }
        }
    }

    #[test]
    fn wire_discriminants_round_trip() {
        assert_eq!(BrushBlend::ALL.len(), MAX_BRUSH_BLEND_MODES as usize);
        for (i, &mode) in BrushBlend::ALL.iter().enumerate() {
            // The array index IS the wire discriminant.
            assert_eq!(mode.to_u8() as usize, i, "{mode:?} discriminant != index");
            assert_eq!(BrushBlend::from_u8(i as u8), mode, "from_u8 != ALL[{i}]");
            assert!(!mode.name().is_empty(), "{mode:?} has no display name");
        }
        // Out-of-range falls back to Mix (never panics).
        assert_eq!(BrushBlend::from_u8(MAX_BRUSH_BLEND_MODES), BrushBlend::Mix);
        assert_eq!(BrushBlend::from_u8(255), BrushBlend::Mix);
    }

    #[test]
    fn display_names_are_unique() {
        let mut names: Vec<&str> = BrushBlend::ALL.iter().map(|m| m.name()).collect();
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate brush blend display name");
    }
}
