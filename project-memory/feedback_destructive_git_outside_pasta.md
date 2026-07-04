---
name: destructive-git-outside-pasta
description: "NUNCA use `git restore --staged --worktree` (ou git checkout --/reset --hard) em paths fora da pasta exclusiva do agente sem coordenar via Enio primeiro — pode silenciosamente apagar trabalho não-commitado de outro agente em paralelo"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1af06e2c-6bc6-4bb5-9d01-ac735fc64cc4
---

Em operação multi-agente paralela, NUNCA rode comandos git destrutivos
(`git restore --worktree`, `git checkout -- <path>`, `git reset --hard`)
em paths fora da sua pasta exclusiva sem coordenar via Enio primeiro.

Mesmo que o `git status` mostre arquivos como "modified" no working tree,
esses arquivos podem conter WIP do Coordenador ou de outro Agente Periférico
no meio de um commit. Restaurar pra HEAD apaga silenciosamente o trabalho.

**Why:** Na sessão de bgremoval (slot 2) em 2026-05-15, o agente rodou
`git restore --staged --worktree crates/ph2d-editor/src/grid_snap/* crates/ph2d-grid/src/snap.rs shells/desktop/src/main.rs`
pra desbloquear build local (HEAD parecia broken por timing race com commit
em andamento do Coord). Coord tinha ~700 LOC de panel rewrite não-commitado
no working tree naquele momento. O restore acabou rodando DEPOIS do commit
`d70bea1` do Coord landar (então nada foi perdido), mas se tivesse rodado
30 segundos antes, **o panel rewrite inteiro teria sido apagado em silêncio**.
Coord flagou explicitamente como ação destrutiva a evitar.

**How to apply:** se um build estiver broken por código fora da sua pasta:

1. PARE. Não rode `git restore`, `git checkout --`, `git reset --hard`,
   ou qualquer comando que mute working tree em paths compartilhados.
2. Inspecione sem mutar: `git status --short`, `git diff --stat <path>`,
   `git log -p <path> | head`.
3. Reporte ao Enio com file:line do erro de compilação e o context (qual
   símbolo ausente, qual commit suspeito).
4. Se precisa de build verde pra avançar mesmo assim: escreva seu código
   normalmente, comente partes que dependem do mundo lá fora, e defira
   validação `cargo test` para quando Coord landar o fix.
5. NUNCA use `git stash apply stash@{N}` de um stash que pertence a outro
   agente sem instrução explícita do Coord/Enio — pode trazer WIP alheio
   pro seu working tree.

Estende [[feedback_parallel_agent_collision]] (que cobria só stage/commit) —
agora também restore/checkout/reset.
