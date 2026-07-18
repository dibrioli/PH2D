---
name: reference-topic-impasto-physics
description: Física do impasto/sculpt — 10 lições de operador/unidade/âncora/amostragem/canal
metadata: 
  node_type: memory
  type: reference
  originSessionId: b294ecd6-99c8-41cf-ac4b-c6001c30b1c7
---

- [[feedback_an_absolute_unit_that_should_feel_relative_must_scale_with_the_geometry]] — altura ∝ raio: preserve a razão de aspecto
- [[feedback_a_hard_clamp_is_not_a_ceiling_it_is_an_eraser]] — colapsa o detalhe; a luz lê DERIVADA
- [[feedback_growing_geometry_without_growing_matter_grows_nothing]] — o render multiplica por cobertura: meça ELA
- [[feedback_a_lateral_effect_needs_a_nonlocal_operator]] — fórmula pontual vira CONSTANTE no dado real
- [[feedback_a_2_5d_analog_of_a_3d_operator_needs_the_lateral_recovered]] — morfológico; parábola separável salva a perf
- [[feedback_an_effect_anchors_at_the_material_edge_not_the_gizmo]] — o aro nasce na borda do CORPO (W_TAIL), não no círculo do gizmo
- [[feedback_a_sequential_accumulation_is_sampling_dependent]] — produto sobre incrementos = fase do dab; telescope pela SOBRA
- [[feedback_a_write_once_channel_has_no_repair_verb]] — o Inflate escreve `covers` e ninguém edita ⇒ a borda é imune
- [[feedback_fill_concave_keep_convex_is_a_morphological_closing]] — dilatação é isotrópica (cresce convexo E côncavo); pra encher só a axila e deixar a borda convexa, use um CLOSING (dilata+erode com a mesma bola). O enclausuramento angular (soma de vetores unitários) NÃO separa flank-grosso de axila (ambos span > 180°) — fiddly = bug de design; o closing não tem threshold, `ρ` é o único parâmetro (Inflate do Painter, 2026-07-18)
- [[feedback_derive_the_field_from_the_smooth_value_not_the_argmax_position]] — o VALOR de um max de funções contínuas é sem-costura; a POSIÇÃO do argmax salta nas células de Voronoi. A cobertura do Inflate lida da posição do argmax virava raios radiais num blob redondo; lida do campo de altura `hbuf` (o valor) é lisa
