# Medida 31/10, rodada 8 — blocos 4–16, `momentum` no bloco, stores não-temporais, janelas médias (base `fb8ff6eec7`)

Régua a da [rodada 7](MEDIDA_31_10_RODADA7.md): wash de `nucleos_wash` (4096×2560 salvo onde dito), perfil `smoke`,
`PH2D_RELEASE="mede velocidade"`, fatia da linha, pool de **16 fios**; sonda temporária `mede_rodada8_tmp.rs`
(apagada): `layers`+`sim` restaurados por corrida, quadros 7–78 cronometrados, hash FNV da grelha nos quadros
6 · 42 · 78, **8 rondas intercaladas com a ordem rodada**, overrides EM EXECUÇÃO (`ADVECT_FASES`, `BLOCO_LINHAS`,
`MOMENTO_FORA`), resumo = o MÍNIMO. loadavg 1 min 0,8 → 14 (só a sonda).

## 1. Tamanho do bloco e `momentum` no bloco (4096×2560, ms/passo)

| variante | corrida A | corrida B |
|---|---|---|
| fases (rota antiga) | 52,62 | 52,11 |
| **fundida bloco 16, momentum à parte (produto)** | **50,30** | **50,29** |
| fundida bloco 12, momentum à parte | — | 50,83 |
| fundida bloco 16, momentum no bloco | 51,27 | — |
| fundida bloco 12, momentum no bloco | 50,63 | — |
| fundida bloco 8, momentum no bloco | 51,52 | — |
| fundida bloco 4, momentum no bloco | 55,48 | — |

Hash 6/42/78 `8906085e3c8cd2c7` `ee910c30c5ccaaab` `9030587bc1867c8d` em TODAS as variantes (= HEAD da rodada 6).
⇒ **bloco 16 fica**: 12 empata dentro do ruído, 8 e 4 perdem (o halo de `2R = 4` linhas recalculadas pesa).
⇒ **o `momentum` no bloco NÃO ganha** (51,27 contra 50,30): só poupa reler `film`/`active`, e no bloco disputa a
cache com os planos do bloco. O código fica atrás do override (`MOMENTO_FORA = 0`, só `rf = 1`), coberto pelo gate.
Halo do momentum, derivado do código: zero linhas. A linha de fluxo `cy` lê `film`/`active` só na linha fina probe
`flow_probe(cy)` (= `cy` com `rf = 1`) e o fluxo (que nenhuma fase escreve) no retro-traço.

## 2. Janelas médias — a fundida PERDE abaixo de ~4 M células (fases × fundida bloco 16, momentum à parte)

| faixa | rota | fases | fundida |
|---|---|---|---|
| 4096×2560 (10,5 M) | `Parallel` | 52,11 | **50,29** |
| 2048×1280 (2,6 M) | `Parallel` | **13,03** | 13,16 |
| 1448×900 (1,3 M) | `Parallel` | **6,50** | 6,89 |
| 1000×400 (0,4 M) | `Parallel` | **2,58** | 2,73 |

Hashes iguais entre as rotas em cada tamanho. Abaixo da L3 cheia as fases já moram na cache e o halo só custa ⇒
**`MIN_CELULAS_FUNDIDA = 4 M`** (entre o empate de 2,6 M e o ganho de 10,5 M) e a rota `Medida` (janelas médias)
fica nas fases. O caminho `Medida` da fundida (`par::medida_reduce` sobre os blocos) existe e o gate prova-o ao
bit, mas o produto não o usa.

## 3. Stores não-temporais — ⛔ impossíveis nesta workspace

`_mm_stream_ps`/`_mm_stream_si32` recebem ponteiro cru ⇒ `unsafe`, e `unsafe_code = "forbid"` vale para TODA a
workspace (`Cargo.toml` e `#![forbid(unsafe_code)]` no `lib.rs`). Não há API segura de escrita não-temporal no
`std`. O *write-allocate* dos `dst_*` (~20 B/célula) e do `vel_x/y` (8) fica — cortá-lo só com bytes a menos
(planos `f16`/`u16`, item 3 da rodada 6, com a tolerância invisível provada) ou trocando o commit por escrita
directa no grid (impossível: o Jacobi lê o estado anterior inteiro).

## Gates (por nome, `smoke`)

`advect_jacobi_fundida_bits_equal_the_phases` alargado: 8 casos (tecto 1/3, KM, `Serial`/`Parallel`/`Medida(0)`/
`Medida(MAX)`, `rf` 1/3/4) × blocos 1/3/7/32 × momentum à parte E no bloco, grelha ao bit contra as fases ·
`advect_jacobi_bits_are_the_recorded_ones` · `advect_jacobi_bits_by_each_commit_route` ·
`advect_jacobi_commit_writes_exactly_the_live_span` · `flow_bit_tests` · `a_borda_graduada` · `sim_resumable` ·
`assenta_bit` — verdes.

## ⛔ Recusas MEDIDAS

| proposta | medida | veredito |
|---|---|---|
| bloco 4 / 8 | 55,48 / 51,52 contra 50,3 (16) | ⛔ halo recalculado pesa |
| `momentum` fundido no bloco | 51,27 contra 50,30 | ⛔ não ganha; atrás do override |
| fundida em faixas < 4 M células / `Medida` | 13,16 × 13,03 · 6,89 × 6,50 · 2,73 × 2,58 | ⛔ perde; limiar 4 M |
| stores não-temporais | — | ⛔ exigem `unsafe` (proibido na workspace) |

## Onde estamos

4K, 16 fios: **52,1–52,6 → 50,3 ms/passo** (−2,1 ms, ao bit). Rebelle 37,3. O que sobra na advecção é tráfego do
estado (`film`/`susp`/`susp_rgb` lidos, `dst_*` escritos com *write-allocate*): o próximo corte é de BYTES —
planos mais estreitos com tolerância invisível provada — ou as fases pequenas (suavizar, fluxo, projecção ≈ 11 ms).
