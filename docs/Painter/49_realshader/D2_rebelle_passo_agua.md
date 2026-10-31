# D2 — Rebelle: ms por passo da água, 4K e 1K (oráculo, 31/10)

**Comando:** `R=3 bash ~/Apps/rebelle/oraculo/mede_passo_agua_v2.sh passo1b` (saída em `~/Apps/rebelle/oraculo/sonda/passo_agua/passo1b/`).
A v1 (`passo1`) saiu com código 4: o título escreve `64 x 64 pixels`, não `px`/cm. A v2 só troca a assinatura para `pixels`.
**Ecrã:** o ecrã real `:0`, desbloqueado (`GetActive=false`), sem uso pelo dono. **Máquina:** 32 threads.
**loadavg:** 0,33 no início. Subiu até 9,9 por causa do próprio Rebelle (~20 núcleos). Nenhum outro processo pesado (`ps`).
Processo ocioso: 0,104 → 0,126 núcleo.

**O SIMULATION É fila.** No 4K com K=200, o título mostra a tela do wash em +0,6 s. A CPU fica a 19–20 núcleos até a marca aparecer em +9,5 s e cai a 0,2–1,3 na amostra seguinte. O declive é limpo, ≠ 0.

| tela | K=50 mín | K=200 mín | **ms/passo** | por ronda | núcleos no passo | CPU ms·núcleo/passo | RSS máx |
|---|---|---|---|---|---|---|---|
| 4096×2560 | 3 950 ms | 9 538 ms | **37,3** | 37,3 · 36,0 · 49,5 | 21,3 (mediana 19,6–19,9) | 793 | 4 166 MB |
| 1000×640 | 801 ms | 1 388 ms | **3,9** | 4,1 · 3,9 · 4,3 | 12,4 | 48 | 1 710 MB |

**Contra o PH2D (4096×2560, cobertura 100 %):** 100,5 ms/passo depois dos cortes de 31/10 (antes 128).
⇒ O Rebelle é **2,7× mais rápido por passo** (26,8 contra 9,95 passos/s). Não temos o PH2D a 1000×640.

**Linearidade em pixels:** em trabalho de CPU, o custo é praticamente igual: 75,6 ns·núcleo/px em 4K e 75 em 1K.
O relógio cresce só 9,6× para 16,4× pixels, porque o 1K não enche os núcleos (12,4 contra ~20).
**Linearidade em K:** só há dois pontos (50 e 200), por isso a linearidade **não está provada**. O declive por ronda é estável no 1K (±0,2) e nas rondas 1–2 do 4K.
**B/px:** ΔRSS (4K − 1K) / Δpx ≈ (3 700–4 166 − 1 600–1 710 MB) / 9,83 Mpx ≈ **230–260 B/px**. É um RSS de processo inteiro, inclui o undo/telas do Wine e serve só de ordem de grandeza.

## Incertezas
- Um «passo» do `SIMULATION repeats` do Rebelle pode **não** equivaler a um passo do Wet Paint (substeps, difusão, secagem). A razão 2,7× compara iterações declaradas, não física igual.
- A ronda 3 do 4K (49,5 ms) correu com loadavg 5,6–9,9, causado pela carga do próprio Rebelle e talvez por térmica. Vale o mínimo (MEDIR_VELOCIDADE).
- Tudo corre via Wine (caixa-preta, sem perfilar). O custo fixo (wash, criar telas, título) é ~2,1 s no 4K e ~0,6 s no 1K, removido pelo declive.
- A coluna «núcleos no passo» vem de mín/mín (21,3). As amostras de 1 s dão 19,6–19,9.
