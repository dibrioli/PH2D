//! Wave 2 PR 11.6 — anti-regression lint enforcing HR-15 extended:
//! widgets and screens must reach all color via
//! [`ph2d_tokens::ColorToken`] / [`crate::paint::resolve`]; no hex
//! literals. The pre-PR audit found zero offenders (the migration to
//! token-resolution finished in Wave 1), so this test is purely
//! preventative: it catches future regressions that re-introduce
//! hand-typed colors and silently break the theme system.
//!
//! ## Scope
//!
//! - `crates/ph2d-editor/src/widget/**/*.rs`
//! - `crates/ph2d-editor/src/screens/**/*.rs`
//!
//! Other crates either don't paint chrome (`ph2d-tokens` owns the
//! palette; `ph2d-vector` is geometry-only) or have separate concerns
//! (`ph2d-render` does sim sprites, not chrome).
//!
//! ## What counts as a "color literal"
//!
//! Two complementary families are gated:
//!
//! 1. **Hex literals** — 6- or 8-hex-digit literal (`0x` prefix or
//!    `#` prefix). Three- or four-hex shorthands aren't lintered
//!    because they're never colors in Rust source. Underscores inside
//!    literals are tolerated (`0x4422_88FF`).
//! 2. **Non-hex aliases (Wave 4 stage C)** — direct construction or
//!    use of a Vello/peniko color outside the token system:
//!    - `Color::WHITE` / `Color::BLACK` / `Color::TRANSPARENT`
//!    - `Color::rgba8(` / `Color::rgb8(` /
//!      `Color::from_rgba8(` / `Color::from_rgba(` / `Color::from_rgb(`
//!    - All of the above prefixed with `VelloColor::` (the editor's
//!      common alias).
//!
//! Both families share the same escape hatches (comment + path
//! allowlist) below.
//!
//! ## Escape hatch
//!
//! A trailing comment `// LITERAL-COLOR-OK: <reason>` on the same
//! line suppresses the lint for that line. Use only when the value
//! genuinely IS a color but lives outside the theme system (e.g.,
//! Blender-style HSV math tables in
//! `widget/blender_color_picker/`, theme-invariant drop overlays,
//! token-resolved → VelloColor bridge calls). Every allowlist site
//! must carry a written reason.

use std::fs;
use std::path::{Path, PathBuf};

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

/// Identify a 6- or 8-hex-digit literal starting at `bytes[start]`.
/// Returns the matched length in bytes if it qualifies as a color
/// literal, `None` otherwise.
///
/// Accepts:
/// - `0xRRGGBB` / `0xRRGGBBAA` (optionally with `_` separators inside)
/// - `#RRGGBB` / `#RRGGBBAA`
fn hex_color_at(bytes: &[u8], start: usize) -> Option<usize> {
    // Match `0x` or `#` prefix.
    let prefix_len = if start + 2 <= bytes.len() && bytes[start] == b'0' && bytes[start + 1] == b'x'
    {
        2
    } else if bytes[start] == b'#' {
        1
    } else {
        return None;
    };
    // Make sure the byte before the prefix isn't an alphanumeric
    // (avoid matching the tail of an identifier like `foo0x...`).
    if start > 0 {
        let prev = bytes[start - 1];
        if prev.is_ascii_alphanumeric() || prev == b'_' {
            return None;
        }
    }
    let mut digits = 0usize;
    let mut i = start + prefix_len;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_hexdigit() {
            digits += 1;
        } else if b == b'_' {
            // Underscore separators don't count toward digit total.
        } else {
            break;
        }
        i += 1;
    }
    if digits == 6 || digits == 8 {
        // Reject if the character right after the literal is
        // alphanumeric (then it's a partial match inside a longer
        // identifier).
        if i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            return None;
        }
        Some(i - start)
    } else {
        None
    }
}

/// Detect a non-hex Vello color expression starting at `bytes[start]`.
/// Returns the matched length in bytes if recognized, else None.
///
/// Recognized prefixes (case-sensitive):
/// - `Color::WHITE`, `Color::BLACK`, `Color::TRANSPARENT`
/// - `Color::rgba8(`, `Color::rgb8(`
/// - `Color::from_rgba8(`, `Color::from_rgba(`, `Color::from_rgb(`
/// - All of the above with `VelloColor::` instead of `Color::`
///
/// Wave 4 stage C — closes a bypass where agents reached non-token
/// colors via `Color::from_rgba8(255, 0, 0, 255)` etc., which the
/// hex-only lint never caught.
fn vello_color_at(bytes: &[u8], start: usize) -> Option<usize> {
    // The byte before the match must not be alphanumeric / `_` —
    // otherwise this is the tail of an identifier (e.g. `MyColor::`,
    // `_Color::`) that happens to end in `Color::`.
    if start > 0 {
        let prev = bytes[start - 1];
        if prev.is_ascii_alphanumeric() || prev == b'_' {
            return None;
        }
    }

    // Match the type prefix. `VelloColor::` is tried first so
    // disambiguation is unambiguous (otherwise `Color::` would match
    // partially inside `VelloColor::`, but we already gated that via
    // the prev-char check above; explicit ordering is just clearer).
    let prefix_len = if bytes_starts_with(bytes, start, b"VelloColor::") {
        b"VelloColor::".len()
    } else if bytes_starts_with(bytes, start, b"Color::") {
        b"Color::".len()
    } else {
        return None;
    };

    let after = start + prefix_len;

    // Try named constants first.
    for kw in [
        b"WHITE".as_slice(),
        b"BLACK".as_slice(),
        b"TRANSPARENT".as_slice(),
    ] {
        if bytes_starts_with(bytes, after, kw) {
            let end = after + kw.len();
            // Reject if followed by alphanumeric — e.g. `WHITESPACE`,
            // `BLACKHOLE` — that would be a partial id match.
            if end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                continue;
            }
            return Some(end - start);
        }
    }

    // Try constructor function names (note: longer variants first so
    // `from_rgba8` wins over `from_rgba`).
    for ctor in [
        b"from_rgba8(".as_slice(),
        b"from_rgba(".as_slice(),
        b"from_rgb(".as_slice(),
        b"rgba8(".as_slice(),
        b"rgb8(".as_slice(),
    ] {
        if bytes_starts_with(bytes, after, ctor) {
            return Some(after + ctor.len() - start);
        }
    }

    None
}

/// Byte-slice equality with bounds check, returning false on overflow.
fn bytes_starts_with(bytes: &[u8], start: usize, needle: &[u8]) -> bool {
    start + needle.len() <= bytes.len() && &bytes[start..start + needle.len()] == needle
}

/// Strip the `// LITERAL-COLOR-OK: ...` allowlist comment from a line.
/// Returns `true` if found.
fn line_has_allowlist(line: &str) -> bool {
    line.contains("LITERAL-COLOR-OK")
}

/// Detect whether a line is fully inside a doc comment. Two flavors:
///
/// - `///` outer doc
/// - `//!` inner doc
///
/// Regular `//` line comments are also exempt, because writing
/// `0xRRGGBB` in a comment is documentation, not code. The boundaries
/// of multi-line `/* ... */` blocks are NOT walked — they are rare in
/// this codebase and any hit inside one is a strong signal we want
/// flagged anyway.
fn line_is_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//")
}

/// Walks the file once and returns the half-open byte ranges that
/// fall inside a `#[cfg(test)] mod tests { ... }` block. Test code
/// regularly uses 8-hex `entity_bits` fixtures (`0xDEAD_BEEF` etc.)
/// that are NOT colors; skipping the whole test module avoids
/// false-positives without weakening the lint over production code.
///
/// The walker is brace-counting and naive about strings/comments —
/// good enough for this codebase, where test braces inside string
/// literals don't occur.
fn cfg_test_byte_ranges(src: &str) -> Vec<(usize, usize)> {
    let bytes = src.as_bytes();
    let mut ranges = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        // Find next `#[cfg(test)]` marker.
        let Some(rel) = src[i..].find("#[cfg(test)]") else {
            break;
        };
        let attr_start = i + rel;
        // Find the following `{` that opens the module body.
        let mut j = attr_start + "#[cfg(test)]".len();
        while j < bytes.len() && bytes[j] != b'{' {
            j += 1;
        }
        if j >= bytes.len() {
            break;
        }
        // Track brace depth from j (which is `{`).
        let body_start = j;
        let mut depth = 0i32;
        let mut k = j;
        while k < bytes.len() {
            match bytes[k] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        // Include the closing brace.
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

/// True if `byte_offset` falls inside any `cfg(test)` body range.
fn in_test_module(ranges: &[(usize, usize)], byte_offset: usize) -> bool {
    ranges
        .iter()
        .any(|(lo, hi)| byte_offset >= *lo && byte_offset < *hi)
}

/// Module path under widget/blender_color_picker/ is allowed to carry
/// HSV math tables with raw hex. Listed explicitly so a typo in a
/// neighbouring widget doesn't silently inherit the allow.
///
/// Uses [`Path::components`] instead of string-matching so it's robust
/// to platform separator differences — on Windows CI we observed
/// mixed-separator paths like `src/widget\blender_color_picker\foo.rs`
/// from `crate_root.join("src/widget")` which neither a pure-forward
/// nor pure-back string match catches.
fn path_is_allowlisted(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str() == "blender_color_picker")
}

#[test]
fn no_hex_color_literals_in_widget_or_screens() {
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
            // Build a byte-offset table: line index → start-of-line
            // byte offset, so we can map hits back into the file as a
            // whole when checking against `test_ranges`.
            let mut line_offset = 0usize;
            for (line_no, line) in text.lines().enumerate() {
                let line_start = line_offset;
                line_offset += line.len() + 1; // +1 for '\n' (close enough on CRLF too)
                if line_is_comment(line) || line_has_allowlist(line) {
                    continue;
                }
                let bytes = line.as_bytes();
                let mut i = 0;
                while i < bytes.len() {
                    // Two matchers in sequence — hex literal first
                    // (cheap byte check), then Vello/Color alias.
                    let matched = hex_color_at(bytes, i).or_else(|| vello_color_at(bytes, i));
                    if let Some(len) = matched {
                        let abs = line_start + i;
                        if !in_test_module(&test_ranges, abs) {
                            let literal = &line[i..i + len];
                            let rel = path.strip_prefix(&crate_root).unwrap_or(&path);
                            hits.push(format!(
                                "  {}:{}: `{literal}` — use `ColorToken::X.resolve(theme)` \
                                 instead, or add `// LITERAL-COLOR-OK: <reason>` if the value \
                                 genuinely lives outside the theme system (e.g. HSV math table, \
                                 token-cast-to-vello bridge, theme-invariant overlay).",
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

    if !hits.is_empty() {
        panic!(
            "HR-15 violation — hex color literal(s) in widget/screens. \
             Theme-driven color must flow through \
             `ColorToken::resolve(theme)` / `crate::paint::resolve`:\n{}",
            hits.join("\n"),
        );
    }
}

/// Smoke: a fabricated literal IS detected by the matcher. If this
/// test ever fails, the lint above is silently letting violations
/// through and needs investigation.
#[test]
fn matcher_smoke_detects_hex_literals() {
    let cases = [
        ("0xFF00FF", true),
        ("0xFF00FF00", true),
        ("0x33aa_bb88", true),
        ("#aabbcc", true),
        ("#FFAA88FF", true),
        ("0xFFFF", false),       // 4 digits — not a color
        ("0xFFF", false),        // 3 digits — not a color
        ("0xFFAABBCCDD", false), // 10 digits — too many
        ("foo0xAABBCC", false),  // embedded in identifier
        ("0xAABBCC bar", true),  // standalone, trailing text OK
    ];
    for (input, want) in cases {
        let bytes = input.as_bytes();
        let mut found = false;
        let mut i = 0;
        while i < bytes.len() {
            if let Some(len) = hex_color_at(bytes, i) {
                found = true;
                i += len;
            } else {
                i += 1;
            }
        }
        assert_eq!(found, want, "matcher disagrees on input {input:?}");
    }
}

/// Wave 4 stage C smoke — the extended matcher catches non-hex
/// Vello/Color paths that the hex matcher silently lets through.
#[test]
fn matcher_smoke_detects_vello_color_aliases() {
    let cases = [
        ("Color::WHITE", true),
        ("Color::BLACK", true),
        ("Color::TRANSPARENT", true),
        ("VelloColor::WHITE", true),
        ("VelloColor::BLACK", true),
        ("Color::rgba8(255, 0, 0, 255)", true),
        ("Color::rgb8(0, 0, 0)", true),
        ("Color::from_rgba8(255, 255, 255, 255)", true),
        ("Color::from_rgba(0.5, 0.5, 0.5, 1.0)", true),
        ("Color::from_rgb(0.0, 0.0, 0.0)", true),
        ("VelloColor::from_rgba8(0, 0, 0, 0)", true),
        // ── Negatives ──
        ("MyColor::WHITE", false),         // not Color::/VelloColor::
        ("colorize::WHITE", false),        // identifier ends in `colorize::`, prev char `:`
        ("Color::WHITESPACE", false),      // alpha-suffix → not a constant
        ("Color::BLACKBERRY", false),      // alpha-suffix
        ("Color::from_oklch(...)", false), // legitimate token-aware constructor
        ("Color::resolve(...)", false),    // ColorToken bridge
        ("foo_Color::WHITE", false),       // `_Color::` is an identifier
        ("Color::SHADE", false),           // not a recognized constant
    ];
    for (input, want) in cases {
        let bytes = input.as_bytes();
        let mut found = false;
        let mut i = 0;
        while i < bytes.len() {
            if let Some(len) = vello_color_at(bytes, i) {
                found = true;
                i += len;
            } else {
                i += 1;
            }
        }
        assert_eq!(found, want, "matcher disagrees on input {input:?}");
    }
}
