---
name: topic-authored-state-and-clocks
description: "Família: estado autorado, relógios, âncoras e ciclo de vida de snapshot/preview/load"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 85e38f84-1b86-49d2-aee2-91da101e1fd7
  modified: 2026-07-21T01:04:49.505Z
---

# Estado autorado, relógios e âncoras (índice de família — detalhe em cada arquivo)

- [[feedback_one_ruler_measures_one_clock]] — uma régua mede UM relógio; bases diferentes no mesmo eixo = bug de modelo
- [[feedback_an_impossible_inverse_is_a_reason_for_a_second_clock_not_a_readonly_control]] — inverso impossível justifica 2º relógio, não read-only
- [[feedback_a_gap_is_not_silence_two_answers_across_one_pixel]] — "ausente" e "influência 0" têm de coincidir no limite
- [[feedback_mirroring_time_must_mirror_the_shape]] — espelhar tempo espelha a FORMA; o interp muda de dono E se espelha
- [[feedback_a_view_publisher_must_not_require_a_primed_cache]] — quem PUBLICA view prima sozinho
- [[feedback_derived_coordinate_seed_must_match_sample]] — coordenada derivada: seed = sample (mesma transform)
- [[feedback_anchor_must_be_invariant_under_user_transforms]] — ancore em geometria, não em aparência
- [[feedback_an_anchor_is_not_a_feature]] — âncora é parametrização; casar formas por âncora força rotação
- [[feedback_works_then_silently_forgets_recook_wipes_authored_state]] — recook varre o autorado dentro do derivado
- [[feedback_a_restored_snapshot_resurrects_its_id_counter]] — snapshot restaurado ressuscita o contador de ids; preview = GESTO
- [[feedback_what_survives_a_load_is_adopted_not_stale]] — o que sobrevive a um load é ADOTADO
- [[feedback_a_sentinel_needs_a_gate_on_its_reader]] — sentinel exige gate no LEITOR (`from_bits(0)` panica)
- [[feedback_a_snapshot_must_be_a_fixed_point_of_the_systems]] — snapshot = ponto fixo dos sistemas
- [[feedback_capture_stroke_session_before_pen_up]] — capture a sessão ANTES do pen-up
- [[feedback_gizmo_verify_hit_target_before_transform_math]] — gizmo errado: logue o HIT antes da math
- [[feedback_overlay_cut_at_boundary_check_draw_order]] — overlay cortado = ORDEM de draw
- [[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]] — unidades mistas: converta com a const do CONSUMIDOR
- [[feedback_an_escape_that_never_helps_is_a_design_bug]] — escape que nunca ajuda é enfeite
- [[feedback_a_boolean_leaves_slivers_and_a_zero_area_piece_paints_a_line]] — peça sem área pinta LINHA; oráculo = densidade
