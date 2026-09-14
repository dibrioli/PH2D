---
name: a-cache-that-does-the-expensive-work-before-checking-is-not-a-cache
description: 456 µs por quadro para não fazer nada — o O(V) da chave corria antes da comparação dela; a cura é reconstruir a chave com o resultado ANTERIOR
metadata:
  type: feedback
---

Medido em 2026-09-14 (`line/sculpt3d`, o indicador do pincel de pose). A cache
tinha a chave certa, o gate *«sobrevoar parado não constrói nada»* estava **verde
e a dizer a verdade** — e um quadro de sobrevoo **parado** custava `456 µs` numa
malha de `24 386` vértices (`2,7 %` de um quadro), mais uma alocação da malha
inteira por quadro.

A causa: um campo da chave — o vértice sob o cursor — é `O(V)` a achar, e ele era
calculado **antes** da comparação. *A cache perguntava «preciso de reconstruir?»
depois de já ter feito a parte cara da reconstrução.*

⭐ **A cura não é tirar o campo da chave** (ele é o que detecta a malha a mudar
por baixo de um cursor parado): é **reconstruir a chave com o resultado
ANTERIOR** — se o vértice eleito do quadro passado continua onde estava e o resto
bate, o eleito de hoje é o mesmo. `456 µs → 7,4 µs`, e a comparação passa a ser
`O(1)`.

**Why:** um contador de reconstruções mede *quantas vezes o trabalho foi
refeito*, e é cego ao trabalho que se faz **para decidir** que ele não é preciso.
Nenhum gate de cache deste repo mede essa metade — só uma **sonda de relógio no
caminho do produto** a revela.

**How to apply:** ao escrever uma cache, pergunte o custo do caminho **frio E do
quente**, e meça o quente com um relógio (não com um contador). Se algum campo da
chave for `O(n)`, derive-o do estado guardado em vez de o recalcular, e compare a
**struct inteira** — uma lista de `&&` escrita à mão é onde um campo novo é
esquecido no dia em que alguém o acrescenta. Irmã de
[[a-sampled-maximum-that-becomes-a-safety-bound-errs-only-downwards]] na forma:
*o instrumento que valida a decisão não é o instrumento que mede o preço dela*.
