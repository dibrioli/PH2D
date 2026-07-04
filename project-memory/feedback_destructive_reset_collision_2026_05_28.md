---
name: feedback-destructive-reset-collision-2026-05-28
description: "Outro agente fez `git reset --hard HEAD` durante sessão Coord-A T1.9, apagando ~800 LOC de WIP em tracked files. Lição: STAGE imediatamente após cada edit foundational, mesmo sem commit ainda."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 72e6898d-e3df-4276-8ee8-f05e2d2ee259
---

Em 2026-05-28 durante T1.9 Painter wire (Coord-A), outro agente (provavelmente asset-cooker W1.T6 / commit `2ab3fac`) fez `git reset --hard HEAD` após committar SEU trabalho, obliterando ~800 LOC do meu WIP UNCOMMITTED em arquivos tracked:

- `crates/ph2d-tool-painter/src/tool.rs` (T1.9 wire completo + 14 remediações: 9 fields, begin/queue/end_stroke, attach/detach_journal, 4 helper free-fns, R-9 threading doc)
- `crates/ph2d-tool-painter/Cargo.toml` (deps ph2d-painter-stroke + ph2d-color)
- `crates/ph2d-painter-stroke/src/durability/journal.rs` (R-6 reusable buffer + AttachDuringActiveStroke variant + Display + PartialEq arms)
- `crates/ph2d-painter-stroke/src/record.rs` (4 SAMPLE_FLAG_* constants)
- `crates/ph2d-painter-stroke/src/lib.rs` (re-exports SAMPLE_FLAG_*)
- `docs/plans/2026-05-wave-11-carry-overs.md` (seção Painter T1.9 — 7 itens deferidos)

Só sobreviveu o test file `crates/ph2d-tool-painter/tests/history_integration_t19.rs` porque era **untracked** — `git reset --hard` não toca untracked.

**Rule:** Em sessões longas com parallel agents, STAGE files com `git add -- <my-paths>` imediatamente após edit foundational pra criar fence contra `git reset --hard` alheio. Stage não bloqueia outro agente nem cria commit — só preserva o snapshot no index. Commit pode aguardar até final como sempre, mas index protege contra wipe.

**Why:** `git reset --hard HEAD` reverte tracked files no working tree pra HEAD. Files staged no index sobrevivem (próximo commit ainda os carrega). Untracked também sobrevivem. **Apenas tracked + uncommitted são vulneráveis.**

**How to apply:**
- Após terminar um bloco foundational de edits (e.g., struct redesign + impl rewrite), `git add -- crates/<my-scope>/src/<my-file>.rs` mesmo que vá editar mais depois.
- Antes de qualquer rodada de validation/audit que vai durar minutos, `git add -- <all-my-touched-paths>`.
- Não usar `git add -A` ou `git add .` (pega arquivos alheios — vide [[feedback_destructive_git_outside_pasta]]).
- Se outro agente fez reset DESTRUTIVO igual aqui, vide reflog `HEAD@{n}: reset: moving to HEAD` — meu trabalho está perdido (não há ORIG_HEAD pra recovery porque não houve commit/merge).

Companion to [[feedback_parallel_agent_collision]] (git index collision) e [[feedback_destructive_git_outside_pasta]] (eu quase apaguei trabalho alheio em 2026-05-15). Esta vez foi o reverso — apagaram meu trabalho.

Cross-reference: SESSION_ACTIVE.md (DIRETRIZ §1.1.1) não previne — outro agente pode ignorar a reserva de pastas. Stage é defesa-em-profundidade.
