# W3 K3a — a geometria da faca (Óleo Real W3, 27/10)

Família «geometria da faca» num bloco (§0.10): planalto, lâmina fixa que estreita, crista na sombra, risca branca.
Oráculo: `fixtures/rebelle_bateria/OIL_AND_ACRYLIC/knife_knife_*_gestos_A`. Folha: [`W3_K3a_faca.jpg`](W3_K3a_faca.jpg)
(antes | depois | Rebelle, Óleo Real `PH2D_EXAME_OLEO=1`; «depois» = o motor novo + os valores propostos para a etapa P
abaixo — a fábrica NÃO foi tocada). Commit do motor: `4c6ca8b95e`.

## Réguas (scratchpad, refazem-se com a bancada)

- **Planalto**: no `recto_lento` (x 150..500) a mediana por linha da luminância através do traço; no miolo (60 %
  central) o desvio-padrão (sulcos) e o declive = |1.º terço − último terço| (tubo/cúpula).
- **Risca**: na faixa central (y 80..100) a luminância média de cada coluna; risca = o maior excesso de uma coluna
  sobre a mediana móvel de 41 colunas (lento x 80..540 · rápido x 640..940).
- **Rampa**: largura pintada na `rampa_0_1` (pressão 0 → 1 em x 60 → 560) em x 80..550.

## 1. A faca NIVELA (motor novo, knob `knifeLevel`)

**Causa medida.** Os sulcos e o tubo no Óleo Real não são a crista: com `bristleHeight` 0 e `knifeRidge` 0 o Smooth 2
continua com sulcos (std do miolo 24,5) — é a ESPESSURA (massa × factor) que varia através do traço (as faixas do
depósito a `pigmentPerDab` 2000); no Smooth 3 o resto é o relevo dos fios (`altura`) que o pente do rebordo só deixa
em equilíbrio com o que o dab acabou de pentear.

**Lei** (`lamina::nivelar`): por dab, a espessura de cada célula com tinta sob a lâmina vai, pela cobertura `w` e a
fracção `knifeLevel`, para a MÉDIA (pesada por `w`) da espessura sob a lâmina; a `altura` do miolo vai ao perfil do
rebordo pela mesma fracção. Só a altura muda: o factor pode ficar < 1 (`grid::espessura::nivela`), a massa e a cor
ficam ao bit (gate). `0` = ao bit (nem se chama).

| faca (Óleo, `knifeLevel`) | std miolo | declive | Rebelle std / declive |
|---|---|---|---|
| Smooth 2 · 0 → 0,3 → 1 | 24,5 → **10,1** → 12,4 | 26,1 → 9,1 → 14,7 | 0,1 / 0,1 |
| Smooth 3 · 0 → 1 | 4,7 → **1,8** | 5,2 → 2,6 | 0,8 / 0,1 |
| Textured · 0 → 1 | 5,5 → **3,8** | 6,2 → **0,4** | 2,9 / 4,4 |

**O que fica (não é da faca):** o Smooth 2 continua a ler-se como um TUBO de paredes macias: o render desfoca a massa
`raio_suave` 3 duas vezes (≈ 6 células por lado) num traço de 16 células — sobra um topo plano de ~4. O Rebelle tem
paredes de 1–2 px. Cura no `oleo_real.rs` (K4): as células niveladas pela faca fora do desfoque largo, ou um
`raio_suave` que não passe de ~1/8 da largura.

## 2. Lâmina fixa que estreita com a pressão — a COMPOSIÇÃO já o exprime (sem motor)

A rampa do Textured do Rebelle (largura ÷ a do recto) segue `p^1,5` sem piso: 0,075 · 0,25 · 0,40 · 0,60 · 0,80 · 0,93
a p = 0,24 · 0,40 · 0,56 · 0,72 · 0,88 · 0,98. O nosso tinha `pressureSize` 0,6 (piso 40 %): 0,45 · 0,61 · 0,73 · 0,85 ·
0,91 · 0,97. Com **`pressureSize` 1 + `pressureSizeCurve` 1,5** (knobs existentes): 0 · 0,21 · 0,42 · 0,61 · 0,85 · 0,97
— a ponta triangular (folha, linha Textured). A lâmina fixa a 86° estreita pelo raio como a seguidora.

## 3. A crista num campo PRÓPRIO (`Grid::crista`) — produtor feito, consumidor por ligar

O rebordo penteia agora a `altura` (a normal, como antes) E a `Grid::crista` (só a faca). Numa grade sem fios os dois
campos são iguais ao bit (gate) ⇒ ligar o consumidor dá exactamente a medição da W2 com `fios_na_sombra` 1 — **topo
0,74 / baixo 1,29** à luz de fábrica, **0,44 / 1,26** com Força do ambiente 0,15 (Rebelle **0,40 / 1,39**) — e o Oil
Bristle não muda (a `crista` dele fica vazia). Acessor: `crate::grid::crista::em(g, i) -> f32` (`0` sem o campo).

**A ligar em `oleo_real.rs` (`CampoAltura::de_com`)**: no laço por célula, ao lado de `fios[x] += altura·k`, um 3.º
acumulador `crista[x] += f64::from(crate::grid::crista::em(lg, i)) * k;` multiplicado por `cobre` como os fios; e na
`sombra`: `s + (crista[k] * k_fios) as f32` (sempre, sem a alavanca `fios_na_sombra`, que pode sair).

## 4. A risca branca vertical — causa: a ESCALA da textura (dado, etapa P)

**Causa medida.** Só o `paperGate` 0 a apaga; `dabRotation`, rebordo, nivelamento, pickup, travão e cápsula não. O
deslocamento da textura (`offset` 0,25) MOVE-a. O `Noise` `Tiled` a Size 6 tem a rede de 256/6 = **42,7 px** — a largura
do traço (40). Um nó baixo da rede cobre o traço inteiro na vertical ⇒ uma coluna de grão baixo de alto a baixo, que o
portão do papel abre (réplica numérica do campo: as colunas mais baixas a Size 6 são 309 e 836–843 — as riscas
medidas 302–310 e 835–853). Razão desvio das colunas ÷ desvio dos píxeis: 0,90 a Size 6 · 0,35 a 48 · 0,26 a 96.
O `Warp` 0 só muda o sítio da risca (refutado como causa).

| faca (clássico) | risca lento / rápido | Rebelle |
|---|---|---|
| Smooth 3 · Size 6 → **48** | 41,7 / 45,3 → 37,6 / **23,3** (a coluna 309/836 some; fica pontilhado) | 13,6 / 13,5 |
| Stain · Size 5 → **48** | 65,1 / 78,0 → **48,7 / 31,3** | 61,7 / 68,1 (ralo) |
| Dry · Size 6 → **48** | 1,9 / 5,7 → 9,2 / 5,4 (sem risca nos dois) | 15,7 / 13,5 |

## Valores propostos para a etapa P (NÃO estão na fábrica)

| faca | proposta |
|---|---|
| Smooth 2 | `knifeLevel` 0,3 |
| Smooth 3 (aprovada) | `knifeLevel` 1 · textura Size 48 |
| Textured | `knifeLevel` 1 · `pressureSize` 1 · `pressureSizeCurve` 1,5 |
| Stain · Dry (aprovada) · Smooth · Soft · Dry 2 | textura Noise Size 48 (a rede abaixo de 1/8 da largura) |

As 4 aprovadas (Smooth 3, Oily, Grainy, Dry) ficam ao bit no clássico (gate das miniaturas verde); no Óleo mudam só
quando o consumidor da crista for ligado (Smooth 3 8 px, Oily 3, Grainy 2 de rebordo).

## Gates (vistos VERMELHOS sob mutação)

`lamina::a_crista_e_o_rebordo_da_lamina_e_so_dele` · `lamina::a_faca_nivela_a_tinta_em_pe_num_planalto_sem_mudar_a_massa`
· `espessura::nivelar_desce_abaixo_da_massa_sem_mudar_a_massa` · `altura::a_crista_viaja_no_instantaneo_e_no_recorte`.
Mutações: crista não penteada · nivelamento × 0 · piso da massa no `nivela` · crista fora do instantâneo · fora do
recorte — 5/5 vermelhas. Verdes: `every_visible_knob_has_a_row` (87 → 88, Stroke 9 → 10), `every_word…string_table`,
`known_keys_round_trip_to_english`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`, os da `lamina` do tool-painter.

## Aberto

- `dab_route.rs` (K3b): a porta da lâmina só corre com `knifeRidge > 0` (`if rebordo > 0.0`); uma faca com
  `knifeLevel` e sem rebordo não nivela. Trocar por `if rebordo > 0.0 || nivel > 0.0` (com
  `nivel = …get(Knob::KnifeLevel)`).
- O tubo de paredes macias (§1) e o consumidor da crista (§3): `oleo_real.rs` (K4).
- O knob `subsurfaceRadius` passou o tecto de 64 a 16 (= `RAIO_SUBSUPERFICIE_MAX`, M1).
