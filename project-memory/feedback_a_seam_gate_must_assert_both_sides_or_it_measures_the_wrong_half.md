---
name: a-seam-gate-must-assert-both-sides-or-it-measures-the-wrong-half
description: Gate que afirma que todo consumidor PERGUNTA não afirma que alguém RESPONDE — e fica verde com a resposta ausente
metadata:
  type: feedback
---

Numa costura com dois lados — *quem pergunta* e *quem responde* —, um gate escrito de um lado só
fica verde com o outro inteiro em falta.

**Why:** medido em 2026-08-30. `the_chrome_swallows_the_click_it_was_given` afirma, por consumidor
de canvas, que ele **chama** `pointer_over_chrome`. Todos chamavam. Nada afirmava que o chrome
**REGISTA** um rectângulo que faça essa função responder `true` — e duas barras novas pintavam
faixas opacas com só os botões registados: **86,9 %** e **70,6 %** da área pintada deixavam a
caneta passar para a arte por baixo. Sintoma para o artista: pintar *através* do chrome.

**How to apply:** ao escrever o gate de uma costura, escreva a pergunta **e** a resposta —
tipicamente uma varredura densa de pontos sobre a superfície pintada, a exigir `hit(x,y).is_some()`.
E generalize: *«todo consumidor chama X»* nunca é o mesmo que *«X responde o que o produto precisa»*.
Irmãos: [[feedback_the_three_ui_seam_questions_miss_the_fourth_the_sequence]] ·
[[feedback_paint_and_hit_test_must_project_through_one_door]].

## ⚠️ A mesma lei, na forma que morde um gate de LEI PURA

Uma lei pura (uma função sem I/O, testada com os argumentos passados à mão) é o formato de gate mais
fácil de escrever — e ele **não mede quem a alimenta**. Medido duas vezes em 2026-08-30, na mesma
linha:

- o gate do gizmo de navegação afirmava que colunas docadas não movem a peça, passando a área à
  mão. A mutação que devolvia **o viewport inteiro** ao produto **SOBREVIVEU**;
- o gate do chrome afirmava que cada consumidor **chama** `pointer_over_chrome`. Ninguém afirmava
  que o chrome **regista** um rect que a faça responder `true`.

**How to apply:** quando a lei é pura, o gate dela é metade — a outra é a decisão *«que argumento o
produto lhe dá?»*. Extraia essa decisão para uma função com nome (`area_for`, `intern_active_tool`)
e gateie-a também; três linhas inline num laço de render não são mensuráveis por nada.

