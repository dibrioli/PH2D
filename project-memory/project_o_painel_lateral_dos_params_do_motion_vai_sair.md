---
name: project-o-painel-lateral-dos-params-do-motion-vai-sair
description: Decisão do dono (2026-09-19) — os params de um nó passam a viver só no CARTÃO; o painel lateral do Motion sai e outra linha trabalha nisso.
metadata:
  type: project
---

**Enio, 2026-09-19**, no smoke da varredura de elisões do painel de params do Motion:

> *«as propriedades dos nós não existirão no painel lateral em versões futuras. Apenas nos próprios
> nós dos grafos. Então não vale a pena investir no painel lateral dos motion nodes. Temos outra
> linha trabalhando nele.»*

**Why:** o painel lateral já nasce DESLIGADO desde 2026-09-07 (`PH2D_MOTION_PANEL=1` traz-no de
volta), quando o censo `what_the_card_still_cannot_reach` chegou a zero e a autoria se mudou para o
cartão do nó. Esta decisão é o passo seguinte — ele **sai**, e o trabalho pertence a outra linha.

**How to apply:**
- ⛔ **Não invista no `ph2d-panel-motion-params` nem no painel lateral do Motion** — nem largura de
  coluna, nem chips, nem frases cortadas. Os `12` cortes que a varredura mediu ali vivem em
  `FORA_POR_DECISAO_DO_DONO` (em `nenhum_rotulo_do_app_pinta_nada.rs`) e **não são dívida**.
- ⛔ **Não toque no módulo Motion** de forma geral: os cartões e o grafo são de outra linha.
- ⭐ O que a passagem já curou **não se desfaz**, e a porta que a curou é da CASA
  (`ph2d_editor_core::widget::wrapped_cells_for` — a fileira segmentada que mede as palavras na
  largura **e** na quebra). Ela serve toda fileira segmentada que quebre, e o cartão do nó é uma
  delas.
- ⚠️ A **armação** da varredura fica: enquanto o painel estiver no registo ele é medido, e o censo
  de obsolescência manda apagar cada linha no dia em que ela deixar de descrever um corte. *É assim
  que esta lista deve morrer — sozinha.*

Relacionado: [[feedback-a-declared-blind-spot-is-still-a-blind-spot]] ·
[[reference-topic-measurement-discipline]]
