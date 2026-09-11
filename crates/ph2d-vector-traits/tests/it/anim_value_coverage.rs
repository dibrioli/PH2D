//! Coverage test per ADR-0056 §2.5 criterion 4: every `AnimValue` variant
//! is interpolable via `LinearInterp::lerp`.

use glam::{Vec2, Vec3};
use ph2d_color::OklchColor;
use ph2d_vector_traits::{AnimValue, LinearInterp};

#[test]
fn float_lerp_midpoint() {
    let a = AnimValue::Float(0.0);
    let b = AnimValue::Float(10.0);
    assert_eq!(AnimValue::lerp(a, b, 0.5), AnimValue::Float(5.0));
}

#[test]
fn float_lerp_endpoints() {
    let a = AnimValue::Float(2.0);
    let b = AnimValue::Float(8.0);
    assert_eq!(AnimValue::lerp(a, b, 0.0), a);
    assert_eq!(AnimValue::lerp(a, b, 1.0), b);
}

#[test]
fn vec2_lerp_midpoint() {
    let a = AnimValue::Vec2(Vec2::new(0.0, 0.0));
    let b = AnimValue::Vec2(Vec2::new(2.0, 4.0));
    assert_eq!(
        AnimValue::lerp(a, b, 0.5),
        AnimValue::Vec2(Vec2::new(1.0, 2.0))
    );
}

#[test]
fn vec3_lerp_endpoints() {
    let a = AnimValue::Vec3(Vec3::ZERO);
    let b = AnimValue::Vec3(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(AnimValue::lerp(a, b, 0.0), a);
    assert_eq!(AnimValue::lerp(a, b, 1.0), b);
}

#[test]
fn color_lerp_short_hue_arc() {
    // 350° → 10° must cross 0° (20° arc), not the long way (340° arc).
    let a = AnimValue::Color(OklchColor::opaque(0.5, 0.1, 350.0));
    let b = AnimValue::Color(OklchColor::opaque(0.5, 0.1, 10.0));
    let mid = AnimValue::lerp(a, b, 0.5);
    let AnimValue::Color(c) = mid else {
        panic!("expected Color variant, got {mid:?}");
    };
    let dist_from_zero = c.h.min(360.0 - c.h);
    assert!(
        dist_from_zero < 1.0,
        "hue {} should be near 0° (short arc), not 180° (long arc)",
        c.h
    );
}

#[test]
fn color_lerp_lightness_chroma_linear() {
    let a = AnimValue::Color(OklchColor::opaque(0.0, 0.0, 100.0));
    let b = AnimValue::Color(OklchColor::opaque(1.0, 0.4, 100.0));
    let mid = AnimValue::lerp(a, b, 0.5);
    let AnimValue::Color(c) = mid else {
        panic!("expected Color variant, got {mid:?}");
    };
    assert!((c.l - 0.5).abs() < 1e-6, "lightness midpoint: {}", c.l);
    assert!((c.c - 0.2).abs() < 1e-6, "chroma midpoint: {}", c.c);
}

#[test]
fn bool_lerp_step_at_half() {
    let a = AnimValue::Bool(false);
    let b = AnimValue::Bool(true);
    assert_eq!(AnimValue::lerp(a, b, 0.49), AnimValue::Bool(false));
    assert_eq!(AnimValue::lerp(a, b, 0.51), AnimValue::Bool(true));
}

#[test]
fn enum_lerp_step_at_half() {
    let a = AnimValue::Enum(0);
    let b = AnimValue::Enum(7);
    assert_eq!(AnimValue::lerp(a, b, 0.0), AnimValue::Enum(0));
    assert_eq!(AnimValue::lerp(a, b, 1.0), AnimValue::Enum(7));
    assert_eq!(AnimValue::lerp(a, b, 0.49), AnimValue::Enum(0));
    assert_eq!(AnimValue::lerp(a, b, 0.51), AnimValue::Enum(7));
}

#[test]
fn variant_mismatch_snaps_to_second_value() {
    // Canonical fallback: variant mismatch returns `b` unchanged.
    let a = AnimValue::Float(5.0);
    let b = AnimValue::Bool(true);
    assert_eq!(AnimValue::lerp(a, b, 0.3), AnimValue::Bool(true));
}
