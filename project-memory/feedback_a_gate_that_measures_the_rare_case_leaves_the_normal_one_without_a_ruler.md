---
name: feedback-a-gate-that-measures-the-rare-case-leaves-the-normal-one-without-a-ruler
description: O espelho 3D era um no-op EXACTO (`0.000000`) na forma — e ficou verde meses porque o gate media o caso raro (numa operação) em vez do gesto normal.
metadata:
  type: feedback
---

Report do Enio, 2026-09-04: *«Mirror não funcionou»*. Medido: o espelho aplicado a uma **forma** dá
`0.000000` de diferença de campo — na origem **e** com a forma movida. A causa é estrutural: uma
primitiva é construída em volta da origem local dela **por construção**, o plano do espelho era essa
origem, e a pose do nó é aplicada *depois* da pilha ⇒ o plano viaja com o objecto e **nenhum gesto
do produto o deslocava**.

O gate existente — `a_mirror_on_an_operation_folds_an_off_centre_child` — provava o espelho numa
**operação** com filho descentrado, e estava verde. Um artista chega primeiro à forma; grupos são
uma coisa que ele aprende depois (ou nunca).

**Why:** um gate escrito na altura em que a feature nasceu tende a medir o caso em que o autor a
viu funcionar, que é quase sempre o caso **construído**. O caso **normal** é o que ninguém escreve,
porque parece óbvio demais para precisar de régua.

**How to apply:** ao gatear um controlo, pergunte *qual é o objecto que existe primeiro numa cena
vazia?* e meça ali. E prefira a lei **derivada e exaustiva** à fixtura escolhida: aqui ela é
*«ou o modificador muda o campo ao nascer, ou ele oferece um número para arrastar»*, varrida sobre
`UnaryKind::ALL` — o espelho era o único que não fazia nem uma coisa nem outra. Ver também
[[feedback_a_missing_knob_cell_can_hide_a_defect_measure_before_pricing]] e
[[feedback_a_gate_that_copies_the_formula_goes_green_over_a_law_nobody_ships]].
