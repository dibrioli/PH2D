---
name: reference-one-field-eval-test-costs-55-minutes-of-the-debug-suite
description: A suíte da workspace em DEBUG parece pendurada por UM teste — `ph2d-field-eval::the_census_of_every_primitive` custa 55 min sozinho (medido 2026-09-09).
metadata:
  type: reference
---

Correr `cargo nextest run --workspace` em **debug** dá a impressão de estar pendurado. Não está:
é **um** teste.

Medido em 2026-09-09 (`line/UIUX`, binário isolado, máquina com carga baixa):

```
Summary [3308.316s] 28 tests run: 28 passed (7 slow), 0 skipped
```

— e `3 308 s` = **55 minutos**, quase todos num só caso:
`ph2d-field-eval::the_census_of_every_primitive every_row_of_every_primitive_marches_safely_across_its_range`.
Outros do mesmo binário passam dos 2 min (`every_counted_shape_marches_safely_at_its_own_ceiling`
mede `144 s`).

**Why:** uma corrida de fecho que apanhe este binário fica ~1 h sem imprimir veredito nenhum, e a
reacção natural — matá-la — deita fora a suíte inteira. Já aconteceu nesta linha: uma corrida foi
morta aos 44 min a olhar para linhas `SLOW` deste mesmo teste. ⚠️ O CI corre com
`--cargo-profile ci-test` (optimizado), então o preço é **do laço local**, não do CI — é relógio
do agente (`CLAUDE.md` §2), não risco de produto.

**How to apply:** ao fechar uma linha, corra a workspace **sem** este binário e corra-o **à parte**,
com o veredito lado a lado — nunca o exclua em silêncio. O filtro certo é
`-E 'binary(the_census_of_every_primitive)'`; ⛔ `test(/the_census_of_every_primitive/)` casa
**ZERO** (o `test()` do nextest lê o nome do TESTE, não o do binário) e devolve `exit 4` com
*«no tests to run»* — que se lê como verde se ninguém olhar
([[feedback_a_swallowed_panic_silently_shrinks_the_candidate_set]] é a mesma família).
O dono é a `line/3DModeling`; acelerá-lo é wave dela.
