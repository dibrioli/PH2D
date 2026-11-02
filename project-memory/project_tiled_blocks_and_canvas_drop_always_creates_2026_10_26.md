---
name: project_tiled_blocks_and_canvas_drop_always_creates_2026_10_26
description: "Dono 26/10 (smoke GEF 13): imagem largada no canvas SEMPRE cria objeto; blocos que repetem = modo Tiled com Largura/Altura e Scale 1, nunca escala + Sampling"
metadata:
  node_type: memory
  type: project
  originSessionId: 313ab8d4-937f-477c-9ee6-4353d54a60f7
  modified: 2026-10-26T22:25:03.704Z
---

Duas decisões do dono no smoke da onda 13 do GameEngineFuncional (26/10):

1. **Largar uma imagem na tela SEMPRE cria um objeto novo** (como a Unity). Trocar a imagem de um objeto
   existente só pela caixa/ranhura Image do Inspector. Feito em `6c3e8b1011`.
2. **Bloco/chão/plataforma que repete = «Tiled com tamanho»** (Draw Mode Tiled da Unity): Scale fica 1, o
   objeto tem Largura/Altura e a imagem repete sozinha. Recusado: o desenho antigo (Transform.scale
   0.8,1 + Sampling/UvTransform a repetir) — dois controles que têm de combinar.

**Why:** o dono viu o Platform largado com Scale (40,16) (caiu no céu, que cobre a vista) e a Platform Low
com Scale (0.8,1) e Sampling mexido, e perguntou «porque tanta complexidade?».

**How to apply:** um Platform largado e a Platform Low têm de ser o MESMO tipo de objeto, só com largura
diferente. Ficheiros antigos (escala + UV) abrem iguais. Ligado a [[project_isolation_by_use_and_atlas_strategy_stays_on_object_2026_10_26]].
