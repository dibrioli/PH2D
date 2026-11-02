---
name: project-pixel-palette-reorder-like-aseprite-2026-11-01
description: "Dono 01/11 — no Pixel, reordenar/ordenar/inverter a paleta indexada NÃO remapeia; aparece «Remap Palette» (2.º passo), como o Aseprite"
metadata:
  node_type: memory
  type: project
  originSessionId: f17ea438-bccf-446a-93f7-516bedd7722e
  modified: 2026-11-01T17:06:01.140Z
---

Dono, 01/11/2026 (linha PixelArt, P6k): no sprite INDEXADO, arrastar uma cor na paleta, ordenar ou
inverter as cores muda só a paleta — os pixels guardam o índice, o desenho muda de cor — e aparece o
botão «Remap Palette», que remapeia os pixels num SEGUNDO passo do desfazer. Revoga a lei do MVP/P2
«reordenar a paleta nunca muda a imagem» (`02_plano_modulo.md` §P2). Medido no Trial:
`docs/PixelArt/ferramentas/oraculo_paleta.md`.

**Why:** o Aseprite é a espinha do módulo ([[project-pixel-art-module-standalone-aseprite-spine-2026-10-25]]);
permite trocar as cores de propósito.

**How to apply:** RGBA não muda (só a paleta, sem botão). Ctrl+arrastar copia e insere. Não volte a
fundir reordenar+remap num passo.
