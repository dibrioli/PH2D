---
name: feedback-paint-and-dispatch-must-read-the-same-source
description: Um controlo que PINTA do snapshot e DECIDE do WidgetStore tem duas fontes de verdade — e só diverge quando um terceiro (o motor) escreve o facto
metadata: 
  node_type: memory
  type: feedback
  originSessionId: d971358c-b4ab-4ed0-ab84-65cd6d892c68
  modified: 2026-08-23T15:45:57.560Z
---

Um checkbox/toggle que se **pinta** a partir do snapshot do mundo e cujo **despacho** lê o valor
guardado no `WidgetStore` tem **duas fontes de verdade para um facto só**. Elas concordam enquanto
só o painel escrever — e partem no instante em que um **terceiro escritor** (o motor, o solver, o
undo) muda o facto sem passar pela UI.

**Why:** medido na §11 Animation (2026-08-23, report do Enio: *«às vezes preciso clicar mais de uma
vez para checar Playing»*). Uma animação de uma volta põe `playing = false` **sozinha** ao acabar —
sem mudar a entidade nem a linha aberta, que eram as duas arestas em que o `sync` re-semeava o
store. A partir daí o artista via a caixa vazia (o pintor lê o mundo) e o clique mandava
`Playing(false)` (o despacho lê a memória do widget). O 1.º clique não fazia nada, o 2.º ligava —
daí o «às vezes»: numa sprite que só o painel tocou os dois valores nunca divergem.

⚠️ **O gate de registo NÃO apanha isto.** «Todo id pintado está registado no store» cobre o
**segundo** dos três sítios (pintar · registar · despachar); um id perfeitamente registado que
despacha a partir da fonte errada passa sem um arranhão. É preciso um gate que **carregue no pixel**
e leia o barramento ([[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]).

**How to apply:** decida qual é a fonte e faça as duas pontas lerem-na.
- Se o **motor** escreve o campo → o snapshot é a verdade: o clique afirma `!info.<campo>`, e o
  store passa a **espelho por quadro** (ele já não decide nada, só publica para a a11y).
- Se só o **painel** escreve → o store pode ser a verdade, e a semente por aresta de seleção basta.
- O sintoma de diagnóstico é sempre o mesmo: **«às vezes»** + o gesto funcionar ao segundo clique.

Irmã de [[feedback_the_seed_owns_the_value_the_dispatch_owns_the_state]] (aquela é sobre seed vs
dispatch do ESTADO visual; esta é sobre quem é dono do VALOR quando existe um terceiro escritor).
Família: [[reference_topic_ui_seam_discipline]].
