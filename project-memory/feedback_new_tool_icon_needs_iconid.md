---
name: feedback_new_tool_icon_needs_iconid
description: "Toda Image Tool nova com SVG em docs/design/icons/ precisa de um IconId variant em ordem alfabética, senão TODOS os ícones quebram"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 3ddadfd6-3d6c-449e-88a1-41bdaa4c7fc9
---

Ao adicionar uma Image Tool nova, criar `docs/design/icons/<slug>.svg` (exigido pelo gate `tool_manifest_design_sync`) **obriga** a adicionar o variant `IconId::<Name>` correspondente em `crates/ph2d-editor-core/src/icons.rs` — em **ordem alfabética por filename do SVG** — E na const de teste `ALL_ICONS`.

**Why:** o `build.rs` do editor-core gera `ICON_CMDS_BY_ID` lendo `docs/design/icons/*.svg` **sorted (byte order / LC_ALL=C)**, e `IconId::cmds()` indexa essa tabela por **discriminante do enum** (`self as usize`). Adicionar um SVG sem o IconId insere uma entrada no meio da tabela e **desloca o índice de todos os ícones alfabeticamente posteriores** → o app inteiro renderiza ícones errados. O gate `icons::tests::enum_order_matches_svgs` (count + ordem enum==svgs) trava isso — mas só roda no hook/CI, então commits `--no-verify` (fast-mode) passam por cima e quebram main. `./scripts/ship.sh` pega antes do push.

**How to apply:**
- No briefing de tool nova: incluir "adicione `IconId::<Name>` em ordem alfabética + entrada em `ALL_ICONS`" junto do "crie o SVG". (Foi o gap que quebrou os ícones com padding+real-size em 2026-05-20, fix `874fd34`.)
- Posição = onde o filename do SVG cai num `ls docs/design/icons/*.svg | LC_ALL=C sort` (ex.: `padding` entre `open`/`palette`; `real-size` entre `prefab`/`redo`).
- Convenção: make_square/trim/bgremoval já têm SVG + IconId pareados. Ver [[feedback_scoped_commit_shared_index]] p/ commitar fix em tree compartilhado.
