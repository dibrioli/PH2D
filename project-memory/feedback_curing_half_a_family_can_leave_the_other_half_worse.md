---
name: feedback-curing-half-a-family-can-leave-the-other-half-worse
description: "Duas leis com o mesmo defeito e um consumidor comum: curar UMA pode piorar o resultado, porque a outra estava a compensar o erro dela — meça o par antes de shipar a metade"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-02T23:56:37.481Z
---

Quando duas leis partilham um defeito **e** um consumidor, o erro de uma pode estar a compensar o da
outra. ⇒ curar só uma delas é uma mudança que a medição pode **reprovar**, e reprovar com razão.

Caso medido (PH2D, `line/3DModeling`, 2026-09-02): o **filete** e o **chanfro** supunham os dois que
as faces eram ortogonais. O filete foi curado (o arco passa a ser exacto em qualquer quina). A cura
do chanfro — derivada, construída e **gateada**, com o recuo a ficar exactamente o pedido nos seis
ângulos — foi **revertida**: o corte de hoje desce `1,61×` mais do que o número diz numa ponta de
estrela, e essa profundidade a mais estava a esconder a ponta que o filete-depois-do-chanfro não
sabe alisar. A/B: a estrela vai de `5,02 %` para **`15,10 %`** de vinco, e o pior giro de `45,8°`
para `84,9°`.

O bloqueio ficou **nomeado**: o filete depois do chanfro mistura **três** superfícies e a lei n-ária
supõe *todos* os pares ortogonais; generalizá-la pede a matriz de Gram inteira, e o recorte que
torna a lei de duas faces exacta não tem forma fechada em `N ≥ 3`.

**How to apply:**
- ⭐⭐ **Antes de shipar a cura de UMA lei, corra a medição do PAR** — a célula `(curado, curado)`
  pode ser inalcançável e a `(curado, cru)` pode ser pior que `(cru, cru)`
  ([[feedback_two_halves_of_a_cure_each_refused_alone_do_not_refute_the_cure]] é o caso simétrico:
  ali as duas metades sozinhas eram recusadas e juntas funcionavam).
- ⭐ **Uma recusa medida precisa de GATE**, e o gate afirma o **erro que shipa**, não a lei certa:
  assim o número não se perde e quem trocar o operador sabe em que direcção ele mudou. *Uma recusa
  sem gate é uma frase que a próxima pessoa não pode confirmar.*
- ⚠️ **Reconfira a medição que declarava a lei correcta**: a do chanfro dizia *«recuo pedido = recuo
  entregue»* e tinha sido feita no aro de um **cilindro**, que é ortogonal — a mesma armadilha do
  filete ([[feedback_a_corpus_sitting_at_a_knobs_neutral_point_does_not_test_that_knob]]).
