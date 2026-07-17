---
name: reference-topic-process-cadence
description: "Cadência de processo (commit/CI/smoke/fase) — o gist já mora em CLAUDE.md §2-§3"
metadata:
  node_type: memory
  type: reference
---

- [[feedback_ci_handling]] — forneça o link da run; não fique em polling
- [[feedback_ci_batching]] — acumule commits; push único no fim
- [[feedback_commit_cadence]] — não comite a cada fix; acumule em blocos
- [[feedback_smoke_at_end]] — 1× no fim de TODA a implementação
- [[feedback_refactor_workflow]] — commits locais; Enio testa antes de push
- [[feedback_phase_cascade_2026_05_19]] — cada fase fecha + handoff + spawna a próxima (Modo C)
- [[feedback_codificacao_rapida]] — `cargo check -p <crate>`, nunca `--workspace`
- [[feedback_precommit_arch_gates]] — arch-gate antes de commit estrutural
- [[feedback_full_gate_periodically]] — re-lock cook hash ao mudar serialização
- [[feedback_ship_prep_no_fail_fast]] — `nextest --no-fail-fast` enumera TODAS
