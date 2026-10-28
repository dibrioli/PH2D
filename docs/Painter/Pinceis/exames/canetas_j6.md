# J6w — canetas e marcadores no Wet Paint (C-9 · C-15 · E-3 · E-4), 28/10

Exame por VALORES (sem motor novo), ≤ 2 rondas × ≤ 4 variantes. Receitas em `EM_ESTUDO_J6_WET` (fora da fábrica):
`familias/canetas.rs` (Reed Pen, Wood Stick) e `familias/caligrafia_marcadores.rs` (Marker Chisel, Dry Marker); o
escritor de estudos (`aguarela/estudo.rs`, `"receita"`) lê as duas listas. Presets de estudo:
`/tmp/claude-1000/j6_canetas/estudo/<Nome>.ph2d-brush.png`. Bancada `exame.sh` (cópia com `FIXA`), biblioteca fixa
`/tmp/claude-1000/j6_digital/biblioteca` (65 presets; a ronda 1 correu sobre a `j6_wet`, incompleta — os d da ronda 1
não valem, os da ronda 2 sim). Oráculo: as fotos do Rebelle (`fixtures/rebelle_bateria/{INK_PEN,MARKER}`). Pranchas
`pranchas/J6w_<Nome>{,_cega}.jpg` + `_chave.txt` (ordem sorteada por linha; o título diz a variante: «C»/«D» = a
receita final). Réguas `exames/canetas_j6/J6w_<Nome>_regua.{txt,json}`. Revisão cega: POR FAZER (coordenador).

| pincel | variante | régua | escala | vizinho (d; limiar 0,591) | estado |
|---|---|---|---|---|---|
| C-9 Reed Pen | C | **418/454** | 0,42 | Dip Pen 1,450 | pronto p/ cega; motor em falta (200 px, serrilha) |
| C-15 Wood Stick | C | 368/454 | 1,43 | Ink Brush 1,062 | pronto p/ cega; motor em falta (velocidade, elipse sob dual) |
| E-3 Marker Chisel | C | 358/432 | 0,40 | Marker Bullet 1,072 | pronto p/ cega |
| E-4 Dry Marker | D | 368/454 | 0,74 | Marker Bullet 1,723 | pronto p/ cega; motor em falta (riscas) |

## ⚠️ Achado de motor: no Wet o dual DESFAZ o achatado

Medido (recto horizontal × vertical, cena `gestos` A, à escala do oráculo): Reed Pen com o dual (L6) w0 33 · w90 **32**
(r1 A e B, e D com `jitter_scale` 0); a MESMA receita sem o dual (C) w0 39 · w90 **24** (Rebelle 34 · 23). O Wood Stick
com `achatado` (0,17, 0) e dual: w90 29 = sem achatado (Rebelle 36). Sem o dual o achatado do Wet funciona (Chisel w0
15 · w90 38, Rebelle 16 · 39). ⇒ a pegada do recorte do dual (`dab_route.rs`, `recorte_do_dual_no_wet` → `padrao(…,
&sil, …)`) volta ao disco. **Fila** (não feito: a ordem desta onda é «motor em falta: meça e escreva»).

## C-9 Reed Pen — cana cortada

Receita: a da J3 (nasce do Dip Pen; água 0,3, pigmento 200, gate 0,6, glaze 0,5, reservatório 0,4) com `achatado`
(0,43, 90°), quina 0,8, **sem** dual. Rondas: r1 A (dual 0,3 D) 355 · B (dual 0,2 D + pontas rectas) 358 — a borda
saía em bolhas onduladas a 200 px e o achatado sumia; r2 **C (sem dual) 418** · D (dual 0,12 D, curva da largura 2) 338.
Razão de uso: a tinta rústica de traço chato — largo deitado, fino de pé — que nenhuma caneta da biblioteca (todas
redondas) dá. Ainda se vê: (1) **200 px: 194 contra 81** — o Size do Wet com a ponta achatada de pé pinta ~2,3× a
largura prometida (escala 0,42) e o tecto (`size_max_px` 95) não a apanha: é a fila J3 nº 1 («a lei da largura de uma
ponta achatada»); (2) a borda lisa (o Rebelle serrilha 1–2 px; o dual que o faria apaga o achatado — acima); (3) o
cruzamento: o Rebelle pousa uma NÓDOA redonda escura, o nosso uma faixa escura ao comprido; (4) a rampa de pressão
mais larga na ponta leve (largura 20 contra 18, ok) mas mais clara (cobertura 0,52 contra 0,60).

## C-15 Wood Stick — pau mergulhado em tinta

Receita: a da J3 (nasce do Dip Pen; água 0,5, pigmento 60, glaze 0,6, reservatório 0,3, taper 1,5 D) + os grumos pelo
dual (`GRUMOS_DO_PAU`: ruído 0,35 D, espalhar 0,25, através 0,3, Multiply), `jitter_scale` 0, `pressureOpacity` 0,7,
`achatado` (0,17, 0) (hoje sem efeito — acima). Rondas: r1 A (dual 0,6 D) 363 · B (dureza 0,8, sem taper) 358; r2
**C 368** · D (+ glaze 0,8, reservatório 0,5) 346 (o cruzamento mais escuro custou a régua da cena `sobre`). Razão de
uso: o esboço a tinta solto — borda grumosa e irregular, tinta mais rala que o Dip Pen. Ainda se vê: (1) o recto
RÁPIDO do Rebelle sai fino e liso (o Wet não tem `speedSize` — fila J3 nº 3); (2) a vertical mais estreita (29 contra 36:
a elipse sob o dual); (3) os grumos ainda mais fundos que os do Rebelle (rugosidade 1,09 contra 0,70) e a 200 px a
largura 53 contra 71; (4) as nódoas do cruzamento sobre o vermelho menores; (5) a rampa leve mais larga (26 contra 15).

## E-3 Marker Chisel — bisel fixo deitado

Receita: nasce do Marker Bullet (glaze 1, sem arrasto, ponta lisa), `achatado` (0,61, 4°) (L3: L 0,94·D × T 0,37·D),
dureza 0,3, quina 0,8, `pressureSize` 0, opacidade pela pressão (curva 3), pontas rectas (além 0,3) que esbatem (4).
Rondas: r1 A (dureza 0,5, esbate 2) 354 · B (0,3, esbate 1) 360; r2 **C (0,3, esbate 4, curva 3) 358** · D (0,4, esbate
6) 348. Escolhida C: o esbate maior é a identidade da ponta (o Rebelle esbate as tampas ao comprido) e a régua fica
dentro do ruído da B. Vizinhos: Marker Bullet 1,072, Dip Pen 1,612, **Marker Liner 1,632** (o bisel Digital: distinto).
Razão de uso: preencher áreas a marcador — faixa larga de pé, fio deitado, a tinta que multiplica a meio no cruzamento.
Ainda se vê: (1) a 200 px a tampa do nosso é recta e cheia; a do Rebelle esbate ~40 px e sai do quadro (o esbate a 4 mal
se vê a 200); (2) as bordas laterais do Rebelle são macias (rugosidade 0,07; nós 0,17–0,35 — a quina 0,8); (3) a rampa:
o Rebelle começa já a 0,87 de cobertura, o nosso do zero (o esbate da entrada come o início leve: `pontas_ini` 0).

## E-4 Dry Marker — marcador a acabar

Receita: nasce do Marker Bullet; água 0,05, **pigmento 90**, dureza 0,9, `achatado` (0,7, 100°) (L3), quina 0,8, a forma
`Gradient` (`PONTA_QUE_ACABA`, a da J3 Digital: cheia de um lado), pontas rectas, `spacing` 3, opacidade pela pressão
(curva 2,5). Rondas: r1 A (pigmento 300) 274 · B (+ spacing 5, achatado 0,8) 266 — chapado: a rampa da forma só aparece
ABAIXO da saturação do pigmento; r2 C (pigmento 40) 238 (some: cobertura 0,28) · **D (90) 368**. Razão de uso: a textura
de marcador gasto — faixa que esvai de um lado, fio de pé. Ainda se vê: (1) as **riscas** dos carimbos ao comprido a
200 px (o Rebelle: linhas verticais finas; o nosso liso — fila J3 nº 6, motor: dabs soltos com a borda visível); (2) a
200 px 98 contra 71 de largura e mais cheio (0,81 contra 0,65 — o Rebelle tem Max Size; o tecto a medir); (3) a 40 px
mais claro (0,50 contra 0,65: o pigmento entre 90 e ~130 — sem 3.ª ronda); (4) o lado cheio da rampa: o do Rebelle é o
de CIMA em todo o gesto; confirmar na cega se o nosso segue (a forma `ViewPlane` na pegada a 100°).

## Custo, portão, mutação

Não medidos nesta onda (a ordem: exame por valores; o custo e o portão por foto entram com a promoção). Load 5–20
durante as corridas (o relógio não valeria). Escrever os presets de estudo é o passo lento: ~2,5 min por preset (a tabela
da largura), 22 min para 8.

## Fila (motor — medido, não feito)

1. **Dual × achatado no Wet** (Reed, Wood): o recorte do dual devolve o disco (medição acima). Desbloqueia a serrilha da
   Reed Pen e a elipse do Wood Stick.
2. **A lei da largura da ponta achatada de pé** (Reed): Size 84 (200 × 0,42) pinta 194; o tecto não a apanha.
3. **`speedSize` no Wet** (Wood: o recto rápido fino).
4. **As riscas dos carimbos** (Dry Marker a 200 px).

## J6x — o dual DENTRO do achatado (motor) e a ronda com o dual de volta, 28/10

**Mecanismo** (fila nº 1 acima): a máscara da 2.ª ponta é uma faixa ISOTRÓPICA à volta do caminho, do tamanho da ponta
REDONDA, e o Wet lia-a no píxel onde a tinta pousa (`RecorteNoWet::padrao`). Uma 2.ª ponta mais estreita que o eixo maior
cortava os dois eixos à largura dela ⇒ disco. (O «w90 32 contra 24» da J6w juntava a isto a escala do exame: o recto
horizontal recortado subia a escala.) **Cura** (`9dfb735b24`): a máscara lê-se no ponto da ponta redonda que corresponde
ao píxel (`PegadaDaPrimeira::redonda`, o eixo menor esticado a 1/menor à volta do centro do dab); sem achatado, a leitura de
sempre (neutro ao bit: as três portas de fábrica dão os MESMOS vermelhos, todos Oil/* da obra Real Oil, com e sem a cura).
Gate `o_dual_brush::o_dual_modula_dentro_da_ponta_achatada_no_wet` (razão vertical÷horizontal por desvio-padrão, ponta 0,43
a 90°, 2.ª ponta 0,3 D): sem dual 0,552 · com 0,639 · mutação (sem a pegada) **0,925 VERMELHO**.

**O tamanho em pé (200 px: 194 contra 81) NÃO é o mesmo mecanismo** — a Reed C não tinha dual. Com o dual de volta a
Reed pinta 94 a 200 px (o recorte da 2.ª ponta segura a largura que a tabela deixava crescer); a lei da largura da ponta
achatada continua na fila (nº 2).

Ronda única, 4 variantes cada, biblioteca fixa `j6_digital`, bancada `PH2D_EXAME_DIR=/tmp/claude-1000/j6_dual`:

| pincel | variantes (régua /456) | escolhida | w0 · w90 (Rebelle) | 200 px | vizinho (d) |
|---|---|---|---|---|---|
| C-9 Reed Pen | E 417 · **F 423** · G 417 · H 419 (J6w C sem dual: 418) | F: `SERRILHA_DA_CANA` (0,8 D, dura 0,6, através 0,15) | 34 · 21 (34 · 23) | 94 (81) | Dip Pen 0,924 |
| C-15 Wood Stick | E 377 (os grumos de antes) · F 374 · **G 392** · H 355 (J6w C: 368) | G: grumos 0,45 D, espalhar 0,15, através 0,2 | 30 · 36 (30 · 36) | 97 (71) | Dip Pen 1,563 |

Pranchas `pranchas/J6x_{Reed_Pen,Wood_Stick}{,_cega}.jpg` + `_chave.txt` (ordem sorteada por linha); réguas
`exames/canetas_j6/J6x_*_regua.{txt,json}`. Revisão cega: POR FAZER (coordenador). Ainda se vê — **Reed**: a 200 px as
tampas arredondadas e a serrilha mais fina que a do Rebelle (ele: tampa a esquadro e dentes de 2–3 px); o cruzamento sem a
NÓDOA redonda escura; as rampas mais claras e esfiapadas na entrada. **Wood**: a espiral cheia (o Rebelle deixa as voltas
abertas); o recto rápido não afina (`speedSize`, fila nº 3); a 200 px a entrada esbatida e os grumos maiores (97 contra 71);
as rampas mais largas e claras.
