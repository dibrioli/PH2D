---
name: project_layer_chip_no_double_click_buttons_above_collides_2026_11_08
description: "Dono 08/11 — sem duplo clique no seletor Layer do Inspector (já há o botão); «Rename layers…» e «Open in tab…» sobem para logo abaixo de Layer, acima de «Collides with»"
metadata:
  node_type: memory
  type: project
  originSessionId: 9d1a79ae-34ac-4187-b357-f3813c29209d
  modified: 2026-11-08T14:01:29.577Z
---

Dono 08/11/2026, depois do smoke do fecho da `line/GameEngineFuncional02` (cartão Collision Layers do Inspector):
1. *«Duplo clique em layer vamos retirar pois já temos o botão.»* — o seletor Layer não renomeia por duplo clique; o renomear é pelo botão «Rename layers…».
2. *«Vamos passar os botões Rename layer e Open in tab para acima de collides with e abaixo de layers.»* — ordem do cartão: Layer → [Rename layers… · Open in tab…] → Collides with → frase.

**Why:** um gesto escondido duplicava o botão; os botões ficam junto do seletor que eles servem.
**How to apply:** não reintroduzir o duplo clique no chip Layer do Inspector (a correção genérica do `editor-core`, o `DoubleClick` de listas suspensas, fica — é fundação). O duplo clique nos nomes DENTRO da aba «Collision Layers» não foi pedido a retirar. Relacionado: [[project_physics_cards_keep_all_config_power_2026_11_07]].
