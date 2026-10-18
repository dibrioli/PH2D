# 49 — O relevo de cada fio (W3): o que se MEDIU (18/10)

> Prompt [49](49_PROMPT_W3_o_relevo_de_cada_fio.md). Sonda: `tests/it/o_relevo_dos_fios.rs::sonda_a_barra_da_w3`
> (a régua da imagem + o CAMPO do motor, `PH2D_SONDA_KNOBS="nome:k=v,-k|…"`, `PH2D_SONDA_RAIOS`) e
> `sonda_a_largura_molhada_e_seca`; a do leve/forte é a da W2 (`os_fios::sonda_a_barra_da_w2`). Cena seca da
> bancada (03 §1). Cinco rodadas, todas no mesmo binário, a escolha de cada uma pela anterior.

## §1 — A barra da W3 HOJE (antes, `430e586e0d^^`) — três óleos, à largura da foto e a 40 px

A régua ganhou a textura **ATRAVÉS × AO LONGO** (`Medida::atraves/ao_longo`: o resíduo a 1 eixo, `c` menos a média
de 5 px na vertical / na horizontal). Nas fotos: óleo **5,84 · 1,09**, express 8,85 · 1,80, oilround 3,09 · 1,31.
(Os 5,48 · 0,98 do 48 §2 não se reproduzem com esta nem com a variante «resíduo 5×5 suavizado a 1 eixo» — 5,78 ·
2,17; a conclusão é a mesma: a textura do óleo é quase toda através.)

| pincel · raio | textura | através · ao longo | borda/miolo | carris | CV | Rebelle (foto) |
|---|---|---|---|---|---|---|
| Oil Bristle · 20,5 | 3,85 | 3,83 · 0,76 | **0,897** | 3,03 | 0,017 | oil 5,86 · 5,84/1,09 · 0,872 · 3,32 · 0,011 |
| Oil Bristle · 20 | 3,80 | 3,78 · 0,80 | 0,829 | 3,06 | 0,017 | |
| Acrylic Round · 19,5 | 30,36 | 30,29 · 6,82 | 0,657 | 6,17 | 0,024 | oilround 3,13 · 3,09/1,31 · 0,595 |
| Express Oil · 21 | 7,82 | 7,82 · 0,41 | 0,470 | 0,86 | 0,006 | express 9,00 · 8,85/1,80 · 0,80 |

- ⚠️ **O «hoje 0,52» da barra (05 §3) era o Wet Round de 08/10**: o Oil Bristle já está em 0,83–0,90 (o portão das
  mídias guarda-o em 0,872). A borda NÃO é o que a W3 tem de mover.
- **Onde o relevo nasce hoje** (`render.rs`): o emboss é o GRADIENTE DA MASSA × `Grid::relevo` (o Emboss do pincel
  que pousou a tinta). O campo, no miolo do recto: o Oil Bristle tem massa 3 621 que varia **18,5 % através com os
  fios e 18,9 % sem eles** — a variação é do TAPETE (as cerdas arrastadas); os fios da W2 só lhe juntam variação ao
  longo (1,4 → 3,1 %). ⇒ **o relevo de hoje não seguia os fios**. Emboss 2 sem mais nada: textura 6,97, mas carris
  4,49 (o tapete) — ⛔.

## §2 — A oleosidade como viscosidade: nada a mover (recusada, medida)

A barra dela é «acrílico: largura A = B» (molhada a 2 s = seca a 30 s). Medida nos três óleos:

| variante | Oil Bristle | Acrylic Round | Express Oil |
|---|---|---|---|
| fábrica · viscosidade 0 · viscosidade 2 | 40 = 40 | 29 = 29 | 40 = 40 |
| água 1 · água 1 + viscosidade 2 | 40 = 40 | 29 = 29 | 40 = 40 |

A largura não muda com o tempo em NENHUMA variante (a água não alarga o traço — 05 §1.1, como o Rebelle) e o motor
já tem o knob `viscosity` (0–2). ⇒ um knob «oleosidade» não teria leitor que decida: ⛔ não construído. (Fora da
onda: `water 1 + drag 1` apaga o traço do Express — largura 0 — e estreita o Oil para 27.)

## §3 — A lei (`ph2d-wet-paint/src/bristles_relevo.rs`) e a varredura

Cada fio da W2 deixa na tinta a altura `bristleHeight × ganho × contacto²`, **para cima se constrói, para baixo se
escava** (o tipo é do fio, por hash: `bristleRemove` = a fracção que escava). Cada dab PENTEIA a célula para o alvo
do fio (`h += 0,5 · cobertura · (alvo − h)`): fica o que os últimos pelos deixaram. O campo é `Grid::altura`
(preguiçoso: vazio até o 1.º pincel com relevo — 67 MB por camada a 4096² para quem não o usa), aceso pela luz do
Impasto (o rig do artista) na linha de pigmento, por uma porta livre (`impasto_light_porta.rs`).

| rodada | achado (Oil Bristle à largura da foto) |
|---|---|
| altura 0,5 → 1 → 2 → 4 (emboss 0,7) | textura 4,59 → 5,22 → 5,38 → 5,47: **a luz satura** (2 px em fios de 1,5 px ≈ 45°) |
| escava 0,65 · 0,35 · 0,2 (altura 1–2) | 4,82 · 5,31 · 5,43 contra 5,22–5,38 a 0,5: fica o padrão 0,5 |
| emboss 0,7 → 1 sozinho | 4,63 (carris 3,25) — o gradiente da massa sozinho não chega |
| **altura 1,25 + emboss 1** | **5,87** (5,86), carris 3,26, CV 0,017, forte 5,29 (5,04), borda 0,895 |
| contacto¹ → **contacto²** | leve 6,61 → 6,32; forte igual (a película fina E o sulco raso) |
| Express Oil altura 1 | 7,94 → 8,79 (9,00) — dentro do ruído da foto (recto × rápido 9,0 · 9,9) ⛔ |

## §4 — A fábrica

| à largura da foto | textura | através · ao longo | carris | CV | período | borda | leve · forte |
|---|---|---|---|---|---|---|---|
| Oil Bristle antes | 3,85 | 3,83 · 0,76 | 3,03 | 0,017 | 2 % | 0,897 | 5,36 · 3,31 |
| **Oil Bristle depois** (`bristleHeight` 1,25, emboss 1) | **5,87** | 5,85 · 0,90 | 3,26 | 0,017 | 2 % | 0,895 | 6,32 · 5,29 |
| Rebelle oil | 5,86 | 5,84 · 1,09 | 3,32 | 0,011 | 9 % | 0,872 | 5,01 · 5,04 |

A 40 px: 5,81 · 5,80/0,93 · carris 3,37 · borda 0,819 · leve 6,39 · forte 5,19. O Express Oil e o Acrylic Round não
mudam (ao byte). Regerado o Oil Bristle; a cena dele em `a_lei_da_tinta` repinada (`0x7041…` → `0xf1c4…`).

## §5 — Portões

`o_relevo_dos_fios.rs`: `o_oleo_tem_o_relevo_de_cada_fio` (vermelho antes: 6 falhas com o preset de ontem — a
textura e o forte no vale, ruído 0,25; o campo; borda 0,8–0,9 e através ≥ 4× ao longo passavam já) ·
`cada_knob_do_relevo_morde` (a fracção sem altura = o traço de antes AO BYTE) · `o_relevo_e_so_da_luz` (lâmpadas
apagadas = o traço sem relevo ao byte) · `a_borracha_tira_o_relevo` · 6 unitários (`bristles_relevo_tests.rs`,
`grid/altura_tests.rs`: o instantâneo, o recorte do undo e o limpar com o campo preguiçoso). Os controlos da W2
(`os_fios::sem_fios`, `verde_no_soltar_tests::sem_os_fios`) passam a tirar também o relevo — são veículos.

**Mutação 15/15** (a escala da altura sobrevivia aos portões de produto — a luz satura — e ganhou o unitário
`relevo_luz::tests::a_altura_e_em_px_de_tela`, a 1 e a 2 px por célula).

**Custo** (`sonda_o_custo_do_relevo`, 2048², raio 40, as variantes intercaladas, o mínimo de 7): o move sem relevo
0,99 ms · com relevo e a luz apagada 1,58 · com 1,55 — **o custo é o PENTE, não a luz**. A 1.ª versão (o quadrado do
raio e o ganho de cada fio recalculado por célula: dois Perlin) dava 2,1–2,2; a caixa da pegada (a lâmina chata é ⅓
do quadrado) e o ganho tabelado por fio, ao byte. O tick não muda (0,002 ms).

## §6 — Aberto

- **O leve do Oil Bristle** 6,32 contra 5,01 (a W2 já o deixava em 5,36; o emboss 1 e o relevo sobem-no): o
  Rebelle tem leve ≈ forte. Nenhuma lei desta onda o baixa sem tirar o forte (a régua: `cena_seca_com_rampa`).
- **O Express Oil ao longo** 0,41 contra 1,80: o Hairy dele é carimbo (período 47 %), não fios (48 §3).
- B15 (o Acrylic liso?) continua do dono; nenhuma lei do relevo toca os carris do tapete esparso.

## ⛔ Recusas MEDIDAS

| recusado | porquê (medido) |
|---|---|
| o emboss da massa sozinho (2) para bater 5,86 | 6,97, mas carris 4,49: a massa varia pelo TAPETE, não pelos fios |
| um knob «oleosidade = viscosidade» | largura molhada = seca em todos os óleos e viscosidades (0–2), até com água 1 |
| alturas > 2 px | a luz satura: 5,38 → 5,47 de 2 para 4 px, carris 3,33 → 3,83 |
| escavar 0,65 (o `thin_remove` do Bristle Flat) | 4,82 contra 5,22 a 0,5: o Rebelle tem também fios grossos que constroem |
| o relevo no Express Oil | o controlo 7,94 não se distingue do Rebelle 9,0 (ruído 0,9) |
| o relevo pelo contacto¹ | o leve sobe tanto como o forte (6,61); contacto² 6,32 |

## §7 — O smoke do dono (18/10): ⛔ «ficou pior, muito pixelado»

[Foto do dono](49_smoke_dono_pixelado.png) (o Oil Bristle de fábrica, ~103 px, traço INCLINADO): «em minha opinião
ficou pior que estava, muito pixelado, talvez porque as cerdas tenham ficado mais finas». ⇒ **o relevo SAIU da fábrica**
(o preset de ontem ao byte; a cena de `a_lei_da_tinta` de volta a `0x7041…`); a lei fica no motor, desligada
(`bristleHeight` 0), e os portões dela leem-na com os knobs no teste (`o_relevo_dos_fios::com_relevo`).

- **A régua não o via:** no recto da bancada (horizontal, 41 px) os saltos de 1 px através são 3,97 % contra 3,49 %
  do Rebelle e a textura fina/total 0,73 contra 0,74 — iguais. A bancada é ALINHADA AOS EIXOS: os fios caem nas
  linhas de píxeis. Num traço inclinado cada fio corta os píxeis em diagonal.
- **O mecanismo provável (por medir no inclinado):** a altura é um DEGRAU por fio (`Relevo::alvo` é constante na faixa
  de 1,5 px) e a luz satura (declive ~1) — cada fronteira de fio vira uma linha preta/branca dura, em escada.
  E as faixas têm 1,5 px fixos: a 103 px são ~70 fios (o «cerdas mais finas» do dono).
- O conserto e a régua: [prompt 50](50_PROMPT_W3b_o_relevo_sem_escada.md).
- **Medido no prompt 50** ([50_medicao](50_medicao_o_relevo_sem_escada.md)): a escada confirmada (15,24 · 12,33 contra
  2,48 · 2,30 do Rebelle a 103 px) e uma 2.ª causa — a luz do Impasto MULTIPLICA a cor (3,1× o contraste dele); a
  largura de 1,5 px NÃO era o defeito. Os números das §3–§4 acima são de uma régua cega à sombra (a cobertura).
