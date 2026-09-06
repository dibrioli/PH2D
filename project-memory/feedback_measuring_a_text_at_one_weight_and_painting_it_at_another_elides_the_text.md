---
name: feedback_measuring_a_text_at_one_weight_and_painting_it_at_another_elides_the_text
description: prefix_width mede no peso NORMAL e paint_text_title_elided pinta em SEMI_BOLD — dar ao texto a largura que ele mediu faz o pintor cortá-lo ("0....", e ao afastar desaparece); a cura é a porta title_elided_width, ao lado do pintor
metadata:
  type: feedback
---

**Uma medição num peso e uma pintura noutro são duas perguntas diferentes.** Report do Enio,
2026-09-05 (duas fotos): os números dentro dos cartões do Motion liam-se `0....`, `Re...`, e ao
afastar o zoom **desapareciam** (`Rows  ...`).

**O mecanismo:** eu media com `TextSystem::prefix_width` (peso NORMAL) e pintava com
`paint_text_title_elided`, que pinta em **`FontWeight::SEMI_BOLD`** — mais largo. O pintor
decide numa linha (`prefix_width_weighted(text, size, weight) <= max_width`), e o texto **não
cabia na largura que ele próprio tinha medido**. A diferença cresce quanto menor é a fonte, por
isso o defeito piorava ao afastar.

**A cura é uma PORTA ao lado do pintor:** `ph2d_editor_core::text_elide::title_elided_width`,
no MESMO ficheiro, com o mesmo peso e meia unidade de folga (a elisão decide-se por comparação
de floats). ⇒ quem alinha à direita, reserva coluna ou pergunta *«cabe?»* chama isto, nunca o
`prefix_width` cru.

⚠️ **O gate tem DUAS metades**, e a segunda é a que importa: a primeira assere que a largura da
porta chega; a segunda prova que o peso normal **realmente corta** em algum caso — sem ela o
gate ficaria verde sobre uma porta que voltasse ao `prefix_width`
([[feedback_a_correlation_with_zero_counterexamples_may_describe_another_question]]).

⚠️ **E o módulo já tinha a lição para o CORTE** (`the_cut_follows_the_weight_it_will_be_painted_in`,
e um comentário longo a dizer *«o peso ATRAVESSA, não é escolhido por um `if`»*) — faltava-lhe
para a **LARGURA**. *Uma lei aprendida numa metade de um par não se aplica sozinha à outra.*

Relacionado: [[feedback_a_reserved_band_is_not_a_painted_band_and_geometry_gates_go_green_over_a_blank_screen]] ·
[[reference_topic_ui_seam_discipline]].
