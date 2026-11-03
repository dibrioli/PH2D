---
name: project-brush-tests-cheap-alarm-owner-decides-2026-11-03
description: Dono 03/11 — testes de pincel = alarme automático BARATO (segundos) que só aponta o que mudou; o DONO decide a olho (gizmo × traço) e a aprovação dele vira a referência. Fim das tabelas de largura de 40 tamanhos.
metadata:
  type: project
---

Decisão do dono (03/11/2026, rodada 2026-11-02c): os testes lentos de pincel (tabela de largura com
física completa em ~40 tamanhos até 600 px, 20–40 min e 26–31 GB por pincel) saem do caminho. No lugar:
um **alarme automático barato** (poucos tamanhos, traço pequeno, segundos) que só diz «o pincel X mudou»,
e o **dono decide** pelo protocolo manual dele (compara o gizmo com o traço). O que ele aprova vira a
nova referência (modelo «blessing» do Blender/Skia Gold/`cargo insta review`).

**Why:** a rodada 02/11c levou >36 h, a maior parte a regerar tabelas de largura; a pesquisa
(`/home/enio/PESQUISA_DESENVOLVIMENTO_RAPIDO_2026-11-03.md`) não achou NENHUM app (Krita, Painter,
Rebelle, Procreate, Fresco) nem artigo que calibre o cursor pela tinta espalhada — todos desenham a
pegada. O olho sozinho deixa passar deriva subtil, por isso o alarme fica.

**How to apply:** não proponha regerar tabelas de 40 tamanhos nem teste de largura de horas; o próximo
passo técnico é o P1/P2 do anexo da pesquisa (ajustar a reta contra as tabelas existentes; teste rápido
3–4 tamanhos com prova de mutação; relatório antes/depois/diferença só dos pincéis que mudaram).
Ver [[feedback-brush-method-owner-triage-and-simulate-before-building]].
