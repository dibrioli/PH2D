---
name: feedback-a-local-relaxation-swaps-the-species-of-a-defect-as-much-as-it-removes-it
description: "Um reparo geométrico local pode converter um defeito noutro da mesma família, e um censo que só conta uma espécie lê a conversão como cura"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1246816c-63cf-414b-842d-663a8baa86ca
  modified: 2026-09-03T22:43:10.474Z
---

Uma relaxação local (Laplaciano + reprojecção) sobre uma face defeituosa **troca-lhe a espécie**
tanto quanto a apaga. Medido em 2026-09-03 (`line/quadextract`): a ronda que desfaz uma *gravata*
(quad que se cruza a si próprio) deixa a **mesma** face a apontar contra a vizinhança — vira
*dobra*. O censo do reparo contava só gravatas, logo leu `1 → 0` e declarou cura; o artista
fotografou a dobra resultante como uma fenda escura.

**Why:** as duas são a mesma coisa — *a face está do avesso em relação à vizinhança* — e um censo
que conta uma delas transforma uma conversão numa vitória. Pior: o selector a jusante usava esse
mesmo censo como chave, então a conversão **mudava a escolha do produto**.

**How to apply:** antes de escrever a aceitação de um reparo, pergunte *«que outras formas tem
este defeito?»* e conte-as **todas** no mesmo número. Dois corolários medidos no mesmo dia:

- **Repare um GRUPO de cada vez.** Com um censo global por chamada, um grupo teimoso repõe a
  malha e apaga a cura de outro que já tinha cedido (custou uma regressão de `0/5` para `2/5`
  pontas amputadas).
- **Julgue o grupo, guarde o total.** O critério é *«as faces DESTE grupo deixaram de estar do
  avesso?»*, com a guarda *«o total não SOBE»* — «o total desce» faz um reparo esperar pelo outro.

Ver [[feedback-a-decision-log-that-omits-one-key-explains-every-choice-but-the-one-that-matters]]
e [[reference-topic-measurement-discipline]].
