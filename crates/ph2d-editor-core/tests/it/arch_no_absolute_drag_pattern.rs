//! Wave 10 / Etapa 5.3 — ban the "absolute-from-start" drag pattern.
//!
//! Recurring bug class (Burning 4 — CEQ slider, padding chip, brush
//! cursor, color picker hue ring): drag handlers computed
//! `delta = event.x - start_x` and then APPLIED `start_value + delta`
//! directly. Result: any frame the handler dropped (because the panel
//! re-laid-out between frames, or because the start_x was captured
//! before a scroll) made the next move snap by the full accumulated
//! delta. Symptom: knob jumps wildly.
//!
//! The canonical pattern is delta-per-frame: `delta = event.x - prev_x;
//! prev_x = event.x; value += delta * sensitivity`. The previous-pos
//! lives on the InteractiveState so it can't drift relative to the panel.
//!
//! Gate: scan `crates/ph2d-editor-core/src/interaction/` and the
//! `dispatch/` subtree for any line matching `event.[xy] - .*start_[xy]`
//! (case-sensitive, ignoring comments). Hits are flagged.
//!
//! Allowlist via trailing `// DRAG-ABS-OK: <reason>` comment per line
//! — for the rare case where the absolute-from-start is genuinely
//! correct (e.g. computing the initial click offset on press).

use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn no_absolute_drag_pattern_in_interaction_or_dispatch() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut scan_roots: Vec<PathBuf> = Vec::new();
    let candidates = [
        crate_root.join("src/interaction"),
        crate_root.join("src/dispatch"),
        crate_root.join("src/input_dispatch"),
    ];
    for c in &candidates {
        if c.is_dir() {
            scan_roots.push(c.clone());
        }
    }
    // Also scan shells/ + panel-* for the same anti-pattern (Burning 4
    // was a panel-side bug too).
    let workspace_root = crate_root.join("../..");
    let shell = workspace_root.join("shells/desktop/src");
    if shell.is_dir() {
        scan_roots.push(shell);
    }
    let crates_root = crate_root.join("..");
    if let Ok(entries) = fs::read_dir(&crates_root) {
        for entry in entries.flatten() {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if name.starts_with("ph2d-panel-") && name != "ph2d-panel-registry-init" {
                let src = p.join("src");
                if src.is_dir() {
                    scan_roots.push(src);
                }
            }
        }
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
            if line.contains("DRAG-ABS-OK") {
                continue;
            }
            if line_matches_absolute_drag(line) {
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
        "Absolute-from-start drag pattern (Burning 4 class) found:\n  {}\n\n\
         Replace with delta-per-frame:\n  \
         let delta = event.x - prev_x;\n  \
         prev_x = event.x;\n  \
         value += delta * sensitivity;\n\n\
         If the absolute form is genuinely correct (e.g. initial click \
         offset on press), append `// DRAG-ABS-OK: <reason>`.",
        violations.join("\n  ")
    );
}

/// Detect `event.x - <expr>start_x` and y variants. The `<expr>` part
/// can be a field path (`drag.start_x`, `self.start_x`) or bare.
fn line_matches_absolute_drag(line: &str) -> bool {
    for token in ["event.x", "event.y"] {
        let mut search_from = 0;
        while let Some(rel) = line[search_from..].find(token) {
            let pos = search_from + rel;
            let after_token = pos + token.len();
            // Token must end at end-of-line OR at a non-identifier char
            // (so `event.x_pos` doesn't match `event.x`).
            let next_is_id = line[after_token..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
            if next_is_id {
                search_from = after_token;
                continue;
            }
            if let Some(minus_rel) = line[after_token..].find('-') {
                let tail = &line[after_token + minus_rel + 1..];
                if tail.contains("start_x") || tail.contains("start_y") {
                    return true;
                }
            }
            search_from = after_token;
        }
    }
    false
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
fn detector_finds_pattern() {
    assert!(line_matches_absolute_drag(
        "let new_x = anchor_x + (event.x - drag.start_x);"
    ));
    assert!(line_matches_absolute_drag("value = event.y - start_y;"));
    // Negative: delta-per-frame pattern.
    assert!(!line_matches_absolute_drag("let delta = event.x - prev_x;"));
    // Negative: comment.
    assert!(line_matches_absolute_drag("event.x - start_x")); // line content; comment skip happens upstream
}
