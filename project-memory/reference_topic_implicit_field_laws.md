---
name: reference-topic-implicit-field-laws
description: "Família: as leis do CAMPO IMPLÍCITO (SDF) e as réguas de FORMA — o que uma mistura arredondada faz à peça, onde uma aresta fica viva, e por que um gate sobre a forma pode estar a medir outra peça"
metadata:
  type: reference
---

⚠️ **Índice de família, 2 saltos.** Estas linhas viviam soltas no `MEMORY.md` e empurraram-no acima
do limite de leitura — o que faz as últimas **desaparecerem em silêncio**. Cada uma continua a ser
um ficheiro próprio; o que mudou foi o endereço. Mesmo desenho de
[[reference_topic_measurement_discipline]].

*Perguntas-mãe: **o que esta mistura faz à peça fora da aresta que eu queria?** e **o número que
li vale em todo o curso do controlo, ou só onde a forma nasce?***

## A geometria — o que a operação faz por fora

- [União com filete de duas faces COPLANARES incha PARA FORA delas (até `r·(√2−1)`) — componha o perfil em 2D e aplique a laje uma vez, ou troque a união por uma SUBTRACÇÃO](feedback_a_rounded_union_of_two_coplanar_faces_swells_past_that_face.md)
- [`max`/`min` cru é uma intersecção/união SEM raio: a aresta fica VIVA e nem filete nem chanfro lá chegam (3× num dia)](feedback_a_hard_max_is_an_intersection_without_a_radius_and_that_is_a_live_edge.md)
- [Um chanfro ENCHE uma aresta côncava, então «o ponto ficou de fora?» é a pergunta errada — meça o MÓDULO do campo](feedback_a_chamfer_fills_a_concave_edge_so_asking_if_the_point_went_outside_is_the_wrong_question.md)
- [`max(coeficiente, 0)` num referencial OBLÍQUO não é projectar: acerta o vértice e desloca a face](feedback_clamping_a_coefficient_in_an_oblique_frame_is_not_projecting.md)
- ⭐⭐⭐ [Duas peças QUASE COINCIDENTES numa mistura n-ária contam a superfície duas vezes — a cura é dizer ao par o ÂNGULO, não pôr uma cerca no controlo](feedback_two_nearly_coincident_pieces_in_a_blend_need_the_angle_not_a_fence.md)
- [Um recorte que ENCOSTA na peça pára o traçador de esferas — a margem vive na marcha, e varre-se](feedback_a_clip_box_that_touches_the_piece_stalls_a_sphere_tracer.md)

- ⭐⭐⭐ [Uma recusa medida pode nomear uma propriedade que o CONSUMIDOR nunca precisou — «a distância não é fechada» barrou duas formas, e a marcha só pede um MINORANTE](feedback_a_measured_refusal_can_name_a_property_the_consumer_never_needed.md)
- ⭐⭐⭐ [Para saber se a crista é do FILETE ou da FORMA, corra a régua com o knob nos dois extremos (gyroid: `36,74 → 7,86`, o filete TIRA-a)](feedback_to_tell_a_cures_ridge_from_the_shapes_own_curvature_run_the_knob_to_both_ends.md)
- ⭐⭐ [Contínuo numa costura não é contínuo em todas — uma expressão com `round` tem uma costura por operador de escolha (`‖∇f‖ = 2596`)](feedback_continuous_at_one_seam_is_not_continuous_at_every_seam.md)

## As réguas — a forma medida no sítio errado

- ⭐⭐⭐ [Uma COERÇÃO estaciona NA cerca, e uma cerca é por definição onde a forma DEGENERA — varra e ponha-a onde a peça volta a marchar (3 tentativas a aterrar no mesmo sítio mau)](feedback_a_coercion_parks_at_the_fence_which_is_where_the_shape_degenerates.md)
- ⭐⭐⭐ [Um gate no REPRESENTANTE deixa todo o curso do controlo sem régua — derive o censo das faixas DECLARADAS (7 defeitos em 5 formas na 1.ª corrida)](feedback_a_gate_that_measures_the_representative_leaves_the_control_travel_unmeasured.md)
- [Uma sonda com o knob noutro ponto mede OUTRA peça (0,88 contra 1,05) — duas réguas da mesma grandeza a discordar É o achado](feedback_a_probe_with_the_knob_at_another_point_measures_another_piece.md)
- [Um tecto escrito pelo PREÇO pode ser o dobro do que a MARCHA permite — e o representante típico nunca corre o tecto](feedback_a_ceiling_written_from_price_may_be_double_what_the_march_allows.md)
- [Bissecção a partir da ORIGEM supõe a origem DENTRO — a 1.ª peça de miolo vazio recebe uma acusação inventada](feedback_a_ruler_that_walks_from_the_origin_assumes_the_origin_is_inside.md)
- [Uma régua que conta «quanto defeito SOBROU» premeia exagerar a cura — mede quantidade, não correcção](feedback_a_ruler_that_counts_leftover_defect_rewards_overshooting.md)
- [Um tecto em graus SOBE quando a cura correcta piora o número — troque-o por uma IGUALDADE analítica](feedback_a_ceiling_in_degrees_ratchets_up_an_analytic_equality_does_not.md)
- [Um gate que copia a FÓRMULA fica verde sobre uma lei que ninguém shipa — com oráculo analítico, atravesse o PRODUTO](feedback_a_gate_that_copies_the_formula_goes_green_over_a_law_nobody_ships.md)

Vizinhas que ficam no índice por serem gerais: [[feedback_a_gate_that_measures_the_rare_case_leaves_the_normal_one_without_a_ruler]] ·
[[feedback_a_second_error_can_be_load_bearing_for_the_first]] ·
[[feedback_two_halves_of_a_cure_each_refused_alone_do_not_refute_the_cure]] ·
[[feedback_curing_half_a_family_can_leave_the_other_half_worse]]
