---
name: feedback_projecting_to_the_nearest_surface_point_never_picks_an_apex
description: "Todo acabamento que pousa vértices na superfície EMBOTA pontas — o pé da perpendicular cai no flanco, nunca no bico"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1246816c-63cf-414b-842d-663a8baa86ca
  modified: 2026-09-04T22:19:03.961Z
---

Um passo de acabamento que projecta cada vértice no **ponto mais próximo** de uma superfície
não consegue escolher um ápice: o bico é um ponto de medida nula e a perpendicular cai sempre
no **flanco**. ⇒ cada ronda embota a ponta um pouco, e *quanto* depende de cercas do próprio
acabamento (no quad remesh, a cerca de viagem: a MESMA extracção acabada de duas maneiras deu
`gap 0,18` e `0,51` no mesmo bico). **É isso que faz uma ponta sair boa e a vizinha não.**

**Why:** o defeito lê-se como *«a cadeia amputa pontas»* e a acusação cai a montante (grade,
campo, densidade) — mas ali a grade pode estar dentro da barra. A grandeza que separa as duas
histórias é *o que falta é CÉLULA ou FASE?*: se a grade do bico já tem o tamanho pedido, o que
sobra é meia célula de desalinhamento, e isso cura-se com posição.

**How to apply:** meça a distância do ápice da entrada à SUPERFÍCIE da saída (ponto→face) ao
lado da grade do bico. Com grade dentro da barra e `gap` sub-célula, o remate legítimo é
encostar o vértice mais próximo no ápice — com três cercas: só **dentro** da barra de amputação
(acima dela falta célula, e mexer no vértice esconderia o defeito do selector), viagem máxima de
uma célula, e o **censo global** (faces péssimas, faces do avesso) não pode subir, uma ponta de
cada vez. ⚠️ E confira a recusa vizinha antes de a herdar: *«puxar o vértice mais avançado»*
tinha sido medida e refutada — com um deslocamento de **23 células** sobre uma grade grosseira,
que é outra pergunta ([[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]]).
