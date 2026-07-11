//! Unit tests for [`super`] (`color.rs`) — extracted to a sibling module
//! (`#[path]`) so `color.rs` stays under the workspace LOC cap.

use super::*;

#[test]
fn black_white_contrast_is_21() {
    let black = Color::from_hex(0x000000);
    let white = Color::from_hex(0xFFFFFF);
    let ratio = black.contrast_ratio(&white);
    assert!((ratio - 21.0).abs() < 0.01, "expected 21.0, got {ratio}");
}

#[test]
fn from_hex_alpha_round_trips() {
    let c = Color::from_hex_alpha(0x12_34_56_78);
    assert_eq!(c.r, 0x12);
    assert_eq!(c.g, 0x34);
    assert_eq!(c.b, 0x56);
    assert_eq!(c.a, 0x78);
}

#[test]
fn from_hex_defaults_alpha_opaque() {
    let c = Color::from_hex(0xFF0000);
    assert_eq!(c.a, 0xFF);
}

#[test]
fn oklch_white_round_trips_to_white_ish() {
    // L=1, C=0 should yield pure white in sRGB.
    let [r, g, b] = oklch_to_srgb(1.0, 0.0, 0.0);
    assert_eq!((r, g, b), (255, 255, 255));
}

#[test]
fn oklch_black_round_trips_to_black() {
    // L=0, C=0 should yield pure black in sRGB.
    let [r, g, b] = oklch_to_srgb(0.0, 0.0, 0.0);
    assert_eq!((r, g, b), (0, 0, 0));
}

#[test]
fn oklch_mid_gray_is_neutral() {
    // C=0 always yields R=G=B (achromatic). L=0.5 in OKLAB maps to
    // sRGB ~99 (perceptually "half of white" — not 128, because
    // OKLAB is perceptually uniform, not linear in sRGB).
    let [r, g, b] = oklch_to_srgb(0.5, 0.0, 0.0);
    assert_eq!(r, g);
    assert_eq!(g, b);
    assert!((90..=110).contains(&r), "expected ~99, got {r}");
}

/// Every timeline token (W2.E9) resolves in every theme. `resolve` panics
/// on a key that's in the enum but missing from a theme's JSON table, so
/// this is the permanent guard against an enum/`tokens.json` mismatch —
/// including the Workshop *overrides* for the accent-derived slots. Asserts
/// resolution only, never a specific value, so a later retint won't fight it.
#[test]
fn every_timeline_token_resolves_in_all_themes() {
    const TIMELINE: &[ColorToken] = &[
        ColorToken::TimelineRulerBg,
        ColorToken::TimelineRulerTick,
        ColorToken::TimelinePlayhead,
        ColorToken::TimelineRowAlt,
        ColorToken::TimelineKey,
        ColorToken::TimelineKeySelected,
        ColorToken::TimelineKeyActive,
        ColorToken::TimelineCurve,
        ColorToken::TimelineHandle,
        ColorToken::TimelineHandleLine,
        ColorToken::TimelineLoopRegion,
        ColorToken::TimelineLoopBrace,
        ColorToken::TimelineMarker,
        ColorToken::TimelineSummaryKey,
        ColorToken::TimelineSummaryRing,
        ColorToken::TimelineMissing,
    ];
    for theme in [
        Theme::Forge,
        Theme::Workshop,
        Theme::Sunstone,
        Theme::Blueprint,
    ] {
        for &tok in TIMELINE {
            // Panics here if the key is absent for this theme.
            let _ = tok.resolve(theme);
        }
    }
}

/// The 9 accent-derived timeline slots must carry a Workshop *override* —
/// otherwise they'd inherit forge's pink and the timeline would visibly
/// change hue under Workshop (a silent regression the seed intends to avoid).
/// Guards it by asserting Workshop differs from Forge for those slots (they
/// were seeded byte-identical to accent/-soft/-press, which Workshop retints).
#[test]
fn workshop_retints_the_accent_derived_timeline_slots() {
    for tok in [
        ColorToken::TimelinePlayhead,
        ColorToken::TimelineKeySelected,
        ColorToken::TimelineKeyActive,
        ColorToken::TimelineCurve,
        ColorToken::TimelineHandle,
        ColorToken::TimelineHandleLine,
        ColorToken::TimelineLoopRegion,
        ColorToken::TimelineLoopBrace,
        ColorToken::TimelineSummaryRing,
    ] {
        let forge = tok.resolve(Theme::Forge);
        let workshop = tok.resolve(Theme::Workshop);
        assert_ne!(
            (forge.r, forge.g, forge.b),
            (workshop.r, workshop.g, workshop.b),
            "{tok:?} must have a Workshop override, not inherit forge"
        );
    }
}

/// **WCAG 2.2 AA gate** — text-on-bg1 contrast ≥ 4.5:1 across the 4 themes.
#[test]
fn text1_on_bg1_meets_aa_in_all_themes() {
    for theme in [
        Theme::Forge,
        Theme::Workshop,
        Theme::Sunstone,
        Theme::Blueprint,
    ] {
        let bg = ColorToken::Bg1.resolve(theme);
        let fg = ColorToken::Text1.resolve(theme);
        let ratio = bg.contrast_ratio(&fg);
        assert!(
            ratio >= 4.5,
            "{theme:?}: text-1 on bg-1 = {ratio:.2}:1, need ≥ 4.5"
        );
    }
}

#[test]
fn text2_on_bg1_meets_aa_in_all_themes() {
    for theme in [
        Theme::Forge,
        Theme::Workshop,
        Theme::Sunstone,
        Theme::Blueprint,
    ] {
        let bg = ColorToken::Bg1.resolve(theme);
        let fg = ColorToken::Text2.resolve(theme);
        let ratio = bg.contrast_ratio(&fg);
        assert!(
            ratio >= 4.5,
            "{theme:?}: text-2 on bg-1 = {ratio:.2}:1, need ≥ 4.5"
        );
    }
}

/// WCAG SC 1.4.11 — non-text UI components (focus rings, borders).
#[test]
fn border_emph_meets_ui_aa_in_all_themes() {
    for theme in [
        Theme::Forge,
        Theme::Workshop,
        Theme::Sunstone,
        Theme::Blueprint,
    ] {
        let bg = ColorToken::Bg1.resolve(theme);
        let fg = ColorToken::BorderEmph.resolve(theme);
        let ratio = bg.contrast_ratio(&fg);
        assert!(
            ratio >= 3.0,
            "{theme:?}: border-emph on bg-1 = {ratio:.2}:1, need ≥ 3.0"
        );
    }
}

#[test]
fn accent_meets_ui_aa_in_all_themes() {
    for theme in [
        Theme::Forge,
        Theme::Workshop,
        Theme::Sunstone,
        Theme::Blueprint,
    ] {
        let bg = ColorToken::Bg1.resolve(theme);
        let fg = ColorToken::Accent.resolve(theme);
        let ratio = bg.contrast_ratio(&fg);
        assert!(
            ratio >= 3.0,
            "{theme:?}: accent on bg-1 = {ratio:.2}:1, need ≥ 3.0"
        );
    }
}

#[test]
fn color_value_constants_match_extremes() {
    assert_eq!(ColorValue::BLACK.rgba, [0, 0, 0, 255]);
    assert_eq!(ColorValue::WHITE.rgba, [255, 255, 255, 255]);
    assert_eq!(ColorValue::TRANSPARENT.rgba, [0, 0, 0, 0]);
}

#[test]
fn color_value_from_rgba_round_trips_alpha() {
    let cv = ColorValue::from_rgba8(120, 80, 200, 128);
    assert_eq!(cv.rgba, [120, 80, 200, 128]);
    assert!((cv.oklch.3 - 128.0 / 255.0).abs() < 1e-6);
}

#[test]
fn color_value_oklch_round_trip_within_one_byte() {
    // sRGB → OKLCH → sRGB must round-trip within ±1 byte per
    // channel (rounding noise from the linearization step).
    for sample in [
        [0u8, 0, 0],
        [255, 255, 255],
        [128, 128, 128],
        [255, 0, 0],
        [0, 255, 0],
        [0, 0, 255],
        [231, 231, 231],
    ] {
        let cv = ColorValue::from_rgba8(sample[0], sample[1], sample[2], 255);
        let (l, c, h, _) = cv.oklch;
        let cv2 = ColorValue::from_oklch(l, c, h, 1.0);
        for i in 0..3 {
            let delta = (cv.rgba[i] as i32 - cv2.rgba[i] as i32).abs();
            assert!(
                delta <= 1,
                "channel {i} drifted by {delta} bytes (sRGB→OKLCH→sRGB) for {sample:?}"
            );
        }
    }
}

#[test]
fn srgb_to_oklch_red_yields_red_hue() {
    // OKLCH hue for pure red sits around 29 degrees.
    let (_, _, h) = srgb_to_oklch(255, 0, 0);
    assert!(
        (20.0..40.0).contains(&h),
        "expected ~29° hue for red, got {h}"
    );
}

#[test]
fn srgb_to_oklch_white_zero_chroma() {
    let (l, c, _) = srgb_to_oklch(255, 255, 255);
    assert!(l > 0.99);
    assert!(c < 0.001);
}

#[test]
fn color_value_from_oklch_clamps_alpha() {
    let cv = ColorValue::from_oklch(0.5, 0.0, 0.0, 2.0);
    assert_eq!(cv.rgba[3], 255);
    let cv = ColorValue::from_oklch(0.5, 0.0, 0.0, -1.0);
    assert_eq!(cv.rgba[3], 0);
}
