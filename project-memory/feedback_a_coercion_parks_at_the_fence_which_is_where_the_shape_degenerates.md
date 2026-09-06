---
name: feedback-a-coercion-parks-at-the-fence-which-is-where-the-shape-degenerates
description: `keep_above` põe o valor a um milésimo da cerca, e uma cerca é por definição onde a forma degenera — arrastar o controlo abaixo do limite entrega SEMPRE o pior caso, e pôr a cerca "ao lado" do ponto degenerado continua a ser o ponto degenerado (escudo, 3 cercas).
metadata:
  type: feedback
---

Medido em 2026-09-05, na wave da nuvem (doc 06 §122.6). O escudo tinha a cerca `2s > w`, coagida
por `keep_above(half, w * 0.5)`, que devolve `cerca/(1 − 1e-3)`. ⇒ **todo utilizador que arrasta o
controlo para baixo do limite aterra a um milésimo da cerca**, e a cerca era exactamente onde os
dois arcos do escudo coincidem e a peça deixa de ter interior. A marcha rasgava (`passo × ‖∇f‖`
acima de `1`, isto é, o traçado atravessa a superfície) e o que se via não era a peça.

Foram precisas **três** tentativas — `0,5 → 0,7 → 0,8` — cada uma a aterrar noutro ponto ainda
degenerado, antes de eu varrer a razão e ler a tabela:

| `s/w` | 0,50 | 0,70 | 0,80 | **0,90** | 1,00 | 1,20 |
|---|---|---|---|---|---|---|
| marcha | degenera | 1,10 | 1,05 | **0,80** | 0,78 | 0,71 |

**Why:** uma cerca escreve-se a pensar *«abaixo disto é inválido»*, e a coerção transforma-a em
*«este é o valor que o produto de facto entrega»*. As duas leituras têm sinais opostos: a primeira
diz que o ponto nunca é usado, a segunda diz que ele é o **caso normal** de quem arrasta o slider
até ao fim. Escolher a cerca por raciocínio (*«ponho um pouco acima do sítio mau»*) é escolher onde
o produto vai passar a viver, sem medir lá.

**How to apply:** ao pôr um `keep_above`/`keep_below`, **varra a grandeza que ele limita e ponha a
cerca onde a peça volta a marchar**, não onde ela pára de ser inválida — e escreva a tabela ao lado
da constante (§0). Um gate que só mede o representante não vê nada disto: é preciso o que arrasta
cada linha pela faixa declarada
([[feedback_a_gate_that_measures_the_representative_leaves_the_control_travel_unmeasured]]).
Irmãs: [[feedback_a_fence_can_guard_two_things_and_name_only_one]] ·
[[feedback_a_declared_fence_chooses_the_shape_of_its_own_cure]] ·
[[feedback_tightening_an_input_invalidates_the_constant_that_was_calibrated_around_it]]
