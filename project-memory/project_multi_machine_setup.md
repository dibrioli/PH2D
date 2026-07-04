---
name: project_multi_machine_setup
description: Projeto sincronizado em 3 máquinas via GitHub; memória vendorizada no repo por symlink
metadata:
  type: project
---

O PH2D roda **idêntico em 3 máquinas**: Mac mini (testes da game engine / Metal), PC Linux 128 GB (dev rápido), notebook Windows (build target Windows). **GitHub `dibrioli/PH2D` é a fonte única**; cada máquina tem clone LOCAL nativo (NÃO compartilhar o drive externo — APFS não é cross-platform, e o Linux quer NVMe local). Sync = `git pull` no início / `git push` no fim.

**Memória do Claude vendorizada:** os arquivos de memória vivem agora em `project-memory/` no repo (versionados). O Claude Code lê/escreve via **symlink** `~/.claude/projects/<key>/memory` → `project-memory/`. Cada máquina faz esse symlink uma vez no bootstrap. `<key>` = path absoluto do projeto com `/`→`-` (difere por máquina; por isso o symlink). `project-memory/**` está excluído do typos.

**Runbook completo:** [`docs/DevOps/MULTI_MACHINE_SETUP.md`](../docs/DevOps/MULTI_MACHINE_SETUP.md) — bootstrap por-OS, secrets fora do git (`docs/_api-claude.md`, `.env`), `.gitattributes` (LF em todas), linker per-usuário global (nunca no repo — cerca de Chesterton em `.cargo/config.toml`), e rebaseline da stack de velocidade no Linux (o teto "≤3 cargos"/rust-analyzer-bloqueado era pelos 8 GiB do Mac).

**Landou 2026-07-04:** `.gitattributes` (LF), memória vendorizada + symlink no Mac, docs de plano do Deform. Ver também [[project_diretriz_v68_2026_05_22]].
