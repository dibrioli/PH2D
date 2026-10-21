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
