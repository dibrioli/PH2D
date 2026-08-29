---
name: feedback_a_smooth_fixture_cannot_tell_two_smoothers_apart_the_pointed_one_can
description: Duas leis de alisamento davam a MESMA linha em três peças lisas e leis opostas na peça com ponta — a fixtura sem o fenómeno não escolhe
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-28T22:04:13.432Z
---

Medido 2026-08-28 (quad remesh, acabamento da saída): o Laplaciano tangencial e o ajuste de
quadrado davam **a mesma linha em toda coluna** nas três peças lisas do corpus (`quad 2N ≡
lap N`). Na peça com **ponta** (`sculpt_hooked`) separaram-se por completo: à mesma mediana
de enviesamento (`3,0°`), o Laplaciano entregou `4` dobras, `5` faces péssimas e `3,95 %` de
perda de ponta; o ajuste de quadrado entregou `0`, `0` e `1,40 %`.

**Why:** o Laplaciano manda o vértice para o **centróide dos vizinhos** — numa ponta todos os
vizinhos estão para trás, logo a ponta é cortada e a reprojecção aterra do lado errado do
vinco. O ajuste de quadrado pede que a **face** seja um quadrado, o que uma face na ponta pode
ser sem sair da ponta. Numa peça lisa os dois pontos fixos coincidem, e por isso três fixturas
concordantes não escolheram nada.

**How to apply:** ao comparar duas leis, pergunte **qual fixtura contém o fenómeno em que elas
divergem** antes de concluir empate — e quando N fixturas concordam, isso é evidência de que
falta a fixtura, não de que as leis são a mesma. Relacionado:
[[feedback_a_sweep_whose_cells_all_agree_has_not_chosen_anything]] ·
[[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]] ·
[[feedback_where_new_objects_are_born_is_the_fixture_your_gates_are_missing]]
