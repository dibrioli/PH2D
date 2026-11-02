---
name: feedback_filtered_hunk_commits_by_many_agents_misplace_lines
description: "Vários agentes a comitar SÓ os seus hunks (git apply --cached) no MESMO ficheiro deixaram o HEAD sem compilar enquanto a árvore de trabalho compilava (20/10, Pincéis J2)"
metadata:
  node_type: memory
  type: feedback
  originSessionId: eeca69b4-6105-48de-8fb4-34b27bea6d50
  modified: 2026-10-20T15:10:53.037Z
---

Na missão Pincéis J2 (20/10), 4–6 agentes na MESMA worktree editaram `wetpaint/dab_route.rs` e `lote.rs` e cada
um comitou só os seus hunks por patch filtrado (`git apply --cached`). Os hunks aplicados contra um índice que já
divergia da árvore caíram no sítio ERRADO: um `let` antes do `.collect()`, um `.then(..)` depois de código alheio,
um `mod tests` dentro de um doc-comment. A árvore de trabalho estava certa e todos os portões corriam verdes nela —
só o HEAD não compilava, e o «M» que sobrou parecia «só linhas movidas».

**Why:** os portões e o `cargo check` correm sobre a ÁRVORE, nunca sobre o que foi comitado; um HEAD quebrado passa
despercebido até ao integrador ou à janela seguinte.

**How to apply:** quando vários agentes partilham ficheiros, no FIM da onda o coordenador corre `git status` e
`git diff` — um «M» residual de «linhas movidas» é suspeito: compare com a árvore que compila e comite a árvore.
Melhor ainda: depois de comitar por patch filtrado, `git diff HEAD -- <ficheiro>` tem de mostrar só os hunks alheios
no sítio deles. Ver [[feedback_a_commit_with_paths_never_picks_up_an_untracked_file]].
