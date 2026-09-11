//! Wave 2 PR 11.9 — HR-18 file-LOC cap gate, **active**.
//!
//! Every `.rs` file in `shells/desktop/src/` must stay under 600 LOC,
//! per HR-18 in `SKILL_Stack_PH2D_Definitiva.md` §HR-18:
//!
//! > Arquivos em `shells/<plataforma>/src/` respeitam caps de tamanho:
//! > - Qualquer arquivo `.rs`: ≤ 600 LOC
//!
//! Exceptions are declared inline at the top of the offending file
//! using the comment marker `// ph2d-loc-cap: <reason>` (per SKILL
//! §HR-18 "Exceções por `// ph2d-loc-cap: <razão>` no topo do
//! arquivo"). The marker MUST appear within the first 20 lines.
//!
//! When PR 11.8 (Action Bus) lands and decomposes `main.rs` +
//! `hero_intents.rs`, those `// ph2d-loc-cap:` markers come out and
//! this lint enforces unconditionally.

use std::fs;
use std::path::{Path, PathBuf};

/// Hard cap from HR-18 §2. The function/body caps (200/400) live in
/// a separate test file when activated — this test only enforces the
/// file-level cap.
const FILE_LOC_CAP: usize = 600;

/// First-line window in which the `// ph2d-loc-cap:` exception
/// marker is accepted. Keeps the escape hatch impossible to hide deep
/// in a file — a reviewer scanning the head of the file SEES the
/// debt declaration.
const EXCEPTION_WINDOW_LINES: usize = 20;

/// Walk a directory and return every `.rs` file.
fn collect_rs(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
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

/// Returns `Some(reason)` if the file declares a `// ph2d-loc-cap:`
/// exception within the first [`EXCEPTION_WINDOW_LINES`] lines.
fn loc_cap_exception(text: &str) -> Option<String> {
    for (i, line) in text.lines().enumerate() {
        if i >= EXCEPTION_WINDOW_LINES {
            break;
        }
        if let Some(idx) = line.find("ph2d-loc-cap:") {
            let after = line[idx + "ph2d-loc-cap:".len()..].trim();
            return Some(after.to_string());
        }
    }
    None
}

#[test]
fn shell_files_respect_hr18_loc_cap() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_root = crate_root.join("src");
    let files = collect_rs(&src_root);
    assert!(
        !files.is_empty(),
        "expected to find at least one `.rs` file under {}; HR-18 scope is empty?",
        src_root.display(),
    );

    let mut over: Vec<(PathBuf, usize)> = Vec::new();
    for path in files {
        let text = fs::read_to_string(&path).expect("read shell file");
        let loc = text.lines().count();
        if loc > FILE_LOC_CAP && loc_cap_exception(&text).is_none() {
            over.push((path, loc));
        }
    }
    if !over.is_empty() {
        let mut msg = String::from(
            "HR-18 violation — shell files exceed 600 LOC. Either decompose \
             into sub-modules, or — if the file is mid-refactor — declare an \
             explicit exception with `// ph2d-loc-cap: <reason>` near the top \
             (within the first 20 lines):\n",
        );
        for (path, loc) in &over {
            let rel = path
                .strip_prefix(env!("CARGO_MANIFEST_DIR"))
                .unwrap_or(path);
            msg.push_str(&format!("  {} — {} LOC\n", rel.display(), loc));
        }
        panic!("{msg}");
    }
}

/// Audit: list every `ph2d-loc-cap:` exception currently active.
/// Doesn't fail; just emits the inventory for review in CI logs. New
/// exceptions show up here so reviewers can spot rising debt.
#[test]
fn loc_cap_exceptions_inventory() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_root = crate_root.join("src");
    let mut active = Vec::new();
    for path in collect_rs(&src_root) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if let Some(reason) = loc_cap_exception(&text) {
            let loc = text.lines().count();
            let rel = path
                .strip_prefix(env!("CARGO_MANIFEST_DIR"))
                .unwrap_or(&path)
                .to_path_buf();
            active.push((rel, loc, reason));
        }
    }
    if active.is_empty() {
        // No exceptions active — that's the long-term goal. Print a
        // line so the inventory is visible in the test output either
        // way.
        eprintln!("HR-18 loc-cap exceptions inventory: NONE (cap fully active)");
    } else {
        eprintln!("HR-18 loc-cap exceptions inventory:");
        for (path, loc, reason) in &active {
            eprintln!("  {} — {} LOC — {}", path.display(), loc, reason);
        }
    }
}
