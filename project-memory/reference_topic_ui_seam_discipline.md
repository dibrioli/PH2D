---
name: topic-ui-seam-discipline
description: "Família: costura de UI — pintado/populado/despachado/clicado, portas únicas, defaults que são lei"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 85e38f84-1b86-49d2-aee2-91da101e1fd7
  modified: 2026-07-21T01:05:09.375Z
---

# Costura de UI (índice de família — detalhe em cada arquivo)

- [[feedback_context_menu_closes_on_down_repaint]] — menu "não faz nada" = falta populate; grep o id PRIMEIRO
- [[feedback_tool_unit_green_integration_dead]] — unit-verde ≠ funciona no produto; só e2e pega
- [[feedback_a_default_feature_list_does_not_reach_a_consumer_that_disables_defaults]] — lista `default` não alcança quem desliga defaults; gate mora onde o binário compila
- [[feedback_a_click_is_a_press_that_drifted]] — clique humano é um press que DESLIZOU; Down/Up na mesma coord é robô
- [[feedback_painted_is_not_populated_paint_gate]] — pintado ≠ populado; teste a PINTURA
- [[feedback_widget_is_done_when_a_test_clicks_it]] — widget pronto = um teste CLICA nele
- [[feedback_a_condition_that_enumerates_its_readers_rots]] — condição que ENUMERA seus leitores apodrece no 3º consumidor
- [[feedback_a_default_that_fits_the_majority_is_still_a_law]] — default da maioria ainda é LEI; sem porta o 3º caso não mora
- [[feedback_the_fullest_card_premise_rots]] — "o card mais cheio" apodrece; pergunte a CADA modo
- [[feedback_two_doors_to_the_same_question_diverge]] — duas portas para a mesma pergunta DIVERGEM
- [[feedback_disabled_button_still_dispatches]] — botão dimmed ainda despacha; recuse no event.rs
