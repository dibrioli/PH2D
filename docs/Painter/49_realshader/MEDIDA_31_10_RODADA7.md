# Medida 31/10, rodada 7 — a advecção FUNDIDA por blocos de linhas (ao bit), base `9ef95d36a7`

Cena e régua as das rodadas 3–6: o wash de `nucleos_wash` (4096×2560, `Lei::Cobre`), perfil `smoke`
(`codegen-units = 1` no `ph2d-wet-paint`), `PH2D_RELEASE="mede velocidade"` pela fatia da linha, pool `rayon` de
**16 fios**. Sonda temporária `mede_rodada7_tmp.rs` (apagada, não comitada): wash pintado uma vez, `layers`+`sim`
restaurados antes de CADA corrida; por corrida, quadros 1–78 (tempo dos quadros 7–78), hash FNV da grelha
(`film`, `susp`, `susp_rgb`, `vel_x`) nos quadros 6 · 42 · 78; **8 rondas × 4 variantes intercaladas com a ordem
rodada**, overrides EM EXECUÇÃO (`ADVECT_FASES`, `BLOCO_LINHAS`); resumo = o MÍNIMO. loadavg 1 min 0,4 → 11 durante
a corrida (só a sonda).

## O corte

`prepare → outflow → transport` passam a correr **por bloco de linhas** (`solver/advect_fundida.rs`): cada bloco
calcula o `uv`/`bt` das suas linhas + halo, a saída das suas linhas + halo e o transporte das suas linhas, com
`uv`/`bt`/`outflow` em planos PEQUENOS por fio (`thread_local`, reutilizados), em vez de planos da grelha inteira
escritos por uma fase e relidos da RAM pela seguinte. As três fases são as MESMAS funções por linha nas duas rotas
(`prepare_row`, `outflow_row`, `transport_row` — refactor do `advect_jacobi.rs`, uma lei só).

- **Halo, derivado do código:** o transporte de `y` lê a saída das linhas `y0, y0+1` com `|y − y0| ≤ R`
  (`R = destination_reach = ceil(maxVelocity)+1`) ⇒ saída em `[a−R, b+R]`, até a linha de borda `h+1` (que a rota
  de fases nunca escreve: zero — o primeiro teste apanhou-a, `sy == h` ⇒ `y0+1 = h+1`); a saída de `y` lê
  `uv`/`bt` de `[y−R, y+R]` ⇒ `uv`/`bt` em `[a−2R, b+2R]`, recortado à faixa `[by0, by1]`.
- **Fora da faixa** a rota de fases lia o rascunho velho, que só multiplica massa zero (módulo do
  `advect_jacobi`); a fundida conta zero.
- O `momentum` fica fora (só escreve `vel_x/vel_y`, que nenhuma das três lê). O Jacobi (lê só o estado anterior,
  escreve `dst_*`) e o commit por troca de planos ficam iguais.
- A rota `Medida` (janelas médias) continua nas fases; `Serial`/`Parallel` (o wash) vão à fundida.

## A medida (16 fios, ms/passo, mínimo de 8 rondas)

| variante | ms/passo | hash 6 / 42 / 78 |
|---|---|---|
| fases (HEAD) | 52,90 | `8906085e3c8cd2c7` `ee910c30c5ccaaab` `9030587bc1867c8d` |
| fundida, bloco 64 | 54,23 | idêntico |
| fundida, bloco 32 | 52,47 | idêntico |
| **fundida, bloco 16 (produto)** | **50,86** | idêntico |

⇒ **−2,0 ms/passo (−3,9 %)**, ao bit nos três quadros e em todos os blocos. Muito menos que os ~10 ms da conta da
rodada 6: blocos maiores PIORAM (64 linhas: 72 × 4098 × 9 B ≈ 2,7 MB de `uv`/`bt` + 1,1 MB de saída por fio ×
16 fios ≫ L2 e disputam a L3), e o transporte continua a ler `film/susp/susp_rgb` (os 32 B/célula do chão medido)
e a escrever `dst_*` com *write-allocate* — que a fusão não toca. A advecção cai de ~31,5 para ~29,5 ms.

## Gates (por nome, `smoke`)

`advect_jacobi_fundida_bits_equal_the_phases` (NOVO: os 4 casos do gate gravado — tecto 1 e 3, KM, série e
paralela — com blocos de 1, 3, 7 e 32 linhas, grelha ao bit contra as fases) · `advect_jacobi_bits_are_the_recorded_ones`
· `advect_jacobi_bits_by_each_commit_route` · `advect_jacobi_commit_writes_exactly_the_live_span` · `flow_bit_tests`
· `a_borda_graduada` · `sim_resumable` · `assenta_bit` — verdes. Os gates que hasheiam o rascunho da grelha inteira
seguram a rota de fases por um CONTADOR (`ADVECT_FASES`), para não se atropelarem em paralelo.

## O que falta (não feito nesta rodada — prazo)

1. **Bloco 8 e 4** (a tendência 64 → 32 → 16 ainda desce) — uma corrida da mesma sonda.
2. **Escritas não-temporais** nos `dst_*` do transporte (e `vel_x/y`): o *write-allocate* (~20 + 8 B/célula) é agora
   a maior parte do que sobra acima do chão; medir contra o chão de 11,6 ms da rodada 6.
3. Fundir o `momentum` no mesmo bloco (relê `film`/`active`).
