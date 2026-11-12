---
name: project-controls-button-in-play-bar-2026-11-10
description: "Dono 10/11 — a tela Controls do jogo tem atalho visual: ícone de comando à direita do Stop, separador vertical, moldura própria"
metadata:
  node_type: memory
  type: project
  originSessionId: 02df03c3-d69c-4b94-957e-d5cd69283f1a
  modified: 2026-11-11T00:20:40.487Z
---

Dono 10/11 (smoke da onda 3 do Input Map): «Controls deveria ter um atalho visual. Coloque um ícone de controle ao lado de stop na barra superior, numa moldura diferente da moldura dos controles, e separado por separador vertical.»

Feito na `line/InputMap` (`a3eac373a0`): botão `TOPBAR_PLAYER_CONTROLS` com ícone `gamepad`; no Play abre/fecha a tela pelo MESMO caminho da tecla (`host::toggle` → `host::key`), aceso com a tela aberta; fora do Play desligado com o porquê, e a dica cita a tecla ATUAL do mapa (nunca F1 fixo).

**Why:** o dono não descobre o F1 sozinho; o atalho visual ensina a tecla.
**How to apply:** quem mexer na barra do Play mantém o botão separado do grupo de reprodução; a tecla mostrada vem do mapa. Ver [[project-inputmap-three-versions-2026-11-10]].
