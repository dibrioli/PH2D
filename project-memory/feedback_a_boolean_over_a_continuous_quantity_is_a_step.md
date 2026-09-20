---
name: feedback-a-boolean-over-a-continuous-quantity-is-a-step
description: Escolher entre duas leis por um limiar não dá um salto — dá CHATTER, e o artista vê a forma piscar
metadata:
  type: feedback
---

A lei da pele escolhia entre *«deformar os pontos de controlo»* e *«refazer o contorno inteiro»*
perguntando **«o desvio passa da tolerância?»**. O dono reportou com duas fotos: *«em determinado
momento da deformação as alças sofrem uma mudança e o path muda repentinamente, como se o handle
mudasse de tipo»*. Medido numa dobra a passos de `0,01 rad`, não era um salto: a decisão **oscilava
entre quadros vizinhos** a partir de `1,44 rad`, cada oscilação valendo `0,038`–`0,050` numa peça de
espessura `1`.

**Why:** duas leis que respondem coisas diferentes, comutadas por um limiar, são **descontínuas na
fronteira** — e perto dela o ruído do gesto atravessa-a muitas vezes por segundo. Pior: se a decisão
é por CONTORNO e o remédio re-deriva tudo, uma oscisão troca a representação de segmentos que
estavam bem.

**How to apply:** não comute leis — faça a correcção ser **a mesma lei com magnitude contínua**, e
que ela tenda a **zero exacto** onde não há nada a corrigir. ⭐ A forma prática: ajuste a
**DIFERENÇA** (`verdade − aproximação`), não a grandeza. Onde a aproximação já está certa a
diferença é zero, o sistema é homogéneo e a saída é **byte-idêntica** — o que também mata o defeito
que obrigou o limiar a existir. Relacionado:
[[feedback-a-gate-written-over-a-number-fails-when-the-owner-changes-the-number]] ·
[[reference-topic-measurement-discipline]]
