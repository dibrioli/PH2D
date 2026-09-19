---
name: git-mv-stages-the-index-blob-not-the-worktree
description: `git mv` de um ficheiro com edições NÃO encenadas grava o blob do ÍNDICE (o de HEAD) no destino — o commit "tem sucesso" e a árvore dele não compila.
metadata:
  type: feedback
---

⛔⛔ **`git mv <a> <b>` renomeia a ENTRADA DO ÍNDICE.** Se `<a>` tinha edições por encenar, o
destino `<b>` fica no índice com o blob de **HEAD**, e as edições ficam por encenar em `<b>`.

Medido em 2026-09-19: um corte de tecto de LOC moveu `widget/tag.rs → widget/tag/mod.rs` com
`git mv`, e o commit levou a versão **PRÉ-corte** do ficheiro **mais** o irmão novo
`tag/geometria.rs` ⇒ **definições duplicadas: a árvore do commit não compilava**, com
`cargo check`, `clippy` e `19 738` testes verdes na árvore de TRABALHO.

⚠️ **O sinal é um `git status` que ainda diz `M <destino>` DEPOIS do commit** — é a única coisa que
o denuncia, e ele lê-se como ruído.

⇒ depois de todo `git mv`, **encene o destino outra vez** (`git add -- <b>`) e confirme com
`git diff HEAD --stat -- crates/` **vazio**: *a árvore do commit tem de ser a árvore que foi
testada, e um `git status` limpo é a única prova disso.*

⛔ E `git add -A -- <path-que-não-existe>` **aborta a chamada inteira** (`fatal: pathspec … did not
match any files`): com o `stderr` silenciado, nada foi encenado e o comando seguinte parece ter
funcionado. Irmão de [[a-commit-with-paths-never-picks-up-an-untracked-file]].
