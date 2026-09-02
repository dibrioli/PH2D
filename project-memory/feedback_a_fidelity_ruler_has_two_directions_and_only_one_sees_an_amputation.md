---
name: feedback_a_fidelity_ruler_has_two_directions_and_only_one_sees_an_amputation
description: "Distância entre duas malhas tem duas direcções; a que todos medem (saída→entrada) dá ZERO numa ponta comida"
metadata:
  type: feedback
---

Medido 2026-08-30, a caçar uma amputação de `−20 %` a `−35 %` do alcance de um espinho:
a 1.ª medição foi **saída → entrada** (*«as faces novas estão fora do lugar?»*) e devolveu
`≤ 3,4 ×` a aresta de entrada em toda a parte — **limpo**. Verdade, e irrelevante.

⭐ **O que se perdeu não estava na saída; estava na ENTRADA, sem ninguém do outro lado.**
A direcção que acusa é **entrada → saída**: para cada vértice do original, a distância à
superfície nova. Na mesma peça: `6,01 %` da diagonal na casca exterior contra `0,095 %`
numa configuração boa — `63×`.

**Why:** uma amputação remove; ela não desloca. Tudo o que **fica** continua pousado no
original, então toda régua que parte da saída passa. A assimetria é da operação, não da
malha — vale para recorte, simplificação, remalhagem, booleana, qualquer coisa que possa
**deixar de entregar** uma parte.

**How to apply:** ao medir «a saída representa a entrada?», escreva as **duas** direcções e
diga qual responde à sua pergunta. ⭐ E parta por **casca** (ou por região): o número global
não se move — duas pontas comidas não mexem uma mediana de milhares, e foi a mesma cegueira
que o `edge_max` global e o `χ` já cobraram. ⚠️ A distância tem de ser ao **triângulo**, não
ao vértice mais próximo: amostrar por vértices sobre-estima em até meia aresta, que numa
grade grossa é da ordem do defeito que se quer medir (`0,280 %` contra `0,095 %` reais).
⛔ E `0` amostras é **NÃO MEDIDO**, nunca «perfeito». Relacionado:
[[feedback_a_closed_surface_can_contain_a_second_one_count_the_components]] ·
[[feedback_a_bucket_nobody_fills_reads_as_perfect]] ·
[[feedback_ask_what_number_the_opposite_answer_would_print]]
