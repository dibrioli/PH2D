---
name: feedback-a-fn-cap-and-a-file-cap-measure-different-things
description: Curar um gate de LOC de FUNÇÃO extraindo dentro do mesmo arquivo pode estourar o gate de ARQUIVO — os dois medem coisas diferentes e a cura empurra o problema de lado
metadata:
  type: feedback
---

Um gate de **LOC por função** (200) e um de **LOC por arquivo** (600) parecem a mesma família e
não são. Extrair um helper **dentro do mesmo arquivo** cura o primeiro e **piora** o segundo —
o corpo continua ali, mais o doc-comment que o helper novo merece.

Medido (integração 2026-08-04, `ph2d-panel-motion-graph`): `paint` em 201/200 ⇒ extraí
`draw_canvas_overlays` no próprio `paint.rs` ⇒ o gate de fn ficou verde e o de ARQUIVO nasceu
vermelho (596 → 613) **na mesma rodada da suíte**.

**Why:** só o arquivo IRMÃO move as duas grandezas na mesma direção. E é a convenção que esses
painéis já seguem (`paint_grid` · `paint_wire` · `paint_menu` · `paint_stamp`).

**How to apply:** ao cortar por causa de um cap de LOC, meça **as duas** grandezas antes e
depois — e prefira o irmão. ⚠️ O irmão novo herda as varreduras por-arquivo do repo (a11y,
`no_magic_numeric`, o gerador de `mod`), então o corte não termina no `mv`: ele termina quando
essas varreduras estão verdes. Ver [[reference_topic_gate_discipline]].
