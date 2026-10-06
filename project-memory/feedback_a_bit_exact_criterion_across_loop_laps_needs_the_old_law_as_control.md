---
name: feedback-a-bit-exact-criterion-across-loop-laps-needs-the-old-law-as-control
description: "Critério «a 2.ª volta do Loop dá os bits da 1.ª» era impossível (o passo é playhead−sim_t em f32); a lei antiga também falhava — meça o A/A antes de escrever um critério ao bit"
metadata:
  type: feedback
---

06/10 (doc 121 §9.20): escrevi como kill-criterion, antes de medir, que a 2.ª volta de uma zona em `Loop` daria os
bits da 1.ª. Falhou — e a lei de antes também (`1,5e-2` no tique 150; o mundo novo `1,06e-3`): o `dt` é
`playhead − sim_t` em `f32`, e `3,6 + k/60` arredonda diferente de `k/60`. O que É ao bit (e tem gate): o recuo
seguido de Play, e o mundo NOVO no recomeço.

**Why:** um critério de igualdade sem controlo A/A (a lei de antes, a mesma medição) mede a aritmética do relógio.

**How to apply:** antes de escrever «ao bit», corra a mesma régua sobre o produto de HOJE; se ele falha, o critério
é uma barra medida com as duas leis ao lado. Família: [[reference-topic-measurement-discipline]].
