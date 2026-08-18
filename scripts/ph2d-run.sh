#!/usr/bin/env bash
# ph2d-run — roda um comando (cargo/teste/smoke) num scope PRÓPRIO, com teto de
# memória e SEM swap.
#
# POR QUE ISTO EXISTE (2026-08-14 23:26): um teste da `ph2d-painter-brush`
# alocou 90,2 GB. O kernel matou o processo certo, mas ele rodava dentro do
# `app-code-1844.scope` — o cgroup do VSCode, porque quem o lançou foi um agente
# na janela — e o systemd, com `OOMPolicy=stop`, parou o scope INTEIRO:
#
#   systemd[1223]: app-code-1844.scope: Failed with result 'oom-kill'.
#
# O earlyoom não podia salvar: a condição dele é `mem avail <= 10%` **E**
# `swap free <= 10%`, e no instante do OOM havia 27,6 GB de RAM livres (~23%).
# O que tinha acabado era o SWAP (24 kB de 32 GB) — o modo de falha do zram,
# que vive dentro da RAM.
#
# Um scope próprio remove as duas metades de uma vez: o cgroup mata só ESTE
# comando, a janela do agente não é tocada, e o teto é atingido em ~1 s em vez
# de a máquina inteira entrar em reclaim. Medido no mesmo dia: sob
# `MemoryMax=8G` o teste morreu sozinho e o VSCode ficou de pé.
#
# USO:
#   ph2d-run cargo test -p ph2d-painter-brush --release
#   ph2d-run cargo build -p ph2d-host-desktop --release
#   PH2D_MEM_MAX=48G ph2d-run cargo test --workspace     # teto maior p/ gate
#
# ⚠️ `MemorySwapMax=0` é metade da proteção, não um detalhe: sem ele o kernel
# comprime o excesso para o zram (32 GiB) e o que morre não é o comando — é a
# capacidade de reclaim da máquina inteira. O teto tem de ser em RAM.
#
# ⚠️ O teto NÃO é um palpite escondido: 24 GiB é ~20% dos 123 GiB da máquina,
# folgado para um `rustc` com LTO (8-16 GiB no pior crate deste repo) e apertado
# o bastante para que cinco linhas paralelas não somem a RAM inteira. Se um
# build legítimo bater nele, suba pelo env — mas MEÇA antes de subir, porque
# bater em 24 GiB é mais provavelmente um produtor sem teto que um build grande.
set -euo pipefail

if [ $# -eq 0 ]; then
  echo "uso: ph2d-run <comando> [args...]" >&2
  echo "     PH2D_MEM_MAX=48G ph2d-run cargo test --workspace" >&2
  exit 64
fi

MAX="${PH2D_MEM_MAX:-24G}"

exec systemd-run --user --scope --quiet --collect \
  --description="ph2d-run: $*" \
  -p MemoryMax="$MAX" \
  -p MemorySwapMax=0 \
  -- "$@"
