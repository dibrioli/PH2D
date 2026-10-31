# Medida 31/10, rodada 3 — ms/passo da água no CPU a 4K, HEAD `fc56456c43`, contra o Rebelle

Cena: o wash de `nucleos_wash` ([B_nucleos.md](B_nucleos.md)), 4096×2560, água 100 % e pigmento 85 %. A `Engine` é
montada como no produto. Perfil `smoke` (`PH2D_RELEASE="mede velocidade"`). Tudo corre na fatia da linha
(`scripts/ph2d-run.sh`: **CPU ≤ 1600 % de 32**, porque o dono pode estar a usar o PC). Alvo: o Rebelle a **37,3 ms/passo**
([D2](D2_rebelle_passo_agua.md), ~20 núcleos, sem tecto).

**Variantes no MESMO processo:** um `rayon::ThreadPool` por nº de fios (2 · 8 · 16 · 32), com `pool.install`, × a rota do
*commit* da advecção (`flow::COMMIT_ROTA`: **0** = pela faixa, o produto/HEAD · **1** = sempre cópia, a rota antiga · **2** =
sempre troca de planos). Dá 12 variantes. Cada bloco tem 6 passos, um ciclo de secagem (`dry_every = 6`). A ordem roda 5
posições por processo.

Usei um exemplo temporário, `mede_rodada3_tmp.rs`: é o `nucleos_wash` com o laço acima no lugar do laço de água. Foi
apagado depois da medida e não foi comitado.

## ⚠️ A 1.ª tentativa (6 rondas num só wash) não vale como régua

Um único wash com 6 rondas seguidas leva a água até ao quadro 438. Depois do quadro ~300 o passo **barateia** à medida
que a região activa encolhe: 2 fios caem de 530 para 17 ms, e 16 fios de 89 para 34 ms. O «mínimo» saía do estado da
tela, não da variante. ⇒ A régua passou a ser **um wash NOVO por ronda**: 6 processos, cada um com UMA ronda das 12
variantes nos quadros 6–78. É o regime do wash cheio, comparável ao K = 50..200 do Rebelle.

## Resultado (6 processos × 12 variantes, ordem rodada)

```text
bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo build --profile smoke -p ph2d-wet-paint --example mede_rodada3_tmp
bash scripts/ph2d-run.sh run6.sh   # for off in 0..5: OFF=$off RONDAS=1 K=6 target/smoke/examples/mede_rodada3_tmp
```

loadavg (1 min) no início de cada processo: 3,90 · 3,58 · 2,85 · 3,66 · 5,20 · 4,24. No fim: 4,26. É calmo para esta
máquina; a rodada 2 correu a 45–105.

| fios | rota | ms/passo nas 6 rondas | **MÍNIMO** | vs Rebelle 37,3 |
|---|---|---|---|---|
| 2 | 0 (HEAD) | 526,8 · 521,0 · 529,6 · 523,3 · 531,7 · 529,1 | **521,0** | 14,0× |
| 2 | 1 (cópia) | 521,0 · 542,2 · 526,7 · 546,0 · 527,3 · 536,5 | 521,0 | 14,0× |
| 2 | 2 (troca) | 528,5 · 520,9 · 529,2 · 526,2 · 531,7 · 511,2 | 511,2 | 13,7× |
| 8 | 0 (HEAD) | 142,2 · 145,2 · 140,7 · 146,6 · 142,8 · 144,1 | **140,7** | 3,8× |
| 8 | 1 | 154,3 · 155,1 · 154,7 · 149,9 · 155,1 · 150,3 | 149,9 | 4,0× |
| 8 | 2 | 141,2 · 149,2 · 141,5 · 144,3 · 142,4 · 144,4 | 141,2 | 3,8× |
| 16 | 0 (HEAD) | 86,4 · 85,4 · 86,6 · 85,1 · 87,8 · 85,0 | **85,0** | **2,28×** |
| 16 | 1 | 94,8 · 95,8 · 94,7 · 97,0 · 95,7 · 96,4 | 94,7 | 2,54× |
| 16 | 2 | 86,7 · 84,5 · 87,7 · 84,4 · 89,2 · 84,6 | 84,4 | 2,26× |
| 32* | 0 (HEAD) | 131,1 · 132,3 · 127,4 · 128,1 · 127,6 · 129,8 | **127,4** | 3,4× |
| 32* | 1 | 152,6 · 150,2 · 151,6 · 151,4 · 153,0 · 151,1 | 150,2 | 4,0× |
| 32* | 2 | 132,6 · 134,6 · 133,0 · 142,3 · 132,8 · 134,9 | 132,6 | 3,6× |

\* 32 fios **dentro do tecto de 16 núcleos** (cgroup). O escalonador faz *throttle* e o passo fica 50 % mais caro do que com 16
fios. Isto é artefacto da fatia, não é o produto sem tecto. O «todos sem tecto» não foi medido nesta rodada; a D2 leu
100,5 ms com 32 fios livres, num HEAD anterior e sob outra carga.

**Leituras:**
- A dispersão por variante é ≤ 3 % (16 fios: 85,0–87,8). A régua decide ao 5 %.
- **Rota 1 (cópia) / rota 0 (HEAD), pelo mínimo:** 1,00 com 2 fios · 1,07 com 8 · 1,11 com 16 · 1,18 com 32*. O corte do
  commit vale 7–11 % do passo quando há núcleos.
- **Rota 2 (troca) ≈ rota 0 dentro do ruído** (84,4 contra 85,0 com 16 fios). A troca de planos sempre não ganha nada
  mensurável sobre a escolha por faixa.
- Escala: 2 → 8 fios dá 3,7×; 8 → 16 dá 1,66×. Com 16 núcleos o passo custa ≤ 85 × 16 ≈ **1 360 ms·núcleo**, contra **793**
  no Rebelle ⇒ além do relógio, gastamos ~1,7× mais CPU por passo.

## Fases do passo (perf, HEAD, 16 fios, 8 s de água)

`RAYON_NUM_THREADS=16 perf record -F 999 -g -D 12000 target/smoke/examples/nucleos_wash 8`, num binário com
`CARGO_PROFILE_SMOKE_STRIP=none CARGO_PROFILE_SMOKE_DEBUG=line-tables-only`. Resultado: 90 passos a **89,3 ms/passo**,
96 k amostras, loadavg 4,0 → 4,6. ms = partilha × 89,3; a secagem é a média de 1 passo em 6.

| fase | % | ms/passo |
|---|---|---|
| **secagem** (`assenta` + `assenta{closure#5}` + `Parada::d` + `prepara`/`cor_e_alfa`/`na_caixa` + `colorops::Perfil::de` + `drying::dry_cell` + `pow`) | **48,1** | **43,0** |
| **advecção** (`transport_rows` 15,6 · `outflow_rows` 5,9 · `prepare_rows` 4,5) | **26,0** | **23,2** |
| fluxo (`flow_jacobi::flow_rows` 4,4 · `flow_at_point` 3,5) | 8,7 | 7,8 |
| momentum (`momentum_rows`) | 3,9 | 3,5 |
| projecção | 3,5 | 3,1 |
| suavizar (`smooth_velocity_rows`) | 3,4 | 3,1 |
| outros (`parede_do_traco` 1,2 · `fill` 1,0 · região activa 1,7 · f32→f64) | 5,5 | 4,9 |
| espera (rayon/futex) | 0,7 | 0,6 |

## O que falta para bater 37,3 ms (com 16 núcleos)

Faltam **−48 ms** por passo (85 → 37). **Só tirar a secagem inteira não chega**: ficaria em ~42 ms. Uma combinação que
chega: dividir a **secagem por ~4** (43 → ~11; `assenta`/`Parada::d`/`prepara` ainda em f64, célula a célula, com `pow` e
`Perfil::de` por célula), **E** dividir a **advecção por ~2** (23 → ~12; `transport_rows` sozinho = 13,9 ms). Isso dá
85 − 32 − 11 ≈ **42 ms**. O resto viria de fluxo + momentum + projecção + suavizar (17,5 ms juntos). O Rebelle usa ~20
núcleos: com o tecto da fatia levantado ganhamos no máximo ~1,25× por escala, e isso ainda não basta sozinho.
