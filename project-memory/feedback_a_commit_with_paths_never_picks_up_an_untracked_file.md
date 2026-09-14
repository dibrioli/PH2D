---
name: feedback_a_commit_with_paths_never_picks_up_an_untracked_file
description: `git commit -- <paths>` NÃO apanha ficheiros por rastrear debaixo daqueles caminhos — o commit «tem sucesso» e a árvore dele NÃO COMPILA.
metadata:
  type: feedback
---

Medido em 2026-09-14 (`line/components`, W4a das Tags). O commit correu
`git commit --no-verify -m "…" -- crates/… shells/desktop` e disse **54 ficheiros**. Os **11
ficheiros NOVOS** daquela wave — o módulo de vocabulário, os ids, o handler de chrome, a fase-filha,
o painel, os quatro irmãos cortados por tecto de LOC — ficaram de fora: a árvore daquele commit não
tinha `tags_edits.rs`, e **não compila**. Nada falhou alto.

**Why:** o `-- <paths>` do `git commit` é um filtro sobre o que o commit *inclui*, aplicado às
mudanças **rastreadas** (o mesmo `pathspec` do `git add -u`). Um ficheiro `??` não é uma mudança
rastreada; ele só entra se alguém o **stage** antes. ⚠️ E a regra §0.4 do `CLAUDE.md` — *«nunca
`git add .`»* — empurra exactamente para esta forma, porque ela é a segura contra ficheiro alheio.

**How to apply:** antes de comitar com `-- <paths>`, **`git status --short | grep '^??'`** e faça
`git add --` explícito de cada ficheiro novo. ⭐ E a prova barata de que o commit está inteiro é
`git status --short | grep '^??'` DEPOIS dele: se ainda sobrar um ficheiro da wave, ele ficou de
fora. ⛔ Um commit que não compila é um ponto de `git bisect` partido e um CI por-commit vermelho —
a cura é `--amend` enquanto ele for `HEAD`. Irmãs: [[reference_topic_git_hazards]] ·
[[feedback_a_tail_is_a_window_not_a_verdict]]
