# Fluxo e cópias do passo — cortes ao bit (31/10)

Régua: `flow_bit_tests` (hash FNV-1a de film/susp/susp_rgb/sett/sett_rgb/vel/flow/wet/active/bloom
após 14 `step_simulation`, sem e com gravidade), gravado em `cfa0a413c3` antes de mexer:
`[0xf959f6a57bbfc1b2, 0x4650940dd0f2d7fd]`. Corre as rotas do commit 1 (cópia) e 2 (troca) e a do produto.

## De onde vem o «libc sem símbolo» (7 %)

`perf record -F 199 --call-graph dwarf` no `nucleos_wash` 4K (8 s de água, load ~47), binário `smoke` com
`STRIP=none DEBUG=line-tables-only`. O endereço `libc 0x1b7d6b` é `__memcpy_avx512_unaligned_erms`.
Primeiro chamador das 3 326 amostras em libc:

| chamador | % da libc |
|---|---|
| `advect_jacobi::commit_rows` (3 `copy_from_slice` por linha) | **70,7** |
| crossbeam epoch (rayon) | 10,9 |
| `Vec::extend_from_slice` (undo COW `TileFino::tira`) | ~12 |
| outros | ~6 |

⇒ o memcpy é o **commit da advecção**, não o undo.

## Corte 1 — commit da advecção: troca de planos + cópia do complemento

Com a faixa viva ≥ 80 % das células (`(cells − viva)·4 ≤ viva`, o wash), `commit_rows` faz `mem::swap`
de `film/susp/susp_rgb` com `scratch.dst_*` e copia de volta só o COMPLEMENTO (fora da faixa, do plano
antigo). Não precisa do invariante «fora da faixa = 0»: o complemento é copiado. Faixa pequena ⇒ a cópia
da faixa de antes. Override em execução: `flow::COMMIT_ROTA` (0 produto · 1 cópia · 2 troca).
Hash igual nas três rotas; `advect_jacobi_bits`, `order_invariant`, `sim_resumable` verdes.

⚠️ Ganho em ms **por medir** (sonda `sonda_fluxo` não feita — prazo). Estimativa pelo perfil: ~70 % de
12,3 ms ≈ 8 ms/passo no wash 4K.

## Por fazer

- `examples/sonda_fluxo.rs`: rotas 1×2 intercaladas no mesmo processo (MEDIR_VELOCIDADE).
- `flow_at_point` (2,4 % self): chamado por célula no `momentum_rows` (`advect_jacobi.rs` ~291) com
  ponto retro-traçado arbitrário — não iça por linha como o `FlowRowSampler`; o ramo `rf=1` pode ler
  directo sem clamp/`half`.
- `flow_rows`, suavizar, projecção: não tocados.
