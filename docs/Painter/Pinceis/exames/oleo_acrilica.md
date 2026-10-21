# Exame da família `oleo_acrilica` (J4, catálogo G — óleo e acrílica)

Receitas e portões: `tests/it/familias/oleo_acrilica.rs`. Pranchas `pranchas/oleo_acrilica/`
(cegas e chaves em `cegas/`, `chaves/`); réguas `exames/oleo_acrilica/<slug>_regua.txt`. Oráculo Rebelle
`fixtures/rebelle_bateria/OIL_AND_ACRYLIC` e `EXPRESS_OIL`; o ArtRage `OilBrush/ThickGloss` como 2.º do Thick. Tamanho de
fábrica = `volume_params.size` do preset do Rebelle (`NewFeaturesWetPaint/fixtures/rebelle_presets_281.json`). Variantes:
`PH2D_EXAME_VARIANTES` (15 lotes, ~90), régua `exame_regua.py` (✓/227, foto A). Custo a load 16–25: o relógio NÃO vale.

**Resultado: 1 aprovado (o Filbert, refeito)**; os pré-existentes Oil Bristle, Express Oil e Dabber **curados, não
aprovados**; a Acrylic Round **funde-se no Hard Round**; Flat Bristle, Palette Knife, Rake e Thick Impasto **reprovados** (em
estudo, a receita fica sem ficheiro); Dry Brush e Fan sem candidato; Wet Blend, Rigger e Nikko Rull não couberam. Motor:
nada mudou. Portão da família `cada_oleo_tem_a_sua_identidade` (com mutação) e `os_reprovados_do_oleo_nao_estao_na_fabrica`.

## G-1 Oil Bristle — CURADO, NÃO APROVADO
- Cura: o claro sorteado por fio 0,25 → **0,1**, o `emboss` 0,7 → **0,8** (a 0 o através a 200 px caía a 8,7 e o portão
  `o_relevo_nao_cresce_com_o_pincel` quer 11,5 ± 2; com 0,8 dá 10), o vão dos fios (`bristleSpan` 1 a 40 px).
- Régua 410 → **422/454**; origem 2.º → 2.º dos 228 (0,774 → 0,707); vizinho fora da família Flat Block-in 1,03.
- Revisão cega (4 revisores): r1 (antes) escolheu o NOSSO como original nas 6 linhas (média) — «mais luminoso e
  manchado» (curado); r2, r3, r4 acharam o Rebelle nas 6 (média/baixa) e **repetem a REGULARIDADE**: os nossos fios são
  um pente paralelo e contínuo, os do Rebelle aparecem e somem, há fios quase pretos soltos e o traço ondula. Tentado e
  medido (régua 206–211/227 em todos): `bristleWave` 0,3 (λ 200/500), `bristleOscillation` 1, `bristleSpan` 1,
  `bristleNoiseLength` 60, o claro 0–0,18 × emboss 0,5–0,9. Fica para a fila (o fio com carga que varia ao longo).
- **Os dois abertos da J2:** (a) a rampa 0→1 que esbate em fiapos claros — é a **decisão 9** (a mão do dono): o portão
  `a_pressao_de_cada_preset_e_a_da_sua_origem` dá mão 3,26 (≥ 2,5) a 0,063 da largura do Rebelle; a rampa do Rebelle
  (mão 1,30) violaria o R_MAO — divergência decidida. (b) o azul sobre o vermelho molhado **fechou**: vermelho em cima
  0,533 (Rebelle 0,598 ✓), cor (112,56,101) × (122,46,84) ✓ (na J2 0,346 e (88,82,143)).
- Custo p95 10,9 · 12,1 · 28,5 ms a 5/40/200 px, 80–100 MiB, load 21 (a J2 mediu 8,6 calmo a 200).

## G-2 Acrylic Round — FUNDIDA no Hard Round
Régua 172/454 contra o Rebelle `Round/Round Hard`, a origem **69.º** (1,854): a de hoje é estriada e translúcida, o Rebelle
um redondo opaco. A cura (feltro 0,5: 197/227) cai a **0,37 do Hard Round** (Digital) — o próprio Rebelle tem o `Round Hard`
do óleo a 0,46 do `Round Hard` digital. A acrílica fiel é o Hard Round ⇒ funde-se. ⚠️ Tirá-la mexe em ficheiros partilhados
(coordenador): `a_lei_da_tinta.rs`, `os_fios.rs`, `o_relevo_dos_fios.rs`, `sonda_spacing_continuo.rs`, `src/…/sonda_shape_casa.rs`,
`fixtures/leis_rebelle.json`, `exame_matriz.py`. Fica na fábrica até lá.

## G-3 Express Oil — CURADO, NÃO APROVADO
Cura: os TUFOS do `Round/Hairy` (cerdas 4 × 900 → **7 × 300**, faixa 2 → **4 px**, pigmento 60 → **75**). Régua 366 → 356
(a rampa e o 5 px), origem 77.º → 57.º. Revisores: r1, r2 (j4b) escolheram o NOSSO como original (6/6, 4/6, média — o Rebelle
mostra «contas de carimbo»); r3 (j4c, pigmento 90) achou o Rebelle (6/6, baixa). Repetido pelos três: o nosso mais
**translúcido sobre o vermelho** e as **pontas** desfiadas contra rombas. O pigmento 90–100 que o cobria parte a largura
prometida (raio 7: 17 px, pedido 14 ± 2) ⇒ 75, e a Tinta da foto do prompt 23 (0,772; dá 0,666) sai do `MIDIAS`. Custo p95
17 · 18 · 15 ms (load 22).

## G-4 Dabber — CURADO, NÃO APROVADO
Cura: o claro por fio 0,55 → **0,1**, o relevo 1,2 → **0,5**, o jitter 0,4 → **0,3**. Régua 368 → **388/454**, origem 12.º →
**8.º** (1,008 → 0,780). O revisor (alta) escolheu o NOSSO como original, mas as diferenças são grandes: o nosso rasga a borda
e risca, opaco a 200 px; o do Rebelle é um granulado fino, semitransparente. ⚠️ O Dabber do Rebelle está a **0,55 do
Graphite 8B** do próprio Rebelle: curado fiel ele cai ao lado do nosso 8B (decisão de produto para a J5: manter a esponja
riscada, ou fundir). MIDIAS: sai a Textura da foto do 52 (10,7; a bateria quer 6,6). Custo p95 3,7 · 9,1 · 24,5 ms (load 20).

## G-5 Filbert — APROVADO (refeito)
- Receita (Wet, `Oil/Flat/Filbert`, **64 px** = o do Rebelle): o Oil Bristle com a pegada REDONDA que segue o traço (Rebelle
  FOLLOW_TRAJECTORY: 40 px em qualquer direcção; `achatado` 0, quina 0), as faixas de cerdas de **4 px**, as pontas
  desfiadas em dedos (ρ **1,6**, o dobro do Oil Bristle), a tinta mais rala (pigmento **250**), o relevo 0,4, a mão que não
  levanta os fios (`bristleLift` 0; a largura quase não muda, a carga sim), a cápsula a **4 px** (o custo), sem Glaze.
- Oráculo Rebelle `Flat/Filbert`. Régua **376/454**; vizinho fora da família **Dabber 0,89** (Wet Liner 0,90, Soft Pastel
  0,95, Hard Round 0,98) — o «nada novo» do P53 (= a Acrylic Round) não volta: a Acrylic funde-se no Hard Round. Origem 38.º
  dos 228 (1,21; o Filbert do Rebelle está a 0,65 do `Metallic/Flat 3` dele).
- Revisão cega: **quatro revisores novos (r1 j4, r2 j4d, r3 j4e, r4 j4f) escolheram o NOSSO como o original nas 24 linhas**
  (média) e o Rebelle como a imitação. As diferenças que listam são do lado do Rebelle: «um pente de 10–15 faixas largas e
  regulares, chapado, de dentes iguais a 200 px», a rampa de largura constante (decisão 9), o cruzamento que lhes parece
  translúcido (medido: o do Rebelle COBRE, (28,99,175) sobre o vermelho × (24,102,179) no papel; o Glaze 0,4 e 0,1 que o
  imitava foram medidos e saíram — escureciam a (30,66,117) e baçavam a faixa seca). Pelo 05 §3 (o revisor toma o nosso pelo
  original) e a decisão 12, aprovado.
- Razão de uso: o pincel do retrato — cerdas que seguem a mão com a mesma largura em qualquer direcção e acabam em fios
  desiguais (o Oil Bristle é a lâmina fixa).
- Custo p95 5,9 ms a 40 e **14,5 ms a 200 px** (load 16; com a cápsula a 2 px dava 71 ms a load 22), RSS 87–103 MiB.
- Portão `cada_oleo_tem_a_sua_identidade`: de pé 1,00 (segue o traço), estria 0,036 (> 0,018; o Hard Round 0), rasgo 0,05
  (< 0,9; o Dabber 1,75). Mutação (Filbert ← Hard Round, e os quatro trocados pelos vizinhos): SANGRA nas 5 regras.
  `a_pressao_de_cada_preset_e_a_da_sua_origem`: largura a 0,174 da do Rebelle (≤ 0,18), mão 4,4.

## G-6 Flat Bristle — REPROVADO (em estudo)
A lâmina do Oil Bristle (Rebelle α 84°, 39 × 8,5 — a MESMA do Bristle Flat) com a tinta rala (pigmento 200, cobertura 0,67).
Régua 364/454, vizinho Oil Bristle 1,09 (o Rebelle tem Flat × Bristle Flat a 1,11). O revisor achou o Rebelle nas 6 (média):
a tinta rala do nosso sai em **grão sal-e-pimenta** (o fio a pouca carga deposita aos pontos) onde a do Rebelle é uma risca
lisa translúcida; o 5 px firme contra quase invisível. Medido e o grão fica: cápsula 0/2/4/8 px, feltro 0–0,2, ruído ao
longo 2000, força 1–2, `pigmentSpacing`. ⇒ motor (o fio translúcido contínuo).

## G-7 Palette Knife — REPROVADO (em estudo)
Refeita no eixo CERTO (Rebelle `Knife Oily` PEN_TILT α **172°**, 33 × 10 a 40 px; o P53 tinha 0°) e com a CARGA EM DEGRADÊ ao
longo da lâmina (forma `Gradient` no Digital, `offset`/`size` 0,5 = uma rampa só; a Wet não lê a forma, medido). O `Knife Oily` passou a ser o **1.º dos 228** (d 1,00–1,03).
Régua 314 → 328/454. Dois revisores acharam o Rebelle (baixa, média) e repetem: o dele mostra os CARIMBOS (escamas em arco a
200 px, contas a 5 px) e corta em bico diagonal; o nosso é um degradê liso de pontas rectas. Fluxo 0,15 → 0,11 (o r1 via-o
mais opaco).

## G-9 Rake — REPROVADO (em estudo)
O redondo que segue o traço com metade das faixas tiradas (`bristleRemove` 0,5). Régua 390/454 (Rake Sharp), vizinho Soft
Pastel 0,73, Filbert 0,83. r1 escolheu o nosso como original (baixa); r2 achou o Rebelle (média). Os dois: as pontas do Rebelle
cortadas a direito (as nossas desfiadas) e, a 200 px, ~15 riscas escuras espaçadas contra o nosso campo denso.

## G-10 Thick Impasto — REPROVADO (em estudo)
O primeiro preset do meio Impasto: pegada redonda sem dureza, jitter 0,1, relevo 0,3 GRANULOSO (grão `Clouds`, fonte `Grain`).
Régua 370/454 (Rebelle `Flat/Thick`), 166/227 (ArtRage `ThickGloss`); vizinho Soft Pastel 0,76. O revisor escolheu o NOSSO
como «a tinta grossa de verdade» nas 6 (média), mas o Rebelle é outro pincel (riscas e fiapos, pouco relevo) e o ArtRage
também (o tubo liso brilhante com estrias). Nenhum dos dois oráculos é o que o nosso Impasto pinta.

## Não aprovados sem preset (o candidato não chegou à prancha)
- **G-11 Dry Brush** (Rebelle `Flat/Flat Dry`): ≤ 157/227 em 20 variantes (gate do papel, `bristleRemove`, faixas, ruído ao
  longo, `extDryBrush`): os fios secos com falhas CONTÍNUAS ao longo do traço e bordas nítidas não saem.
- **G-8 Fan** (Rebelle `Dabber/Fan`, α 6°): ≤ 146/227: o salpico de pêlos curtos espalhados é partícula/dispersão
  (recurso da família `texturas`).
- **Não couberam** (por ordem): G-12 Wet Blend Round, G-14 Rigger (o `Liner` do Rebelle está a 0,66 do Hard Round:
  risco de «nada novo»), G-13 Nikko Rull (doc. Procreate).

## Motor
Nada mudou. Os recursos do Wet desta família (troca de pigmento entre DUAS tintas molhadas; entrada clara) não foram
feitos: nenhum pincel dela os pediu na medição (o azul sobre o vermelho molhado do Oil Bristle está ✓).
