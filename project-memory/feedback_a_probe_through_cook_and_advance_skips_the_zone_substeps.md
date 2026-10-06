---
name: feedback-a-probe-through-cook-and-advance-skips-the-zone-substeps
description: "Sonda/gate que anda por cook()+advance_tick() NÃO corre os sub-passos da zona — só a porta do app (advance_or_scrub_scoped) corre; 8 e 1 sub-passo deram os MESMOS bits"
metadata:
  type: feedback
---

Medido em 06/10 (doc 121 §9.20, a prova do mundo de contacto): a 1.ª redacção da sonda variava os `substeps` da
`sim.zone` e media por `pump.cook.cook()` + `pump.cook.advance_tick()` — as variantes de `8` e de `1` sub-passo saíram
com os MESMOS bits. Os sub-passos são corridos por `substep_declared_zones`, que só a porta do app
(`pump.advance_or_scrub_scoped` / `advance_or_scrub_to_nodes_scoped`) chama. A `prova_dos_impulsos_da_placa` de
05/10 mediu assim a lei de contacto a 1 passo por tique enquanto a cena shipava 8 — a tabela dela não era o produto.

**Why:** dois caminhos de marcha que parecem o mesmo «avançar um tique»; só um tem os sub-passos.

**How to apply:** toda sonda/gate de simulação que mede o PRODUTO marcha pela porta do app e lê o sink com
`pump.cook.peek(sink)`; e põe um controlo que PROVA que a variável da variante chegou (bits diferentes entre
variantes que têm de diferir). Família: [[reference-topic-measurement-discipline]].
