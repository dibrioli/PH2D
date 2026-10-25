---
name: project_game_engine_plan_owner_decisions_2026_10_25
description: Dono 25/10 — engine de jogo: nomes simples com tooltip ilustrado (Fusion 360, não o nome Godot); janela de jogo separada E em ecrã cheio; ordem = essencialidade × um jogo de plataforma médio que é o smoke
metadata:
  type: project
---

Decisões do dono (25/10) para tornar a PH2D uma engine 2D tão completa e mais fácil que a Godot
(`line/GameEngineFuncional`, estudo `docs/GameEngineFuncional/02_godot_2d_catalogo.md`):

1. Objectos com **nomes simples**; ao passar o rato, **explicação completa com ilustração** (como o Fusion 360) —
   ⛔ não o nome da Godot.
2. A janela de jogo: **as duas** — separada e dentro do canvas em ecrã cheio.
3. Começar por uma **janela de runtime rudimentar** que melhora durante a obra; depois a ordem é a
   **essencialidade** (Node antes de Node2D) e as **necessidades de um jogo de plataforma de complexidade média**,
   que é o smoke e cresce a cada onda.

**Why:** a PH2D já tem a largura (componentes); falta a organização em objectos reconhecíveis, e um jogo real
diz o que é essencial.

**How to apply:** componentes por dentro, objectos (conjuntos fixos de componentes) por fora; cada objecto entra
completo quando o jogo o pede. Plano: `docs/GameEngineFuncional/03_plano_jogo_de_plataforma.md`. Ver
[[project_modes_share_names_and_switching_object_leaves_2026_10_24]].
