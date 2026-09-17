---
name: feedback_a_declared_blind_spot_can_be_the_majority
description: "Uma régua que DECLARA o que não vê tem de medir o TAMANHO disso — o censo HR-15 do texto pintado declarava não ver construtores/tabelas/format!, e eles eram 56 % do Painter e 95 % da editor-core (e uma cura foi \"provada\" com ela)"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e990a2f5-7d16-405a-8ddf-54393edf203d
  modified: 2026-09-13T21:35:10.781Z
---

Medido 2026-09-13 (`line/UIUX`): o censo do texto pintado (ponto fixo de pintores `paint_*`/`draw_*`
com parâmetro `&str`) declarava por escrito não ver construtores de widget (`Button::new(id, "Apply")`),
tabelas (`["Paint", "Erase"]`) e `format!`. A régua LEXICAL (todo literal com cara de língua fora de
teste e do que nunca pinta, crate `ph2d-label-census`) contou: Painter `166 → 376`, editor-core
`32 → 641`, crates de UI `445 → 5 035`. O handoff de 10/09 tinha fechado a editor-core como «língua de
produto 0» com a régua cega, e o gate molde afirmava-o.

**Why:** declarar um ponto cego lê-se como honestidade e é recebido como «pequeno»; ninguém mede o que
a própria régua diz não ver, e uma cura medida por ela passa por completa.

**How to apply:** ao escrever ou herdar um censo com «o que ele NÃO vê», corra UMA vez uma régua mais
larga (mesmo grosseira, lexical) sobre a mesma população e escreva a razão ao lado. E uma régua nova
lê-se contra um caso que se CONHECE (a lexical nasceu a perder todo texto com escape `\u{…}` — apanhado
por uma frase da Hierarquia que eu sabia existir). Ligado: [[reference_topic_measurement_discipline]],
[[feedback_a_tail_is_a_window_not_a_verdict]].
