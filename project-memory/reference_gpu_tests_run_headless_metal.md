---
name: reference-gpu-tests-run-headless-metal
description: Headless GPU compute tests RUN on Metal in this sandbox — validate shaders/kernels directly; only interactive pen-input needs the GUI
metadata: 
  node_type: memory
  type: reference
  originSessionId: da867ef3-9b65-4b2c-b452-604f23cca0f9
---

Os testes GPU `#[ignore]` (que pedem `GpuContext::new`) **rodam de verdade** neste ambiente (Mac, Metal via wgpu) — confirmado 2026-06-13 com `cargo test -p ph2d-painter-wash --features gpu --test wash_invariants -- --ignored --nocapture` (10/10 passaram, incl. composite/transporte/parity num device real).

**Lição:** separe duas coisas que parecem "headless impossível":
- **Lógica de GPU** (shaders WGSL, kernels compute, parity Rust↔GPU, composite, undo de campo via read/upload buffer) → **valida headless aqui**. `cargo check` NÃO pega erro de WGSL (naga compila em runtime); o teste GPU pega. Use-os como alavanca — é a forma de não codar no escuro.
- **Comportamento interativo de pen-input** (undo/redo no app, pintura ao vivo, toggle de UI) → aí sim precisa do Enio na GUI; sem harness e2e de ponteiro.

O agente do wash BLOQUEADO (`HANDOFF_wash_undo_color_BLOCKED.md`) disse "não conseguiu reproduzir interativamente / sem GUI headless" e tratou TUDO como não-validável — mas a metade de GPU-lógica (cor, parity, composite) era testável o tempo todo. Ligado a [[feedback_tool_unit_green_integration_dead]] (unit-verde ≠ produto-vivo) — mas a recíproca também vale: nem todo bug visual é "só o Enio pega"; muito se prova num teste GPU headless antes de mandar pro olho dele.
