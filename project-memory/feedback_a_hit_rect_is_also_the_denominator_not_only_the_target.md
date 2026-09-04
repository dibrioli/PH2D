---
name: a-hit-rect-is-also-the-denominator-not-only-the-target
description: "O rect registado no HitIndex tem DOIS papéis — quem recebe o gesto E por quanto dividir; registá-lo mais estreito que o pintado não recorta, ESCALA"
metadata:
  type: feedback
---

Ao registar o alvo de um controlo **contínuo** (slider, trilha, barra de progresso arrastável), o
`Rect` que vai ao `HitIndex` responde a **duas** perguntas, e quase toda a gente só vê a primeira:

1. *este ponto é meu?* — o hit-test;
2. *quanto vale este ponto?* — `t = (px − rect.x) / rect.w`, no despacho.

⇒ **Registar um rect mais estreito do que o que se PINTA não recorta a zona sensível: multiplica
todo valor** por `w_pintado / w_registado`. O erro é `0` na borda esquerda e **cresce ao longo do
curso** — que é exactamente como um humano o descreve: *«offset e drift em relação ao cursor»*
(Enio, 2026-09-03).

**Why:** o raciocínio que produz o defeito é plausível e local — *«a coluna da direita é para
escrever, logo não é para arrastar; então não a registo»*. Ele responde à pergunta 1 e não sabe que
a 2 existe. Medido na caixa única do PH2D: `240 px` de linha com `12 + 72` de coluna de valor ⇒
factor **1,62×**. A bancada tinha o **mesmo defeito ao contrário** (registava a coluna de animação,
pintava sem ela, `1,07×`) e por isso passou despercebido meses: *a mesma lei partida dos dois lados
opostos, e nenhum sítio onde comparar.*

**How to apply:**
- **UMA função devolve a superfície** que o preenchimento atravessa, e ela tem dois leitores: o
  pintor e quem regista (`ph2d_editor_core::widget::surface_rect`). ⛔ Nunca duas expressões.
- **Excluir uma sub-zona faz-se por ORDEM de registo, não por largura** — o `HitIndex` resolve em
  `rev()`, então registar o filho (o campo numérico) **depois** do pai (a trilha) dá-lhe a coluna
  sem mexer no denominador. A ordem passa a ser load-bearing: gateie-a.
- O topo da faixa continua alcançável porque o ponteiro sai pela direita e o `clamp` entrega `1.0`
  (é o que o Blender faz) — ⛔ não «corrija» isso encolhendo o rect.
- **O gate corre o GESTO.** Comparar `hit.rect(id)` com `surface_rect(...)` é a mesma expressão dos
  dois lados — vácua ([[feedback_a_proxy_predicate_that_becomes_constant_leaves_a_vacuous_assertion]]).
  O gate certo carrega o ponteiro em 3 pontos do curso e exige que `rect.x + rect.w·t` caia a `≤1 px`
  do cursor, **com um controlo** que reproduz o registo estreito e prova que a barra o apanha
  (`the_fill_lands_under_the_cursor`).

Relacionado: [[feedback_two_doors_to_the_same_question_diverge]] ·
[[feedback_one_parameter_two_roles_makes_the_wrong_call_defensible]] ·
[[reference_topic_ui_seam_discipline]]
