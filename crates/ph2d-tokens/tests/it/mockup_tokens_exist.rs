//! Wave 9 Eixo C.1 — every CSS custom property referenced by a design
//! mockup in `docs/design/screens/*.html` must be defined somewhere
//! the browser can actually resolve: either in `docs/design/styles/*.css`
//! (canonical alias layer over `tokens.json` — e.g. `--fw-medium: 500`),
//! in `docs/design/tokens.json` directly (raw token key), or as a
//! `--var-name: value;` inside the mockup itself (per-screen knob).
//!
//! Why both `styles/*.css` AND `tokens.json`: the CSS layer carries
//! the short alias names (`fw-medium`, `fs-sm`, `r-lg`) that the
//! mockups actually write, while `tokens.json` is the raw OKLCH /
//! numeric source. A `var(--accent-glow)` whose name resolves nowhere
//! means either:
//! - a new design token landed in a mockup before the alias layer was
//!   updated, or
//! - the mockup has a typo and the var silently resolves to nothing.
//!
//! Either way it's a review gate. To fix, add the token (raw value in
//! `tokens.json` AND alias in `styles/tokens.css`) or correct the
//! mockup.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn screens_dir() -> PathBuf {
    repo_root().join("docs/design/screens")
}

fn collect_var_references(html: &str) -> BTreeSet<String> {
    let bytes = html.as_bytes();
    let mut out = BTreeSet::new();
    let mut i = 0;
    while i + 5 < bytes.len() {
        if &bytes[i..i + 6] == b"var(--" {
            let start = i + 6;
            let mut end = start;
            while end < bytes.len() {
                let c = bytes[end];
                if c.is_ascii_alphanumeric() || c == b'-' || c == b'_' {
                    end += 1;
                } else {
                    break;
                }
            }
            // Skip vars that carry a fallback (`var(--X, fallback)`) —
            // those are per-instance knobs the author already declared
            // optional. We only gate vars whose absence would silently
            // resolve to the empty string.
            let has_fallback = end < bytes.len() && bytes[end] == b',';
            if end > start && !has_fallback {
                out.insert(String::from_utf8_lossy(&bytes[start..end]).into_owned());
            }
            i = end;
        } else {
            i += 1;
        }
    }
    out
}

fn collect_var_definitions(html: &str) -> BTreeSet<String> {
    let bytes = html.as_bytes();
    let mut out = BTreeSet::new();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2].is_ascii_alphabetic() {
            let start = i + 2;
            let mut end = start;
            while end < bytes.len() {
                let c = bytes[end];
                if c.is_ascii_alphanumeric() || c == b'-' || c == b'_' {
                    end += 1;
                } else {
                    break;
                }
            }
            // Treat as a definition only if followed by ':' (CSS prop).
            let mut j = end;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b':' {
                out.insert(String::from_utf8_lossy(&bytes[start..end]).into_owned());
            }
            i = end;
        } else {
            i += 1;
        }
    }
    out
}

/// Walk the tokens.json text and collect every leaf key under the
/// first theme block. We don't need a JSON parser for this — a regex-
/// free scan over `"<key>":` lines is sufficient given the
/// hand-authored shape of `tokens.json`.
fn collect_tokens_json_keys(json: &str) -> BTreeSet<String> {
    let bytes = json.as_bytes();
    let mut out = BTreeSet::new();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'"' {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end] != b'"' {
                if bytes[end] == b'\\' {
                    end += 2;
                } else {
                    end += 1;
                }
            }
            if end < bytes.len() {
                // Followed by `":` (allowing whitespace) means this is
                // a JSON key.
                let mut j = end + 1;
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b':' {
                    let key = String::from_utf8_lossy(&bytes[start..end]).into_owned();
                    if !key.is_empty() && !key.starts_with('$') {
                        out.insert(key);
                    }
                }
                i = end + 1;
            } else {
                break;
            }
        } else {
            i += 1;
        }
    }
    out
}

fn list_html_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !dir.exists() {
        return out;
    }
    for entry in std::fs::read_dir(dir).expect("dir readable") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("html") {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn collect_styles_definitions() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let styles_dir = repo_root().join("docs/design/styles");
    if !styles_dir.exists() {
        return out;
    }
    for entry in std::fs::read_dir(&styles_dir).expect("styles/ readable") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("css") {
            continue;
        }
        let css = std::fs::read_to_string(&path).expect("css file readable");
        for var in collect_var_definitions(&css) {
            out.insert(var);
        }
    }
    out
}

#[test]
fn mockup_vars_resolve_to_known_tokens_or_local_definitions() {
    let tokens_json_path = repo_root().join("docs/design/tokens.json");
    let tokens_json = std::fs::read_to_string(&tokens_json_path).unwrap_or_default();
    let token_keys = collect_tokens_json_keys(&tokens_json);
    let styles_defs = collect_styles_definitions();

    let files = list_html_files(&screens_dir());
    if files.is_empty() {
        eprintln!(
            "no mockup HTML files at {} — skipping (early-bringup)",
            screens_dir().display()
        );
        return;
    }

    let mut unknown: Vec<(String, String)> = Vec::new();
    for path in &files {
        let html = std::fs::read_to_string(path).expect("mockup HTML readable");
        let used = collect_var_references(&html);
        let local = collect_var_definitions(&html);
        for var in &used {
            if local.contains(var) {
                continue;
            }
            if styles_defs.contains(var) {
                continue;
            }
            if token_keys.contains(var) {
                continue;
            }
            unknown.push((
                path.file_name().unwrap().to_string_lossy().into_owned(),
                var.clone(),
            ));
        }
    }
    assert!(
        unknown.is_empty(),
        "design mockups reference CSS vars that aren't defined locally, \
         in docs/design/styles/*.css, or in docs/design/tokens.json:\n  {}\n\
         fix: add the token to tokens.json + alias in styles/tokens.css, \
         or correct the mockup.",
        unknown
            .iter()
            .map(|(f, v)| format!("{f}: --{v}"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
