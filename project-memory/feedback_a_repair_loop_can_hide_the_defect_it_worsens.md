---
name: feedback_a_repair_loop_can_hide_the_defect_it_worsens
description: Um laço de reparo cujo critério de parada é "não há mais sinalizados" pára quando AGRAVA o defeito para uma forma que o detector não vê
metadata:
  type: feedback
---

O laço de limpeza do traçado dissolvia paredes enquanto houvesse *patch degenerado*.
Num toro ele corria dez rondas: as nove primeiras não mudavam nada, e a décima
trocava **«um anel, sinalizado»** (duas fronteiras — o detector apanha) por **«uma asa
dentro de um patch, não sinalizada»** (uma fronteira — o detector é cego). Aí a lista
de sinalizados ficava vazia e o laço declarava sucesso.

⇒ **Ele agravava e apagava o aviso no mesmo passo**, e a cadeia devolvia uma malha de
género errado com 100 % de quads, zero arestas de bordo e zero não-manifold.

**Why:** *"parar quando não há mais sinalizados"* é um critério sobre o **detector**,
não sobre o **defeito**. Quando o detector é incompleto — e este era: contava
fronteiras, que apanha o anel e é cego ao género — o laço tem um caminho de saída que
passa por piorar. E a saída por agravamento é **indistinguível** da saída por cura em
tudo o que o laço olha.

**How to apply:** todo laço de reparo precisa de **uma segunda régua, invariante**,
que ele esteja proibido de piorar — separada da lista que ele consome. Aqui é
`|χ(complexo) − χ(peça)|`. E a ronda corre sobre uma **cópia**, adoptada só se
passar: reparar no original e desfazer depois deixa uma janela em que o estado está
errado, e o `break` dessa janela é justamente o caminho do defeito.

⚠️ **E o teto de «rondas paradas» NÃO é essa régua** — foi construído, medido e
rejeitado: uma esfera precisava de **cinco** rondas idênticas antes de fechar e o toro
gastava **dez** antes de partir. Indistinguíveis enquanto correm; *uma paciência que
decide certo num caso e errado no outro é um palpite, não uma constante.*

Irmã de [[feedback_a_new_features_gate_can_expose_a_pre_existing_bug_check_the_control_first]]
e de [[feedback_a_conserved_invariant_cannot_grade_quality]].
