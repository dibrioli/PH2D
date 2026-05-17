//! Wave 4 stage D — bans bare `f32` / `f64` literals in
//! `crates/ph2d-editor/src/widget/**/*.rs` and
//! `crates/ph2d-editor/src/screens/**/*.rs` outside a small set of
//! **structural** values that arise from coordinate math, not design.
//!
//! ## Why this exists
//!
//! Wave 4 stage A made `Spacing` / `Radius` / `StrokeToken` /
//! `Density` / `TypeToken` / `Duration` codegen-driven from
//! `docs/design/tokens.json`. The danger is that a widget paint
//! function still pulls a value out of thin air (`Stroke::new(1.5)`,
//! `12.0` padding, `400.0` ms animation) — which silently breaks the
//! "designer edits tokens.json → Rust replicates" pipeline. This
//! lint forces every dimensional / temporal value through one of
//! those tokens.
//!
//! ## What is allowed
//!
//! - **Structural numerics**: `0.0`, `-0.0`, `0.5`, `-0.5`, `1.0`,
//!   `-1.0`, `2.0`, `-2.0`. These arise from centering math
//!   (`rect.h * 0.5`), ratios, sign flips. They are not designed.
//! - **Token-derived values**: anything that comes back from
//!   `Spacing::X.px()`, `Radius::X.px()`, `StrokeToken::X.px()`,
//!   `Density::X.row_h_px()`, `TypeToken::X.px()`,
//!   `Duration::X.ms()`, the `ROW_H_PX` / `ICON_BTN_SIZE_PX` /
//!   `SECTION_GAP_PX` consts, or `LineHeight::X.ratio()` /
//!   `LetterSpacing::X.em()`. Those return tokens, not literals.
//! - **Float-typed constants** declared via the token system —
//!   `f32::INFINITY`, `f32::NEG_INFINITY`, `f32::MIN`, `f32::MAX`.
//!   These are not literals.
//! - **`std::f32::consts::*`** (PI, FRAC_PI_2, etc.) — accessed by
//!   path, not literal.
//! - **`blender_color_picker/`** path allowlist — HSV math tables
//!   need raw numerics (60.0, 360.0). Same allow list as
//!   `no_literal_color`.
//! - **Per-line allowlist** — a trailing `// LITERAL-PX-OK: <reason>`
//!   comment on the same line. Use this for legitimate non-token
//!   numerics:
//!     - sRGB byte normalize (`/ 255.0`, `* 255.0`),
//!     - HSV math constants outside the picker (60.0, 360.0 hue),
//!     - mathematical ratios (3.0 for thirds, 4.0 for quarters when
//!       genuinely structural), animation cubic-bezier parameters
//!       that the `Easing` enum already encodes elsewhere.
//!
//! Every allowlist site **must** carry a written reason.
//!
//! ## What is banned
//!
//! Everything else. Migrate to the appropriate token:
//!
//! | Literal context | Token |
//! |-----------------|-------|
//! | padding / gap   | `Spacing::X.px()` |
//! | border-radius   | `Radius::X.px()` |
//! | stroke width    | `StrokeToken::X.px()` (cast to `f64` for kurbo) |
//! | font size       | `TypeToken::X.px()` |
//! | row height      | `Density::X.row_h_px()` or `ROW_H_PX` |
//! | section gap     | `SECTION_GAP_PX` |
//! | icon button     | `ICON_BTN_SIZE_PX` |
//! | animation ms    | `Duration::X.ms()` |
//!
//! ## Mode
//!
//! `WAVE_4_STAGE_D_MODE` constant below toggles between **warn**
//! (test prints inventory, never fails) and **deny** (test fails on
//! any non-allowed bare numeric). Wave 4 ships in `deny` after the
//! migration sweep completes.

use std::fs;
use std::path::{Path, PathBuf};

/// Toggle the lint behavior.
///
/// - `LintMode::Warn` → inventory printed to stdout; test always passes.
///   Used **during the Wave 4 stage D migration sweep** so CI stays
///   green while a multi-session migration of ~493 sites lands.
/// - `LintMode::Deny` → test fails with the inventory if any hit
///   remains. **Flipped to this at the end of Wave 4 stage D**,
///   when every site is migrated or annotated.
const WAVE_4_STAGE_D_MODE: LintMode = LintMode::Warn;

#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Both variants are referenced as the const value alternates between warn (sweep in progress) and deny (sweep complete).
enum LintMode {
    Warn,
    Deny,
}

/// Walk a directory tree and collect every `.rs` file path.
fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Detect a bare float literal starting at `bytes[start]`. Returns the
/// matched length in bytes if recognized, else None.
///
/// Match form: `\d+\.\d+` with optional leading sign, optional `_`
/// separators in either integer or fractional part, and optional
/// `f32` / `f64` suffix.
///
/// Reject if the byte before the match is alphanumeric / `_` /
/// `.` — that means the literal is the tail of an identifier or
/// a method call (e.g. `Spacing::Md.px()`, `entity_2.foo`).
fn float_literal_at(bytes: &[u8], start: usize) -> Option<(usize, String)> {
    // Reject if preceded by something that means this is not a fresh
    // literal but the tail of a larger token.
    if start > 0 {
        let prev = bytes[start - 1];
        if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'.' {
            return None;
        }
    }
    let mut i = start;
    // Optional sign — only when previous char is a delimiter, so we
    // don't mismatch `a - 1.0` as a negative literal.
    if bytes.get(i) == Some(&b'-') {
        if start > 0 {
            let prev = bytes[start - 1];
            if prev.is_ascii_alphanumeric() || prev == b')' || prev == b']' || prev == b'_' {
                // `a - 1.0` — subtraction, not a literal.
                return None;
            }
        }
        i += 1;
    }
    // Need at least one digit before the dot.
    let int_start = i;
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
        i += 1;
    }
    if i == int_start {
        return None;
    }
    // Need a dot, followed by ≥ 1 digit.
    if bytes.get(i) != Some(&b'.') {
        return None;
    }
    i += 1;
    let frac_start = i;
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
        i += 1;
    }
    if i == frac_start {
        // `1.something` where something is not a digit — could be
        // method call `1.0.method()` (still a literal), but bare
        // `1.foo` is not. Conservative: skip.
        return None;
    }
    // Optional `f32` / `f64` suffix.
    if i + 3 <= bytes.len() && (&bytes[i..i + 3] == b"f32" || &bytes[i..i + 3] == b"f64") {
        i += 3;
    }
    // Reject if followed by alphanumeric / `_` — partial identifier
    // match (e.g. `1.0foo` is not a literal we want to gate).
    if i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        return None;
    }
    let lit = std::str::from_utf8(&bytes[start..i]).ok()?.to_string();
    Some((i - start, lit))
}

/// True when the literal is structurally common math, not a designed
/// dimension. These pass without an allowlist comment.
///
/// We normalize the literal first (strip the optional underscore
/// before the `f32`/`f64` suffix) so `2.0f32`, `2.0_f32`, `2.0f64`,
/// `2.0_f64`, and plain `2.0` are treated identically.
fn is_structural(lit: &str) -> bool {
    let normalized = lit.replace("_f32", "f32").replace("_f64", "f64");
    matches!(
        normalized.as_str(),
        "0.0"
            | "-0.0"
            | "0.5"
            | "-0.5"
            | "1.0"
            | "-1.0"
            | "2.0"
            | "-2.0"
            | "0.0f32"
            | "0.0f64"
            | "0.5f32"
            | "0.5f64"
            | "1.0f32"
            | "1.0f64"
            | "2.0f32"
            | "2.0f64"
            | "-0.5f32"
            | "-0.5f64"
            | "-1.0f32"
            | "-1.0f64"
            | "-2.0f32"
            | "-2.0f64"
    )
}

/// Detect whether a line is fully a comment line (starts with `//`,
/// `///`, or `//!` after trimming).
fn line_is_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//")
}

/// Per-line allowlist: trailing `// LITERAL-PX-OK: <reason>`.
fn line_has_allowlist(line: &str) -> bool {
    line.contains("LITERAL-PX-OK")
}

/// Path allowlist. `blender_color_picker/` carries HSV/hue-math
/// tables that legitimately use raw numerics (60.0 / 360.0 hue,
/// 0.6 / 0.7 lightness, etc.); same allow as `no_literal_color`.
fn path_is_allowlisted(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str() == "blender_color_picker")
}

/// Half-open byte ranges that fall inside a `#[cfg(test)] mod tests { … }`
/// block. Same walker shape as `no_literal_color::cfg_test_byte_ranges`.
fn cfg_test_byte_ranges(src: &str) -> Vec<(usize, usize)> {
    let bytes = src.as_bytes();
    let mut ranges = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let Some(rel) = src[i..].find("#[cfg(test)]") else {
            break;
        };
        let attr_start = i + rel;
        let mut j = attr_start + "#[cfg(test)]".len();
        while j < bytes.len() && bytes[j] != b'{' {
            j += 1;
        }
        if j >= bytes.len() {
            break;
        }
        let body_start = j;
        let mut depth = 0i32;
        let mut k = j;
        while k < bytes.len() {
            match bytes[k] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        ranges.push((body_start, k + 1));
                        break;
                    }
                }
                _ => {}
            }
            k += 1;
        }
        i = k.saturating_add(1);
    }
    ranges
}

fn in_test_module(ranges: &[(usize, usize)], byte_offset: usize) -> bool {
    ranges
        .iter()
        .any(|(lo, hi)| byte_offset >= *lo && byte_offset < *hi)
}

#[test]
fn no_magic_numeric_in_widget_or_screens() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scan_roots = [
        crate_root.join("src/widget"),
        crate_root.join("src/screens"),
    ];

    let mut hits: Vec<String> = Vec::new();
    for root in &scan_roots {
        for path in collect_rs_files(root) {
            if path_is_allowlisted(&path) {
                continue;
            }
            let text = match fs::read_to_string(&path) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let test_ranges = cfg_test_byte_ranges(&text);
            let mut line_offset = 0usize;
            for (line_no, line) in text.lines().enumerate() {
                let line_start = line_offset;
                line_offset += line.len() + 1;
                if line_is_comment(line) || line_has_allowlist(line) {
                    continue;
                }
                let bytes = line.as_bytes();
                let mut i = 0;
                while i < bytes.len() {
                    if let Some((len, lit)) = float_literal_at(bytes, i) {
                        let abs = line_start + i;
                        if !in_test_module(&test_ranges, abs) && !is_structural(&lit) {
                            let rel = path.strip_prefix(&crate_root).unwrap_or(&path);
                            hits.push(format!(
                                "  {}:{}: `{lit}` — use Spacing/Radius/StrokeToken/TypeToken/\
                                 Density/Duration token, or `// LITERAL-PX-OK: <reason>` on \
                                 this line if the value is genuinely non-design (sRGB byte \
                                 normalize, math constant, etc.).",
                                rel.display(),
                                line_no + 1,
                            ));
                        }
                        i += len;
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }

    if hits.is_empty() {
        return;
    }

    let message = format!(
        "Wave 4 stage D — magic-numeric violation(s) in widget/screens \
         ({} site(s) need migration to a canonical token or allowlist):\n{}",
        hits.len(),
        hits.join("\n"),
    );

    match WAVE_4_STAGE_D_MODE {
        LintMode::Warn => {
            eprintln!("[warn] {message}");
        }
        LintMode::Deny => {
            panic!("{message}");
        }
    }
}

/// Smoke: the matcher accepts/rejects the cases we care about.
#[test]
fn matcher_smoke_detects_float_literals() {
    let cases = [
        ("0.0", false),    // structural
        ("0.5", false),    // structural
        ("1.0", false),    // structural
        ("2.0", false),    // structural
        ("-1.0", false),   // structural (when preceded by delimiter)
        ("4.0", true),     // designed
        ("12.5", true),    // designed
        ("16.0f32", true), // designed with suffix
        ("999.0", true),   // designed
        ("a.b", false),    // not a literal
        ("foo1.0", false), // tail of an identifier — alphabetic prev
        ("x.0", false),    // tail of `.0` field access
        ("1.0bar", false), // followed by alpha — partial id
        ("1_000.0", true), // underscore separator OK (designed because not structural)
    ];
    for (input, should_be_flagged) in cases {
        // Wrap the input with a leading space so the matcher's "prev
        // char" check sees a delimiter, not the start-of-line.
        let wrapped = format!(" {input} ");
        let bytes = wrapped.as_bytes();
        // Start at index 1 (after the space).
        let result = float_literal_at(bytes, 1);
        let flagged = match &result {
            Some((_, lit)) => !is_structural(lit),
            None => false,
        };
        assert_eq!(
            flagged, should_be_flagged,
            "matcher disagrees on input {input:?} (got {result:?})"
        );
    }
}
