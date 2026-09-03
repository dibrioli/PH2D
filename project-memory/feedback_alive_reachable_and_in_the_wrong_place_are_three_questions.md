---
name: feedback_alive_reachable_and_in_the_wrong_place_are_three_questions
description: VIVO, ALCANÇÁVEL e NO SÍTIO CERTO são três perguntas; este repo tem instrumento para as duas primeiras e nenhum para a terceira, e por isso um layout partido passa com a suíte verde.
metadata:
  type: feedback
---

Report do Enio com foto (2026-09-02, *«layout ruim»*): uma faixa de filtro nova nascia à largura
toda de um painel, por cima da coluna de catálogos, com o rótulo cortado por baixo de um botão.
**Os oito gates daquela fatia passavam**, e nenhum por descuido — eles perguntavam:

1. *«o id está no índice de toque?»* (`hit_indexed_ids_are_registered`) — **estava**.
2. *«o clique chega a um efeito?»* (os `seam_*`) — **chegava**.

As duas eram verdadeiras sobre a foto do defeito.

**Why:** a caça ao controlo morto deste repo cobre as duas primeiras perguntas com instrumentos
bons, e isso cria a ilusão de cobertura. Mas *estar vivo* e *estar no sítio certo* são
independentes: um botão perfeitamente ligado, desenhado por cima de outro, é um app partido do lado
do artista e um app verde do lado da suíte. **Nada neste repo mede se dois controlos do mesmo
painel se pisam.**

E o mecanismo do defeito é sempre o mesmo: uma medida tirada da caixa ERRADA. Aqui foram duas — a
largura tirada do painel inteiro quando o controlo pertence à grade (*a largura de um controlo é
uma afirmação sobre o que ele manda*), e uma linha de base escrita como se fosse uma margem
(`y + row_h − pad`, que põe o texto no bordo de baixo, em vez de
`y + (row_h − size)·0,5`, que é a centragem que o `paint_list_item` já usa).

**How to apply:** quando pintar um controlo NOVO ao lado de outros, publique o rectângulo dele
(`probe_*_rect`) e escreva um gate que o cruza com o que os vizinhos registaram — a mutação que
repõe a medida errada tem de sangrar. ⚠️ E a ORDEM de pintura é parte da lei: quem cede largura a
um vizinho tem de ser pintado **depois** dele, senão a largura que ele lê ainda não existe.
⛔ Um censo cego sobre todos os rects de um painel **não** serve: uma linha dentro de uma coluna
sobrepõe-se legitimamente à região dela. A lei geral precisa de *irmão na mesma faixa de layout*,
que este repo ainda não sabe exprimir.

Irmão de [[reference_topic_ui_seam_discipline]] e de
[[reference_topic_control_design_hazards]]; a família do gate que mede a coisa errada está em
[[reference_topic_gate_discipline]].
