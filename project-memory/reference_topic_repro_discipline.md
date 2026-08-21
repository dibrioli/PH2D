---
name: topic-repro-discipline
description: "Família: reprodução e diagnóstico — harness, medição antes de causa, controles positivos"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 85e38f84-1b86-49d2-aee2-91da101e1fd7
  modified: 2026-07-21T01:04:08.371Z
---

# Disciplina de reprodução/diagnóstico (índice de família — detalhe em cada arquivo)

- [[feedback_a_gesture_report_needs_a_fixture_containing_the_gesture]] — report de GESTO pede fixture com o gesto; chamar a mutação direto pula a costura input→bus→drain
- [[feedback_harness_reproduces_mechanism_not_context]] — harness reproduz mecanismo, não contexto; instrumente o app real
- [[feedback_a_windowed_drive_races_the_real_cursor]] — drive janelado disputa com o cursor FÍSICO/WM; re-afirme a posição por frame; anomalia só-em-alguns-runs = ambiente
- [[feedback_nonreproduction_is_not_proof_of_fix]] — não-reprodução ≠ correção; cheque o `git diff`
- [[feedback_first_case_rescued_by_side_effect_test_repetition]] — 1º caso salvo por efeito colateral; teste a REPETIÇÃO
- [[feedback_try_to_build_the_harness_before_declaring_it_impossible]] — CONSTRUA o harness antes de desistir ("o App exige janela" era falso)
- [[feedback_a_negative_search_needs_a_positive_control]] — busca negativa pede controle positivo; grep vazio mente
- [[feedback_remeasure_a_documented_residual_before_curing_it]] — re-meça resíduo anotado; a causa E o número podem estar errados
- [[feedback_measure_perf_symptom_scale]] — meça a ESCALA antes da causa (frame 4-16ms vs ⅓s muda a classe)
- [[feedback_a_frontier_is_not_a_census]] — fronteira não é censo; o custo é da POSIÇÃO; meça antes de construir sobre um "N" herdado
- [[feedback_a_conservative_verdict_must_separate_unchanged_from_unmeasurable]] — colapsá-los mata a otimização com todos os gates VERDES
- [[feedback_a_cited_number_whose_probe_lost_its_caller_stops_being_reproducible]] — devolva a chamada e confira o valor, nunca silencie o lint
- [[feedback_a_persistent_default_bug_lives_in_a_reset_path_not_the_create_path]] — enumere toda porta que reconstrói o estado (new/default/purge/load)
- [[feedback_an_approximation_inside_a_fixed_point_walks_it_does_not_merely_err]] — tabela num laço de realimentação: meça deriva sob iteração, não erro de chamada única
- [[feedback_probes_that_measure_parallelism_must_run_alone]] — concorrentes disputam o pool e medem uma à outra; o controle interno é o detector
- [[feedback_a_component_missing_its_contract_suspect_the_caller_first]] — trocar o componente esconde a causa e costuma trazer um 2º defeito
- [[feedback_a_per_pass_gain_becomes_a_product_gain_only_through_the_cadence]] — 1,56× virou 1,10×; e razão não se transporta entre cenas
- [[feedback_a_rule_copied_to_a_second_site_may_lose_its_premise]] — ablacione um braço por vez; o outro sítio pode ser o controle positivo
- [[feedback_a_flattening_curve_may_need_more_points]] — curva de 4 pontos que achata pode ser uma de 6 que nao; nao declare 2o mecanismo cedo
