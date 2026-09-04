---
name: feedback-a-decision-log-that-omits-one-key-explains-every-choice-but-the-one-that-matters
description: Um log de escolha que imprime n−1 das n chaves do critério parece completo e é inútil justamente no caso que interessa
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1246816c-63cf-414b-842d-663a8baa86ca
  modified: 2026-09-03T18:47:28.710Z
---

Um selector com `n` chaves ordinais precisa de um registo com **as `n` colunas**. Com `n−1` ele
explica todas as escolhas menos aquela em que as outras empatam — que é sempre a que se está a
investigar.

Medido em 2026-09-03 (`line/quadextract`): o `worse` da cascata de retopologia decide por
`furos → ilhas → gravatas → amputação → …`, e o log da candidata imprimia **`bordo`** (uma
grandeza VIZINHA da chave: a chave é `bordo + não-manifold`), **nunca** as ilhas e **nunca** as
gravatas. Uma candidata com `0` pontas amputadas e grade `0,81` era deitada fora por outra com
`3` amputadas — e o log não tinha coluna que o dissesse. Custou **três** corridas de ~10 min
(uma por coluna acrescentada) para achar o que uma linha teria dito à primeira: a diferença
estava numa **única** gravata.

**Why:** o log de uma decisão não é diagnóstico geral — é a prova da decisão. Uma coluna que
mede *quase* a chave (`bordo` contra `bordo+nm`) é pior que nenhuma: ela dá a sensação de estar
a ler o critério.

**How to apply:** ao escrever ou depurar um selector, confira que **cada** chave do comparador
tem coluna no registo, com a **mesma função** que o comparador chama (não uma irmã). Quando uma
escolha surpreender, a primeira pergunta é *«todas as chaves estão impressas?»* — antes de
qualquer hipótese sobre o algoritmo. Ver [[feedback-a-tool-is-adopted-only-when-a-written-step-names-it]]
e [[reference-topic-measurement-discipline]].
