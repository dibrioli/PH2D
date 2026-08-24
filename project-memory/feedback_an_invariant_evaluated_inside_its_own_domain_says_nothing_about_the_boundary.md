---
name: feedback-an-invariant-evaluated-inside-its-own-domain-says-nothing-about-the-boundary
description: Um gate que corta as amostras pelas fronteiras que ele testa é cego às fronteiras — falta-lhe a metade da COBERTURA
metadata:
  type: feedback
---

Gate: *"toda amostra cai dentro da região que construiu a fita que a avalia"*. Ele iterava as fatias
e, para cada uma, media o raio **cortado por aquela fatia**. ⇒ duas mutações que **apagavam a
primeira e a última fronteira** sobreviveram: o pedaço de raio que ficava de fora simplesmente não
era medido por fatia nenhuma.

**Why:** um invariante tem duas metades — *o que está dentro está certo* (**containment**) e *não
sobra nada fora* (**cobertura**). Avaliar o primeiro dentro do domínio que ele define é uma
tautologia sobre a fronteira.

**How to apply:** sempre que um gate percorre uma partição, acrescente a asserção de que a partição
**cobre** o que o produto de facto percorre (`bounds[0] ≤ entrada && saída ≤ bounds[último]`). E se
três mutações da mesma família sobrevivem, suspeite de que o gate tem uma **cópia** da lei: aqui ele
reconstruía as fronteiras dentro do teste, e a cura foi uma porta só chamada pelos dois
([[feedback-a-hand-written-list-beside-a-predicate-is-two-answers]]).
