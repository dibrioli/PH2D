---
name: feedback_a_shared_widget_slot_has_two_questions_and_only_one_was_answered
description: "Um widget partilhado por N rows tem DUAS perguntas — quem comita o buffer e o que o buffer mostra; a 2.ª faltava, e tocar no campo APAGAVA o valor"
metadata:
  type: feedback
---

Um `TextInput` por slot, partilhado por **quatro** tipos de row (`Text` · `Channels` ·
`Source` · `File`). O `on_text_commit` respondia *«de quem é o buffer quando ele volta?»* com
as quatro armas. Ninguém respondia *«o que é que o buffer MOSTRA?»* — o `seed_rows` espelhava
**só a `Text`**.

Sintoma que o dono reportou: *"O painel não imprime o caminho do CSV em lugar nenhum"*. Metade
visível. A outra metade é que **`Blur` comita**: um campo que abre vazio e é tocado por engano
grava o vazio ⇒ **clicar no caminho e clicar noutro sítio APAGAVA o ficheiro do nó**, sem nada
vermelho em lado nenhum.

**Why:** as duas metades da mesma pergunta viviam em **módulos diferentes** (`events.rs` e
`row_state.rs`), e cada lista parecia completa onde estava. E os doc-comments das três structs
**prometiam** o comportamento em palavras — *"shown in the Custom field"*, *"fills the field"*,
*"the current path"* — com zero código por baixo: *uma promessa escrita na struct não é um
leitor* ([[feedback_a_promise_that_justifies_a_decision_must_have_a_reader]]).

⚠️ **As quatro condições de UI da casa estavam TODAS verdes** — o widget existe · é pintado e
registado · o clique chega ao barramento · a sequência leva a algum lado. Elas perguntam pelo
**WIDGET**; esta pergunta é pelo **VALOR**. É a quinta condição, e é irmã da
[[feedback_the_fifth_seam_link_is_whoever_paints]].
⚠️ E os gates existentes escondiam-na por **fixtura**: todos os da `File` row usavam
`snapshot_with_file("")` — *um gate que só testa o valor vazio nunca vê o valor não chegar*
([[feedback_where_new_objects_are_born_is_the_fixture_your_gates_are_missing]]).

**How to apply:** quando N produtores partilham UM widget de estado, escreva as duas metades
**no mesmo ficheiro**, cada uma `match` exaustivo **sem braço curinga** (uma variante nova é
erro de compilação nas duas), e gateie que elas concordam variante a variante — com um censo
CONTADO ao lado, porque *um match exaustivo não guarda a lista que um laço percorre*
([[feedback_an_exhaustive_match_does_not_guard_the_list_a_loop_iterates]]).
⇒ `crates/ph2d-panel-motion-params/src/shared_field.rs` e o gate
`every_row_that_can_commit_the_shared_field_also_fills_it`.

Relacionado: [[feedback_two_doors_to_the_same_question_diverge]] ·
[[feedback_a_dead_knob_has_two_species_no_probe_catches]]
