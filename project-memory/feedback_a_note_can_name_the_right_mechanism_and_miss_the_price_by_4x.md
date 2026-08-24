---
name: feedback-a-note-can-name-the-right-mechanism-and-miss-the-price-by-4x
description: Uma nota de wave pode acertar o MECANISMO e errar a ordem de grandeza — meça as duas contas que puxam para lados opostos antes de a executar
metadata:
  type: feedback
---

A W56d fechou prometendo que fatiar em profundidade traria `5×`. O mecanismo estava **certo** (a
pegada da região é o que manda) e o preço estava errado por **4×**: medido, `1,17×`.

**Why:** a nota via uma só conta — repartir **divide** o custo de avaliar. A outra, que ela não
mediu, é que repartir **multiplica** o de montar, e a montagem era 96% **JIT** (`2 334 µs` de
`2 430` por ladrilho). Uma cai com a **média** sobre as fatias, a outra sobe com a **soma**.

**How to apply:** antes de executar uma nota de otimização, meça **as duas metades separadas** (aqui:
montar 18% do quadro, marchar 82%) e a curva de cada uma no parâmetro que se vai mexer. Uma sonda que
mede a metade errada mente com confiança — a 1.ª versão desta imprimiu `197%` do quadro, porque o
denominador corria em 32 núcleos e o numerador em série. *Uma régua tem de correr no mesmo regime do
que ela mede.* Irmã de [[feedback-a-correct-mechanism-can-prescribe-the-wrong-cure]].
