---
name: feedback-a-perturbation-that-breaks-a-greedy-needs-a-seed-not-a-smaller-perturbation
description: "Quando um termo novo faz um algoritmo guloso explodir, a cura é semear o ponto de partida — não diminuir o termo nem suavizá-lo"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: edbb014f-4ffb-40ff-bd89-2200158288ca
  modified: 2026-08-22T12:30:34.904Z
---

Um termo novo na energia (alinhamento à curvatura) fazia o arredondamento guloso do
MIQ explodir: 21 → 104 patches com o peso **mínimo**. Duas curas óbvias foram
construídas e falharam — **baixar o peso** (a explosão acontecia em todos) e
**suavizar o guia** (piorou: 20,4° → 24,5°). A cura foi **semear**: correr uma
passagem inteira SEM o termo só para produzir o ponto de partida. Resultado: 17
patches, menos irregulares que a própria base, zero dobras, e o relevo de 25,7° para
16,7°.

**Why:** o guloso congelava decisões irreversíveis sobre a **primeira** resolução, e
essa partia de `θ = 0` — um estado que não é solução de nada. O termo não era o
problema; o *estado inicial* era. Diminuir a perturbação não cura isso porque o
ponto de partida continua errado; suavizar o guia também não, porque a fragilidade
não está na qualidade do guia.

⚠️ **E a escada de pesos (continuação clássica) foi MEDIDA E REJEITADA** — custou 8×
o relógio e devolveu um resultado *pior* na régua que importava, porque o árbitro
entre passagens minimizava a energia total, onde o termo novo pesava pouco. *Um laço
que melhora o número que mede não melhora o número que importa, quando são dois
números.*

**How to apply:** ao ver um algoritmo guloso/irreversível partir com um termo novo,
pergunte primeiro *"sobre que estado ele toma a primeira decisão?"* antes de mexer no
termo. E se puser um laço de refinamento, confira que a régua que escolhe o vencedor
é a mesma que decide o produto — ver [[feedback-a-collapsed-field-does-not-go-neutral-it-takes-over]].
