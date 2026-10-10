# 02 — Presets para o Wet Painter: como o Rebelle configura cada mídia, e os nossos

> **Fonte dos dados:** os **281 presets de fábrica** do Rebelle 8.3.4 (`Brushes/` + `Brushes Bristle/`), lidos
> por [`ferramentas/extrai_pinceis.py`](ferramentas/extrai_pinceis.py) — cada preset é um PNG (miniatura) com
> os parâmetros em *chunks* de texto (`BrushID`, `Name`, `Textures`, `Parameters` ≈ 14 KB de JSON,
> `ParametersEx`, `DefaultParameters[Ex]`) e as texturas embutidas (`Texture_N_0`, base64). As tabelas
> abaixo saem de [`schema.py`](ferramentas/schema.py) (as ~200 chaves e as suas faixas),
> [`por_meio.py`](ferramentas/por_meio.py) (as medianas por mídia) e [`curvas.py`](ferramentas/curvas.py). Os
> nomes de painel vêm do manual instalado (`Documents/manual/interface/panel-brush-creator/`).
>
> ⛔ **O que é DADO e o que é ASSET.** Os valores dos parâmetros são o que o painel *Brush Creator* mostra a
> qualquer utilizador — dado do produto, como os defaults de um `.blend` (método §2b). As imagens de
> forma/grão/papel embutidas são **assets** e **não se copiam**: os nossos presets usam as nossas texturas
> procedurais (`TextureSettings`) e imagens nossas. ⛔ A mistura por pigmentos do Rebelle é o **Mixbox,
> CC BY-NC** — a nossa continua a ser a Kubelka–Munk própria (`ph2d-pigment`).
>
> **Leitura em 1 minuto:** um pincel do Rebelle é **(forma × grão) sequenciados pela pressão/inclinação +
> uma lei de MISTURA por curvas de pressão + uma lei de PAPEL (força/contraste/arranhar) + volumes
> (tamanho, opacidade, água/oleosidade, pressão, comprimento)**. A mídia é dada pela **ferramenta** (14
> `BrushID`), não pelo preset: o mesmo esquema serve a todas, e cada mídia liga um subconjunto. É isso que
> o §4 traduz para o nosso `BrushSpec` e o §6 transforma em presets.

---

## §1 — O ficheiro de preset do Rebelle (o que contém)

| chunk | conteúdo | n (de 281) |
|---|---|---|
| `BrushID` | a FERRAMENTA (mídia): `AQUARELLE_PAINT_{WATERCOLOR,OIL,EXPRESS_OIL,PENCIL,PASTEL,INK,MARKER,AIRBRUSH}` · `AQUARELLE_{WET,DRY,BLEND,SMUDGE,ERASER,CLONE}` | 47·62·32·44·24·10·11·5 · 10·7·8·6·10·5 |
| `Name` | o nome no painel | 281 |
| `Textures` | mapa `Shape_i / Texture_i (grão) / PaperTexture_0 / StructureTexture_0 / DualBrushShape_0 → Texture_k` | 1–4 formas, 1–4 grãos |
| `Parameters` | o JSON do Brush Creator (§2) — `meta.version` 700…807 | 281 |
| `ParametersEx` | `volume_params {size px, opacity, water, pressure, length}` + `painting_params {enabled, paint_type, erase_mode}` | 281 |
| `DefaultParameters[Ex]` | a cópia de fábrica para *Reset Brush Changes* | 2 / 13 |
| `Texture_N_0` | PNG base64 da textura N (≤ 2048², usada a ≤ 1024²) | — |
| `asset_id`, `asset_type` | `rebelle_brush` (loja/partilha) | 281 |

A pasta diz o **grupo** do painel (`Brushes/<Tool>/<Grupo>/<Nome>.rebelle-brush.png`; `settings.ini` por
pasta guarda a ordem). Importar/exportar é um ficheiro só (*Import Brush Preset… from the image file*).

---

## §2 — O esquema (as chaves que importam, por secção do Brush Creator)

Faixas = o que os 281 presets usam (não o máximo do painel). `curve` = `{points[(x,y)…], fromParams}`;
`fromParams=true` significa «derivada do slider» (reta), `false` = curva editada à mão.

### 2.1 Stroke

| chave | painel | faixa | o que faz |
|---|---|---|---|
| `max_brush_size` | Max Size | 70…2048 px | o slider *Size* é % disto (não linear: `size_px ≈ max·(s/100)^1.5`, conferido em 5 presets) |
| `splat_spacing` (+`_jitter`) | Spacing | 0,01…1 (jitter 0…0,65) | fração do diâmetro entre carimbos |
| `splat_opacity` · `splat_opacity_multiplier` · `splat_opacity_jitter` | Opacity · Multiplier · Jitter | 0,1…1 · 1…50 · 0…1 | opacidade por carimbo; o **multiplicador** (até ×50 nos óleos/rake) é o que dá «corpo» sem saturar a cobertura |
| `splat_size_jitter` · `splat_rotation` · `splat_rotation_jitter` · `splat_scattering_jitter` · `scatter_mode` | Size Jitter · Angle · Angle Jitter · Scatter · Parallel/Perpendicular | 0…1 · 0/90 · 0…180° · 0…1 | |
| `splat_smudge` · `smudge_density` | Smudge · Smudge Density | bool · 0,9…9,9 | **esborrata os carimbos entre si** (esconde a «cadeia de contas»); 112/281 ligado — todos os óleos e metade das aquarelas |
| `smudged_lines_length` · `smudged_lines_pressure_curve` · `smudged_lines_speed_curve` | Smudge on Paper Bumps | 0/1 · curvas | riscos nos relevos do papel em função da **pressão** ou da **velocidade** (a única dinâmica de velocidade do produto) |
| `pressure_size_influence` · `pressure_size_curve` | Size Behavior | 0…1 · curva | tamanho × pressão |
| `pressure_opacity_influence` · `pressure_opacity_curve` | Opacity Behavior | 0…1 · curva | opacidade × pressão |
| `pressure_max` · `pressure_multiplier` | (derivados) | 0,25…1 · 0,8…2 | tecto e ganho da pressão lida |
| `tilt_size_multiplier` · `tip_tilt_curve` · `tip_tilt_width_curve` · `precise_tip_tilt` | Tip Tilt (+ editor width/height) | 1…5 · curvas · bool | **altura** (eixo do traço) e **largura** do carimbo em função da inclinação; desligado = `(0,42,0),(0,75,0)` |
| `position_smoothing_type/strength` · `pressure_smoothing_strength` · `pressure_sensitivity_softness/curve` | Line Smoothing (guardado no preset) | MOVING_AVERAGE/NONE · 0…4 · 0…9 · 3…5 | só 23 presets o gravam (tintas, óleos redondos, lápis) |
| `speed_size_*` · `speed_opacity_*` | — | **sempre neutros** (281/281) | existe a chave, nenhum preset a usa |
| `use_AA` | (antialias) | 194/281 | desligado nos lápis «ásperos» de propósito |
| `cursor_mode` · `cursor_threshold` | Cursor | AUTO/CIRCLE/RECTANGLE · 0…10 | |

### 2.2 Shape & Grain

| chave | painel | faixa | o que faz |
|---|---|---|---|
| `shape_params[i]` `{scale, scaleY, brightness, contrast, blend_mode=MULTIPLY, lockAspectRatio}` | Brush Shape (1–4) | scale 0,3…2 · bright −0,6…2 · contr −0,16…2 | a silhueta; `lockAspectRatio=false` + `scaleY≠scale` = carimbo achatado (flat/filbert) |
| `grain_params[i]` `{…, blend_mode}` | Brush Grain (1–4) | scale 0,15…2 · modos NORMAL/LIGHTEN/HARD_LIGHT/MULTIPLY/OVERLAY | o interior |
| `texture_order` · `texture_{0..3}_{start,end}` · `texture_random_order_ratio` | Texture Order · Image Sequence | SEQUENTIAL 93 · PRESSURE 92 · RANDOM 68 · TILT 28 · faixas 0…100 | **qual par forma/grão em cada carimbo**: por sequência, sorteio ponderado, **faixa de pressão** (com zonas de mistura entre pares) ou inclinação |
| `brush_rotation_mode` · `force_rotation_mode` | Rotation · Force | NONE 103 · FOLLOW_TRAJECTORY 97 · PEN_ROTATION 50 · PEN_TILT 31 | |
| `grain_tiles` · `grain_follow_brush_size` · `grain_follow_distance` · `grain_tile_follow_distance` · `grain_follow_shape_rotation/size` · `grain_random_start/splat(_angle)` · `grain_scale` · `grain_canvas_dpi_influence` · `grain_dpi` · `grain_brush_size` | Follow / Stretched / Tiled · DPI · Random Start/Stamp Offset · … | | o grão **preso à tela** (tiled, 101/281) ou **arrastado** pelo carimbo (follow/stretched); `grain_dpi` 100…300 escala o grão com o DPI do documento |
| `background_texture_params[0..1]` `{enabled, blend_mode, blend_weight, paper_strength(+curve), opacity_multiplier(+curve), random_start, random_offset, invert_texture, influenced_by_prerend, weight_curve, texture_params{scale,brightness,contrast}}` | Background Texture 1 / 2 | 86 ligados · modos NORMAL/HARD_MIX/HEIGHT/INV_MULTIPLY/MULTIPLY/OVERLAY | **vazios, pó, scumbling, dry-brush**: uma 2.ª textura que FURA o carimbo, por defeito nos vales do papel (`HEIGHT` nos óleos = relevo) |
| `dual_brush_*` (`enabled, alternative_mode, splat_count 1…15, splat_size 0,04…2,5(+jitter), opacity(+jitter), splat_spacing(+jitter), scattering_jitter, min_scatter, rotation_jitter, follow_trajectory, layer_blend_mode(+weight), size/opacity/scatter curves`) | Dual Brush | 11 ligados (lápis/pastel/tinta) | um **2.º pincel** que adiciona ou remove partes do 1.º (carvão, pó de pastel, pontas partidas) |
| `use_structure_textures` · `structure_texture_offset{,_mid,_start}` · `structure_texture_strength_curve` | Shape Border | 4 ligados (rake) | textura de **borda** que segue a trajectória/inclinação (Rake Sharp) |

### 2.3 Paint

| chave | painel | faixa | o que faz |
|---|---|---|---|
| `X.painting_params.paint_type` | Paint · Paint & Mix · Paint & Blend · Blend · Erase | PAINT_MIX 136 · PAINT 101 · PAINT_BLEND 4 · BLEND 2 | **o modo de fábrica**: todas as aquarelas/óleos/tintas em *Paint & Mix*; lápis/pastel/marcador/airbrush em *Paint* |
| `paint_mix_curve` · `paint_mix_curve_2` · `paint_mix_threshold` | Paint & Mix – Curve Editor · Paint Pressure Threshold | (0,65,0)→(1,1) · (0,0),(0,1,0,1),(0,5,0,9),(0,6,0,95) · 0,7…1 | 2 passos: **sobe a pressão = pinta**; quando a pressão **cai** mais do que o limiar (relativo ao MÁXIMO do traço) entra a mistura; a `_2` diz quanto tempo a cor carregada persiste |
| `paint_blend_curve` · `paint_blend_curve_2` · `blend_curve` | Paint & Blend · Blend | (0,4,0),(0,6,1) · … | modo contínuo: pouca pressão = blend, muita = paint |
| `maximum_blending` | (derivado) | 0,25…2 | tecto da mistura (óleos 0,25…2; aquarela 0,94…2) |
| `apply_loading_to_paint` | Loading | 168/281 | o slider *Loading/Opacity* é **tinta carregada** que se esgota ao longo do traço (com `volume_params.length`) |
| `use_glazing` (+`_lighten`, `_watercolor`) · `glaze_rgb_blend_mode` | Rendering Normal/Glaze · Paint Blending | 143/281 · NORMAL 274 + 7 modos (Marker/Effect) | **Glaze** = camadas transparentes limitadas pela opacidade; *Paint Blending* é o modo de blend da cor (Dodge/Burn/Hue… só nos marcadores de efeito) |
| `use_default_brush_color` · `default_brush_color` | Default Color | 22 (lápis) | o preset traz a cor (grafite `#686865`, carvão `#121212`) |
| `bristle_noise{h,l,s,enabled}` · `bristle_noise_stamp` · `bristle_noise_stroke` | Color Noise per Bristle / Stamp / Stroke | h ≤ 0,5 · l ≤ 0,7 · s ≤ 0,2 | ruído de cor em **HSLuv** por fio (altura do pincel), por carimbo, por traço |
| `impasto_strength` · `impasto_add` · `thick_impasto` | Max Impasto Height · Smudge · Thick | 0…1,8 · 0,5 · 2 | relevo por carimbo; **as aquarelas também têm 0,1…1** (textura de pigmento, não espessura) |
| `paper_structure_influence` (+`_curve`) | Paper Texture Strength | 0…1 + curva por pressão | quanto o relevo do papel fura a tinta — **cai com a pressão** em quase todos (ex. `(0,0,71),(0,71,0)`) |
| `paint_structure_influence` | Paper Texture Contrast | 0…1 | contraste da textura vs pressão |
| `paint_structure_scratch_mode` | Paper Texture Scratch | AUTO 235 · LIGHT 46 | a tinta «raspa» as partes claras ou escuras do papel (LIGHT = ferramentas utilitárias) |
| `legacy_paper_mode` | Legacy Paper Mode | 0/39 | sem height-map (compat R6) |
| `length_water_reduction` · `paint_fadeout_curve` · `paint_fadeout_(end_)length` | Length Oiliness Fadeout · (Length) | 0,22…0,32 · (0,1)→(1,0) · 1 / 0,25 | a água/óleo **acaba ao longo do traço** |
| `dont_limit_pickax` · `use_automatic_smudge_to_blend` · `smudge_to_blend_ratio` · `use_blend_instead_of_average` · `particle_color_smudge_freq` | Average Smudge Color · Use Automatic Value · Use Blend instead of Average | | só nos pincéis novos (39) |

### 2.4 Bristle Brushes (motor de partículas, `brush_mode=PARTICLE`, 15 presets) e `TEXTURE` (24)

| chave | painel | faixa |
|---|---|---|
| `bristles_random_span` (+`_swap`) · `bristles_oscillation` | Random Span · Oscillation | 0,1…2 · 5…10 · 0…12 px |
| `particle_pressure_mul` · `particle_pressure_contrast_jitter` · `particle_pressure_perlin_scale` · `particle_pressure_perlin_random_ratio` | Per-bristle Pressure (+Multiplier) · Noise Frequency · Random Noise | 1…2,2 · 1…5,5 · 0,75…2 · 0,2…0,75 |
| `particle_thin_{enabled,density,paint_struct,remove_impasto_strength,per_line_count,max_instances}` · `thin_{min_width,wave_length,wave_jitter}` | Thin: Density · Add/Remove Impasto · Wave Length · Jitter · Min Width | dens 0,19…2,5 · remove 0…1 · onda 500…700 |
| `particle_thick_{enabled,density,line_density,size,size_jitter,struct_multiplier,remove_impasto_strength,size_curve}` · `thick_{min_width,wave_length,wave_jitter}` | Thick: Density · Width (+Behavior, Jitter) · Quality · Wave · Min Width | size 5…100 px · onda 400…1000 |
| `particle_edge_{enabled,size,line_density,rising_portion,struct_multiplier,tilt_azimuth_strength,pressure_curve,tilt_curve}` · `edge_{min_width,wave_length,wave_jitter}` | Edges (Pressure + Tilt curves) | size 10…50 · azimute 85…90° |
| `particles_add_vs_resize` · `particle_impasto_pressure_curve` | Brush Form · Impasto Behavior | 0…1 · curva |

Os fios são de dois tipos — **Add impasto** (cristas) e **Remove impasto** (sulcos) — e cada fio tem a sua
própria resposta à pressão (Perlin + ruído), largura ondulante ao longo do traço e cor com ruído. É isto
que dá ao traço de óleo dele os **riscos paralelos que variam** sem textura repetida (fotografado em
[03](03_tracos_medidos_rebelle_x_ph2d.md)).

---

## §3 — Como cada mídia é configurada (as medianas dos presets de fábrica)

`T n/N` = ligado em n dos N presets · `a…m…b` = mínimo…mediana…máximo. Volumes (`X.volume`) em px / 0…1.

| | Watercolor (47) | Oil (62) | Express Oil (32) | Pencil (44) | Pastel (24) | Ink (10) | Marker (11) | Airbrush (5) |
|---|---|---|---|---|---|---|---|---|
| **modo de pintura** | PAINT_MIX 43 | PAINT_MIX 51 | PAINT_MIX 28 | PAINT 44 | PAINT 24 | PAINT_MIX 10 | PAINT 11 | PAINT 5 |
| **volume size (px)** | 19…93…409 | 19…78…283 | 40…75…178 | 4…36…326 | 14…36…248 | 4…17…42 | 2…248 | 172…248 |
| **volume opacity** | 0,15…0,6…1 | 0,1…1 | 0,35…1 | 0,2…1 | 0,31…1 | 1 | 0,25…0,4…1 | 1 |
| **volume water / oiliness** | 0…0,24…0,49 | 0,09…0,70…1 | 0,49…0,82…1 | — | — | 0,09…0,27 | — | — |
| `apply_loading_to_paint` | T15 | **T58** | T24 | **T39** | T17 | T2 | T8 | T5 |
| `use_glazing` | **T25** | **T50** | T9 | T5 | T2 | T8 | T3 | T0 |
| `splat_spacing` | 0,03…0,12…1 | 0,06…0,12…0,52 | 0,04…0,115…0,25 | 0,03…0,16…0,24 | 0,04…0,075…0,2 | 0,08…0,1…0,12 | 0,04…0,2 | 0,1…0,15…0,3 |
| `splat_opacity_multiplier` | 1…1…10 | **1…5,5…50** | 1…5…35 | 1…2…7 | 1…2…6 | 1…2…6 | 1…1…2 | 1 |
| `splat_smudge` | T22 | **T53** | T27 | T4 | T1 | T4 | T0 | T0 |
| `smudge_density` | 0,9…4…9 | 0,9…4…9 | 2…5…9 | 4…4…9,9 | 4…7,5 | 4…7 | 4 | 4 |
| `pressure_size_influence` | 0…0,6…1 | 0…0,45…1 | 0,31…0,95…1 | **0…1…1** | 0…0,55…1 | 0,4…1 | 0…0,25…0,85 | 0…0…0,8 |
| `pressure_opacity_influence` | 0…0,5…1 | 0…0…1 | 0…0…1 | **0…1…1** | **0…1…1** | 0…0…1 | 0,15…1 | 1 |
| `pressure_max` | 0,5…1 | 0,6…1 | 0,6…1 | 0,5…0,6…1 | 0,25…0,6…0,9 | 0,55…1 | 0,7…1 | 1 |
| `tilt_size_multiplier` | 1…1…3,4 | 1…1…4 | 1…1…3 | **1…3,8…5** | 1…1,6…4,2 | 1…1…4,2 | 1…1…1,6 | 1,76…1,8…5 |
| `texture_order` | PRESSURE 19 / SEQ / RANDOM | **PRESSURE 44** / TILT 13 | PRESSURE 18 / TILT 8 | **RANDOM 34** | RANDOM 14 / SEQ 8 | SEQ 6 / PRESSURE | SEQ 10 | PRESSURE 4 |
| `brush_rotation_mode` | NONE 17 / FOLLOW 15 / … | **FOLLOW 38** / PEN_ROT | FOLLOW 18 | NONE 24 / PEN_TILT 10 | FOLLOW 13 | NONE 5 | NONE 8 | NONE 5 |
| `impasto_strength` | 0,1…1 | 0,1…0,5…1,8 | 0…0,8…1,6 | 1 | 1 | 1 | 1 | 1 |
| `paper_structure_influence` | 0…0,53…1 | 0…0,7…1 | 0…0,7…1 | 0…0,7…0,9 | 0…0,65…0,9 | 0…0,5…0,9 | 0…0…0,6 | 0 |
| `paint_structure_influence` | 0…0,5…1 | 0…0,5…1 | 0…0,5…1 | 0…0,5…1 | 0…0,59…1 | 0,01…0,68…1 | 0…0…1 | 0,5 |
| `grain_tiles` (grão preso à tela) | T15 | T20 | T16 | T15 | T15 | T6 | **T11** | T1 |
| `grain_canvas_dpi_influence` | **T45** | **T59** | T26 | T9 | T11 | T2 | T3 | T0 |
| `bg0.enabled` (textura de fundo) | T14 (NORMAL/HARD_MIX) | **T25 (HEIGHT 32, HARD_MIX 23)** | T12 | **T25 (INV_MULTIPLY)** | T8 (OVERLAY) | T2 | T0 | T0 |
| `dual_brush_enabled` | 0 | 0 | 0 | **T6** | **T5** | 0 | 0 | 0 |
| `use_default_brush_color` | 0 | 0 | 0 | **T22** | 0 | 0 | 0 | 0 |
| `brush_mode` PARTICLE / TEXTURE | 5 / 0 | 10 / 5 | — | 0 / **19** | — | — | — | — |
| `use_AA` | T35 | T40 | T14 | T24 | T12 | T9 | T11 | T4 |
| formas × grãos por preset | 1…3 × 1…3 | 1…2 × 1…2 | 1…2 | **1…4 × 1…4** | 1…4 | 1…2 | 1 | 2 |

### O que faz cada mídia parecer o que é (lido da tabela e dos diffs por preset)

- **Aquarela.** `PAINT_MIX`, opacidade **0,5–0,7** (o *Glaze* liga em metade: camadas transparentes limitadas
  pela opacidade), água 0,1–0,5 (o *Wash Round* a 0,24 e spacing **0,03** — o carimbo denso é o que faz a
  aguada contínua), `impasto_strength` 1 (textura do pigmento, não espessura), `paper_structure_influence`
  0,5–0,6 com curva que **desce com a pressão** (`(0,0,71),(0,71,0)`: pouca pressão = o papel fura; pressão =
  cobre), `grain_canvas_dpi_influence` em 45/47 (o grão escala com o DPI do documento, não com o pincel),
  `grain_follow_brush_size` + `grain_follow_distance` no *Round* (o grão **anda** com o carimbo). O
  *Granulation* desliga o glaze, põe o grão em `HARD_LIGHT` com brilho −0,65/contraste 1 e `grain_random_splat`.
  Os *Bristle* (partículas): `apply_loading_to_paint`, `length_water_reduction` 0,32 (a água acaba), `blend_curve`
  `(0,0)→(1,0,9)`, `smudge_to_blend` automático. Gouache: `maximum_blending` 2, paper 1 com curva `(0,1),(0,7,0,3)`.
- **Óleo & Acrílico.** `PAINT_MIX`, `apply_loading_to_paint` em 58/62 (**a tinta acaba**), `splat_opacity_multiplier`
  **5,5** (até 50 no rake/metálico — corpo sem saturar), `splat_smudge` + `smudge_density` ~4 (esborrata entre
  carimbos → sem contas), `texture_order=PRESSURE` em 44 (a pressão escolhe entre pares forma/grão: pouca
  pressão = cerdas secas, muita = cheio), `FOLLOW_TRAJECTORY` em 38, `impasto_strength` 0,5 (óleo) e
  **`bg0.blend_mode=HEIGHT`** em 32 (a textura de fundo é **relevo** — o que o RealShader ilumina),
  oiliness 0,7. *Knife*: `tip_tilt_width_curve (0,0),(1,−5)` (a lâmina **estreita** com a inclinação),
  `PaperTexture_0` própria, `bg0 HARD_MIX` com `paper_strength` 0,33. *Rake*: `use_structure_textures` (borda),
  `splat_opacity_multiplier` 25, `texture_order=TILT`. *Bristle Flat* (partículas): `shape scale 0,7 × 1,1` sem lock,
  `smudge_to_blend_ratio` 1, `paint_mix_threshold` 0,85.
- **Express Oil.** O mesmo que o óleo com `impasto_strength` 0,8–1,6 (mais relevo), oiliness 0,82, opacidade
  0,35–1, `maximum_blending` 1,5, `pressure_size` ~0,95 (a pressão manda no tamanho), *Glaze* só no grupo *Glaze*
  (`impasto 0`, `pressure_multiplier 1,6`).
- **Lápis.** `PAINT`, `texture_order=RANDOM` com **4 pares** forma/grão (`texture_random_order_ratio` até 80),
  `pressure_opacity_influence` **1** e `pressure_max` **0,6** (nunca satura a 100 % — o grafite nunca é preto),
  `tilt_size_multiplier` **3,8** com `tip_tilt_curve (0,55,0),(0,7,0,97),(0,75,3,06)` (a partir de 55° de
  inclinação o lado do lápis cresce até 3×), `paper_structure_influence` 0,9 com curva em 3 pontos,
  `bg0 INV_MULTIPLY` (os vales do papel ficam brancos), `brush_mode=TEXTURE` nos *Graphite* (motor novo com
  fios finos: `particle_thin_density` 2,5), `use_default_brush_color` (cinza `#686865`), `use_AA=false` nos
  ásperos. *Charcoal*: `dual_brush` com **14 carimbos** a 0,3 de opacidade (o pó), `PEN_TILT`, bg `MULTIPLY` 0,72 +
  `HEIGHT`.
- **Pastel.** `PAINT`, `RANDOM` com 2–4 pares, `pressure_max` 0,55–0,6, `splat_spacing` **0,05**, `shape scaleY 2`
  (bastão), `dual_brush` em modo *stamp* (pó), `tilt` 1,6–4,2. *Oil Pastel*: bg `OVERLAY` 0,48 + `HEIGHT`, tilt
  `(0,6,0),(0,68,0,95),(0,74,3,63)`.
- **Tinta (Ink).** `PAINT_MIX` com `paint_mix_curve (1,0,8)` (quase só pinta), opacidade 1, água 0,1–0,5
  (*Dip Pen* 0,49: a tinta ESCORRE no papel molhado), `pressure_opacity_curve (0,0),(0,3,0,91),(1,1)` (sobe
  depressa — a pena marca ao toque), `pressure_smoothing` 5 + `MOVING_AVERAGE` (a linha é limpa),
  `splat_size_jitter` 0,35 no *Dip Pen* (irregularidade do aparo), `max_brush_size` 100. *Brush*: 2 pares por
  pressão (`texture_0_end` 22), `FOLLOW_TRAJECTORY`, `maximum_blending` 2, tilt 3.
- **Marcador.** `PAINT`, `splat_opacity` **0,3** com spacing **0,04** (a cobertura vem da sobreposição densa —
  é assim que marcador sobre marcador escurece), `grain_tiles` em 11/11 (o grão é o papel, preso à tela),
  `paper_structure` 0, `pressure_size` 0, `PEN_TILT` no *Chisel*. *Effect*: `glaze_rgb_blend_mode`
  Burn/Dodge/Hue/Saturation/Luminosity/Overlay/Color — o marcador como **filtro**.
- **Airbrush.** `PAINT`, `splat_opacity` 0,15, spacing 0,1, `pressure_opacity` 1 e `pressure_size` 0 (a
  pressão é só fluxo), `paper` 0, `tilt` 1,76 com curva `(0,75,0,95)` (o cone alonga-se), `use_glazing=false`.
- **Utilitários.** *Water* (WET): `water` 1, `PAINT` (só água, `only_water`), `pressure` 0,55. *Dry*: opacidade
  0,85, spacing 0,4 (absorve). *Blend/Smudge/Eraser/Clone*: `paint_structure_scratch_mode=LIGHT`, `use_glazing`
  em quase todos, spacing 0,01–0,1, `paper_structure` 0–0,5.

### As curvas que se repetem (a «gramática» das dinâmicas)

| curva | forma mais comum | leitura |
|---|---|---|
| `pressure_size_curve` | `(0,0)→(1,1)` 32 · `(0,0,6…0,7)→(1,1)` 24 · editadas 118 | tamanho mínimo de 60–70 % em metade dos óleos: o traço nunca «some» |
| `pressure_opacity_curve` | `(0,1),(1,1)` 55 (neutra) · `(0,0)→(1,1)` 49 · `(0,0),(0,35,0,11),(0,73,0,61)` 10 | lápis/pastel: côncava — meia pressão dá só 1/3 da opacidade |
| `tip_tilt_curve` | `(0,42,0),(0,75,0)` 171 (desligada) · `(0,42,0),(0,75,0,75…2,5)` | a inclinação só conta a partir de ~42° |
| `paper_structure_influence_curve` | `(0,0,5…0,8)→(1,0)` | o papel fura com pouca pressão e desaparece com muita |
| `paint_mix_curve` / `_2` | `(0,65,0)→(1,1)` / `(0,0),(0,1,0,1),(0,5,0,9),(0,6,0,95)` | a mistura entra quando a pressão **cai** (limiar `paint_mix_threshold` 0,7…1 do máximo do traço) |
| `smudged_lines_speed_curve` | `(0,0)→(1,1)` 146 · `(0,6,0),(1,0,49)` 16 | riscos no papel só acima de 60 % da velocidade |

---

## §4 — Tradução para o PH2D: o que o nosso `BrushSpec` já tem e o que falta

Legenda: ✅ existe (campo em `crates/ph2d-painter-brush/src/spec.rs`) · ◐ parcial · ❌ falta (é item do
plano em [01](01_o_que_o_rebelle_tem_e_nos_nao.md)).

| conceito do Rebelle | PH2D hoje | o que falta |
|---|---|---|
| Max Size + slider % | ✅ `radius_px` (0,5…4096) | nada (o preset guarda px) |
| Spacing (+jitter), Scatter (+count, parallel/perp.) | ✅ `spacing`, `jitter_spacing`, `jitter`, `spray_count` | «perpendicular» = jitter só no eixo normal ❌ |
| Size/Opacity/Angle jitter | ✅ `jitter_scale`, (opacity ❌), `jitter_rotate` | **opacity jitter** ❌ |
| Opacity × Multiplier | ✅ `strength` + `flow` | o multiplicador até ×50 **com `accumulate`** (hoje `flow` ≤ 1) ◐ |
| Pressure → size/opacity (curvas) | ✅ `dynamics.rs` (4 curvas) | `pressure_max` (tecto) ❌ · ganho ❌ |
| Tilt → altura **e largura** (duas curvas) + Precise | ◐ `tilt_size`/`tilt_opacity` | a **largura** (eixo normal ao traço) e a rotação do carimbo pela inclinação (`PEN_TILT`) ❌ |
| Rotation: None/Follow/PenTilt/PenRotation + Force | ◐ `shape.rake` (follow), `dab_angle_deg` | `PenRotation` (barrel) e `PenTilt` ❌ |
| Position/Pressure smoothing, Pulled String | ◐ `stabilizer`, `input_samples` | **pressure smoothing** ❌ · *pulled string* ❌ |
| Smudge entre carimbos (+density) | ❌ | **o esborrato entre carimbos** (a cura do «colar de contas»): doc 01 §4.2 |
| Smudge on Paper Bumps (pressão/velocidade) | ❌ | riscos no relevo do papel ❌ |
| Shape × Grain (1–4 pares) + Texture Order (seq/random/pressure/tilt) + faixas | ◐ `shape` + `texture` (1 par) | **até 4 pares** e a **sequência por pressão/inclinação** ❌ |
| Grain Follow / Stretched / Tiled · DPI | ◐ `texture.mapping` (tiling), `grain_depth` | *stretched* (segue devagar) ❌ · DPI ❌ |
| Background Texture 1/2 (vazios; modos HEIGHT/INV_MULTIPLY/HARD_MIX) | ❌ (há `stencil`) | **3.º slot «fundo»** que fura o carimbo nos vales/picos do papel ❌ |
| Dual Brush (stamp/stroke, count, size, opacity, spacing, scatter, curvas) | ❌ | **pincel secundário** ❌ |
| Shape Border (start/mid/end + curva) | ❌ | textura de borda ❌ |
| Paint & Mix / Paint & Blend / Blend (curvas + threshold da queda de pressão) | ◐ `wet_charge`, `wet_pull`, `wet_smudge` (aquarela) e o Smear | a **lei por curvas de pressão** e o **limiar da queda** para TODOS os meios ❌ |
| Loading que se esgota (+Length) | ❌ | **tinta carregada finita** ❌ |
| Dirty Brush / MultiColored | ◐ `wet_charge<1` apanha da base | «sujo» que persiste ENTRE traços ❌ · multicolor ❌ |
| Glaze (acumula até à opacidade) · Paint Blending | ✅ `accumulate=false` + `strength` · ✅ `blend` (24) | nada |
| Default Color no preset | ❌ | campo `color_default: Option<[u8;3]>` ❌ |
| Color Noise per Bristle / Stamp / Stroke (HSLuv) | ◐ `color_jitter_{hue,sat,val}` (por carimbo) | **por fio** (ao longo da altura do pincel) e **por traço** ❌; HSLuv vs HSV ◐ |
| Max Impasto Height · Smudge · Thick | ✅ `impasto_depth`, `impasto_plow/push`, `impasto_body` | nada |
| Paper Texture Strength (curva) · Contrast · Scratch Auto/Light/Dark | ◐ `paper_depth`, `paper_edge`, `granulation` | **força do papel em função da pressão**, o **contraste** e o **lado** (claros/escuros) ❌ |
| Length Oiliness/Water Fadeout | ❌ | água/óleo que acaba ao longo do traço ❌ |
| Bristle engine (fios Add/Remove, Thin/Thick/Edges, Perlin por fio, Brush Form) | ◐ `ph2d-wet-paint` tem fios (`Bristle Count/Size`) só no fluido | **motor de partículas** para Digital/Impasto/Aquarela ❌ — doc 01 §4.1 |
| Cursor Mode / Threshold | ◐ | — |
| Volumes: Size · Opacity · Water/Oiliness · Pressure · Length + presets de volumes | ◐ sliders por secção | o **painel de volumes** (5 sliders de topo, presets rápidos) ❌ |
| Preset = 1 ficheiro (PNG + JSON + texturas), grupos, import/export, favoritos | ❌ (só snapshot em memória) | **sistema de presets** — §5 |

---

## §5 — O formato de preset do PH2D (proposta)

**Decisão a tomar pelo padrão-ouro (§0.6):** um preset é **UM ficheiro portátil** com miniatura, parâmetros e
texturas — o que Rebelle (`.rebelle-brush.png`), Procreate (`.brush` = zip) e Krita (`.kpp` = PNG com
texto) fazem; o artista arrasta-o, partilha-o e vê-o na pasta.

```
<Mídia>/<Grupo>/<Nome>.ph2d-brush.png
  PNG 256×256 (miniatura = o traço do preset renderizado pelo nosso motor)
  zTXt  "ph2d-brush"        JSON { schema: 1, media, name, group, spec: BrushSpec (serde), volumes, paint_mode,
                                   textures: { shape: [id…], grain: [id…], background: [id…], dual: id } }
  zTXt  "ph2d-texture-<id>" PNG base64 (só texturas NOSSAS; as procedurais ficam por parâmetros)
  tEXt  "ph2d-origin"       "factory" | "user" | "imported"
```

> ⚠️ **Como ficou na W1** ([05 §4b](05_plano_da_linha_e_o_antes.md)): `iTXt` comprimido e não `zTXt` (o
> `zTXt` é Latin-1, os nomes são UTF-8); texturas num chunk privado `phTx` (PNG cinzento), não base64; zlib
> por `miniz_oxide`, não `flate2` (bytes estáveis por conjunto de build); os mapas `engine`/`brush` são por
> NOME do campo.

- **Onde:** fábrica em `assets/brushes/` (versionado, com gate de contagem — *«número que soma conta-se»*);
  do utilizador em `~/.local/share/ph2d/brushes/`; o painel lista os dois, grupos = pastas, ordem em
  `settings.ron` por pasta.
- **Crate nova `ph2d-brush-presets`** (drop-crate, ADR-0075): `ler(&[u8]) -> Preset`, `escrever(&Preset) ->
  Vec<u8>`, `miniatura(&Preset) -> LayerImage` (corre o motor numa tela 256²), migração por `schema`.
  Nenhum pacote externo novo: o PNG escreve-se à mão (chunks zTXt com `flate2`, que já está no workspace).
- **Volumes** no preset (`size_px, opacity, water, pressure, length`) e o **modo de pintura**, com a
  política do Rebelle: *guardar volumes e modo* / *só volumes* / *não guardar* (menu do painel).
- **Gates (red-first):** round-trip byte-idêntico de cada preset de fábrica; `BrushSpec` com campo novo sem
  default quebra a leitura de um preset `schema` velho ⇒ o gate de migração reprova; a miniatura é
  determinística (seed fixa) — a comparação por hash apanha mudança de lei do motor **em todos os presets
  de uma vez** (é o corpus de regressão que o §1 do método promete).

---

## §6 — Os presets de arranque por mídia (valores NOSSOS)

Notação: campos do `BrushSpec` actual por nome; **negrito** = campo NOVO do plano (01 §4) — o preset
guarda-o já, o motor ignora-o até a onda chegar (gate: *«preset com campo desconhecido não falha a
leitura»*). `shape`/`grain`/`fundo` são as nossas texturas procedurais (`TextureSettings` kinds) ou imagens
nossas em `assets/brushes/texturas/`. Tamanhos em px de diâmetro a 200 dpi. Cores sRGB.

### 6.1 Aquarela (`PaintMedia::Watercolor`) — 8 presets

| preset | size | spacing | shape · grain | strength/fill | água/dilution | edge_gain | granulation | paper_depth | **papel×pressão** | **mix** | notas |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Round | 90 | 0,15 | round soft 1 · paper(canvas) | 0,6 / 0,7 | 0,25 | 0,8 | 0,2 | 1,0 | (0,0,7)→(1,0) | PAINT_MIX, limiar 0,75 | o pincel de todos os dias |
| Wash Round | 250 | 0,03 | round soft 3 · paper | 0,5 / 0,6 | 0,25 | 0,6 | 0,1 | 0,9 | (0,0,6)→(0,6,0) | PAINT_MIX | spacing denso = aguada contínua; `maximum_blending` 1,5 |
| Flat | 90 | 0,12 | flat (scaleY 2) · paper | 0,6 | 0,2 | 0,8 | 0,2 | 1,0 | (0,0,7)→(1,0) | PAINT_MIX | `shape.rake` ON |
| Liner | 12 | 0,08 | round hard · plain | 0,9 | 0,1 | 0,4 | 0 | 0,5 | (0,0,2)→(1,0) | PAINT_MIX | pressure→size (0,0)→(1,1) |
| Granulation | 250 | 0,05 | round noise 2 · granulation(HARD_LIGHT −0,65/1) | 1,0 | 0 | 0,3 | 1,0 | 1,2 | — | PAINT | glaze OFF, `grain_random_splat` |
| Gouache Round | 64 | 0,10 | round soft · paper 1,6 | 0,65 / **opacity (body) 1** | 0,4 | 0,2 | 0 | 1,0 | (0,1)→(0,7,0,3) | PAINT_MIX, `maximum_blending` 2 | o corpo cobre |
| Sumi-e | 78 | 0,12 | 2 formas por **TILT** · 2 grãos 0,25 | 1,0 | 0,19 | 0,9 | 0 | 0,75 | (0,0,84)→(1,0) | PAINT_MIX | `precise_tip_tilt`, tilt×3 |
| Spatter | 120 | 0,3 | splat · plain | 0,8 | 0,3 | 0,6 | 0 | 0,5 | — | PAINT | `spray_count` 6, scatter 1, size jitter 1 |

Comuns à mídia: `pigment=true` (K–M nosso), `paper = documento`, `wet_rewet 0,5`, `wet_charge 0,8`,
`smooth_edges=true`, `impasto=false`; **`grain_dpi` 200** (o grão escala com o documento, não com o pincel).

### 6.2 Óleo & Acrílico (`Impasto`) — 8 presets

| preset | size | spacing | shape · grain | impasto_depth | **opacity×mult** | **smudge/density** | **loading/length** | **texture_order** | **fundo** | notas |
|---|---|---|---|---|---|---|---|---|---|---|
| Round | 78 | 0,12 | round hard · bristle grain 0,5 | 0,5 | 1 × 5 | ON / 4 | ON / 1 | PRESSURE: seco→cheio | — | `shape.rake` ON |
| Flat Oily | 64 | 0,12 | flat (0,96×lock off) bright −0,46/contr 1,6 · grain bright 2 | 0,1 | 1 × 5 | ON / 9 | ON / 1 | PRESSURE | — | oiliness 0,85 → `impasto_plow` 0,8 |
| Filbert | 70 | 0,12 | filbert · bristle | 0,5 | 1 × 5 | ON / 4 | ON / 1 | PRESSURE | HEIGHT 0,5 | |
| Bristle Flat (partículas) | 103 | 0,12 | flat 0,7×1,1 contr 2 | 0,5 | 0,85 × 1 | ON / 0,9 | ON / 1 | — | — | **motor de fios**: thin 0,52 · thick 0,32/size 5 · edges OFF · Brush Form 0 |
| Knife Smooth | 51 | 0,12 | knife · 0,5 | 0,35 | 1 × 6 | OFF | ON | PRESSURE | HARD_MIX papel 0,33 | **tilt_width** (0,0)→(1,−5); `impasto_plow` 1 |
| Rake Sharp | 78 | 0,12 | rake 1,5 lock off · 0,15 | 0,5 | 1 × 25 | ON / 6 | ON | TILT | HARD_MIX | **borda** (structure) −74/−57/−74 |
| Dabber | 60 | 0,5 | sponge · 0,5 | 0,8 | 1 × 3 | OFF | ON | RANDOM | — | scatter 0,2 |
| Metallic Flat | 78 | 0,09 | flat bright −0,49 · 0,15 bright 1 | 0,5 | 1 × 25 | ON / 5 | ON | PRESSURE | HARD_MIX bright 2 | `impasto_metallic` 1, `impasto_shine` 0,9 |

Comuns: `accumulate=true` com **multiplicador** (o corpo), `impasto_smoothing` 0,2, `impasto_shine` 0,5,
`paper_depth` 0,7 com **papel×pressão** `(0,0,7)→(1,0)`, `pigment=true`, **Color Noise per bristle** l 0,1
nos de fios. Express Oil = os mesmos com `impasto_depth` ×1,6, `strength` 0,35–1 e `wet_rewet` (oiliness) 0,8.

### 6.3 Lápis (`Digital` + **papel×pressão**) — 7 presets

| preset | size | spacing | shape · grain | strength | pressure→opacity | **pressure_max** | **tilt (altura)** | **fundo** | **cor** |
|---|---|---|---|---|---|---|---|---|---|
| 2H | 5 | 0,2 | round small 0,85×1,1 · 4 grãos RANDOM | 0,5 | (0,0),(0,35,0,11),(0,73,0,61) | 0,45 | ×3,8 a partir de 55° | INV_MULTIPLY 0,85 | `#8a8a86` |
| HB | 6 | 0,2 | idem | 0,7 | idem | 0,55 | idem | idem | `#6f6f6c` |
| 2B | 6,5 | 0,2 | idem | 0,85 | idem | 0,6 | idem | idem | `#686865` |
| 8B | 8 | 0,2 | idem · grão 0,15 | 1,0 | (0,0)→(0,6,0,8) | 0,75 | idem | idem | `#3a3a38` |
| Charcoal | 37 | 0,12 | flat 1,6 lock off · 0,15 bright 1 | 1,0 | (0,0),(0,41,0,15),(0,59,0,46) | 0,6 | PEN_TILT ×4,2 | MULTIPLY 0,72 + HEIGHT | `#121212` + **dual 14×0,3** |
| Colored 4B | 7 | 0,2 | round · 3 grãos | 0,9 | idem 2B | 0,6 | ×3,8 | INV_MULTIPLY | (cor do artista) |
| Scratch | 3 | 0,1 | round hard · plain | 1,0 | (0,0)→(1,1) | 1 | — | — | — |

Comuns: `accumulate=false`, `use_AA=false` nos ásperos, `paper_depth` 0,9 com curva 3 pontos
`(0,0,71),(0,38,0,56),(0,7,0,37)`, `grain_depth` 1, **smoothing de pressão** 5.

### 6.4 Pastel · Giz · Oil Pastel (`Digital` + fundo + dual) — 5 presets

| preset | size | spacing | shape | strength | pressure_max | **dual (stamp)** | **fundo** | tilt |
|---|---|---|---|---|---|---|---|---|
| Dry Thin | 29 | 0,05 | round scaleY 2 (bastão) · 2 grãos 0,3/0,45 RANDOM 80 | 1 | 0,55 | ON: 0,73 size, opacity curva (0,0),(0,52,0,33),(0,86,1) | — | ×1,6 |
| Square Hard | 40 | 0,06 | square · grain 0,5 | 1 | 0,6 | OFF | — | ×1,6 |
| Soft | 50 | 0,05 | round soft 4 · 0,45 | 0,8 | 0,55 | ON | — | ×2 |
| Oil Pastel | 51 | 0,1 | flat 0,7×1,4 · 1 | 1 | 0,6 | OFF | OVERLAY 0,48 + HEIGHT | ×4,2 a partir de 60° |
| Wax Scrape | 40 | 0,1 | flat · 0,5 | 0,9 | 0,6 | OFF | HEIGHT | — |

### 6.5 Tinta (`Digital` + água) — 5 presets

| preset | size | spacing | shape | pressure→opacity | pressure→size | **smoothing pos/press** | água | notas |
|---|---|---|---|---|---|---|---|---|
| Bullet | 4 | 0,1 | round hard | (0,0),(0,3,0,91),(1,1) | (0,0)→(1,1) | MA 0 / 5 | 0,24 | `max 100` |
| Dip Pen | 15 | 0,1 | round hard · size jitter 0,35 | idem | (0,0)→(1,1) | MA 0 / 5 | 0,49 | a tinta escorre no molhado: `wet_dilution` 0,3 |
| Fountain Square | 12 | 0,1 | square | idem | idem | MA / 5 | 0,2 | `dab_angle` 45 |
| Brush | 29 | 0,12 | 2 formas por PRESSURE (0…22 · 56…68) · 2 grãos 0,5 | (0,0,6),(1,0,6) | (0,0)→(1,1) | — | 0,14 | FOLLOW, `maximum_blending` 2, tilt ×3 |
| Reed Pen | 20 | 0,1 | flat hard | idem Bullet | idem | — | 0,3 | `dab_flatten` 0,6 |

### 6.6 Marcador (`Digital` + glaze) — 4 presets · 6.7 Airbrush — 2 · 6.8 Utilitários

| preset | size | spacing | strength (opacity) | grain | notas |
|---|---|---|---|---|---|
| Marker Bullet | 20 | 0,04 | 0,3 × accumulate ON, tecto 1 | paper **tiled** | sobreposição escurece |
| Marker Chisel | 20 | 0,04 | 0,3 | tiled | `dab_flatten` 0,7 + **PEN_TILT** ×1,6 |
| Marker Liner | 2 | 0,1 | 1 | — | |
| Marker Effect (Hue/Burn/Dodge) | 20 | 0,04 | 0,3 | tiled | `blend` = Hue/ColorBurn/ColorDodge |
| Airbrush Spray | 250 | 0,1 | 0,15 × accumulate | — | pressure→opacity 1, size 0; tilt ×1,76 `(0,75,0,95)` |
| Airbrush Spray 2 | 250 | 0,15 | 0,15 | — | `jitter` 0,3 |
| Water (Wet) | 180 | 0,2 | água 1, só água | — | `WetTool::Wet` |
| Dry | 230 | 0,4 | absorvência 0,85 | — | `WetTool::Dry` |
| Blend Soft | 108 | 0,1 | 0,4 | — | Smear com `impasto_plow` 0 |
| Smudge Soft | 116 | 0,01 | 1 | — | Smear |
| Eraser Round Soft | 128 | 0,1 | 0,3 | — | `toggle_brush_eraser` |

### 6.9 Wet Paint (`WetPaint`, o fluido) — 4 presets

> ✅ **W1e (09/10):** a coluna **Gravity** já não vale — a gravidade é a de fábrica (0,005) nos quatro (F5);
> e cada preset guarda o **Tilt** (F4): Round, Wash e Dry Brush com a tela direita, Wet Drip no máximo, para
> baixo. A nota abaixo (08/10) descreve a W1b, que foi desfeita; a água continua a desta tabela. 05 §4b.

> ⚠️ **Corrigido pela bancada das gotas** ([06](06_gotas_rebelle_x_ph2d.md), decisão do dono 08/10): a
> coluna **Gravity sai do preset** (no Rebelle a gravidade é da TELA e as gotas do PAPEL — 0 de 281 presets a
> têm) e a coluna **Water desce** para a faixa de fábrica da aquarela do Rebelle (9–49 %: Round 29, Wash
> Round 24, Flat Dry 9, Dip Pen 49). O «Wet Drip» passa a ser «muita água», não «muita gravidade». A W1b
> regera a fábrica com isto.

| preset | Water | Pigment/dab | Pickup | Paper Gate | Evaporation | Edge Darkening | Gravity | Bristle Size/Count |
|---|---|---|---|---|---|---|---|---|
| Wet Round | 1,0 | 800 | 0,5 | 0,4 | 1,0 | 0,8 | 0,5 | 2 / 2000 |
| Wet Wash | 1,0 | 400 | 0,3 | 0,3 | 0,7 | 0,6 | 0,3 | 3 / 3000 |
| Wet Dry Brush | 0,2 | 1200 | 0,8 | 0,6 | 1,5 | 0,9 | 0,2 | 1,5 / 1500 |
| Wet Drip | 1,0 | 600 | 0,4 | 0,2 | 0,6 | 0,8 | 1,0 | 2 / 2000 |

> **O que os presets NÃO podem fingir:** o esborrato entre carimbos, a sequência de texturas por pressão,
> o fundo que fura, o pincel secundário, a tinta finita, a mistura por queda de pressão, a largura por
> inclinação e os fios de partículas são **motor**, não preset — é o plano do doc 01. Os presets acima
> já os nomeiam para que, quando a onda chegar, o ficheiro não mude de formato.
