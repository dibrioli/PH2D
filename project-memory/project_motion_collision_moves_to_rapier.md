---
name: project-motion-collision-moves-to-rapier
description: "DECISÃO DO DONO 06/10: a colisão das formas do Motion passa a usar o rapier2d (o motor da Física), no lugar do PBD próprio — o oráculo mediu ~20-30× por algoritmo"
metadata:
  type: project
---

Em 06/10 o dono, depois do smoke da `=114` (*«estamos muito aquém da performance de uma Unity»*), escolheu **usar o
motor da casa** (`rapier2d`, o da Física, ADR-0131) para o contacto entre as peças do Motion com `Collide`. O
oráculo (doc 121 §9.19; `docs/Motion Nodes/ferramentas/oraculo_rapier_pilha/`): a mesma pilha `0,40` ms por tique a
`1 024` caixas no rapier contra `~15` no nosso PBD, `3,0` (`1,7` em paralelo) contra `48`–`93` a `4 096` — o nosso
refaz cada contacto `64×` por tique e não guarda nada entre tiques.

**Why:** a diferença é de ALGORITMO; levar o PBD para a placa (o plano de 05/10) portaria o desperdício.

**How to apply:** a próxima onda segue o prompt `HANDOFF_CONTINUACAO_line_motion_value_2026-10-06_CONTACTO_RAPIER.md`;
o comportamento aprovado da pilha (os gates de `motion_state_pilha_demo_tests.rs`) é o critério, e o desvio vai ao
dono no smoke. Não reabra o «contacto PBD na placa» sem reler isto. Liga com
[[feedback_a_refusal_measured_on_the_cheap_half_of_a_step_renews_falsely]].
