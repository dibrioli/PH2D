---
name: feedback-precommit-arch-gates
description: "Run editor-core arch/HR test gates locally before committing STRUCTURAL changes, to avoid the ~5min pre-commit-hook abort cycle"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f0521d79-636c-4f46-a1fa-5c161bcf6d2e
---

The pre-commit hook's real cost is the **~3min workspace compile** (clippy --workspace + nextest build), NOT the tests (~15s). A deterministic arch-gate failure surfaces only AFTER that compile, so each missed gate = a wasted ~5min cycle + re-stage/re-commit churn (hit this 3× in one session: widget showcase-coverage, HR-12 a11y, cook-hash determinism).

**Why:** structural changes trip fast file-scanning arch/HR tests; I ran only the feature's own tests and forgot the gates.

**How to apply** — before `git commit` on a structural change, run the matching gate (seconds with a warm build):
- new/removed file under `src/widget/` → `cargo test -p ph2d-editor-core --tests` (covers, in one shot: architecture_widget_showcase_coverage, hr12_widgets_a11y, architecture_widget_loc_cap, no_literal_color, no_magic_numeric, hr15_no_hardcoded_ui_strings — each has a `*_OPT_OUT` list a chrome-internal painter must be added to)
- new serialized component field (derive Serialize) → `cargo test -p ph2d-asset-cooker` (prefab_cook_hash_is_locked); often the fix is `#[serde(skip)]` for a runtime-only field, not bumping the fixture
- new `readback_individual` / texture path → the chokepoint arch gates

This is NOT duplicating the slow hook matrix ([[feedback_codificacao_rapida]]) — it front-runs only the cheap deterministic gates. Also: always run `git commit` with `run_in_background: true` — the hook outlasts the 2-min foreground Bash timeout (a foreground commit gets killed mid-hook).

**CI clippy ≠ local hook clippy (2026-05-20).** The pre-commit hook runs `cargo clippy --workspace -- -D warnings` (NO `--all-targets`) — it does NOT lint TEST code. CI's `lint` job runs the EXACT command (spike.yml:43): `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings`. The `--all-targets` gap caught `needless_range_loop` in `premul.rs` TESTS after a clean local hook + push. Prevention: before pushing, run that EXACT command locally (the key missing flag is `--all-targets`, which lints `#[cfg(test)]` modules). DO NOT use `--all-features` to "verify" — it enables the flecs spike path (`c11_flecs`) that CI never lints, producing false-positive errors. CI lint job name: `lint (fmt + clippy + deny + audit)` in `spike.yml`.
