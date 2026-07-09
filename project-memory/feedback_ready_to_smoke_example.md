---
name: feedback-ready-to-smoke-example
description: Sempre deixe a feature nova PRONTA PRA SMOKE num grafo/documento default ou demo — nunca instrua o Enio a montar à mão
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 14afaada-70a5-49d0-a3c1-e84cd2bb2756
---

Ao entregar um nó/feature nova, **autore um exemplo que a exercita já no documento default** (ou um preset de 1-ação) que abre mostrando/rodando a coisa — não peça pro Enio montar à mão.

**Why:** o Enio valoriza smoke imediato e sem atrito; montar 20×20 + 5 nós manualmente é tedioso e vira barreira. Quando o gate M1 virou o documento default (grid→tint→falloff→stagger→oscillator→output, auto-play), ele respondeu "tudo perfeito! Mantenha esse padrão de já deixar o exemplo pronto para smoke" (2026-07-08).

**How to apply:** feature nova → autore o exemplo mínimo que a exercita no grafo default/demo (ou preset carregável) + o comando `cd <worktree> && cargo run -p ph2d-host-desktop` copiável ([[feedback_run_command_include_cd]]). Combine com o teste headless irrefutável ([[feedback_painter_inefficiency_4_causes]]): o teste prova a costura, o exemplo deixa o smoke instantâneo. Vale pro smoke 1× no fim ([[feedback_smoke_at_end]]).
