---
name: a-constrained-fit-buys-continuity-by-killing-the-feature
description: Prender cada grau de liberdade à direcção que ele já tinha compra a continuidade e mata a capacidade — e o gate que morre a seguir é o da feature, não o da lei.
metadata:
  type: feedback
---

**Prender um grau de liberdade à direcção que ele já tinha dá a propriedade que se quer e apaga a
capacidade que estava lá.** É a primeira cura que ocorre, e ela mede-se bem na régua da propriedade
e mal em todas as outras.

Medido (PH2D, 2026-09-19): a correcção das alças de uma Bézier presa à direcção de cada alça dá
quebra de tangente `0,000°` — e numa **aresta recta** as duas pontas ficam sobre a mesma linha, logo
o segmento **não consegue arquear**. ⇒ *pintar peso no meio de uma aresta volta a mover
`0,000000`*, que é a feature inteira a morrer. O desvio à verdade também piora (`0,03049` contra
`0,02451` da cura certa).

⭐⭐ **Quem avisou foi um gate da FEATURE, não um da lei** — `pintar_peso_entre_dois_nos_move_a_arte`
e o gate de fidelidade reprovaram enquanto o gate da continuidade ficava verde. *Quando uma cura
acende a régua nova e apaga duas antigas, a cura é a errada.*

⇒ a saída é **acoplar em vez de prender**: o que é partilhado entre duas peças roda JUNTO, e cada
peça mantém a liberdade que tinha ([[a-per-piece-fit-breaks-the-property-that-lives-on-the-seam]]).
