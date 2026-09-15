---
name: feedback-the-inner-channel-fixture-is-below-the-break
description: 24 provas de mutação verdes sobre um componente que não andava — todas entravam pelo canal interno, que fica ABAIXO da metade do gesto que estava partida.
metadata:
  type: feedback
---

Medido 2026-09-15 (`line/components`, TOP-20 #13, report do dono: *«a simulação não funciona. Nada se
move»*). A entrega do dedo do jogador é escrita em **duas** metades que correm em fases diferentes do
mesmo tique: a **ENTREGA** (`hand_input_to_players`, na shell — e a contagem que ela devolve decide se
a fita grava) e o **REPLAY** (`take_taped_input`, na ponte). A wave ensinou a segunda a conhecer o
controlador novo e não a primeira.

Resultado medido pela porta do produto: **`0.0000 m` em 30 tiques** com a tecla segurada — e, porque
a contagem de players dava `0`, a fita **nem gravava o tique**, matando o replay pela mesma linha.
*Dois sintomas, uma causa.*

**Why:** as **24** provas de mutação da wave entravam por `bridge.set_player_input(...)` à mão ou por
uma fita falsa — o **canal interno**, que fica abaixo da rotura. Elas provavam a LEI e a PONTE, e a
rotura estava na pergunta *«quem são os players?»* que o produto faz **antes** de chegar lá.

**How to apply:** quando uma feature nova se liga a um gesto que já existe, **conte as metades do
gesto por `grep` antes de fechar** e ponha pelo menos UM gate na porta mais alta do produto — aquela
que o quadro de facto chama. Se a cena e a porta viverem em crates que não se conhecem, o gate mora
na shell: é isso que *«a shell é composição»* quer dizer. ⚠️ E uma agulha TEXTUAL que nomeia a
chamada (*«o quadro chama X»*) é **cega ao corpo dela** — as três que existiam ficaram verdes o tempo
todo. Relacionado:
[[feedback_a_gesture_written_in_two_halves_accepts_a_new_variant_in_only_one]] ·
[[feedback_a_sweep_that_freezes_an_order_must_read_the_same_key_the_tree_reads]] ·
[[feedback_the_door_with_the_right_law_had_no_caller_and_the_consumer_used_a_third]]
