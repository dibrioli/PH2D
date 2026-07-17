---
name: project-modo-l-speed-hole-worktree-targets-slow-path
description: No Modo L a lentidão com N agentes paralelos NÃO é RAM — é disco + recompilação 6×. O tmpfs foi ligado só ao primário e o sccache é pequeno demais para servir N worktrees.
metadata:
  type: project
---

Diagnóstico (2026-07-14, jornada de 6 linhas na workstation 128 GB): a jornada
ficou lenta e a intuição foi "o target ficou gigante / estourou a RAM". **A RAM
nunca foi o gargalo** — pico de 16 GiB usados de 123, tmpfs a 8%. O gargalo é
**disco + recompilação redundante**, e tem 3 causas estruturais:

1. **Recompilação 6×, porque o cache compartilhado é pequeno demais.** O Modo L
   dá um `target/` por worktree (isolamento). O sccache existe pra tornar os
   DEPS grátis entre worktrees (1º compila, os outros pegam hit). Mas o cap era
   **30 GB** e a parte de deps de UM só target (wasmtime/bevy/vello/naga/88
   crates-nó) já passa disso → LRU-thrash → hit rate desaba → cada worktree
   compila tudo do zero. **836 GB de target somados = ~6 cópias do mesmo build
   de deps.**

2. **O truque de velocidade foi pro lugar errado.** `target/` em tmpfs
   (`/dev/shm/ph2d-target`) foi ligado SÓ ao primário — o único checkout que
   NÃO faz o trabalho paralelo. As 6 worktrees, onde os builds de fato rodam,
   ficaram no btrfs com `compress=zstd:1` + CoW + `discard=async`, a 84% cheio —
   caminho patológico pra um target dir (zstd comprime objeto descartável, CoW
   multiplica escrita de arquivinhos, 6 escritores num NVMe = I/O saturado).

3. **Oversubscription de CPU:** cada `cargo build` usa `-j num_cpus` por padrão;
   6 × 32 ≈ 190 threads em 32 cores. O `hw-profile` "5 build / 10 check" é
   quantas WORKTREES rodar, não capeia o `-j` interno de cada uma.

**Consertos APLICADOS (2026-07-14):**
- **sccache 30 G → 100 G** em `~/.cargo/config.toml` `[env]`, daemon reiniciado
  com o cap (o `--show-stats` reportava "Max 10 GiB" = daemon subiu sem o env
  `SCCACHE_CACHE_SIZE`; reiniciar assa o cap novo). Verificado: check num target
  FRIO deu **77,8% de hit** e terminou em 1,4s → 6× build de deps virou ~1×.
- **`chattr +C` (nodatacow) nos 6 target dirs** recriados vazios → builds futuros
  nascem sem CoW e sem zstd (verificado: `.rmeta` novo herda o `C`).
- **`rm -rf` nos 6 targets** liberou o disco de **84% → 8%** (btrfs a >80% já
  está no regime lento). Fonte/git intactos — `target/` nunca contém fonte nem
  objetos git (a worktree os guarda no `.git`).

**Conserto NÃO persistido de propósito:** capear `CARGO_BUILD_JOBS` ≈ cores/N.
Um `[build] jobs=N` global REGRIDE o build solo diário (que quer os 32 cores).
É knob POR-JORNADA (exportar no shell de cada agente quando rodar N em paralelo),
não config permanente.

Regra: no Modo L a métrica de velocidade a vigiar é **hit rate do sccache** e
**onde os TARGETS DAS WORKTREES moram**, não a RAM. Relaciona: [[feedback_measure_perf_symptom_scale]]
(meça a escala/classe antes da causa) e o ADR-0104 (estratégia de velocidade
por tier de hardware).
