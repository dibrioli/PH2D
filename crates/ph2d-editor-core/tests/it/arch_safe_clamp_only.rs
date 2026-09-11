//! Wave 10 / Etapa 5.3 — flag `.clamp(min, max)` without comment in
//! UI-painting code; force authors to either use the canonical
//! `safe_clamp` helper (NaN-aware) or document why the standard clamp
//! is fine on this line.
//!
//! Recurring bug class (UI_Bugs §8): `f32::clamp(min, max)` panics if
//! either bound is NaN OR if `min > max` after a malformed config edit.
//! Several painters fed `clamp(prev_min, prev_max)` where the bounds
//! came from store values — a momentary inversion crashed the editor.
//!
//! Canonical alternative:
//!   - `safe_clamp(v, min, max)` from `ph2d_editor_core::math` (returns
//!     `min` if NaN, swaps bounds if inverted), OR
//!   - trailing comment `// CLAMP-OK: <reason>` documenting why this
//!     site is safe (literal bounds, both non-NaN-able f64 ints, …).
//!
//! Scope: `crates/ph2d-editor-core/src/{widget,screens}/` +
//! `crates/ph2d-panel-*/src/**`. Other crates (math libs, geometry)
//! are out of scope — they handle their own invariants.

use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn clamp_calls_in_ui_are_safe_or_justified() {
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
            if line.contains("CLAMP-OK") {
                continue;
            }
            if line.contains("safe_clamp") {
                continue;
            }
            // Bare `.clamp(` call — flag.
            if line.contains(".clamp(") && line_clamp_has_nonliteral_bounds(line) {
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
        "`.clamp(min, max)` without `safe_clamp` or `// CLAMP-OK`:\n  \
         {}\n\n\
         Use `safe_clamp(v, min, max)` (NaN-aware, swap-tolerant) from \
         `ph2d_editor_core::math`, or — if both bounds are literal \
         non-NaN constants — append `// CLAMP-OK: <reason>`.",
        violations.join("\n  ")
    );
}

/// True if the `.clamp(...)` call has at least one bound that looks
/// dynamic (variable/expression, not a numeric literal). Literal
/// bounds are safe on this gate; dynamic ones risk NaN / inversion.
fn line_clamp_has_nonliteral_bounds(line: &str) -> bool {
    let Some(start) = line.find(".clamp(") else {
        return false;
    };
    let args_start = start + ".clamp(".len();
    let mut depth = 1i32;
    let mut end = args_start;
    let bytes = line.as_bytes();
    while end < bytes.len() {
        match bytes[end] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        end += 1;
    }
    if end >= bytes.len() {
        return false;
    }
    let args = &line[args_start..end];
    // Naive comma split (no parens / no commas in literals expected
    // for clamp bounds; nested generics not relevant in clamp args).
    let parts: Vec<&str> = args.split(',').collect();
    if parts.len() != 2 {
        return false;
    }
    for arg in parts {
        let a = arg.trim();
        if !is_numeric_literal(a) {
            return true;
        }
    }
    false
}

fn is_numeric_literal(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_digit() || first == '-' || first == '+') {
        return false;
    }
    // Allow: digits, `_`, `.`, suffix `f32`/`f64`/`i32`/`u32`/`usize`.
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-' || c == '+')
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
fn detector_distinguishes_dynamic_vs_literal_bounds() {
    // Dynamic bounds — flag.
    assert!(line_clamp_has_nonliteral_bounds("v.clamp(min, max)"));
    assert!(line_clamp_has_nonliteral_bounds(
        "value.clamp(state.lo, state.hi)"
    ));
    // Literal bounds — don't flag.
    assert!(!line_clamp_has_nonliteral_bounds("v.clamp(0.0, 1.0)"));
    assert!(!line_clamp_has_nonliteral_bounds("v.clamp(0.0, 255.0)"));
    assert!(!line_clamp_has_nonliteral_bounds("n.clamp(1, 64)"));
}
