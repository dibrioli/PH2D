---
name: feedback_a_varying_target_field_is_projected_out_if_it_is_not_integrable
description: "Pôr um campo variável no ALVO de um solver de mínimos quadrados que resolve um potencial escalar não o entrega: a parte não-integrável é projectada fora"
metadata:
  type: feedback
---

Medido 2026-08-28: para dar densidade adaptativa a um remalhador de quads, o passo da grade
passou de escalar a **campo por vértice** no alvo do gradiente (`direcção / h(x)`). O campo
chegava ao solver a variar **`4×`** — e a saída movia-se **`7 %`** (expoente de
`aresta ∼ curvatura` de `+0,047` para `+0,014`), pagando `15 %` da contagem e o dobro das
faces péssimas.

⭐ **O mecanismo:** o solver resolve um **potencial escalar** por região cujo gradiente se
aproxima do alvo. Com `h` constante o campo alvo é **integrável**; com `h` a variar o
rotacional deixa de ser nulo, e a projecção de mínimos quadrados fica com a parte integrável
— que é, quase exactamente, o campo uniforme. *A adaptação não é ignorada: é projectada fora.*

**Why:** «pôr a variação no alvo» parece a mudança de uma linha e não é uma mudança de
mecanismo nenhuma. O sintoma é traiçoeiro: o campo **está** lá (imprimir a sua faixa
confirma-o), a saída **muda** um pouco, e sem uma régua da *forma* da saída lê-se como
«funcionou pouco» em vez de «foi projectado fora».

**How to apply:** antes de pôr um campo variável no alvo de um solver, pergunte se o espaço
de soluções o pode **representar**. Se a incógnita é um potencial escalar, só a parte
integrável sobrevive — e a cura é construir o campo **integrável por construção** (para um
factor de escala: resolver `Δ log h` contra a curvatura, `h = h₀·e^{−s}`), nunca afinar o
peso. ⚠️ E meça a **forma** da saída, não só a média: a régua tem de ser `saída ∼ campo`
(um expoente), porque toda régua global fica igual. Relacionado:
[[feedback_a_constraint_imposed_in_one_phase_and_not_the_next_is_a_starting_point]] ·
[[feedback_a_parameter_that_changes_nothing_is_discarded_downstream]] ·
[[feedback_a_correct_mechanism_can_prescribe_the_wrong_cure]]
