---
name: feedback_the_app_standard_is_a_checkbox_not_a_toggle
description: Dono 10/11 — o padrão do app para um liga/desliga é a CAIXA DE SELEÇÃO (checkbox), não o interruptor deslizante (toggle/pílula)
metadata:
  type: feedback
---

Dono, 10/11/2026, ao ver o cartão «Script options» do editor de scripts com pílulas à macOS: *«O padrão do
app é checkbox e não toggle»*.

**Why:** a pílula tinha entrado por «estado da arte» (macOS/iOS) em 09/11 sem perguntar; o dono quer UM
idioma de liga/desliga no app inteiro, e esse idioma é a caixa de marcar dos cartões
(`ph2d_editor_core::widget::paint_card_check`, a única, T64).

**How to apply:** opção liga/desliga nova em cartão/popover = `card_check_rect` + `paint_card_check`, nunca
`paint_switch`. «Escolha de acabamento ao estado da arte» ([[feedback_finish_choices_go_to_the_state_of_the_art]])
NÃO passa por cima do idioma que o app já tem: primeiro o padrão da casa.
