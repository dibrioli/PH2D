---
name: feedback_a_mean_wall_over_a_loop_that_starts_cold_hides_one_slow_frame
description: "A parede MÉDIA de uma sonda que começa antes do regime esconde UM quadro lento — 76 ms em 250 quadros eram os 0,30 ms «nunca explicados» da parede − soma (doc 121 §9.15)"
metadata:
  type: feedback
---

Doc 121 §9.15 (04/10, line/motion-value): a sonda das estrelas tracejadas lia parede `1,58` contra a soma
dos passes `1,18`, e duas rodadas supuseram custo de CPU ou de submissão. Medido por quadro: o `span`
da placa iguala a soma, a CPU é `0,04` ms, e o 1.º quadro cronometrado custa `76` ms (ainda pixel a
pixel: a capacidade medida chega dois quadros depois). `76 ÷ 250 = 0,30` ms. Na MEDIANA a parede é
`span + 0,07`, igual no contínuo. Sem perfilador a sonda corre `40` quadros e a mesma média dava `3,2` ms.

**Why:** a média dilui um evento raro num número que parece um custo por quadro; a diferença entre duas
réguas (parede − soma) só se explica medindo a DISTRIBUIÇÃO, não supondo a fonte.

**How to apply:** toda sonda de relógio imprime a mediana e os primeiros quadros ao lado da média; uma
«parede − soma» inexplicada lê-se primeiro quadro a quadro. E o quadro lento era um defeito do PRODUTO
(cada cena nova pagava-o): a cura foi medir a capacidade no início (c2). Ver
[[reference_topic_measurement_discipline]].
