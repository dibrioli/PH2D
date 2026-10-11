---
name: feedback-a-narrow-mutation-table-reads-survived-where-the-suite-bleeds
description: "A coluna «corre» de uma tabela de mutação é um FILTRO: estreita, lê «sobrevive» onde a suíte da crate sangra"
metadata:
  type: feedback
---

Medido no fecho do ciclo 20 do Motion (11/10): 4 das 8 «sobreviventes» das provas W0–W4 (Q29–Q31,
Q34, passe do campo em `ph2d-render`) **não sobreviviam** — a tabela nomeava 5 gates, e a suíte
`fx_stack` inteira já as derrubava (`the_raster_bevel_holds_still_under_subpixel_motion`,
`the_noise_is_pinned_to_the_shape_not_to_the_scratch`, …). O controlo «o filtro casa ≥ 1 teste»
passava — casava, só que os testes ERRADOS.

**Why:** quem escreve a tabela escolhe os gates que ACHA que medem a lei; a suíte sabe quais medem.
Um «sobrevive» lido assim manda escrever gates para defeitos que já estavam cobertos e, pior,
convence que a cobertura era pior do que é.

**How to apply:** antes de declarar SURVIVE, corra a suíte da crate inteira (ou o prefixo do
módulo, p. ex. `fx_stack`, sem as sondas de relógio) sob a mutação; só sobrevive o que sobrevive a
ISSO. A tabela estreita serve para iterar rápido, não para o veredito. Ligado a
[[feedback_a_mutation_harness_needs_a_positive_control_that_a_test_ran]].
