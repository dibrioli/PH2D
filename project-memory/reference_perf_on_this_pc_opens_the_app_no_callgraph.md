---
name: reference_perf_on_this_pc_opens_the_app_no_callgraph
description: perf neste PC - o perf ABRE o app (ptrace_scope), -k CLOCK_MONOTONIC sem -g (paranoid 2), binário smoke é strip=symbols
metadata:
  type: reference
---

`perf record -p <pid>` num não-filho é recusado pelo `ptrace_scope` (Yama): o `perf` tem de ABRIR o app. Com
`perf_event_paranoid=2` o `-g` não abre: use `-k CLOCK_MONOTONIC -F 997` e a amostra é só a função FOLHA (casa com
`[quadro-lento] parede=` de `PH2D_QUADRO_LENTO_MS`). O perfil `smoke` herda `strip=symbols`: compilar com
`CARGO_PROFILE_SMOKE_STRIP=none bash scripts/ph2d-run.sh cargo build --profile smoke -p ph2d-host-desktop`. Corre em Bash com o sandbox
desligado. Roteiro pronto: `docs/Painter/ferramentas/mede_pen_up_4k.sh`; método em `docs/Painter/Pinceis/COMO_ACELERAR_UM_PINCEL.md` §6.
(Corrige a nota antiga «perf bloqueado»: o `perf` simples abre; `samply` e `-g` seguem fechados.)
