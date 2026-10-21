# Exame da família `mistura` (J4, catálogo L — borradores, misturadores, borrachas)

Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/mistura.rs` (`MEIOS` = a fábrica, `EM_ESTUDO` = os
reprovados com receita e portão prontos). Pranchas: `pranchas/mistura/` (cegas e chaves em `cegas/`, `chaves/`).
Régua: `exames/<slug>_regua.txt`. Identidade por foto: `exames/mistura_foto.py` (as contas do `Identidade::de`).

**Aprovados: Kneaded Eraser (L-8) · Hard Eraser (L-9)** — Digital, pasta `Eraser/Soft|Hard`.
**Reprovados (em estudo): Smudge Finger (L-3) · Blur (L-5).** **Fundido:** Smudge Spiky (L-4) → Smudge Finger.
**Bloqueados pela FÁBRICA (não pelo motor):** Blender Soft (L-1) · Blender Grainy (L-2) · Water Brush (L-6) · Dry/Lift (L-7).

## A bancada (era a 1.ª coisa: estes pincéis só se vêem sobre tinta) — `ecdb50bd6e`

1. **A foto é a camada SOBRE o papel** (`exame_bancada.rs::escreve_ppm`): a borracha (`EraseAlpha`) só mexe no alfa e
   deixa o RGB — a foto de antes lia o apagado como tinta intacta. Com alfa 255 (todo pincel que pinta) sai ao byte.
2. **A base das faixas «sobre» de uma ferramenta que não pinta** (`Pincel::nao_pinta`: modo ≠ Paint ou `EraseAlpha`)
   é a tinta do meio dela (`base_padrao`: Hard Round no Digital, Acrylic Round nos outros) — na biblioteca e na
   matriz também; um pincel que pinta continua com as faixas dele (a cache e a matriz dos outros não mudam).
   `PH2D_EXAME_BASE` vale só para o candidato e o vizinho da prancha.
3. `exame.sh` corre o cargo NA árvore do script (chamado de outra pasta testava o `it` dessa outra árvore) e, com um
   oráculo sem a largura do recto (só «sobre», ou o recto vazio de uma ferramenta), usa a escala nominal 1.
4. **O Rebelle trocava o tamanho no 1.º traço depois de mudar de ferramenta**: nas fotos `rebelle_bateria/` o
   `cruza_1` do Smudge Soft tinha 68 px contra 15 no `cruza_2`, o Square Hard 75 × 39. `bateria_rebelle.py` com
   `PH2D_REBELLE_REPETE_TROCA=1` repete o SET_BRUSH na troca; 5 fotos refeitas em `fixtures/rebelle_mistura/`
   (README com o cabeçalho): 15·15·15, 39·39·39, 38·38·38.

⚠️ **A matriz não mede ferramentas** (d = NaN para as borrachas): o vector dela é de tinta POSTA, e o traço de uma
borracha é «vazio». Distinção pela construção (nenhum pincel da biblioteca tira tinta) e pelo portão dos irmãos.

## L-9 Hard Eraser — APROVADO

- Receita (Digital): `EraseAlpha`, `hardness 0,9` + `borda_px 1` (a dureza 1 dava a orla 0 — corte binário; a 0,9
  dá 2 px como o Rebelle), `dab_quina 1` (ponta quadrada do Square Hard), cápsula; a pressão leva a força (o
  `pressure_opacity_influence` 1 do preset dele), o tamanho não. Sem tamanho de fábrica (o do artista).
- Oráculo Rebelle `ERASER/Default/Square Hard` (refeito, `rebelle_mistura/`); 2.º: Krita `a) Eraser Circle` (39 px,
  orla 2 — o mesmo número). Identidade (resto · largura · orla): Rebelle 0 · 39 · 2,0; nosso 0 · 40 · 2,0.
- Régua **48/56**: os 8 ✗ são as FAIXAS (rugosidade/textura do óleo do Rebelle contra o Hard Round liso), não o corte.
- Prancha `pranchas/mistura/hard_eraser.png`. Vizinho: Acrylic Round (pinta azul — «claramente outro» para o revisor).
- Razão de uso: recortar e corrigir — tira a tinta toda, borda limpa de 1–2 px, ponta quadrada.
- Revisão cega r1: achou o oráculo com confiança **baixa** e só pelas faixas (*«o que difere é o vermelho por baixo,
  não o corte. O apagar em si (largura do buraco, borda nítida, corte que vai até ao papel, nenhuma mudança ao secar)
  parece igual nos dois»*); imitação = o nosso; o terceiro claramente outro. Cruzamento: sem diferença do pincel.

## L-8 Kneaded Eraser — APROVADO

- Receita (Digital): `EraseAlpha`, `hardness 0` queda `Sharp` (o disco macio do Soft Round), Strength **0,26** (uma
  passagem deixa 0,71 da tinta; o Rebelle 0,72); a pressão leva a força.
- Oráculo Rebelle `ERASER/Default/Round Soft` (refeito). Identidade: Rebelle 0,72 · 31 · 14,5; nosso 0,71 · 30 · 13.
  O perfil na faixa casa coluna a coluna (2·3·6·9·15·21·28…72 contra 0·2·6·10·16·23·30…74).
- Régua **72/80** (os ✗: as faixas). Prancha `pranchas/mistura/kneaded_eraser.png`.
- Razão de uso: levantar luz numa camada pintada — clareia em parte, sem buraco, borda larga.
- Revisão cega r1 e r2 (confiança média, os dois pelas faixas): a imitação = o nosso nas 4 linhas; o terceiro claramente
  outro. Repetidas pelos dois: *«a marca do original é levemente torta (cauda ~1 px à direita); a nossa simétrica»*
  e *«14 contra 15 px»*. Cruzamento medido: o perfil do Rebelle é simétrico a si mesmo deslocado ~1 px — o registo de
  meio píxel da bancada (J3, basicos §fila 3), não o pincel; a largura a 25 % difere 1 px (30 × 31). O resto que
  listaram é a faixa (riscas, fiapos, a orla mais macia) — o substrato, não a borracha.

## L-3 Smudge Finger — REPROVADO (motor: o fio do arrasto)

- A composição já o exprime: o modo **Smear** do Digital é um campo de deslocamento composto (`smear_field.rs`), a
  mesma lei do Smudge do Rebelle. Receita: `hardness 0`, queda **Smooth**, Strength 1 (a 0,8–0,95 a corrente não
  relaia: o arrasto 119 → 47 px). V no cruzamento (y 130·140·160): 22·18·12 contra o Rebelle 21·15·9; arrasto abaixo
  da faixa 119 = 119 px. Régua 94/146. Vizinho Palette Knife d 3,96 (≥ 0,596).
- Revisão cega r1 (confiança média, imitação = o nosso): além das faixas, **o fio**: *«no ORIGINAL a cauda segue como
  um fio vermelho fino e contínuo até tocar a faixa de baixo; na IMITAÇÃO morre logo abaixo da cúspide e vira
  pontinhos soltos… o azul horizontal deixa um rastro rosa liso; na IMITAÇÃO uma fileira de pontos espaçados»*.
- Medido (verde no eixo, x 470..530): o Rebelle leva o fio até y 450 (130 linhas abaixo do traço «papel», verde 105 a
  y 400); o nosso 95 linhas e partido; nenhuma queda (Linear, Sphere, Root, InvSquare, 4 curvas Custom), Spacing
  (0,01–0,1) nem Strength o junta. O campo re-amostrado a cada pingo perde o traço sub-píxel → **motor** (fila).
- L-4 Smudge Spiky: o Spikey do Rebelle é este dedo com a pressão no tamanho (influence 1) e a força 0,7 — mais fino,
  o mesmo arrasto: funde-se (decisão 13); volta com o Smudge Finger.

## L-5 Blur — REPROVADO (não se vê na bateria)

- Modo **Blur** do Digital, macio, Strength **0,2**: a orla de cima da faixa abre 4 px no eixo, como o Krita
  `x) Filter Blur` (0,4 → 6; 1 → 10). Régua 80/80; resto 1 (não tira tinta).
- Revisão cega r1: *«em cada linha, dois dos três pincéis não deixaram nenhuma marca»* — o original e o nosso lidos
  como «nada pintado». Um desfoque que a prancha não mostra não passa o P53. Falta um oráculo em que o desfoque se VEJA.

## L-1 · L-2 · L-6 · L-7 — a composição exprime-os; a FÁBRICA não os aceita

O Wet tem as ferramentas do Rebelle (`WetTool` Blend/Wet/Dry, no preset como `paint_mode`). Medido na bancada com
candidatos em ficheiro (`pranchas/mistura/mx_w_blend.png`, `mx_w_wet1.png`): o **Blend** sobre a acrílica arrasta e
esbate o vermelho para baixo nas duas faixas, A = B, como o `BLEND/Default/Soft` (régua 60/150 — o nosso a partir do
Wet Round é largo e com cerdas; afina-se com a ponta). O **Wet** (`wetLift` 1) sobre a aguarela do Wet Round só
clareia por igual; o Rebelle `WATER/Default/Soft` abre BRANCO na faixa molhada (resto 0,14 numa mancha de 114 px).
Bloqueios (ficheiros do coordenador): (1) a `Receita` do Wet não tem `modo` — o gerador não cria uma ferramenta;
(2) `a_largura_e_a_prometida`, `tabela_de_largura` e a miniatura pintam o traço no PAPEL branco — uma ferramenta que
não pinta mede 0 px e reprova. Ver o retorno.

## Custo (`exame.sh --custo`, smoke)

p95 a 5 · 40 · 200 px: **Kneaded Eraser 0,14 · 0,27 · 1,07 ms**, RSS 26 MiB (load 24–27) · **Hard Eraser 0,15 ·
0,94 · 4,74 ms**, RSS 26 MiB (load 10–14). Nenhum quadro > 16,7 ms. O relógio sob carga é indicativo (re-medir calmo
na J5). Rato e caneta: a pressão leva a FORÇA nas duas (a bateria «sobre» é à pressão 1; a rampa só se vê sobre tinta).

## Portões (verdes por nome)

- `familias::mistura::cada_ferramenta_da_mistura_se_reconhece_pela_foto` — as 4 receitas (fábrica e em estudo) na
  cena «sobre» sobre o Hard Round: resto · largura · orla · arrasto · orla vertical, contra a foto e a lei dos IRMÃOS.
- `familias::mistura::o_portao_da_mistura_sangra_com_cada_mutacao` — a prova de mutação PERMANENTE: Smudge Strength
  0,8 (arrasto 47) · Smudge modo Paint · Kneaded Strength 1 (resto 0) · Hard dureza 0 (orla 8) · Blur Strength 1
  (orla 10) — as 5 sangram.
- Verdes por nome também: `the_factory_presets_are_counted`, `cada_preset_de_fabrica_e_a_sua_receita`,
  `cada_preset_de_fabrica_volta_ao_byte`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`,
  `familias::cada_familia_e_um_ficheiro`, `presets_dos_meios::a_contagem_por_meio`, `cada_pasta_de_fabrica_tem_rotulo`.
