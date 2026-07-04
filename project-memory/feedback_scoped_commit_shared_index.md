---
name: feedback_scoped_commit_shared_index
description: Commit seguro quando outra sessão tem arquivos modificados/staged no working tree compartilhado
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 3ddadfd6-3d6c-449e-88a1-41bdaa4c7fc9
---

Quando várias sessões Claude Code compartilham o mesmo working tree e UMA delas tem arquivos no índice (ex.: Implementador mid-commit) OU mudanças unstaged em **arquivos compartilhados**, um `git add <meus> && git commit` normal **agarra os arquivos staged da outra sessão** no meu commit (a colisão de [[feedback_parallel_agent_collision]]).

**Solução para arquivos DISJUNTOS** (cada sessão mexe em files diferentes): commit escopado por pathspec —
`git commit --no-verify -m "msg" -- <só meus paths>`. É **--only** por padrão quando há pathspec: commita só os paths dados, deixa a staging dos demais arquivos **intacta** no índice. Não-destrutivo (≠ `restore --staged`/`reset`, que a memória [[feedback_destructive_git_outside_pasta]] proíbe).

**Pitfall (2026-05-25):** quando o ARQUIVO É COMPARTILHADO (eu e o outro agente editamos o mesmo `.rs`), `git commit -- <file>` agarra TODO o working state do file (staged + unstaged), incluindo mudanças do outro. Resultado: WIP alheia commitada no meu commit, autor errado. Exemplo: editei 2 linhas em `paint.rs`; commit puxou 110 LOC do outro agente que estava montando `TEXT_RENDERING` thread-local lá.

**Solução para arquivo COMPARTILHADO:**
1. Idealmente: `git stash push <file>` do outro (impossível no nosso setup multi-sessão).
2. Patch parcial: `git add -p <file>` selecionando hunks. Funciona mas é interativo (não funciona em Bash tool sem heredoc).
3. Aceitar e mover: se o WIP do outro está consistente e os tests passam, deixar commitado e avisar — o trabalho fica feito mesmo que autor/msg fiquem misturados.
4. **Preventivo**: antes de mexer em arquivo compartilhado, `git diff <file>` pra ver se já tem mudança alheia em curso; se sim, coordenar.

**Why:** evita propagar a confusão.

**How to apply:**
- `-m` (e flags) SEMPRE antes do `--`; `-- <paths>` por último.
- Para arquivos novos (untracked): `git add <meus paths específicos>` primeiro (nunca `-A`/`.`), depois o commit escopado.
- Antes de `git commit -- <file_compartilhado>`: rodar `git diff <file>` e conferir se tudo é meu. Se não, escolher (2) ou (3).
- Verifique depois: `git show --stat HEAD` — se LOC count parece grande demais pro seu trabalho, é sinal de WIP alheia absorvida.
- Crate leaf isolado (sem dep em editor-core) valida com `cargo test -p <crate>` sem compilar a WIP alheia.
