//! Layer blend modes (W3.T3.3) — the 22 canonical modes from
//! `docs/Painter_projeto/02_layers.md` §2.2.
//!
//! # Color space
//!
//! [`apply`] operates on **straight (non-premultiplied) linear-sRGB**
//! RGBA in `[0, 1]`. The compositor decodes each layer's sRGB texels to
//! linear once, composites top-down via [`apply`], then encodes the final
//! result back to sRGB (mirror of the stamp pipeline's linear-space
//! compositing — `cpu_render.rs`). Blend math is defined in linear light
//! per the W3C *Compositing and Blending Level 1* spec; the HSL
//! non-separable modes use the spec's `SetLum`/`SetSat`/`ClipColor`
//! helpers verbatim.
//!
//! # Formula
//!
//! For every mode except the two specials, [`apply`] is the W3C simple
//! alpha-compositing-with-blending:
//!
//! ```text
//!   Cs' = (1 - αb)·Cs + αb·B(Cb, Cs)        // blended source color
//!   αo  = αs + αb·(1 - αs)                   // Porter-Duff source-over
//!   Co  = (αs·Cs' + αb·(1 - αs)·Cb) / αo     // straight (un-premultiplied)
//! ```
//!
//! where `B` is the per-mode blend function. `Behind` and `Clear` bypass
//! `B` (they are compositing operators, not blend functions).
//!
//! # Discriminants
//!
//! `#[repr(u8)]` with the popover order as the stable wire value, so a
//! future GPU compositor (`stamp.wgsl` sibling) can dispatch on the same
//! integer — mirror of [`crate::rendering_mode::RenderingMode`].

use serde::{Deserialize, Serialize};

/// Number of canonical blend modes (popover count). Keep in sync with the
/// `BlendMode` variants + `docs/Painter_projeto/02_layers.md` §2.2.
pub const MAX_BLEND_MODES: u8 = 22;

/// Layer blend mode. Discriminants follow the popover order (5 groups +
/// specials) and are the stable wire value shared with the GPU path.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum BlendMode {
    // Normal
    #[default]
    Normal = 0,
    // Darken group
    Multiply = 1,
    Darken = 2,
    ColorBurn = 3,
    LinearBurn = 4,
    // Lighten group
    Lighten = 5,
    Screen = 6,
    ColorDodge = 7,
    /// Linear Dodge.
    Add = 8,
    // Contrast group
    Overlay = 9,
    SoftLight = 10,
    HardLight = 11,
    VividLight = 12,
    LinearLight = 13,
    // Difference group
    Difference = 14,
    Exclusion = 15,
    // Color (HSL non-separable) group
    Hue = 16,
    Saturation = 17,
    Color = 18,
    Luminosity = 19,
    // Specials
    /// Paint only where the backdrop is transparent (composite under).
    Behind = 20,
    /// Destructive erase — reduces backdrop alpha by the source alpha.
    Clear = 21,
}

impl BlendMode {
    /// Map a wire discriminant back to a [`BlendMode`]; out-of-range
    /// falls back to [`BlendMode::Normal`] (mirror of
    /// `RenderingMode::from_u32`, so a corrupt cooked value never panics).
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Multiply,
            2 => Self::Darken,
            3 => Self::ColorBurn,
            4 => Self::LinearBurn,
            5 => Self::Lighten,
            6 => Self::Screen,
            7 => Self::ColorDodge,
            8 => Self::Add,
            9 => Self::Overlay,
            10 => Self::SoftLight,
            11 => Self::HardLight,
            12 => Self::VividLight,
            13 => Self::LinearLight,
            14 => Self::Difference,
            15 => Self::Exclusion,
            16 => Self::Hue,
            17 => Self::Saturation,
            18 => Self::Color,
            19 => Self::Luminosity,
            20 => Self::Behind,
            21 => Self::Clear,
            _ => Self::Normal,
        }
    }

    /// `true` for the four non-separable (HSL) modes, whose blend function
    /// mixes whole RGB triples rather than per-channel.
    #[must_use]
    pub fn is_hsl(self) -> bool {
        matches!(self, Self::Hue | Self::Saturation | Self::Color | Self::Luminosity)
    }

    /// Stable popover/wire discriminant.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Display name for the layer panel + blend-mode popover
    /// (`02_layers.md` §2.2 wording). UI string — English.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Multiply => "Multiply",
            Self::Darken => "Darken",
            Self::ColorBurn => "Color Burn",
            Self::LinearBurn => "Linear Burn",
            Self::Lighten => "Lighten",
            Self::Screen => "Screen",
            Self::ColorDodge => "Color Dodge",
            Self::Add => "Add",
            Self::Overlay => "Overlay",
            Self::SoftLight => "Soft Light",
            Self::HardLight => "Hard Light",
            Self::VividLight => "Vivid Light",
            Self::LinearLight => "Linear Light",
            Self::Difference => "Difference",
            Self::Exclusion => "Exclusion",
            Self::Hue => "Hue",
            Self::Saturation => "Saturation",
            Self::Color => "Color",
            Self::Luminosity => "Luminosity",
            Self::Behind => "Behind",
            Self::Clear => "Clear",
        }
    }
}

/// Composite `src` over `dst` under `mode`.
///
/// Inputs and output are **straight (non-premultiplied) linear-sRGB**
/// RGBA in `[0, 1]`. See the module docs for the formula.
#[must_use]
pub fn apply(mode: BlendMode, dst: [f32; 4], src: [f32; 4]) -> [f32; 4] {
    let ab = dst[3].clamp(0.0, 1.0);
    let as_ = src[3].clamp(0.0, 1.0);
    let cb = [dst[0], dst[1], dst[2]];
    let cs = [src[0], src[1], src[2]];

    match mode {
        // Erase: keep backdrop color, attenuate its alpha by the source's.
        BlendMode::Clear => return [cb[0], cb[1], cb[2], ab * (1.0 - as_)],
        // Paint under the backdrop: dst stays on top of src.
        BlendMode::Behind => return over(dst, src),
        _ => {}
    }

    // B(Cb, Cs) — blended color where both layers cover.
    let blended = if mode.is_hsl() {
        blend_hsl(mode, cb, cs)
    } else {
        [
            blend_sep(mode, cb[0], cs[0]),
            blend_sep(mode, cb[1], cs[1]),
            blend_sep(mode, cb[2], cs[2]),
        ]
    };

    let ao = as_ + ab * (1.0 - as_);
    if ao <= f32::EPSILON {
        return [0.0, 0.0, 0.0, 0.0];
    }
    let mut out = [0.0f32; 4];
    for (ch, o) in out.iter_mut().take(3).enumerate() {
        // Cs' = (1-αb)·Cs + αb·B
        let cs_eff = (1.0 - ab) * cs[ch] + ab * blended[ch];
        // premultiplied source-over, then un-premultiply by αo
        let co_premul = as_ * cs_eff + ab * (1.0 - as_) * cb[ch];
        *o = sanitize01(co_premul / ao);
    }
    out[3] = ao;
    out
}

/// Clamp to `[0, 1]`, mapping non-finite (NaN/±∞) to 0 — `f32::clamp`
/// passes NaN through, so a single corrupt input texel would otherwise
/// poison a whole layer. Defense-in-depth (audit W3 L-1).
#[inline]
fn sanitize01(v: f32) -> f32 {
    if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 }
}

/// Plain source-over (`Normal`) — used by `Behind` with swapped roles.
fn over(top: [f32; 4], bottom: [f32; 4]) -> [f32; 4] {
    let at = top[3].clamp(0.0, 1.0);
    let abm = bottom[3].clamp(0.0, 1.0);
    let ao = at + abm * (1.0 - at);
    if ao <= f32::EPSILON {
        return [0.0, 0.0, 0.0, 0.0];
    }
    let mut out = [0.0f32; 4];
    for (ch, o) in out.iter_mut().take(3).enumerate() {
        let premul = at * top[ch] + abm * (1.0 - at) * bottom[ch];
        *o = sanitize01(premul / ao);
    }
    out[3] = ao;
    out
}

// ---------------------------------------------------------------------------
// Separable (per-channel) blend functions — W3C Compositing L1 §9.
// ---------------------------------------------------------------------------

fn blend_sep(mode: BlendMode, cb: f32, cs: f32) -> f32 {
    match mode {
        BlendMode::Normal => cs,
        BlendMode::Multiply => cb * cs,
        BlendMode::Screen => screen(cb, cs),
        BlendMode::Overlay => hard_light(cs, cb), // overlay = hard-light w/ swapped operands
        BlendMode::Darken => cb.min(cs),
        BlendMode::Lighten => cb.max(cs),
        BlendMode::ColorDodge => color_dodge(cb, cs),
        BlendMode::ColorBurn => color_burn(cb, cs),
        BlendMode::HardLight => hard_light(cb, cs),
        BlendMode::SoftLight => soft_light(cb, cs),
        BlendMode::Difference => (cb - cs).abs(),
        BlendMode::Exclusion => cb + cs - 2.0 * cb * cs,
        BlendMode::LinearBurn => (cb + cs - 1.0).clamp(0.0, 1.0),
        BlendMode::Add => (cb + cs).min(1.0),
        BlendMode::LinearLight => (cb + 2.0 * cs - 1.0).clamp(0.0, 1.0),
        BlendMode::VividLight => {
            if cs <= 0.5 {
                color_burn(cb, (2.0 * cs).clamp(0.0, 1.0))
            } else {
                color_dodge(cb, (2.0 * cs - 1.0).clamp(0.0, 1.0))
            }
        }
        // HSL + specials never reach here.
        _ => cs,
    }
}

fn screen(cb: f32, cs: f32) -> f32 {
    cb + cs - cb * cs
}

fn hard_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        2.0 * cb * cs
    } else {
        1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
    }
}

fn color_dodge(cb: f32, cs: f32) -> f32 {
    if cb <= 0.0 {
        0.0
    } else if cs >= 1.0 {
        1.0
    } else {
        (cb / (1.0 - cs)).min(1.0)
    }
}

fn color_burn(cb: f32, cs: f32) -> f32 {
    if cb >= 1.0 {
        1.0
    } else if cs <= 0.0 {
        0.0
    } else {
        1.0 - ((1.0 - cb) / cs).min(1.0)
    }
}

fn soft_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb)
    } else {
        let d = if cb <= 0.25 {
            ((16.0 * cb - 12.0) * cb + 4.0) * cb
        } else {
            cb.sqrt()
        };
        cb + (2.0 * cs - 1.0) * (d - cb)
    }
}

// ---------------------------------------------------------------------------
// Non-separable (HSL) blend functions — W3C Compositing L1 §10.
// ---------------------------------------------------------------------------

fn blend_hsl(mode: BlendMode, cb: [f32; 3], cs: [f32; 3]) -> [f32; 3] {
    match mode {
        BlendMode::Hue => set_lum(set_sat(cs, sat(cb)), lum(cb)),
        BlendMode::Saturation => set_lum(set_sat(cb, sat(cs)), lum(cb)),
        BlendMode::Color => set_lum(cs, lum(cb)),
        BlendMode::Luminosity => set_lum(cb, lum(cs)),
        _ => cs,
    }
}

fn lum(c: [f32; 3]) -> f32 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

fn clip_color(c: [f32; 3]) -> [f32; 3] {
    let l = lum(c);
    let n = c[0].min(c[1]).min(c[2]);
    let x = c[0].max(c[1]).max(c[2]);
    let mut out = c;
    if n < 0.0 {
        let denom = l - n;
        if denom.abs() > f32::EPSILON {
            for c in &mut out {
                *c = l + (*c - l) * l / denom;
            }
        }
    }
    if x > 1.0 {
        let denom = x - l;
        if denom.abs() > f32::EPSILON {
            for c in &mut out {
                *c = l + (*c - l) * (1.0 - l) / denom;
            }
        }
    }
    out
}

fn set_lum(c: [f32; 3], l: f32) -> [f32; 3] {
    let d = l - lum(c);
    clip_color([c[0] + d, c[1] + d, c[2] + d])
}

fn sat(c: [f32; 3]) -> f32 {
    c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
}

/// W3C `SetSat`: stretch the mid/max channels to hit saturation `s`,
/// preserving the min→max ordering of the original channels.
fn set_sat(c: [f32; 3], s: f32) -> [f32; 3] {
    // Sort the three channels by value, remembering original indices.
    let mut idx = [0usize, 1, 2];
    idx.sort_by(|&a, &b| c[a].partial_cmp(&c[b]).unwrap_or(std::cmp::Ordering::Equal));
    let (i_min, i_mid, i_max) = (idx[0], idx[1], idx[2]);
    let mut out = [0.0f32; 3];
    if c[i_max] > c[i_min] {
        out[i_mid] = (c[i_mid] - c[i_min]) * s / (c[i_max] - c[i_min]);
        out[i_max] = s;
    } else {
        out[i_mid] = 0.0;
        out[i_max] = 0.0;
    }
    out[i_min] = 0.0;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    fn approx(a: [f32; 4], b: [f32; 4]) -> bool {
        (0..4).all(|i| (a[i] - b[i]).abs() < EPS)
    }

    #[test]
    fn from_u8_round_trips_all_variants() {
        for v in 0..MAX_BLEND_MODES {
            assert_eq!(BlendMode::from_u8(v).to_u8(), v, "discriminant {v}");
        }
    }

    #[test]
    fn from_u8_out_of_range_falls_back_to_normal() {
        assert_eq!(BlendMode::from_u8(MAX_BLEND_MODES), BlendMode::Normal);
        assert_eq!(BlendMode::from_u8(255), BlendMode::Normal);
    }

    #[test]
    fn every_mode_has_a_distinct_nonempty_name() {
        let mut seen = std::collections::BTreeSet::new();
        for v in 0..MAX_BLEND_MODES {
            let n = BlendMode::from_u8(v).name();
            assert!(!n.is_empty(), "mode {v} has empty name");
            assert!(seen.insert(n), "duplicate blend name {n:?}");
        }
        assert_eq!(seen.len(), MAX_BLEND_MODES as usize);
    }

    #[test]
    fn normal_over_opaque_is_source() {
        let dst = [0.2, 0.4, 0.6, 1.0];
        let src = [0.8, 0.1, 0.3, 1.0];
        assert!(approx(apply(BlendMode::Normal, dst, src), src));
    }

    #[test]
    fn fully_transparent_source_leaves_backdrop() {
        let dst = [0.2, 0.4, 0.6, 1.0];
        let src = [0.9, 0.9, 0.9, 0.0];
        // Every mode with a transparent source must return the backdrop.
        for v in 0..MAX_BLEND_MODES {
            let mode = BlendMode::from_u8(v);
            if mode == BlendMode::Clear {
                continue; // Clear with αs=0 is also a no-op; tested separately
            }
            let out = apply(mode, dst, src);
            assert!(approx(out, dst), "{mode:?} mutated backdrop on αs=0: {out:?}");
        }
    }

    #[test]
    fn multiply_by_white_is_identity_on_color() {
        let dst = [0.3, 0.5, 0.7, 1.0];
        let white = [1.0, 1.0, 1.0, 1.0];
        assert!(approx(apply(BlendMode::Multiply, dst, white), dst));
    }

    #[test]
    fn multiply_by_black_yields_black() {
        let dst = [0.3, 0.5, 0.7, 1.0];
        let black = [0.0, 0.0, 0.0, 1.0];
        assert!(approx(apply(BlendMode::Multiply, dst, black), [0.0, 0.0, 0.0, 1.0]));
    }

    #[test]
    fn screen_by_black_is_identity() {
        let dst = [0.3, 0.5, 0.7, 1.0];
        let black = [0.0, 0.0, 0.0, 1.0];
        assert!(approx(apply(BlendMode::Screen, dst, black), dst));
    }

    #[test]
    fn darken_and_lighten_pick_extremes() {
        let dst = [0.2, 0.8, 0.5, 1.0];
        let src = [0.6, 0.3, 0.5, 1.0];
        assert!(approx(
            apply(BlendMode::Darken, dst, src),
            [0.2, 0.3, 0.5, 1.0]
        ));
        assert!(approx(
            apply(BlendMode::Lighten, dst, src),
            [0.6, 0.8, 0.5, 1.0]
        ));
    }

    #[test]
    fn difference_with_self_is_black() {
        let c = [0.4, 0.6, 0.2, 1.0];
        assert!(approx(apply(BlendMode::Difference, c, c), [0.0, 0.0, 0.0, 1.0]));
    }

    #[test]
    fn add_saturates_at_one() {
        let dst = [0.7, 0.2, 0.9, 1.0];
        let src = [0.6, 0.1, 0.4, 1.0];
        let out = apply(BlendMode::Add, dst, src);
        assert!(approx(out, [1.0, 0.3, 1.0, 1.0]));
    }

    #[test]
    fn clear_zeros_alpha_where_source_opaque() {
        let dst = [0.3, 0.5, 0.7, 1.0];
        let src = [0.0, 0.0, 0.0, 1.0];
        let out = apply(BlendMode::Clear, dst, src);
        assert!(out[3].abs() < EPS, "Clear should zero alpha, got {}", out[3]);
    }

    #[test]
    fn behind_paints_under_transparent_backdrop() {
        // Backdrop transparent → Behind reveals the source fully.
        let dst = [0.0, 0.0, 0.0, 0.0];
        let src = [0.8, 0.1, 0.3, 1.0];
        assert!(approx(apply(BlendMode::Behind, dst, src), src));
        // Backdrop opaque → Behind keeps the backdrop on top.
        let dst2 = [0.2, 0.4, 0.6, 1.0];
        assert!(approx(apply(BlendMode::Behind, dst2, src), dst2));
    }

    #[test]
    fn luminosity_keeps_backdrop_hue_takes_source_lum() {
        let dst = [0.8, 0.2, 0.2, 1.0]; // reddish
        let src = [0.5, 0.5, 0.5, 1.0]; // neutral gray
        let out = apply(BlendMode::Luminosity, dst, src);
        // Output luminance ≈ source luminance.
        assert!((lum([out[0], out[1], out[2]]) - lum([src[0], src[1], src[2]])).abs() < 1e-3);
    }

    #[test]
    fn color_takes_backdrop_lum_source_chroma() {
        let dst = [0.5, 0.5, 0.5, 1.0];
        let src = [0.8, 0.2, 0.2, 1.0];
        let out = apply(BlendMode::Color, dst, src);
        assert!((lum([out[0], out[1], out[2]]) - lum([dst[0], dst[1], dst[2]])).abs() < 1e-3);
    }

    #[test]
    fn every_mode_distinct_from_normal_on_midtones() {
        // Each non-Normal mode must produce a pixel that differs from
        // Normal for at least one representative midtone pair (guards
        // against a copy-paste no-op blend fn). Distinctness, not golden.
        let dst = [0.35, 0.6, 0.25, 1.0];
        let src = [0.7, 0.3, 0.55, 1.0];
        let normal = apply(BlendMode::Normal, dst, src);
        for v in 1..MAX_BLEND_MODES {
            let mode = BlendMode::from_u8(v);
            let out = apply(mode, dst, src);
            assert!(
                !approx(out, normal),
                "{mode:?} produced the same output as Normal"
            );
        }
    }

    #[test]
    fn output_alpha_is_porter_duff_over() {
        let dst = [0.1, 0.2, 0.3, 0.4];
        let src = [0.5, 0.6, 0.7, 0.5];
        // αo = αs + αb(1-αs) = 0.5 + 0.4·0.5 = 0.7 for the non-special modes.
        for v in 0..MAX_BLEND_MODES {
            let mode = BlendMode::from_u8(v);
            if matches!(mode, BlendMode::Clear | BlendMode::Behind) {
                continue;
            }
            let out = apply(mode, dst, src);
            assert!((out[3] - 0.7).abs() < EPS, "{mode:?} αo wrong: {}", out[3]);
        }
    }

    #[test]
    fn nan_input_is_sanitized_not_propagated() {
        // A single corrupt (NaN) source channel must not poison the output
        // (audit W3 L-1 — `f32::clamp` passes NaN through).
        let out = apply(BlendMode::Normal, [0.2, 0.4, 0.6, 1.0], [f32::NAN, 0.5, 0.5, 1.0]);
        assert!(out.iter().all(|c| c.is_finite()), "NaN propagated: {out:?}");
        assert_eq!(out[0], 0.0, "NaN channel sanitized to 0");
    }
}
