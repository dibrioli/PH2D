# Exame da família `mistura` (J4, catálogo L — borradores, misturadores, borrachas)

Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/mistura.rs` (`MEIOS` = a fábrica, `EM_ESTUDO` = os
reprovados com receita e portão prontos). Pranchas: `pranchas/mistura/` (cegas e chaves em `cegas/`, `chaves/`).
Régua: `exames/<slug>_regua.txt`. Identidade por foto: `exames/mistura_foto.py` (as contas do `Identidade::de`).

**Aprovados: Kneaded Eraser (L-8) · Hard Eraser (L-9)** (Digital, `Eraser/Soft|Hard`) · **Blender Soft (L-1)** (Wet,
ferramenta Blend, `WetPainter/Blend/Soft`). **Reprovados:** Smudge Finger (L-3) · Blur (L-5) (em estudo, com receita) ·
Blender Grainy (L-2) · Dry/Lift (L-7) (motor feito, exame por fazer). **Curado, não aprovado:** Water Brush (L-6). **Fundido:** Smudge Spiky (L-4) → Smudge Finger.

## A bancada (`ecdb50bd6e`)

A foto é a camada SOBRE o papel; a ferramenta que não pinta leva as faixas da base do seu meio; o Rebelle refeito com
`PH2D_REBELLE_REPETE_TROCA=1` (`fixtures/rebelle_mistura/README.md`).

## A fábrica das FERRAMENTAS (2.ª onda, `bcb4e5425d`; o detalhe no commit)

`ReceitaFerramenta`/`FERRAMENTAS_WET` · a largura de uma ferramenta = a DIFERENÇA sobre a faixa da base
(`base_da_ferramenta`: Wet Round para Wet/Dry, Ink Brush o resto) · a miniatura sobre uma faixa vermelha · o grupo
«ferramenta» da matriz (`matriz/escala_ferramenta.json`) · `PH2D_EXAME_ESCALA_BASE`.

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

## Motor J4 (`8b80f021cc`, `1257a6e547`; portões `familias::mistura::motor::*`, verdes; neutro = ao bit)

- **L-6 Water Brush — `Knob::WaterPush`** (`tools::apply_wet_push`): a água leva `push·w` do pigmento SUSPENSO de cada
  célula molhada para a frente da água de cada lado (≤ 6,6 r, linha a linha, pela normal; `w = e^{−(|s|−r)/2r}` fora da
  pegada; 15 % numa rampa de 1 r antes da frente, 85 % pelo caminho); sobre tinta SECA arrasta pelo traço `0,5·push` do
  que o `wetLift` dissolveu (o escorrido). Bancada (escala 0,56 = 38 px, base Wet Round): eixo 0,11 × Rebelle 0,13;
  ±30/40 px 0,37/0,63 × 0,34/0,50; as linhas a x 380 · 620 × 380 · 630. Mutações que sangram: empurrão desligado ·
  frente sem «passa da pegada» · tudo pelo caminho.
  Preset de exame (fora da fábrica): o Wet Round modo **Wet** + `waterPush 0,87`, `wetLift 0,1`. Régua 36/68 (os ✗: a A
  do Rebelle é a água a CIANO, e as faixas da base). Vizinho Kneaded Eraser d 2,656. Prancha `pranchas/mistura/water_brush.png`.
  Revisor cego r1 (sonnet, baixa): não achou o oráculo — juntou o nosso ao vizinho e pôs o Rebelle de «terceiro»: *«mais
  largo e mais vermelho … "dentes" arredondados nos cruzamentos de baixo»*. **CURADO, NÃO APROVADO** (regra de custo:
  sem 2.ª ronda): o resto é a BASE (as faixas do Wet Round 33 px e pálidas × 44–49 do Watercolor Round do Rebelle) e o
  escorrido na faixa seca mais fraco que o «U» do Rebelle (a `wetLift 0,2` ele cresce; por medir na prancha).
- **L-2 Blender Grainy — o Grain na máscara do Blend** (`Trail::accumulate_blend_grao`, a porta da lane e o `dab_route`):
  com o Grain armado ele substitui as cerdas na máscara; sem Grain, a porta antiga ao bit. As seis texturas dão agora
  cinco imagens. Portão `o_grao_modula_o_misturador` (752 · 801 células; mutação «grão ignorado» 0 · 0). Exame NÃO feito
  (ordem de custo): melhor variante medida = Blender Soft + Noise 12 + `intensity 0,2` à escala 1,0 (manchas como o
  Rebelle «Textured», mas mais estreito: ~70 × ~95 px, e o rasto abaixo mais longo).
- **L-7 Dry/Lift — `Knob::DryerEdge`** (`tools::apply_dry_edge`): antes de selar, nivela a tinta suspensa (caixa 0,2 r,
  onde a maioria tem pigmento) e puxa para dentro da borda a faixa molhada de 0,15 r de cada lado. Portão
  `o_secador_assenta_e_deixa_orla` (0,5: dentro 1,225 · fora 0,738 · desvio 0,555 → 0,272). Bancada: a linha por dentro
  1,2–1,3 e o claro 0,56–0,78 (Rebelle 1,37 · 0,84), mas a mancha sai ESTREITA: a faixa do Wet Round nasce com ~28 px e
  alarga para 60 a secar; o secador congela-a fresca. Exame NÃO feito; o que falta é da aguarela (a faixa a nascer larga).
- **O vermelho que cobre mais que o azul** (medido, não curado): cobertura em RGB linear na mesma escova (vermelho seco
  y 470 · azul no papel y 310): Rebelle Round Hard 2 0,766 · 0,752 (azul/vermelho 0,98); nosso Wet Round 0,416 · 0,394
  (0,95), Gouache Flat 0,888 · 0,817 (0,92), Ink Brush 0,988 · 0,980 (0,99). Em sRGB a razão é ~1 (Wet Round 1,10 ×
  Rebelle 1,12). ⇒ não é K–M do pigmento: é o alpha-over do render feito em sRGB (o canal que vai a ~0 cai mais depressa
  em luz linear) e só se vê na tinta rala. A cura (compor em linear / o `km_glaze`) muda TODOS os presets — não aplicada.

## Custo (`exame.sh --custo`, smoke, load 10–27)

p95 a 5 · 40 · 200 px: Blender Soft 0,72 · 1,23 · 7,30 ms (RSS ≤ 97 MiB) · Kneaded 0,14 · 0,27 · 1,07 · Hard 0,15 · 0,94 · 4,74.

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
