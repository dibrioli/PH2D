---
name: feedback-a-proxy-predicate-that-becomes-constant-leaves-a-vacuous-assertion
description: "Quando o desenho muda e um predicado-proxy passa a devolver sempre o mesmo, toda asserção sobre ele fica VERDE POR CONSTRUÇÃO com o nome de uma protecção"
metadata:
  type: feedback
---

A linha de propriedade deixou de empilhar (o rótulo passou a viver dentro da caixa), então
`slider_with_chip_is_stacked` passou a devolver **`false` sempre**. Isso é a decisão, não um bug.

⛔ **Mas duas asserções perguntavam outra coisa ATRAVÉS dele:** o `debug_assert!` do
`chrome::input_map` e o gate `the_zone_numbers_never_stack_at_the_windows_width` perguntavam
*«a janela é larga que chegue?»* pelo **proxy** do empilhamento. Com o proxy constante, as duas
ficaram **verdes por construção — com o nome de uma protecção**. A pergunta continuava boa; o
instrumento é que deixou de a medir.

**Why:** um predicado é frequentemente usado como *proxy* de uma pergunta que ele não nomeia. Quando
o desenho muda, o compilador acompanha o **tipo** e ninguém acompanha o **significado** — e um
`assert!(!f(x))` sobre um `f` que virou `|_| false` não dá erro, não dá warning, e continua na lista
de gates verdes a dar confiança que já não compra nada.

**How to apply:** ao tornar um predicado constante (ou ao apagar o caso que o fazia variar),
**grepe os consumidores dele antes de commitar** e pergunte, um a um: *este sítio queria saber o
predicado, ou uma pergunta que ele representava?*

- Se queria o predicado ⇒ a chamada pode sair.
- Se era um **proxy** ⇒ ela precisa de um instrumento novo, e a pergunta antiga tem de continuar a
  poder falhar. No caso real nasceu o `slider_with_chip_min_w` (*«cabe o valor + folgas»*).
- ⭐ **O sinal de que era proxy é a mensagem de erro:** ela fala de uma coisa (*«a janela ficou
  estreita e a linha vai vazar»*) e a condição mede outra (*«empilhou»*).
- ⚠️ Deixe o predicado com a **assinatura intacta** quando os chamadores *perguntam ao widget* em vez
  de adivinhar — eles passam a ler a verdade nova de graça. O que não pode ficar é a **asserção**.

Irmãos: [[feedback_a_new_feature_can_empty_an_existing_gates_population]] ·
[[feedback_a_bucket_nobody_fills_reads_as_perfect]] ·
[[feedback_a_literal_corpus_count_in_a_gate_makes_every_new_feature_edit_someone_elses_test]] ·
[[feedback_a_parameter_that_changes_nothing_is_discarded_downstream]]
