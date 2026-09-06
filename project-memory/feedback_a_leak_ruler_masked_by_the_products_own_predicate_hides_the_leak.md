---
name: feedback_a_leak_ruler_masked_by_the_products_own_predicate_hides_the_leak
description: "Régua de vazamento cuja máscara de «tinta legítima» usa o MESMO predicado do produto lê 0,00 % sobre o defeito — numa peça não convexa a aresta que atravessa é ela própria «de frente»"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1246816c-63cf-414b-842d-663a8baa86ca
  modified: 2026-09-05T02:34:18.191Z
---

A sonda do wireframe media *«tinta que caiu onde nenhuma aresta DE FRENTE passa»*, e a máscara de
arestas de frente era construída pelo mesmo predicado que o produto usa (a NORMAL encara o olho).
Numa peça **não convexa** a aresta que atravessa a superfície **é de frente** — é a malha de um
vale visto através da montanha à frente dele. ⇒ a régua leu **`0,00 %`** e eu respondi ao dono
*«não é geometria escondida»* sobre uma tela em que ele via geometria escondida. A régua honesta
(uma esfera atrás de uma chapa opaca, tinta no miolo da chapa) leu **`4,82 %`**, com **83 %** de
toda a tinta do quadro a ser a malha escondida.

**Why:** duas perguntas leem-se iguais e têm respondedores diferentes — *«esta face está de
COSTAS?»* (descarte por normal) e *«esta face está ATRÁS de outra?»* (teste de profundidade).
Uma máscara feita da primeira **contém** todos os defeitos da segunda: *a máscara continha
exactamente o defeito que ela existia para acusar*. É o parente do espelho
([[feedback_a_gate_that_copies_the_formula_goes_green_over_a_law_nobody_ships]]): ali o oráculo
copia a fórmula, aqui copia o **filtro**.

**How to apply:** ao escrever uma régua de *«o que sobra é vazamento»*, pergunte de que predicado
a máscara do legítimo é feita e se o produto usa o mesmo. Se usar, construa a fixtura em que a
resposta certa é **conhecida por construção** e não derivada de predicado nenhum — aqui, uma
malha OPACA à frente de outra, onde zero é a única resposta admissível — e meça as duas metades
que a lei separa (fechada/aberta), porque uma delas desarma a outra lei e interroga a que sobra
sozinha. Ver [[feedback_the_example_the_user_points_at_may_be_the_exception_of_its_family]] e
[[feedback_an_aggregate_that_already_measures_item_by_item_must_return_the_table]].
