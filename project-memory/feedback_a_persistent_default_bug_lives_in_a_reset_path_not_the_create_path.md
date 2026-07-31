---
name: feedback_a_persistent_default_bug_lives_in_a_reset_path_not_the_create_path
description: "Um bug de \"valor padrão errado\" que sobrevive a consertos no CREATE está num caminho de RESET/purge, não na criação — enumere TODA porta que reconstrói o estado."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 299e185e-eeb8-4b7e-8d3a-858f1557aa9f
  modified: 2026-07-29T00:16:03.326Z
---

Quando o Enio reporta várias vezes que um DEFAULT abre errado (ex.: a timeline abrindo com Dur
∞ em vez de 4 s + véu) e cada conserto no caminho de CRIAÇÃO não resolve, o vazamento está num
caminho de **RESET/reconstrução do estado**, não na criação.

Caso real (2026-07-28, `line/anim`): gastei ROUNDS consertando `AddClip`/boot/load e provando
headless que "app novo + Autokey = 4 s" — e o bug persistia. O gatilho verdadeiro era **deletar
o último objeto animado**: `timeline_persist::purge_the_dead` resetava `timeline.doc =
TimelineDoc::new()` (DERIVADO). Toda fixture minha usava criação, nunca deleção, então minha
"prova" era verde sobre o caminho errado.

**Why:** um estado padrão nasce em N portas — `new`/`default`, boot, criar, duplicar, LOAD, e os
**resets** (purge de bindings mortas, "cena vazia", troca de projeto, undo). Fixar a criação
deixa os resets intactos, e um deles reconstrói o estado derivado/zerado.

**How to apply:** ao ouvir "o padrão abre errado" que resiste a consertos, **enumere e grepe TODA
porta que produz uma instância FRESCA/RESETADA** (`= TypeState::new()`, `= Type::default()`,
`.doc = Doc::new()`, purge, clear, reset, install/load) — não só a criação. Reproduza no gatilho
que o usuário deu (aqui: DELETAR), não num espelho do caminho que você suspeita. Corolário do
[[reference_topic_repro_discipline]]: uma fixture que não contém o gatilho é verde sobre nada.
Ver também [[feedback_derived_coordinate_seed_must_match_sample]] (a mesma família: o estado
derivado tem de casar com o autorado).
