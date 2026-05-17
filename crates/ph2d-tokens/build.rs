//! build.rs — gera `OUT_DIR/tokens_generated.rs` lendo
//! `docs/design/tokens.json` (canonical source-of-truth).
//!
//! Wave 2 PR 11.1. tokens.json é o que designer (Claude Design / Enio)
//! edita; este script transforma em const arrays Rust consumidos por
//! `color.rs::ColorToken::resolve`, `spacing.rs::Spacing`, etc.
//!
//! ## Parser
//!
//! JSON ad-hoc line-based — zero build-deps. tokens.json tem schema
//! estável + indentação consistente, então pattern matching simples
//! (`"key": "value"` por linha) basta. Não suportamos JSON arbitrário;
//! suportamos exatamente o schema de tokens.json. Se o schema mudar
//! (e.g., embed structures), este parser precisa estender.
//!
//! ## Output
//!
//! ```rust
//! pub struct OklchRaw { pub l: f64, pub c: f64, pub h: f64, pub a: f64 }
//! pub const COLORS_FORGE: &[(&str, OklchRaw)] = &[ ... ];
//! pub const COLORS_WORKSHOP: &[(&str, OklchRaw)] = &[ ... ];  // $inherits resolved
//! pub const COLORS_SUNSTONE: &[(&str, OklchRaw)] = &[ ... ];
//! pub const COLORS_BLUEPRINT: &[(&str, OklchRaw)] = &[ ... ];
//! ```
//!
//! Consumer in `src/color.rs`: `ColorToken::resolve_forge` looks up by
//! key string. O(n) over 33 entries = ~ns; not a hot path.
//!
//! ## Invalidação
//!
//! `cargo:rerun-if-changed=` em tokens.json + build.rs garante rebuild
//! quando design canonical muda.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Locate tokens.json — relative to crate root (CARGO_MANIFEST_DIR).
    let crate_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let tokens_path = crate_root
        .parent() // crates/
        .unwrap()
        .parent() // workspace root
        .unwrap()
        .join("docs/design/tokens.json");

    println!("cargo:rerun-if-changed={}", tokens_path.display());
    println!("cargo:rerun-if-changed=build.rs");

    let json = fs::read_to_string(&tokens_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", tokens_path.display()));

    let themes = parse_themes(&json);
    let spacing = parse_scalar_block(&json, "spacing");
    let radius = parse_scalar_block(&json, "radius");
    let stroke = parse_scalar_block(&json, "stroke");
    let density = parse_scalar_block(&json, "density");
    let chrome = parse_scalar_block(&json, "chrome");
    let typography_size = parse_scalar_block_with_px_suffix(&json, "typography", "size");
    let typography_weight = parse_scalar_block_with_path(&json, "typography", "weight");
    let typography_line = parse_scalar_block_with_path(&json, "typography", "line");
    let typography_track = parse_em_block(&json, "typography", "track");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_path = out_dir.join("tokens_generated.rs");
    fs::write(
        &out_path,
        emit_rust(
            &themes,
            &spacing,
            &radius,
            &stroke,
            &density,
            &chrome,
            &typography_size,
            &typography_weight,
            &typography_line,
            &typography_track,
        ),
    )
    .unwrap_or_else(|e| panic!("write {}: {e}", out_path.display()));
}

/// Parse a top-level scalar block (e.g. `"spacing": { "xs": 4, "sm": 6, ... }`)
/// into a key→f64 map. Numeric values only; no unit suffix. Panics if the
/// block is missing — Wave 4 adds these as required sections.
fn parse_scalar_block(json: &str, key: &str) -> BTreeMap<String, f64> {
    let body = extract_object_block(json, key)
        .unwrap_or_else(|| panic!("tokens.json missing top-level `{key}` block"));
    let out = parse_pairs(&body, parse_scalar_value);
    if out.is_empty() {
        panic!("tokens.json `{key}` block parsed empty — schema drift?");
    }
    out
}

/// Parse a nested scalar block under `parent.child` where values are
/// integers/floats (used for `typography.weight`, `typography.line`).
fn parse_scalar_block_with_path(json: &str, parent: &str, child: &str) -> BTreeMap<String, f64> {
    let parent_body = extract_object_block(json, parent)
        .unwrap_or_else(|| panic!("tokens.json missing `{parent}` block"));
    let child_body = extract_object_block(&parent_body, child)
        .unwrap_or_else(|| panic!("tokens.json missing `{parent}.{child}` block"));
    let out = parse_pairs(&child_body, parse_scalar_value);
    if out.is_empty() {
        panic!("tokens.json `{parent}.{child}` block parsed empty — schema drift?");
    }
    out
}

/// Parse a nested block where values are strings with `"px"` suffix,
/// like `"xxs": "10px"`. Used for `typography.size`.
fn parse_scalar_block_with_px_suffix(
    json: &str,
    parent: &str,
    child: &str,
) -> BTreeMap<String, f64> {
    let parent_body = extract_object_block(json, parent)
        .unwrap_or_else(|| panic!("tokens.json missing `{parent}` block"));
    let child_body = extract_object_block(&parent_body, child)
        .unwrap_or_else(|| panic!("tokens.json missing `{parent}.{child}` block"));
    let out = parse_pairs(&child_body, parse_px_string_value);
    if out.is_empty() {
        panic!("tokens.json `{parent}.{child}` block parsed empty — schema drift?");
    }
    out
}

/// Parse a nested block where values are strings with `"em"` suffix or
/// the special value `"0"`. Used for `typography.track`.
fn parse_em_block(json: &str, parent: &str, child: &str) -> BTreeMap<String, f64> {
    let parent_body = extract_object_block(json, parent)
        .unwrap_or_else(|| panic!("tokens.json missing `{parent}` block"));
    let child_body = extract_object_block(&parent_body, child)
        .unwrap_or_else(|| panic!("tokens.json missing `{parent}.{child}` block"));
    let out = parse_pairs(&child_body, parse_em_string_value);
    if out.is_empty() {
        panic!("tokens.json `{parent}.{child}` block parsed empty — schema drift?");
    }
    out
}

/// Walk a flat block body splitting on `,` and parsing each `"key": value`
/// pair via `parse_value`. Robust to both multi-line bodies (one pair per
/// line) and single-line bodies (`{ "xs": "10px", "sm": "12px", ... }`).
///
/// The blocks we target (spacing, radius, stroke, density, chrome,
/// typography.{size,weight,line,track}) have no nested braces, so a naive
/// comma split is safe. If schema evolves, generalize then.
fn parse_pairs<F>(body: &str, parse_value: F) -> BTreeMap<String, f64>
where
    F: Fn(&str) -> Option<f64>,
{
    let mut out = BTreeMap::new();
    for chunk in body.split(',') {
        let trimmed = chunk.trim();
        if !trimmed.starts_with('"') {
            continue;
        }
        let after_open = &trimmed[1..];
        let Some(close_key) = after_open.find('"') else {
            continue;
        };
        let key = after_open[..close_key].to_string();
        let after_key = &after_open[close_key + 1..];
        let Some(colon) = after_key.find(':') else {
            continue;
        };
        let raw_value = after_key[colon + 1..].trim();
        if let Some(value) = parse_value(raw_value) {
            out.insert(key, value);
        }
    }
    out
}

/// Numeric value: `4`, `4.5`, `-0.02`. Trim quotes if accidentally
/// wrapped (defensive — top-level scalar blocks emit raw numerics).
fn parse_scalar_value(raw: &str) -> Option<f64> {
    let s = raw.trim_matches('"').trim();
    s.parse().ok()
}

/// String value with `px` suffix: `"10px"` → `10.0`. Strips the quotes
/// and the suffix.
fn parse_px_string_value(raw: &str) -> Option<f64> {
    let s = raw.trim_matches('"').trim();
    let stripped = s.strip_suffix("px").unwrap_or(s);
    stripped.trim().parse().ok()
}

/// String value optionally suffixed `em`: `"-0.02em"` → `-0.02`,
/// `"0"` → `0.0`. Strips quotes and optional suffix.
fn parse_em_string_value(raw: &str) -> Option<f64> {
    let s = raw.trim_matches('"').trim();
    let stripped = s.strip_suffix("em").unwrap_or(s);
    stripped.trim().parse().ok()
}

/// Transform a JSON key like `"2xl"`, `"full"`, `"row-h"`,
/// `"icon-btn-size"` into a Rust const-name suffix like `XL2`, `FULL`,
/// `ROW_H`, `ICON_BTN_SIZE`. Rules:
///
/// - hyphens → underscores
/// - leading digits move to the end (Rust identifier rule)
/// - everything uppercased
fn key_to_const_suffix(key: &str) -> String {
    let underscored = key.replace('-', "_");
    // Split leading-digit prefix from the rest.
    let mut leading_digits = String::new();
    let mut rest = String::new();
    let mut still_leading = true;
    for c in underscored.chars() {
        if still_leading && c.is_ascii_digit() {
            leading_digits.push(c);
        } else {
            still_leading = false;
            rest.push(c);
        }
    }
    let normalized = if leading_digits.is_empty() {
        underscored
    } else {
        format!("{rest}{leading_digits}")
    };
    normalized.to_uppercase()
}

/// One color entry: `("bg-0", OklchRaw { l, c, h, a })`.
#[derive(Debug, Clone)]
struct ColorEntry {
    key: String,
    l: f64,
    c: f64,
    h: f64,
    a: f64, // 1.0 = opaque
}

/// Per-theme map of color tokens. Order preserved (BTreeMap → sorted alphabetically
/// for deterministic diff; consumer doesn't depend on order).
#[derive(Debug, Default, Clone)]
struct Theme {
    inherits: Option<String>,
    colors: BTreeMap<String, ColorEntry>,
}

/// Parse `themes.<name>` blocks from tokens.json. Returns map theme-name → Theme.
///
/// Line-based: walks character-by-character looking for `"forge": {` etc.
/// at level 2 nesting (inside `"themes": { ... }`). Inside each theme,
/// looks for `"$inherits": "..."` and `"color": { ... }` with color entries.
fn parse_themes(json: &str) -> BTreeMap<String, Theme> {
    let mut out = BTreeMap::new();

    // Locate the "themes" block start.
    let themes_start = json
        .find("\"themes\"")
        .expect("tokens.json missing top-level `themes` key");
    let after_themes = &json[themes_start..];
    let block_open = after_themes
        .find('{')
        .expect("`themes` value not an object");

    // Walk theme entries: at depth 1 inside the themes block, each
    // `"theme_name":` starts a theme.
    let body = &after_themes[block_open + 1..];

    let theme_keys = ["forge", "workshop", "sunstone", "blueprint"];
    for theme_name in theme_keys {
        if let Some(block) = extract_object_block(body, theme_name) {
            let theme = parse_theme(&block);
            out.insert(theme_name.to_string(), theme);
        }
    }

    out
}

/// Find `"name":` followed by `{ ... }` and return the inner contents
/// (without surrounding braces). Returns None if key not found.
fn extract_object_block(s: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\":", key);
    let start = s.find(&needle)?;
    let after_key = &s[start + needle.len()..];
    let open_brace = after_key.find('{')?;
    let body_start = open_brace + 1;
    // Find matching `}` by depth counting.
    let bytes = after_key.as_bytes();
    let mut depth = 1;
    let mut i = body_start;
    let mut in_string = false;
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if escape {
            escape = false;
        } else if b == b'\\' && in_string {
            escape = true;
        } else if b == b'"' {
            in_string = !in_string;
        } else if !in_string {
            if b == b'{' {
                depth += 1;
            } else if b == b'}' {
                depth -= 1;
                if depth == 0 {
                    return Some(after_key[body_start..i].to_string());
                }
            }
        }
        i += 1;
    }
    None
}

/// Parse a theme block body. Looks for `$inherits` and `color` keys.
fn parse_theme(body: &str) -> Theme {
    let mut theme = Theme::default();

    // $inherits is a string value.
    if let Some(inherits_start) = body.find("\"$inherits\":") {
        let after = &body[inherits_start + "\"$inherits\":".len()..];
        if let Some(open) = after.find('"') {
            let after_open = &after[open + 1..];
            if let Some(close) = after_open.find('"') {
                theme.inherits = Some(after_open[..close].to_string());
            }
        }
    }

    // color is an object.
    if let Some(color_block) = extract_object_block(body, "color") {
        for line in color_block.lines() {
            if let Some(entry) = parse_color_line(line) {
                theme.colors.insert(entry.key.clone(), entry);
            }
        }
    }

    theme
}

/// Parse one line like:
///   `        "bg-0":           "oklch(0.105 0.004 320)",`
///   `        "bg-elev":        "oklch(0.245 0.008 320)",`
///   `        "bg-scrim":       "oklch(0.055 0.004 320 / 0.60)",`
///   `        "grid-line":      "oklch(1 0 0 / 0.035)",`
///
/// Returns None for lines that don't match (whitespace, comments, etc).
fn parse_color_line(line: &str) -> Option<ColorEntry> {
    let trimmed = line.trim();
    if !trimmed.starts_with('"') {
        return None;
    }
    // Extract key between first pair of quotes.
    let after_open = &trimmed[1..];
    let close_key = after_open.find('"')?;
    let key = after_open[..close_key].to_string();

    // After `:`, find `"oklch(...)"`.
    let after_colon = &after_open[close_key + 1..];
    let oklch_start = after_colon.find("\"oklch(")?;
    let after_paren = &after_colon[oklch_start + "\"oklch(".len()..];
    let paren_close = after_paren.find(')')?;
    let oklch_args = &after_paren[..paren_close];

    // oklch_args looks like:
    //   "0.105 0.004 320"
    //   "0.055 0.004 320 / 0.60"
    //   "1 0 0 / 0.035"
    let (lch_part, alpha) = if let Some(slash_idx) = oklch_args.find('/') {
        let lch = oklch_args[..slash_idx].trim();
        let alpha_str = oklch_args[slash_idx + 1..].trim();
        let a: f64 = alpha_str.parse().ok()?;
        (lch, a)
    } else {
        (oklch_args.trim(), 1.0)
    };

    let mut parts = lch_part.split_whitespace();
    let l: f64 = parts.next()?.parse().ok()?;
    let c: f64 = parts.next()?.parse().ok()?;
    let h: f64 = parts.next()?.parse().ok()?;

    Some(ColorEntry {
        key,
        l,
        c,
        h,
        a: alpha,
    })
}

/// Emit Rust source for `tokens_generated.rs`.
#[allow(clippy::too_many_arguments)]
fn emit_rust(
    themes: &BTreeMap<String, Theme>,
    spacing: &BTreeMap<String, f64>,
    radius: &BTreeMap<String, f64>,
    stroke: &BTreeMap<String, f64>,
    density: &BTreeMap<String, f64>,
    chrome: &BTreeMap<String, f64>,
    typography_size: &BTreeMap<String, f64>,
    typography_weight: &BTreeMap<String, f64>,
    typography_line: &BTreeMap<String, f64>,
    typography_track: &BTreeMap<String, f64>,
) -> String {
    let mut s = String::new();
    s.push_str(
        "// AUTO-GENERATED by build.rs from `docs/design/tokens.json`.\n\
         // DO NOT EDIT BY HAND — changes here are overwritten every build.\n\
         // Source-of-truth: `docs/design/tokens.json` (Wave 2 PR 11.1 +\n\
         //                  Wave 4 stage A/B extends to spacing/radius/\n\
         //                  stroke/density/chrome/typography codegen).\n\
         \n\
         /// Raw OKLCH color from canonical design tokens. `a` field is\n\
         /// the alpha multiplier (1.0 = opaque). Consumer in `color.rs`\n\
         /// resolves to a concrete sRGB `Color` via\n\
         /// `Color::from_oklch` (alpha 1.0) or `from_oklch_alpha`.\n\
         pub struct OklchRaw {\n\
         \x20   pub l: f64,\n\
         \x20   pub c: f64,\n\
         \x20   pub h: f64,\n\
         \x20   pub a: f64,\n\
         }\n\
         \n",
    );

    for theme_name in ["forge", "workshop", "sunstone", "blueprint"] {
        let theme = themes
            .get(theme_name)
            .unwrap_or_else(|| panic!("theme {theme_name} missing in tokens.json"));

        // Resolve inheritance: if theme has $inherits, fill missing keys
        // from parent. Done in a single pass — `inherits` only goes one
        // level (forge is the root).
        let resolved = if let Some(parent_name) = &theme.inherits {
            let parent = themes
                .get(parent_name)
                .unwrap_or_else(|| panic!("inherited theme {parent_name} missing"));
            let mut merged = parent.colors.clone();
            for (k, v) in &theme.colors {
                merged.insert(k.clone(), v.clone());
            }
            merged
        } else {
            theme.colors.clone()
        };

        s.push_str(&format!(
            "/// `{}` theme colors. Generated from tokens.json.\n",
            theme_name
        ));
        s.push_str(&format!(
            "pub const COLORS_{}: &[(&str, OklchRaw)] = &[\n",
            theme_name.to_uppercase()
        ));
        for (key, entry) in &resolved {
            s.push_str(&format!(
                "    (\"{}\", OklchRaw {{ l: {:.6}, c: {:.6}, h: {:.6}, a: {:.6} }}),\n",
                key, entry.l, entry.c, entry.h, entry.a
            ));
        }
        s.push_str("];\n\n");
    }

    // Wave 4 stage A — scalar token sections.
    emit_scalar_consts(
        &mut s,
        "SPACING",
        spacing,
        "Spacing scale (px). Source: `tokens.json::spacing`.",
    );
    emit_scalar_consts(
        &mut s,
        "RADIUS",
        radius,
        "Border-radius scale (px). Source: `tokens.json::radius`.",
    );
    emit_scalar_consts(
        &mut s,
        "STROKE",
        stroke,
        "Stroke width scale (px). Source: `tokens.json::stroke`.",
    );
    emit_scalar_consts(
        &mut s,
        "DENSITY",
        density,
        "Density row-height scale (px). Source: `tokens.json::density`.",
    );
    emit_scalar_consts(
        &mut s,
        "CHROME",
        chrome,
        "Chrome dimensional constants (px). Source: `tokens.json::chrome`.",
    );

    // Wave 4 stage B — typography codegen.
    emit_scalar_consts(
        &mut s,
        "TYPOGRAPHY_SIZE",
        typography_size,
        "Type-scale font sizes (px). Source: `tokens.json::typography.size`.",
    );
    emit_typography_weight_consts(&mut s, typography_weight);
    emit_scalar_consts(
        &mut s,
        "TYPOGRAPHY_LINE",
        typography_line,
        "Line-height ratios. Source: `tokens.json::typography.line`.",
    );
    emit_scalar_consts(
        &mut s,
        "TYPOGRAPHY_TRACK",
        typography_track,
        "Letter-spacing em values. Source: `tokens.json::typography.track`.",
    );

    s
}

/// Emit a block of `pub const <PREFIX>_<KEY>: f32 = <VALUE>;` lines.
/// Keys are normalized via [`key_to_const_suffix`].
fn emit_scalar_consts(s: &mut String, prefix: &str, values: &BTreeMap<String, f64>, doc: &str) {
    s.push_str(&format!("/// {doc}\n"));
    for (key, value) in values {
        let suffix = key_to_const_suffix(key);
        s.push_str(&format!(
            "pub const {prefix}_{suffix}: f32 = {value}_f32;\n",
            value = format_f32_literal(*value),
        ));
    }
    s.push('\n');
}

/// `typography.weight` values are integers (CSS font-weight: 400, 500,
/// 600, 700) → emitted as `u16`, not `f32`.
fn emit_typography_weight_consts(s: &mut String, weights: &BTreeMap<String, f64>) {
    s.push_str("/// Font weights (CSS units). Source: `tokens.json::typography.weight`.\n");
    for (key, value) in weights {
        let suffix = key_to_const_suffix(key);
        let int_value = *value as u16;
        s.push_str(&format!(
            "pub const TYPOGRAPHY_WEIGHT_{suffix}: u16 = {int_value};\n"
        ));
    }
    s.push('\n');
}

/// Format an f64 → f32 literal with platform-independent precision.
/// Integers emit as `N.0`; fractions use up to 6 decimal places,
/// trailing zeros stripped. Determinism HR-5: same byte output across
/// linux/mac/windows.
fn format_f32_literal(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{:.1}", value)
    } else {
        let s = format!("{:.6}", value);
        // Strip trailing zeros after the decimal point, but keep at
        // least one digit (e.g., "1.500000" → "1.5", "1.000000" → "1.0").
        let trimmed = s.trim_end_matches('0');
        if trimmed.ends_with('.') {
            format!("{trimmed}0")
        } else {
            trimmed.to_string()
        }
    }
}
