---
name: feedback-a-literal-corpus-count-in-a-gate-makes-every-new-feature-edit-someone-elses-test
description: "Gate que afirma `covered == 16` fica vermelho quando uma feature legítima cresce o corpus — derive a contagem dos dados"
metadata:
  type: feedback
---

O `every_toggle_row_of_the_bar_is_marked_by_its_own_state` (barra de menus) terminava em
`assert_eq!(covered, 16)`. Acrescentar **uma linha ao menu *Window*** — um painel novo, obra
legítima de outra linha — pôs o gate **vermelho sem nada estar errado**.

A grandeza que ele quer mesmo é *«toda linha do Window, mais as três de fora»*. Escrita assim
(`menu_rows(MenuBarWindow).len() + OUTSIDE_WINDOW_TOGGLES`) ela continua a apanhar o defeito real
— uma linha de alternância sem marca de estado — e deixa de reagir ao tamanho do corpus.

**Why:** um literal ali transforma **cada feature nova** numa edição ao teste de outra pessoa. O
sinal que isso produz é *«o teu painel partiu um gate»*, que é ruído com cara de defeito — e o
custo real é o hábito que ele ensina: **baixar/subir o número sem ler o que o gate mede**. Duas ou
três voltas disso e a asserção passa a ser um carimbo.

⚠️ **A distinção que importa:** um número literal é legítimo quando é um **piso de população**
(*"vi só 3 rows, o corpus está vazio e um gate que não vê nada passa sempre"*) — ali ele defende
contra o verde por vácuo e **não** cresce com o produto. É ilegítimo quando é a **igualdade exacta
de uma contagem que o produto faz crescer**.

**How to apply:** ao escrever um gate que conta, pergunte *«esta contagem cresce quando alguém faz
o trabalho dele?»*
- **Sim** ⇒ derive-a dos dados, no próprio teste.
- **Não** (é um piso contra o vácuo) ⇒ literal, com o número medido e a data ao lado.

Irmãos: [[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]] ·
[[feedback_a_new_feature_can_empty_an_existing_gates_population]] ·
[[feedback_a_bucket_nobody_fills_reads_as_perfect]]
