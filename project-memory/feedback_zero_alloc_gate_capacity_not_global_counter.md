---
name: feedback-zero-alloc-gate-capacity-not-global-counter
description: Gate HR-3 de zero-alloc que diffa o total_blocks GLOBAL do dhat numa janela é flaky (o contador do processo pega alocação de qualquer thread/harness). Asserte estabilidade de CAPACIDADE dos buffers — determinístico.
metadata:
  type: feedback
---

`dhat::HeapStats::total_blocks` é um contador **global do processo** — incrementa em QUALQUER alocação (harness de teste, threads de fundo), não só no código sob teste. Um gate que faz `assert_eq!(after.total_blocks - before.total_blocks, 0)` sobre uma janela de N iterações **é flaky**: o MESMO código passa numa run de CI e falha noutra com um punhado de blocks espúrios (aconteceu no `layers_no_alloc_hot_compose`, ubuntu+macOS, 2026-07-05 — 4 blocks; localmente instrumentei e a janela era 0 estável).

**Why:** a janela é wall-clock; alocações não-relacionadas caem dentro dela nondeterministicamente. Localmente (janela curta, sem carga) dá 0; no CI, ocasionalmente, o ruído entra.

**How to apply:** se o hot path só aloca via `Vec`/buffers conhecidos (puro `clear`+`push`, sem `Box`/`format!`/temporários — confirme LENDO o código), **asserte estabilidade de CAPACIDADE** dos buffers ao longo do loop quente (`assert_eq!(scratch.capacity(), warm_cap, …)`) em vez de diffar o contador global. É determinístico, imune ao ruído do processo, e nomeia a falha exata (um realloc) numa regressão. Se com isso não sobra uso de dhat na crate, remova o dev-dep `dhat` do `Cargo.toml` (senão `cargo machete` no CI reclama de unused). Vale para os gates irmãos (`*_no_alloc_hot_path` em `ph2d-ecs`/`editor-core`/`painter`) se algum flakar. Liga em [[feedback-audit-scope-discipline]] (o teste era de crate alheio — só mexi com aval do Enio).
