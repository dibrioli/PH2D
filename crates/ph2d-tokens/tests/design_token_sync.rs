//! Wave 4 stage A+B — cross-validates that every public token API
//! (`Spacing::px()`, `Radius::px()`, `StrokeToken::px()`,
//! `Density::row_h_px()`, `TypeToken::px()`, `FontWeight::value()`,
//! `LineHeight::ratio()`, `LetterSpacing::em()`, plus the
//! `SECTION_GAP_PX`/`ICON_BTN_SIZE_PX`/`ROW_H_PX` chrome consts)
//! agrees with the canonical `docs/design/tokens.json` source.
//!
//! ## Why this exists
//!
//! `build.rs` parses tokens.json ad-hoc (zero build-deps) and emits
//! `crate::generated::*` consts. Enums in the runtime crate read those
//! consts. The danger is silent drift if someone edits tokens.json but
//! the build script's parser misses the change (a parser bug) — or
//! someone renames a JSON key and the build.rs `extract_object_block`
//! still finds *something* but returns wrong values.
//!
//! This test re-parses tokens.json with a **fully independent** parser
//! (`serde_json`, dev-only dep) and asserts that the public token API
//! agrees with the JSON. If they diverge → CI fails with an inline diff.
//!
//! ## Coverage
//!
//! - `spacing.{xxs..4xl}` ↔ `Spacing::*.px()`
//! - `radius.{xs..full}` ↔ `Radius::*.px()`
//! - `stroke.{hairline..heavy}` ↔ `StrokeToken::*.px()`
//! - `density.{compact..comfortable}` ↔ `Density::*.row_h_px()`
//! - `chrome.{row-h,icon-btn-size,section-gap}` ↔ `ROW_H_PX`,
//!   `ICON_BTN_SIZE_PX`, `SECTION_GAP_PX`
//! - `typography.size.*` ↔ `TypeToken::*.px()`
//! - `typography.weight.*` ↔ `FontWeight::*.value()`
//! - `typography.line.*` ↔ `LineHeight::*.ratio()`
//! - `typography.track.*` ↔ `LetterSpacing::*.em()`

use std::path::PathBuf;

use ph2d_tokens::{
    Density, FontWeight, ICON_BTN_SIZE_PX, LetterSpacing, LineHeight, ROW_H_PX, Radius,
    SECTION_GAP_PX, Spacing, StrokeToken, TypeToken,
};
use serde_json::Value;

/// Tolerance for f32 equality. Codegen emits up to 6 decimal places;
/// f32→f64 conversion loses precision near the 7th decimal.
const EPSILON: f64 = 1e-5;

/// Locate `docs/design/tokens.json` from `CARGO_MANIFEST_DIR`.
fn tokens_json_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .join("docs/design/tokens.json")
}

/// Load and parse tokens.json with `serde_json` (independent from
/// build.rs's ad-hoc parser).
fn load_tokens() -> Value {
    let path = tokens_json_path();
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// Read a JSON entry that may be either a raw number (`4`, `0.5`) or a
/// `"4px"`/`"-0.02em"` string. Strip unit suffixes.
fn extract_numeric(v: &Value) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().expect("numeric value fits in f64"),
        Value::String(s) => {
            let stripped = s.trim_end_matches("px").trim_end_matches("em");
            stripped
                .parse::<f64>()
                .unwrap_or_else(|_| panic!("cannot parse JSON string `{s}` as number"))
        }
        _ => panic!("expected number or string, got {v:?}"),
    }
}

/// Assert that `enum_value` (from a Rust token API) matches `json_value`
/// (from tokens.json) within `EPSILON`. Failure prints the JSON key for
/// quick diagnosis.
fn assert_close(json_key: &str, json_value: f64, enum_value: f64) {
    let diff = (json_value - enum_value).abs();
    assert!(
        diff <= EPSILON,
        "drift at `{json_key}`: tokens.json={json_value} but Rust API={enum_value} (diff {diff})"
    );
}

#[test]
fn spacing_enum_matches_tokens_json() {
    let tokens = load_tokens();
    let spacing = &tokens["spacing"];
    let pairs: &[(&str, Spacing)] = &[
        ("xxs", Spacing::Xxs),
        ("xs", Spacing::Xs),
        ("sm", Spacing::Sm),
        ("md", Spacing::Md),
        ("lg", Spacing::Lg),
        ("xl", Spacing::Xl),
        ("2xl", Spacing::Xl2),
        ("3xl", Spacing::Xl3),
        ("4xl", Spacing::Xl4),
    ];
    for (key, variant) in pairs {
        let json_value = extract_numeric(&spacing[*key]);
        assert_close(&format!("spacing.{key}"), json_value, variant.px() as f64);
    }
}

#[test]
fn radius_enum_matches_tokens_json() {
    let tokens = load_tokens();
    let radius = &tokens["radius"];
    let pairs: &[(&str, Radius)] = &[
        ("xs", Radius::Xs),
        ("sm", Radius::Sm),
        ("md", Radius::Md),
        ("lg", Radius::Lg),
        ("xl", Radius::Xl),
        ("2xl", Radius::Xl2),
        ("full", Radius::Full),
    ];
    for (key, variant) in pairs {
        let json_value = extract_numeric(&radius[*key]);
        assert_close(&format!("radius.{key}"), json_value, variant.px() as f64);
    }
}

#[test]
fn stroke_enum_matches_tokens_json() {
    let tokens = load_tokens();
    let stroke = &tokens["stroke"];
    let pairs: &[(&str, StrokeToken)] = &[
        ("hairline", StrokeToken::Hairline),
        ("thin", StrokeToken::Thin),
        ("default", StrokeToken::Default),
        ("thick", StrokeToken::Thick),
        ("heavy", StrokeToken::Heavy),
    ];
    for (key, variant) in pairs {
        let json_value = extract_numeric(&stroke[*key]);
        assert_close(&format!("stroke.{key}"), json_value, variant.px() as f64);
    }
}

#[test]
fn density_enum_matches_tokens_json() {
    let tokens = load_tokens();
    let density = &tokens["density"];
    let pairs: &[(&str, Density)] = &[
        ("compact", Density::Compact),
        ("cozy", Density::Cozy),
        ("comfortable", Density::Comfortable),
    ];
    for (key, variant) in pairs {
        let json_value = extract_numeric(&density[*key]);
        assert_close(
            &format!("density.{key}"),
            json_value,
            variant.row_h_px() as f64,
        );
    }
}

#[test]
fn chrome_consts_match_tokens_json() {
    let tokens = load_tokens();
    let chrome = &tokens["chrome"];
    assert_close(
        "chrome.row-h",
        extract_numeric(&chrome["row-h"]),
        ROW_H_PX as f64,
    );
    assert_close(
        "chrome.icon-btn-size",
        extract_numeric(&chrome["icon-btn-size"]),
        ICON_BTN_SIZE_PX as f64,
    );
    assert_close(
        "chrome.section-gap",
        extract_numeric(&chrome["section-gap"]),
        SECTION_GAP_PX as f64,
    );
}

#[test]
fn typography_size_enum_matches_tokens_json() {
    let tokens = load_tokens();
    let size = &tokens["typography"]["size"];
    let pairs: &[(&str, TypeToken)] = &[
        ("xxs", TypeToken::Xxs),
        ("xs", TypeToken::Xs),
        ("sm", TypeToken::Sm),
        ("base", TypeToken::Base),
        ("md", TypeToken::Md),
        ("lg", TypeToken::Lg),
        ("xl", TypeToken::Xl),
        ("2xl", TypeToken::Xl2),
        ("3xl", TypeToken::Xl3),
    ];
    for (key, variant) in pairs {
        let json_value = extract_numeric(&size[*key]);
        assert_close(
            &format!("typography.size.{key}"),
            json_value,
            variant.px() as f64,
        );
    }
}

#[test]
fn typography_weight_enum_matches_tokens_json() {
    let tokens = load_tokens();
    let weight = &tokens["typography"]["weight"];
    let pairs: &[(&str, FontWeight)] = &[
        ("regular", FontWeight::Regular),
        ("medium", FontWeight::Medium),
        ("semibold", FontWeight::Semibold),
        ("bold", FontWeight::Bold),
    ];
    for (key, variant) in pairs {
        let json_value = extract_numeric(&weight[*key]);
        assert_close(
            &format!("typography.weight.{key}"),
            json_value,
            f64::from(variant.value()),
        );
    }
}

#[test]
fn typography_line_enum_matches_tokens_json() {
    let tokens = load_tokens();
    let line = &tokens["typography"]["line"];
    let pairs: &[(&str, LineHeight)] = &[
        ("tight", LineHeight::Tight),
        ("snug", LineHeight::Snug),
        ("normal", LineHeight::Normal),
        ("loose", LineHeight::Loose),
    ];
    for (key, variant) in pairs {
        let json_value = extract_numeric(&line[*key]);
        assert_close(
            &format!("typography.line.{key}"),
            json_value,
            variant.ratio() as f64,
        );
    }
}

#[test]
fn typography_track_enum_matches_tokens_json() {
    let tokens = load_tokens();
    let track = &tokens["typography"]["track"];
    let pairs: &[(&str, LetterSpacing)] = &[
        ("tight", LetterSpacing::Tight),
        ("normal", LetterSpacing::Normal),
        ("wide", LetterSpacing::Wide),
        ("caps", LetterSpacing::Caps),
    ];
    for (key, variant) in pairs {
        let json_value = extract_numeric(&track[*key]);
        assert_close(
            &format!("typography.track.{key}"),
            json_value,
            variant.em() as f64,
        );
    }
}
