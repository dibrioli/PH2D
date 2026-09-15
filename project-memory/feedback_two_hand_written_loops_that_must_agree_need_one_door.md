---
name: feedback-two-hand-written-loops-that-must-agree-need-one-door
description: Quando um subsistema anda o relógio por DOIS laços escritos à mão (frente e replay), toda lei por-tique nova é ensinada a um e esquecida no outro — três vezes na mesma ponte, duas descobertas por report do dono.
metadata:
  type: feedback
---

A ponte de física do PH2D anda o relógio por **dois** laços: o de avanço (`bridge::dispatch`) e o de
replay do rewind (`bridge::rewind`). Três controladores foram ligados ao primeiro e esquecidos no
segundo:

| wave | controlador | como se soube |
|---|---|---|
| W7 | `drive_players` | report — o personagem caía pelos tiques replayados |
| TOP-20 #13 | `drive_topdown` | só em 15/09, por acaso |
| TOP-20 #14 | `drive_projectiles` | **report do dono**: *«comportamento diferente a cada rewind»* |

**Why:** um gate de comportamento mede os membros que EXISTEM; ele não pode reprovar sobre o
próximo, que é precisamente onde a família se repete. E o modo de falha é mudo: o mundo volta ao
sítio certo e o controlador é que corre outra simulação — o produto «funciona», só não reproduz.

**How to apply:** ao achar a segunda cópia de um laço que tem de concordar, **não** acrescente a
linha em falta: extraia a **porta** que os dois chamam (aqui `drive_controllers`) e escreva o
**censo** que proíbe chamar um membro fora dela, com a metade que exige que os dois laços chamem a
porta (senão apagá-la dos dois deixa o censo verde). Ver também [[feedback_a_gesture_written_in_two_halves_accepts_a_new_variant_in_only_one]]
e [[feedback_the_door_with_the_right_law_had_no_caller_and_the_consumer_used_a_third]].
