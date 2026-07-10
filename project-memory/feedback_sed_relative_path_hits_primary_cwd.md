---
name: feedback_sed_relative_path_hits_primary_cwd
description: "`cd <dir> && sed -i file` pode rodar no primary working directory — mutação por caminho relativo atinge o repo errado (Modo L: o main, não o worktree)"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6d3039ad-668d-4133-a295-f69680a93752
---

O `cd` dentro de um comando composto do Bash **pode não ter efeito** (o sandbox roda a partir do *primary working directory*). Um `sed -i`/`>`/`mv` com **caminho relativo** então grava no repo ERRADO. Em Modo L isso significa: você acha que está no worktree da sua linha e escreve no `main`.

Aconteceu 2026-07-09 (linha `line/audio`): `cd crates/ph2d-panel-audio-editor/src && sed -i 's/HitIndex/ClippedHits/' paint.rs` editou `/home/enio/Documentos/Projetos/PH2D/crates/.../paint.rs` (o main). A **pista** foi o `ugrep: warning: paint_fx.rs: No such file` no mesmo comando — `paint_fx.rs` só existe na minha linha, então o cwd não era o worktree. Um `pwd` isolado depois mentiu (mostrou o worktree).

**Why:** `cd` em compound command é intercept-ável; e um `sed -i` não falha quando o caminho relativo *casualmente existe* no outro repo — ele só corrompe silenciosamente. `git status` do worktree fica limpo, então nada avisa.

**How to apply:** (1) toda mutação de arquivo por shell usa **caminho absoluto**, nunca relativo + `cd`; (2) prefira as ferramentas `Edit`/`Write` (exigem path absoluto por contrato); (3) se um comando reclamar de um arquivo que você SABE que existe na sua linha, pare — o cwd está errado, não o arquivo; (4) rode `git -C <main> status` ao suspeitar, e reverta só o que você sujou (`git checkout -- <arquivo>` depois de conferir o diff). Vide [[feedback_run_command_include_cd]] e [[feedback_destructive_git_outside_pasta]].
