# I4 — a lista de completação aberta na bancada tecla→pixel (17/10)

Sonda `bancada::tecla_pixel` (`crates/ph2d-app-code/tests/it/bancada.rs`), com as duas células novas «lista aberta»: o servidor falso propõe `dtxwmzk` e 17 `dt_<membro>`, a lista abre na palavra `dt` do fim da linha-alvo e fica aberta em TODA a tecla (asserção fora do relógio: a lista pintada é a da célula). Mesmo processo, intercalado com as outras células.

Re-correr: `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-code --test it bancada -- --ignored --nocapture`

⚠️ `loadavg` 88 no início (outras linhas a compilar): as caudas valem pouco; as réplicas da célula 51 distam até 0,95 ms no p99.9. A comparação que vale é a do par na mesma rodada (p50).

```
E14 — perfil otimizado (smoke/release) · placa NVIDIA GeForce RTX 5060 Ti (Vulkan) · loadavg início 88.16 68.33 46.68 · fim 39.02 58.61 45.33 · rodada 89.5 s
alturas da área: ~43 linhas = 862 px · 100 linhas = 1945 px (largura 1600)

Tecla→pixel, ms (min / p50 / p99 / p99.9); fases em p50; n por célula = 1000
| cena · documento · vista | linhas pintadas | 1.º quadro | tecla→pixel | tecla (buffer+AutoEdit) | quadro (+cena) | paint | placa + leitura | a11y por tecla | paint sem mudança |
|---|---|---|---|---|---|---|---|---|---|
| parada · bob.luau (51) · ~43 linhas | 43 | 16.6 | 1.09 / 1.50 / 3.24 / 4.16 | 0.034 | 0.004 | 0.598 | 0.829 | 0.07 / 0.10 / 0.19 / 0.23 | 0.40 / 0.50 / 0.85 / 1.16 |
| a tocar · bob.luau (51) · ~43 linhas | 43 | 4.1 | 1.12 / 1.49 / 3.81 / 4.47 | 0.034 | 0.033 | 0.573 | 0.793 | 0.07 / 0.09 / 0.20 / 0.25 | 0.41 / 0.51 / 0.92 / 1.28 |
| parada · 7 500 · ~43 linhas | 43 | 3.8 | 1.07 / 1.44 / 3.04 / 4.15 | 0.041 | 0.004 | 0.553 | 0.816 | 0.07 / 0.11 / 0.22 / 0.23 | 0.36 / 0.48 / 0.86 / 1.51 |
| a tocar · 7 500 · ~43 linhas | 43 | 3.4 | 1.13 / 1.48 / 3.62 / 4.43 | 0.040 | 0.068 | 0.549 | 0.805 | 0.07 / 0.10 / 0.21 / 0.30 | 0.37 / 0.47 / 0.84 / 0.94 |
| parada · bob.luau (51) · 100 linhas | 52 | 6.7 | 1.25 / 1.72 / 4.04 / 4.74 | 0.036 | 0.004 | 0.719 | 0.928 | 0.08 / 0.11 / 0.22 / 0.31 | 0.48 / 0.62 / 1.13 / 1.58 |
| a tocar · bob.luau (51) · 100 linhas | 52 | 3.8 | 1.28 / 1.74 / 3.53 / 4.82 | 0.037 | 0.036 | 0.687 | 0.913 | 0.08 / 0.11 / 0.23 / 0.29 | 0.49 / 0.62 / 1.10 / 1.41 |
| parada · 7 500 · 100 linhas | 100 | 6.0 | 1.77 / 2.26 / 4.99 / 5.64 | 0.042 | 0.005 | 0.993 | 1.186 | 0.15 / 0.18 / 0.36 / 0.41 | 0.74 / 0.88 / 1.53 / 1.77 |
| a tocar · 7 500 · 100 linhas | 100 | 5.4 | 1.82 / 2.40 / 4.95 / 6.12 | 0.043 | 0.076 | 1.054 | 1.207 | 0.15 / 0.19 / 0.36 / 0.50 | 0.75 / 0.94 / 1.59 / 2.17 |
| parada · bob.luau (51) · ~43 linhas (réplica) | 43 | 3.4 | 1.09 / 1.44 / 3.57 / 4.39 | 0.034 | 0.004 | 0.567 | 0.803 | 0.07 / 0.10 / 0.18 / 0.24 | 0.40 / 0.50 / 0.94 / 1.63 |
| a tocar · bob.luau (51) · ~43 linhas (réplica) | 43 | 5.0 | 1.13 / 1.46 / 3.47 / 5.42 | 0.033 | 0.033 | 0.564 | 0.782 | 0.07 / 0.09 / 0.19 / 0.31 | 0.41 / 0.49 / 0.85 / 1.05 |
| parada · bob.luau (51) · ~43 linhas · lista aberta | 43 | 4.1 | 1.11 / 1.49 / 3.42 / 4.55 | 0.045 | 0.008 | 0.594 | 0.817 | 0.08 / 0.10 / 0.22 / 0.25 | 0.41 / 0.53 / 1.01 / 1.28 |
| parada · 7 500 · ~43 linhas · lista aberta | 43 | 3.5 | 1.09 / 1.46 / 3.84 / 5.15 | 0.050 | 0.008 | 0.558 | 0.819 | 0.08 / 0.11 / 0.22 / 0.31 | 0.37 / 0.46 / 0.87 / 1.02 |

Regra de Fatin (vista ~43 linhas, tecla→pixel, ms): 7 500 − max(51 a, 51 b) ≤ |51 a − 51 b|
| cena | quantil | 51 (a) | 51 (b) | ruído | 7 500 | excesso | veredito |
|---|---|---|---|---|---|---|---|
| parada | min | 1.091 | 1.093 | 0.002 | 1.067 | -0.026 | passa |
| parada | p50 | 1.501 | 1.435 | 0.065 | 1.442 | -0.059 | passa |
| parada | p99 | 3.241 | 3.575 | 0.334 | 3.040 | -0.534 | passa |
| parada | p99.9 | 4.159 | 4.391 | 0.233 | 4.153 | -0.238 | passa |
|   fases · parada · bob.luau (51) · ~43 linhas | tecla 0.02 / 0.03 / 0.11 / 0.15 · quadro (+cena) 0.00 / 0.00 / 0.02 / 0.03 · paint 0.44 / 0.60 / 1.08 / 1.17 · placa 0.62 / 0.83 / 2.33 / 3.36 |
|   fases · parada · bob.luau (51) · ~43 linhas (réplica) | tecla 0.02 / 0.03 / 0.15 / 0.25 · quadro (+cena) 0.00 / 0.00 / 0.02 / 0.02 · paint 0.44 / 0.57 / 1.03 / 1.15 · placa 0.62 / 0.80 / 2.84 / 3.38 |
|   fases · parada · 7 500 · ~43 linhas | tecla 0.03 / 0.04 / 0.35 / 1.18 · quadro (+cena) 0.00 / 0.00 / 0.02 / 0.04 · paint 0.40 / 0.55 / 0.97 / 1.13 · placa 0.63 / 0.82 / 2.31 / 3.13 |
| a tocar | min | 1.123 | 1.133 | 0.010 | 1.127 | -0.006 | passa |
| a tocar | p50 | 1.489 | 1.459 | 0.030 | 1.478 | -0.012 | passa |
| a tocar | p99 | 3.807 | 3.474 | 0.332 | 3.618 | -0.188 | passa |
| a tocar | p99.9 | 4.469 | 5.415 | 0.947 | 4.425 | -0.990 | passa |
|   fases · a tocar · bob.luau (51) · ~43 linhas | tecla 0.02 / 0.03 / 0.13 / 0.16 · quadro (+cena) 0.02 / 0.03 / 0.11 / 0.14 · paint 0.45 / 0.57 / 1.02 / 1.27 · placa 0.62 / 0.79 / 2.89 / 3.60 |
|   fases · a tocar · bob.luau (51) · ~43 linhas (réplica) | tecla 0.02 / 0.03 / 0.12 / 0.19 · quadro (+cena) 0.02 / 0.03 / 0.10 / 0.14 · paint 0.45 / 0.56 / 1.02 / 1.20 · placa 0.63 / 0.78 / 2.64 / 4.61 |
|   fases · a tocar · 7 500 · ~43 linhas | tecla 0.02 / 0.04 / 0.68 / 1.16 · quadro (+cena) 0.05 / 0.07 / 0.16 / 0.20 · paint 0.39 / 0.55 / 0.99 / 1.34 · placa 0.63 / 0.81 / 2.55 / 3.57 |

Ctrl+S → selo «Reloaded · 3 objects» (cena a tocar), ms; n = 1000
| documento | Ctrl+S → selo | dos quais a recarga (`scene_sync` + passo) |
|---|---|---|
| 51 linhas | 0.04 / 0.07 / 0.25 / 0.35 | 0.04 / 0.06 / 0.24 / 0.33 |
| 7500 linhas | 2.53 / 3.33 / 5.29 / 5.53 | 2.50 / 3.27 / 5.17 / 5.34 |

Gramática por tecla, à parte, ms; n = 1000
| documento | reparse incremental (no worker, fora da tecla) | estrutura mudada (`changed_ranges`, só com dobras fechadas) | dobras do ficheiro (`folds`, só o Fold All) | dobras de 100 linhas (`folds_in`, a pintura) | erros | realce de 100 linhas |
|---|---|---|---|---|---|---|
| 51 linhas | 0.02 / 0.02 / 0.10 / 0.12 | 0.01 / 0.01 / 0.04 / 0.04 | 0.05 / 0.05 / 0.13 / 0.15 | 0.05 / 0.05 / 0.10 / 0.18 | 0.00 / 0.00 / 0.00 / 0.00 | 0.21 / 0.27 / 0.43 / 0.46 |
| 7500 linhas | 0.38 / 0.59 / 1.71 / 2.22 | 0.34 / 0.43 / 0.95 / 1.13 | 4.84 / 7.08 / 11.68 / 12.46 | 0.07 / 0.08 / 0.15 / 0.18 | 0.00 / 0.00 / 0.00 / 0.00 | 0.36 / 0.44 / 0.76 / 0.86 |
test bancada::tecla_pixel ... ok


```
