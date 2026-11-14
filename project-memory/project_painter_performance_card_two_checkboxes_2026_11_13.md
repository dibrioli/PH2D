---
name: project_painter_performance_card_two_checkboxes_2026_11_13
description: "Dono 13/11 — o palpite da assenta e o AVX2 (unsafe isolado) entram SIM, mas como opções a mais: 2 checkboxes num cartão novo «Performance» no painel do Painter; o de fábrica não muda"
metadata:
  node_type: memory
  type: project
  originSessionId: 98d6aeb8-f8ba-41f7-acbd-2ab8791a9adb
  modified: 2026-11-13T20:30:04.827Z
---

Dono 13/11, verbatim: «Ambas SIM, mas como opções a mais e não a substituição do que já existe. OU seja: 2 checkbox
numa seção nova: performance (card novo no painel).»

As duas perguntas (handoff ONDA7_DESEMPENHO §6, prompt n.º 8 item 11):
1. `PH2D_ASSENTA_PALPITE` (Newton com 0 voltas; sim a 6 traços 0,91 → 0,97 sim-s/s; 15 miniaturas e 3 larguras mudam,
   pranchas em `docs/Painter/missao_4k/medicoes/n7_assenta/`).
2. Liberar `unsafe` num módulo ISOLADO do motor Wet para AVX2 (`core::arch` + detecção em execução, ao bit contra o
   escalar, cai para o escalar sem AVX2). A workspace tem `unsafe_code = "forbid"`: a exceção é só desse módulo.

**Why:** o dono quer a velocidade disponível, mas «adoro os pincéis como estão» — nada substitui o caminho de hoje.

**How to apply:**
- Ambos DESLIGADOS por omissão; o produto de fábrica fica ao bit como hoje. A chave de ambiente pode ficar como
  atalho de teste, mas a escolha do artista é o checkbox.
- Cartão novo «Performance» no painel do Painter (UI em cartões, texto em inglês, CHECKBOX nunca toggle —
  [[feedback_the_app_standard_is_a_checkbox_not_a_toggle]], [[feedback_app_ui_english_only]]).
- AVX2 sem detecção no processador = checkbox sem efeito visível ⇒ diga-o na ajuda (Plain English), nunca minta.
- Ligado a [[project_painter_owner_three_yes_2026_11_13]] e [[project_painter_4k_mission_decisions_2026_11_10]].

**Resultado (13/11 n.º 8):** «Faster wet paint» entrou (cartão Performance, gravado em prefs `wet_guess`). A caixa
AVX2 foi MEDIDA e NÃO entrou: compilar o transporte com AVX2 = ruído (13,44 → 13,36 ms) e o tecto da conta inteira é
~4 % do passo — seria um controlo morto. O dono foi avisado; reabrir só com medição nova
(`docs/Painter/missao_4k/medicoes/N8_NOTAS.md`).

