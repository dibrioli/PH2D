---
name: project-field-render-it-suite-is-green-single-threaded
description: Os 7-8 vermelhos do `ph2d-field-render --test it` são contadores GLOBAIS a envenenarem-se dentro do binário — 24/24 verde com --test-threads=1, mesmo a load 75
metadata:
  type: project
---

`cargo test -p ph2d-field-render --test it` reprova **7 a 8** gates de orçamento
(`edge_pass_budget`, `march_budget`, `tape_budget`, `tape_cache_budget`,
`tape_cache_alternation`, `the_bend_does_not_starve_the_march`,
`the_piece_does_not_vanish_as_shapes_are_added`, `what_a_stack_of_deformers_costs_the_march`).

⭐ **Com `--test-threads=1`: `24 passed, 0 failed` — e isso medido a `load 75`.**

**Why:** eles leem estáticos de PROCESSO (`TAPE_HITS`, `SPECIALISED`, `TILE_COSTS`, `TILE_MAX`) e
correm no mesmo binário, logo cada um conta o trabalho dos vizinhos. ⚠️ **O discriminador aqui NÃO é
a carga** — é o fan-out **dentro** do binário; a assinatura é o **conjunto de reprovadas MUDAR entre
corridas** (medido: três corridas, três conjuntos, a `load 3,0`, `27` e `40`).

**How to apply:** ao fechar uma linha que toque a `ph2d-field-render`, **não** trate isto como
regressão sua — confirme com `--test-threads=1` antes de olhar para o diff. Pré-existente:
verificado numa worktree em `HEAD` limpo (2026-09-14) com os mesmos gates.
⛔ A cura de fundo é dar a cada um o seu contador (ou um lock de suite), **não** baixar barras.

Espécie: *censo que partilha estado* ([[reference_topic_gate_discipline]]); parente da família de
flakes de recurso do `CLAUDE.md` §5.0, mas com causa **determinística** e por isso curável.
