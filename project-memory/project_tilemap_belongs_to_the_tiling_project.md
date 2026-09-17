---
name: project_tilemap_belongs_to_the_tiling_project
description: O tilemap (TOP-20 #17) NÃO se constrói na line/components — o dono mandou pular (2026-09-16); ele é do projeto separado docs/Tilling, cujo porte para Rust (Fase P) está atrás da Fase R que o dono suspendeu
metadata:
  type: project
---

Em 2026-09-16, ao chegar ao TOP-20 **#17** (`TilemapLayer` + `TileSet` + `TilemapCollider`) do
levantamento dos componentes, o dono escolheu **pular para o #18**.

**Why:** o tilemap tem um projeto próprio e gateado em `docs/Tilling/Tiling/` (fora do git por
decisão dele): Fase E (browser, features) → Fase R (referência, goldens) → **Fase P** (porte Rust:
`ph2d-tilemap`, `TilemapRef`, render por chunk, painel, tool, undo, `tilemap_c9`, import `.ldtk`,
smoke com arte real). A Fase R está **suspensa** por ordem dele (31/07, reafirmada 19/08) e a Fase E
continua a receber features (auditoria 07/09, fatias até 12/09). Construir o #17 no app seria abrir
a Fase P por fora do portão e com um segundo desenho.

**How to apply:** numa linha de componentes, o #17 fica fora da fila até o dono reabrir o portão
do Tiling. Quem precisar de tilemap lê `docs/Tilling/Tiling/research/tilemap/synthesis/plan.md`
(§0 premissas travadas, §7 fatias P1–P9) antes de propor qualquer coisa. Relacionado:
[[user_role]].
