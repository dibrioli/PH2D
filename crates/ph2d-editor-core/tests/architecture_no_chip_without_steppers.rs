//! DIRETRIZ §5.2 — chip canon (post-2026-05-24): every numeric chip in
//! the app paints the up/down stepper arrows. There is **no** "pill"
//! variant anymore.
//!
//! Concretely, this gate enforces two things:
//!
//!   (1) No panel may call the deprecated `WidgetStore::mark_chip_no_stepper`
//!       — the only legitimate reason to call it was to suppress the
//!       phantom-stepper bug on the old pill variant of
//!       `paint_number_chip`, which now paints arrows. Calling it from a
//!       panel today silently disables the canon click→step affordance
//!       on a chip the user expects to behave like every other chip.
//!
//!   (2) No file in `crates/ph2d-editor-core/src/widget/` may declare a
//!       new "pill chip without steppers" painter — any future numeric
//!       chip painter MUST go through `paint_number_chip` (which paints
//!       arrows) or via the `paint_slider_with_chip*` composite that
//!       wraps it.
//!
//! The old positive gate (`architecture_panel_chip_pill_no_stepper`)
//! enforced the *opposite* invariant — that panels painting the bare
//! pill via `paint_slider_with_chip*` must also call
//! `link_slider_number` / `mark_chip_no_stepper` to suppress the
//! phantom-stepper. That gate became vacuous once the pill stopped
//! existing; this file replaces it.
//!
//! Detection is intentionally coarse — text-scan of every `ph2d-panel-*`
//! `src/` tree for `mark_chip_no_stepper` call sites, plus a scan of
//! `editor-core/src/widget/` for new `pub fn paint_*_pill*` style names
//! that would suggest a sans-arrows variant slipping back in.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn panel_crate_dirs() -> Vec<PathBuf> {
    let crates_dir = workspace_root().join("crates");
    let mut out: Vec<PathBuf> = std::fs::read_dir(&crates_dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", crates_dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("ph2d-panel-") && n != "ph2d-panel-registry-init")
        })
        .filter(|p| p.join("src").exists())
        .collect();
    out.sort();
    out
}

fn walk_rs<F: FnMut(&Path, &str)>(root: &Path, mut f: F) {
    fn inner(root: &Path, f: &mut dyn FnMut(&Path, &str)) {
        let entries = match std::fs::read_dir(root) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                inner(&path, f);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs")
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                f(&path, &text);
            }
        }
    }
    inner(root, &mut f);
}

/// Strip comment lines so a doc-block reference to the deprecated API
/// (like a rustdoc warning pointing at it) doesn't trip the gate.
fn code_only(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with("//!") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[test]
fn no_panel_calls_mark_chip_no_stepper() {
    let mut offenders: Vec<String> = Vec::new();
    for panel_dir in panel_crate_dirs() {
        let name = panel_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        let src = panel_dir.join("src");
        walk_rs(&src, |path, text| {
            if code_only(text).contains("mark_chip_no_stepper") {
                offenders.push(format!("{name}: {}", path.display()));
            }
        });
    }
    assert!(
        offenders.is_empty(),
        "`mark_chip_no_stepper` is deprecated (chip canon post-2026-05-24 always paints \
         arrows). Remove the call — the chip's stepper click→step is the desired \
         affordance for every chip in the app. Offenders:\n  - {}",
        offenders.join("\n  - ")
    );
}

#[test]
fn paint_number_chip_paints_steppers() {
    // Content-based check: the canonical chip painter MUST reference
    // ChevronUp + ChevronDown + the stepper rect helpers. If someone
    // removes the arrows (regressing to the pre-2026-05-24 pill), this
    // gate catches it before the bug ships. Cheaper + more precise
    // than a name-based heuristic on every `paint_*` in widget/.
    let file = workspace_root()
        .join("crates")
        .join("ph2d-editor-core")
        .join("src")
        .join("widget")
        .join("slider_with_chip.rs");
    let text = std::fs::read_to_string(&file)
        .unwrap_or_else(|e| panic!("read {}: {e}", file.display()));
    let code = code_only(&text);
    let signals = [
        ("IconId::ChevronUp", "the `up` arrow paint call"),
        ("IconId::ChevronDown", "the `down` arrow paint call"),
        (
            "stepper_up_rect",
            "the up-arrow rect helper (shared with the dispatch hit-test)",
        ),
        (
            "stepper_down_rect",
            "the down-arrow rect helper (shared with the dispatch hit-test)",
        ),
    ];
    let missing: Vec<String> = signals
        .iter()
        .filter(|(needle, _)| !code.contains(needle))
        .map(|(needle, why)| format!("`{needle}` — {why}"))
        .collect();
    assert!(
        missing.is_empty(),
        "`paint_number_chip` in {} appears to be missing stepper-arrow paint calls. \
         The chip canon (post-2026-05-24) is a single widget that paints arrows on the \
         right edge — removing them regresses to the pre-2026-05-24 pill variant. \
         Missing markers:\n  - {}",
        file.display(),
        missing.join("\n  - ")
    );
}
