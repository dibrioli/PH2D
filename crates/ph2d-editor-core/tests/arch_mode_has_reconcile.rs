//! Wave 10 / Etapa 5.3 — every `pub fn set_<X>_mode` setter must
//! either reconcile dependent state or be on the opt-out list.
//!
//! Recurring bug class (Image Tools Bug §2): a setter wrote a single
//! field (`self.mode = m`) but ignored downstream state that the
//! mode-switch invalidates (e.g. cached previews, derived params,
//! widget enable masks). Result: the tool reverted to stale state on
//! the next paint.
//!
//! Canonical pattern: a setter that switches MODE-style state should
//! either (a) reconcile downstream state inline, or (b) call a
//! companion `reconcile_<X>_state` (or `reset_<X>_caches`,
//! `invalidate_<X>`) helper.
//!
//! Gate scans all crates' src/ for `pub fn set_*_mode` signatures,
//! reads the setter body, and verifies one of:
//!   - body contains the substring `reconcile_` OR `invalidate_`
//!     OR `reset_` (companion call), OR
//!   - body is a simple field write AND the symbol is on `BENIGN_SET_MODE`
//!     opt-out below (benign single-field setters).

use std::fs;
use std::path::{Path, PathBuf};

/// Setters that legitimately need only a single field write. Each
/// entry: (function name, justification). Adding here requires
/// confirming the setter has NO downstream state to reconcile (most
/// new tools/panels DO; resist the urge).
const BENIGN_SET_MODE: &[(&str, &str)] = &[
    (
        "set_tool_view_mode",
        "TopBar icon view mode — purely visual, no dependent caches",
    ),
    (
        "set_blender_channel_mode",
        "BlenderPicker RGB↔HSV display only; channel values are derived per-paint",
    ),
];

#[test]
fn every_set_mode_setter_reconciles_or_is_benign() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let crates_dir = workspace_root.join("crates");
    let shells_dir = workspace_root.join("shells");

    let mut files: Vec<PathBuf> = Vec::new();
    collect_rs(&crates_dir, &mut files);
    if shells_dir.is_dir() {
        collect_rs(&shells_dir, &mut files);
    }
    // Don't scan this gate's own source.
    files.retain(|p| !p.ends_with("arch_mode_has_reconcile.rs"));

    let benign_names: Vec<&str> = BENIGN_SET_MODE.iter().map(|(n, _)| *n).collect();
    let mut violations: Vec<String> = Vec::new();
    for path in &files {
        let Ok(src) = fs::read_to_string(path) else {
            continue;
        };
        for (fn_name, body) in extract_set_mode_setters(&src) {
            if benign_names.contains(&fn_name.as_str()) {
                continue;
            }
            // Reconciliation signal: body either uses one of the
            // canonical helper names OR makes at least one method call
            // beyond the bare field write (which is what the bug
            // pattern lacked). A "method call beyond field write" is a
            // `.<ident>(` pattern that is NOT a `<field>.<field>` access
            // (e.g. `self.surface.configure(...)`,
            // `self.atlas.rebuild_caches()`, `crate::sampler::new(...)`).
            let canonical_helper = body.contains("reconcile_")
                || body.contains("invalidate_")
                || body.contains("reset_")
                || body.contains("on_mode_changed");
            let has_method_call = body_has_method_call(&body);
            let reconciles = canonical_helper || has_method_call;
            if !reconciles {
                violations.push(format!(
                    "{}::{} — body is a bare field write with no reconcile / method call",
                    path.display(),
                    fn_name
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "`set_*_mode` setter(s) without a reconcile/invalidate call:\n  \
         {}\n\n\
         Either: (a) reconcile downstream state inline (clear caches, \
         reset derived params, etc.), (b) call a companion \
         `reconcile_<X>_state` helper, or (c) — only if the setter is \
         truly a single-field write with no dependents — add the name \
         to BENIGN_SET_MODE in this test with a 1-line justification.",
        violations.join("\n  ")
    );
}

/// Find every `pub fn set_<name>_mode(...) { ... }` and return
/// `(fn_name, body_source)` pairs. Body is the brace-delimited block.
fn extract_set_mode_setters(src: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let bytes = src.as_bytes();
    let needle = "pub fn set_";
    let mut search_from = 0;
    while let Some(rel) = src[search_from..].find(needle) {
        let pos = search_from + rel;
        // Extract the function name: `pub fn set_<name>(` — name is
        // everything between `set_` and the first `(` (must contain `_mode`).
        let after = &src[pos + "pub fn ".len()..];
        let Some(paren) = after.find('(') else {
            break;
        };
        let name = after[..paren].trim().to_string();
        if !name.contains("_mode") || !name.starts_with("set_") {
            search_from = pos + 1;
            continue;
        }
        // Walk to opening `{`.
        let mut i = pos + "pub fn ".len() + paren;
        let mut paren_depth = 1i32;
        while i < bytes.len() && paren_depth > 0 {
            i += 1;
            if i >= bytes.len() {
                break;
            }
            match bytes[i] {
                b'(' => paren_depth += 1,
                b')' => paren_depth -= 1,
                _ => {}
            }
        }
        // Skip the return type until `{`.
        while i < bytes.len() && bytes[i] != b'{' && bytes[i] != b';' {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b';' {
            search_from = pos + 1;
            continue;
        }
        let body_start = i;
        let mut depth = 0i32;
        while i < bytes.len() {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let body = src[body_start..=i].to_string();
        out.push((name, body));
        search_from = i + 1;
    }
    out
}

/// True if `body` contains at least one `.<ident>(` call that isn't
/// a simple field access pattern. We strip strings and comments first
/// so they don't false-positive.
fn body_has_method_call(body: &str) -> bool {
    let stripped = strip_strings_and_comments(body);
    let bytes = stripped.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' {
            // Look ahead: must be `.<ident>(` (with no whitespace, or
            // ignore trailing whitespace before `(`).
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if j == i + 1 {
                i += 1;
                continue;
            }
            // After the ident, skip any whitespace.
            let mut k = j;
            while k < bytes.len() && bytes[k] == b' ' {
                k += 1;
            }
            if k < bytes.len() && bytes[k] == b'(' {
                // It's `.<ident>(...)` — a method call. Done.
                return true;
            }
        }
        // Also accept free-fn calls like `foo::bar(...)` or `crate::baz(...)`.
        if bytes[i] == b':' && i + 1 < bytes.len() && bytes[i + 1] == b':' {
            let mut j = i + 2;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if j > i + 2 {
                let mut k = j;
                while k < bytes.len() && bytes[k] == b' ' {
                    k += 1;
                }
                if k < bytes.len() && bytes[k] == b'(' {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

/// Strip string and char literals + line/block comments. Naive but
/// good enough for setter bodies.
fn strip_strings_and_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let n = bytes.len();
    let mut out = String::with_capacity(n);
    let mut i = 0;
    while i < n {
        // Line comment
        if i + 1 < n && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Block comment
        if i + 1 < n && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(n);
            continue;
        }
        // String literal
        if bytes[i] == b'"' {
            out.push('"');
            i += 1;
            while i < n && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < n {
                    i += 2;
                    continue;
                }
                i += 1;
            }
            i = (i + 1).min(n);
            out.push('"');
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            // Skip target/ and node_modules/-style trees.
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if name == "target" || name == "node_modules" || name.starts_with('.') {
                continue;
            }
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn detector_extracts_set_mode_signatures() {
    let src = r#"
pub fn set_view_mode(&mut self, m: u8) {
    self.view = m;
}
pub fn set_brush_mode(&mut self, m: BrushMode) {
    self.brush = m;
    self.reconcile_brush_state();
}
"#;
    let setters = extract_set_mode_setters(src);
    assert_eq!(setters.len(), 2);
    assert_eq!(setters[0].0, "set_view_mode");
    assert_eq!(setters[1].0, "set_brush_mode");
    assert!(setters[1].1.contains("reconcile_"));
}
