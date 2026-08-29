---
name: feedback_a_blind_smoother_converges_to_a_mesh_that_forgot_the_field
description: "Um alisador que só olha para a forma da face converge para uma grade mais quadrada e CEGA ao relevo — o alvo dele tem de carregar a direcção, não só a forma"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-28T22:17:54.630Z
---

Medido 2026-08-28 (acabamento da cadeia de quads): a relaxação por ajuste de quadrado leva o
enviesamento mediano de `8,6°` a `3,2°` **e o relevo de `11,9°` a `18,8°`** (`22,5°` = uma
grade que ignora a forma). Ela desliza a grade pela superfície até os quads serem quadrados,
e apaga a propriedade que distingue uma retopologia por campo cruzado de um remesh por voxel.

⛔ **Duas curas erradas, as duas medidas:** *alisar menos* (o oráculo prova que dá para ter
as duas — o passe dele melhora a forma e **melhora** o relevo no gancho), e uma **cerca de
distância** (a `0,35 h` guarda o relevo e paga o `p99` do enviesamento, `52,8°` contra
`34,5°`, porque prende exactamente os vértices que mais precisavam de andar).

⭐ **A cura é dar DIRECÇÃO ao alvo:** o quadrado mais próximo tem forma fechada e o tamanho
vem dos pontos — a **orientação** não precisa de vir. Rodá-la para a direcção principal da
superfície (dobrada em `[−45°, 45°]`, que é o módulo de uma grade) com peso = a **anisotropia
crua** dá enviesamento `4,5°` **e** relevo `11,2°`, melhor que os dois lados.

**Why:** o peso ser a própria confiança (sem constante por cima) faz a lei degenerar **ao
bit** onde não há direcção preferida — numa esfera as linhas `x0`..`x4` da varredura são
idênticas em toda coluna. *Um alinhamento sem confiança põe costura onde a forma não pede
nenhuma; um alisador sem alinhamento apaga a costura que a forma pedia.*

**How to apply:** quando um pós-processo optimiza uma forma LOCAL sobre um resultado que
nasceu de um campo GLOBAL, pergunte o que ele faz à propriedade global **antes** de escolher
quantas rondas. E prefira pôr a informação no ALVO da iteração a limitá-la por fora — uma
cerca é sempre a mesma para todos, e a informação não é. Relacionado:
[[feedback_a_smooth_fixture_cannot_tell_two_smoothers_apart_the_pointed_one_can]] ·
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]] ·
[[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]]
