# ADR-0105 — Workspace file LOC cap raised 600 → 700

**Status:** Accepted (Enio, 2026-07-04) · **Supersedes:** none · **Amends:** the
`architecture_workspace_file_loc_cap` gate (`FILE_LOC_CAP`) + DIRETRIZ §5.1 · **Related:**
blindagem Fase 0.4 (which introduced the 600 cap), ADR-0075 (build-speed / decomposition discipline).

## Context

The workspace-wide file cap (`crates/ph2d-editor-core/tests/architecture_workspace_file_loc_cap.rs`)
was set to **600 LOC** at the blindagem Fase 0.4 baseline (2026-06-20), when `crates/` was full of
900–2400-line god-files and the goal was a ratchet **down**.

The cap is a **proxy for code health** — cohesion / single-responsibility, git-collision surface in
multi-agent work, review navigability. It is **not** a model-capability limit (the LLM reads 2000-line
files fine). But the **raw-line metric is crude for two file classes**:

1. **Large data structs + their `Default`** — e.g. `ph2d-tool-painter/src/tool/paint.rs`'s ~60-field
   `PaintState`. You cannot "split a struct definition" without fragmenting one responsibility across
   files; its `Default` body already lives in a sibling (`state_default.rs`) and the struct still sits at
   the wall. Adding **one** feature field (Deform's `deform: warp::DeformState`) overflowed 600.
2. **Large match / dispatch tables** — a single cohesive `match` that is legitimately long.

Two forces also weakened the original 600 rationale: the multi-agent git-collision argument is much
softer now that development is often **solo on one workstation** (one implementer + coordinator), and the
Fase 3 decomposition of the true god-files is a separate, ongoing effort that a slightly higher trigger
does not undo.

Trigger data at decision time (production `crates/*/src/**`, test/panel/widget/runtime-excluded):
**51 files ≥ 600 · 26 ≥ 700 · 13 ≥ 800 · 1 ≥ 1000.**

## Decision

**Raise `FILE_LOC_CAP` from 600 to 700.**

- 700 removes the artificial friction on cohesive files that naturally sit at 600–700 (big structs,
  enums, dispatch tables) while **still flagging the 26 genuine god-files** (≥ 700) as split candidates —
  the ratchet keeps biting. (800 would neuter it: only 13 would remain flagged.)
- The `FILE_OVERAGE_OK` allowlist is **pruned** of every entry now ≤ 700 (20 entries removed), keeping the
  `overage_allowlist_has_no_stale_entries` guard honest. Entries > 700 stay FROZEN (may shrink, never
  grow); driving them down remains blindagem Fase 3.2.
- **Unchanged:** the sibling caps stay where they are — panel cap (`architecture_panel_loc_cap`),
  widget cap 500 (`architecture_widget_loc_cap`), runtime cap 650 (`architecture_runtime_loc_cap`).
  Those own their own surfaces and were not part of this decision.

## Consequences

- A NEW file is born under **700**, not 600. Reviewers still bounce a file that grows toward 700 without
  a cohesion reason — the number is a ceiling, not a target; the principle (split by responsibility)
  is unchanged.
- DIRETRIZ §5.1 (the gate table row `arquivo crates/*/src/** > 600 LOC`) and CLAUDE.md references to
  the 600 file-cap are updated to 700. The "600" that appears for the **panel** cap is a different gate
  and stays.
- Reversible: a single `const` + the pruned allowlist. If 700 proves too loose, lower it and re-freeze.
