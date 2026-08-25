---
name: one-feature-one-line-minimum-windows
description: "Enio (2026-08-25) — cada feature vive numa ÚNICA linha e no MÍNIMO de janelas; na troca, a janela nova assume a MESMA linha e a anterior PARA."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 72d4b421-a8d1-4bc2-9c35-126afd2239e6
  modified: 2026-08-25T21:03:41.285Z
---

Enio, 2026-08-25, após a jornada do quad remesh (agentes abriram várias linhas em
sequência para UMA feature, e o clean-room pedia 4 janelas encadeadas): **cada
feature = UMA linha** (`line/<módulo>`) do início ao gate — nunca uma segunda linha
para a mesma feature; e **o número de janelas é o mínimo possível** — quando um
agente passa o trabalho adiante, a janela nova **assume a MESMA linha** e a anterior
**encerra** (nunca duas janelas na mesma linha).

**Why:** várias linhas por feature multiplicam o custo de integração ao main; e a
corrente de janelas encadeadas é operacionalmente desagradável para quem cola os
prompts — o Enio quer colar UM bloco e, no máximo, um bloco de retomada.

**How to apply:** separação de papéis (ex.: E/I/R do clean-room) vira SUBAGENTES de
uma janela orquestradora, nunca janelas ou linhas extras; a troca de janela usa o
`MODELO_TROCA_DE_AGENTE_NA_LINHA.md` (e, no clean-room, o BLOCO-RETOMADA da
`SKILL_Cleanroom_Reimplementacao.md`, reescrita em 2026-08-25 sob este veredito).
Antes de propor um fluxo que abre N linhas ou N janelas para uma feature, a
pergunta é: *o que impede isto de caber numa linha e numa janela com subagentes?*
