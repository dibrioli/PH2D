---
name: project_oleo_real_mission_w2_2026_10_27
description: "Óleo Real: W1c e W2 aprovadas no smoke 27/10 (backups em tag); dono decidiu o material = Principled SIMPLIFICADO com nomes do Blender (8 controlos, com SSS), no pincel E no cartão; MISSÃO W3 autónoma = material + motor + os 13 presets que faltam, testando com o Rebelle"
metadata:
  node_type: memory
  type: project
  originSessionId: 816b225d-8f3d-4d28-bfd7-7e82b95b71bc
  modified: 2026-10-27T16:19:28.781Z
---

Smoke W1c aprovado em 27/10 (*«Realmente muito bom»*) e W2 aprovada no mesmo dia (*«MUito bom!»*). Backups:
tags `backup/oleo-real-W1c-2026-10-27` e `backup/oleo-real-W2-2026-10-27` + `~/Documentos/Projetos/PH2D-backups/`.

**Decisão do dono (27/10, depois do smoke W2):** o material da tinta usa os NOMES e as propriedades universais do
Principled BSDF do Blender, SIMPLIFICADO, COM SSS — exactamente 8 controlos: **Metallic · Roughness · IOR Level ·
Coat · Coat Roughness · Sheen · Subsurface · Subsurface Radius**. Saem Spec tint, Anisotropy, Translucency (o SSS a
substitui) e os nomes do Rebelle Reflectivity/Map scale/Map strength (o mapa fica no seletor Environment). Os
controlos vivem no PINCEL (material por célula) E no cartão Real Oil (ajuste da tela inteira), com os mesmos nomes.
Depois: o motor que falta e os 13 presets não aprovados, automático, testando com o Rebelle.

**Why:** o dono quer um material que qualquer artista de 3D reconhece, com poucos botões, e a tinta luminosa (SSS).

**How to apply:** missão W3 em `docs/Painter/49_realshader/PROMPT_MISSAO_W3_material_motor_presets.md`
(line/PainterWatercolor); a W2 em `ESTADO_MISSAO_W2.md`. Ver [[project_the_worlds_best_brushes_mission_2026_10_19]],
[[feedback_brush_mission_cost_budget]], [[feedback_an_autonomous_mission_must_never_stop_on_a_permission_prompt]].
