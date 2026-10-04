---
name: feedback_a_route_census_counts_the_cook_not_the_draw
description: "«19 de 23 cenas com forma vão à placa» contava a rota do COZIMENTO; as 4 «de fora» já DESENHAVAM as formas pela placa — leia o [formas] do app antes de curar uma cena por um censo"
metadata:
  type: feedback
---

Doc 121 §9.9 e o handoff de 03/10 nomeavam 4 cenas (`=70`, `=114`, `=115`, `=120`) como «formas que
ainda não vão à placa», a partir do `motion_route_census` (que pergunta o `gpu_route`, o COZIMENTO).
Medido em 04/10 numa tela virtual (`fotografa_cena.sh` com `PH2D_MOTION_ROUTE_LOG=1`): as QUATRO
imprimem `[formas] … pela PLACA` — a rota da CPU do passe de formas desenha-as. O que ficava na CPU
era o cozimento (contacto do colisor, uma cena sem estágio que despache, o `fx.glow`), e só a `=70`
escondia um defeito real (o halo lia as listas velhas da CPU num quadro do dispositivo).

**Why:** o censo responde «quem COZE», a frase do item dizia «quem DESENHA»; a mesma palavra
(«placa») nomeava duas rotas. Curar o item pela frase teria mandado portar contacto e IK para a placa.

**How to apply:** antes de abrir trabalho a partir de um censo, corra a cena e leia as DUAS linhas
(`[motion-route]` e `[formas]`) — ~1 min por cena na tela virtual. Ver
[[feedback_a_frontier_is_not_a_census]] · [[reference_topic_measurement_discipline]].
