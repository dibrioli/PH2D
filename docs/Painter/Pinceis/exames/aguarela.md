# Exame da família `aguarela` (J4, catálogo I — aguarela e sumi-e)

Receitas: `crates/ph2d-tool-painter/tests/it/familias/aguarela.rs` (`WET`; os reprovados em `EM_ESTUDO`, sem ficheiro).
Pranchas: `pranchas/aguarela/` (+ `cegas/`, `chaves/`). Régua das fotos: `exames/aguarela_foto.py`. Oráculo: Rebelle
`rebelle_bateria/WATERCOLOR/` (A molhada · B seca); Krita/MyPaint onde ele não tem. Bancada com
`PH2D_EXAME_DIR=/tmp/ph2d_exame_aguarela`, variantes num processo (`PH2D_EXAME_VARIANTES`); load 2–33. Seis revisores
cegos novos (05 §2 tal e qual, a prancha copiada para um nome neutro).

**Aprovados: nenhum. Pré-existentes não aprovados, ficam COMO ESTAVAM (decisão 3; as curas medidas partem leis da J2/J3
ou não convencem o revisor): Wet Round · Wet Wash · Wet Liner · Wet Flat · Wet Dry Brush · Wet Drip. Reprovados (em
estudo, sem ficheiro): Mop · Bristle Watercolor · Sumi-e · Granulation · Fringe Bloom · Spatter Watercolor · Optical
Wash. Fundidos: nenhum (a fusão Dry Brush → Wet Flat fica proposta).** Motor: nada entra (duas leis medidas e
revertidas, I-10). ⚠️ As pranchas de Wash, Liner e Dry Brush mostram a CURA medida (não a receita, que voltou).

## ⭐ A lei que os revisores repetiram (6 revisores, 5 pranchas)

Em Wash (r1, r2), Liner (L1, L1b) e Flat (F2) o revisor escolheu o NOSSO como «original» em 6/6 linhas (confiança
média/baixa) e listou do nosso lado o mesmo: o miolo MOSQUEADO/listrado, o pigmento a juntar-se nas pontas e numa orla
de miolo oco na pressão fraca, o halo esfiapado a 200 px. O Rebelle pinta a aguarela LISA (um véu uniforme com um fio de
1 px). Nenhum valor desfaz isto (medido: `bristleStrength` 0 apaga o traço; `visualGrain`/`paperVisibility` 0 → 0 px
diferentes; `leveling` 0,2; evaporação 0,4–2; `bristleLaneWidth` 6; `edgeDarkening` 0–200: régua igual ou pior). ⇒
**motor**: o miolo liso do fluido e a orla que esmaece com a pressão.

## Pré-existentes

**I-1 Wet Round** — controlo dos portões e pincel sem preset: não mexido. Régua 297/454 (o Rebelle `Round` pinta 30 px
a «40»: escala 0,769); a cor é clara (cobertura 0,23 × 0,32). Cura MEDIDA: pigmento 78 → 100 · **125** · 150 · 185 =
régua 378 · **419** · 341 · 296/456. Não aplicada: muda o controlo das `MIDIAS` de todas as famílias a meio da janela,
o `painter_default_knobs` e as cenas de `a_lei_da_tinta` — decisão do coordenador/J5.

**I-2 Wet Wash** — cura medida: dura 0,9, papel 0, pigmento 5, orla 200, água por dab 2 (com 1 o início era uma cunha
com um borrão pálido; com 2 a cápsula redonda do Rebelle). Régua 349 → **384/454**, a origem passa a 1.ª da matriz
(0,966). r1 e r2: nosso = «original» 6/6; repetidos o miolo mosqueado e a orla oca nas pontas fracas. E parte duas leis
(`leis_sistemicas`): o traço de 5 px cobre 0,77 do de 40 (Rebelle 0,98) e a rampa fica a 0,277 da origem (> 0,18; com
a largura fixa). ⇒ receita de antes.

**I-3 Wet Flat** — pigmento 44 casava o recto (383 → 399/456) mas a rampa saía da origem (0,212 > 0,18): fica 50 (régua
382/454, vizinho Charcoal 1,542). F1: o revisor emparelhou-o com o Dry Brush curado (abaixo). F2 (sem ele): nosso =
«original» 6/6; listou as riscas, a borda escurecida e as pontas serrilhadas (o Rebelle liso, de pontas redondas) e o
vermelho que o nosso empurra no molhado. Não aprovado.

**I-4 Wet Liner** — cura medida: dura 1, papel 0, orla 200, pigmento 65: régua 322 → **376/454** (vizinho Gouache Flat
0,952). L1 e L1b: nosso = «original» 6/6 nos dois; repetidos o grão, o contorno escuro nas tampas e o halo esfiapado a
200 px. E sobe a textura fora do vale da foto `wcliner` (6,28 × 1,71; `cada_preset_parece_a_sua_midia`). ⇒ receita de antes.

**I-5 Wet Dry Brush** — régua 220/454: riscos soltos contra o chato cheio do Rebelle `Flat Dry`. Curá-lo para a origem
(a forma do Wet Flat, cerdas desfiadas, sem esmaecer) deu 308/454, mas F1 emparelhou-o com o Wet Flat em 6/6 (o oráculo
«claramente outro»): a cura É o Wet Flat com outra curva (decisão 13). Receita de antes; **fusão no Wet Flat proposta**
(tirar o ficheiro mexe em `a_lei_da_tinta`, `os_fios`, `sonda_shape_casa`, `mistura_wet_drip_tests`). O dente de serra
das pontas do Flat Dry não sai (`bristleTipRagged` 1, `bristleStrength` 2).

**I-6 Wet Drip** — sem oráculo (o alvo é o Water Drip do Procreate, documentado). À escala 1 a água desce e junta um
debrum escuro em baixo, mas não ESCORRE em fios (régua 139/418 contra o Round, d 2,34). O fio longo é o «mecanismo não
local» que `solver.rs` (a lei da GOTA) já escreve em falta.

## Novos (todos reprovados)

**I-13 Sumi-e** (Rebelle `Sumi-e soft`) — o Ink Brush com água (sem reservatório nem vidrado, ponta desfiada 1,
pigmento 45). Régua 306 → 378/454, vizinho Charcoal 1,01 (no Rebelle `Brush` × `Sumi-e soft` = 1,37). S1: achou o
Rebelle em 6/6, o nosso como imitação, o vizinho claramente outro; listou as cerdas em PENTE do Rebelle (riscas finas,
um fio escuro de um lado, falhas brancas que mostram as voltas da espiral, bordas duras a 200 px) contra o nosso liso de
pontas redondas e halo esfiapado. (A rampa que «entra mais tarde» é a decisão 9; o cruzamento molhado é do
`oleo_acrilica`.) `bristleStrength` 1,6 + `bristleNoiseLight` 0,45 dão riscas largas e ondeadas (295). Motor: as cerdas
em pente do Wet.

**I-7 Mop** (Rebelle `Mop`) — a lavagem sem o fio, pigmento 15, água 1,5, a pressão no tamanho e na carga. Régua 368/445,
vizinho Sumi-e 0,937 (distinta: no Rebelle Mop × Wash = 1,60). O miolo listrado e um anel claro dentro da orla (o
Rebelle é liso): a lei de cima. ⚠️ `watercolor_mop_sobre_A.png` é a `tamanhos_B` (sha igual): refotografar.

**I-8 Bristle Watercolor** (Rebelle `Bristle Flat`, 46 × 16) — o Wet Flat a 0,65: a forma casa (a espiral abre os
buracos). O pigmento é por COMPRIMENTO e a lâmina deitada varre 3× a área da de pé: pigmento 60 → recto 0,28 · vertical
0,43 (Rebelle 0,44 · 0,42); 80 → 0,56 · 0,73. Régua ≤ 283/456. Motor: o pigmento pela largura da pegada.

**I-9 Granulation** — `extGranulation` 1 + `extValleyGran` 1 + papel 0,6–0,85 = véu liso; `dust` 0,6–0,85 só pontilha o
traço de 5 px. O pigmento não MIGRA para os vales (o viés da secagem muda o tempo; no fim tudo assenta). Motor.

**I-10 Fringe Bloom** (Krita `Watercolor_Fringe`, MyPaint `large_watercolor_fringe`; dono de CAP-14/15) — duas leis
medidas e REVERTIDAS (o diff em `scratchpad/aguarela/pavio_tentativa.{diff,rs}`): (1) aliviar o freio de absorvência nos
vales à frente da orla: +0,4 a +5 % de área, nenhum dedo; (2) o «pavio», um passe de água para os vales vizinhos,
independente de ordem: +0,4 a +1,4 %, nenhum dedo — a água fina da orla evapora antes de andar. `extBackrun` 2 levanta
o vermelho no cruzamento, sem anel de couve-flor. Motor grande: fibras com direcção no papel (ele só guarda a altura).
Os dois oráculos são carimbos espinhosos — um carimbo de franja é recurso do `texturas`.

**I-11 Spatter Watercolor** — dispersão, contagem e tamanho em lei de potência: recursos do `texturas` (o Splatter da J3
reprovou pelo mesmo). Não feito.

**I-12 Optical Wash** (Watercolor, Krita `Waterpaint_Soft_Edges`) — `fill` 0, `edge_gain` 1,5–8, `edge_spread` 6–24,
`opacity` 0,4–1, `warp` 0–10: o contorno oco sai; régua ≤ 157/282. O oráculo apaga os verticais e tem a orla irregular;
o alvo documentado (CP Simple Water, Pc Wet Glaze) não corre aqui.

## Portão

Sem pincel aprovado não há identidade a prender. `os_reprovados_da_aguarela_nao_estao_na_fabrica` prende a saída dos
três novos (mutação: o Sumi-e renomeado para «Wet Flat» → VERMELHO; reposto → verde). Os pré-existentes continuam
presos pelos portões de antes (`MIDIAS`, `a_lei_da_tinta`, `leis_sistemicas`), que a família deixa ao bit. Custo: não
medido (nenhum aprovado).

## Para a fila (coordenador)

1. **O miolo liso da aguarela + a orla que esmaece com a pressão** (Wash, Liner, Flat, Mop) — o motor que destrava os
   três pré-existentes e o Mop; com ele, as curas medidas acima aplicam-se.
2. **O Wet Round**: pigmento 78 → 125 (régua 297 → 419) — com o `painter_default_knobs` e o controlo das MIDIAS.
3. **As cerdas em pente do Wet** (Sumi-e): riscas finas, fio escuro de um lado, falhas brancas, bordas duras a 200 px.
4. **Pigmento pela largura da pegada** (lâminas fixas: Bristle Watercolor; vale também para o Oil Bristle a 0,68).
5. **Granulação**: o pigmento a migrar para os vales.
6. **Franja/couve-flor** (CAP-14/15): fibras com direcção; ou um carimbo de franja (`texturas`).
7. **Escorrido em fio** (Wet Drip; o Drips da J3).
8. Do `oleo_acrilica`: azul sobre vermelho molhado — o Rebelle cobre (lente lisa), o nosso levanta uma flor e empurra
   o vermelho (todas as pranchas).
9. Foto: `watercolor_mop_sobre_A.png` duplicada da `tamanhos_B`.

## Motor J4 — o miolo liso (`Knob::StrokeFilm`)

**O mecanismo (medido, sonda `familias::aguarela::sonda`: o grid logo depois do traço — caneta em baixo, a água
parada — e depois de 240 quadros):** (1) o miolo listrado é o **tapete das cerdas** (a ladrilha 128² do Wet, que a
migração põe no Shape Bristles de todo o preset com `bristleCount` no mapa) lido no desvio ao centro de cada dab: a
mesma linha do tapete é arrastada pelo traço inteiro, e a fase de cada dab (a 8 células) faz um padrão de ±20 %. Mop a
200 px, depósito: listras 0,065 → **0,0004** sem o tapete (tudo o resto igual); um dab sozinho é a ladrilha de pontos.
(2) O domo é a **soma das cordas** dos discos (ombros 0,75–0,80 do centro); as voltas da espiral e as pontas somam mais.
(3) Depois da água, arcos por janela do rasto: o 2.º gole da célula virgem (mancha 0,095 → 0,038) e o tecto de água da
célula, que só morde na água que chega aos bocados. Medidas do Rebelle (as fotos A, a 200 px): listras 0,002–0,013,
manchas 0,002–0,007, ombros ÷ centro 0,90–1,05. O Edge Darkening, o mapeamento Random do tapete e o `extRivulets` não
mexem nisto (lote v1/v2). Régua do miolo: `scratchpad/aguarela_motor/liso.py`.

**A lei** (aditiva; 0 = a soma de sempre, ao bit — o byte e a miniatura de todo preset de fábrica, a impressão digital
do motor, `deposit_rows`, `product_doors` verdes): o traço deixa UM filme — cada célula fica com o MÁXIMO que um dab lhe
trouxe, na espessura do miolo de um traço largo (`1 ÷ CapsuleSpacing` dabs, o peso do fino desfeito: a mesma a 5 e a
200 px), nunca a soma; sem o tapete (o Grain armado continua); sem o 2.º gole, sem o tecto da janela e sem o peso por
comprimento (o 1.º dab da lane pesava 1 contra 0,838: o «disco do 1.º toque» que o revisor viu). O pincel de filme leva
`drag` 0 (o arrasto puxa tinta de trás acima do filme). Commits `0b50329ea2` · `8f7d8166bf` · `d92a015b11` · `16e98dddea`.
Portões (`familias/aguarela/portoes.rs`, red-first): `o_filme_do_traco_e_liso` (Liner 200/40 px: listras 0,103/0,195 →
0,000, ombros 0,75 → 1,00), `…continua_liso_depois_da_agua` (seco e molhado), `o_filme_tem_a_mesma_espessura_a_todo_o_tamanho`,
`o_filme_nao_tem_o_disco_do_primeiro_toque`. Mutações (seis, todas sangram): o máximo, o tapete, a espessura pela corda,
o tecto de água, o peso por comprimento, o 2.º gole. Custo (smoke, load 18–22): Mop p95 6,4 · 9,4 · 6,4 ms, Liner 3,3 ·
3,0 · 4,5 ms a 5/40/200 px; RSS ≤ 94 MiB.

**Os pincéis** (bancada `PH2D_EXAME_DIR=/tmp/ph2d_exame_aguarela_motor`; pranchas `pranchas/aguarela/*_filme.png`):
- **Wet Liner** (pré-existente) — com o filme, papel 0, orla 200, pigmento 17: régua 322 → **375/456**, vizinho Gouache
  Flat 1,29; todas as leis por nome verdes (`a_lei_da_tinta` com o fnv novo). Revisor q3 (o 1.º filme): leu o NOSSO como
  original 6/6. Revisor t9 (o final): achou o Rebelle 6/6 (média) e listou do nosso lado: sem o **fio escuro da borda** e
  sem as pontas mais claras; a 200 px o Rebelle tem o miolo mais carregado e a orla esbatida em arcos (o nosso chapado);
  a espiral sem os anéis finos; a rampa começa opaca (o Rebelle num fio); 5 px mais claro; o azul arrasta o vermelho
  molhado e o cruzamento seco fica violeta-cinza (o Rebelle azul-marinho). ⇒ **curado, NÃO aprovado**; fica o filme.
- **Mop** (em estudo) — filme, pigmento 5,2, pressão do tamanho em curva 0,5, opacidade 1,3: régua 368/445 →
  **405/456**, vizinho Wet Flat 1,03 (claramente longe). Revisores k7 e m2 (novos): o NOSSO lido como o original 6/6 nos
  dois (baixa → média). Do nosso lado ficou (m2): o recto de 5 px quase some; a rampa 0,15→1→0,15 de tampas redondas (o
  Rebelle afina a bico); um grão de papel orgânico no miolo (o Rebelle mais liso); o cruzamento molhado é uma mancha lilás
  com «contas» acima e abaixo. ⇒ **reprovado ainda**, de volta ao `EM_ESTUDO` com a receita do filme (sem ficheiro).
- **Wet Wash** — o filme cura o miolo (régua 350 → 375/456, manchas 0,046 → 0,009, ombros 0,57 → 0,99), mas a rampa de
  pressão fica a 0,196–0,23 da origem (`leis_sistemicas` ≤ 0,18; grelha de 32 variantes de tamanho/opacidade): o Rebelle
  não pinta abaixo da pressão 0,3 e salta à largura cheia. É a **decisão 15** (a régua da rampa acima do início do
  oráculo, por fazer na lei) — com ela, o filme entra. Fica a receita de antes.
- **Wet Flat** — com o filme o traço de 5 px cobre 0,61 do de 40 (< 0,71): a lâmina de 0,6 px a 5 px só é coberta pela
  SOMA das passagens; o máximo de cada dab não a cobre. Régua 383 → 367. Fica a receita de antes. Motor: o filme de uma
  pegada mais fina que o passo pede a pegada VARRIDA (a cápsula da lâmina entre dois dabs).
- **Wet Round** (controlo das `MIDIAS`, pincel sem preset) — o filme NÃO o cura: a ponta macia (dura 0) passa a mostrar a
  cauda do fall-off (largura ×2, escala 0,52) e a régua cai 299 → 137–213. Não aplicado; as `MIDIAS` das famílias não
  mudam. (A cura do pigmento 125, régua 419, é da soma — decisão do coordenador, sem o filme.)
- **Wet Dry Brush → Wet Flat** (proposta, não tocada): tirá-lo mexe em `assets/…/Watercolor/Fluid/Wet Dry Brush.ph2d-brush.png`,
  `familias/aguarela.rs` (a receita), `tests/it/a_lei_da_tinta.rs` (a linha fnv), `tests/it/os_fios.rs`,
  `tests/it/sonda_tilt_do_preset.rs`, `src/tool/paint/wetpaint/sonda_presets.rs`, `sonda_papel_seco.rs`,
  `sonda_shape_casa.rs`, `src/tool/paint/wetpaint_preset/medida_preview.rs`, `01_CATALOGO.md`, `matriz/` (regenerar).
- **Sumi-e** — não feito (as cerdas em pente são outro recurso).

**Foto**: `rebelle_bateria/WATERCOLOR/watercolor_mop_sobre_{A,B}.png` refeitas (a A era a `tamanhos_B`), `421cc42dc7`.

**Para a fila**: (1) a régua da rampa acima do início do oráculo (decisão 15) — destrava o Wash com o filme; (2) a
**orla de 1 px** do Rebelle (o Liner e o Mop: nenhum knob a dá com o filme — `capillary`, evaporação, `leveling`,
`extEdgeTint`, `retention`: orla 1,00–1,04 contra 1,07–1,28); (3) a pegada varrida sob o filme (o Flat fino); (4) o
cruzamento molhado (o azul arrasta o vermelho; contas) — do `oleo_acrilica`; (5) o recto de 5 px do Mop.

## 3.ª ronda (J4, 21/10) — a decisão 15 na lei, o cruzamento molhado, o fio da borda

Bancada `PH2D_EXAME_DIR=/tmp/ph2d_exame_aguarela3`, variantes num processo; réguas novas (scratchpad `aguarela3/`):
`cruz.py` (o vermelho que o azul LEVA: R−B do vertical abaixo da faixa molhada menos acima dela), `cor.py` (as cores do
cruzamento molhado/seco), `liso.py` (orla/miolo). Load 8–260 (o relógio não vale; nenhum custo novo medido).

**1. A decisão 15 na LEI** (`leis_sistemicas::a_pressao_de_cada_preset_e_a_da_sua_origem`, `399a387c6d`): abaixo do 1.º
decil em que o oráculo PINTA (largura > 0 na foto da rampa) o decil conta no RMS com erro 0. ⛔ Medido e recusado: TIRAR
o decil do RMS — infla quem casava os zeros (Charcoal 0,179 → 0,192, Gouache Flat 0,176 → 0,186: reprovavam). Com o
denominador da rampa inteira a régua nova nunca lê mais longe que a de antes: os 25 de fábrica todos ≤ (Wet Wash 0,179 →
0,143, Wet Flat 0,170 → 0,115, Express Oil 0,168 → 0,113…; os que começam no decil 0 iguais). CONTROLO novo no portão (o
Wet Wash com a largura torta acima do início, `pressureSize` 1 em curva 3): 0,533. Mutações: o corte de TUDO (d0 = 10) →
o controlo 0,000, VERMELHO; o controlo sem a rampa torta → 0,143, VERMELHO. Com ela: Flat Bristle 0,202 → **0,130**, o
Wash de filme 0,28–0,39 → **0,12** (passam). `pressao_contra_a_origem` fica `pub(crate)`.

**2. O cruzamento molhado — NÃO era motor.** Medido no Wet Liner (Rebelle `Liner`): R−B do vertical abaixo da faixa
molhada **+33** (o nosso) contra **−3,2** (Rebelle): a ponta apanhava o vermelho (`Knob::Pickup` 0,005 por janela; a ponta
limpa-se a 0,001) e levava-o pelo vertical inteiro — o «azul que arrasta o vermelho» dos revisores. `pickup` 0 → −4,2 e a
régua 375 → 391/456; `tipClean` 0,05 dá o mesmo. No Mop a lente lilás PÁLIDA era o escoamento da tinta nova para a molhada
de outro traço: a parede da J2 (`levelingOwnStroke` 1) dá a cor do cruzamento [183,170,194] contra o Rebelle [182,176,200]
(sem ela [206,195,211]); régua 375 → 393. O seco: `glaze` 0,2 leva o violeta-cinza [86,116,179] ao azul-marinho [77,105,160]
(Rebelle [78,96,155]); 0,5 já é roxo escuro. ⛔ O que FICA aberto (motor, devolvido): o Rebelle `Wash`/`Mop` EMPURRA o
vermelho molhado para os lados (empurra 1,16) e o azul abre uma mancha REDONDA escura; o nosso, com a parede, um quadrado
limpo, sem ela uma lente pálida — a mistura subtractiva de duas tintas na MESMA água que se espalha (o recurso «molhado ×
molhado»). Portão `familias::aguarela::portoes::o_azul_nao_leva_o_vermelho_molhado` (Liner 1,1; controlo = a ponta que se
suja 36,1 — a mutação do valor).

**3. O fio escuro da borda — NÃO era motor.** A orla nasce na borda que SECA antes do miolo (`drying.rs`, a evaporação
pela borda) e o pigmento corre para ela. Sonda (`sonda_o_miolo_liso`, `PH2D_SONDA_PERFIL`, agora com a água): com o filme
a célula tem **3,5** de água (sem ele ~1) e aos 240 quadros ainda 3,0 — nada secou, a orla 1,00–1,04; sem o filme a borda
seca (água 0) e junta 272 contra 155 do miolo. Água por dab 1 → 0,3 · 0,15 · 0,1: orla a 40 px 1,02 → 1,06 · 1,18 · 1,20
(Rebelle 1,25); 0,1 mancha (0,025). Fica 0,15 + capilaridade 0,05 (mancha 0,005): régua 417. Portão
`o_liner_tem_o_fio_escuro_da_borda` (1,195; controlo com a água de omissão 1,012). No **Wash** o fio é ERRADO (o revisor:
«contorno duro»; o Rebelle esbate): o Wash fica sem a água pouca, orla 100. No **Mop** a água pouca não deu orla (0,97–1,04).

**Os pincéis** (seis condições; revisores cegos NOVOS, pranchas `pranchas/aguarela/*_a3*.png`, `cegas/`, `chaves/`):
- **Wet Liner** — receita: filme + `pickup` 0 + parede + vidrado 0,2 + água por dab 0,15 + capilaridade 0,05. Régua 375 →
  **419/456**; vizinho Soft Pastel **0,705** (≥ 0,596). Revisor q3: achou o Rebelle 6/6 (baixa/média) e lista: a 200 px as
  pontas do Rebelle esbatem em degradê e o miolo é mais carregado (o nosso chapado — repetido do t9 ⇒ visível), a espiral
  dele tem costuras ténues entre as voltas, a rampa dele esmaece no bico, os nossos cruzamentos azul × azul mais escuros e o
  cruzamento molhado dele um bloco escuro. Já NÃO lista o fio nem o arrasto. ⇒ **CURADO, NÃO APROVADO**.
- **Wet Wash** (com o filme) — dura 0,9, papel 0, pigmento 2,8, orla 100, largura fixa, `pickup` 0, vidrado 0,2: régua 349 →
  **373/456** no exame do preset regerado (variantes sobre o de orla 200 — orla 20/50/100/200: 373/369/385/375); rampa 0,12. Revisor q4 (a versão com a água pouca, 375): achou o Rebelle
  6/6 (média): o nosso contorno escuro (saiu: orla 100 sem a água pouca), as pontas duras a 200 px, o vermelho molhado que o
  Rebelle levanta e empurra, os azul × azul mais escuros, a entrada tardia (decisão 9). ⇒ **CURADO, NÃO APROVADO**; fica o
  filme (o miolo mosqueado e a orla oca dos revisores r1/r2 já não aparecem).
- **Mop** (em estudo; o preset de estudo pela receita com a tabela da largura MEDIDA, `familias/aguarela/estudo.rs` — a de um
  ficheiro base mentia: 62 px a 40) — `pickup` 0 + parede + vidrado: régua 405 → **390/456**. Revisor q5: tomou o NOSSO pelo
  original nos gestos (baixa), achou o Rebelle nos tamanhos e no «sobre» (média): as pontas do 200 px, a mancha redonda que
  o azul abre no vermelho molhado (o nosso um quadrado), a espiral com anéis. ⇒ **REPROVADO ainda**.
- **Flat Bristle** (`fixtures/presets_teste/`, receita intacta) — régua **368/456**, 1.º dos 228 do Rebelle (0,838), vizinho
  Oil Bristle 1,078, a rampa 0,130 (passa). Revisor q1 (novo): o Rebelle 6/6 (baixa) e REPETE o o2c: o vermelho opaco (K–M,
  não do preset), os nós escuros onde o azul cruza o azul, as pontas desfiadas contra as redondas, a estria fina regular ⇒
  repetida por dois revisores (decisão 12) ⇒ **CURADO, NÃO APROVADO** (em estudo).
- **Wet Round** pigmento 125 (estudo, não aplicado): régua 297 → **419/456**, vizinho Gouache Round 0,924. Revisor q2: tomou o
  VIZINHO pelo original e o NOSSO pela imitação 6/6 (média), o Rebelle «claramente outro» — a cura aproxima-o da guache, não
  do Rebelle (a 200 px uma nuvem de centro escuro e halo contra a faixa chapada riscada dele; 5 px contínuo contra o
  pontilhado). ⇒ **não passa: fica como está**; as `MIDIAS` das famílias não mudam.

Portões tocados (por nome, verdes): `leis_sistemicas` (3), `cada_preset_parece_a_sua_midia`, `…_e_a_sua_receita`,
`…_volta_ao_byte`, `a_miniatura_…`, `a_largura_e_a_prometida`, `a_regra_do_spacing_…`, `the_factory_presets_are_counted`,
`familias::aguarela` (7). `a_lei_da_tinta`: as cenas do Wash e do Liner (sem e com as caixas) repinadas; o
`o_glaze_do_pincel_escurece_o_cruzamento` fixa no seu Liner a água, a capilaridade, a sujidade e a parede de omissão (a régua
do Glaze; com as do Liner novo o «sem Glaze» lia 0,617). Os motores do filme (`portoes.rs`) fixam o mesmo (a régua do motor).

**Para a fila**: (1) motor — duas tintas na MESMA água: o azul abre uma mancha redonda e escura e empurra o vermelho
(Wash, Mop, o bloco do Liner; e os nós azul × azul do Flat Bristle); (2) motor — a 200 px a lavagem do Rebelle esbate as
pontas em degradê e carrega o miolo (Liner, Wash, Mop: o filme é chapado); (3) as costuras das voltas da espiral do Liner;
(4) o recto de 5 px do Mop e o Sumi-e: não feitos.
