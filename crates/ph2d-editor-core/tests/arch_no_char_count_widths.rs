//! Wave 10 / Etapa 5.3 — ban `.chars().count() *` as a string-width
//! proxy in any UI-painting crate.
//!
//! Recurring bug class (UI_Bugs §3.3, §9.16, §10.1): code measured a
//! string's display width via `s.chars().count() * GLYPH_W`. This
//! breaks for:
//!   - non-monospaced glyphs (Inter is proportional, NOT monospace)
//!   - combining marks (`é` = 1 char, 1 grapheme, but multiple cluster)
//!   - emojis / wide CJK (1 char ≠ 1 advance)
//!   - kerning pairs
//!
//! Canonical replacement: `text_system.measure_text(label, font_size)`
//! which uses the live `cosmic-text` shaping pipeline.
//!
//! Scope: `crates/ph2d-editor-core/src/{widget,screens}/` +
//! `crates/ph2d-panel-*/src/**`. Gate skips comments and the
//! `text_system` crate itself (where char-counting is part of glyph
//! iteration, not width measurement).

use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn no_chars_count_as_width_proxy() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crates_root = crate_root.join("..");
    let mut scan_roots: Vec<PathBuf> = vec![
        crate_root.join("src/widget"),
        crate_root.join("src/screens"),
    ];
    if let Ok(entries) = fs::read_dir(&crates_root) {
        let mut panel_srcs: Vec<PathBuf> = entries
            .flatten()
            .filter_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_str()?.to_string();
                if path.is_dir()
                    && name.starts_with("ph2d-panel-")
                    && name != "ph2d-panel-registry-init"
                {
                    let src = path.join("src");
                    if src.is_dir() { Some(src) } else { None }
                } else {
                    None
                }
            })
            .collect();
        panel_srcs.sort();
        scan_roots.extend(panel_srcs);
    }

    let mut files = Vec::new();
    for r in &scan_roots {
        collect_rs(r, &mut files);
    }

    let mut violations: Vec<String> = Vec::new();
    for path in &files {
        let Ok(src) = fs::read_to_string(path) else {
            continue;
        };
        for (lineno, line) in src.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            if line.contains("CHARS-COUNT-OK") {
                continue;
            }
            if line.contains(".chars().count()") && line_uses_as_width(line) {
                violations.push(format!(
                    "{}:{} — `{}`",
                    path.display(),
                    lineno + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "`.chars().count() * <width>` used as a string-width proxy:\n  \
         {}\n\n\
         Use `text_system.measure_text(label, font_size)` for true width \
         (Inter is proportional — `chars().count()` ≠ pixel advance). \
         If the count is genuinely needed as a count (not width), append \
         `// CHARS-COUNT-OK: <reason>`.",
        violations.join("\n  ")
    );
}

/// True when the line uses `.chars().count()` in a context that looks
/// like width arithmetic — multiplied by something, or compared to a
/// pixel/width-named variable.
fn line_uses_as_width(line: &str) -> bool {
    // Pattern A: `.chars().count() *` (arithmetic)
    if line.contains(".chars().count() *") || line.contains(".chars().count()*") {
        return true;
    }
    // Pattern B: `width = .chars().count()` or similar assignment
    let Some(cc_pos) = line.find(".chars().count()") else {
        return false;
    };
    let pre = &line[..cc_pos];
    let pre_low = pre.to_ascii_lowercase();
    // The assignment target / surrounding token mentions width/w/advance
    // — heuristic but catches the real bug patterns.
    pre_low.contains("width =")
        || pre_low.contains("_w =")
        || pre_low.contains("advance")
        || pre_low.contains("text_w")
        || pre_low.contains("label_w")
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn detector_flags_width_misuse_only() {
    assert!(line_uses_as_width("let w = label.chars().count() * 7.0;"));
    assert!(line_uses_as_width(
        "let text_w = s.chars().count() as f32 * GLYPH_W;"
    ));
    // Negative: counting for iteration limit, not width.
    assert!(!line_uses_as_width("if s.chars().count() > 64 { … }"));
    assert!(!line_uses_as_width("let n = s.chars().count();"));
}
