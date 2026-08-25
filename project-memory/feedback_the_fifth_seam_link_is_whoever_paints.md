---
name: feedback-the-fifth-seam-link-is-whoever-paints
description: "As quatro condições de costura podem estar verdes e o widget ler MORTO: o quinto elo é quem PINTA, e um rect desenhado à mão não lê o estado que o despacho já escreve"
metadata:
  type: feedback
---

Enio, 2026-08-24 (janela do Input Map, 2ª volta, com foto): *"A caixa de texto parece morta, não se
vê que o foco está nela ao clicar."*

Estava certo — e **nada na costura estava partido**. O campo existia, era **registado**, o clique
**chegava**, e a sequência **levava a algum lugar**: o `pointer_down` já escrevia
`TextInputState::Focused` no `WidgetStore`. O defeito era o pintor: ele desenhava um
`stroke_rounded_rect` + `paint_text` **à mão**, em vez de chamar o widget da casa — logo não havia
preenchimento, nem anel de foco, nem cursor, nem hover. **Ninguém lia o estado.**

**Why:** as quatro perguntas de costura ([[reference_topic_ui_seam_discipline]]) são sobre o caminho
do **evento**. Nenhuma delas pergunta se o caminho de **volta** existe. Um widget cujo estado é
escrito e nunca lido passa nos quatro e lê-se como avariado — e o report que ele gera (*"parece
morto"*) é **idêntico** ao de um controlo nunca pintado e ao de um morto sob o ponteiro, que esta
mesma linha já pagou duas vezes.

**How to apply:** quando um controlo "não reage", **não comece pelo despacho** — grepe o pintor e
veja se ele constrói o widget com `.visual(store.…_visual(id))` / lê o `InteractiveState`. Um
pintor com `fill_rounded_rect`/`stroke_rounded_rect` cru onde a casa tem `paint_text_input` /
`paint_button` / `paint_list_item` é o suspeito. E o gate que apanha isto **tem de medir TINTA**:
pintar duas vezes, com o estado calmo e com o estado que se quer ver, e exigir que a codificação da
cena **difira** — um gate que lesse `store.get(id)` fica verde com o campo morto. Irmão de
[[feedback_paint_and_dispatch_must_read_the_same_source]] e de
[[feedback_the_seed_owns_the_value_the_dispatch_owns_the_state]]; a sonda de tinta tem a cerca de
[[feedback_a_probe_that_sums_two_signals_cannot_say_which_failed]].
