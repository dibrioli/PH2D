---
name: feedback-a-smoke-scene-opens-in-the-state-of-step-one
description: "Cena de smoke abre JÁ no estado do 1.º passo (objecto seleccionado, MODO certo, Play parado) — reutilizar uma cena antiga que ensina outra coisa foi reprovado"
metadata:
  node_type: memory
  type: feedback
  originSessionId: a0080127-b09b-4ff2-8804-aac5af83d76e
  modified: 2026-10-17T12:12:50.872Z
---

Uma cena de smoke tem de abrir no estado exacto do 1.º passo do dono: o objecto seleccionado, no MODO
em que ele vai agir (ex.: Paint), a ferramenta pronta, o clipe PARADO se o passo é «pinte e depois dê
Play». Reutilizar uma cena antiga cujo desenho é outro (a `PH2D_MACRO_SMOKE=1` toca sozinha, com a imagem
em Object) e mandar o dono «clicar em X, entrar em Y» foi reprovado (17/10, Macros W7: «a cena foi mal
montada, imagem no Modo object, play já começa sozinho»).

**Why:** o dono é o produto, não um engenheiro; preparar a cena é trabalho da linha, e uma cena que
contradiz o passo ensina o contrário ([[feedback-a-smoke-for-the-owner-explains-what-each-thing-on-screen-is]]).

**How to apply:** passo do smoke que precisa de outro estado ⇒ cena NOVA (o próximo número do roteador),
fotografada antes, e a foto conferida contra o passo (barra da ferramenta certa, Time 0, Play parado).
