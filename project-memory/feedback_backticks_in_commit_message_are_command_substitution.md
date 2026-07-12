---
name: feedback-backticks-in-commit-message-are-command-substitution
description: Backtick numa mensagem de commit entre aspas duplas vira substituição de comando (fish/zsh) e APAGA a palavra em silêncio
metadata:
  type: feedback
---

`git commit -m "…um campo \`speed\` que…"` — no fish e no zsh, **backtick dentro de aspas duplas é
substituição de comando**. O shell *executa* `speed`, o comando não existe, a saída é vazia, e a palavra
**some da mensagem**. O commit passa, o exit code é 0, e você só descobre relendo o `git log`.

Aconteceu em 2026-07-12 (`c05ca829`): "um strip que guarda um `speed` que nunca aplica" virou "um strip que
guarda um  que nunca aplica". Corrigido com `--amend -F <arquivo>`.

**Why:** o hábito de citar identificadores com crase (certo em Markdown, e o que a doc do projeto usa em
todo lugar) é exatamente o que quebra no shell — então o erro reincide justamente nas mensagens boas, as que
citam código.

**How to apply:** mensagem de commit com identificador citado → **escreva num arquivo e use
`git commit -F <arquivo>`** (ou aspas simples no `-m`). Nunca crase dentro de `-m "…"`. E **releia o
`git log -1`** depois de qualquer mensagem longa: o exit code 0 não prova que a mensagem chegou inteira —
mesma família de [[feedback-pipe-masks-script-exit-code]] (o estado é a verdade, não o status).
