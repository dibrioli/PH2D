---
name: CI handling — divisão de trabalho
description: Quem polla CI no PH2D pós-v6.0 (Coord absorve PRCI). Implementador não polla. Push sempre acompanhado de link da run.
type: feedback
originSessionId: 3810fc76-ee39-499c-932e-822ab7813c1b
---

**Modelo atual (v6.0+, 2026-05-19):**

- **Implementador** não pusha nem polla CI. Reporta commit local pronto pro Enio relay.
- **Coordenador** (que absorve o papel antes chamado PRCI) faz `git push` no fim do ciclo e **polla CI ativamente** com intervalo de 15min (`Monitor` com `sleep 900` ou `gh run watch`).
- **Enio** não confere visualmente — Coord automatiza. Enio recebe reporte final ("CI 9/9 verde, sha bom = X" ou "falhou em job Y").

**Why:** Modelo anterior (pré-v6.0) tinha papel PRCI separado e Enio conferia visualmente. v6.0 colapsou 4 papéis → 2 e o Coord absorveu o trabalho de babysit. Reduz fricção operacional.

**How to apply (Coordenador):**
- Após `git push`, **forneça SEMPRE o link da run** no formato `https://github.com/dibrioli/PH2D/actions/runs/<run-id>` (use `gh run list --workflow=spike.yml --limit=1` para pegar o ID).
- Se um job falhar, fornecer também o `gh run view --job=<id>` ou link direto para o job que falhou.
- Polling com intervalo de **15min** (não 1min — desperdício de contexto LLM).
- Diagnostique falhas: `gh run view --log-failed | tail -80`. Aplique fix local, push, re-watch.
- Loop fecha em `success` OU 3 ciclos consecutivos de falha do mesmo job (aí escalona pro Enio).

**How to apply (Implementador):**
- **Nunca push nem `gh`.** Reporte commit local + sha. Coord assume daí.
- Se Coord pediu pra você diagnosticar falha de CI específica (caso raro de auditoria cross-cutting), aplica fix local na sua pasta e volta ao Coord.

**Quando Coord NÃO polla (delegação reversa):**
- Enio explicitamente diz "pode deixar a CI rodando, não babysit agora" (raro).
- Próxima ação não depende do CI verde (ex: trabalho local segue paralelo).

Vide DIRETRIZ §7 (Smoke + Push + CI) pro protocolo canônico.
