---
name: feedback-ship-prep-no-fail-fast
description: "Ship-prep over a big multi-agent batch — enumerate all gate failures in one nextest --no-fail-fast pass instead of ship.sh's fail-fast loop"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 08f6a613-4a63-4a4e-8305-1b658212543e
---

Shipping an accumulated multi-agent batch (e.g. closing a wave: Painter W3 + Vector W2 together) surfaces a **cascade** of hidden gate failures that per-task `cargo check` masks — because implementers commit `--no-verify` in fast mode and only run `cargo check -p`. In one 2026-06-02 ship I hit, in sequence: tree-wide rustfmt drift (23 files vs pinned 1.95), `chrome-sync` mod/dispatch drift, typos in other crates (`applyable`, `tpos`), panel LOC-cap, magic-numeric, concrete-tool downcast allowlist, and a dhat no-alloc flake.

**Why:** `./scripts/ship.sh` runs nextest **fail-fast** — it stops at the FIRST failing test, so each ~10-min ship.sh cycle reveals exactly one gate. Discovering N gate failures one-at-a-time = N×10min.

**How to apply:** before the first ship.sh, run `cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast` (in a CoW slot) to enumerate **every** failing test/gate in a single pass, then fix them as a batch. ship.sh's non-test gates (fmt/clippy/machete/deny/audit/typos) already run all checks and print a summary, so the only fail-fast blind spot is nextest — `--no-fail-fast` closes it. Reinforces [[feedback_full_gate_periodically]] (run the full gate during the wave, not just at the end) and [[feedback_codificacao_rapida]] (cargo-check hides gates). Note: nextest `retries` are scoped (asset-cooker only) — a genuinely environmental flake (e.g. dhat heap-block measurement under concurrent load) needs its own narrow `package(...) and test(...)` retry override, never a global one.
