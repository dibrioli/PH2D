---
name: feedback-a-click-is-a-press-that-drifted
description: Um clique humano SEMPRE desliza 1px — o dispatcher chama isso de arrasto; teste com Down/Up na mesma coordenada é verde e a feature é inusável
metadata:
  type: feedback
---

O menu de nós do Motion "não inseria nenhum nó ao clicar" (Enio, 2026-07-13). Causa: o
dispatcher classifica press-release **com qualquer movimento** como `End` (arrasto), não
`Click`. **Mão humana sempre mexe um pixel.** O menu tratava `End` como dispensa → fechava sem
escolher nada.

**Why:** todos os testes do menu mandavam **Down e Up na MESMA coordenada** — a única coisa que
uma mão de verdade nunca faz. Os gates ficavam verdes e a feature era inusável. É o irmão
pointer-side de [[feedback_painted_is_not_populated_paint_gate]]: o teste assumia a resposta da
metade que quebra.

**How to apply:**
- Todo gate de clique em UI manda **Down → Move(+1px) → Up**. Se ele só passa sem o Move, ele
  não testa um clique — testa um robô.
- Regra de menu/popup: enquanto está aberto, **o ponteiro pertence a ele**. Onde o botão SOBE é
  o que vale (sobre a linha → escolhe; fora → dispensa). De brinde sai o aperta-desliza-solta
  que todo menu de SO tem.
- Um campo de texto que toma foco tem que **devolvê-lo** quando o dono fecha — senão todo
  atalho do app passa a digitar num buffer invisível. Assente o foco em **um** lugar, nunca em
  cada caminho de fechamento.
- Teste que empurra o gesto na mão pula o dispatcher. Pinte o painel, leia o hit index que a
  **pintura** registrou, e despache `PointerEvent` de verdade (`MockPanelHost::paint_with_layout`
  + `dispatch_pointer_event`).
