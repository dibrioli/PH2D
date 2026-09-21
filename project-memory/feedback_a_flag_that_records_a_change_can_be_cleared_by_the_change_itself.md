---
name: feedback_a_flag_that_records_a_change_can_be_cleared_by_the_change_itself
description: A cerca «nada mexeu» lia a janela suja, e a função que regista a mudança de topologia é a mesma que LIMPA essa janela — o atalho disparava exactamente quando não podia
metadata:
  type: feedback
---

Um atalho guardado por *«nada mexeu»* costuma ler um acumulador de sujidade (uma janela de índices
tocados). ⛔ **Se a operação grande — a que de facto invalida o atalho — LIMPA esse acumulador para
pedir um upload inteiro, a cerca inverte-se:** ela deixa de ler *«nada mudou»* e passa a ler
*«mudou tanto que a janela foi deitada fora»*, que é precisamente o caso em que o atalho não pode
correr.

**Medido** (2026-09-21, tinta fina): a cerca do atalho era `!mexeu` = `!dirty.is_empty()`, e o
`mesh_rebuilt()` — que TODA mudança de topologia chama — faz `dirty.clear()` e `uploaded = false`.
⇒ depois da triangulação do pen-down de um verbo com **ÂNCORA** (que *não carimba, ele PEGA*, logo
não volta a encher o `dirty`) existe um quadro com a malha NOVA, `dirty` vazio e o atalho a
disparar — saltando a porta que devia ter recusado o plano velho.

⚠️ **O comentário da cerca dizia por escrito o que ela assumia** (*«a topologia e as posições não
mexeram»*) e isso era falso; o modo de falha é mudo, porque as duas estruturas ficam internamente
consistentes e só discordam **entre si**.

**Why:** «esta janela está vazia» tem DUAS causas opostas — *ninguém tocou* e *alguém tocou em tudo
e a janela foi descartada* — e o mesmo `is_empty()` devolve `true` nas duas. Um acumulador
incremental nunca é a resposta para *«o consumidor ainda tem a versão certa?»*; essa é uma pergunta
sobre o ESTADO do consumidor.

**How to apply:** ao escrever `if nada_mexeu { atalho }`, procure quem CHAMA `clear()` nesse
acumulador — se for a operação grande, a cerca precisa de uma segunda metade que pergunte ao
consumidor (aqui: *«o device já tem esta malha?»*, o `SlotJob::Full`). E ponha a decisão numa
função **pura** com um gate que mova **uma entrada de cada vez**: com duas a mexer, uma cerca
apagada passa despercebida atrás da outra.

Vizinhas: [[feedback_a_fence_that_never_bites_hides_the_one_that_does]] ·
[[feedback_a_promise_written_in_a_debug_assert_is_not_a_promise_the_product_makes]] ·
[[feedback_two_guards_that_exclude_each_other_disable_a_feature_silently]]
