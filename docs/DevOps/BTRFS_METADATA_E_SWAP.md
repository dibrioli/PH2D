# BTRFS na workstation — metadata faminta, swap cheio e checksum corrompido *(2026-08-22)*

> **O que é:** o runbook do disco desta máquina (CachyOS, btrfs `compress=zstd:1`, NVMe 950 GB,
> zram 32 GiB). Nasceu da jornada em que «o disco encheu» com **526 GB livres**, o swap estava a
> **100% com 61 GiB de RAM livre**, e o `mold` morria em SIGBUS **45 vezes em 3 dias**. São três
> doenças distintas que parecem uma («a máquina está lenta com várias linhas»), e cada uma tem o
> seu instrumento, a sua cura e o seu preço. **Medir primeiro:** `bash scripts/btrfs-health.sh`
> (sem root, 1 s, exit 1 em vermelho).
>
> ⚠️ A cura de §1 e a de §3 exigem **root** — o agente mede e entrega o comando; quem roda é o Enio
> (§4 é o bloco colável). O agente executa sozinho só §2 (`target-to-disk.sh`) e o `chattr +C`.

## §0 — TL;DR: os três achados, com os números

| # | Sintoma que se vê | O que é de facto | Instrumento | Cura |
|---|---|---|---|---|
| 1 | `ENOSPC`/«disco cheio» a meio do build, `df` a 45% | **metadata sem espaço para crescer**: 937,85 GiB dos 950 estavam alocados a blocos de DADOS (410 usados), **0 byte não-alocado**, metadata 6 GiB com 5,46 usados | `btrfs-health.sh` → «não-alocado», «metadata livre» | **`btrfs balance -dusage=N`** (root) + timer semanal |
| 2 | Swap 32/32 GiB com 61 GiB de RAM livre; OOM derruba a janela no pico | o `target/` do primário vive em **tmpfs** (33 GB); 30 GB dele estavam **swapados para o zram**, que é RAM (12,1 GiB comprimidos) | `btrfs-health.sh` → «swap», «target em tmpfs» · `zramctl` | **`scripts/target-to-disk.sh`** (depois de 1) — tmpfs **retirado** do tier |
| 3 | `mold` SIGBUS / `rustc` SIGSEGV em laço, ~10 min por tentativa; `cargo clean -p` «cura» | artefatos **recém-escritos voltam do disco com checksum errado** (`csum failed`, dezenas de inodes novos); **0 nos dois boots com kernel 7.1.8, 208+ nos dois com 7.2.0** | `btrfs-health.sh` → «corrupção» · `coredumpctl` · journal | **A/B de kernel** (LTS já instalado) · `btrfs scrub` · memtest86+ |

A ordem importa: **1 antes de 2** (a cópia de 33 GB precisa de metadata para nascer), e **3 é
independente** (é um reboot noutro kernel, e pode ser o primeiro passo se houver agentes a construir).

---

## §1 — Metadata faminta (o «disco cheio» que o `df` não vê)

**Mecanismo.** O btrfs divide o disco em *chunks* de dados e de metadata, alocados sob demanda a
partir do espaço **não-alocado**. Quando um dia de jornada escreve ~800 GB de `target/`, o FS aloca
chunks de dados até ao fim do disco. O `rm -rf` do fim de dia liberta o espaço **dentro** dos chunks,
mas o kernel só devolve ao não-alocado os chunks que ficam **100% vazios** — os meio-cheios ficam
alocados. Resultado medido: 937,85 GiB alocados a dados com 410 usados (depois 331, com a limpeza de
outra linha), não-alocado **0**, e a metadata (6 GiB DUP, 5,46 usados, depois 4,92) sem ter de onde
crescer. A partir daí cada arquivo novo disputa os ~500 MB que sobram; quando acabam, `write()`
devolve `ENOSPC` com o `df` a mostrar 500 GB livres — e um `.o` escrito pela metade é o que o `mold`
mapeia e morre em SIGBUS.

**Por que o `df` mente:** ele soma o livre *dentro* dos chunks de dados. A metadata não mora lá.

**Medir (sem root):**
```bash
bash scripts/btrfs-health.sh         # «não-alocado» e «metadata livre» com veredito
btrfs filesystem df /                # a fonte: Data total/used · Metadata total/used
```

**Curar (root, progressivo — pode interromper com Ctrl+C, o que já foi feito fica):**
```bash
sudo btrfs balance start -dusage=10 /      # relocaliza só chunks ≤10% cheios: rápido, liberta vários GiB
sudo btrfs balance start -dusage=30 /      # depois os ≤30%
sudo btrfs filesystem usage / | grep -E 'unallocated|Metadata'   # alvo: unallocated ≥ 50 GiB
```
`-dusage=N` toca **só** chunks de dados com ≤N% de uso; o custo é I/O (cada chunk relocado é 1 GiB
lido e reescrito) e o ganho é o chunk inteiro a voltar ao não-alocado. Com 606 GiB de folga dentro
dos chunks, o `-dusage=10` termina em minutos e já desbloqueia a metadata para semanas.

**Prevenir (root, uma vez):** o timer semanal em [`systemd/`](systemd/) — o pacote
`btrfsmaintenance` **não está** nos repositórios configurados desta máquina (é AUR), e dois arquivos
de unit fazem a mesma coisa sem dependência:
```bash
cd /home/enio/Documentos/Projetos/PH2D && sudo cp docs/DevOps/systemd/ph2d-btrfs-*.{service,timer} /etc/systemd/system/ \
  && sudo systemctl daemon-reload && sudo systemctl enable --now ph2d-btrfs-balance.timer ph2d-btrfs-scrub.timer \
  && systemctl list-timers 'ph2d-*'
```

**O que a linha pode fazer sozinha:** `chattr +C` (nodatacow) em todo `target/` de worktree
(`find <target> -type d -exec chattr +C {} +` — o flag num dir só vale para arquivos **novos**, e os
subdirs que já existem não o herdam, daí o `find`). Sem CoW nem zstd, um `.rlib` de 500 MB é ~4
extents em vez de ~4.000 (extent comprimido tem teto de 128 KiB) — é **menos metadata por GB
escrito**, além de menos CPU. A DIRETIVA_FIM_DE_DIA §2 já recria assim; em 2026-08-22 três targets
tinham nascido sem e foram corrigidos.

---

## §2 — Swap cheio com RAM sobrando (o target em tmpfs vive no zram)

**Mecanismo.** `target/` do primário → `/dev/shm/ph2d-target` (tmpfs = RAM). Com 33 GB lá e 105 GB
de page cache a servir 370 GB de targets de worktree, o kernel (swappiness 150) **swapa as páginas
do tmpfs** para o zram — que também é RAM, comprimida. Medido: `Shmem` residente 3,1 GB, zram
**31,6 GB de dados em 12,1 GiB**, swap 99% ocupado, e `VmSwap` de todos os processos somava ~1,5 GB
— os outros 30 GB são o target. Consequências: (a) 12 GiB de RAM permanentemente pagos por um
artefato descartável; (b) **zero folga de swap** para o próximo pico, que é exatamente o modo de
falha de 2026-08-14 («o swap acaba com RAM sobrando», o earlyoom não fecha o AND, o OOM do kernel
mata dentro do scope e o `OOMPolicy=stop` derruba a janela); (c) o build evapora no reboot.

**O que o tmpfs prometia:** «grande ganho de link/IO» — sem tabela ao lado (CLAUDE.md §0.0). O
rustc é CPU-bound; o `mold` lê objetos do page cache de qualquer modo; num NVMe com nodatacow a
escrita é sequencial. O preço acima está medido; o ganho nunca esteve.

**Curar (a linha pode; só DEPOIS do §1, e com o primário ocioso):**
```bash
bash scripts/target-to-disk.sh --dry-run   # portões: symlink? ninguém constrói/executa? disco aguenta 33 GB?
bash scripts/target-to-disk.sh             # copia tmpfs → target/ real com +C, troca o link, apaga o tmpfs
```
O último passo devolve os ~30 GB de swap na hora (apagar um arquivo de tmpfs liberta os slots dele).
`--cold` pula a cópia (próximo build frio; o sccache serve os deps).

**Política (ADR-0104, emenda 2026-08-22):** `target/` em tmpfs **retirado** do tier workstation.
`scripts/target-on-tmpfs.sh` só arma com `--force`. A regra `/etc/tmpfiles.d/ph2d-ramtarget.conf`
é inofensiva (recria um dir vazio no boot); retirar: `sudo rm /etc/tmpfiles.d/ph2d-ramtarget.conf`.

---

## §3 — Checksum corrompido em arquivos recém-escritos (kernel 7.2.0, e o SIGBUS do `mold`)

**A evidência, por boot:**

| boot | kernel | `csum failed` (linhas no journal, + suprimidas) | coredumps `mold`/`rustc` |
|---|---|---:|---:|
| 19/08 19:55 → 20/08 00:03 | 7.1.8 | **0** | 0 |
| 20/08 15:33 → 21/08 00:05 | 7.1.8 | **0** | 0 |
| 21/08 13:50 → 22/08 04:39 | **7.2.0-1** (instalado 21/08 00:04) | 100 | 17 |
| 22/08 07:47 → … | **7.2.0-1** | 108 (+16 «callbacks suppressed») | 16 |

Contador persistente do device: `corruption_errs 2804` (`write 0, read 0, flush 0`). Os inodes são
**dezenas, todos novos** (66,6 M–68,8 M — artefatos de build), e cada evento é seguido em segundos
por um coredump: `csum failed` 17:24:12 → `mold` SIGBUS 17:24:17; 18:25:08 → 18:25:19. O `mold`
faz `mmap` do `.o`; a página que falha checksum devolve EIO no page fault ⇒ SIGBUS. **Não é o `.o`
truncado** (essa é a outra causa possível do mesmo sintoma, pelo §1) — é o arquivo inteiro no disco
com bytes diferentes dos que foram checksumados ao escrever.

**As três hipóteses, e o que separa cada uma:**
1. **Regressão do kernel 7.2.0** (o `.0` de uma série nova, com btrfs+zstd+zram sob carga de 32
   núcleos). A correlação acima é **perfeita** e o A/B é barato: o `linux-cachyos-lts` 6.18.42 **já
   está instalado**. Um dia de jornada no LTS com `btrfs-health.sh` a zero responde.
2. **DDR5 marginal** (4 DIMMs AM5 sem ECC — a hipótese antiga, de
   [`project_workstation_freeze_memory_reclaim`](../../project-memory/project_workstation_freeze_memory_reclaim.md):
   «byte zerado», 2× `mold` SIGBUS + 2× `rustc` SIGSEGV em julho/agosto, **antes** do 7.2.0). Se o LTS
   **não** zerar os erros, é isto — e só o **memtest86+ overnight** fecha (pacote não instalado:
   `sudo pacman -S memtest86+`, e escolher no menu de boot).
3. **NVMe** (XPG GAMMIX S70 BLADE, fw 03020P5E): `sudo smartctl -a /dev/nvme0 | grep -iE 'media|error|percentage|temperature'`
   — `Media and Data Integrity Errors` > 0 aponta para cá.

⚠️ **O `nodatacow` dos targets (§1) tem um efeito colateral aqui:** arquivo sem CoW **não tem
checksum**. Nos targets com `+C`, a mesma corrupção passa **em silêncio** — vira teste que falha sem
motivo, `rustc` que segfaulta, binário estranho — em vez de um `csum failed` no journal. Os canários
que sobram são o `~/.cache/sccache` (CoW), a fonte e o `.git`. É mais uma razão para resolver a
causa (kernel/RAM), não para voltar ao CoW nos targets.

**Curar / diagnosticar (root):**
```bash
# 1) reboot no kernel LTS: no menu de boot, a entrada "linux-cachyos-lts". Depois, ao fim do dia:
bash scripts/btrfs-health.sh                       # «checksum: 0 nesta boot» = hipótese 1 confirmada
# 2) scrub: lê TUDO (≈330 GB, minutos num NVMe) e imprime no journal o CAMINHO de cada arquivo corrompido
sudo btrfs scrub start -B /                         # -B = espera e mostra o sumário
sudo journalctl -k --since '1 hour ago' | grep -iE 'checksum error|unable to fixup' | head
# 3) zerar o contador só DEPOIS de anotar o valor (ele é a baseline do btrfs-health):
sudo btrfs device stats /  &&  sudo btrfs device stats -z /
```
Um arquivo corrompido de `target/`/`sccache` apaga-se (regenera); um de fonte restaura-se do git;
um de `~/.config`/`.local` restaura-se do backup ou do pacote.

---

## §4 — O bloco do Enio (colável, na ordem)

```bash
# A) METADATA — primeiro; minutos; pode rodar com agentes vivos (só I/O a mais)
sudo btrfs balance start -dusage=10 / && sudo btrfs balance start -dusage=30 / \
  && sudo btrfs filesystem usage / | grep -E 'unallocated|Metadata'
# B) PREVENÇÃO — uma vez
cd /home/enio/Documentos/Projetos/PH2D && sudo cp docs/DevOps/systemd/ph2d-btrfs-*.{service,timer} /etc/systemd/system/ \
  && sudo systemctl daemon-reload && sudo systemctl enable --now ph2d-btrfs-balance.timer ph2d-btrfs-scrub.timer
# C) SWAP — com o primário ocioso (nenhum cargo na raiz); a linha pode rodar isto
cd /home/enio/Documentos/Projetos/PH2D && bash scripts/target-to-disk.sh
# D) CORRUPÇÃO — reboot na entrada "linux-cachyos-lts" do menu de boot; no fim do dia:
cd /home/enio/Documentos/Projetos/PH2D && bash scripts/btrfs-health.sh
```

---

## §5 — ⛔ Recusas e decisões (para ninguém refazer)

- ⛔ **Não** balancear metadata (`-musage`): ela está a 82-91% de uso **por estar certa**; relocar
  metadata quase cheia só gasta I/O e precisa do não-alocado que ainda não existe. Só dados.
- ⛔ **Não** aumentar o zram de 32 GiB (a configuração de 08/08 já baixou de 123): o problema não é
  swap pequeno, é um tmpfs de 33 GB a morar nele. Retirado o tmpfs, o swap sobra.
- ⛔ **Não** usar `swapoff/swapon` como cura: traz os 30 GB de volta para RAM residente e no próximo
  pico vão de novo. A cura é o §2.
- ⛔ **Não** tratar `cargo clean -p` como cura do SIGBUS: cura o **sintoma** (apaga o artefato
  corrompido). Sem o §3 o próximo build escreve outro.
- ⛔ **Não** pôr `+C` em fonte, `.git` ou `~/.cache/sccache`: são os canários de checksum que restam.
- ⚠️ **Alternativa não testada:** o kernel tem *periodic reclaim* de chunks
  (`/sys/fs/btrfs/<uuid>/allocation/data/periodic_reclaim`, com `bg_reclaim_threshold=75` já
  exposto): um balance automático por commit de transação. Não foi ligado — o timer é mais previsível
  e o comportamento sob 32 núcleos a escrever não está medido. Fica como hipótese com endereço.
- ⚠️ O `.typos`/`ship.sh` não sabem nada disto: a saúde do disco **não é gate de CI**, é gate de
  **jornada** — por isso `hw-profile.sh` (o primeiro comando de todo agente) passa a apontar
  `btrfs-health.sh` pelo nome, e a DIRETIVA_FIM_DE_DIA o chama **antes** dos 3 portões.
