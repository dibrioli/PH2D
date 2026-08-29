---
name: project-projects-live-on-a-dedicated-2tb-disk
description: Os projetos moram num disco dedicado (Viper 2TB) montado no MESMO caminho de sempre; /home ficou só com o sistema.
metadata:
  type: project
---

Desde 2026-08-29, `/home/enio/Documentos/Projetos` **não é mais um diretório do
`/home`** — é o ponto de montagem de um disco inteiro e dedicado:

| | |
|---|---|
| dispositivo | `/dev/nvme0n1p1` (Viper VP4300L 2TB), LABEL `projetos` |
| filesystem | btrfs, subvolume `@projetos` (irmão: `@snapshots`, ainda vazio) |
| opções | `noatime,compress=zstd:1,discard=async,nofail` |
| UUID | `a840e453-57f8-4f8d-94f7-5ac43af26454` |

⚠️ **O caminho é IDÊNTICO ao de antes, e isso é a razão de ser do desenho.** Tudo
neste repo depende do caminho absoluto: o `gitdir:` das 6 worktrees, o symlink
`~/.claude/projects/<key>/memory`, e as permissões em `~/.claude/settings.json`.
Montar em qualquer outro lugar obrigaria a reescrever os três.

⚠️ **`nofail` + sentinela:** por baixo da montagem existe um
`LEIA-disco-nao-montado.txt`. Se ele aparecer, o disco não subiu — os projetos
não se perderam, só não estão montados (`sudo mount /home/enio/Documentos/Projetos`).
Sem essa sentinela, um disco não montado leria como *«a pasta ficou vazia»*.

**Medido antes de decidir** (fio, carga < 3, 3 corridas cada, os dois discos em
slots PCIe 4.0 x4 idênticos):

| | XPG (sistema) | Viper (projetos) |
|---|---|---|
| escrita sequencial | 1.400 MB/s | **2.140 MB/s** |
| escrita aleatória 4K | 528 MB/s | **584 MB/s** |
| leitura aleatória 4K | **644 mil IOPS** | 477 mil IOPS |

⚠️ **O disco do sistema ganha em leitura aleatória (+35%) e ainda assim a mudança
está certa:** com 123 GB de RAM a leitura de fonte vem do page cache, e o que
chega ao dispositivo numa build é **escrita** — onde o Viper ganha. O ganho maior
nem é velocidade: é a churn de `target/` sair do disco que hospeda o SO, que é o
mecanismo por trás de [[project_disk_full_corrupts_objects_mold_sigbus]] e de
[[project_btrfs_metadata_starved_not_disk_full_2026_08_22]].

⛔ **`target/` NÃO é subvolume btrfs, e a recusa é medida:** a
`DIRETIVA_FIM_DE_DIA` manda `rm -rf <target>` como limpeza cirúrgica, e `rm -rf`
falha na raiz de um subvolume (`Operation not permitted`). A regra de vocês
escolheu o formato.

⚠️ **`ph2d-btrfs-balance` e `ph2d-btrfs-scrub` cobriam só `/`** — hoje cobrem os
dois discos, por drop-in em `/etc/systemd/system/<unidade>.service.d/10-disco-projetos.conf`,
com guarda `mountpoint -q` (sem ela, o disco desmontado faria o comando recair em
`/` e balancear o do sistema duas vezes).
