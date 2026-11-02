---
name: project_pixel_brushes_live_on_the_computer_2026_11_02
description: "Dono 02/11 (P7) — os pincéis guardados do Pixel Art (inclui os de Ctrl+B) ficam NO COMPUTADOR, como no Aseprite, valem em todo projeto; não entram no projeto"
metadata:
  node_type: memory
  type: project
  originSessionId: 52c953e3-a827-47c5-aea1-d4c53c5a6806
  modified: 2026-11-02T13:58:17.981Z
---

Dono, 2026-11-02, ao aprovar o plano da P7 (`docs/PixelArt/P7_plano.md`): a lista de pincéis do Pixel Art fica **no computador** (`~/.ph2d/pixel_brushes/`, como `~/.ph2d/palettes.txt`), não dentro do `.ph2dproj`. Escolheu isto contra «no projeto» e «os dois».

**Why:** é o idioma do Aseprite (referência do dono) e não muda o formato do projeto (`PROJECT_SCHEMA` intocado).

**How to apply:** `ph2d-pixel-io::brush_file` + pasta do utilizador; nada no blob `pixel_sprites`. Mandar um projeto a outra pessoa não leva os pincéis — se o dono pedir isso depois, é decisão nova (degrau de schema). Ver [[project_pixel_art_module_standalone_aseprite_spine_2026_10_25]].
