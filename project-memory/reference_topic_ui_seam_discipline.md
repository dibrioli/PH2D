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
- [[feedback_a_capability_without_a_door_passes_every_gate]] — motor completo e INALCANCAVEL: sem porta, todo gate fica verde
- [[feedback_painted_is_not_populated_paint_gate]] — pintado ≠ populado; teste a PINTURA
- [[feedback_widget_is_done_when_a_test_clicks_it]] — widget pronto = um teste CLICA nele
- [[feedback_a_condition_that_enumerates_its_readers_rots]] — condição que ENUMERA seus leitores apodrece no 3º consumidor
- [[feedback_a_default_that_fits_the_majority_is_still_a_law]] — default da maioria ainda é LEI; sem porta o 3º caso não mora
- [[feedback_the_fullest_card_premise_rots]] — "o card mais cheio" apodrece; pergunte a CADA modo
- [[feedback_two_doors_to_the_same_question_diverge]] — duas portas para a mesma pergunta DIVERGEM
- [[feedback_disabled_button_still_dispatches]] — botão dimmed ainda despacha; recuse no event.rs
- [[feedback_ship_the_ui_in_the_same_wave_not_later]] — atalho com valores fixos é harness vazando; sem indicador não há diagnóstico
- [[feedback_one_parameter_two_roles_makes_the_wrong_call_defensible]] — parametro com dois papeis: o produto e o unico chamador que os separa
- [[feedback_the_fifth_seam_link_is_whoever_paints]] — as 4 condicoes verdes e o widget ainda le morto: o pintor desenhou o rect a mao e nao le o estado que o despacho JA escreve
- [[feedback_a_hit_rect_is_also_the_denominator_not_only_the_target]] — o rect registado tambem e' o DENOMINADOR: mais estreito que o pintado nao recorta, ESCALA (1,62x)
- [[feedback_a_shared_widget_slot_has_two_questions_and_only_one_was_answered]] — widget partilhado por N rows: quem COMITA tinha 4 armas, o que MOSTRA tinha 1; tocar no campo APAGAVA o valor
