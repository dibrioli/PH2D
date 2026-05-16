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

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_path = out_dir.join("tokens_generated.rs");
    fs::write(&out_path, emit_rust(&themes))
        .unwrap_or_else(|e| panic!("write {}: {e}", out_path.display()));
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
fn emit_rust(themes: &BTreeMap<String, Theme>) -> String {
    let mut s = String::new();
    s.push_str(
        "// AUTO-GENERATED by build.rs from `docs/design/tokens.json`.\n\
         // DO NOT EDIT BY HAND — changes here are overwritten every build.\n\
         // Source-of-truth: `docs/design/tokens.json` (Wave 2 PR 11.1).\n\
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

    s
}
