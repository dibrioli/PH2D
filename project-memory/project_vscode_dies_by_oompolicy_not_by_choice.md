---
name: project_vscode_dies_by_oompolicy_not_by_choice
description: "O VSCode morre num OOM por POLÍTICA do systemd (OOMPolicy=stop no scope dele), não por alguém o escolher — e o earlyoom fica mudo porque a condição dele é um AND"
metadata: 
  node_type: memory
  type: project
  originSessionId: 4d854466-7014-4e58-98f6-b338a1e78fc3
  modified: 2026-08-15T03:29:32.461Z
---

Em **2026-08-14 23:26** um teste do PH2D alocou **90,2 GB** de RAM anônima e o
VSCode foi encerrado. As duas metades do mecanismo são independentes e nenhuma
é "o kernel escolheu o VSCode".

**1. O VSCode morre por POLÍTICA, não por escolha.** O kernel matou o processo
CERTO (`ph2d_painter_br`, `oom_score_adj:200`). Mas ele rodava dentro do
`app-code-1844.scope` — o cgroup do VSCode, porque quem o lançou foi um agente
na janela. Com `DefaultOOMPolicy=stop` (o default do systemd), **um oom-kill do
KERNEL dentro de um scope faz o systemd parar o scope inteiro**:

```
kernel: Out of memory: Killed process 2784265 (ph2d_painter_br)
systemd[1223]: app-code-1844.scope: Failed with result 'oom-kill'.
```

⚠️ **Só o OOM killer do KERNEL dispara isso.** Se quem mata é o *earlyoom* (um
processo de userspace mandando SIGTERM), o systemd não vê nada e a janela
sobrevive. Aplicado: `~/.config/systemd/user.conf` com `DefaultOOMPolicy=continue`.
A defesa estrutural é rodar build/teste num scope PRÓPRIO
(`systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0`) — medido no
mesmo dia: o cgroup matou o scope da medição e o VSCode ficou de pé.

⚠️ **Ela virou ferramenta: `~/.local/bin/ph2d-run <comando>`** (teto 24G por
default, `PH2D_MEM_MAX` sobrescreve). Ela **não é só para cargo** — envolve
qualquer comando, e vale para toda sessão nesta máquina, não só as do PH2D.
Medido em 2026-08-15 no SmarthCODE (suíte Playwright + Chrome): pico de
**801 MiB** contra o teto de 24 GiB, e os gates sensíveis a tempo passaram
iguais sob o scope (362/362) — ou seja o custo de adotá-la é zero e ela não
perturba medição de gesto.

**2. O earlyoom fica mudo porque a condição é um AND.** `-m 10 -s 10` significa
`mem avail <= 10%` **E** `swap free <= 10%` (o próprio log escreve "and"). No
instante do OOM: swap em **0,00007%** (24 kB de 32 GB) mas **27,6 GB de RAM
LIVRES** (~23%, acima do gatilho). O AND nunca fechou.

⚠️ **O modo de falha é "o SWAP acaba com RAM sobrando", que é característico do
zram** — o zram vive *dentro* da RAM, então swapar agressivamente (swappiness
100) o enche e o kswapd fica sem para onde evacuar anônimo. Com page cache em
**164 MB** não havia mais nada recuperável ⇒ OOM global com a zona Normal em 27 GB.
Isto é DIFERENTE do travamento de 2026-08-08 ([[project_workstation_freeze_memory_reclaim]]),
que foi livelock de reclaim com 105 GB de page cache e **nenhum** oom-kill.

**Why:** os dois eventos parecem "faltou memória" e pedem curas opostas — um é
marca d'água/reclaim, o outro é teto de swap + política de cgroup. E o
`ph2d-check-memoria` dava **9 OK** durante este incidente: ele não cobre nem o
`OOMPolicy` nem o AND do earlyoom.

**How to apply:** num OOM, leia SEMPRE `Free swap` e `Node 0 Normal free` no
dump do kernel antes de concluir que a RAM acabou — e confira quem era o
`task_memcg`, porque é ele que decide quem morre por tabela.
