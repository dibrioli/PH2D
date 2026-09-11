//! Wave 11 §2.3 (ADR-0042 §6 #6) — every `### N.M ...` heading in
//! `docs/UI_Bugs/README.md` + `docs/Image Tools Bugs/README.md` must
//! declare a `**Gate:** <reference>` line documenting either (a) the
//! arch test that prevents recurrence, OR (b) `gate-deferred: <reason>`
//! when no recurring class is identifiable.
//!
//! Recurrence is the failure mode this prevents: the same bug class
//! has resurfaced ≥ 3× across waves (DIRETRIZ §4.1). Forcing every
//! entry to declare its gate makes the absence of a gate an explicit
//! decision instead of an oversight.
//!
//! Heading format expected:
//!
//! ```text
//! ### 1.2 Some title
//!
//! **Gate:** crates/ph2d-editor-core/tests/arch_no_X.rs::test_name
//! ```
//!
//! OR:
//!
//! ```text
//! ### 9.7 Some title
//!
//! **Gate:** gate-deferred: documented incident; no recurring class
//! ```
//!
//! The `**Gate:**` line must appear within the next 6 lines after the
//! heading (allowing for an optional blank line between heading and
//! the gate marker, plus a couple lines of context if the author put
//! the Sintoma block before the Gate annotation).

use std::fs;
use std::path::PathBuf;

const SEARCH_WINDOW: usize = 6;

#[test]
fn every_bug_entry_declares_a_gate() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let docs = [
        workspace_root.join("docs/UI_Bugs/README.md"),
        workspace_root.join("docs/Image Tools Bugs/README.md"),
    ];

    let mut violations: Vec<String> = Vec::new();
    for doc in &docs {
        let Ok(src) = fs::read_to_string(doc) else {
            continue;
        };
        let lines: Vec<&str> = src.lines().collect();
        // Per-file heading convention:
        // - `UI_Bugs/README.md` — bug entries are `### N.M Title`.
        //   `## N.` lines are CHAPTER containers with sub-entries; skip.
        // - `Image Tools Bugs/README.md` — bug entries are `## N. Title`
        //   (flat numbering; no `### N.M` children).
        let doc_name = doc.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let is_image_tools = doc
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            == Some("Image Tools Bugs");
        let _ = doc_name; // reserved for future per-file rules
        let entry_prefix = if is_image_tools { "## " } else { "### " };

        for (i, line) in lines.iter().enumerate() {
            let Some(body) = line.strip_prefix(entry_prefix) else {
                continue;
            };
            // Skip the next-deeper heading level (e.g. `### Sintoma`
            // inside an Image Tools `## 1.` entry).
            if line.starts_with(&format!("{entry_prefix}#")) {
                continue;
            }
            if !body.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                continue;
            }
            // Look ahead up to SEARCH_WINDOW lines for a `**Gate:**` marker.
            let end = (i + 1 + SEARCH_WINDOW).min(lines.len());
            let has_gate = lines[i + 1..end].iter().any(|l| l.contains("**Gate:**"));
            if !has_gate {
                let rel = doc
                    .strip_prefix(&workspace_root)
                    .unwrap_or(doc)
                    .to_string_lossy()
                    .into_owned();
                violations.push(format!("{}:{} — {}", rel, i + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Wave 11 §2.3 — {} bug heading(s) missing the required `**Gate:**` \
         declaration:\n  {}\n\n\
         Append a line under each heading: either \
         `**Gate:** crates/.../tests/X.rs::test_name` pointing at the \
         arch test that prevents recurrence, OR \
         `**Gate:** gate-deferred: <reason>` when no class-level gate \
         applies. The marker must appear within the next {} lines after \
         the `### ` heading.",
        violations.len(),
        violations.join("\n  "),
        SEARCH_WINDOW,
    );
}
