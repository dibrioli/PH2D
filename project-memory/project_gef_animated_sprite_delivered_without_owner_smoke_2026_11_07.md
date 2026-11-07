---
name: project_gef_animated_sprite_delivered_without_owner_smoke_2026_11_07
description: "07/11: a linha GameEngineFuncional entregou Sprite + Animated Sprite ao integrador SEM smoke do dono do Animated; smoke dele amanhã, depois da integração"
metadata:
  type: project
---

Linha `line/GameEngineFuncional` fechada em 07/11 (HEAD `59ad2121e3`, 55 commits sobre `bbfefaf393`, nada no `main`, nenhum push). Entra a Sprite (smoke OK em 06/11) e o Animated Sprite (GEF 24, E0–E6) **sem smoke do dono do Animated**: o dono está fora e testa `PH2D_GAME_SMOKE=5` e `=4` amanhã, com o binário do `main` depois da integração.

**Why:** o dono decidiu entregar já e smokar depois; as seis decisões delegadas (▶ armado, aviso no save, «N components», censo 17/48, Frames/s sem chave, resumo do Look) esperam o veredito dele.

**How to apply:** o integrador lê `docs/GameEngineFuncional/handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-07_ANIMATED_SPRITE.md`; não marque a ficha `fechado` antes do veredito. Integrar não é aprovar.
