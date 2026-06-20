//! Architecture gate (blindagem Fase 1.2) — every interactive panel must ship
//! a **behavioral seam test**.
//!
//! ## Why this exists
//!
//! The diagnosis root cause: the project measured STRUCTURAL correctness
//! (compiles / contract surface / LOC) and almost nothing BEHAVIORAL (does the
//! widget actually drive the tool?). 3,354 unit tests, ~2 behavioral harnesses
//! — and the one for the painter was deleted. "Audit" collapsed into "run the
//! gates", and the gates are symbol counters that pass while a feature is dead.
//!
//! This gate makes the behavioral seam test MANDATORY: every `ph2d-panel-*`
//! with an `apply_event` (i.e. an interactive panel — it has `src/event.rs`)
//! must have a test under `tests/` that exercises the real seam through
//! [`ph2d_ui_testkit::MockPanelHost`] (populate → apply_event → bus →
//! handle_panel_event / state → assert the observable effect). A panel that
//! compiles + registers but is dead on click can no longer ship green.
//!
//! Panels not yet backfilled are listed in `BEHAVIORAL_TEST_DEBT` — frozen
//! debt to drive to ZERO, not a permanent exemption. The companion test
//! `debt_list_has_no_stale_entries` fails when a debt panel gains a test, so
//! the list can only shrink.
//!
//! Detection is a text check: the panel has a `tests/**` file mentioning
//! `ph2d_ui_testkit`. That's deliberately loose on *what* the test asserts —
//! quality (the assertion must prove the effect LANDED, not just that an
//! action was emitted) is a review responsibility (DIRETIVA §3/§4). The gate
//! guarantees the test EXISTS; the reviewer guarantees it BITES.
//!
//! Dep-free (std only).

use std::path::{Path, PathBuf};

/// Interactive panels that do NOT yet have a behavioral seam test. FROZEN
/// DEBT — drive to zero (each is a panel a human must currently QA by hand to
/// know it works). Removing an entry once its `tests/` gains a seam test is
/// enforced by `debt_list_has_no_stale_entries`.
const BEHAVIORAL_TEST_DEBT: &[&str] = &[
    // EMPTY — debt cleared 2026-06-20 (blindagem Fase 1): all 12 interactive
    // panels ship a behavioral seam test. Any NEW interactive panel must ship
    // one too (this gate fails otherwise). Do not re-add entries to dodge the
    // requirement — write the test.
];

fn crates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ dir")
        .to_path_buf()
}

/// `ph2d-panel-*` dirs (minus registry-init) that have `src/event.rs` — i.e.
/// are interactive (handle widget events).
fn interactive_panels(root: &Path) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = std::fs::read_dir(root)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", root.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter_map(|p| {
            let name = p.file_name()?.to_str()?.to_string();
            if name.starts_with("ph2d-panel-")
                && name != "ph2d-panel-registry-init"
                // Interactive = handles widget events: a hand-written `event.rs`
                // OR a `panel_seam!`-generated `seam.rs` (Fase 2).
                && (p.join("src/event.rs").is_file() || p.join("src/seam.rs").is_file())
            {
                Some((name, p))
            } else {
                None
            }
        })
        .collect();
    out.sort();
    out
}

/// True iff some `tests/**.rs` file in `panel_dir` references the seam harness.
fn has_seam_test(panel_dir: &Path) -> bool {
    fn walk(dir: &Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                if walk(&p) {
                    return true;
                }
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs")
                && std::fs::read_to_string(&p)
                    .map(|b| b.contains("ph2d_ui_testkit"))
                    .unwrap_or(false)
            {
                return true;
            }
        }
        false
    }
    walk(&panel_dir.join("tests"))
}

#[test]
fn interactive_panels_have_a_behavioral_seam_test() {
    let root = crates_root();
    let debt: std::collections::BTreeSet<&str> = BEHAVIORAL_TEST_DEBT.iter().copied().collect();

    let mut offenders: Vec<String> = Vec::new();
    for (name, dir) in interactive_panels(&root) {
        if has_seam_test(&dir) || debt.contains(name.as_str()) {
            continue;
        }
        offenders.push(name);
    }

    assert!(
        offenders.is_empty(),
        "interactive panels with NO behavioral seam test (compile-green ≠ works):\n  {}\n\n\
         fix: add `crates/<panel>/tests/seam.rs` that drives a real WidgetEvent through \
         `MockPanelHost::apply_panel_event` and asserts the OBSERVABLE effect landed \
         (tool state / grid-snap state changed) — mirror \
         `crates/ph2d-panel-padding/tests/seam.rs`. Add `ph2d-ui-testkit` to the \
         panel's [dev-dependencies]. (Only as a last resort, add the panel to \
         BEHAVIORAL_TEST_DEBT with a tracking note — the goal is to empty that list.)",
        offenders.join("\n  ")
    );
}

#[test]
fn debt_list_has_no_stale_entries() {
    let root = crates_root();
    let mut stale: Vec<String> = Vec::new();
    for name in BEHAVIORAL_TEST_DEBT {
        let dir = root.join(name);
        if !dir.is_dir() {
            stale.push(format!("{name} (crate gone — remove from debt list)"));
        } else if has_seam_test(&dir) {
            stale.push(format!(
                "{name} (now HAS a seam test — remove from debt list)"
            ));
        }
    }
    assert!(
        stale.is_empty(),
        "stale BEHAVIORAL_TEST_DEBT entries (drive-to-zero ratchet must stay honest):\n  {}",
        stale.join("\n  ")
    );
}
