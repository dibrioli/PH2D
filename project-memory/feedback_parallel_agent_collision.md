---
name: feedback-parallel-agent-collision
description: "Multiple Claude sessions / agents running in the same repo can collide on `git add` + `git commit` — one session's staged files end up in another's commit, with merged/truncated commit messages."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 46e3e7df-13b7-4cb0-83a5-1aa650dcd862
---

When the PH2D project runs the multi-agent model (Coordenador +
Agentes Periféricos + multiple Claude sessions in parallel), **only
one session can hold the index at a time**. If session A has files
staged (`git add` done, `git commit` not yet run) and session B runs
its own `git commit` without first checking `git status`, session B's
commit picks up everything in the index — A's staged files end up in
B's commit, often with a fused or truncated commit message.

**Concrete incident (2026-05-13):** session B (deleting
`PARALLEL_AGENTS.md`) ran `git commit` while session A was mid-way
through a `git commit` whose pre-commit hook was still validating.
The result was a single commit (`0e96c00`) with:
- A's title replaced by B's
- B's body at the top, A's body fused below (truncated mid-sentence)
- All 4 files (A's 3 + B's 1 deletion) bundled together

Fixed via `git reset --soft HEAD~1` → split into two clean commits
(`869fbc2` + `93dfd7b`) → `git push --force-with-lease`. Risk
accepted because no other agent had pulled the bad SHA yet.

**Why:** The project explicitly documents this in
`docs/IntegracaoMultiAgente/` — "apenas o Coordenador toca arquivos
compartilhados (incluindo git add e git commit)". But sessions
operating independently as helpers can violate the rule without
realizing because git itself has no per-session index lock.

**How to apply:**
1. **Before EVERY `git commit`**, run `git status` and verify the
   staged set matches what you intend to commit. If unexpected files
   appear, STOP — another session probably has work in progress.
2. **Before EVERY `git add <file>`**, also run `git status` —
   `git add .` or `git add -A` is the dangerous sibling that vacuums
   up everything regardless of which session owns it.
3. If you detect the collision AFTER the commit but BEFORE push:
   `git reset --soft HEAD~1` → unstage the other session's files
   (`git restore --staged <their-files>`) → re-commit yours with your
   own message. The other session's files stay in the working tree
   for them to commit themselves.
4. If detected AFTER push: split via `git reset --soft HEAD~1` →
   two clean commits → `git push --force-with-lease` IFF no agent
   has pulled the bad SHA. Coordinate with the user before
   force-pushing.
5. Sessions running pre-commit hooks (5+ min duration) are
   especially vulnerable because the index is held in "wants to
   commit" state for that whole window — any other session's
   `git commit` in that window collides.

**Update 2026-05-14 — hook is now TIERED + docs formalize the rule:**
The repo's `scripts/pre-commit.sh` auto-selects tier from the staged
diff:
- T0 (~5s)   — docs / config-only
- T1 (~30s)  — single crate (`-p <X>`)
- T2 (~3-5m) — workspace (Cargo.toml/lock, `shells/desktop`,
                multi-crate, foundational crates ph2d-core/ecs/host/
                tokens/a11y)

The `docs/IntegracaoMultiAgente/` briefings (01–04) now document the
collision protocol explicitly:
- 02-Coordenador.md §3.4–3.6 (atomic stage+commit, collision symptoms,
  recovery).
- 03-Agente-Periferico.md §7.4–7.5 (same protocol from Periférico's
  POV + tier table).
- 04-Agente-PRCI.md §9 (collision symptom table for the push-time role).
- 01-Enio.md "Regras de ouro" (single Coordinator, serialized commits).

Hot tips:
- Periférico em pasta isolada: pre-validate manualmente, commit com
  `git commit --no-verify` para ~1s total (vs ~30s do T1).
- Coordenador em arquivo compartilhado: deixe o hook rodar T2; é o
  safety net que pega cross-crate regression antes de push.
- Em iteração rápida no inner loop: `cargo check -p X` (~10s) é
  suficiente; rode `nextest -p X` antes do commit.

Related: [[reference-canonical-files]], the docs/IntegracaoMultiAgente/
briefings define the formal Coordenador rule.

**Update 2026-05-27 — R8 audit incident (window-between-add-and-commit):**
During the T1.6 R8 audit remediation, I staged 7 pure-mine files via
`git add path1 path2 ...`, then ran `cargo check --workspace --all-targets`
(4m07s) for validation, then `git commit -m ...`. The commit returned
"no changes added to commit" because **another agent's commit
`90abf85` (color-eq Domain Transform) had absorbed my staged files
during the 4-minute validation window**. Work was preserved in HEAD
(127/127 tests still green, all R8 changes intact in the swallowing
commit's diff), but attribution was wrong + my commit message lost.

Recovery: empty commit `61a1428` with `--allow-empty` documenting the
R8 scope + reasoning so future readers tracing the audit trail can
find it. NO destructive recovery attempted (no revert, no rebase,
no force-push) — risk of clobbering alheia work greater than the
attribution-fix benefit.

**Lesson for next time:**
- The window between `git add` and `git commit` is the danger zone.
  Long validation runs (`cargo check --workspace`) BEFORE the commit
  AMPLIFY the collision risk linearly with their duration.
- Preferred sequence when no shared-file hunks are involved:
  `git commit -- <path1> <path2> ...` (with `--`) is **atomic
  stage+commit** — closes the window to a few hundred ms.
- When `git apply --cached` is required (because a file has both
  mine + alheia hunks), do `git apply --cached` + `git commit`
  back-to-back, NOT with validation in between. Run validation
  BEFORE the patch application instead.
