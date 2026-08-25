---
name: feedback-reading-the-rule-is-not-the-same-as-being-on-the-right-side-of-it
description: Pus código dentro de uma guarda cujo parágrafo imediatamente acima proibia exactamente isso — ao acrescentar num arquivo, confira de que lado das guardas vizinhas a coisa nova pertence
metadata:
  type: feedback
---

A moldura do laço de seleção 3D ficou **dentro** da guarda `if let Some(anchor) = smoke.gizmo` (a
guarda de **seleção**) ⇒ sem nada escolhido ela não pintava, e o laço mais comum de todos é o
primeiro. ⚠️ **O parágrafo imediatamente acima já escrevia a lei**, sobre o gizmo de navegação:
*"ele é pintado SEMPRE, e não dentro da guarda de seleção que vem a seguir"*.

**Why:** ao acrescentar numa função longa, a escolha do sítio é feita por proximidade do código
parecido (*"o gizmo também é overlay"*), não por qual **pergunta** a guarda responde. As guardas de
um `paint` são leis, e cada uma tem um doc-comment a dizer de quem ela é.

**How to apply:** antes de colar um bloco novo, leia a **guarda** em que ele vai cair e pergunte se a
coisa nova depende do que ela testa. Aqui: *a moldura depende de haver seleção?* — não, ela diz o que
a **mão** está a fazer. E gateie a **pintura**, não só o gesto: as três perguntas são
pintado / populado / clicado ([[reference-topic-ui-seam-discipline]]), e esta wave tinha respondido
só às duas últimas.
