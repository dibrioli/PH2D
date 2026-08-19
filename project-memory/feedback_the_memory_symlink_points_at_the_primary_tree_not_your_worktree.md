---
name: feedback_the_memory_symlink_points_at_the_primary_tree_not_your_worktree
description: Modo L — salvar memória pelo caminho do Claude Code escreve no `main`; a linha tem de escrever no `project-memory/` da PRÓPRIA worktree
metadata:
  type: feedback
---

`~/.claude/projects/<key>/memory` é um symlink para
`/home/enio/Documentos/Projetos/PH2D/project-memory` — **a árvore PRIMÁRIA**, que
está em `main`. Ele não sabe que existem worktrees.

⇒ Um agente de linha que salve memória pelo caminho do Claude Code deposita um
arquivo **não rastreado dentro do `main`**, fora do seu diff, fora do seu commit
e fora do seu handoff.

**Why:** é a mesma armadilha que o
[[MODELO_TROCA_DE_AGENTE_NA_LINHA]] descreve para o código — *o mesmo caminho
existe nas duas árvores, e escrever na errada não levanta erro nenhum* —, só que
aqui nem o `pwd` protege: o caminho é absoluto e resolve para o primário mesmo
com a worktree correta como cwd. A falha é silenciosa e só aparece quando alguém
faz `git status` no primário e encontra um `??` de dono desconhecido, ou quando a
memória some porque ninguém a commitou. Ver
[[feedback_bash_cwd_resets_and_slips_to_the_primary]].

**How to apply:** em Modo L, escreva a memória em
`Worktrees/line-<módulo>/project-memory/` e acrescente a linha no `MEMORY.md`
**dessa** árvore — assim ela viaja no commit da linha e chega ao `main` pela
integração. ⚠️ `MEMORY.md` é lista compartilhada: **só ACRESCENTE**
([[feedback_a_shared_list_is_merged_against_todays_main]]).
