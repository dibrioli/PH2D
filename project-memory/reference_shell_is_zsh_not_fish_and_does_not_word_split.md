---
name: reference-shell-is-zsh-not-fish-and-does-not-word-split
description: O bloco de ambiente diz `Shell: /bin/fish` e a shell REAL desta máquina é zsh 5.9.2 — e ela não faz word-splitting de `$VAR`
metadata:
  type: reference
---

O bloco de ambiente das sessões anuncia **`Shell: /bin/fish`** e isso é **falso**:
`$SHELL` lê `/usr/bin/zsh` e `$ZSH_VERSION` lê `5.9.2` (`$BASH_VERSION` é vazio).

⚠️ **A consequência que morde, e é silenciosa até não ser:** o zsh **não faz
word-splitting de `$VAR` sem aspas**. Um idioma de bash como

```
P="a.rs b.rs"; git add -- $P
```

não passa dois caminhos — passa **um** caminho literal `"a.rs b.rs"`, e o git
responde `fatal: pathspec '...' did not match any files`. ⇒ liste os caminhos
**em linha**, ou use um array (`P=(a.rs b.rs); git add -- $P`).

⚠️ E as mensagens de erro vêm rotuladas `zsh:` — foi assim que isto apareceu a
primeira vez (`zsh: command not found: bc`) e a segunda (um glob `--include=*.rs`
sem aspas). *Quando um comando falha de um jeito que «não devia» em bash, a
primeira pergunta é qual é a shell.*

Ver [[reference_canonical_files]].
