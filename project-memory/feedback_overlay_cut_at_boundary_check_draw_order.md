---
name: overlay-cut-at-boundary-check-draw-order
description: "Overlay \"cortado\"/invisível numa região = cheque a ORDEM de draw na mesma cena (z-order) antes de caçar clamp/clip na geometria"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2763a9af-144e-488a-b803-b06687b3c3ed
---

Overlay vetorial que "para" exatamente numa fronteira (borda de sprite, edge de painel) enquanto a
geometria está comprovadamente correta: o suspeito nº 1 é **outro draw NA MESMA cena, chamado DEPOIS,
pintando por cima** — não clamp, não clip, não gate de input. Caso real (2026-07-11, watercolor
tiling): o overlay editável de forma "parava na borda da sprite" com Tiling+Repeat Image; 6 commits
tentaram teorias de geometria/cópias 3×3, e a causa era `draw_repeat_image` (8 blits full-canvas
opacos) rodando depois de `draw_overlays` na mesma `VectorScene` (`painter_bridge.rs`). Fix = 1
reordenação + gate de fonte (`repeat_image_tiles_draw_under_the_editing_chrome`).

**Why:** em cena imediata (Vello), ordem de chamada = z-order; um blit opaco posterior é
indistinguível de "não desenhou" na foto. A geometria pode estar 100% certa e o pixel sumir mesmo
assim — o diagnóstico por leitura da geometria nunca converge.

**How to apply:** sintoma "some/corta numa fronteira retangular" → primeiro liste TODOS os writers da
cena no dispatch e a ordem deles (grep pelo `vector_scene` no bridge); só depois cace clamp. A
fronteira do corte geralmente coincide com a bbox do draw que cobre. Fixou ordem = escreva gate de
ordem no fonte (padrão `shells/desktop/tests/`). Relacionado: [[feedback_visual_bug_debug]],
[[feedback_gizmo_verify_hit_target_before_transform_math]].
