# 43 — O pó fino do aerógrafo: o que se MEDIU antes de construir (17/10)

> Prompt [43](43_PROMPT_o_po_fino_aerografo.md). Barra: a do [18 §2](18_mapa_das_midias.md) (as métricas do
> recto + a largura ± 2 px), contra `fixtures/rebelle/airbrush.png` (Spray 2, 40 px). Sonda:
> `sonda_midias::as_midias_com_os_knobs_de_hoje` com receitas em JSON (`PH2D_SONDA_RECEITAS`), 19 lotes, ~1 300 cenas.

## §1 — Onde o grão vive (perguntado ao CÓDIGO)

- **A célula NÃO é o limite**: a razão da grelha é 1 por omissão (`grid_map::DEFAULT_RATIO`), a célula é o píxel; o
  render copia ao byte (`SampleU` identidade). O 18 §3 dizia «o grão do motor não desce abaixo da célula» — errado.
- **O que fazia as manchas**: o candidato do 18 usava o grão Noise de lado 0,02/0,05, abaixo do corte
  `TEX_SIZE_MIN = 0,1` (`ph2d-painter-brush/src/texture.rs`) — os dois liam 0,1, uma célula de lattice de 2 560 px.
  Por isso «0,02 = 0,05 dá o mesmo». (Mais um caso do 18 §4.5: o eixo não foi varrido.)
- **O que já existia e não chega** (lotes 1–2, 94 receitas): o Grain de lado 100 PRESO À TELA (`Tiled`) dá a
  textura (24,8 %), mas é o mesmo desenho em todo traço; o grão que salta por dab (`Random`) faz MÉDIA entre os
  ~10 dabs que cobrem cada píxel (2–11 %). O melhor da composição: 5/8 (orla 15, CV 0,15, período 0,26).
- (b) o grão abaixo da célula no render e (c) a razão 1 só para o pincel de pó: inúteis com a razão 1 (já é).

## §2 — O oráculo corrido (fixtures `airbrush20`, `airbrush40b`, `airbrush80`)

| foto | cob | tex | borda | orla | cv | per | cruz | larg | grão (aut. 0,5 / 0,2) |
|---|---|---|---|---|---|---|---|---|---|
| 40 (07/10) | 0,381 | 22,7 | 0,537 | 12 | 0,088 | 0,046 | 0,583 | 26 | 1,28 / 1,82 |
| 40 repetição | 0,372 | 23,3 | 0,544 | 13 | 0,086 | 0,041 | 0,611 | 26 | 1,23 / 1,75 |
| 20 | 0,390 | 22,5 | 0,600 | 7 | 0,132 | 0,043 | 0,574 | 13 | 1,29 / 1,85 |
| 80 | 0,368 | 23,4 | 0,513 | 23 | 0,072 | 0,050 | 0,534 | 54 | 1,29 / 1,84 |

- **O grão do pó é FIXO em píxeis (~1,3 px)**; a textura e a cobertura não mudam com o tamanho; a largura e a
  orla escalam (0,66 e 0,3 × o tamanho).
- **O «período» não conta nesta mídia**: a repetição varia 0,005 e o controlo fica a 0,013 da foto (< 3× o ruído).

## §3 — A lei e as variantes medidas (o MESMO processo, `PH2D_SONDA_LEI`)

- **L1 (entrou)**: a gota cai com `p = 1 − 0,95·dust` (fixa no dab) e leva a dose `÷ p` — a média do carimbo é a de
  sempre (antes do portão do papel). **L3** (`p` seguindo a queda, `p·stamp^dust`): reprova a borda em todo o lote 5.
- **O lado da gota** (lote 14–15): 1 célula → grão 0,79 px, textura 27,5; 1,5 → 1,17; **2 → 1,40** (textura 25,2);
  2,5 → 1,59; 3 → 1,89. Nenhuma de 1,75 entre as 8 mais perto do Rebelle. `GOTA_DO_PO = 2`, fase por dab.
- **A semente**: um passo do fluxo do pincel (`tex_rng`, splitmix64) por dab, só com o pó ligado (nenhum outro pincel
  vê o fluxo mudar), misturado ao centro do dab no motor.
- ⚠️ **A armadilha das cerdas** (lotes 3–16): uma receita com `felt` no mapa do motor recria as CERDAS por cima do
  Shape «None» (`wet_preset_aplicar`, a migração de antes da W2) — todas as medições até ao lote 16 tinham cerdas
  de poros 0 por baixo. O pó SOZINHO (lote 17–18) passa melhor: distância ao Rebelle 0,74 contra 1,08.

## §4 — O Airbrush de fábrica

Seco, sem cerdas, dureza 0,3 (Smooth), pó 0,8, pigmento 9; o portão pinta-o no raio 15 da lei dele (a largura 26).

| | cob | tex | borda | orla | cv | cruz | larg |
|---|---|---|---|---|---|---|---|
| Rebelle | 0,381 | 22,7 | 0,537 | 12 | 0,088 | 0,583 | 26 |
| Airbrush | 0,359 | 21,9 | 0,596 | 12 | 0,068 | 0,523 | 26 |
| controlo (Wet Round, seco) | 0,206 | 8,3 | 0,993 | 1 | 0,028 | 0,964 | 23 |
| o mesmo com o pó a 0 | 0,346 | 0,9 | 0,466 | 9 | 0,000 | 0,540 | 28 |

A textura do Airbrush a 40 e 80 px: 21,9 · 22,0 (Rebelle 22,7 · 23,4). **Aberto:** a 20 px lê 28,7 (Rebelle 22,5).

## §5 — Portões e mutação

`ph2d-wet-paint` `it/a_lei_do_po.rs` (5): pó 0 = o carimbo ao bit · a massa conserva-se (24 sementes, ± 3 %) · as
gotas são de 2 células e caem na fracção 0,24 (correlação ½ a 1 célula, 0 a 2) · a semente escolhe as gotas · o
centro do dab muda-as. `ph2d-tool-painter` `it/o_po_do_aerografo.rs` (3): o segundo traço cai noutras gotas
(correlação < 0,3) · o pincel gravado leva o pó · o pó é do píxel (40 e 80 px). `midias_de_fabrica`: o Airbrush com
7 métricas (a régua portada ganhou `cv` e `largura`). A identidade: as 17 de antes, pinadas em `ddc782e9f0`.

## ⛔ Recusas MEDIDAS

| proposta | medido | porquê não |
|---|---|---|
| grão Noise/Grain `Random` (a composição de hoje) | tex 2–11 % | os dabs sobrepostos fazem média |
| grão `Tiled` lado 100 | tex 24,8, borda 0,78 | o mesmo desenho em todo traço; a borda do controlo |
| L3 (densidade pela queda) | borda 0,71–0,80 | o miolo escurece, a orla clareia |
| gota de 1 célula | grão 0,79 px, tex 27,5 | o grão do Rebelle é 1,28 |
| gota de 3 células | grão 1,89 px | grosso demais |
