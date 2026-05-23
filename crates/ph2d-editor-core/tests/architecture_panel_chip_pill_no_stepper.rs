//! DIRETRIZ §4.2 — every `ph2d-panel-*` that paints a slider+chip pair
//! via `paint_slider_with_chip` / `paint_slider_with_chip_layout` (i.e.
//! uses the bare pill `paint_number_chip` underneath) must either:
//!
//!   (a) call `store.link_slider_number(slider, chip)` for that pair
//!       in its populate (the link auto-marks the chip as no-stepper),
//!       OR
//!   (b) call `store.mark_chip_no_stepper(chip)` explicitly for every
//!       chip painted as a pill.
//!
//! Without (a) or (b), the dispatch's default stepper hit-test carves
//! a 16-22 px column on the chip's right edge, arming
//! `number_stepper_hold` on click and incrementing the value every
//! 30 ms via `dispatch_tick` while the cursor stays still ("valor
//! sobe sozinho com mouse parado"). Burned: CEQ slot 1 (commit
//! `2f58b73`).
//!
//! Detection is intentionally coarse — text-scan of every panel's
//! `src/` tree: if any `.rs` file uses `paint_slider_with_chip` and
//! NO file in the panel's `src/` calls `link_slider_number` OR
//! `mark_chip_no_stepper`, the gate fails. Catches the obvious
//! "forgot to populate the link" omission; a panel that scrubs by
//! some other mechanism can opt out via `OPT_OUT`.

use std::path::{Path, PathBuf};

/// Panels that legitimately paint via `paint_slider_with_chip*` but
/// don't need link/mark.
const OPT_OUT: &[(&str, &str)] = &[(
    "ph2d-panel-grid-snap",
    "Opacity slider passes `NodeId(0)` for the chip id — the painter \
     skips the chip hit-index register and the chip is paint-only, so \
     the dispatch never resolves it as a NumberInput target.",
)];

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

#[test]
fn every_pill_chip_panel_links_or_marks_no_stepper() {
    let mut failures: Vec<String> = Vec::new();

    for panel_dir in panel_crate_dirs() {
        let name = panel_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        if OPT_OUT.iter().any(|(n, _)| *n == name.as_str()) {
            continue;
        }

        let src = panel_dir.join("src");
        let mut paints_pill_chip = false;
        let mut wires_correctly = false;

        walk_rs(&src, |_path, text| {
            // Strip comment lines so a doc-block reference doesn't
            // satisfy the wiring check. Coarse but sufficient — the
            // calls we care about are real statements at column 0+.
            let mut code_only = String::with_capacity(text.len());
            for line in text.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("//") || trimmed.starts_with("//!") {
                    continue;
                }
                code_only.push_str(line);
                code_only.push('\n');
            }

            if code_only.contains("paint_slider_with_chip") {
                paints_pill_chip = true;
            }
            if code_only.contains("link_slider_number")
                || code_only.contains("mark_chip_no_stepper")
            {
                wires_correctly = true;
            }
        });

        if paints_pill_chip && !wires_correctly {
            failures.push(format!(
                "{name}: paints `paint_slider_with_chip*` (pill chip without arrows) but populate \
                 has neither `link_slider_number` nor `mark_chip_no_stepper`. \
                 The dispatch will carve a phantom stepper column on the chip's right edge — \
                 click there arms a continuous-hold that increments the value at 30 ms intervals \
                 with the cursor still. See DIRETRIZ §4.2 + commit `2f58b73`."
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Panels paint pill chips but don't wire `link_slider_number` / `mark_chip_no_stepper`:\n  - {}",
        failures.join("\n  - ")
    );
}
