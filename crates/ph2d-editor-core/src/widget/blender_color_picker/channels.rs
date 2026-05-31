//! Legacy `paint_slider_row` (used by the non-store
//! `paint_blender_color_picker` path) + `rgba_to_hsv` /
//! `hsv_to_rgba8` color-space helpers. The interactive channel
//! row is now drawn by `crate::widget::paint_slider_with_chip`.

use crate::paint::{paint_text_centered, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ColorValue, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

pub fn paint_slider_row(
    label: &str,
    value: f32,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let label_w = 70.0;
    let val_w = 60.0;
    let track_x = rect.x + label_w + Spacing::Sm.px();
    let track_w = rect.w - label_w - val_w - Spacing::Sm.px() * 2.0;
    let label_rect = Rect::new(rect.x, rect.y, label_w, rect.h);
    let track_rect = Rect::new(track_x, rect.y + 6.0, track_w, rect.h - 12.0);
    let val_rect = Rect::new(rect.x + rect.w - val_w, rect.y, val_w, rect.h);

    // Plain text label, no pill background — the previous
    // AccentPress fill made every channel label read as a "selected
    // button". Channel rows are not interactive at the label, so a
    // chrome-less label keeps the eye on the slider track.
    paint_text_centered(
        text_system,
        scene,
        label,
        label_rect,
        TypeToken::Xs.px() - 1.0,
        resolve(ColorToken::Text2, theme),
    );

    // Canonical slider track (Bg2 + Accent fill) — shared with every
    // other slider in the app. Was a one-off Border-filled track.
    crate::widget::paint_slider_track(
        track_rect,
        value,
        crate::widget::SliderOrientation::Horizontal,
        scene,
        theme,
    );

    // Canonical numeric chip (single source of truth — same chip the
    // app-wide `paint_slider_with_chip` uses).
    let display = format!("{value:.3}");
    crate::widget::paint_number_chip(
        val_rect,
        crate::widget::TextInputState::Normal,
        value as f64,
        Some(&display),
        None,
        0,
        None,
        scene,
        text_system,
        theme,
    );
}

// `paint_channel_chip` was the picker-local interactive chip; it
// has been folded into `crate::widget::paint_number_chip`
// (the canonical app-wide chip used by `paint_slider_with_chip`).
// Callers should use the new module instead.

/// Absolute upper bound on OKLCH chroma searched for the in-gamut
/// maximum. The sRGB gamut never exceeds ~0.37; 0.4 is a safe ceiling.
pub const OKLCH_CHROMA_MAX: f32 = 0.4;
/// Hue is stored in degrees; normalize against a full turn.
pub const OKLCH_HUE_MAX: f32 = 360.0;

/// Largest OKLCH chroma representable in sRGB for the given lightness +
/// hue, found by binary search over [`oklch_in_gamut`]. The Chroma
/// slider is normalized against this (rather than a fixed ceiling) so
/// its top — `1.0` — always lands on the most-saturated color that L+H
/// can actually display, instead of pinning early as the fixed-ceiling
/// scale did (most L/H reach well under 0.4 in sRGB).
pub fn max_in_gamut_chroma(l: f64, h_deg: f64) -> f64 {
    // L outside (0,1) has no representable chroma (pure black/white).
    if l <= 0.0 || l >= 1.0 {
        return 0.0;
    }
    let mut lo = 0.0_f64;
    let mut hi = OKLCH_CHROMA_MAX as f64;
    // 20 bisections → ~4e-7 precision, far below 8-bit quantization.
    for _ in 0..20 {
        let mid = (lo + hi) * 0.5;
        if ph2d_tokens::oklch_in_gamut(l, mid, h_deg) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// A [`ColorValue`]'s STORED OKLCH `(L,C,H,A)` as channel norms in 0..1
/// for the uniform slider/chip model: `[L, C/maxC(L,H), H/360, A]`.
///
/// Reads `value.oklch` directly rather than re-deriving from `rgba` —
/// `ColorValue::from_oklch` keeps the exact OKLCH the user dialed in, so
/// the chroma + hue survive even when the sRGB form is gray or gamut-
/// clamped. (Re-deriving via `srgb_to_oklch(value.rgba)` lost hue at
/// chroma≈0 and snapped the chroma thumb back through 8-bit
/// quantization — that was the "can't move Chroma/Hue until I change
/// Lightness" bug.) Chroma is **gamut-relative** (see
/// [`max_in_gamut_chroma`]) so the slider top is always reachable.
pub fn oklch_norm_channels(oklch: (f64, f64, f64, f64)) -> [f32; 4] {
    let (l, c, h, a) = oklch;
    let max_c = max_in_gamut_chroma(l, h);
    let c_norm = if max_c > 1e-6 {
        (c / max_c).clamp(0.0, 1.0) as f32
    } else {
        0.0
    };
    [
        (l as f32).clamp(0.0, 1.0),
        c_norm,
        (h as f32 / OKLCH_HUE_MAX).clamp(0.0, 1.0),
        (a as f32).clamp(0.0, 1.0),
    ]
}

/// Edit one OKLCH channel (0=L,1=C,2=H,3=A) of `oklch` by normalized
/// `norm` (0..1) and return a [`ColorValue`] built via
/// [`ColorValue::from_oklch`] — so the edited OKLCH is stored EXACTLY
/// (round-trip-free) alongside the clamped sRGB. The other channels are
/// preserved, so dragging Hue at chroma≈0 still records the hue (it
/// shows once Chroma is raised) and dragging Chroma never snaps back.
/// Chroma denormalizes against [`max_in_gamut_chroma`] for the current
/// L+H, so `norm = 1.0` reaches the gamut boundary.
pub fn oklch_set_channel(oklch: (f64, f64, f64, f64), idx: u8, norm: f32) -> ColorValue {
    let (mut l, mut c, mut h, mut a) = oklch;
    let n = norm.clamp(0.0, 1.0) as f64;
    match idx {
        0 => l = n,
        1 => c = n * max_in_gamut_chroma(l, h),
        2 => h = n * OKLCH_HUE_MAX as f64,
        3 => a = n,
        _ => {}
    }
    ColorValue::from_oklch(l, c, h, a)
}

pub fn rgba_to_hsv(rgba: [u8; 4]) -> (f32, f32, f32, f32) {
    let r = rgba[0] as f32 / 255.0;
    let g = rgba[1] as f32 / 255.0;
    let b = rgba[2] as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let v = max;
    let s = if max == 0.0 { 0.0 } else { (max - min) / max };
    let h = if (max - min).abs() < f32::EPSILON {
        0.0
    } else if (max - r).abs() < f32::EPSILON {
        ((g - b) / (max - min) % 6.0) / 6.0
    } else if (max - g).abs() < f32::EPSILON {
        ((b - r) / (max - min) + 2.0) / 6.0
    } else {
        ((r - g) / (max - min) + 4.0) / 6.0
    };
    let h = h.rem_euclid(1.0);
    (h, s, v, rgba[3] as f32 / 255.0)
}

/// Inverse of [`rgba_to_hsv`]: HSV+A in 0..=1 → 8-bit RGBA.
pub fn hsv_to_rgba8(h: f32, s: f32, v: f32, a: f32) -> [u8; 4] {
    let h = (h * 6.0).rem_euclid(6.0);
    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h.floor() as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [
        (((r + m) * 255.0).round()).clamp(0.0, 255.0) as u8,
        (((g + m) * 255.0).round()).clamp(0.0, 255.0) as u8,
        (((b + m) * 255.0).round()).clamp(0.0, 255.0) as u8,
        ((a.clamp(0.0, 1.0) * 255.0).round()) as u8,
    ]
}
