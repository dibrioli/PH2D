---
name: feedback-the-owners-fixture-is-the-one-that-must-be-measured
description: A régua de um report corre na cena do DONO; uma fixtura de laboratório pode ter um mecanismo verdadeiro e diferente, e curá-lo não toca no produto
metadata:
  type: feedback
---

⛔⛔ **Uma fixtura de laboratório pode conter um defeito REAL que o produto não tem — e a decomposição
dela aponta o dedo com toda a confiança ao mecanismo errado.**

Medido 2026-09-17 (`docs/Render3d/08` §13). O report era sobre o **vaso** da cena `=5`. A régua correu
primeiro na **caixa de Cornell**, e a decomposição por direcção foi limpíssima: UMA direcção de `48`
fazia `64,8` de um salto total de `65,6`, a bater a `6 cm` de uma lâmpada pontual, com `1/r²` a ler
`385` contra `1,3`–`5,6` de todas as outras. *Um polo, provado com números.*

⚠️ **E aquela lâmpada não é a do produto.** A do modelador nasce a `2 × half_extent` do alvo, **fora
da peça** (`ph2d_app_field3d::lights::opening_distance`, com a razão escrita lá) — no vaso a
superfície mais próxima dela está a `0,99` ⇒ `1/r²` ≤ `~1` e **não há polo nenhum**. A causa na peça
do dono era outra, e maior: o **lóbulo especular** no ponto acertado
([[feedback-a-near-delta-in-a-fixed-sample-set-stops-the-estimator-converging]]).

⇒ **antes de curar, corra a régua na cena do report, com as ENTRADAS do produto lidas pela porta que
o produto usa** — nunca um literal, nunca a fixtura que já estava montada. É mais barato do que a
cura errada, e é a única coisa que separa *«o mecanismo»* de *«um mecanismo»*.

⚠️ Esta casa já pagou a mesma forma cinco vezes antes (a fixtura da paridade sem o fenómeno; a
fixtura do pincel de pose numa esfera lisa; a peça recentrada em Python em vez de pela porta do
importador). **A assinatura é sempre a mesma: a medição é boa e o sujeito dela é outro programa.**
