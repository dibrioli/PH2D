---
name: a-term-with-a-unit-bearing-minimum-imposes-its-own-scale
description: Um termo de energia cujo mínimo é um valor CONCRETO (det J = 1) não é neutro — ele impõe a sua escala, e o repouso tem de estar nas unidades do alvo
metadata:
  type: feedback
---

Ao acrescentar um termo a uma energia, pergunte **onde ele é minimizado** e **em que unidades**.
Um termo como `g(J) = (det²J + 1)/det J` tem mínimo em **`det J = 1`** — um valor concreto, não
uma direcção. Ele **não é neutro em relação à escala**: ele *impõe* a escala `1` do referencial em
que foi escrito.

**Why:** medido em 2026-08-30 (`line/quadextract`, `ph2d_gridmap::injective_solve`). O referencial
de repouso saía do triângulo 3D **em unidades do mundo**, e o alvo da fase era *uma célula de grade
por `h`*, com `h ≈ 0,038`. ⇒ a barreira pedia `~1/h²` vezes a densidade pedida e **lutava contra a
fase inteira**. Sintoma no produto: `2,3×` os quads pedidos, enviesamento mediano `6,4° → 30,3°`,
`2 → 2 955` faces com canto pior que 60°.

⛔ E o mais caro: uma **varredura de orçamento inteira** (4 configurações, ~2 h de relógio) foi
gasta a medir o planalto do problema mal posto, e o `33` que ela devolveu foi lido como *«o limite
do método»* quando era **o limite de uma unidade errada**. Com o repouso dividido pelo passo, o
mesmo código dá **`0`** dobras em `5` de `64` iterações e **`13×`** menos relógio.

**How to apply:** antes de somar um termo, escreva o valor do argumento no mínimo dele e confira se
esse valor **é** o que a fase quer. Se o termo tem um mínimo dimensional, o repouso/referencial tem
de estar nas **unidades do alvo** — e o gate certo é a **invariância** (escalar o problema e o alvo
juntos tem de dar a mesma saída), nunca um número fixo, porque um número fixo é reescrito à mão a
cada mudança da energia. Ver [[an-exact-invariance-gate-needs-an-exact-transformation]] para a
armadilha aritmética de escrever esse gate, e [[the-ceiling-is-the-hardwares-never-the-fallbacks]]
para a família (§0.0: *um limite legítimo diz de que recurso ele é*).
