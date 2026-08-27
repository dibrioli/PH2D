---
name: feedback-a-ruler-placed-after-the-tidying-step-measures-the-tidying
description: Uma sonda colocada depois do passo que ARRUMA le' a arrumacao, nao o defeito — e devolve zeros que parecem uma boa noticia.
metadata:
  type: feedback
---

Ao instrumentar um pipeline que **termina a arrumar** (arredondar, saturar, normalizar,
ordenar), a sonda tem de correr **antes** desse passo. Colocada depois, ela mede a
arrumação e devolve `0` em tudo — que se lê como *«não há defeito»*.

**Why:** medido no `ph2d-gridmap` (2026-08-27). A sonda `arc_cycle_integrality` ficou
depois do laço que força as translações a inteiro, e imprimiu *«`0` com valor
fraccionário, `0` termos de entrada fraccionários»* ao lado de uma régua a acusar `0,3675`
de distância a inteiro. Movida para antes, a mesma sonda nomeia o defeito (`2` donos
obsoletos). ⚠️ *A contradição entre as duas colunas foi o único sinal — se a outra régua
não existisse, o `0` teria fechado a investigação.*

**How to apply:** pergunte *«o que corre entre esta sonda e o fim?»*. Se houver um passo
que satura, arredonda ou reordena, mova a sonda para cima dele — ou meça **as duas**
posições e imprima a diferença, que é o que a arrumação está a esconder. E quando duas
colunas do mesmo relatório discordarem, suspeite do **tempo** antes de suspeitar da lógica.
Parente de [[feedback-a-bucket-nobody-fills-reads-as-perfect]] e
[[feedback-counting-the-work-done-is-not-counting-the-work-delivered]].
