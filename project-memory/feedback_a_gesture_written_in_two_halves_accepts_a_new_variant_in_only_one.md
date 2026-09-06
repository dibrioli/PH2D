---
name: feedback_a_gesture_written_in_two_halves_accepts_a_new_variant_in_only_one
description: "Um gesto escrito em DUAS metades (pen-down decide · arrasto executa) aceita uma variante nova em só uma delas, e o resultado é silêncio absoluto — sem erro, sem log, sem efeito"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1246816c-63cf-414b-842d-663a8baa86ca
  modified: 2026-09-05T18:49:03.405Z
---

O pincel de tecido shipou com solver gateado, chip pintado e clicável, e o dono reportou
***«não funciona, nada aconteceu ao pintar»***. A causa foi **uma linha**: o gesto tem duas
metades em ficheiros diferentes — o **pen-down** escolhe entre *tomar a âncora* e *carimbar*
(`Verb::anchors()`), e o **arrasto** roteia por `Grip` para ramos que abrem com
`let Some(..) = self.grab else { return; }`. O grip novo entrou na 2.ª metade e não na 1.ª:
o pen-down carimbava, a âncora ficava `None`, e **todo evento de arrasto saía no primeiro
`if`**. Sem erro, sem log, sem um vértice movido.

**Why:** ⚠️ **o `match` exaustivo cobre a metade que ele vê, e só ela.** O compilador obrigou
o grip novo a responder no roteador do arrasto (metade 2) e **não tinha como** obrigá-lo a
entrar numa lista `matches!(...)` da metade 1 — uma lista pelo lado positivo obriga o ITEM
novo a declarar-se e não obriga QUEM O ESCREVE a lê-la. ⚠️ O doc daquela lista até previa a
classe por escrito (*«a forma que sobrevive ao sexto grip em vez de o adotar em silêncio»*)
e o sexto grip chegou sem ninguém a ler. ⛔ E as minhas seis fixturas eram verdes porque
chamavam a porta INTERNA (`stroke.dab`) direto, **saltando as duas metades** — *uma fixtura
que não atravessa a porta de entrada do produto fica verde sobre uma feature que não faz
nada*.

**How to apply:** ao acrescentar uma variante a um enum que um GESTO consome, procure as
outras metades antes de shipar: `grep` pelo enum e conte os sítios que decidem, não só os
que executam. E quando a rota do ponteiro não for alcançável de um teste, o gate que fecha a
classe é o que **prende as duas listas uma à outra** (ler o roteador no fonte e cobrar que a
lista de decisão diga o mesmo) — com prova de mutação, senão ele fica verde por vácuo.
Irmãos: [[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]] ·
[[feedback_a_leak_ruler_masked_by_the_products_own_predicate_hides_the_leak]] ·
[[feedback_a_new_feature_can_empty_an_existing_gates_population]].
