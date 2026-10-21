# Exame da família `mistura` (J4, catálogo L — borradores, misturadores, borrachas)

Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/mistura.rs` (`MEIOS` = a fábrica, `EM_ESTUDO` = os
reprovados com receita e portão prontos). Pranchas: `pranchas/mistura/` (cegas e chaves em `cegas/`, `chaves/`).
Régua: `exames/<slug>_regua.txt`. Identidade por foto: `exames/mistura_foto.py` (as contas do `Identidade::de`).

**Aprovados: Kneaded Eraser (L-8) · Hard Eraser (L-9)** (Digital, `Eraser/Soft|Hard`) · **Blender Soft (L-1)** (Wet,
ferramenta Blend, `WetPainter/Blend/Soft`). **Reprovados:** Smudge Finger (L-3) · Blur (L-5) (em estudo, com receita) ·
Blender Grainy (L-2) · Water Brush (L-6) · Dry/Lift (L-7) (motor). **Fundido:** Smudge Spiky (L-4) → Smudge Finger.

## A bancada (`ecdb50bd6e`)

A foto é a camada SOBRE o papel (a borracha só mexe no alfa; com alfa 255 sai ao byte) · a ferramenta que não pinta
(`Pincel::nao_pinta`) leva as faixas «sobre» da base do seu meio, também na biblioteca · `exame.sh` corre na árvore
dele e usa a escala 1 sem a largura do recto do oráculo · o Rebelle pintava o 1.º traço depois de TROCAR de ferramenta
ao tamanho da base (`cruza_1` do Smudge 68 × 15 px): `PH2D_REBELLE_REPETE_TROCA=1` e 10 fotos refeitas em
`fixtures/rebelle_mistura/` (README com o cabeçalho).

## A fábrica das FERRAMENTAS (2.ª onda, autorizada pelo coordenador; aditiva)

1. `ReceitaFerramenta { receita, modo }` + `familias::FERRAMENTAS_WET` (a lista ao lado do `Familia`; a família declara
   `WET_FERRAMENTAS`): o gerador, a contagem e `receitas()` lêem-nas; os literais `Receita` das outras não mudam.
2. **A largura de uma ferramenta** (`presets_de_fabrica::largura_da_ferramenta`): a DIFERENÇA da tela com e sem o
   traço, sobre uma faixa da base (`receitas_de_fabrica::base_da_ferramenta`: Wet Round para Wet/Dry, Ink Brush para o
   resto — a Acrylic riscava a faixa, o Marker Bullet tem tecto de 101 px) com a orla de baixo na linha do traço; a
   meia altura (a ¼ a franja de um misturador dobrava num degrau: 36 → 75 px); a cena é a do raio do MOTOR nas duas
   leis (com a base a acompanhar o pedido, a tabela e a promessa mediam 52 × 48 px). Os pincéis que pintam: a lei de antes.
3. **A miniatura de uma ferramenta** (`wetpaint_preset/preview.rs::faixa_da_ferramenta`): uma faixa vermelha cheia por
   baixo do S (só quando `EraseAlpha` ou a ferramenta molhada ≠ Paint) — as borrachas saíam EM BRANCO. Os outros ao byte.
4. **A matriz: o grupo «ferramenta»** (`ferramentas/matriz_ferramenta.py`, escala própria `matriz/escala_ferramenta.json`:
   1,4826·MAD sobre 65 fotos de ferramentas do Rebelle): resto · largura · orla · arrasto abaixo e acima · orla vertical
   nos cruzamentos 2 e 3, A e B. Só no vector de uma ferramenta (`"ferramenta": true` no `meta.json`): a distância
   entre pincéis que pintam não muda (a escala congelada também não). A Blender Soft: Kneaded Eraser a 2,88.
5. Bancada: `PH2D_EXAME_ESCALA_BASE` (as faixas da base fora da escala da ferramenta) e a base pelo modo.

## L-9 Hard Eraser — APROVADO

- Receita (Digital): `EraseAlpha`, `hardness 0,9` + `borda_px 1` (a dureza 1 dava a orla 0 — corte binário; a 0,9
  dá 2 px como o Rebelle), `dab_quina 1` (ponta quadrada do Square Hard), cápsula; a pressão leva a força (o
  `pressure_opacity_influence` 1 do preset dele), o tamanho não. Sem tamanho de fábrica (o do artista).
- Oráculo Rebelle `ERASER/Default/Square Hard` (refeito; o Krita `Eraser Circle` dá os mesmos 39 px · orla 2).
  Identidade (resto · largura · orla): 0 · 39 · 2,0 × nosso 0 · 40 · 2,0. Régua **48/56** (os ✗: as faixas).
- Razão de uso: recortar e corrigir — tira a tinta toda, borda limpa de 1–2 px, ponta quadrada.
- Revisão cega r1 (baixa, só pelas faixas): *«o apagar em si (largura do buraco, borda nítida, corte até ao papel)
  parece igual nos dois»*; imitação = o nosso; o terceiro claramente outro.

## L-8 Kneaded Eraser — APROVADO

- Receita (Digital): `EraseAlpha`, `hardness 0` queda `Sharp` (o disco macio do Soft Round), Strength **0,26** (uma
  passagem deixa 0,71 da tinta; o Rebelle 0,72); a pressão leva a força.
- Oráculo Rebelle `ERASER/Default/Round Soft` (refeito). Identidade: Rebelle 0,72 · 31 · 14,5; nosso 0,71 · 30 · 13.
  O perfil na faixa casa coluna a coluna (2·3·6·9·15·21·28…72 contra 0·2·6·10·16·23·30…74).
- Régua **72/80** (os ✗: as faixas).
- Razão de uso: levantar luz numa camada pintada — clareia em parte, sem buraco, borda larga.
- Revisão cega r1 e r2 (média, pelas faixas): imitação = o nosso; repetido *«marca levemente torta (cauda ~1 px à
  direita)»* e *«14 × 15 px»* — medido: o perfil do Rebelle é o nosso deslocado ~1 px (o registo de meio píxel da
  bancada, J3) e a largura difere 1 px; o resto é a faixa (o substrato).

## L-1 Blender Soft — APROVADO

- Receita (Wet, `ReceitaFerramenta` modo **Blend**, nasce do Wet Round): `felt 0,2` (sem o feltro a mistura sai em
  riscas de cerdas), `blendForce 0,5`, cerdas 4000·2. Base Ink Brush; à largura do Rebelle (escala 0,45: o «40» do
  Blend dele mistura ~33 px, o nosso 37). Oráculo `rebelle_mistura/BLEND/default_soft` (refeito, A = B).
- O vermelho levado (verde mínimo a y 100·110·115 · 185·190·200·210): 138·56·36 · 125·139·177·207 contra o Rebelle
  131·78·48 · 107·130·183·217; alcance acima · abaixo 24 · 21 px contra 26 · 23. Régua 116/142 (os ✗: as faixas).
- Matriz: Kneaded Eraser 2,88 (grupo ferramenta; ≥ 0,596). Razão de uso: fundir as cores onde passa sem as levar longe.
- Revisão cega: r1 (escala 0,4, baixa) achou o Rebelle; r2 (média) escolheu o NOSSO como original nas 2 linhas;
  repetido por r1 e r2 *«o original sobe mais acima da faixa»* (19 × 26 px) ⇒ curado (escala 0,45: 24). **r3** (nova
  prancha, média): escolheu de novo o NOSSO como original nas 2 linhas (*«grão, fim rasgado, arraste com bordas
  escuras, como tinta real a ser empurrada»*); o resto que listou é a faixa da base (riscas × grão) e o nosso rasto 4 px
  mais largo. Custo p95 0,72 · 1,23 · 7,30 ms, RSS ≤ 97 MiB (load 19–21). Pranchas `blender_soft{,_r3}.png`.

## L-3 Smudge Finger — REPROVADO (motor: o fio do arrasto)

- A composição já o exprime: o modo **Smear** do Digital é um campo de deslocamento composto (`smear_field.rs`), a
  mesma lei do Smudge do Rebelle. Receita: `hardness 0`, queda **Smooth**, Strength 1 (a 0,8–0,95 a corrente não
  relaia: o arrasto 119 → 47 px). V no cruzamento (y 130·140·160): 22·18·12 contra o Rebelle 21·15·9; arrasto abaixo
  da faixa 119 = 119 px. Régua 94/146. Vizinho Palette Knife d 3,96 (≥ 0,596).
- Revisão cega r1 (média): **o fio** — *«no ORIGINAL a cauda segue como um fio contínuo até à faixa de baixo; na
  IMITAÇÃO vira pontinhos soltos»*.
- Medido: o Rebelle leva o fio 130 linhas abaixo do «papel», o nosso 95 e partido; nenhuma queda (8), Spacing nem
  Strength o junta — o campo re-amostrado a cada pingo perde o traço sub-píxel → **motor** (fila da J5).
- L-4 Smudge Spiky: o Spikey do Rebelle é este dedo com a pressão no tamanho (influence 1) e a força 0,7 — mais fino,
  o mesmo arrasto: funde-se (decisão 13); volta com o Smudge Finger.

## L-5 Blur — REPROVADO (não se vê na bateria)

- Modo Blur do Digital, Strength 0,2: a orla da faixa abre 4 px, como o Krita `x) Filter Blur`; régua 80/80. O revisor
  leu o original e o nosso como «nada pintado» — falta um oráculo em que o desfoque se VEJA.

## L-2 · L-6 · L-7 — REPROVADOS (motor; a fábrica já os aceitaria)

- **L-2 Blender Grainy** (Rebelle `BLEND/Default/Textured`, refeito: manchas granuladas): o Blend do Wet IGNORA o grão
  — 6 texturas (Noise 30/60, Voronoi, Dots, Clouds, Musgrave) dão a MESMA imagem ao byte (md5 iguais). Falta o grão a
  modular a máscara do misturador (`trail` do Wet; os ficheiros estavam a ser editados por outra família).
- **L-6 Water Brush** (`WATER/Default/Soft`, refeito; a A é a água a ciano, compara-se a B): o Rebelle ABRE branco na
  faixa molhada (resto 0,14 numa mancha de 114 px, com a orla escura onde a água pára). O nosso Wet não abre: resto
  0,87–1,00 com `wetLift` 1, `waterPerDab` 4, `waterCap` 30, `leveling` 2, `capillary` 1, `extBackrun` 2. Falta a água
  EMPURRAR o pigmento suspenso para fora da pegada (um knob novo em `tuning/defs.rs`, ocupado por outra família).
- **L-7 Dry/Lift** (`DRY/Default/Sponge`, refeito): o Rebelle deixa a mancha seca com uma ORLA nítida onde ela encontra a
  tinta ainda molhada; o nosso Dry congela a mancha com as riscas das cerdas e sem orla. Falta a orla do secador.

## Custo (`exame.sh --custo`, smoke)

p95 a 5 · 40 · 200 px: **Blender Soft 0,72 · 1,23 · 7,30 ms**, RSS 69–97 MiB (load 19–21) · **Kneaded Eraser 0,14 · 0,27 · 1,07 ms**, RSS 26 MiB (load 24–27) · **Hard Eraser 0,15 ·
0,94 · 4,74 ms**, RSS 26 MiB (load 10–14). Nenhum quadro > 16,7 ms. O relógio sob carga é indicativo (re-medir calmo
na J5). Rato e caneta: a pressão leva a FORÇA nas duas (a bateria «sobre» é à pressão 1; a rampa só se vê sobre tinta).

## Portões (verdes por nome)

- `familias::mistura::cada_ferramenta_da_mistura_se_reconhece_pela_foto` — as 5 (fábrica e em estudo) na cena «sobre»
  sobre a base do meio (a Blender Soft à escala 0,45): resto · largura · orla · arrasto · orla vertical, contra a foto e
  a lei dos IRMÃOS.
- `familias::mistura::o_portao_da_mistura_sangra_com_cada_mutacao` — a prova de mutação PERMANENTE: Smudge Strength
  0,8 (arrasto 47) · Smudge modo Paint · Kneaded Strength 1 (resto 0) · Hard dureza 0 (orla 8) · Blur Strength 1
  (orla 10) · Blender `blendForce` 0 · Blender modo Paint — as 7 sangram.
- Verdes por nome também: `a_largura_e_a_prometida` (Blender Soft 4·7·14·26·52·110·236,5·603 para 2·3,5·…·300 × 2),
  `the_factory_presets_are_counted`, `cada_preset_de_fabrica_e_a_sua_receita`,
  `cada_preset_de_fabrica_volta_ao_byte`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`,
  `familias::cada_familia_e_um_ficheiro`, `presets_dos_meios::a_contagem_por_meio`, `cada_pasta_de_fabrica_tem_rotulo`.
