# 18 — Revisão de capricho, cartão a cartão (Inspector)

Dono 04/11, foto do 9-Slice: «Faltou conferência e capricho em alguns lugares. Veja como o espaçamento é ruim». Os portões passavam: medem regras, não capricho. Esta lista saiu de OLHAR cada cartão aberto e ampliado (`PH2D_FOTO_DIR=<pasta> cargo test -p ph2d-panel-registry-init --profile smoke --test it q_foto -- --ignored`; arnês `tests/it/q_fotos.rs`, 54 cartões, todos olhados, do objecto impossível `o_inspector_armado`, tema `Dark`).

Estados: **curado** · **aberto** · **dúvida** (decisão para o dono). Lei dos vãos: [05 §4 «Ritmo vertical»](05_ESPECIFICACAO_UI_COMPLETA.md); portão `lei_do_ritmo`.

## Famílias (a cura é uma, os cartões são muitos)

| # | Família | Cartões | Estado |
|---|---|---|---|
| A | Fila «+ Add X · x Remove X» colada na lista, peças unidas, «x » em ASCII, texto cortado («+ Add Transiti…») | Timers, Signal Actions, State Machine (2), Counter Watch, Trigger, Tween, Shake Emitter, Health/Resistances, Audio Source | **curado**: `rows::fila_de_botoes` (colunas do cartão, `FIELD_H`, vão 6), «x » saiu de 12 textos |
| A2 | Move Up/Down/Delete da pilha de Constraints ainda na fila antiga | Constraints | aberto |
| B | Título de grupo colado ao controlo de cima (vão 0–5) | Particles, Animation (Signals, idle, Playback), Sockets, Health, State Machine | **curado**: `rows::titulo_de_grupo`, `rotulo_do_bloco`, `antes_da_lista` |
| C | Frase de ajuda colada (vão 2–4) | 9-Slice (duas), Counter Watch | **curado**: `rows::aviso` |
| D | Letra que não diz o que é | 9-Slice; H Frames/V Frames; Repath At/Stuck After; From/To/Initial state | **curado**: `Eixos::Letras`, regra de `letra_da_grandeza` |
| D2 | Duas letras iguais lado a lado | Max Bounces e Bounciness; Lasting e Lasts; Radius e Out Radius | aberto (rótulo diferente em cima: baixo) |
| E | Nome repete as letras | 9-Slice Size; Sockets Position, Bounds, Center | **curado** |
| F | Número cortado com «…» em 4 colunas | Sockets Bounds e Center; Visibility Enabler Rect | **curado**: `Eixos::Retangulo` em 2×2, como o Figma |
| G | Nome críptico | 9-Slice (legenda), Sampling, Animation Direction, Texture & Atlas | **curado**; Sequence «Play nothing»: dúvida |
| H | Unidade de duas maneiras | «(deg)» × «(°)»; «px/s2» × «px/s²»; «m» em frases ao lado de campos em px (Ray, Nav Agent, Player, Light 2D, Shake Emitter) | **curado**: `Unit::no_rotulo`, `distancia_na_frase`, `velocidade_na_frase` |
| H2 | «Radius 30.000» × «40»: casas diferentes no mesmo cartão | Pulley Wheel, Particles (Gravity), Player | aberto: outro agente mexe em `number_input/casas.rs` |
| I | Botões de acção com vão de 9 em vez de 6; irmãos que dividiriam a linha | todos os 46 de `caixa_do_botao`; Joint Copy/Paste, Player Fit, Animation Add/Remove | **curado**: `abaixo_do_botao`; pares em `fila_de_botoes`. Aberto: Constraints Bake/Add (o Add abre menu ancorado) |
| J | Chip sem rótulo a 22 contra 28 | Physics Body, Tags | **curado**: `rows::caixa_do_chip`. Aberto: Script (chip, Browse, Edit, «greeting») |
| K | Corpo antigo no cartão: caixa dentro de caixa, botão sem face, vãos 0–4 | Instance/Variant, Properties | aberto: `PENDENTES_RITMO` (4 das 6 do Inspector). 5 ficheiros `instance*`, `line` é a altura do botão |
| L | Conta-gotas pequeno, longe da caixa | Joint (Body A/B), Pulley Wheel, Game Camera | aberto |
| M | Listas: linha quadrada, cortada com «…» no resumo | Timers («2.50s · repeats · → resp…») | aberto |
| N | Cor com alfa 0: «FF0000» sem amostra visível | Particles (Color at Death) | aberto |
| O | Chip de tag (`Enemy ×`) medido como rótulo: 2 pendentes falsos | Tags, régua | aberto: a régua precisa de saber que é chip |

A primeira ronda de fotos saiu em `Forge` (fundos bordô, botões com traço): artefacto do `MockPanelHost::new()`, não do app. Daí em diante, `in_theme(Dark)`.

## Cartão a cartão (o que ficou de cada um)

| Cartão | Olhado | Estado |
|---|---|---|
| Object, Transform, Color & Tint, Ordering, Material & Blend, Macro, Top-Down Player, Ray Sensor, Nav Region, Nav Cost Area, Nav Link, Factory, Lifecycle, Path Follow, Weapon, HUD, Health Bar, Motion Input, Game Camera (salvo L), Camera Shake, Parallax, Light Occluder, Unlit, Damage, Projectile Motion | sim | sem defeito de vão (D2 baixo) |
| 9-Slice, Texture & Atlas, Sampling, Image, Animation, Physics Body, Joint (salvo L), Platform Player, Tags, Particles (salvo N), Sockets/Anchors, Nav Agent, Timers, Signal Actions, State Machine, Counter Watch, Trigger, Tween, Health, Audio Source 2D, Shake Emitter | sim | curados por família |
| Pulley Wheel | sim | aberto: H2, L |
| Constraints | sim | aberto: A2, I |
| Sequence Player | sim | dúvida: «Play nothing» |
| Light 2D | sim | dúvida: quatro frases seguidas, longas para estarem sempre à vista |
| Script | sim | aberto: J |
| Instance/Variant, Properties | sim | aberto: K |
| Visibility | sim | grade de 32 com `GRID_CELL`, vão menor de propósito (05 §4) |

## Os outros painéis (fatia q3, 04/11)

Arnês: `PH2D_FOTO_DIR=<pasta> PH2D_FOTO_PAINEIS=vector,physics cargo test -p ph2d-panel-registry-init --profile smoke --test it q_foto_paineis -- --ignored` (de fábrica, armado e em cada estado de `censo_estados`). Olhados: Vector (6 estados), Painter (`molhado`, 3 fatias de 6), Physics, Flip, Pixel, Tags, Audio Mixer, Audio Editor, Macros, Padding, Upscale, Equalize, Background Removal, Color Equalization, Bones. O Wet Tuning, Grid, Design Tokens e Assets não saem na coluna do retrato (por rever).

### Famílias

| # | Família | Onde | Estado |
|---|---|---|---|
| P | Grupo de botões colados em peça única (vão 0, cantos só nas pontas) num cartão | Audio Editor (Transport, Loop, Edit, Effects, Markers, Variations, Spectral) | **curado**: `cards_grupo` (FIELD_GAP, FIELD_ROW_GAP, cantos inteiros). `PH2D_CARDS=0` continua encostado |
| Q | Texto a sair por baixo da caixa do nome; fileira vazia em Delivery (`stepper_row` já somava, somava-se outra vez: o painel de antes também a tinha) | Audio Editor (Transport, Delivery) | **curado** (`text_input` centra a linha medida num cartão; `y += gap`) |
| Q2 | `◀ ▬ ▶` sem texto: em Effects e Delivery o nome vem do motor; na foto sem motor fica vazio | Audio Editor | olhar no app (a cena do arnês não publica presets) |
| R | Slider com rótulo à esquerda, fora da grelha; valor 3 px abaixo da barra | Audio Editor (Crossfade, Amount, Pitch/Gain jitter, Quality) | **curado**: rótulo em cima, barra e valor na mesma linha |
| R2 | «Add clips to build a set», tempo do clip, «RAM —» | Audio Editor | **curado**: `grupo::linha_de_texto` |
| R3 | Ícones de Chain e o de repor maiores que os dos cartões aprovados; `Chain` à esquerda | Audio Editor (Effects) | aberto |
| S | Frase indentada (x do corpo + 8) | Macros | **curado**: `bloco`/`linha` sem recuo num cartão |
| T | Lista `1. Stroke 1 ▪ 0.50`: linha baixa, texto abaixo da amostra | Macros | **curado**: `FIELD_H`, texto no meio |
| U | Título de sub-grupo a 8 do controlo de cima | Vector Advanced | **curado**: 14 até às letras (`o_advanced_separa…`) |
| U2 | Chevron do sub-grupo 3 px acima do texto. A cura em `section_header` (centrar a linha medida num cartão) está lá; a foto não mudou: o pintor do título não lê a linha que eu medi | Vector Advanced | aberto |
| V | «Pick Shapes» com a letra menor que «Blend»: é um segmentado de uma peça (o modo acende) ao lado de um botão | Vector Advanced | dúvida (Widget::Choice × Button; o tom do modo é decisão) |
| W | Abreviatura que cabia inteira: `Persp`, `Symm`, `Accel`, `Morph t`, `Norm LUFS`, `Rm DC`, `Comp`, `Iso`, `Stag Sq`, `Stag Hex`, `Tri`, `Von4`, `Moore8` | Vector, Audio, Grid | **curado** |
| W2 | Ainda crípticos: `Offset (1/n)`, `Jetty`, `+M`, `DTCG`, `Cross` (Snap) | Vector, Timeline, Design Tokens | dúvida: sem nome melhor sem o dono (`Jetty` é o nome do Visio) |
| X | Duas letras iguais lado a lado (D2): Size/Spacing «S S», Spacing/Slide, Width/Slant/Optical, Temperature/Tint, Cover/Dry cover, Fall-off/Full for, Random 2nd/Pressure 2nd «2» | Vector, Painter, Color Equalization | aberto: o rótulo em cima desambigua; pede letra declarada por campo |
| Y | Align mais estreito que a coluna, Width fora dela | Vector Paragraph | **curado**: `Widget::Choice` |
| Y2 | `Morph` sob o vazio do rótulo vizinho (botão à esquerda, campo com rótulo à direita) | Vector Advanced | baixo |
| Z | Rótulo solto a 10 de um controlo sem rótulo: `Whole canvas`, `no canvas`, `Lighting`, `Modifiers`, `Overlay Color`, `Taper` | Painter | **curado**: `rotulo_sobre_controlo`; o visor do Taper entra no censo com o tamanho dele |
| Z2 | Rótulo curto a meio da faixa quando o vizinho quebra em duas linhas (11,5) | Wet Tuning | **curado**: cola no controlo (`cards_rotulos`) |
| AA | «Hello / world» sem caixa, parece título | Vector Text | dúvida: é a leitura do texto, edita-se no canvas |
| AB | Nome do botão repete o título: «Text on Path», «Morph», «Envelope» | Vector | baixo |
| AC | Linhas da lista sem vão | Tags, Macros | não é defeito: a lista contígua é a do Figma e a da porta `list_row` |
| AD | Marca colada ao nome numa caixa de meia coluna | Audio Mixer | não é defeito: `check_em`, lei de 05 §8 |
| AE | Verbos de 1 coluna quando dois cabem | Pixel (Select, Edit, Sprite) | **curado**: `filas_de_botoes` |
| AF | Selector de 5 opções longas em 3 filas | Pixel (Selector) | baixo |

### Painel a painel

| Painel | Olhado | Estado |
|---|---|---|
| Vector | Transform, Snap, Fill & Stroke, Effects, Pattern, Stroke Pattern, Brush, Connector, Vertex, Text, Font, Paragraph, Text on Path, Axes, Advanced | curado salvo X, U2, V, W2; Fill & Stroke e Effects são F43/F44: intocados |
| Painter | Wet Paint (Basics, Mixing, Randomize, Shape, Paper, Grain, Stroke, Pen, Symmetry, Tiling) | curado salvo X. E10 intocado. **Por olhar**: Layers, Mask, Deform, Selection, Composite, Impasto, Watercolor, aberto a aberto (o portão mede-os) |
| Wet Tuning | Paint, Water (2 de 4 fatias) | portão a zero; por olhar: as 2 últimas fatias |
| Physics, Flip, Tags, Assets, Padding, Upscale, Equalize, Background Removal, Color Equalization (1.ª fatia), Bones (1.ª fatia), Design Tokens (1.ª fatia), Timeline (barra) | sim | sem defeito de vão. **Por olhar**: Color Eq, Bones e Tokens, fatias seguintes |
| Grid | 1.ª fatia | W curado; H2 (`Radius 30.000`) é do outro agente; por olhar a 2.ª |
| Audio Mixer, Audio Editor, Macros | sim | curados salvo R3, Q2 |
| Pixel | Pencil, Colors, Layers, Palette, Selector, Symmetry, Select, Edit, Sprite | curado salvo AF |
| Motion Graph (barra) | **não**: `q_foto_paineis` deu 0 fatias sem documento | por fazer: pede cena |
| Flip Frames, Pixel Timeline | fotos em `q_foto_paineis`, **não olhadas** | por olhar |

## Decisões tomadas sem o dono

- Audio Editor: o dono pediu em 2026-09 botões encostados («tudo o que puder ser ajuntado, ajunte»), no painel antigo. Num cartão vale a lei de 05 §4 e o padrão aprovado de Physics e Flip: soltos. `PH2D_CARDS=0` fica como ele pediu.
- Slider do Audio Editor com rótulo em cima, como todo campo (02 §2); o valor continua a ser a leitura do motor, sem caixa.
- «Morph t» virou «Amount», «Von4/Moore8» viraram «4 Neighbors/8 Neighbors», «Comp» virou «Compressor». Os ids não mudam.
- 9-Slice: os dois atalhos saíram de ao lado da grade para uma fila própria nas colunas do cartão; ao lado da grade ficou a legenda, fila a fila. Rive edita N-Slice por eixos no canvas, sem grade: não serve de modelo aqui.
- «Remove X» sem o «x ». Direction com os nomes do Aseprite (Forward, Reverse, Ping-pong, Ping-pong Rev): a fila desce para duas linhas, sem corte.
- Frases com distância passam a seguir a unidade escolhida (px ou m), como o campo ao lado.
- O vão da grade 3×3 é o de toda linha (6), não `GRID_GAP` (4): a lei 3 de 05 §4 vale para ela.

## Ainda por fotografar

Forma (Fill, Stroke, Effects), Texto, e o cartão Objeto de cada tipo: outro retrato; cena `PH2D_VEC_CARTOES_SMOKE=1`. Render Source: só com estado semeado. Cartões nas outras janelas (Bones, Painter, Physics): `PENDENTES_RITMO` aponta os deles.
