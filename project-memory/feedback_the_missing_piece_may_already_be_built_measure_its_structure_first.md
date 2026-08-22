---
name: feedback_the_missing_piece_may_already_be_built_measure_its_structure_first
description: Antes de construir a peça que falta, MEÇA a estrutura do que já está lá — três tentativas de "corte" falharam porque o corte já existia e ninguém o percorria
metadata:
  type: feedback
---

Faltava cortar uma asa numa decomposição de superfície, e eu estava a orçar um
algoritmo (tree-cotree, caminho mais curto entre fronteiras). Antes disso medi a
**estrutura** das paredes que o traçado já tinha deixado por dentro dos patches:

⇒ nas **três** fixturas, sem excepção, era **um caminho entre duas junções, dentro do
patch anel** — ou seja, **a ponte que abre o anel em disco já estava traçada**. O
passeio da fronteira é que se recusava a percorrê-la, porque exigia que a face do
outro lado fosse de *outro* patch.

O conserto foi uma condição a menos, não um algoritmo a mais.

**Why:** eu tinha feito **três** tentativas antes (honrar em todo patch · honrar só no
patch doente · tratar a ponta como canto), todas revertidas — e as três falharam por
eu estar a raciocinar sobre *que forma a peça deveria ter*, sem ter medido *que forma
ela tem*. A medição que resolveu custou uma sonda de 40 linhas.

**How to apply:** quando um passo pede "construir X que falta", gaste primeiro uma
sonda a **classificar o que já existe** naquele sítio — quantos, de que forma, ligados
a quê, dentro de quem. ⚠️ Em particular: *material que o sistema produz e depois
ignora* é o primeiro sítio a olhar, porque ele não aparece em nenhuma régua e por isso
ninguém o conta. Irmã de
[[feedback_before_building_an_open_list_item_measure_whether_composition_already_expresses_it]]
e de [[feedback_a_repair_loop_can_hide_the_defect_it_worsens]].
