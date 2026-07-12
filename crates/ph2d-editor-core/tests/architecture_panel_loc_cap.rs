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
    // Enio 2026-05-26: paint_sections.rs cresceu com Dither Strength +
    // Dither Grain sliders (2 rows novas dentro de
    // paint_posterize_quantize_section). Split em paint_helpers.rs é
    // follow-up; mantém o cap visível enquanto isso.
    (
        "ph2d-panel-color-equalization/src/paint_sections.rs",
        660,
        "Enio 2026-05-26 dither strength+grain rows — split deferred",
    ),
    // Coord 2026-06-04 ship-prep: Painter W4 adjustment panels (Curves/Levels/
    // B&W/Selective Color/Gradient Map) grew this orchestrator. Per-adjustment
    // sibling split is a Painter-impl follow-up; frozen at the ship-canonical 829.
    (
        "ph2d-panel-painter-layers/src/paint_adjust.rs",
        829,
        "Painter W4 bespoke adjustment panels — per-adjustment split deferred (Painter impl follow-up)",
    ),
    // Deform Wave 1: the monolithic Click-dispatch match gained ONE predicate call
    // (`is_deform_click`) to forward the Deform panel's clicks. The file was already at the 600 cap;
    // splitting the giant dispatch match is a separate refactor. Frozen at 601.
    (
        "ph2d-panel-painter-layers/src/event.rs",
        601,
        "Deform Wave 1 added one is_deform_click() call to the at-cap dispatch match; match split deferred",
    ),
];

/// Per-function overage allowance. Each entry:
/// (relative file path, function name, allowed LOC, why).
const FN_OVERAGE_OK: &[(&str, &str, usize, &str)] = &[
    // ── RE-BASELINE 2026-07-10 (the "deliberate foundational pass" this gate
    // asked for). Until today the brace walker toggled a char-literal flag on
    // every `'`, so a prose apostrophe in a `//` comment ("doesn't") or a
    // lifetime tick (`&'a`) closed a function early and UNDER-counted it. Every
    // number below is now a real measurement:
    //   · 3 entries were deleted — their fns are, and were, under the cap
    //     (grid-snap populate = 126, inspector color_tint = 124,
    //      painter-layers paint_adjustment_params = 54).
    //   · 2 entries were lying LOW and are corrected UP to the truth
    //     (inspector apply_event_impl 353 → 477; paint_transform_section 212 → 281).
    //   · 8 fns were fully masked and appear here for the first time.
    // This is a correction of the MEASUREMENT, never a licence to grow: the
    // numbers may shrink, never rise, and the honest split (per-section helpers
    // threading `y: f32`) is now unblocked — it is paint/dispatch code with no
    // unit coverage, so each split lands with its own smoke, one panel at a time.
    // ──────────────────────────────────────────────────────────────────────────
    // ph2d-panel-color-equalization populate: 200→203 after `cargo fmt --all`
    // re-flowed long lines (solo Coord 2026-05-29). A data-spec populate; a
    // split into a number_specs helper is the same deferred follow-up.
    (
        "ph2d-panel-color-equalization/src/populate.rs",
        "populate",
        203,
        "fmt --all re-flow pushed it 200→203; data-spec populate, split deferred",
    ),
    // Wave 10 / Etapa 5.2: long paint orchestrators that grew with the panel's
    // feature set. Splitting into per-section helpers is a follow-up Etapa (one
    // panel at a time, with smoke validation).
    (
        "ph2d-panel-hierarchy/src/paint.rs",
        "paint_hierarchy_body",
        388,
        "Wave 10 paint orchestrator; per-section split deferred (needs smoke)",
    ),
    (
        "ph2d-panel-hierarchy/src/row.rs",
        "paint_hierarchy_row",
        291,
        "row painter (icons + twirl + rename + companions); re-baselined 300→291 by the comment-aware parser",
    ),
    (
        "ph2d-panel-hierarchy/src/event.rs",
        "apply_event",
        216,
        "unmasked by the 2026-07-10 parser fix; first-match-wins click dispatch, per-cluster try_* split deferred",
    ),
    // The inspector is the worst offender and the reason the split was blocked:
    // the parser under-counted its dispatcher by 124 LOC.
    (
        "ph2d-panel-inspector/src/event.rs",
        "apply_event_impl",
        477,
        "was frozen at a mis-measured 353; truly 477. Sequence of independent first-match-wins `if let WidgetEvent::…` blocks — per-cluster try_* split is ready and now unblocked",
    ),
    (
        "ph2d-panel-inspector/src/paint.rs",
        "paint_inspector",
        431,
        "unmasked by the 2026-07-10 parser fix; §0-§9 section orchestrator, per-section split deferred (needs smoke)",
    ),
    (
        "ph2d-panel-inspector/src/sections/render_source.rs",
        "paint_render_source_section",
        307,
        "unmasked by the 2026-07-10 parser fix; per-row split deferred (needs smoke)",
    ),
    (
        "ph2d-panel-inspector/src/sections/transform.rs",
        "paint_transform_section",
        281,
        "was frozen at a mis-measured 212; truly 281. Per-row split deferred (needs smoke)",
    ),
    (
        "ph2d-panel-inspector/src/sync.rs",
        "sync_sprite_fields",
        202,
        "unmasked by the 2026-07-10 parser fix; field-by-field mirror, 2 LOC over — split is mechanical",
    ),
    // Painter W4 adjustment dispatch + param rows grew with the bespoke kinds.
    (
        "ph2d-panel-painter-layers/src/event.rs",
        "apply_event_impl",
        281,
        "Painter W4 adjustment event dispatch; re-baselined 299→281 by the comment-aware parser",
    ),
    (
        "ph2d-panel-painter-layers/src/paint.rs",
        "paint",
        273,
        "unmasked by the 2026-07-10 parser fix; layer-stack paint orchestrator, per-section split deferred (needs smoke)",
    ),
    (
        "ph2d-panel-equalize-sizes/src/paint.rs",
        "paint_body_sections",
        255,
        "unmasked by the 2026-07-10 parser fix; per-section split deferred (needs smoke)",
    ),
    (
        "ph2d-panel-audio-mixer/src/paint.rs",
        "paint",
        225,
        "unmasked by the 2026-07-10 parser fix; per-strip split deferred (needs smoke)",
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

/// Guard the allowance list itself. An entry whose function has been split
/// (or deleted) below the cap is dead weight: it silently re-permits the
/// overage if the function ever grows back. The 2026-07-10 re-baseline found
/// three such fossils — `grid-snap::populate` (really 126 LOC, frozen at 235),
/// `inspector::paint_color_tint_section` (124, frozen at 289) and
/// `painter-layers::paint_adjustment_params` (54, frozen at 227) — each one a
/// standing licence to triple in size unnoticed. Mirrors the same guard on
/// `architecture_workspace_file_loc_cap`.
#[test]
fn fn_overage_allowlist_has_no_stale_entries() {
    let crates_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut measured: Vec<(String, String, usize)> = Vec::new();
    for panel_dir in collect_panel_dirs(&crates_root) {
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
                .strip_prefix(&panel_dir)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.display().to_string());
            let key = format!("{crate_name}/{rel}");
            for (fn_name, loc) in extract_fn_locs(&body) {
                measured.push((key.clone(), fn_name, loc));
            }
        });
    }

    let mut stale: Vec<String> = Vec::new();
    for (key, fn_name, allowed, _) in FN_OVERAGE_OK {
        match measured
            .iter()
            .find(|(k, f, _)| k == key && f == fn_name)
            .map(|(_, _, loc)| *loc)
        {
            None => stale.push(format!("{key}::{fn_name} — function no longer exists")),
            Some(loc) if loc <= PANEL_FN_LOC_CAP => stale.push(format!(
                "{key}::{fn_name} — now {loc} LOC, under the {PANEL_FN_LOC_CAP} cap"
            )),
            Some(loc) if loc < *allowed => stale.push(format!(
                "{key}::{fn_name} — now {loc} LOC, entry still frozen at {allowed}"
            )),
            Some(_) => {}
        }
    }

    assert!(
        stale.is_empty(),
        "FN_OVERAGE_OK entries that no longer describe reality:\n  {}\n\
         fix: delete the entry (fn is under the cap) or lower it to the \
         measured LOC. Allowances shrink; they never grow.",
        stale.join("\n  ")
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
/// matching `}`, inclusive, and skips `#[cfg(test)]` modules entirely.
///
/// The brace walk is **comment-aware** (see [`find_matching_brace`]).
/// It used to toggle a naive `in_char` flag on every `'`, so a prose
/// apostrophe inside a `//` comment ("doesn't", "sprite's") or a
/// lifetime tick (`&'a`) desynchronised the depth counter and closed
/// the function early — under-counting it. `apply_event_impl` in
/// `ph2d-panel-inspector` measured 353 that way and is really 477.
fn extract_fn_locs(src: &str) -> Vec<(String, usize)> {
    let stripped = strip_test_modules(src);
    let mut out: Vec<(String, usize)> = Vec::new();
    let mut i = 0;
    while i < stripped.len() {
        let Some((name, body_start)) = find_fn_opener(&stripped, i) else {
            break;
        };
        let Some(body_end) = find_matching_brace(&stripped, body_start) else {
            break;
        };
        out.push((name, stripped[body_start..=body_end].lines().count()));
        i = body_end + 1;
    }
    out
}

/// Walk from the `{` at `open` to its matching `}`, ignoring braces that
/// live inside a comment, a string (raw or not) or a char literal.
/// Returns the byte index of the closing `}`.
fn find_matching_brace(src: &str, open: usize) -> Option<usize> {
    let b = src.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    while i < b.len() {
        match b[i] {
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => i = find_line_end(b, i),
            b'/' if i + 1 < b.len() && b[i + 1] == b'*' => {
                i = i + 2 + src[i + 2..].find("*/")? + 2;
            }
            // `r"…"` / `r#"…"#` (and the `r` of `br"…"`, whose `b` is inert).
            b'r' if i + 1 < b.len() && matches!(b[i + 1], b'"' | b'#') => {
                match skip_raw_string(src, i) {
                    Some(next) => i = next,
                    // A raw *identifier* (`r#type`) — not a string.
                    None => i += 1,
                }
            }
            b'"' => i = skip_string(b, i)?,
            b'\'' => i = skip_char_or_lifetime(b, i),
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

/// From the opening `"`, return the index just past the closing one.
fn skip_string(b: &[u8], from: usize) -> Option<usize> {
    let mut i = from + 1;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'"' => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

/// From the `r` of `r##"…"##`, return the index just past the terminator.
/// `None` when this `r` opens a raw identifier rather than a raw string.
fn skip_raw_string(src: &str, from: usize) -> Option<usize> {
    let b = src.as_bytes();
    let mut i = from + 1;
    let mut hashes = 0usize;
    while i < b.len() && b[i] == b'#' {
        hashes += 1;
        i += 1;
    }
    if i >= b.len() || b[i] != b'"' {
        return None;
    }
    i += 1;
    let mut terminator = String::with_capacity(hashes + 1);
    terminator.push('"');
    terminator.extend(std::iter::repeat_n('#', hashes));
    src[i..]
        .find(&terminator)
        .map(|rel| i + rel + terminator.len())
}

/// From a `'`, return the index just past a char literal (`'x'`, `'\n'`,
/// `b'{'`), or just past the tick alone when it opens a lifetime (`&'a`,
/// `'static`) — a lifetime has no closing quote to find.
fn skip_char_or_lifetime(b: &[u8], from: usize) -> usize {
    let after_tick = from + 1;
    if after_tick < b.len() && b[after_tick] == b'\\' {
        // Escaped: scan to the closing quote (`'\n'`, `'\''`, `'\u{1F}'`).
        let mut j = after_tick + 1;
        while j < b.len() && b[j] != b'\'' {
            j += 1;
        }
        return if j < b.len() { j + 1 } else { after_tick };
    }
    // Unescaped char literals are one scalar wide; a quote further out than
    // that means we are looking at a lifetime, not a literal.
    let mut j = after_tick;
    while j < b.len() && j < after_tick + 4 {
        if b[j] == b'\'' {
            return j + 1;
        }
        j += 1;
    }
    after_tick
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
        // Comment-aware, like `extract_fn_locs` — a brace quoted inside a
        // test's string or comment must not end the module early.
        let Some(k) = find_matching_brace(src, j) else {
            break;
        };
        // Skip the test mod entirely.
        i = k + 1;
    }
    out
}
