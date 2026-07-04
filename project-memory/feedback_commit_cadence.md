---
name: feedback-commit-cadence
description: "Durante waves/refactors em curso, NÃO commitar a cada bug fix pequeno; acumular múltiplos fixes + próximas etapas e commitar em blocos lógicos maiores."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 3cd59062-48fc-4433-8496-0552be468b98
---

Durante waves/refactors multi-stage (Wave 5, Wave 6+7, etc.), **não commitar
a cada bug fix individual**. Acumular múltiplos fixes locais + próximas
etapas do plano e commitar em blocos lógicos maiores (1 commit por
sub-stage do plano, não 1 commit por mudança).

**Why:** Enio cansou de assinar pre-commit hook a cada 5 min durante
Wave 6+7 Sessão 1. T1 hook em ph2d-editor leva ~6min (compile + clippy +
nextest); commits pequenos = espera + interrupção desproporcional ao escopo
da mudança. Histórico granular não é prioridade; bisect compensa.

**How to apply:**
- Bug fix pequeno reportado pelo Enio durante smoke: aplicar local, NÃO
  commitar, mencionar na resposta que "fix está local, acumulado pra
  commit do próximo stage".
- Quando passar pro próximo stage do plano (Phase 1.C → 2 → 3.C…),
  commitar TUDO o que acumulou (fixes + entregável do stage anterior)
  em UM commit por stage.
- Exceção: commit standalone OK quando fix entrega um sub-stage do
  plano explícito (ex.: commit Phase 1.A entrega Phase 1.A, mesmo
  que tenha incluído um fix).
- Relacionado: [[feedback-ci-batching]] (push único no fim da wave).
