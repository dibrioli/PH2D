---
name: feedback-a-simulation-probe-that-does-not-advance-reads-the-state
description: "Sonda de simulação que só chama o cook e não o advance_tick mede a LEITURA do estado — 0,02 ms por tique a 2 048 peças"
metadata:
  type: feedback
---

A 1.ª redacção de `custo_do_cozimento_da_pilha` (doc 121 §9.19 (5)) cronometrava `pump.cook.cook(…, t)` por tique
e lia `0,02` ms a `2 048` peças com colisão — enquanto o app pagava `151` ms. O `cook` só LÊ o estado da zona; quem
avança a simulação (a integração, a separação, os impulsos) é o `advance_tick`. Com ele dentro do relógio: `1,04` ms.

**Why:** a função com o nome do trabalho (*cozer*) não era a que fazia o trabalho; e um número absurdamente baixo
não alarmou por vir de uma sonda nova.

**How to apply:** numa sonda de um laço com estado, imite o laço do PRODUTO inteiro (o gate vizinho — aqui o
`corre` dos testes da pilha — mostra-o) e ponha um CONTROLO de ordem de grandeza (o número tem de reagir à lei:
`Collide` ON ≫ OFF). Família [[reference_topic_measurement_discipline]]; irmã de
[[feedback_an_ablation_that_stops_storing_the_walk_measures_less_than_the_walk]].
