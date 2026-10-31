# Medida 31/10, rodada 2 — secagem + advecção + undo (wash 4096×2560)

Cena e binário: `nucleos_wash` ([B_nucleos.md](B_nucleos.md)), perfil `smoke` com `STRIP=none`
`DEBUG=line-tables-only` (os três iguais), um target por cópia `git worktree add --detach`, binário corrido
directo. 15 s de água por corrida (54–90 passos, secagem 1 em 6). **A** `47d8c1083e` (antes da secagem) ·
**B** `85c0a42c5b` (secagem cortada) · **C** `b4119a7af0` (+ advecção P2 + undo P1).

⚠️ **Sob carga:** loadavg 45–105 em toda a rodada (outras janelas a compilar). O ms/passo absoluto é
«sob carga» (a B de 30/10, com load 6–9, leu 128–133 ms); as RAZÕES são a régua.

## Intercalado, 6 rondas, ordem rodada

| ronda | ordem | A ms | B ms | C ms | A/C | B/C | load (início→fim) |
|---|---|---|---|---|---|---|---|
| 0 | A B C | 294,1 | 254,5 | 214,9 | 1,37 | 1,18 | 44,7 → 82,6 |
| 1 | B C A | 236,1 | 244,4 | 202,7 | 1,16 | 1,21 | 82,6 → 85,6 |
| 2 | C A B | 185,3 | 194,3 | 200,2 | 0,93 | 0,97 | 85,6 → 48,9 |
| 3 | A B C | 247,6 | 200,5 | 162,4 | 1,52 | 1,23 | 44,7 → 74,7 |
| 4 | B C A | 215,3 | 183,5 | 175,1 | 1,23 | 1,05 | 74,7 → 60,5 |
| 5 | C A B | 232,2 | 181,3 | 219,1 | 1,06 | 0,83 | 60,5 → 51,7 |

| | A | B | C |
|---|---|---|---|
| ms/passo MÍNIMO (sob carga) | 185,3 | 181,3 | **162,4** |
| mediana | 234,2 | 197,4 | 201,5 |
| passos/s máx | 5,40 | 5,52 | **6,16** |
| RSS máx | 4,38 GB | 4,38 GB | **1,81 GB** |
| **B/px** | 428 | 428 | **177** |

Razões pelo mínimo: **A/C = 1,14 · B/C = 1,12** (A/B 1,02). Mediana das razões por ronda: A/C 1,20 · B/C 1,12.
⚠️ A mediana de B (197) fica abaixo da de C (201): mínimo e mediana não concordam sobre B×C — com esta carga a
régua de tempo não decide B contra C ao 10 %. A memória decide: **177 B/px, abaixo do Rebelle (~230–260)**.
Rebelle: 37,3 ms/passo 4K ⇒ C (162 ms sob carga; ~110–130 calmo, por estimar) está ainda ~3–4× atrás.

## Perfil de C (perf record -F 999 -g, 10 s de água, load 39 → 46)

74 passos a 176,6 ms/passo; 131,6 k amostras. ms por passe = partilha × 176,6 (sob carga).

| passe | % | ms/passo |
|---|---|---|
| secagem (`assenta.rs:385` + `ponto.rs` + `colorops::Perfil::de` + `drying.rs:448` + `pow`) | 46,6 | 82,3 |
| advecção (`advect_jacobi.rs` transport/outflow/prepare) | 21,8 | 38,4 |
| fluxo (`flow_jacobi.rs:179` + `flow.rs:233`) | 9,8 | 17,2 |
| libc sem símbolo (memcpy/memset provável) | 7,0 | 12,3 |
| momentum (`advect_jacobi.rs:228`) | 3,6 | 6,3 |
| suavizar (`solver.rs:448`) | 3,5 | 6,1 |
| projecção | 2,3 | 4,0 |
| f32→f64 · região activa · outros | 4,1 | 7,3 |
| **espera** (rayon/crossbeam/futex) | **1,3** | 2,4 |

Top 12 (self): `transport_rows` (`advect_jacobi.rs:393`) 12,6 · `assenta` (`assenta.rs:385`) 7,5 ·
`outflow_rows` (`:326`) 6,6 · `Parada::d` (`assenta/ponto.rs:37`) 6,6 · `assenta{closure#5}` 4,9 ·
`flow_rows` (`flow_jacobi.rs:179`) 4,8 · `flow_at_point` (`flow.rs:233`) 3,9 · `prepara{closure#2}`
(`ponto.rs:141`) 3,6 · libc `0x1b7d6b` 3,6 · `momentum_rows` (`:228`) 3,6 · `smooth_velocity_rows` 3,5 ·
`drying_pass_jacobi_rows` (`drying.rs:448`) 3,1 · (`prepare_rows` `:156` 2,6 · `pow` 2,5).

**Leitura:** a secagem continua a ser quase metade do passo médio (média de 1 passo em 6 que seca);
a advecção caiu de 33,7 % para 21,8 %. Próximo alvo: `assenta`/`Parada::d`/`prepara` (f64, célula a célula).
