---
name: feedback_a_defect_can_hide_behind_another_defect_and_blind_the_very_gate_that_would_find_it
description: Um gate que faz `continue` sobre entrada invalida fica cego a tudo o que outro defeito invalida
metadata:
  type: feedback
---

Um gate que **salta** o caso que não consegue construir (`if let Err(_) { continue }`,
`let Ok(x) = … else { continue }`) fica cego a **toda** a região que outro defeito torna
inválida. Curar o primeiro defeito não é só curá-lo: é **descegar o instrumento**, e o que
aparece a seguir parece regressão e não é.

Caso medido (`line/3DModeling`, W126, 2026-09-06). A porta de escrita deixava a peça num
estado que o documento recusa (31 pares *forma × linha*, ~20 formas). O gate da marcha,
`every_row_of_every_primitive_marches_safely_across_its_range`, varre cada linha do painel e
faz `continue` sobre um documento inválido — logo **nunca** mediu a `tag` na largura em que ela
fura (`1,0232` contra uma barra de `1,02`). No commit em que a porta passou a repor as
invariantes, o gate reprovou pela primeira vez sobre um defeito que existia desde que a forma
existe.

**Why:** o `continue` é a decisão certa para a corrida (não se mede o que o produto recusa), e
é exactamente por isso que ele é silencioso: nada distingue *«não há nada aqui»* de *«não
consegui chegar aqui»*. A cobertura do gate passa a depender de um defeito noutro subsistema.

**How to apply:**
1. Ao curar um defeito que produzia **entrada inválida**, espere reprovações novas nos gates a
   jusante e **classifique-as antes de as chamar regressão**: meça a mesma coisa no commit
   anterior por outro caminho (aqui, construindo a peça à mão em vez de pela porta).
2. Num gate que salta casos, **conte os saltos** ao lado das medições e imprima os dois — um
   `medidas >= N` sozinho não distingue *saltei 40* de *saltei 0*.
3. Quando o defeito revelado não tem cura barata, declare-o **com a tabela** e escreva no mesmo
   commit o censo de obsolescência ([[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]]).

Relacionado: [[feedback_a_probe_in_the_failure_branch_cannot_see_the_other_sides_successes]] ·
[[feedback_a_swallowed_panic_silently_shrinks_the_candidate_set]] ·
[[reference_topic_gate_discipline]] · [[reference_topic_implicit_field_laws]]
