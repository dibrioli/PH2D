# A água na placa — degrau 1: a advecção (31/10)

Crate `crates/ph2d-wet-gpu` (drop-crate). A CPU continua canónica (ADR-0146); a placa é reconciliada
por ε medido **por placa** (`tests/paridade.rs`, `#[ignore]`).

## Desenho

A lei do `solver/advect_jacobi.rs` (ADR-0147) em quatro compute WGSL `16×16`, f32, um compute pass cada:
`prepare` (uv bilinear da grade de fluxo, coordenada por divisão INTEIRA — exata para qualquer `rf`) →
`outflow_pass` (gather puro: cada célula varre os `(2·alcance+1)²` destinos, na ordem da CPU — sem `bt`,
sem atômicos) → `transport` (escala da FONTE, média linear da cor) → `commit` (cópia `dst→st` só no
retângulo). O retro-traço tira base+fração de `−uv` (pequeno), não de `x − uv`: a 4096 o ulp de f32
daria 5e-4 na fração. Estado: `st = vec2(film, susp)`, `rgb = 3×f32`, `act = u32` — 9 buffers, ≤ 8
storage por passada (layout automático). 4096×2560 ≈ 590 MB de placa.

## Paridade (320×200, poça com redemoinho, |uv| ≤ 0,9)

| placa | passos | campo | desvio máx | desvio médio | Σ rel. ao inicial |
|---|---|---|---|---|---|
| RTX 5060 Ti | 1 | film · susp · rgb | 2,4e-7 · 6,0e-8 · 2,4e-7 | 1,5e-8 · 3,8e-9 · 2,3e-8 | film 1,2e-9 · susp 1,8e-9 |
| RTX 5060 Ti | 8 | film · susp · rgb | 9,5e-7 · 2,1e-7 · 4,2e-7 | 4,4e-8 · 1,1e-8 · 4,0e-8 | film 1,3e-8 · susp 1,5e-8 |
| Radeon RADV | 1 | film · susp · rgb | 2,4e-7 · 6,0e-8 · 2,4e-7 | 1,4e-8 · 3,8e-9 · 2,3e-8 | film 1,3e-9 · susp 1,8e-9 |
| Radeon RADV | 8 | — | (idem RTX ±) | | film 1,4e-8 · susp 1,5e-8 |

`rf = 1` e `rf = 4` dão o mesmo. Espelho em x a 8 passos: máx 9,5e-7 (film), 1,8e-7 (susp), nas duas.
Gate: `EPS_1PASSO = 1e-6` (4× o medido), `EPS_MASSA = 1e-7` rel. (55× o medido a 1 passo), espelho
`8e-6`. A CPU f64 conserva a 1e-10; a placa a 1e-9 por passo (soma em f32).

## ms (exemplo `mede_advect`, 7 rondas × 10 passos, mínimo, relógio = timestamps da placa)

| medida | RTX 5060 Ti | Radeon RADV (iGPU) |
|---|---|---|
| prepare · outflow · transport · commit, 4096×2560 | 0,43 · 1,52 · 1,45 · 1,09 | 5,6 · 56,6 · 21,5 · 11,1 |
| **passo inteiro 4096×2560** | **4,50** | 95,2 |
| passo inteiro, região suja 1024² | 0,42 | 9,5 |
| upload folha st+rgb (210 MB), parede | 41 | 33 |
| readback folha st+rgb, parede | 213 | 121 |
| upload · readback região 1024² (por linha), parede | 7,8 · 3,6 | 9,1 · 5,6 |

CPU de referência: advecção ~20–38 ms do passo (calma). A corrida desta sonda deu 338/149 ms com
`loadavg` 72–82 — **não vale** (§5.0); as colunas de placa são relógio da placa e valem. ⇒ na RTX a
advecção é **~5–8× mais rápida que a CPU calma** — desde que o estado **more na placa**: um upload +
readback por passo (250 ms) mata o ganho; mesmo a região suja (11 ms) custa o dobro da advecção.

## O que falta para o passo inteiro na placa

1. Os outros estágios do `sim_step` na placa (flow/`project`, `momentum_rows`, sedimenta, secagem,
   `rebuild_active_region`) — o estado só desce quando o render pedir.
2. A mistura K–M (`km_mixing = true`) no `transport` (hoje só a média linear, o default).
3. Faixas vivas por linha (hoje um retângulo); a invariante «massa zero fora da faixa» é o que o
   permite — com retângulo grosso o custo é proporcional à caixa.
4. Na Radeon o `outflow_pass` é 60 % do passo: o gather `(2·alcance+1)²` relê `uv` 25×; candidato a
   memória partilhada do workgroup (tile + halo de `alcance`).
5. Readback por linha a meio-traço: trocar `write_buffer` por linha por um staging contíguo.
