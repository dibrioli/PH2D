---
name: feedback-an-absolute-ceiling-picked-by-eye-approves-what-the-relative-control-sees
description: "Tecto ABSOLUTO escolhido à vista (3 degraus) aprovava um defeito real que o CONTROLO relativo da mesma régua via 28× — os dois juntos, nunca um só"
metadata:
  type: feedback
---

**O caso (03/10, W6 da `line/sculpt3d`):** o gate *«uma amostra de ARESTA da malha lê o mesmo que o
meio de uma face»* (o desfoque na superfície da esfera de fábrica) nasceu com um tecto absoluto
escolhido a olhar para o 1.º número: `pior ≤ 3` degraus de sRGB8 — e passava. O controlo da própria
régua (o mesmo par entre duas amostras do MEIO, longe uma da outra no mundo) deu média `0,007` contra
`0,195` na aresta: **a aresta era `28×` pior**, e a causa era real — o estêncil de eixos nas células de
quad, inconsistente numa célula trapezoidal (a esfera UV é feita delas). Com a triangulação conforme:
pior `1`, igual ao meio.

**Why:** um tecto absoluto mede «é pequeno?»; uma costura pequena e SISTEMÁTICA é pequena. Só a
comparação com a população que não tem a propriedade (o meio de face) diz se a aresta é diferente. E o
inverso também vale (a regra da casa): um gate SÓ relacional sobrevive a uma mutação de espaço.

**How to apply:** um gate de «X lê o mesmo que Y» leva os dois: o valor EXACTO no espaço do tipo
(aqui `pior ≤ 1` degrau) E o controlo dentro do mesmo teste (a mesma estatística em Y contra Y, com o
par forçado LONGE no mundo — o vizinho lê igual por estar perto, não pela simetria afirmada). Antes de
fixar o tecto absoluto, corra o controlo: se ele está muito abaixo do número que ia aceitar, o número é
um defeito à espera. Ver [[reference_topic_measurement_discipline]] ·
[[feedback_a_grid_never_lands_on_a_measure_zero_set_so_it_reports_it_clean]].
