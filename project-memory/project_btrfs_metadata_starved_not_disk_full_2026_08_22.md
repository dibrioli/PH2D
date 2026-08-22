---
name: project_btrfs_metadata_starved_not_disk_full_2026_08_22
description: "«Disco cheio» com 526 GB livres = METADATA do btrfs sem espaço para crescer (não-alocado 0); o target do primário em tmpfs vivia no zram (swap 100%); e o csum corrompido de artefatos novos começou com o kernel 7.2.0 — três doenças que pareciam «lentidão com várias linhas»"
metadata:
  type: project
---

Medido em 2026-08-22 na workstation (CachyOS, btrfs `compress=zstd:1`, NVMe 950 GB, zram 32 GiB),
depois de uma jornada Modo L em que «o disco encheu» a meio de um build e o `mold` morria em SIGBUS
em laço. Eram **três** coisas, cada uma com instrumento próprio — `bash scripts/btrfs-health.sh`
(sem root) mede as três; runbook em `docs/DevOps/BTRFS_METADATA_E_SWAP.md`.

1. **Metadata faminta, não disco cheio.** `df` dizia 45%; o btrfs tinha **937,85 GiB dos 950
   alocados a blocos de DADOS** (410 usados), **0 byte não-alocado**, e a metadata (6 GiB DUP,
   5,46 usados) sem ter de onde crescer → `ENOSPC` com 526 GB «livres». O `rm -rf` do fim de dia
   liberta espaço *dentro* dos blocos; só blocos 100% vazios voltam ao não-alocado. Cura = **root**:
   `sudo btrfs balance start -dusage=10 /` (e 30); prevenção = timer semanal
   (`docs/DevOps/systemd/`, o `btrfsmaintenance` é AUR nesta máquina). ⛔ Não balancear metadata.
2. **Swap 32/32 GiB com 61 GiB de RAM livre.** O `target/` do primário → `/dev/shm` (tmpfs) tinha
   33 GB; `Shmem` residente 3,1 GB, zram com 31,6 GB de dados em 12,1 GiB, e o `VmSwap` de todos os
   processos somava 1,5 GB — **o swap ERA o target**. É o modo de falha de 14/08 (swap acaba com RAM
   sobrando). tmpfs **retirado** do tier (ADR-0104 emenda); `scripts/target-to-disk.sh` migra
   preservando o build, e apagar o tmpfs devolve o swap na hora.
3. **Checksum corrompido em arquivos NOVOS, correlacionado com o kernel.** `csum failed` em dezenas
   de inodes recém-criados: **0** nos dois boots com 7.1.8, **100 e 108 (+suprimidos)** nos dois com
   **7.2.0-1** (instalado 21/08 00:04); `corruption_errs 2804`; **33 coredumps** de `mold`/`rustc`
   desde o 1º boot no 7.2.0 contra 0 nos 5 dias antes, cada um segundos depois de um `csum failed`
   (`mmap` de página com EIO ⇒ SIGBUS). A/B barato: `linux-cachyos-lts` 6.18.42 já instalado. Se
   o LTS não zerar, volta a hipótese de DDR5 marginal de
   [[project_workstation_freeze_memory_reclaim]] (memtest86+ overnight). ⚠️ Nos targets com
   `+C` (nodatacow) a mesma corrupção é **muda** — não há checksum.

**Why:** os três sintomas chegam como «a máquina está lenta/instável com vários agentes», e cada
um pede uma cura diferente; a primeira leitura de outra sessão no mesmo dia foi «disco cheio trunca
os .o» ([[project-disk-full-corrupts-objects-mold-sigbus]]) — o sintoma estava certo, a causa não:
o disco nunca encheu, e os SIGBUS de 17:24 e 18:25 seguiram `csum failed` no journal.

**How to apply:** antes de qualquer limpeza de disco ou de culpar carga/RAM, `btrfs-health.sh`.
«Não-alocado 0» ⇒ balance (Enio, root), não `rm -rf`. Swap cheio com RAM livre ⇒ procure um tmpfs
grande, não aumente o zram. SIGBUS do linker ⇒ `journalctl -k | grep 'csum failed'` e o kernel
por boot (`journalctl -b -N -k | grep -m1 'Linux version'`) antes de `cargo clean -p`, que só cura o
sintoma. Ver também [[project_modo_l_speed_hole_worktree_targets_slow_path]] (o +C nos targets) e
[[project_vscode_dies_by_oompolicy_not_by_choice]] (o AND do earlyoom).
