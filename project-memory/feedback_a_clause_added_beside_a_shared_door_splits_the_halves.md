---
name: feedback_a_clause_added_beside_a_shared_door_splits_the_halves
description: "Condição acrescentada AO LADO de uma porta partilhada (paint && x) parte as duas metades — o gate que só confere que ambas CITAM a porta fica verde; a faixa invisível das réguas comeu 75 % do Select"
metadata:
  type: feedback
---

MiroClone (07/10, report do dono: *«o Select é inseleccionável; toda a barra tem problemas»*). Em 05/10
a linha pôs «sem quadro activo» no PAINT das réguas (`rulers_live() && … && documents.active().is_none()`)
e não na porta `HeroScreen::rulers_live`, que o gesto das guias da SHELL também pergunta. Com um quadro
aberto a faixa de 20 px e as guias ficaram INVISÍVEIS mas VIVAS, e o gesto corre antes do clique de
chrome: cobriam 75 % do Select e metade de cada botão da barra (medido). O gate
`the_paint_and_the_gesture_ask_the_same_door_about_the_rulers` estava verde porque só confere que as
duas metades CITAM `rulers_live`.

**Why:** «visível ⇔ vivo» só se sustenta se a condição INTEIRA mora na porta; uma cláusula ao lado dela
numa metade é uma segunda porta com o mesmo nome.

**How to apply:** condição nova para «X aparece» vai DENTRO da porta que o gesto também lê, nunca no
`&&` de quem pinta. E o teste de UI que clica no CENTRO de um botão não vê uma zona que cobre a borda
dele: conferir cantos e centro contra as zonas geométricas que correm antes do chrome. Ver
[[feedback_a_ruler_that_stops_at_world_space_approves_a_broken_click]].
