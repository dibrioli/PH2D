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
