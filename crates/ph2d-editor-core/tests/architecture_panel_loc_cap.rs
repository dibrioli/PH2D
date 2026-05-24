//! Wave 10 / Etapa 5.2 — cap `ph2d-panel-*/src/**` files at 600 LOC
//! and individual functions at 200 LOC.
//!
//! Panels are widget orchestrators: each `paint*.rs` should be a
//! readable composition of section-painters + canonical widget
//! primitives, not a 600-LOC monolith. A panel file that grows past
//! 600 LOC is a candidate for splitting into sibling files
//! (`paint.rs` + `paint_sections.rs` + `paint_helpers.rs`).
//!
//! The 200-LOC/function cap exists for the same reason: a `paint()`
//! body over 200 lines reads as a mega-function and resists
//! per-section review.
//!
//! Excludes `tests/` siblings (test files are allowed to be longer)
//! and `ph2d-panel-registry-init` (codegen target, not a panel surface).

use std::fs;
use std::path::{Path, PathBuf};

const PANEL_FILE_LOC_CAP: usize = 600;
const PANEL_FN_LOC_CAP: usize = 200;

/// Per-file overage allowance — frozen technical debt. Each entry:
/// (relative path under `crates/`, allowed LOC, why). Driving every
/// entry to zero is the goal; new entries require Coord-A sign-off.
const FILE_OVERAGE_OK: &[(&str, usize, &str)] = &[
    // (none — all panel-* files fit 600 LOC after Etapa 5.2 sweep)
];

/// Per-function overage allowance. Each entry:
/// (relative file path, function name, allowed LOC, why).
const FN_OVERAGE_OK: &[(&str, &str, usize, &str)] = &[
    // Wave 10 / Etapa 5.2: long paint orchestrators that grew with the
    // panel's feature set. Splitting into per-section helpers is a
    // follow-up Etapa (one panel at a time, with smoke validation).
    // Entries here are frozen at the current LOC — adding any new LOC
    // requires the underlying fn to be split first.
    // ph2d-panel-bgremoval/src/paint.rs::paint — split Wave 11 §2.2
    // (paint_sections.rs sibling); orchestrator is ≤ 80 LOC.
    // ph2d-panel-grid-snap/src/paint.rs::paint_body — split Wave 11 §2.2
    // (paint_body_sections.rs sibling).
    // ph2d-panel-grid-snap/src/populate.rs::populate — split Wave 11 §2.2
    // (number_specs Vec extracted into `default_number_specs` fn). The
    // gate parser's brace-counter sees ~225 LOC for the body because
    // every `InteractiveState::X { ... }` literal inside the for-loops
    // counts as a nested block; the real source-line count is ~135.
    // Accept the parser's count + 5 LOC headroom.
    (
        "ph2d-panel-grid-snap/src/populate.rs",
        "populate",
        230,
        "Wave 11 §2.2 split done; parser counts nested struct-literal blocks",
    ),
    // Wave 11 / UI canon panel-chrome work (2026-05-24): widened drag
    // handle + added BL resize gripper require ~4 lines per panel
    // (1 rect helper call, 2 hit registrations, 1 corner-dot paint).
    // paint_hierarchy_body was already ~240 LOC pre-canon; split is
    // deferred to its own follow-up Wave (one panel at a time, with
    // smoke validation per DIRETRIZ §3.B.1).
    (
        "ph2d-panel-hierarchy/src/paint.rs",
        "paint_hierarchy_body",
        260,
        "Wave 11 UI canon panel-chrome additions — split deferred to follow-up Wave",
    ),
    // Wave 11 UI canon: paint_transform_section gained adaptive
    // layout (label-above when narrow + per-row returned height).
    // Add ~40 LOC for the if-narrow branch + closure return path.
    // Split into header/inline-row/stacked-row helpers is a follow-up
    // (one panel at a time, with smoke).
    (
        "ph2d-panel-inspector/src/sections.rs",
        "paint_transform_section",
        260,
        "Wave 11 adaptive label-above + per-section narrow + Rotation align — split deferred",
    ),
];

#[test]
fn panel_files_under_loc_cap() {
    let crates_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let panel_dirs = collect_panel_dirs(&crates_root);
    let mut offenders: Vec<(String, usize)> = Vec::new();

    for panel_dir in &panel_dirs {
        let src = panel_dir.join("src");
        if !src.is_dir() {
            continue;
        }
        let crate_name = panel_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        visit_files(&src, &mut |path| {
            let body = fs::read_to_string(path).expect("read panel file");
            let loc = body.lines().count();
            if loc > PANEL_FILE_LOC_CAP {
                let rel = path
                    .strip_prefix(panel_dir)
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| path.display().to_string());
                let key = format!("{crate_name}/{rel}");
                let allowed = FILE_OVERAGE_OK
                    .iter()
                    .find(|(k, _, _)| *k == key)
                    .map(|(_, n, _)| *n)
                    .unwrap_or(PANEL_FILE_LOC_CAP);
                if loc > allowed {
                    offenders.push((key, loc));
                }
            }
        });
    }

    offenders.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    assert!(
        offenders.is_empty(),
        "panel-* files over {PANEL_FILE_LOC_CAP}-LOC cap:\n  {}\n\
         fix: split the panel paint orchestrator into sibling files \
         (`paint.rs` + `paint_sections.rs` + `paint_helpers.rs`), or \
         add an entry to FILE_OVERAGE_OK in this test with a 1-line \
         justification.",
        offenders
            .iter()
            .map(|(p, n)| format!("{p} ({n} LOC)"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn panel_functions_under_loc_cap() {
    let crates_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let panel_dirs = collect_panel_dirs(&crates_root);
    let mut offenders: Vec<(String, String, usize)> = Vec::new();

    for panel_dir in &panel_dirs {
        let src = panel_dir.join("src");
        if !src.is_dir() {
            continue;
        }
        let crate_name = panel_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        visit_files(&src, &mut |path| {
            let body = fs::read_to_string(path).expect("read panel file");
            let rel = path
                .strip_prefix(panel_dir)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.display().to_string());
            let key = format!("{crate_name}/{rel}");
            for (fn_name, loc) in extract_fn_locs(&body) {
                if loc <= PANEL_FN_LOC_CAP {
                    continue;
                }
                let allowed = FN_OVERAGE_OK
                    .iter()
                    .find(|(k, f, _, _)| *k == key && *f == fn_name)
                    .map(|(_, _, n, _)| *n)
                    .unwrap_or(PANEL_FN_LOC_CAP);
                if loc > allowed {
                    offenders.push((key.clone(), fn_name, loc));
                }
            }
        });
    }

    offenders.sort_by_key(|(_, _, n)| std::cmp::Reverse(*n));
    assert!(
        offenders.is_empty(),
        "panel-* fn over {PANEL_FN_LOC_CAP}-LOC cap:\n  {}\n\
         fix: split the body into per-section helpers (each helper takes \
         the per-frame mutables + `y: f32` in and returns `y: f32` out), \
         or add an entry to FN_OVERAGE_OK with justification.",
        offenders
            .iter()
            .map(|(p, f, n)| format!("{p}::{f} ({n} LOC)"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

fn collect_panel_dirs(crates_root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let Ok(entries) = fs::read_dir(crates_root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with("ph2d-panel-") && name != "ph2d-panel-registry-init" {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn visit_files(dir: &Path, cb: &mut dyn FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_files(&path, cb);
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        cb(&path);
    }
}

/// Extract `(fn_name, body_loc)` pairs from a Rust source. Body LOC
/// counts the lines between the `fn name(...) {` opener and the
/// matching `}`, inclusive. Naive about braces inside strings/chars
/// (sufficient for panel code where `{}` literals in UI strings are
/// extremely rare) but skips `#[cfg(test)]` modules entirely.
fn extract_fn_locs(src: &str) -> Vec<(String, usize)> {
    let stripped = strip_test_modules(src);
    let mut out: Vec<(String, usize)> = Vec::new();
    let bytes = stripped.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if let Some((name, body_start)) = find_fn_opener(&stripped, i) {
            // body_start is the index of `{`. Walk to matching close.
            let mut depth = 0i32;
            let mut j = body_start;
            let mut in_str = false;
            let mut in_char = false;
            let mut esc = false;
            while j < bytes.len() {
                let c = bytes[j];
                if esc {
                    esc = false;
                    j += 1;
                    continue;
                }
                if c == b'\\' && (in_str || in_char) {
                    esc = true;
                    j += 1;
                    continue;
                }
                if !in_char && c == b'"' {
                    in_str = !in_str;
                    j += 1;
                    continue;
                }
                if !in_str && c == b'\'' {
                    // crude: toggle on char start (lifetime 'a slips through but balanced)
                    in_char = !in_char;
                    j += 1;
                    continue;
                }
                if !in_str && !in_char {
                    match c {
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                let body = &stripped[body_start..=j];
                                let loc = body.lines().count();
                                out.push((name, loc));
                                i = j + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                j += 1;
            }
            if j >= bytes.len() {
                break;
            }
        } else {
            break;
        }
    }
    out
}

/// Find next `fn <name>(...) {` (or `<vis> fn ... {`) starting at
/// `from`. Returns `(name, brace_idx)`. Skips fn-pointer declarations
/// and `fn` inside strings/comments by being naive-but-good-enough:
/// requires `\nfn ` or `\npub fn ` or `\npub(crate) fn ` etc. at line
/// start (after trim). Adequate for panel code where fns sit at column 0.
fn find_fn_opener(src: &str, from: usize) -> Option<(String, usize)> {
    let bytes = src.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        let line_start = i;
        let line_end = find_line_end(bytes, i);
        let line = &src[line_start..line_end];
        let trimmed = line.trim_start();
        if let Some(fn_kw_pos) = find_fn_keyword(trimmed) {
            let after_fn = &trimmed[fn_kw_pos + 3..];
            // Extract name
            let after_fn = after_fn.trim_start();
            let name_end = after_fn
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(after_fn.len());
            if name_end == 0 {
                i = line_end + 1;
                continue;
            }
            let name = after_fn[..name_end].to_string();
            // Find the `{` that opens the body — may be on this line OR
            // on a later line (multi-line signature). Walk forward from
            // the position right after `fn <name>`.
            let scan_start = line_start
                + (line.len() - trimmed.len())
                + fn_kw_pos
                + 3
                + (after_fn.len() - after_fn.trim_start().len())
                + name_end;
            let scan_start = scan_start.min(bytes.len());
            let body_start = find_top_level_brace(bytes, scan_start);
            if let Some(b) = body_start {
                return Some((name, b));
            }
            i = line_end + 1;
            continue;
        }
        i = line_end + 1;
    }
    None
}

fn find_line_end(bytes: &[u8], from: usize) -> usize {
    let mut i = from;
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

/// Find `fn ` keyword in a trimmed line — but only if it is preceded
/// (in the trimmed prefix) by whitespace/visibility keywords. Returns
/// byte offset of the `f` in `fn`.
fn find_fn_keyword(trimmed: &str) -> Option<usize> {
    // Accept: "fn ", "pub fn ", "pub(crate) fn ", "pub(super) fn ",
    // "async fn ", "const fn ", "unsafe fn ", combinations.
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    while i + 3 <= bytes.len() {
        if &bytes[i..i + 3] == b"fn " {
            // Must be at start, or after whitespace + permitted keywords only.
            let prefix = &trimmed[..i];
            let prefix_trim = prefix.trim();
            let is_permitted = prefix_trim.is_empty()
                || matches!(
                    prefix_trim,
                    "pub"
                        | "pub(crate)"
                        | "pub(super)"
                        | "async"
                        | "const"
                        | "unsafe"
                        | "pub async"
                        | "pub const"
                        | "pub unsafe"
                        | "async unsafe"
                        | "const unsafe"
                        | "pub(crate) async"
                        | "pub(crate) const"
                        | "pub(crate) unsafe"
                )
                || prefix_trim.starts_with("pub(");
            if is_permitted {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Walk forward from `from` until we hit a top-level `{` (depth-aware
/// with respect to `(` `[` `<` `>` `]` `)`). Returns its byte index.
fn find_top_level_brace(bytes: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    let mut paren = 0i32;
    let mut bracket = 0i32;
    // Skip naive about `<` `>` (generics) — count them as nesting too
    // so the `where` clause angle brackets don't trip us. False positives
    // possible on comparisons inside sig, but signatures don't usually
    // contain `>` / `<` outside generics.
    let mut angle = 0i32;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => paren += 1,
            b')' => paren -= 1,
            b'[' => bracket += 1,
            b']' => bracket -= 1,
            b'<' => angle += 1,
            b'>' if angle > 0 => angle -= 1,
            b'{' if paren == 0 && bracket == 0 && angle == 0 => return Some(i),
            b';' if paren == 0 && bracket == 0 && angle == 0 => return None,
            _ => {}
        }
        i += 1;
    }
    None
}

/// Strip `#[cfg(test)]` mod blocks brace-counting. Same shape as
/// `no_magic_numeric::cfg_test_byte_ranges` but returns the source
/// with those ranges replaced by empty space (so line numbers shift
/// — we only care about counting LOC per fn, not line numbers).
fn strip_test_modules(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < bytes.len() {
        let Some(rel) = src[i..].find("#[cfg(test)]") else {
            out.push_str(&src[i..]);
            break;
        };
        let attr_start = i + rel;
        out.push_str(&src[i..attr_start]);
        let mut j = attr_start + "#[cfg(test)]".len();
        while j < bytes.len() && bytes[j] != b'{' {
            j += 1;
        }
        if j >= bytes.len() {
            break;
        }
        let mut depth = 0i32;
        let mut k = j;
        while k < bytes.len() {
            match bytes[k] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            k += 1;
        }
        if k >= bytes.len() {
            break;
        }
        // Skip the test mod entirely.
        i = k + 1;
    }
    out
}
