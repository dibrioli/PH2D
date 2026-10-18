# 50 — O relevo de cada fio SEM ESCADA (W3b): o que se MEDIU (18/10)

> Prompt [50](50_PROMPT_W3b_o_relevo_sem_escada.md). Régua: `ferramentas/escada.py` e a gémea
> `tests/it/regua_escada.rs` (iguais a 0,01). Sonda: `o_relevo_dos_fios::sonda_a_escada` (PPM em `PH2D_SONDA_OUT`,
> variantes `PH2D_SONDA_KNOBS`, raios `PH2D_SONDA_RAIOS`). Oráculo corrido: `fixtures/rebelle/{oil103,oil40c,
> incl103,incl40}.png` (README da pasta). Dois lotes, o 2.º pelo achado do 1.º. [Prancha](50_prancha_escada.png).

## §1 — A régua da escada (e a régua que mentia)

No miolo do traço (`c > 0,08` erodido 6 px), na direcção LOCAL dos fios (tensor de estrutura da cobertura) e na
**luminância** `Y` (`0..255`): `escada` = rms(ΔY ao longo dos fios, 1 px) · `através` · `contraste` = rms(Y − G₄Y).
O controlo da régua: o mesmo pincel no recto HORIZONTAL (os fios nas linhas de píxeis) e um padrão sintético a 8°
(o degrau lê 3,0× o liso). Cenas: a curva em S da bancada e dois rectos a 8° e 25° (`--cena inclinados`).

| 103 px (Oil Bristle · Bristle Flat) | escada 8° · 25° · curva | recto: escada · contraste | textura (cobertura) |
|---|---|---|---|
| ontem (sem relevo) | 1,74 · 1,25 · 1,50 | 0,51 · 12,3 | 4,68 |
| **18/10** (degrau por fio, luz do Impasto, 1,25 px) | **15,24 · 12,33 · 9,67** | 0,88 · **35,7** | 5,80 |
| Rebelle `incl103`/`oil103` | 2,48 · 2,30 · 2,20 | 0,72 · 11,5 | 6,2 |

A 41 px: Rebelle 2,50 · 2,62 · 2,25 e recto 1,31 · 12,1 (`oil` × `oil40c`: 1,32/1,31 · 11,9/12,3 — o ruído da
sessão ≈ 0,3); ontem 1,98 · 1,77 · 1,81 e 0,85 · 8,7.

- ⛔ **A régua de 18/10 mentia:** a textura pela COBERTURA (o canal que mais se afasta do papel) leu o relevo a 5,87
  contra 5,86 com **3,1× o contraste** do Rebelle na luminância — no azul da bancada o R é 23, e uma sombra preta só
  move a cobertura de 0,91 a 1,0. A barra da W3 mandou a amplitude para onde a régua era cega.
- **A foto do dono** (`49_smoke_dono_pixelado.png`, ~8°) lê do lado mau: contraste 18,4 (Rebelle a 103 px 7,0 no
  inclinado), e a escada a 3 px 1,69 (Rebelle 0,90) — a captura dele está reamostrada (os degraus são blocos de
  2–3 px), por isso a 1 px lê só 2,66.
- **A cor por fio da W2 (degrau) não é escada:** sem ela (`bristleNoiseLight/Sat` 0) o inclinado a 41 px lê 1,81
  contra 1,98 com ela (Rebelle 2,50). A escada era só do relevo.

## §2 — A lei: contínua através dos fios, acesa pelo emboss CINZA

**Lote 1** (a altura contínua, ainda com a luz do Impasto): a mesma amplitude 1,25 desce a escada a 103 px de 15,24 a
5,14, mas o contraste fica em 31,7 — a amplitude e a LUZ.

**O achado** (regressão por píxel do resíduo RGB no recto): o eixo da variação do Rebelle é **cinza** — (0,9 · 1,0 ·
0,9), 75 % da variância; o nosso emboss da massa também (0,84 · 0,88 · 1,0); a luz do Impasto punha-a no eixo do
**albedo** (0,13 · 0,60 · 1,0 — o azul): ela MULTIPLICA a cor, e o R preso perto de 0 precisava de 3× o contraste
para mover a régua. ⇒ a altura entra no MESMO emboss da tinta (`render.rs`: o cinza somado aos três canais, × o
Emboss da célula; `bristles_relevo::LUZ` 20 por px/px) e a ponte para a luz do Impasto saiu.

**A lei** (`bristles_relevo::entre`, `FiosNoDab::entre_fios`): cada fio deixa o seu alvo no CENTRO; entre dois
centros a altura vai pelo meio-cosseno (contínua, declive contínuo, nunca além dos dois; declive máximo
`π/2 · Δ ÷ largura`). O campo guarda px ÷ px por célula (o declive de célula a célula é o de px).

**Lote 2** (contínua + cinza; emboss 0,7 = a fábrica; 41 px | 103 px):

| `bristleHeight` | textura | contraste recto | escada 8° · 25° | textura | contraste recto | escada 8° · 25° |
|---|---|---|---|---|---|---|
| 0 (ontem) | 3,85 | 8,7 | 1,98 · 1,77 | 4,68 | 12,3 | 1,74 · 1,25 |
| 0,5 | 4,69 | 10,8 | 2,09 · 1,83 | 4,65 | 12,5 | 1,81 · 1,33 |
| **0,75** | **5,65** | **13,4** | **2,18 · 1,90** | 5,36 | 14,4 | 1,90 · 1,43 |
| 1 | 6,65 | 16,3 | 2,30 · 2,01 | 6,33 | 17,0 | 2,00 · 1,54 |
| 1,5 | 8,26 | 21,5 | 2,57 · 2,28 | 8,31 | 22,2 | 2,31 · 1,81 |
| Rebelle | 5,86 | 12,1 | 2,50 · 2,62 | 6,2 | 11,5 | 2,48 · 2,30 |

A escada fica do lado do Rebelle em TODAS; o que escolhe é a textura × o contraste. A 103 px o preset sem relevo já
tem o contraste do Rebelle (12,3 · 11,5) e a textura curta (4,68 · 6,2): nenhuma amplitude acerta as duas lá.

**A largura dos fios** (o «cerdas mais finas»), o espectro da luminância através (% por período, recto, 103 px):

| | 2–2,5 | 2,5–3,5 | 3,5–6 | 6–12 | > 12 px |
|---|---|---|---|---|---|
| Rebelle | 8,5 | 23,9 | 23,6 | 34,2 | 9,8 |
| ontem | 2,4 | 10,8 | 46,6 | 20,2 | 19,9 |
| **0,75 · fios 1,5 px** | 2,5 | **18,6** | 38,7 | 18,7 | 21,4 |
| 0,75 · fios 2,5 px | 1,0 | 5,8 | 40,6 | 31,1 | 21,5 |

O Rebelle tem fios FINOS (24 % a 2,5–3,5 px); 1,5 px aproxima-se (18,6), 2,5 px afasta-se (5,8). O de 18/10 tinha
a forma certa (7,8 · 22,7 · 36,4 · 20,1 · 13,0) com **3,6× a energia** e o degrau: o «mais finas» era a escada e o
contraste, não a largura. A largura fica 1,5 px.

## §3 — A fábrica e a barra

Oil Bristle `bristleHeight` 0,75 (o resto como ontem). À largura da foto: textura 3,85 → **5,65** (5,86), forte
3,31 → **4,83** (5,04), contraste 8,7 → 13,4 (12,1), borda 0,887 (0,872), carris 3,07 (3,32), CV 0,016, através 5,63
× ao longo 0,76, a altura no campo 0,63 px. Inclinado: **1,90 · 1,43** a 103 px, **2,18 · 1,90** a 41 (Rebelle 2,48 ·
2,30 · 2,50 · 2,62). Express Oil e Acrylic Round sem relevo (B15: o Acrylic fica com as estrias).

## §4 — Portões

`o_relevo_dos_fios`: `o_relevo_nao_tem_escada` (8°, 25° e curva a 103 e 41 px ≤ Rebelle + 0,3; o contraste do recto
no vale a 41 px e ≤ 1,3× a 103) · `o_relevo_e_do_emboss` (Emboss 0 = sem relevo ao byte; as lâmpadas do Impasto já
não lhe tocam) · `a_altura_e_por_px_de_tela` (grade a 2 px por célula guarda metade) · `o_oleo_tem_o_relevo_de_cada_fio`
(textura e forte no vale, borda, através × ao longo, o campo) · `cada_knob_do_relevo_morde` · `a_borracha_tira_o_relevo`.
`regua_escada`: o controlo sintético e a conferência com o Python. Unitários: `entre_dois_fios_a_altura_e_continua`,
`entre_fios_anda_continuo_de_centro_a_centro`. Identidade: as outras 21 cenas e 19 miniaturas ao byte.

**Mutação 7/7** sangram: o degrau de volta (`entre`), os centros nas fronteiras, `LUZ` ×4, a altura sem o Emboss da
célula, sem o ÷ px por célula (só `a_altura_e_por_px_de_tela` a vê), o relevo apagado no render, e sem o declive
vertical. ⚠️ `LUZ`, o Emboss e o declive vertical só os veem os dois portões de produto (o contraste): o `render.rs`
não tem unitário do termo.

**Custo** (`sonda_o_custo_do_relevo`, 2048², raio 40, o mínimo de 7 rodadas intercaladas, load 2–5): o move 0,68 →
1,44 ms com o relevo (a W3 lia 0,99 → 1,55 a load ~22); o tick 0,001 → 0,002 ms.

## §5 — Aberto

- **A borda a 103 px**: 0,73–0,74 em TODAS as variantes (com e sem relevo) contra 0,87 do Rebelle — de antes da W3.
- **O leve** (C35): 5,36 → 5,72 (Rebelle 5,01; o de 18/10 6,32).
- **O contraste a 103 px**: 14,4 contra 11,5 (1,25×; o preset sem relevo já 12,3) e a textura 5,36 contra 6,2.
- **As linhas ESCURAS do Rebelle**: o resíduo dele tem assimetria −2,2 (41) · −1,1 (103) — sulcos escuros raros num
  campo liso; o nosso é simétrico (+0,2). E a faixa de 2–2,5 px (8,5 % contra 2,5 %): perto do Nyquist de 1 px.

## ⛔ Recusas MEDIDAS

| recusado | porquê (medido) |
|---|---|
| o degrau por fio (o alvo constante na faixa) | escada 15,24 · 12,33 a 103 px (Rebelle 2,48 · 2,30) |
| a luz do Impasto no relevo molhado | multiplica a cor: eixo (0,13 · 0,6 · 1) contra o cinza do Rebelle; 3× o contraste para a textura |
| a textura pela cobertura sozinha como barra | cega à sombra na tinta escura: 5,87 = 5,86 com 35,7 contra 11,5 |
| `bristleHeight` ≥ 1 | contraste 16,3 · 17,0 (1,35× · 1,5× o Rebelle) |
| fios do relevo de 2,5 px | tiram a faixa de 2,5–3,5 px (5,8 % contra 24 % do Rebelle); textura 4,35 a 41 px |
| emboss 1 com o relevo | 0,5 + emboss 1: contraste 14,6 · 16,9 com a textura 6,0 — o emboss da massa sobe também |

## §6 — Smoke

`PH2D_WETPAINT_SMOKE=2`: o Painter abre sozinho em Wet Paint com o Oil Bristle a 103 px e dois traços a ~8° já
pintados — o de hoje em cima, o de ontem (sem relevo) em baixo; o pincel fica o de hoje. [Foto](50_smoke_cena.png)
(tela virtual, `fotografa_cena.sh`, 1.º quadro aos 1,5 s). Os passos ao dono: handoff P50.
