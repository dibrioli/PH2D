---
name: feedback_a_quality_bar_copied_from_another_doc_loses_the_density_it_was_measured_at
description: Barra de qualidade citada de outro doc não carrega as condições da medição — conte as faces dos DOIS lados antes de dizer «estamos fora»
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-28T22:04:00.514Z
---

Uma barra de qualidade copiada para um roteador (`«a barra do oráculo: enviesamento p50
4,8–7,1°»`) **não carrega a condição em que foi medida**. Em 2026-08-28 mediu-se: a nossa
saída tinha `370`–`576` quads e a dele `3 352`–`4 696` — **9×**. À densidade dele, a mesma
cadeia sem uma linha mudada deu `3,8°`–`6,5°`, **dentro da barra**. Uma semana de trabalho
(as amarras dos arcos) foi paga a perseguir um buraco que era da régua.

**Why:** numa malha mais fina cada face cobre menos curvatura, logo desvia-se menos de 90° —
a grandeza é monótona na densidade. Duas medições da mesma grandeza em densidades diferentes
não são comparáveis, e a citação da barra tinha perdido esse contexto dezenas de vezes.

**How to apply:** antes de dizer «estamos acima/abaixo da barra», **imprima a contagem de
faces dos dois lados na mesma linha**. Se elas diferirem mais que ~20%, re-corra o nosso lado
no alvo que iguala a contagem — é mais barato que qualquer hipótese. E quando o alvo grava
mais de uma saída (o oráculo grava a crua **e** a `_smooth`), diga **qual** delas é a barra:
a diferença entre as duas era exactamente o que faltava. Relacionado:
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]] ·
[[feedback_an_unlabelled_probe_column_gets_read_backwards]] ·
[[reference_topic_quad_remesh_rulers]]
