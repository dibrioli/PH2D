---
name: project_physics_fluid_under_animation_2026_11_10
description: Dono 10/11 — a física tem de ficar FLUIDA com limites de junta e tamanhos de forma de colisão animados; o «mínimo» de um limite é o do OBJETO
metadata:
  type: project
---

Dono 10/11, no smoke da cena `PH2D_PHYSICS_SMOKE=66` (Pin A, Auto Key no Limit min −30→−60): «a simulação não obedece à animação dos limites; parece que entra em repouso quando toca no limite»; e animar os tamanhos dos collision shapes «também dá resultados ruins» (com UMA forma só). Quer a física fluida nos dois casos.

**Why:** a curva tem de mandar a meio do Play, sem repouso preso nem salto. Medido na ponte: num pino de mundo o rapier mede o lado B relativo ao A (portão), então o limite «mínimo» do cartão encosta no lado errado e animá-lo não mexe em nada; peças filhas recriam o collider a cada tique; voltar a 0 recria o corpo. Handoff: `docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA6.md` §7.

**How to apply:** o mínimo/máximo de um limite de junta são os do OBJETO (referencial do objeto), não do rapier; reafinar collider/junta no lugar (acordando o corpo) em vez de recriar; reproduzir no APP o caso de uma forma antes de curar.
