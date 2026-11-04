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

**How to apply:** não proponha regerar tabelas de 40 tamanhos nem teste de largura de horas. Em vigor desde
03/11 e aprovado no smoke do dono («muito sucesso»): comandos, relatório e aprovação em `docs/DevOps/PLANO_ACELERACAO_2026-11.md`
§«Como se usa». A reta de P1 passou só em 34/55: não substitui as tabelas gravadas.
Ver [[feedback-brush-method-owner-triage-and-simulate-before-building]].
