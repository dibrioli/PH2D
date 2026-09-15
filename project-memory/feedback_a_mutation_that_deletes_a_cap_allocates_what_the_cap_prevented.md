---
name: feedback-a-mutation-that-deletes-a-cap-allocates-what-the-cap-prevented
description: A prova de mutação de um TECTO fez o teste alocar 27 GB e teve de ser morta à mão — a fixtura de um tecto põe-se LOGO ACIMA dele, nunca em `u32::MAX`.
metadata:
  type: feedback
---

Medido em 2026-09-14 (`line/components`, a W1 da fábrica). O gate do `BURST_MAX = 1024` pedia uma
rajada acima do tecto e a fixtura dizia `burst: u32::MAX`. A mutação que apaga o
`.min(BURST_MAX)` — exactamente a que tem de sangrar — fez o teste construir `4 × 10⁹` pedidos: o
binário chegou a **27 GB de RSS** e foi morto à mão antes de o `OOMPolicy` derrubar a janela.

**Why:** um tecto existe porque o recurso atrás dele é real. A mutação que o apaga **liberta o
recurso**, e uma fixtura calibrada no infinito passa a pedir infinito. O teste continua correcto: é
a fixtura que deixa de ser sobre o tecto e passa a ser sobre a máquina.

**How to apply:** a fixtura de um tecto pede **`TECTO + ε`** (aqui `BURST_MAX + 7`) — o mutante
sangra na mesma (`1031 ≠ 1024`) e nada rebenta. ⚠️ Vale para todo cap com um alocador atrás:
contagens de spawn, tamanhos de buffer, profundidades de recursão, iterações de solver.
⛔ E `u32::MAX` numa fixtura é sempre suspeito: ele não é «muito», é *o maior que o tipo tem*.

Irmãs: [[reference_topic_mutation_proofs]] · [[project_vscode_dies_by_oompolicy_not_by_choice]] ·
[[feedback_a_mutation_that_survives_may_mean_a_missing_gate]]
