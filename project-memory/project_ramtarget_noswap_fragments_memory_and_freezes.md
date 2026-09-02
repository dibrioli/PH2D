---
name: project_ramtarget_noswap_fragments_memory_and_freezes
description: "Os travamentos de 2026-08-30 (2 num dia) têm mecanismo MEDIDO: o tmpfs `noswap` de /mnt/ramtarget fixa dezenas de GB de páginas INMOVÍVEIS e a memória fragmenta até não haver um bloco de 2 MB livre"
metadata:
  type: project
---

**Medido em 2026-08-30, nos dois travamentos do mesmo dia** (boots `-1` e `-2`).
Não foi falta de memória: nas duas vezes havia **68 GB livres e 22 GB de swap
livre**. O que faltava era **CONTIGUIDADE**.

## O mecanismo, na ordem em que o kernel o registou

1. `/mnt/ramtarget` é um tmpfs de 48 GB montado com **`noswap`** — a opção que o
   [[feedback_a_tmpfs_backed_target_reports_file_exists_not_broken_link]] e o
   `scripts/ram-build.sh` introduziram em **24/08 para curar o travamento de
   22/08** (o desenho antigo, em `/dev/shm`, comia o zram).
2. ⚠️ **`noswap` marca as páginas `unevictable`, e a compactação normal do kernel
   NÃO migra página `unevictable`** (o `ISOLATE_UNEVICTABLE` só é ligado para
   `alloc_contig`/CMA). ⇒ elas não se recuperam **e não se movem**.
3. **Prova aritmética, e ela fecha ao kB:** `Unevictable` == uso do ramtarget.
   Agora: `6 490 992 kB` contra `6 487 372 kB`. No travamento das 18:35:
   `unevictable: 28 928 032 kB` com `shmem: 28 988 196 kB` — **29 GB pregados**,
   espalhados pela RAM inteira, com `Mlocked` a valer 3 MB.
4. Resultado no dump da zona Normal às 18:35:03 — `... 4231*1024kB (UM)
   **0*2048kB 0*4096kB** = 68 361 892kB`. **68 GB livres e ZERO blocos de 2 MB.**

## Os dois desfechos, os dois fatais

- **11:09** — `kcompactd0` preso **495 s** num núcleo (`compact_zone` →
  `migrate_pages_sync`), 18 `soft lockup` + `rcu_preempt stall`. Ele tentava
  fabricar o bloco que as páginas pregadas o impedem de fabricar.
- **18:35** — `godot: vmemmap alloc failure: order:9` vindo de
  **`nvidia_uvm` → `memremap_pages` → `vmemmap_populate_hugepages`**: o driver
  precisa de 2 MB contíguos para o `struct page` da VRAM ao registar a GPU.
  Levou `-ENOMEM`, **não tratou**, e 10 s depois `Oops: GPF` em
  `kfifoChidMgrAllocChid_IMPL` com ponteiro não-canónico → morte.

## O que AGRAVA (defaults do CachyOS, não escolha nossa)

`transparent_hugepage=**always**` (todo `rustc` pede blocos de 2 MB o tempo todo)
e `vm.compaction_proactiveness = **0**` (nada desfragmenta em segundo plano ⇒ a
conta chega toda de uma vez, sob pressão).

## Censo dos 20 boots

Travaram **4**: 19/08, 22/08, 30/08 11:09, 30/08 18:35. Os dois de 30/08
deixaram trace completo; os de 19 e 22/08 pararam mudos. ⚠️ **As DUAS versões do
disco em RAM travaram esta máquina, por mecanismos OPOSTOS** — a swappável comeu
o zram (22/08), a `noswap` estilhaça o alocador (30/08).

## A régua de uma linha

```
awk '/Normal/{print "blocos de 2 MB livres: " $(NF-1) "  | de 4 MB: " $NF}' /proc/buddyinfo
```
Saudável logo após o boot: `437 | 2281`. No travamento: **`0 | 0`**.

**Why:** o ganho do disco em RAM está medido no próprio `ram-build.sh` e é
**só escrita de SSD** (−73%); a velocidade é ruído declarado (28→24 s). Trocar
estabilidade da máquina por desgaste de um NVMe com 1,3 TB livres é o negócio
errado — e a nota do §0.0 vale aqui: *o limite legítimo diz de que recurso é*, e
este recurso é **contiguidade**, que nenhuma sonda deste repo mede.

**How to apply:** antes de abrir 6 linhas, corra a régua acima. Para curar:
desmontar o ramtarget (`bash scripts/ram-build.sh --umount`), tirar a linha do
`/etc/fstab`, pôr `vm.compaction_proactiveness = 20` e THP em `madvise`.
⚠️ A hipótese de **DDR5 marginal** de [[project_workstation_freeze_memory_reclaim]]
**não está refutada** (o `Bad page state` com um byte inteiro esborratado é
assinatura dela) — mas ela deixou de ser necessária para explicar 30/08, e o
memtest86+ continua por correr.

## ✅ APLICADO em 2026-08-30 19:39 (`scripts/fix-travamentos.sh`)

| | antes | depois |
|---|---:|---:|
| `Unevictable` | **6 490 448 kB** | **3 076 kB** (−2110×) |
| blocos de 2 MB livres | 454 (e **0** no travamento) | **2 316** |
| swap | só zram 32 G | zram 32 G + **`/swap/swapfile` 32 G, pri 10** |
| `/mnt/ramtarget` | 48 G `noswap` | **64 G swappável** |
| `vm.compaction_proactiveness` | 0 | **20** |
| THP | `always` | **`madvise`** |

⭐ **A régua que prova a cura é a igualdade que DESAPARECEU:** `Unevictable`
acompanhava o uso da tmpfs ao kB, e hoje vale 3 MB (só o `Mlocked`). Se voltar a
acompanhar, o `noswap` voltou.

⚠️ **Três coisas que só a execução ensinou:**
1. **A trava `pgrep rustc|cargo` era o teste ERRADO** e ficou vermelha 2×: das 8
   árvores só a primária está armada, e com muitos agentes os builds entram e
   saem entre o teste e o `umount`. O teste que **prediz** o `umount` é
   `fuser -m` — quem SEGURA o mount. Hoje o script espera por uma janela (120 s).
2. **Quem segurava eram os 2 `rust-analyzer-proc-macro-srv`**, com `.so` de
   proc-macro em `mmap` — não um build. Matá-los é barato (o VS Code recria).
3. **O swapfile precisa de subvolume PRÓPRIO** (`/swap`): o `/` desta máquina tem
   `/.snapshots`, e um swapfile num subvolume snapshotado não activa. Subvolume
   aninhado não entra no snapshot do pai.

⛔ **`umount -l` (lazy) é a cura falsa** — deixa a tmpfs antiga viva até à última
referência, logo a RAM não se liberta e a nova monta por cima.

⚠️ O `/etc/fstab` tem cópia em `/etc/fstab.bak-20260830-193827`; `findmnt
--verify` dá **0 erros** (o aviso do swapfile é normal para ficheiro de swap).
