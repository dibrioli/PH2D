---
name: feedback_brush_mission_cost_budget
description: "Dono (21/10, missão Pincéis J4): «muito gasto e pouco resultado» — agentes de família com ~600–950 mil tokens para 0–3 pincéis; regras de custo para todo agente"
metadata:
  node_type: memory
  type: feedback
  originSessionId: e711a509-054f-45da-96e5-b21f29624544
  modified: 2026-10-22T01:24:08.020Z
---

O dono viu a J4 da missão Pincéis gastar muito (cada agente de família 560–950 mil tokens, rondas de cura e revisão
cega até 6×, grelhas de 20 variantes) para 0–3 pincéis aprovados por agente, e pediu para reduzir gastos.

**Why:** o custo vinha das rondas repetidas sobre o MESMO pincel (cura → revisor novo → cura…) e de 5 agentes
paralelos a disputar 2 vagas pesadas; o rendimento marginal da 3.ª+ ronda era quase zero.

**How to apply:** em missões de agentes, dar no briefing: no máximo 2 rondas de cura por item, revisor/verificador
com modelo mais barato (`model: "sonnet"`) e resposta curta, só o 1.º item de motor (o resto vira fila), ≤ 4
variantes por cura, ≤ 2–3 agentes em paralelo, devolver assim que o item fecha. Texto em
`docs/Painter/Pinceis/` (REGRAS_DE_CUSTO). Ver [[project_the_worlds_best_brushes_mission_2026_10_19]].
