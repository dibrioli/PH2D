---
name: feedback-bash-cwd-resets-and-slips-to-the-primary
description: "Modo L: a cwd do Bash volta ao repo primário entre turnos, e um `cd primário && ...` a move para o resto da sessão — prefixe TODO comando com o cd da worktree"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 4d33a1a5-4cde-45a3-94d1-ae7125cd56bf
  modified: 2026-07-22T20:56:10.395Z
---

No Modo L, a cwd da ferramenta Bash **não é estável**: ela volta ao **repo primário** (`~/Documentos/Projetos/PH2D`, que está em `main`) entre turnos do usuário, e qualquer comando composto que termine com `cd <primário> && ...` (ex.: conferir `git status` do primário) a **deixa lá para todos os comandos seguintes daquele turno**.

**Why:** o mesmo path relativo existe nas DUAS árvores, então o comando errado **não falha** — ele lê/edita a árvore errada em silêncio. É a armadilha nº 1 do Modo L ([[project_multiagent_modo_l_2026_07_05]]), e ela morde a LLM, não só o Enio (o [[feedback_run_command_include_cd]] cobre o outro lado: comandos entregues ao Enio).

Como ela se revela (se revelar): `cargo test --test <alvo>` responde **"no test target named ..."** para um arquivo que você acabou de criar — porque o alvo existe na worktree e você está no primário.

**How to apply:** prefixe **todo** comando com `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-<módulo> && ...`, e nunca termine um comando composto com um `cd` para outro lugar — rode o que for do primário com `git -C <primário>` em vez de `cd`. Um `pwd` no começo de qualquer bloco que vá EDITAR é barato e é a única confirmação que vale.
