# O custo no caminho do app — o Filbert a 12 FPS (22/10)

**Sintoma (dono):** com o Filbert de fábrica o app cai para ~12 FPS ao pintar. A bancada (`exame.sh --custo`) lia-o a
p95 9,74 ms. **O enigma:** a bancada repetia os pontos da bateria — ~3 px por evento, **9 px por quadro**, um traço
LENTO — e o custo de um pincel de cápsula é **por px de caminho**. O app anda 30–135 px por quadro.

Sonda: `tests/it/sonda_custo_no_app.rs` (o `PainterTool` com o preset de fábrica, 3 eventos por quadro, o tique de
60 Hz e a drenagem da prévia, ao ritmo da parede; variantes no MESMO processo, intercaladas, o MÍNIMO das medianas).
Perfil `smoke`, tela 1024², `RAYON_NUM_THREADS=16` dentro da fatia `ph2d-run` (CPU ≤ 50 %), load 1,2–3,9.

## 1. Onde aterra ≠ o que provoca

| fase (Filbert 64 px, 90 px/quadro) | ms/quadro |
|---|---|
| depósito dos dabs (`on_canvas_pointer`) | **56,05** |
| composite | 0,20 |
| tique da água (no fio dela) | 0,00 |
| drenagem da prévia | 0,00 |

Tamanho da tela: 1024² = 2048² (59,2 × 54,2 ms) — **não é a tela**. Ablação no mesmo processo (chave do motor
trocada): `continuous_spacing=false` 61 → 9,5 ms; `capsuleRoundNearest=1` → 30; `capsuleSpacing=0,05` → 20;
`bristleTipRagged=0` → 45; `bristleCount`, `bristleLaneWidth`, `capsuleMaxPx`, `pigmentPerDab`, `smear`, `pickup`,
`leveling`, `dab_angle`: nada. O custo é **sub-dabs × custo de cada um**: o sub-passo da cápsula do Filbert é
`floor(0,025 × 64 = 1,6) = 1 px` (a lei do doc 51 §3, sem batimento) — 90 sub-dabs por quadro.

Por sub-dab (relógios temporários): depósito **512 µs**, transfer 118, relevo 44, resto 10. O Oil Bristle a 103 px
(caixa 2,6× maior) pagava **136 µs** de depósito. **A causa:** `par::MIN_CELLS_DEPOSIT` (6 144 células) foi medido
com a queda redonda do motor, `~7 ns` por célula. A caixa do Filbert (4 225 células) fica abaixo ⇒ **em série**; mas a
silhueta dele (as cerdas, a ponta desfiada que lê 4 células por célula, o relevo e a cor de cada fio) custa `~120 ns`
por célula. O Oil Bristle (10 800 células) já corria em paralelo. O piso em CÉLULAS mente quando o custo por célula é
do pincel.

## 2. A cura — o piso abaixo das células é de TEMPO

`ph2d_wet_paint::par::walk_items_reduce_medido` + `PISO_NS_DEPOSITO = 35 µs` (`trail/deposit.rs`): acima de
`MIN_CELLS_DEPOSIT`, paralelo como sempre; abaixo, a linha do MEIO corre em série, o relógio dela × as linhas estima
a caixa, e a estimativa ≥ 35 µs vai em paralelo. O número sai da tabela do próprio `MIN_CELLS_DEPOSIT` (o pool custa
~25 µs: prejuízo a 15 µs de série, ganho a 48; a linha do meio sobrestima um disco ~1,3×). **A rota só escolhe o
caminhante** (ADR-0175: as duas rotas dão os mesmos bytes) — a imagem não muda um bit: `deposit_rows` (7),
`fingerprint`, `a_lei_da_tinta` (a cena de cada preset de fábrica), `familias::oleo_acrilica`,
`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`, `cada_preset_parece_a_sua_midia` verdes (14 + 7).

| Filbert 64 px (fábrica) | antes | depois |
|---|---|---|
| 9 px/qd (a bateria) | 4,71 | **2,21** |
| 30 px/qd (traço vivo) | 19,69 (52/90 > 16,7) | **7,90** (0/90) |
| 90 px/qd (rápido) | 56,31 | **24,93** |
| 135 px/qd (≈ o 12 FPS do dono) | 96,42 | **38,88** |
| depósito por sub-dab | 512 µs | 83 µs |

⛔ Recusado (medido): o mesmo piso de tempo no BICO do transfer (39 → 22 µs por sub-dab, mas 25,20 → 24,74 ms por
quadro — dentro do ruído); a memória da silhueta crua da ponta (poupa ~11 µs por sub-dab depois de pagar a
iniciação da caixa). Mudar o sub-passo (`capsuleRoundNearest`, `capsuleSpacing`) corta para metade mas muda a imagem.

## 3. Os 45 de fábrica (uma corrida, `todos`, fábrica e 200 px × 30 e 90 px/qd × antes/depois, mediana)

**A mesma família (abaixo do piso de células, silhueta pesada) — curados no mesmo bloco:**

| preset | px | 90 px/qd antes → depois | 30 px/qd antes → depois |
|---|---|---|---|
| Filbert | 64 | 55,9 → **24,4** | 19,8 → **8,5** |
| Express Oil | 64 | 31,4 → **12,7** | 10,4 → **4,8** |
| Dabber | 51 | 16,0 → **8,0** | — |
| Square Hard | 35 | 29,1 → **10,0** | 10,4 → **3,7** |
| Charcoal | 37 | 21,0 → **7,2** | — |
| Pastel Side | 20 | 23,1 → **9,5** | 8,2 → **3,5** |
| Soft Pastel | 29 | 15,6 → **8,5** | — |
| Dip Pen | 91 | 25,2 → **15,4** | — |
| Ink Bullet / Marker Bullet | 93/101 | 25,0 → 17,3 / 25,2 → 20,9 | — |

Nenhum outro mudou além do ruído (0,94–1,07×). **Ainda acima de 16,7 ms a 90 px/qd** (a 30 px/qd todos cabem na
mediana): Filbert 64 (24,4) e 200 (26,5), Oil Bristle 200 (38,3), Ink Brush 200 (33,5), Dabber 200 (23,3), Marker
Bullet 101 (20,9), Square Hard 200 (18,6), Soft Pastel 200 (18,5), Scratch 248 (17,9), Ink Bullet 93 (17,3). Já em
paralelo — **outra causa**: o pouso do transfer (`trail/transfer.rs`, ~106 µs por sub-dab no Filbert, a cada
sub-dab, 4 400 células) é SÉRIE POR SEMÂNTICA (o arrasto lê `susp[si]` de uma célula que o mesmo laço pode ter
escrito — Gauss-Seidel). A cura exacta é paralelizar por LINHAS ao longo do deslocamento da âncora (cada linha é
independente, em série dentro dela), o que pede refazer o corpo do pouso sobre uma vista de células sem `unsafe`
(proibido na workspace). O Filbert cabe em 16,7 ms até ~60 px/qd (antes ~27).

## 4. A régua consertada

- `exame.sh --custo <pincel> [0,40,200] [bateria,30]`: tamanho `0` = o de fábrica; a velocidade é px por evento
  (`PH2D_EXAME_PX_POR_EVENTO`, o traço re-amostrado ao longo da polilinha) ou `bateria`. A coluna `px/qd` na tabela.
  Filbert, antes → depois: fábrica a 90 px/qd p95 **64,23 → 27,57** ms (a bateria: 10,10 → 4,61).
- Portão `o_custo_no_caminho_do_app::o_deposito_de_um_pincel_pesado_de_fabrica_vai_pela_rota_que_paga` (Filbert e
  Express Oil a 90 px/qd, três rotas intercaladas: produto < 1,6 × paralela e de antes > 1,5 × produto). Verde:
  Filbert 23,54 · 23,51 · 51,99; Express Oil 12,96 · 11,96 · 25,01. Red-first (o piso do produto a `u64::MAX` = a
  rota de antes): 52,87 contra 23,53 (2,25×). Mutações (4 com controlo): M1 piso `u64::MAX` SANGROU (52,19 · 23,17);
  M2 depósito sempre em série SANGROU (47,95 · 47,94 · 49,66); M3 o roteador nunca paralelo SANGROU. A 2.ª metade
  pode aproximar-se do limiar com a máquina cheia (família `FLAKES_DE_CARGA`).

## 5. Confirmação no app

O binário `smoke` compila; não há modo de smoke que pinte sozinho e meça ms/quadro — a confirmação no app fica com o
dono (`PH2D_PAINT_PERF=1` imprime o período e o input por janela).
