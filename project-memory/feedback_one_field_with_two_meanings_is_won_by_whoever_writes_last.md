---
name: feedback-one-field-with-two-meanings-is-won-by-whoever-writes-last
description: "Pintar um realce a partir de um campo que outro passe reescreve todo quadro dá um realce que NUNCA acende — o controlo morto pintado, pior que a ausência dele"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T21:45:47.113Z
---

Antes de pintar um estado a partir de um campo, pergunte **quem mais o escreve, e quando**. Um
campo com dois significados não dá metade da resposta a cada leitor: dá **a de quem escreve por
último**, sempre.

**Medido na `line/UIUX`, 2026-08-31 (entrega 30, o cabeçalho da área):** pintei o realce de *hover*
dos dois controlos a partir de `store.button_state(id)`. Para aqueles ids esse campo não significa
*«sob o rato»* — o `menu_bar::publish_toggle_state` reescreve-o em **todo quadro** com
`Pressed`/`Normal` tirado da tabela de verdade dos menus, e corre **depois** do despacho de
ponteiro. ⇒ o ramo `Hovered` era **inalcançável**, e o realce nunca acendia.

> *Um realce que nunca acende é o controlo morto **pintado** — pior do que a ausência dele, porque
> ele promete uma resposta que o app não tem.*

⭐ **E a pista veio de uma mutação SOBREVIVENTE que parecia falar de outra coisa:** apagar o meu
`populate` não partiu nada, porque os ids já eram registados noutro sítio. Puxar por esse fio
destapou quem de facto mantinha o `ButtonState` deles.

⇒ a cura foi **apagar o ramo** e deixar a faixa mostrar só o que ela sabe (*ligado*), com um gate a
fixar o porquê: *o `ButtonState` destes ids é a VERDADE, nunca o hover* — ele reprova no dia em que
o campo voltar a ser do hover, e nesse dia o realce pode voltar.

**Why:** é a mesma raiz do
[[feedback_a_collapsed_field_does_not_go_neutral_it_takes_over]] e do
[[feedback_the_seed_owns_the_value_the_dispatch_owns_the_state]] — um campo tem **um** dono. A
diferença é o sintoma: aqui nada explode, só um pedaço de UI que não responde.

**How to apply:** ao ler um estado de widget para pintar, grepe **quem lhe escreve** e confirme a
ordem no quadro. Se um passe periódico o reescreve, o seu ramo condicional é decoração — e a
mutação que o apaga fica verde.

Relacionadas: [[feedback_a_dead_knob_has_two_species_no_probe_catches]] ·
[[feedback_a_surviving_mutation_can_mean_the_code_is_wrong_not_the_gate_missing]] ·
[[feedback_a_declaration_with_a_default_is_decoration_until_something_reads_it]]
