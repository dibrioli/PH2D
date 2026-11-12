---
name: feedback_a_label_that_does_not_fit_gets_an_ellipsis_never_wraps
description: "Dono 06/09 e 10/11 — rótulo que não cabe = UMA linha com «…», antes do ponto de animação; nunca quebra nem pisa no ponto. «É regra», todo o app"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 557dac86-526f-4eee-8809-693c61a21457
  modified: 2026-11-11T01:56:02.536Z
---

Rótulo de campo que não cabe ao estreitar o painel fica numa linha só com reticências («Stroke wi…»), e
pára ANTES da marca de animação (o ponto) da sua célula. Nunca quebra em duas linhas, nunca passa por cima
do ponto. Dono 10/11, com fotos: *«Label embolando com pontos. a real é ... na label ao estreitar o painel;
pivot from centre também ... é regra»*. A mesma ordem de 06/09 (*«em vez dos três pontos (…) como no
Blender, a palavra passa para baixo e some»*).

**Why:** os cartões tinham uma lei de maquete («Opacity» / «(%)» em duas linhas, maquete F) que contrariava
a regra de 06/09; e um rótulo de grupo que atravessa a coluna vizinha (campo + caixa «Dashed») pintava por
cima do ponto da própria célula.

**How to apply:** a faixa dos rótulos dos cartões (`ph2d-editor-core` `panel/cards_rotulos.rs`) elide numa
linha, e a largura do rótulo pára na primeira célula do troço com marca. Testes
`a_label_that_does_not_fit_stays_on_one_line` e `a_label_stops_before_the_key_mark_of_its_cell` (GEF02,
10/11). Widget novo com rótulo: elide pelo `paint_text_elided`, nunca quebra. Ao estreitar, fotografe com
`FOTO_LAYOUT` (Inspector no piso de 210 px). Ver [[feedback_app_ui_english_only]].
