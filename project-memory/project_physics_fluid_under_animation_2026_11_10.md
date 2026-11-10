---
name: project_physics_fluid_under_animation_2026_11_10
description: Dono 10/11 — a física tem de ficar FLUIDA com limites de junta e tamanhos de forma de colisão animados; o «mínimo» de um limite é o do OBJETO
metadata:
  type: project
---

Dono 10/11, no smoke da cena `PH2D_PHYSICS_SMOKE=66` (Pin A, Auto Key no Limit min −30→−60): «a simulação não obedece à animação dos limites; parece que entra em repouso quando toca no limite»; e animar os tamanhos dos collision shapes «também dá resultados ruins» (com UMA forma só). Quer a física fluida nos dois casos.

**Why:** a curva tem de mandar a meio do Play, sem repouso preso nem salto. Medido na ponte: num pino de mundo o rapier mede o lado B relativo ao A (portão), então o limite «mínimo» do cartão encosta no lado errado e animá-lo não mexe em nada; peças filhas recriam o collider a cada tique; voltar a 0 recria o corpo. Handoff: `docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA6.md` §7.

**How to apply:** o mínimo/máximo de um limite de junta são os do OBJETO (referencial do objeto), não do rapier; reafinar collider/junta no lugar (acordando o corpo) em vez de recriar; reproduzir no APP o caso de uma forma antes de curar.

**Curado na onda 7 (smoke OK do dono, 10/11):** junta de mundo no referencial do objeto, formas reafinadas no lugar e por tique, corpo acorda ao mudar a forma (cena 66 com `PH2D_PHYSICS_SMOKE_ANIMA=1`). O caso de UMA forma não tinha defeito medido. Handoff: `docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA7.md`. **Decisões do dono 10/11 (ao ABERTO):**
- Forma de colisão: «cresce com o pivot da shape como centro se a shape sofre a transformação»; se o objeto é transformado, cresce como filho do objeto (já é assim). **How to apply:** o tamanho animado da forma cresce em volta do PIVOT dela.
- Falta mudar o pivot do collision shape: «implementar a seção Look completa do sprite para o collision shape». **How to apply:** dar à forma de colisão (ADR-0199) a secção Look COMPLETA da Sprite (Pivot e Align incluídos); inventariar a da Sprite antes de desenhar.
- Penetração na rampa: «se conseguir fazer melhor que a Godot faça, desde que não perca performance e precisão». **How to apply:** medir antes; nada que custe desempenho ou precisão.
- Width/Height só 1×/quadro: «Meça». **How to apply:** curar só se a medição mostrar diferença.
- Motor da junta na Timeline: «Corrija para graus». **How to apply:** `JointMotorTarget/Speed` em graus e no referencial do objeto (coordenar com a linha Timeline).
- Herdados do OUTLINE_E_F3 §11.5: «Corrija». **How to apply:** cada um com teste nascido vermelho. Tudo na onda 8 (handoff ONDA7 §7).
