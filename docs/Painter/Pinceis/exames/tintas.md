# EXAME da família `tintas` (J3, catálogo C: C-1 Dip Pen · C-2 Ink Bullet · C-3 Ink Brush · C-8 Fountain · C-9 Reed · C-12 Gloop · C-13 Ink Bleed · C-14 Gesinski · C-15 Wood)

Receitas e portão: `familias/tintas.rs`. Pranchas: `pranchas/tintas/` (cegas e chaves ao lado; `_mypaint` = 2.º oráculo). Régua e custo: `exames/tintas/`. Bancada com
`PH2D_EXAME_DIR=/tmp/ph2d_exame_tintas`. Revisores cegos: 19 agentes novos (3 rondas), o briefing do 05 §2 (os «doc.»
com a frase da identidade, BRIEFING §2.9).

## O que é comum (ler antes dos vereditos)

- **Fotos do Rebelle refeitas** (guarda `foto_estavel.py`, `2e13dbde74` + `8970371f2f`): Fountain Round gestos A era o
  Dip Pen (a B seca da cena anterior, máxΔ 61 — acima do limiar 16 do detector) e tamanhos/sobre tinham o 2.º traço
  fino (o defeito do Rebelle que a sessão nova não repetiu); Reed sobre A era a cena tamanhos. ⚠️ **Para a família
  `caligrafia_marcadores`:** `pen_fountain_square_gestos_A` = `pen_fountain_round_gestos_B` ao píxel (0,06 % > 40) — é
  velha; não a toquei. O recto rápido FINO do Wood é do próprio Rebelle (repetido: máxΔ 66).
- **ArtRage Gloop Pen fotografado** (`fixtures/artrage_bateria/GloopPen/`, com a tabela de tamanho medida).
- **⛔ O borrão do cruzamento molhado (motor grande, para a fila).** Nas CINCO tintas do Rebelle todos os revisores
  puseram em 1.º ou 2.º lugar: o azul sobre o vermelho molhado faz uma gota escura que TRANSBORDA a faixa (acima e
  abaixo, ~20 px); o nosso faz um rectângulo nítido. Sem efeito (`PH2D_EXAME_VARIANTES`): `extDiffusion` 0,015–0,25,
  `waterPerDab` 2–4, `evaporation` 0,2–0,5, `rewet`, `blendForce`, `drag`, `extBackrun`, `extFingering`,
  `edgeDarkening`. O arrasto curto sim (`pickup` 0,05 + `tipClean` 0,05). É física do fluido, não um valor.
- **⚠️ Decisão para o coordenador:** pela regra (§2) um reprovado não entra e um existente reprovado sai. Deixei os
  ficheiros na fábrica (as 3 existentes com a cura, melhor que antes; as 5 novas) — tirá-las deixa a fábrica sem tinta;
  sair = `git rm` + a receita.
- **A cápsula SATURA a tinta carregada:** com pigmento ≥ ~400 a opacidade por carimbo (pressão, taper) não se vê —
  as pontas leves das rampas saíam opacas. Pigmento 150–300 deixa-as claras como no Rebelle (com pontilhado fino).
- **O 200 px do Rebelle é o tecto Max Size 100** (fora da identidade, 04 §4). A decisão 8 (o tecto por preset) está a
  entrar pela família `grafite` (`size_max_px`); quando existir, as canetas levam `100`.
- **Motor (pequeno):** `bf73f3a58b` — a ponta ROMBA do taper (Tip 1 + Opacity) era um controlo morto (o `scale_dab`
  saía em `w ≥ 1`, contra o doc do `tip_start`); `Taper::fade_blunt`, só com tip ≥ 1 e opacity > 0 (nenhum preset de
  fábrica antes ⇒ byte-idêntico). Portão `a_ponta_romba_so_esmaece` vermelho → verde; mutação sangra. Usa-o o Wood.
- **Portão da família:** `cada_tinta_tem_a_sua_identidade` (bateria do EXAME, foto A): 17 regras no meio entre o
  pincel e o que ele separa (grumo, de pé ÷ deitado, rampa, entrada, miolo, orla, sobreposição). Mutação dos nove
  ficheiros (um valor de identidade cada) sangra as nove. `cada_preset_parece_a_sua_midia`: as três tintas verdes (a
  Orla 5 da foto `inkbrush` saiu: era a franja da forma de cerdas que a bateria de 3 cenas desmente — perfil ✓).
- **Custo:** `exames/tintas/custo.txt` (5/40/200 px). ⚠️ load 16–94 em TODAS as medições (as outras famílias): o
  relógio não vale. p95 ≤ 16,7 ms a 5 e 40 px nas oito; a 200 px 21–64 ms nos Wet e no Ink Bleed (Reed 11,7, Gloop
  14,5, Gesinski 16,2). RSS 25–98 MiB; o Ink Bleed (Watercolor) 131–148 MiB. Caneta: só a pressão (a bateria).

## C-1 Dip Pen (WP, existente) — ⛔ REPROVADO (o borrão; os grumos); fica a cura até o motor do borrão
Receita: pigmento 300 (era 1500), `jitter_scale` 0,18 (grumos), sem a forma de cerdas, `pickup`/`tipClean` 0,05,
glaze 0,9. Oráculo Rebelle `pen_dip_pen` (+ MyPaint `soft-dip-pen` 348/454). Régua 430/454. Vizinho
Wood Stick 0,797. Razão: arte-final de tinta com a borda viva do aparo. CAP-13 (a poça onde pára): o oráculo NÃO a
mostra na bateria (nenhum gesto pára). **Cego 1 e 2:** acharam o par (o nosso = o original em 4/6 na 1.ª); R2: «(1) o
original arrasta e escurece a tinta para fora da faixa (2) 200 px o dobro (3) colar de bolas redondas, início fraco
pontilhado». Curado depois do 2: grumos 0,25 → 0,18. Fica: o borrão (motor), o pontilhado fino da ponta leve.

## C-2 Ink Bullet (WP, existente) — ⛔ REPROVADO (o borrão; a tinta que entra clara)
Receita: pigmento 2000, ponta lisa (sem a forma de cerdas — o que o punha a 0,49 do Marker Bullet), `pressureSize`
0,45 (largura 0,55 → 1), `pickup`/`tipClean` 0,05. Oráculo `pen_bullet`. Régua 412/454. Vizinho Marker Liner
0,884 (Marker Bullet passou de 0,49 a > 0,88 — a distinção resolvida do lado do Ink). Razão: contorno de largura quase
fixa a tinta. **Cego 1:** par certo (oráculo ✓, nosso ✓); RESUMO «(1) o original nasce translúcido (2) o escuro do
cruzamento sai da faixa (3) o 200 metade». **Cego 2** (com a entrada clara pela ponta romba a pigmento 200): pôs o
nosso com o Marker Liner — a entrada saía pontilhada ⇒ voltou à tinta lisa. Fica: a entrada clara, o borrão.

## C-3 Ink Brush (WP, existente) — ⛔ REPROVADO (o borrão; a cor mais saturada)
Receita: dureza 0,9 (lados nítidos; era 0,1 salpicado), sem a forma de cerdas, fios do motor (`bristleLaneWidth` 1,0,
`bristleNoiseLight` 0,25 = as riscas, `bristleTipRagged` 0,8), pigmento 150. Oráculo `brush_brush`. Régua 282/454
(a cor: o nosso mais saturado). Vizinho Dip Pen 0,864. Razão: pincel de nanquim de pontas em fios e riscas. **Cego 1 e
2:** par certo; R2: «(1) riscas de cerdas × grão de pontinhos (2) o arrasto do vermelho (3) a rampa entra pela
opacidade». Curado depois do 2: as riscas e sem a orla. Fica: o borrão, o tom mais saturado.

## C-8 Fountain Pen (WP, novo) — 🔀 FUNDIDO no Ink Bullet (saiu)
Rebelle Fountain Round. Fora da rampa é o Ink Bullet ao píxel (cego do Bullet: «o terceiro não é outro pincel»); só
a pressão o separa (rampa 0,14 × 0,60) — a pena lisa de pressão plena é o Studio Pen (`canetas`). Saiu (`8970371f2f`).

## C-9 Reed Pen (WP, novo) — ⛔ REPROVADO (o borrão; a borda e a ponta leve)
Receita: nasce do Dip Pen; a ponta CHATA fixa (`dab_flatten` 0,43 a 90°, quina 0,8), `jitter` 0,04 + `jitter_scale`
0,15 + gate 0,6 (a borda que o papel morde), pigmento 200 + `pressureOpacityCurve` 2,5 (a ponta leve clara). Oráculo
`pen_reed_pen`. Régua 405/454. Vizinho Dip Pen 1,033. Razão: a cana — grossa no horizontal, fina no vertical.
**Cego 2:** escolheu o NOSSO como original nas 6 linhas (o par certo); RESUMO «(1) 200 px metade (o tecto) (2) o
borrão (3) a imitação não tem o grão do papel: degradê liso, borda em degraus» — a (3) é do oráculo (o nosso tem o
grão). Fica: o borrão.

## C-12 Gloop Pen (Digital, novo) — ⛔ REPROVADO (lê-se como irmão do Gesinski, não como o ArtRage)
1.ª receita na Watercolor (fundia a tinta molhada e clareava o vermelho: o cego 1 pôs o nosso com o Technical Pen) ⇒
Digital em Multiply com o traço-camada L8 (tecto 0,7, orla queimada 2,5 px), `pressureSize` 0,8, `speedSize` 0,1.
Oráculo ArtRage `Gloop Pen/default`. Régua 139/227. Vizinho Gesinski 0,884. **Cego 2:** o nosso como original nas 3
linhas (par certo). **Cego 3:** pôs o nosso com o GESINSKI e o ArtRage como «claramente outro» (a orla do ArtRage é
granulada, em salpicos a 200 px; os cruzamentos dele cobrem mais). Fica: a orla granulada; a gota que EXPANDE parada
(CAP-13) não se mede na bateria.

## C-13 Ink Bleed (Watercolor, novo, doc.) — ⛔ REPROVADO (não sangra pelas fibras)
Fonte: Procreate Ink Bleed (01b §3). Receita: Watercolor depth 6, `paper_edge` 1, warp 5,
sem bordas alisadas, `wet_pull` 0,5 + `wet_dilution` 0,3 (sem eles a matriz punha-o a 0,49 do Hard Round). Vizinho
Hard Round 0,946. **Cego 2** (identidade escrita): cumpre a «borda áspera» mas «é ruído granulado uniforme, não tinta a
correr pelas fibras: nem fios, nem halo, nem cresce com o tamanho; o molhado não faz nada». Motor: a capilaridade pelas
FIBRAS (CAP-14) — o Wet tem papel, mas a tinta não corre por ele para fora do traço (fila).

## C-14 Gesinski Ink (Digital, novo, doc.) — ⛔ REPROVADO (a tinta lê-se «chapada/aguada»)
Fonte: Procreate Gesinski Ink (01b §3). Receita: Digital Multiply,
tecto POR TRAÇO 0,55 (L8: um traço não escurece a si mesmo; dois sim), `pressureSize` 0,9 curva 1,4 (um fio na ponta
leve). Vizinho Gloop 0,884 (a 0,7 caía a 0,52). **Cego 2 e 3:** a identidade cumpre-se (translúcida, escurece na
sobreposição, borda limpa, fio → cheio); faltas: «chapada como forma vetorial, sem variação de densidade; pálida;
a ponta 0,15 vira agulha». Curado o fio da ponta leve (cego 2 → 3). Fica: a densidade/textura de tinta (o grão do
Digital não aparece sob o tecto em Multiply — medido: Grain 12 a profundidade 0,2–0,35 invisível).

## C-15 Wood Stick (WP, novo) — ⛔ REPROVADO (o borrão; borda e tom)
Receita: nasce do Dip Pen, água 0,5, pigmento 60 (80 mais escuro, 40 mais claro que o Rebelle), grumos `jitter_scale`
0,3 + `jitter` 0,04, a ponta ROMBA do taper (a tinta que entra clara). Oráculo `pen_wood`. Régua 390/454. Vizinho Dip
Pen 0,797. **Cego 1 e 2:** par certo, o nosso escolhido como original (confiança baixa/média); R2: «o recto rápido do
original vira um rectângulo (é do Rebelle), 200 px metade, a borda do original é serrilhada miúda e mais escura e os
cruzamentos com halo». Fica: o borrão, a borda serrilhada do Rebelle.

## O que falta (fila)
1. **O borrão do cruzamento molhado** (as 5 tintas do Rebelle) — física do fluido do Wet. 2. **A capilaridade pelas
fibras** (Ink Bleed, CAP-14). 3. **O tecto de tamanho** (decisão 8, `size_max_px` da `grafite`) — Max Size 100 nas
canetas do Rebelle. 4. **A entrada clara sem pontilhado** (o loading do Rebelle) — a cápsula satura a tinta carregada.
5. A orla granulada do Gloop do ArtRage; a textura sob o tecto do Digital (Gesinski).
