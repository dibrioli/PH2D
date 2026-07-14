---
name: feedback_measure_perf_symptom_scale
description: "For a perf/latency symptom, establish the MAGNITUDE (ms/scale) before hypothesizing the cause"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: b267a3a6-da8c-4949-84d3-c4f44c8764b5
---

For a perf/latency complaint, **establish the magnitude (ms / scale) before hypothesizing the cause** — the order of magnitude rules whole classes of cause in or out.

**Why:** Debugging the painter "delay entre o clique e o início do traço" (2026-06-08), I assumed render-frame latency (4–16ms) and burned many rounds on frame-scale fixes (pipelined readback, sync-vs-async, priming, upload-order, a 7-agent audit) — even reverting good work. Then Enio said "**não é um frame, deve ser ~⅓ de segundo**." 333ms is ~100× a frame → it's O(grid) CPU work, not a frame. The real cause: `begin_stroke` regenerated the paper-tooth (`grain_noise` per cell, O(grid)) **every stroke** on a large/4K canvas (commit `0cd7802`: cache it). One-line clue, instantly correct scale.

**How to apply:**
- Ask or measure the number FIRST: "how many ms?" "every stroke or once?" "scales with canvas size?" A symptom's scale (frame ≈ ms vs allocation/compute ≈ 100s of ms vs sync-stall) points at the cause class before you read code.
- Instrument the magnitude cheaply (an `Instant` + `eprintln`, env-gated) instead of reasoning from static code — see [[feedback_visual_bug_debug]].
- A **multi-agent audit finds the cause but inherits your framing** — if the prompt assumes "frame latency," the agents chase frames. Put the measured magnitude in the audit prompt.
- `bench-green ≠ live`: a tight-loop bench measured the composite readback transfer at 0.03ms, but live the per-frame `device.poll(wait)` drained the whole GPU queue (~2.6ms stall, 250→140 FPS). The cost was the SYNC, not the transfer. Profile in the real app (`PH2D_FLUID_PROFILE`), cf. [[feedback_tool_unit_green_integration_dead]].
- **A config-gated optimization can show ZERO live improvement if the user isn't on that config** (painter brush-texture, 2026-06-22): I built a Blender-style brush-stamp cache that only engaged for the **View** mapping, measured a real win, but Enio saw "não se observa melhorias!" — because he was painting with **Tiled** (canvas-fixed), which bypassed the cache entirely. I'd measured my *assumed* scenario (2048², Anchored, View), not his. Lesson: before declaring a perf fix, confirm the user's ACTUAL repro config (which mapping / size / layer-count), not just the symptom scale — an optimization gated on a branch the user never takes is invisible. Fix was a second cache for the canvas-fixed mappings (`blit_canvas_cached`: compute each canvas pixel's texture once per stroke; Tiled voronoi ~14ms→4.6ms/move). Also: a GPU mip-chain regen I *expected* to dominate measured only ~1.8ms — measure the suspect, don't assume it's big.

## E a barra de perf NÃO pode morar na suíte do CI (2026-07-13)

Pus uma barra de **wall-clock** num `measure_*` (o preview do ADR-0120). Passou em `--release` e
**nasceu vermelha no `nextest`**: o perfil `ci-test` compila o workspace em **`opt-level = 1`**, e
aí o DSP roda ~30× mais lento enquanto a **memcpy** (intrinsic da libc) não muda um grama. A barra
tinha virado uma medida do **perfil**, não do código.

**How to apply:** num teste que roda no CI, asserte só o que é **robusto a perfil** — uma **razão**
entre dois caminhos que sofrem a mesma compilação ("o incremental faz estritamente menos trabalho
que o completo"), nunca um número de ms. O número absoluto é **printout**, não barra — exatamente
como `the_rack_fingerprint` recusa golden pinado ([[feedback_wide_mechanical_refactor_use_a_fingerprint]]).
