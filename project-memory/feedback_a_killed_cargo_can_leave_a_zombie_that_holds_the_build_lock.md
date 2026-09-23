---
name: a-killed-cargo-can-leave-a-zombie-that-holds-the-build-lock
description: Matar um cargo a meio pode deixar um ZOMBIE com uma thread presa no kernel a segurar os flocks do target — «Blocking waiting for file lock on build directory» para sempre; diagnóstico e cura medidos
metadata:
  type: feedback
---

Medido 2026-09-23 (`line/components`, cura da auditoria 26): matei um `cargo nextest` de fundo
obsoleto (`kill` no nextest e no `ph2d-run.sh`). O `cargo test --no-run` filho ficou `[cargo]
<defunct>` com **uma thread em `R` a 100 % de CPU de SISTEMA** que não morre nem com `kill -9`
(presa no kernel, `stime` a subir) — e **os flocks dele continuaram vivos**. O portão seguinte ficou
em `Blocking waiting for file lock on build directory` sem progresso nenhum.

- ⚠️ `fuser`/`lsof` e a varredura de `/proc/*/fd` **não o mostram** (o zombie não tem fds visíveis);
  quem o mostra é o **`/proc/locks`** cruzado com o inode do `target/<perfil>/.cargo-build-lock`
  (`stat -c %i`) — a linha traz o PID do zombie. Depois `ls /proc/<pid>/task` e o `stat` da thread.
- ⭐ **Cura: um `CARGO_TARGET_DIR` PRÓPRIO dentro da MESMA worktree** (`target/gate`) — o lock é por
  directório de build, e isto não é partilhar entre worktrees (a regra que continua de pé).
- ⛔ Não se cura sem root nem por sinal; só o perfil trancado fica inutilizável (o `debug` do clippy
  seguiu normal).

**Why:** o silêncio de um cargo bloqueado lê-se igual a «a compilar», e eu perdi minutos a achar que
era carga.
**How to apply:** antes de matar um cargo de fundo, prefira deixá-lo acabar; se um build ficar em
`Blocking waiting for file lock` com zero CPU, leia o `/proc/locks` antes de esperar mais — e ligue
isto a [[sharing-a-target-dir-between-worktrees-corrupts-the-build]].
