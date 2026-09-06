---
name: feedback_a_rounded_union_of_two_coplanar_faces_swells_past_that_face
description: Unir com filete duas peças cuja fronteira COINCIDE ao longo de uma face empurra a superfície para fora dela — a peça sai da própria caixa, e a cura é não haver união ali.
metadata:
  type: feedback
---

Um filete de união (`union_round`) mistura duas peças **na região onde as duas estão perto**. Quando
as fronteiras delas **coincidem ao longo de uma face inteira** — duas chapas que partilham a tampa,
dois braços de um «L» que partilham a base — a mistura age sobre essa face inteira e **empurra a
superfície para fora dela**, até `r·(√2 − 1)`.

**Why:** apanhado **duas vezes na mesma wave** (formas 3D, 2026-09-05). (1) Dar a cada peça de uma
seta a própria laje e unir as chapas fechadas pôs material a `0,1088` com a meia-extensão em
`0,1000`. (2) O «L» de uma seta dobrada, feito de dois rectângulos que partilham `y = −rise`, inchou
`0,0059` **por baixo** da própria caixa. Nos dois casos o efeito colateral é o mesmo: *a peça deixa
de caber no bordo que o resto do sistema usa como cerca*.

**How to apply:** componha o perfil em **2D** e aplique a laje **uma vez** no fim (a face partilhada
em Z desaparece); e quando duas peças 2D partilham uma aresta, troque a união por uma **subtracção**
— um «L» é um rectângulo menos um bloco, e aí não há união nenhuma. ⚠️ A subtracção tem o espelho do
mesmo defeito: o bloco removido tem de **passar de largo** pelos lados onde tocaria a fronteira do
outro, senão a intersecção arredondada **come** aquelas faces.

⚠️ **A régua que apanha isto mede o MÓDULO** (`|y|`), não o máximo: duas varreduras densas desta
linha guardavam o `y` máximo e o excesso estava no mínimo — ver
[[feedback_a_ruler_that_walks_from_the_origin_assumes_the_origin_is_inside]].
