---
name: feedback_a_chamfer_fills_a_concave_edge_so_asking_if_the_point_went_outside_is_the_wrong_question
description: Uma régua que pergunta «o chanfro pôs este ponto de FORA?» só vale para arestas convexas — numa côncava ele acrescenta material e o ponto fica enterrado, lido como aresta esquecida.
metadata:
  type: feedback
---

Um chanfro **tira** material numa aresta **convexa** e **acrescenta** numa **côncava**. Uma régua que
mede *«o chanfro alcançou esta aresta?»* perguntando **«o campo é POSITIVO no ponto de vinco?»**
responde certo só para a primeira espécie: na segunda o ponto fica **enterrado** (`f < 0`) e lê-se
como *«esta aresta não foi cortada»*.

**Why:** medido em 2026-09-05 (catálogo de formas 3D, `the_chamfer_reaches_every_edge_of_every_shape`).
O **chevron** é a primeira forma cuja aresta dominante é côncava — o entalhe do «V» corre a peça
inteira. Ele lia `82,3 %` com os **94 pontos por cortar todos no mesmo sítio**, o vértice interior. As
formas anteriores têm vincos côncavos pequenos (as quatro quinas de uma cruz) e o erro cabia na
folga da barra.

**How to apply:** meça o **módulo** — o chanfro tem de mover a superfície **para longe** do ponto, e
o sinal é da geometria, não do defeito. ⛔ Isto **não afrouxa** o gate: uma aresta genuinamente
esquecida deixa o ponto **exactamente sobre** a superfície (`f ≈ 0`), e `|f| > ε` continua a
reprovar. Na troca, três formas que já shipavam **melhoraram** (`98,8 → 100`, `97,3 → 100`) — o sinal
de que a régua media a coisa errada e não de que ela ficou permissiva.
