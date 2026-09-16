---
name: a-per-command-ceiling-does-not-compose-only-a-slice-does
description: Um tecto por comando soma-se a cada corrida; «esta LINHA <= 50%» só um cgroup partilhado o exprime
metadata:
  type: feedback
---

Medido 2026-09-15, com controlo, lendo o `cpu.stat` da própria fatia:

| arranjo | comandos | núcleos de 32 |
|---|---|---|
| fatia **sem** tecto (controlo) | 2 | 22,32 |
| tecto no **COMANDO** | 2 | **30,05** ⛔ soma |
| tecto na **FATIA da linha** | 2 | 16,03 |
| tecto na **FATIA da linha** | 4 | **16,01** ⭐ compõe |

**Why:** *«nunca mais de 50 %» é uma afirmação sobre um AGENTE, não sobre um
processo.* Um `nice`, um `-j`, um `--test-threads`, um `CPUQuota` no scope — são
todos por-processo, e duas corridas do mesmo agente somam dois tectos. Só um
cgroup que o agente inteiro partilha (uma *slice* do systemd, aqui derivada da
worktree) consegue dizer a frase.

**How to apply:** quando alguém pedir um tecto «por X», pergunte primeiro *o que
é X* e ponha o tecto no cgroup de X, não no comando. E meça com a régua de X (o
`cpu.stat` da fatia) — a 1.ª leitura desta medição saiu por `/proc/stat` e veio
contaminada pelo smoke que o dono corria ao lado: *uma régua da máquina inteira
não mede uma linha.*

⭐ E um tecto não é tudo: **quota e peso são grandezas diferentes**. O tecto
limita UMA linha; o que impede N linhas de estrangular o dono é `CPUWeight` —
medido, com a linha a 32 queimadores o smoke foi de `6,8 s` para `7,0 s` (+3 %) e
a linha ainda usou `17,7` núcleos do que sobrava. Um peso não desperdiça máquina
ociosa; um tecto sim. Os dois coexistem de propósito.

Porta, medições e recusas: `docs/DevOps/TETOS_DE_RECURSO_POR_LINHA.md`.
Ver [[the-gpu-cannot-be-divided-only-excluded]] e
[[a-tool-that-no-written-step-names-dies]].
