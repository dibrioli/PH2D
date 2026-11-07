---
name: project_tilemap_rule_discovery_not_intuitive_2026_11_06
description: "Dono 06/11 — a descoberta automática de regras a partir de áreas pintadas (MVP do Tiling) não lhe agradou, pouco intuitiva e imprecisa; P10 só começa depois de ele escolher a abordagem"
metadata:
  node_type: memory
  type: project
  originSessionId: 2c3c3376-6ad2-4b4a-a943-49a7b7e91624
  modified: 2026-11-06T18:07:24.827Z
---

Dono, 2026-11-06, ao aprovar as recomendações de `docs/PixelArt/P10_plano.md` §9: o MVP do projeto Tiling descobre regras a partir de áreas já pintadas, e isso «não me agradou muito, pois me parece pouco intuitivo e impreciso». Pediu as opções usadas por profissionais e jogos de sucesso ANTES de a P10 começar.

**Why:** a palavra dele vem do uso do MVP. A medição concorda em parte: `e1-results.md` dá 53,9–62,5 % de acerto exato no mapa LDtk de referência.

**How to apply:** não propor a inferência por exemplo como caminho principal da automação de tiles. Pode reaparecer só como ajuda opcional, e só se ele pedir. **Ele escolheu em 06/11: carimbo à mão + gabarito de 47; camada de tiles só para a arte, objeto de jogo fica no Tiling; vários terrenos sem transição; iso e hex por último; PNG + JSON** (`docs/PixelArt/P10_plano.md` §9). Ver [[project_tilemap_belongs_to_the_tiling_project]].
