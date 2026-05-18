//! HR-15 enforcement: no hardcoded UI strings in widget production code.
//!
//! Catches the two patterns that feed user-facing text to AccessKit /
//! visible UI text and bypass localization:
//!
//! - `.label("...")` — a11y label setter.
//! - `.placeholder("...")` — text input placeholder.
//!
//! Doc comments, `#[cfg(test)]` mod blocks (and their `#[test]`
//! functions), and strings passed to non-UI APIs (token names, log
//! messages, panics) are NOT targets — they aren't user-visible UI
//! strings, and HR-15 explicitly exempts them.
//!
//! Until the Fluent runtime (`ph2d-i18n`) is wired and `t!(...)`
//! exists, this test enforces a **frozen baseline**: each file in
//! `BASELINE` is allowed exactly the listed number of violations.
//! Any file not in `BASELINE` must have zero. When `t!(...)` ships,
//! reduce `BASELINE` entries to 0 (or remove) and the test becomes
//! strict.
//!
//! Failure modes this catches:
//! - New widget introduces a hardcoded `.label(...)` or
//!   `.placeholder(...)` outside test code.
//! - Existing widget adds violations beyond its baseline count.
//! - Existing widget reduces violations below baseline (forces the
//!   maintainer to drop the baseline entry, keeping the list honest).

use std::fs;
use std::path::{Path, PathBuf};

/// Frozen baseline. Each entry: (relative path under `src/widget/`,
/// allowed violation count in production code). Adding entries
/// requires review — every entry is technical debt against HR-15.
const BASELINE: &[(&str, usize)] = &[
    ("card.rs", 1),    // .label("card") — generic a11y fallback
    ("divider.rs", 1), // .label("separator") — generic a11y fallback
    ("popover.rs", 1), // .label("popover") — generic a11y fallback
    // Wave 8 Phase 2.A — showcase is a hardcoded English reference
    // gallery (peripheral agents read it to see canonical widget
    // appearance). i18n migration tracked separately.
    ("showcase/inputs.rs", 2),
];

#[test]
fn no_new_hardcoded_ui_strings_in_widgets() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/widget");
    let mut counts: Vec<(String, usize)> = Vec::new();
    walk(&root, &root, &mut |relpath, abspath| {
        if abspath.extension().and_then(|s| s.to_str()) != Some("rs") {
            return;
        }
        let content = fs::read_to_string(abspath).expect("read widget file");
        let production = strip_test_modules(&content);
        let n = count_violations(&production);
        if n > 0 {
            let rel = relpath.to_string_lossy().replace('\\', "/");
            counts.push((rel, n));
        }
    });

    let mut errors = Vec::new();

    for (file, found) in &counts {
        let expected = BASELINE
            .iter()
            .find(|(b, _)| *b == file.as_str())
            .map(|(_, n)| *n)
            .unwrap_or(0);
        if *found > expected {
            errors.push(format!(
                "  {file} — found {found} violation(s), baseline allows {expected}"
            ));
        }
    }

    for (b_file, b_count) in BASELINE {
        let current = counts
            .iter()
            .find(|(f, _)| f.as_str() == *b_file)
            .map(|(_, n)| *n)
            .unwrap_or(0);
        if current < *b_count {
            errors.push(format!(
                "  baseline obsolete: {b_file} expected {b_count}, now {current} — please reduce BASELINE entry"
            ));
        }
    }

    assert!(
        errors.is_empty(),
        "HR-15 violation — hardcoded UI strings in widget code:\n{}\n\n\
         Either replace with `t!(...)` (when Fluent runtime ships) or — \
         if the string is a stable generic fallback — adjust BASELINE in \
         this test.",
        errors.join("\n"),
    );
}

/// Strip `#[cfg(test)]` mod blocks (and their `#[test]` functions)
/// from the source so production-only code remains. Brace-counting
/// based: when `#[cfg(test)]` is seen, skip until the matching `}`.
/// Naive about strings containing `{`/`}` — adequate for widget code,
/// where brace literals in user-facing strings are extremely rare.
fn strip_test_modules(src: &str) -> String {
    let lines: Vec<&str> = src.lines().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        if trimmed.starts_with("#[cfg(test)]") {
            i += 1;
            // Find next non-attribute line.
            while i < lines.len() && lines[i].trim_start().starts_with("#[") {
                i += 1;
            }
            if i >= lines.len() {
                break;
            }
            // If the item is a single-line statement (use/const), just
            // skip that one line and continue.
            let next = lines[i];
            if next.trim_end().ends_with(';') && !next.contains('{') {
                i += 1;
                continue;
            }
            // Walk braces until depth returns to 0.
            let mut depth: i32 = 0;
            let mut started = false;
            loop {
                if i >= lines.len() {
                    break;
                }
                for c in lines[i].chars() {
                    match c {
                        '{' => {
                            depth += 1;
                            started = true;
                        }
                        '}' => depth -= 1,
                        _ => {}
                    }
                }
                i += 1;
                if started && depth <= 0 {
                    break;
                }
            }
        } else {
            out.push_str(line);
            out.push('\n');
            i += 1;
        }
    }
    out
}

/// Count `.label("...")` and `.placeholder("...")` occurrences in the
/// production slice. Skips lines that look like comments (`//`).
fn count_violations(production: &str) -> usize {
    let mut n = 0;
    for line in production.lines() {
        let t = line.trim_start();
        if t.starts_with("//") {
            continue;
        }
        n += count_pattern(line, ".label(\"");
        n += count_pattern(line, ".placeholder(\"");
    }
    n
}

fn count_pattern(haystack: &str, needle: &str) -> usize {
    let mut n = 0;
    let mut idx = 0;
    while idx < haystack.len() {
        match haystack[idx..].find(needle) {
            Some(p) => {
                n += 1;
                idx += p + needle.len();
            }
            None => break,
        }
    }
    n
}

fn walk(root: &Path, dir: &Path, cb: &mut dyn FnMut(&Path, &Path)) {
    for entry in fs::read_dir(dir).expect("read widget dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, cb);
        } else if let Ok(rel) = path.strip_prefix(root) {
            cb(rel, &path);
        }
    }
}
