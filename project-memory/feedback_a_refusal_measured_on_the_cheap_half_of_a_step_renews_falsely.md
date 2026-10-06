---
name: feedback-a-refusal-measured-on-the-cheap-half-of-a-step-renews-falsely
description: "Uma recusa medida sobre a METADE barata de um passo renova-se em falso: o contacto na CPU media a separação (grelha) e o custo era os impulsos ao lado (todos-os-pares)"
metadata:
  type: feedback
---

O doc 115 recusou o contacto dos colisores na placa medindo `ph2d_contact::separate` (pela grelha): `1 024`
peças `0,5` ms, `4 096` `1,5` ms — e a 2.ª onda de 05/10 RENOVOU a recusa com esse número. No app (`=114` com
`Collide`, doc 121 §9.19 (5)) o quadro era `247` ms a `1 024` peças e `> 2` s a `4 096`: o `sim.step` chama a
separação E LOGO A SEGUIR `ph2d_contact::impulsos`, cujo `monta` percorria **todos os pares** (`O(n²)` por
sub-passo, `×8` sub-passos). A cura foi exacta (os pares da mesma grelha, os mesmos bits): `151 → 14,9` ms.

**Why:** a sonda media a FUNÇÃO que o nome do problema sugeria (*«a separação»*), não o PASSO do produto; o
termo caro estava na chamada vizinha, que nenhuma régua tocava. E o critério da recusa era sobre o quadro do app —
que nunca tinha corrido.

**How to apply:** antes de recusar (ou renovar a recusa de) uma mudança de arquitectura por custo, meça o PASSO
inteiro que o produto corre (o quadro do app, ou uma sonda que chame o MESMO ponto de entrada do nó — aqui o
`sim.step`, não o `separate`), e liste as chamadas desse passo antes de escolher qual medir. Liga com
[[feedback_a_probe_that_measures_one_piece_of_a_cure_does_not_measure_the_cure]] e
[[feedback_a_defect_attributed_to_a_component_is_swept_before_the_cure]].
