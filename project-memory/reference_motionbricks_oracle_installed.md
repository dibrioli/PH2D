---
name: reference-motionbricks-oracle-installed
description: "NVIDIA MotionBricks instalado como oráculo em ~/Documentos/Projetos/ph2d-motionbricks — código Apache-2.0 (porta ABERTA, não é zona contaminada), corre headless, 4 clips no corpus"
metadata:
  type: reference
---

**NVIDIA MotionBricks** (NVlabs/GR00T-WholeBodyControl, SIGGRAPH 2026) está instalado e **a correr**
em `~/Documentos/Projetos/ph2d-motionbricks/` — **fora** da árvore do PH2D, como a `ph2d-quadbench`
(precedente ADR-0162). Instalado em 2026-09-15.

⭐⭐ **O CÓDIGO é Apache-2.0** ⇒ ler e portar é legal, com atribuição. ⛔ **Por isso ele NÃO vive em
`~/Referencias/`**, que é a zona contaminada e proíbe o papel I de entrar — pô-lo lá pagaria uma
parede que não existe. Os **pesos** são NVIDIA Open Model License (comercial OK, com atribuição);
o corpus **BONES-SEED** tem licença própria **com portão** e **não foi descarregado**.

⚠️ O repo-pai declara-se **`NOASSERTION`** no GitHub — quem triar pelo nome lê *fechado*. A licença
por partes está no `LICENSE` da raiz. *A unidade da triagem é o artefacto* (CLAUDE.md §0.9).

- **Corre sem interface:** `--has_viewer 0 --controller random` — zero X11, zero `pynput`.
  Harness: `oracle_run.py` (grava CSV com cabeçalho). Ambiente: `uv` + CPython 3.12 + torch
  2.14.0+cu130 na RTX 5060 Ti (`sm_120`).
- **Medido nesta máquina:** uma chamada custa **18,2 ms p50** (o modelo é 65 % disso, 11,9 ms) e
  corre **uma vez a cada 267 ms** ⇒ **ocupação 7 %**. Débito num fluxo: **2 418 quadros/s**.
  ⚠️ Os «15 000 FPS / 2 ms» do comunicado são débito em LOTE noutra máquina — outra grandeza.
- ⛔ **`--allowed_mode` compara com `in`:** uma **string** casa por SUBSTRING (pedir `walk_zombie`
  deixa passar `walk` e mistura dois estilos em silêncio). Passe **lista**.

O doc que manda — triagem, arquitectura lida no código, as 6 lições para o PH2D e as armadilhas —
é **`docs/_ComoInvestigarApps/02_MotionBricks.md`**, apontado pela tabela do `01_o_arsenal.md`.
A lição mais barata está na §4.1: a `ph2d-spring` tem a lei certa (`ζ = 1` é o default) com o
rótulo errado — `halflife = 2·ln2/ω` ⇒ o default `ω = 12` é **115,5 ms**.
Ver [[reference_topic_measurement_discipline]].
