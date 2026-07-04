---
name: project-painter-t19-latent-red-macos-2026-05-28
description: "The 4 painter t19 tests were latent-red on macOS since W1 closure, not a T2.1 regression"
metadata: 
  node_type: memory
  type: project
  originSessionId: 7ed61631-ac7c-432d-95de-f4a4cc75ee49
---

Painter W2 TASK 0: the 4 failing tests in
`crates/ph2d-tool-painter/tests/history_integration_t19.rs`
(`current_samples_len_tracks_pushed_samples`, `u7_tilt_unavailable_flag_set_for_zero_tilt`,
`detach_journal_cancels_active_stroke`, `deactivate_cancels_active_stroke_in_wal`)
were **NOT a T2.1 regression**. Verified by building/running at 1485471 (W1 closure)
in an isolated worktree — they failed **identically** there. `ph2d-painter-stroke` and
`ph2d-painter-brush` src are byte-unchanged 1485471..HEAD; the only post-W1 painter
changes (T2.1 commits 28b4a27/4d71324/c82293c) never touch sample/cancel logic. So the
W2-impl handoff's "regression in the T2.1 window" framing was wrong, and the
[[project_painter_w1_complete_2026_05_28]] "W1 green" claim was CI/linux-only — these were
latent-red on macOS.

Two real, latent bugs (fixed in `f13f9ea`):
1. **Code:** `queue_pointer` early-returned on `stamps.is_empty()` before recording the
   sample to `current_samples`+WAL — so sub-spacing / stationary pressure-only samples were
   dropped from the replay source-of-truth (W12 Reproject / W14 Inspector / W13 MCP). Fix:
   stamp emission gates the CANVAS paint only; every finite sample is recorded.
2. **Test:** the two cancel tests asserted `recovered_strokes.len()==0`, impossible under the
   append-only WAL (begin_stroke fsyncs Begin eagerly per ADR-0052; cancel writes a Cancel
   terminator → recovery classifies `Cancelled`, kept in the list). Corrected to the real
   guarantee: `in_progress_at_crash().count()==0` + state `Cancelled` (mirrors recovery.rs's
   own `recovery_detects_cancelled`).

**Why:** prevents the next session re-bisecting a non-existent regression window.
**How to apply:** when a painter test is "red and was supposedly green at W1", build at the
claimed-green commit FIRST (isolated worktree + `CARGO_TARGET_DIR`) before hunting a window;
suspect macOS-vs-CI latent divergence (HR-5 cross-OS) over a recent regression.
