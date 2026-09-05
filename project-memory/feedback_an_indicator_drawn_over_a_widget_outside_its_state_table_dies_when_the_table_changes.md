---
name: feedback_an_indicator_drawn_over_a_widget_outside_its_state_table_dies_when_the_table_changes
description: "Um indicador de estado traçado POR CIMA de um widget, fora da tabela de estados dele (o anel do modo Image Tools no topbar), morre no dia em que a tabela muda (o tema plano apaga molduras) — a cura é o widget saber que está activo, não passar o anel pela porta"
metadata:
  type: feedback
---

2026-09-05, `line/UIUX`, wave 3 do redesenho plano: ao converter os últimos 22 pintores à porta
da moldura (`paint::stroke_frame`), o `screens/hero/topbar/mod.rs` traçava um **anel de acento
por cima do chip do Image Tools** quando o modo estava ligado — reconstruindo o `chip_rect` à mão
(«must mirror `paint_topbar_rail_chip` exactly»), sem o chip saber que estava em modo. O clique
só vira `image_edit.mode_on`; o `ButtonState` do chip nunca fica `Pressed`. **Era o único
indicador do modo.**

Passar o anel pela porta com `Feel::Active` dá **nada** num tema moderno (moldura em repouso e
activo = `0`, por desenho — é o Godot) ⇒ o modo ficaria **invisível**, com a suíte inteira verde
e o censo do fonte satisfeito.

**Why:** um indicador desenhado fora da tabela de estados do widget é uma segunda resposta à
pergunta *«como é que este controlo se mostra activo?»* — e a primeira resposta (a matriz do
rail: `AccentSoft` + contorno de acento) já existia no próprio pintor do chip, a uma linha do
`is_active`. Quando a tabela muda de família (plana), a resposta oficial acompanha e a paralela
não. É a irmã do [[feedback_two_doors_to_the_same_question_diverge]] na costura visual, e do
[[feedback_the_fifth_seam_link_is_whoever_paints]]: o pintor que desenha por cima não lê o estado.

**How to apply:**
- Um estado (modo ligado · ferramenta em mãos · selecção) entra no widget como **entrada do
  pintor** (`active: bool`, `is_active = active || Pressed`), nunca como um segundo traço por
  cima, e o look sai da mesma matriz que os irmãos usam.
- Ao converter um pintor a uma porta de tema, **cada traço que a porta apagaria tem de ter a
  pergunta feita: *isto é decoração, ou é o ÚNICO sinal de alguma coisa?*** Se é sinal, a cura
  não é uma excepção na porta — é mover o sinal para onde o tema já sabe mostrá-lo.
- Gate que o mede: `the_image_tools_chip_shows_the_mode_in_every_family` (o chip com `active`
  emite `draw_data` diferente do em repouso nas três famílias). Caso registado em
  `docs/UI_New_and_Simple/pesquisa/08 §7.5`.
- ⚠️ Foi a única mudança visível no CLÁSSICO da wave: o chip do modo passa de *BgElev + anel* a
  *AccentSoft + contorno* — o look que o rail já dava à ferramenta activa.
