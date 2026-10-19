# I15 — saída bruta da bancada tecla→pixel com a célula completa (19/10)

Re-correr: `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-code --test it bancada -- --ignored --nocapture`. Resumo em [I15_bancada.md](I15_bancada.md).

## Corrida 1 (antes das curas de varredura)

```
E14 — perfil otimizado (smoke/release) · placa NVIDIA GeForce RTX 5060 Ti (Vulkan) · loadavg início 14.00 19.34 25.00 · fim 15.49 18.56 24.06 · rodada 108.5 s
alturas da área: ~43 linhas = 862 px · 100 linhas = 1945 px (largura 1600)

Tecla→pixel, ms (min / p50 / p99 / p99.9); fases em p50; n por célula = 1000
| cena · documento · vista | linhas pintadas | 1.º quadro | tecla→pixel | tecla (buffer+AutoEdit) | quadro (+cena) | paint | placa + leitura | a11y por tecla | paint sem mudança |
|---|---|---|---|---|---|---|---|---|---|
| parada · bob.luau (51) · ~43 linhas | 43 | 17.8 | 1.14 / 1.63 / 3.45 / 4.80 | 0.037 | 0.009 | 0.643 | 0.878 | 0.07 / 0.11 / 0.28 / 0.41 | 0.42 / 0.54 / 1.06 / 1.75 |
| a tocar · bob.luau (51) · ~43 linhas | 43 | 2.8 | 1.18 / 1.56 / 3.78 / 4.96 | 0.036 | 0.040 | 0.608 | 0.839 | 0.07 / 0.10 / 0.25 / 0.34 | 0.43 / 0.53 / 1.20 / 1.57 |
| parada · 7 500 · ~43 linhas | 43 | 2.8 | 1.08 / 1.42 / 3.52 / 4.78 | 0.041 | 0.009 | 0.540 | 0.812 | 0.07 / 0.10 / 0.25 / 0.29 | 0.38 / 0.47 / 1.01 / 2.37 |
| a tocar · 7 500 · ~43 linhas | 43 | 3.8 | 1.16 / 1.58 / 4.87 / 6.81 | 0.042 | 0.075 | 0.571 | 0.868 | 0.08 / 0.10 / 0.26 / 0.37 | 0.39 / 0.48 / 1.07 / 1.49 |
| parada · bob.luau (51) · 100 linhas | 52 | 8.6 | 1.24 / 1.63 / 3.65 / 5.60 | 0.036 | 0.009 | 0.668 | 0.889 | 0.08 / 0.10 / 0.25 / 0.35 | 0.49 / 0.59 / 1.14 / 1.79 |
| a tocar · bob.luau (51) · 100 linhas | 52 | 3.0 | 1.31 / 1.73 / 3.53 / 5.04 | 0.036 | 0.039 | 0.717 | 0.927 | 0.08 / 0.11 / 0.23 / 0.31 | 0.50 / 0.63 / 1.15 / 1.27 |
| parada · 7 500 · 100 linhas | 100 | 5.0 | 1.81 / 2.32 / 5.15 / 7.10 | 0.044 | 0.010 | 1.008 | 1.223 | 0.15 / 0.19 / 0.42 / 0.61 | 0.76 / 0.92 / 1.83 / 2.79 |
| a tocar · 7 500 · 100 linhas | 100 | 3.9 | 1.86 / 2.77 / 5.69 / 8.05 | 0.047 | 0.082 | 1.190 | 1.334 | 0.15 / 0.23 / 0.43 / 0.66 | 0.77 / 1.05 / 2.05 / 3.81 |
| parada · bob.luau (51) · ~43 linhas (réplica) | 43 | 3.8 | 1.13 / 1.59 / 3.94 / 4.89 | 0.036 | 0.009 | 0.611 | 0.851 | 0.07 / 0.10 / 0.26 / 0.33 | 0.42 / 0.53 / 1.13 / 3.07 |
| a tocar · bob.luau (51) · ~43 linhas (réplica) | 43 | 2.7 | 1.17 / 1.58 / 3.15 / 3.91 | 0.036 | 0.040 | 0.603 | 0.851 | 0.07 / 0.10 / 0.23 / 0.32 | 0.42 / 0.52 / 1.19 / 2.91 |
| parada · bob.luau (51) · ~43 linhas · lista aberta | 43 | 2.9 | 1.16 / 1.52 / 3.41 / 4.36 | 0.046 | 0.012 | 0.607 | 0.838 | 0.08 / 0.10 / 0.22 / 0.54 | 0.43 / 0.50 / 1.07 / 2.79 |
| parada · bob.luau (51) · ~43 linhas · luau-lsp: lista + assinatura + cores + dicas | 43 | 4.2 | 1.32 / 1.80 / 3.82 / 5.07 | 0.057 | 0.014 | 0.807 | 0.876 | 0.08 / 0.11 / 0.25 / 0.35 | 0.51 / 0.69 / 1.29 / 1.43 |
| parada · 7 500 · ~43 linhas · lista aberta | 43 | 3.8 | 1.13 / 1.57 / 4.13 / 4.62 | 0.052 | 0.013 | 0.591 | 0.871 | 0.08 / 0.11 / 0.26 / 0.42 | 0.39 / 0.49 / 0.96 / 1.29 |
| parada · 7 500 · ~43 linhas · luau-lsp: lista + assinatura + cores + dicas | 43 | 2.8 | 1.55 / 2.08 / 5.36 / 6.02 | 0.072 | 0.016 | 1.014 | 0.907 | 0.08 / 0.11 / 0.29 / 0.40 | 0.72 / 0.89 / 1.84 / 2.83 |

Regra de Fatin (vista ~43 linhas, tecla→pixel, ms): 7 500 − max(51 a, 51 b) ≤ |51 a − 51 b|
| cena | quantil | 51 (a) | 51 (b) | ruído | 7 500 | excesso | veredito |
|---|---|---|---|---|---|---|---|
| parada | min | 1.143 | 1.131 | 0.012 | 1.077 | -0.067 | passa |
| parada | p50 | 1.633 | 1.590 | 0.044 | 1.420 | -0.213 | passa |
| parada | p99 | 3.448 | 3.936 | 0.488 | 3.517 | -0.419 | passa |
| parada | p99.9 | 4.803 | 4.887 | 0.083 | 4.777 | -0.109 | passa |
|   fases · parada · bob.luau (51) · ~43 linhas | tecla 0.02 / 0.04 / 0.12 / 0.22 · quadro (+cena) 0.00 / 0.01 / 0.05 / 0.09 · paint 0.46 / 0.64 / 1.38 / 1.78 · placa 0.62 / 0.88 / 2.37 / 3.40 |
|   fases · parada · bob.luau (51) · ~43 linhas (réplica) | tecla 0.02 / 0.04 / 0.14 / 0.24 · quadro (+cena) 0.01 / 0.01 / 0.05 / 0.09 · paint 0.46 / 0.61 / 1.28 / 1.71 · placa 0.63 / 0.85 / 2.64 / 3.62 |
|   fases · parada · 7 500 · ~43 linhas | tecla 0.03 / 0.04 / 0.90 / 1.28 · quadro (+cena) 0.01 / 0.01 / 0.04 / 0.06 · paint 0.41 / 0.54 / 1.17 / 2.79 · placa 0.62 / 0.81 / 2.15 / 3.42 |
| a tocar | min | 1.179 | 1.172 | 0.007 | 1.157 | -0.022 | passa |
| a tocar | p50 | 1.562 | 1.576 | 0.014 | 1.584 | +0.008 | passa |
| a tocar | p99 | 3.779 | 3.150 | 0.629 | 4.871 | +1.092 | REPROVA |
| a tocar | p99.9 | 4.957 | 3.911 | 1.046 | 6.811 | +1.854 | REPROVA |
|   fases · a tocar · bob.luau (51) · ~43 linhas | tecla 0.03 / 0.04 / 0.14 / 0.27 · quadro (+cena) 0.03 / 0.04 / 0.18 / 0.23 · paint 0.47 / 0.61 / 1.32 / 1.97 · placa 0.63 / 0.84 / 2.70 / 3.93 |
|   fases · a tocar · bob.luau (51) · ~43 linhas (réplica) | tecla 0.02 / 0.04 / 0.15 / 0.23 · quadro (+cena) 0.03 / 0.04 / 0.17 / 0.22 · paint 0.46 / 0.60 / 1.20 / 1.72 · placa 0.64 / 0.85 / 1.69 / 2.45 |
|   fases · a tocar · 7 500 · ~43 linhas | tecla 0.03 / 0.04 / 0.79 / 1.73 · quadro (+cena) 0.06 / 0.08 / 0.26 / 0.45 · paint 0.42 / 0.57 / 1.24 / 1.75 · placa 0.62 / 0.87 / 3.33 / 5.35 |

Ctrl+S → selo «Reloaded · 3 objects» (cena a tocar), ms; n = 1000
| documento | Ctrl+S → selo | dos quais a recarga (`scene_sync` + passo) |
|---|---|---|
| 51 linhas | 0.04 / 0.07 / 0.31 / 0.49 | 0.04 / 0.07 / 0.29 / 0.45 |
| 7500 linhas | 2.59 / 3.34 / 6.14 / 7.20 | 2.56 / 3.25 / 5.91 / 6.58 |

Gramática por tecla, à parte, ms; n = 1000
| documento | reparse incremental (no worker, fora da tecla) | estrutura mudada (`changed_ranges`, só com dobras fechadas) | dobras do ficheiro (`folds`, só o Fold All) | dobras de 100 linhas (`folds_in`, a pintura) | erros | realce de 100 linhas |
|---|---|---|---|---|---|---|
| 51 linhas | 0.02 / 0.02 / 0.11 / 0.14 | 0.01 / 0.01 / 0.04 / 0.05 | 0.05 / 0.05 / 0.13 / 0.16 | 0.05 / 0.05 / 0.11 / 0.14 | 0.00 / 0.00 / 0.00 / 0.00 | 0.21 / 0.24 / 0.46 / 0.56 |
| 7500 linhas | 0.38 / 0.67 / 2.42 / 3.10 | 0.35 / 0.45 / 1.45 / 1.77 | 4.90 / 7.06 / 13.95 / 15.82 | 0.07 / 0.08 / 0.16 / 0.20 | 0.00 / 0.00 / 0.00 / 0.00 | 0.37 / 0.45 / 0.81 / 1.01 |
```

## Corrida 2 (antes das curas; conta os diagnósticos da célula real)

```
célula real (52 linhas): 1 diagnósticos do assistente · 5 dicas de tipo · lista de 4
célula real (7501 linhas): 293 diagnósticos do assistente · 297 dicas de tipo · lista de 4
E14 — perfil otimizado (smoke/release) · placa NVIDIA GeForce RTX 5060 Ti (Vulkan) · loadavg início 8.70 16.44 23.12 · fim 9.44 14.91 22.00 · rodada 87.2 s
alturas da área: ~43 linhas = 862 px · 100 linhas = 1945 px (largura 1600)

Tecla→pixel, ms (min / p50 / p99 / p99.9); fases em p50; n por célula = 1000
| cena · documento · vista | linhas pintadas | 1.º quadro | tecla→pixel | tecla (buffer+AutoEdit) | quadro (+cena) | paint | placa + leitura | a11y por tecla | paint sem mudança |
|---|---|---|---|---|---|---|---|---|---|
| parada · bob.luau (51) · ~43 linhas | 43 | 12.3 | 1.08 / 1.21 / 3.13 / 4.70 | 0.028 | 0.007 | 0.486 | 0.681 | 0.07 / 0.08 / 0.22 / 0.25 | 0.41 / 0.44 / 1.02 / 1.52 |
| a tocar · bob.luau (51) · ~43 linhas | 43 | 2.2 | 1.10 / 1.23 / 2.65 / 3.86 | 0.028 | 0.031 | 0.484 | 0.678 | 0.07 / 0.08 / 0.18 / 0.22 | 0.42 / 0.44 / 0.90 / 1.04 |
| parada · 7 500 · ~43 linhas | 43 | 2.3 | 1.04 / 1.14 / 2.70 / 3.78 | 0.030 | 0.007 | 0.431 | 0.671 | 0.07 / 0.08 / 0.18 / 0.22 | 0.37 / 0.39 / 0.83 / 0.98 |
| a tocar · 7 500 · ~43 linhas | 43 | 2.2 | 1.09 / 1.20 / 2.99 / 3.92 | 0.030 | 0.059 | 0.435 | 0.670 | 0.07 / 0.08 / 0.19 / 0.26 | 0.38 / 0.40 / 0.84 / 0.90 |
| parada · bob.luau (51) · 100 linhas | 52 | 4.3 | 1.22 / 1.32 / 2.84 / 4.60 | 0.027 | 0.007 | 0.551 | 0.735 | 0.08 / 0.08 / 0.19 / 0.25 | 0.49 / 0.51 / 1.07 / 2.60 |
| a tocar · bob.luau (51) · 100 linhas | 52 | 2.7 | 1.25 / 1.35 / 2.95 / 4.60 | 0.027 | 0.031 | 0.554 | 0.739 | 0.08 / 0.08 / 0.20 / 0.24 | 0.49 / 0.51 / 1.08 / 1.28 |
| parada · 7 500 · 100 linhas | 100 | 3.6 | 1.73 / 1.86 / 3.94 / 5.25 | 0.031 | 0.007 | 0.808 | 1.010 | 0.14 / 0.15 / 0.34 / 0.38 | 0.75 / 0.77 / 1.64 / 3.31 |
| a tocar · 7 500 · 100 linhas | 100 | 3.6 | 1.79 / 1.93 / 4.94 / 6.00 | 0.031 | 0.060 | 0.817 | 1.012 | 0.14 / 0.15 / 0.35 / 0.41 | 0.75 / 0.78 / 1.63 / 1.79 |
| parada · bob.luau (51) · ~43 linhas (réplica) | 43 | 2.2 | 1.07 / 1.17 / 2.47 / 4.59 | 0.026 | 0.007 | 0.469 | 0.661 | 0.07 / 0.08 / 0.16 / 0.29 | 0.41 / 0.43 / 0.84 / 0.90 |
| a tocar · bob.luau (51) · ~43 linhas (réplica) | 43 | 2.4 | 1.14 / 1.20 / 2.48 / 3.08 | 0.027 | 0.031 | 0.476 | 0.663 | 0.07 / 0.08 / 0.17 / 0.20 | 0.42 / 0.43 / 0.89 / 1.00 |
| parada · bob.luau (51) · ~43 linhas · lista aberta | 43 | 2.3 | 1.14 / 1.20 / 2.59 / 4.01 | 0.033 | 0.010 | 0.485 | 0.664 | 0.08 / 0.08 / 0.19 / 0.22 | 0.42 / 0.44 / 0.93 / 0.98 |
| parada · bob.luau (51) · ~43 linhas · luau-lsp: lista + assinatura + cores + dicas | 43 | 2.2 | 1.31 / 1.41 / 4.25 / 6.47 | 0.043 | 0.012 | 0.649 | 0.704 | 0.08 / 0.09 / 0.21 / 0.27 | 0.51 / 0.59 / 1.23 / 2.16 |
| parada · 7 500 · ~43 linhas · lista aberta | 43 | 2.2 | 1.11 / 1.18 / 2.72 / 3.70 | 0.038 | 0.010 | 0.449 | 0.674 | 0.08 / 0.09 / 0.20 / 0.24 | 0.38 / 0.40 / 0.83 / 0.89 |
| parada · 7 500 · ~43 linhas · luau-lsp: lista + assinatura + cores + dicas | 43 | 3.1 | 1.53 / 1.66 / 4.49 / 5.27 | 0.054 | 0.013 | 0.822 | 0.748 | 0.08 / 0.09 / 0.23 / 0.26 | 0.71 / 0.75 / 1.54 / 1.80 |

Regra de Fatin (vista ~43 linhas, tecla→pixel, ms): 7 500 − max(51 a, 51 b) ≤ |51 a − 51 b|
| cena | quantil | 51 (a) | 51 (b) | ruído | 7 500 | excesso | veredito |
|---|---|---|---|---|---|---|---|
| parada | min | 1.078 | 1.066 | 0.012 | 1.036 | -0.041 | passa |
| parada | p50 | 1.206 | 1.167 | 0.039 | 1.140 | -0.066 | passa |
| parada | p99 | 3.132 | 2.474 | 0.659 | 2.698 | -0.435 | passa |
| parada | p99.9 | 4.704 | 4.593 | 0.111 | 3.783 | -0.921 | passa |
|   fases · parada · bob.luau (51) · ~43 linhas | tecla 0.02 / 0.03 / 0.09 / 0.18 · quadro (+cena) 0.00 / 0.01 / 0.04 / 0.04 · paint 0.45 / 0.49 / 1.09 / 1.42 · placa 0.59 / 0.68 / 2.15 / 3.99 |
|   fases · parada · bob.luau (51) · ~43 linhas (réplica) | tecla 0.02 / 0.03 / 0.07 / 0.16 · quadro (+cena) 0.00 / 0.01 / 0.03 / 0.04 · paint 0.44 / 0.47 / 0.94 / 1.04 · placa 0.58 / 0.66 / 1.81 / 3.91 |
|   fases · parada · 7 500 · ~43 linhas | tecla 0.03 / 0.03 / 0.09 / 1.30 · quadro (+cena) 0.01 / 0.01 / 0.03 / 0.04 · paint 0.41 / 0.43 / 0.93 / 1.12 · placa 0.59 / 0.67 / 1.90 / 2.61 |
| a tocar | min | 1.100 | 1.143 | 0.043 | 1.089 | -0.054 | passa |
| a tocar | p50 | 1.228 | 1.201 | 0.028 | 1.199 | -0.029 | passa |
| a tocar | p99 | 2.648 | 2.484 | 0.164 | 2.990 | +0.342 | REPROVA |
| a tocar | p99.9 | 3.863 | 3.076 | 0.787 | 3.918 | +0.055 | passa |
|   fases · a tocar · bob.luau (51) · ~43 linhas | tecla 0.02 / 0.03 / 0.08 / 0.10 · quadro (+cena) 0.02 / 0.03 / 0.12 / 0.15 · paint 0.46 / 0.48 / 1.05 / 2.64 · placa 0.59 / 0.68 / 1.89 / 2.23 |
|   fases · a tocar · bob.luau (51) · ~43 linhas (réplica) | tecla 0.02 / 0.03 / 0.08 / 0.09 · quadro (+cena) 0.03 / 0.03 / 0.12 / 0.15 · paint 0.45 / 0.48 / 1.02 / 1.14 · placa 0.63 / 0.66 / 1.50 / 2.33 |
|   fases · a tocar · 7 500 · ~43 linhas | tecla 0.03 / 0.03 / 0.10 / 1.32 · quadro (+cena) 0.06 / 0.06 / 0.19 / 0.20 · paint 0.40 / 0.43 / 1.00 / 1.12 · placa 0.59 / 0.67 / 2.02 / 2.34 |

Ctrl+S → selo «Reloaded · 3 objects» (cena a tocar), ms; n = 1000
| documento | Ctrl+S → selo | dos quais a recarga (`scene_sync` + passo) |
|---|---|---|
| 51 linhas | 0.04 / 0.05 / 0.21 / 0.34 | 0.04 / 0.05 / 0.20 / 0.31 |
| 7500 linhas | 2.53 / 2.67 / 5.08 / 6.33 | 2.50 / 2.64 / 4.93 / 6.11 |

Gramática por tecla, à parte, ms; n = 1000
| documento | reparse incremental (no worker, fora da tecla) | estrutura mudada (`changed_ranges`, só com dobras fechadas) | dobras do ficheiro (`folds`, só o Fold All) | dobras de 100 linhas (`folds_in`, a pintura) | erros | realce de 100 linhas |
|---|---|---|---|---|---|---|
| 51 linhas | 0.02 / 0.02 / 0.08 / 0.12 | 0.01 / 0.01 / 0.03 / 0.04 | 0.05 / 0.05 / 0.10 / 0.14 | 0.05 / 0.05 / 0.10 / 0.13 | 0.00 / 0.00 / 0.00 / 0.00 | 0.21 / 0.22 / 0.40 / 0.46 |
| 7500 linhas | 0.38 / 0.43 / 1.95 / 2.08 | 0.34 / 0.36 / 1.19 / 1.29 | 4.84 / 5.48 / 11.67 / 13.54 | 0.07 / 0.07 / 0.15 / 0.16 | 0.00 / 0.00 / 0.00 / 0.00 | 0.36 / 0.38 / 0.72 / 0.74 |
```

## Corrida 3 (depois das curas: calha por busca, globais desconhecidos uma vez por quadro, cores ordenadas)

```
célula real (52 linhas): 1 diagnósticos do assistente · 5 dicas de tipo · lista de 4
célula real (7501 linhas): 293 diagnósticos do assistente · 297 dicas de tipo · lista de 4
E14 — perfil otimizado (smoke/release) · placa NVIDIA GeForce RTX 5060 Ti (Vulkan) · loadavg início 39.83 19.76 22.72 · fim 28.53 27.29 25.53 · rodada 108.5 s
alturas da área: ~43 linhas = 862 px · 100 linhas = 1945 px (largura 1600)

Tecla→pixel, ms (min / p50 / p99 / p99.9); fases em p50; n por célula = 1000
| cena · documento · vista | linhas pintadas | 1.º quadro | tecla→pixel | tecla (buffer+AutoEdit) | quadro (+cena) | paint | placa + leitura | a11y por tecla | paint sem mudança |
|---|---|---|---|---|---|---|---|---|---|
| parada · bob.luau (51) · ~43 linhas | 43 | 27.4 | 1.14 / 1.31 / 4.50 / 5.37 | 0.034 | 0.009 | 0.533 | 0.732 | 0.07 / 0.08 / 0.27 / 0.35 | 0.42 / 0.46 / 1.41 / 2.94 |
| a tocar · bob.luau (51) · ~43 linhas | 43 | 4.5 | 1.16 / 1.29 / 5.09 / 6.51 | 0.031 | 0.034 | 0.511 | 0.710 | 0.07 / 0.08 / 0.28 / 0.36 | 0.42 / 0.46 / 1.32 / 2.92 |
| parada · 7 500 · ~43 linhas | 43 | 3.0 | 1.09 / 1.25 / 5.06 / 9.70 | 0.035 | 0.008 | 0.475 | 0.719 | 0.07 / 0.09 / 0.30 / 0.38 | 0.38 / 0.42 / 1.28 / 3.27 |
| a tocar · 7 500 · ~43 linhas | 43 | 2.9 | 1.15 / 1.32 / 4.88 / 5.63 | 0.036 | 0.067 | 0.478 | 0.729 | 0.07 / 0.09 / 0.30 / 0.41 | 0.38 / 0.43 / 1.54 / 3.31 |
| parada · bob.luau (51) · 100 linhas | 52 | 7.2 | 1.28 / 1.44 / 5.52 / 7.41 | 0.031 | 0.008 | 0.593 | 0.796 | 0.08 / 0.09 / 0.33 / 0.39 | 0.49 / 0.53 / 1.64 / 3.29 |
| a tocar · bob.luau (51) · 100 linhas | 52 | 3.1 | 1.31 / 1.46 / 4.98 / 8.28 | 0.031 | 0.035 | 0.594 | 0.791 | 0.08 / 0.09 / 0.33 / 0.49 | 0.50 / 0.55 / 1.59 / 3.15 |
| parada · 7 500 · 100 linhas | 100 | 8.4 | 1.80 / 2.03 / 7.17 / 8.43 | 0.039 | 0.010 | 0.886 | 1.075 | 0.14 / 0.16 / 0.53 / 0.58 | 0.76 / 0.83 / 2.63 / 3.66 |
| a tocar · 7 500 · 100 linhas | 100 | 6.7 | 1.85 / 2.09 / 7.83 / 12.25 | 0.038 | 0.071 | 0.889 | 1.068 | 0.14 / 0.16 / 0.54 / 0.83 | 0.77 / 0.83 / 2.49 / 3.70 |
| parada · bob.luau (51) · ~43 linhas (réplica) | 43 | 2.8 | 1.12 / 1.24 / 4.87 / 5.23 | 0.029 | 0.007 | 0.500 | 0.705 | 0.07 / 0.08 / 0.28 / 0.40 | 0.41 / 0.46 / 1.37 / 3.30 |
| a tocar · bob.luau (51) · ~43 linhas (réplica) | 43 | 4.3 | 1.16 / 1.29 / 5.10 / 5.81 | 0.030 | 0.034 | 0.510 | 0.714 | 0.07 / 0.08 / 0.31 / 0.35 | 0.42 / 0.47 / 1.43 / 3.01 |
| parada · bob.luau (51) · ~43 linhas · lista aberta | 43 | 4.5 | 1.15 / 1.27 / 5.23 / 6.27 | 0.043 | 0.012 | 0.515 | 0.710 | 0.08 / 0.09 / 0.33 / 0.41 | 0.42 / 0.47 / 1.33 / 3.31 |
| parada · bob.luau (51) · ~43 linhas · luau-lsp: lista + assinatura + cores + dicas | 43 | 2.8 | 1.33 / 1.48 / 5.61 / 7.10 | 0.046 | 0.013 | 0.671 | 0.738 | 0.08 / 0.09 / 0.35 / 0.45 | 0.51 / 0.61 / 1.83 / 3.56 |
| parada · 7 500 · ~43 linhas · lista aberta | 43 | 2.9 | 1.13 / 1.24 / 5.13 / 7.41 | 0.043 | 0.011 | 0.471 | 0.701 | 0.08 / 0.09 / 0.33 / 0.42 | 0.39 / 0.43 / 1.34 / 1.45 |
| parada · 7 500 · ~43 linhas · luau-lsp: lista + assinatura + cores + dicas | 43 | 2.9 | 1.53 / 1.71 / 7.56 / 9.72 | 0.059 | 0.014 | 0.847 | 0.782 | 0.08 / 0.10 / 0.35 / 0.50 | 0.71 / 0.77 / 2.28 / 4.39 |

Regra de Fatin (vista ~43 linhas, tecla→pixel, ms): 7 500 − max(51 a, 51 b) ≤ |51 a − 51 b|
| cena | quantil | 51 (a) | 51 (b) | ruído | 7 500 | excesso | veredito |
|---|---|---|---|---|---|---|---|
| parada | min | 1.138 | 1.122 | 0.016 | 1.094 | -0.044 | passa |
| parada | p50 | 1.309 | 1.239 | 0.070 | 1.250 | -0.058 | passa |
| parada | p99 | 4.503 | 4.866 | 0.362 | 5.063 | +0.197 | passa |
| parada | p99.9 | 5.371 | 5.229 | 0.142 | 9.697 | +4.326 | REPROVA |
|   fases · parada · bob.luau (51) · ~43 linhas | tecla 0.02 / 0.03 / 0.24 / 0.42 · quadro (+cena) 0.01 / 0.01 / 0.06 / 0.07 · paint 0.46 / 0.53 / 1.60 / 1.92 · placa 0.64 / 0.73 / 3.02 / 4.35 |
|   fases · parada · bob.luau (51) · ~43 linhas (réplica) | tecla 0.02 / 0.03 / 0.24 / 1.99 · quadro (+cena) 0.01 / 0.01 / 0.05 / 0.07 · paint 0.45 / 0.50 / 1.51 / 1.88 · placa 0.63 / 0.70 / 3.34 / 4.02 |
|   fases · parada · 7 500 · ~43 linhas | tecla 0.03 / 0.04 / 1.30 / 1.56 · quadro (+cena) 0.01 / 0.01 / 0.05 / 0.08 · paint 0.41 / 0.47 / 1.38 / 4.01 · placa 0.64 / 0.72 / 3.55 / 5.59 |
| a tocar | min | 1.163 | 1.155 | 0.007 | 1.152 | -0.011 | passa |
| a tocar | p50 | 1.295 | 1.290 | 0.004 | 1.318 | +0.024 | REPROVA |
| a tocar | p99 | 5.086 | 5.099 | 0.013 | 4.876 | -0.223 | passa |
| a tocar | p99.9 | 6.509 | 5.814 | 0.695 | 5.633 | -0.876 | passa |
|   fases · a tocar · bob.luau (51) · ~43 linhas | tecla 0.03 / 0.03 / 0.23 / 0.31 · quadro (+cena) 0.03 / 0.03 / 0.20 / 0.23 · paint 0.47 / 0.51 / 1.53 / 2.19 · placa 0.63 / 0.71 / 3.24 / 4.79 |
|   fases · a tocar · bob.luau (51) · ~43 linhas (réplica) | tecla 0.02 / 0.03 / 0.25 / 0.32 · quadro (+cena) 0.02 / 0.03 / 0.21 / 0.24 · paint 0.46 / 0.51 / 1.75 / 2.08 · placa 0.63 / 0.71 / 3.61 / 4.13 |
|   fases · a tocar · 7 500 · ~43 linhas | tecla 0.03 / 0.04 / 1.35 / 1.54 · quadro (+cena) 0.06 / 0.07 / 0.27 / 0.33 · paint 0.42 / 0.48 / 1.46 / 1.74 · placa 0.64 / 0.73 / 3.45 / 4.52 |

Ctrl+S → selo «Reloaded · 3 objects» (cena a tocar), ms; n = 1000
| documento | Ctrl+S → selo | dos quais a recarga (`scene_sync` + passo) |
|---|---|---|
| 51 linhas | 0.04 / 0.05 / 0.27 / 0.44 | 0.04 / 0.05 / 0.25 / 0.42 |
| 7500 linhas | 2.50 / 2.76 / 9.47 / 16.38 | 2.48 / 2.71 / 9.13 / 14.67 |

Gramática por tecla, à parte, ms; n = 1000
| documento | reparse incremental (no worker, fora da tecla) | estrutura mudada (`changed_ranges`, só com dobras fechadas) | dobras do ficheiro (`folds`, só o Fold All) | dobras de 100 linhas (`folds_in`, a pintura) | erros | realce de 100 linhas |
|---|---|---|---|---|---|---|
| 51 linhas | 0.02 / 0.02 / 0.10 / 0.16 | 0.01 / 0.01 / 0.04 / 0.05 | 0.05 / 0.05 / 0.12 / 0.16 | 0.05 / 0.05 / 0.10 / 0.13 | 0.00 / 0.00 / 0.00 / 0.00 | 0.21 / 0.22 / 0.43 / 0.49 |
| 7500 linhas | 0.38 / 0.59 / 2.84 / 3.86 | 0.35 / 0.40 / 1.86 / 2.18 | 4.96 / 6.35 / 16.50 / 19.85 | 0.07 / 0.08 / 0.17 / 0.21 | 0.00 / 0.00 / 0.00 / 0.00 | 0.37 / 0.41 / 0.85 / 0.97 |
```
