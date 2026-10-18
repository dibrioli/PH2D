# 48 — Os fios e a textura do traço (W2): o que se MEDIU (18/10)

> Prompt [48](48_PROMPT_W2_os_fios_e_a_textura_do_traco.md). Régua: `ferramentas/mede_traco.py` portada em
> `tests/it/regua_midia.rs` (o PERÍODO e os CARRIS novos, iguais ao Python nos 19 PPM). Sonda:
> `tests/it/os_fios.rs::sonda_a_barra_da_w2` (`PH2D_SONDA_KNOBS`, `PH2D_SONDA_RAIOS`), cena seca da bancada (03 §1).

## §1 — A barra da W2 HOJE (antes, `8723538a73`), os 19 pincéis com cerdas ou grão

A barra de 08/10 (05 §3) foi escrita para o Wet Round, o único pincel de então: **textura 5–10 %** (óleo 5,9 ·
express 9) · **CV ≤ 0,02** · **período < 15 %**. Recto da bancada, seco, à largura da foto do Rebelle (e a 40 px).

| pincel | larg | textura | CV | período | Rebelle (foto): textura · CV · período |
|---|---|---|---|---|---|
| Wet Round | 32 | 7,76 | 0,025 | 6 % | wcround 2,35 · 0,038 · 5 % |
| Wet Wash | 39 | 14,29 | 0,036 | 3 % | wcwash 4,96 · 0,018 · 12 % |
| Wet Dry Brush | 28 | 26,81 | 0,047 | 3 % | — (o Flat Dry sob Wine sai liso: o seco é do papel) |
| Wet Drip | 42 | 11,84 | 0,170 | 12 % | — |
| Wet Liner | 38 | 3,93 | 0,020 | 3 % | wcliner 1,71 · 0,008 · 10 % |
| Wet Flat | 34 | 5,45 | 0,025 | 4 % | wcflat 5,08 · 0,020 · 5 % |
| Gouache Flat | 39 | 4,02 | 0,021 | 3 % | gouacheflat 2,50 · 0,023 · 8 % |
| **Oil Bristle** | 40 | **2,75** | 0,001 | 3 % | oil **5,86** · 0,011 · 9 % |
| **Acrylic Round** | **29** | **30,49** | 0,024 | 4 % | oilround **3,13** · 0,000 · 3 % |
| **Express Oil** | 40 | **3,20** | 0,001 | 7 % | express **9,00** · 0,008 · 47 % |
| Graphite 8B | 27 | 8,78 | 0,065 | 12 % | pencil 9,09 · 0,037 · 7 % |
| Charcoal | 15 | 7,61 | 0,104 | 10 % | charcoal 8,30 · 0,053 · 7 % |
| Soft Pastel | 16 | 7,71 | 0,052 | 6 % | pastel 7,21 · 0,042 · 3 % |
| Oil Pastel | 26 | 13,30 | 0,060 | 5 % | oilpastel2 17,35 · 0,053 · 6 % |
| Ink Bullet | 42 | 1,28 | 0,006 | 4 % | ink 0,39 · 0,002 · 7 % |
| Dip Pen | 38 | 0,46 | 0,012 | 2 % | dippen 1,43 · 0,008 · 4 % |
| Ink Brush | 40 | 3,12 | 0,017 | 3 % | inkbrush 1,27 · 0,008 · 4 % |
| Marker Bullet | 40 | 2,01 | 0,010 | 14 % | markerbullet 0,66 · 0,001 · 30 % |
| Marker Liner | 17 | 2,78 | 0,009 | 3 % | marker 2,12 · 0,001 · 24 % |

**O que já passa e o que falta:**
- **Período < 15 %: os 19 passam** (o pior, o Marker Bullet, 14 %). O «colar de contas» de 08/10 (16 %) saiu com a
  largura da W2 de 11/10 e o espaçamento dos óleos.
- **A barra 5–10 % / CV ≤ 0,02 é do ÓLEO** — o Rebelle dos outros meios já a fura (o redondo de aquarela tem CV
  0,038, o lápis 0,037, os pastéis 0,042–0,053). Por pincel, a barra é a FOTO de cada um.
- **Só no Rebelle `PARTICLE` há fios** (`fixtures/rebelle_presets_281.json`: 15 de 281, o óleo Bristle Flat entre eles);
  o Round Hard (acrílica), o Hairy (express), os redondos de aquarela, tinta e marcador são carimbos. ⇒ o que os
  números pedem aos FIOS: **o Oil Bristle** (liso demais: 2,75 contra 5,86) e **o Express Oil** (3,20 contra 9,00).
- **Os carris do Acrylic Round** (30,5 contra 3,1; 29 px contra 39): o tapete é ESPARSO (4 000 pontas de 0,5, os
  valores que o dono afinou a olho em 11/10) — há faixas sem pelo nenhum. Nenhuma lei dos fios o cura (§3).
- Fora dos fios (medido, não desta onda): a textura 2–3× do Rebelle nos redondos de aquarela/tinta/marcador é o
  tapete inteiro como `texv` (a ponta LISA é a B6, pendente do dono); o CV dos grãos (8B, carvão) é o papel (W4).

## §2 — Os carris, medidos

O tapete é lido no desvio ao centro do dab ⇒ cada linha do traço lê a MESMA linha do tapete do princípio ao fim.
A régua nova (`Medida::carril`): o desvio da cobertura AO LONGO de cada linha do miolo, em % da cobertura.

| | Rebelle oil | o nosso Oil Bristle | Rebelle express | o nosso Express |
|---|---|---|---|---|
| carris | **3,32** | **0,00** | 0,85 | 0,03 |

Recto × rápido da mesma foto (o ruído do Rebelle na régua): oil 1,38 · express 0,81 ⇒ ela distingue o óleo (o
controlo a 3,3 dela), não o Express. **Dois mecanismos diferentes:** os fios do óleo do Rebelle variam PELO caminho;
as faixas do Hairy são quase constantes — a textura dele é o CONTRASTE entre faixas (falhas brancas nítidas).
E a textura do óleo é quase toda ATRAVÉS do traço (o resíduo 5×5 só ao longo: Rebelle 0,98 %, através 5,48 %).

## §3 — As leis (`ph2d-wet-paint/src/bristles_fios.rs`, `bristles_cor.rs`) e a varredura

Cada fio = uma faixa de `bristleLaneWidth` px na normal do traço (o rumo do dab); por fio, no comprimento de
caminho do dab: o ganho `max(0, 1 + bristlePressure · ruído − bristleLift · (1 − contacto))` (contacto = raio do
dab ÷ nominal, a pressão pela curva do tamanho), a fronteira que ondula (`bristleWave`), o desvio do span
(`bristleSpan`), a oscilação do pincel inteiro (`bristleOscillation`) e a cor HSLuv (`bristleNoiseHue/Sat/Light`).
O `ruido` é o Perlin com a FASE do fio e uma 2.ª oitava.

Nove rodadas de variantes (todas no mesmo binário, `PH2D_SONDA_KNOBS`), a escolha de cada uma pela anterior:

| achado | medido |
|---|---|
| ⛔ Perlin sem fase: todos os fios passam pelo zero no MESMO arco | Express 149 px a 57 % da potência (`ruido_px` 150) |
| no óleo saturado (pigmento 2 000) a pressão só abre FALHAS brancas | p 2,2: textura 5,5, mas falhas no miolo que o Rebelle não tem |
| a luz HSLuv quase não mexe a régua (o canal que ela lê é o vermelho do azul) | L 0,1: 2,75 → 2,86; a SATURAÇÃO mexe-o |
| o matiz nos valores do preset do Rebelle (h 0,08) em voltas é berrante | roxo e verde-azulado; a unidade dele não é a nossa |
| span / oscilação / onda não fecham os carris do Acrylic | 30,5 → 21–23 no melhor, CV 0,04–0,05 (Rebelle 0,000) |
| Express: o ruído CURTO varia as faixas pelo caminho (carris 4,95 contra 0,85) | o ruído de 2 000 px (quase constante): 0,88 |

## §4 — A fábrica (os dois que os números pedem)

[Prancha](48_prancha_os_fios.png): antes · depois · Rebelle, o recto e a rampa leve → forte → leve. Regerados os dois
(`PH2D_REGERA_SO`); as receitas em `tests/it/receitas_de_fabrica.rs`.

| à largura da foto | knobs | textura | CV | período | carris | cobertura | leve · forte |
|---|---|---|---|---|---|---|---|
| Oil Bristle antes | — | 2,75 | 0,001 | 3 % | 0,00 | 0,907 | 3,15 · 2,66 |
| **Oil Bristle depois** | pressão 1,2 · fio 1,5 px · levanta 0,4 · saturação 0,3 · luz 0,25 | **3,85** | 0,017 | 2 % | **3,03** | 0,913 | 5,36 · 3,31 |
| Rebelle oil | | 5,86 | 0,011 | 9 % | 3,32 | 0,909 | 5,01 · 5,04 |
| Express Oil antes | — | 3,20 | 0,001 | 7 % | 0,03 | 0,715 | 4,83 · 2,96 |
| **Express Oil depois** | pressão 2,5 · fio 2 px · levanta 0,2 · ruído 2 000 px | **7,82** | 0,006 | 5 % | 0,86 | 0,704 | 11,37 · 7,02 |
| Rebelle express | | 9,00 | 0,008 | 47 % (o carimbo) | 0,85 | 0,700 | 10,16 · 7,31 |

A 40 px: Oil 3,80 / 0,017 / 4 % / 3,06 · Express 8,03 / 0,008 / 6 % / 0,95. Os outros 17 iguais ao §1 (ao byte).

## §5 — Portões

`tests/it/os_fios.rs::os_oleos_tem_fios_e_nao_carris` (vermelho antes: 12 falhas; o controlo é o preset sem os
fios) · `cada_knob_dos_fios_morde` (os 12 nomes; os comprimentos sozinhos ao byte) · 11 unitários
(`bristles_fios_tests.rs`, `bristles_cor.rs`: os valores de referência do HSLuv) · identidade: as outras 18
miniaturas, 20 das 22 cenas de `a_lei_da_tinta` e `os_outros_meios_nao_mudam` ao byte; `cada_preset_parece_a_sua_midia`
verde. **Mutação 11/11** (a fase do ruído, a mão leve, a cor do fio, a onda, o span, a oscilação, o hospedeiro sem os fios, a cor que não chega ao motor, os dois knobs lidos como constante, a régua dos carris a zero) — todas sangram.

## §6 — Aberto

- **O resto da textura do óleo** (3,85 contra 5,86) são os fios ESCUROS do Rebelle — a sombra do relevo de cada fio
  (o óleo dele tem Add/Remove impasto por fio): é a W3 (altura/oleosidade), cujo portão já diz «textura na banda da W2».
- **O Acrylic Round** (30 % de carris contra o liso do Rebelle): o tapete esparso que o dono afinou. Pergunta ao dono
  no smoke: quer a acrílica lisa como a do Rebelle (o tapete mais denso ou a ponta lisa)?
- **O Wet Dry Brush** não tem foto honesta (o Flat Dry do Rebelle sob Wine sai liso).
- Thin/Thick/Edges (as três populações do Rebelle) não entraram: nenhum número as pediu.
- O painel dos fios (os 12 knobs estão `Hidden`, por nome no preset): W8.

## ⛔ Recusas MEDIDAS

| recusado | porquê (medido) |
|---|---|
| a pressão por fio forte no Oil Bristle para bater 5,9 | falhas brancas no miolo (o Rebelle tem cobertura cheia 0,909) |
| o ruído curto no Express | carris 4,95 contra os 0,85 do Rebelle: as faixas dele não variam pelo caminho |
| o matiz por fio | berrante já a 0,08 volta; a luz/saturação dão os fios claros e escuros |
| copiar o período 47 % a 6 px do Hairy | é o carimbo dele, não a mídia (03 §3.1) |
| a luz HSLuv sozinha para o óleo | quase não mexe o canal que a régua lê (2,86); com a saturação e a pressão, sim |
