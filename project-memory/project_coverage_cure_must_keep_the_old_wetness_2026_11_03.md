---
name: project-coverage-cure-must-keep-the-old-wetness-2026-11-03
description: Dono 03/11 — manter a cura da cobertura da orla (62e7fc8505) MAS devolver o molhado antigo do Wet Round/Wet Drip (espalhar e gotas); Blend pinta área maior que o círculo; Smear e pincéis de pintura OK no cursor.
metadata:
  type: project
---

Smoke do dono 03/11 (pintando, versão nova × cópia antiga 6cbcce804c): Wet Round e Wet Drip novos ficaram «menos molhados,
espalham menos, menos gotas escorrem» — ainda bons. Cursor × traço OK em todos os pincéis de pintura e no Smear; nos
**Blend a área afetada é maior que o círculo**. Pediu: manter a correção E ter o efeito antigo.

**Why:** a cura `round(byte·cobertura)` da orla tira água das células de borda; o dono quer a física certa sem perder o
caráter aguado dos pincéis.

**How to apply:** a cura deve CONSERVAR a água do dab (redistribuir, não tirar) — medir a água total antes × depois,
compensar no motor, mostrar pelo relatório do alarme e pedir novo smoke. A régua da largura «equivalente» lê o miolo e
diz «mais fino» onde o olho do dono diz OK: o olho manda ([[project-brush-tests-cheap-alarm-owner-decides-2026-11-03]]).
