---
name: feedback_a_loop_that_republishes_the_whole_object_every_round_pays_for_what_it_never_reads
description: "Um laço que chama a porta «recomputa tudo o que é derivado» a cada ronda paga octree, adjacência e curvatura que nunca lê — corra sobre buffers e publique uma vez"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-29T00:08:52.612Z
---

Medido 2026-08-28: o acabamento do quad remesh custava `11,5 s` de `17,7 s` porque cada uma
das `726` rondas chamava `Mesh::rebuild()` — a porta que recomputa **tudo** o que é derivado:
normais de face, **adjacência**, normais de vértice, **curvatura** e **octree**. Uma relaxação
**não muda a topologia** e não lê nenhuma das três últimas (a projecção consulta a octree da
*superfície*, não a da malha que está a ser relaxada). Fora do laço: `1,5 s`, **saída
idêntica**.

⚠️ **A porta única não foi violada, e essa é a parte que interessa.** A cerca dela é declarada
(*«uma wave que reconstrói só metade disto deixa o sistema estável e errado»*) — reconstruir
*metade* seria o defeito. ⭐ A cura respeita-a: o laço corre sobre **buffers** e o objecto é
**publicado uma vez**, no fim, com o `rebuild` inteiro.

**Why:** uma porta «recomputa tudo» é a coisa certa entre operações e a coisa errada **dentro**
de um laço iterativo, porque ali a maior parte do derivado é invariante. E o custo não aparece
em régua nenhuma de saída — só no relógio.

**How to apply:** num laço que altera **posições** (relaxação, alisamento, otimização) mas não
a **estrutura**, pergunte o que do derivado é invariante, hoiste-o, corra sobre buffers e
publique no fim. ⭐ E se paralelizar: some sempre na **mesma ordem** (por consumidor, lendo a
incidência ordenada) — a forma óbvia, por produtor a acumular, muda o resultado com o número
de threads; gateie isso correndo a mesma peça com `1` e com `8` e comparando **bit a bit**.
Relacionado: [[feedback_a_test_that_got_slow_is_a_cost_measurement_nobody_asked_for]] ·
[[feedback_documented_decision_chesterton_fence]] ·
[[feedback_a_cost_only_defect_is_invisible_to_every_output_gate]]
