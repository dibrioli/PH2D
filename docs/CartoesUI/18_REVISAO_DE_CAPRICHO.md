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
| P | Grupo de botões colados em peça única (vão 0, cantos só nas pontas) num cartão; `Play`, `Load`, `Cut`… | Audio Editor (Transport, Loop, Edit, Effects, Markers, Variations) | aberto |
| Q | Selector com o texto a sair por baixo («Footsteps»), caixa vazia sem nome («◀ ▬ ▶»), vão sem nada em Delivery | Audio Editor (Transport, Effects, Delivery) | aberto |
| R | Slider com rótulo à esquerda e 3 colunas próprias, fora da grelha do cartão; `Add clips to build a set` centrada e a 14 | Audio Editor (Crossfade, Amount, Pitch/Gain jitter, Quality) | aberto |
| S | Frase indentada 6 px (x do corpo + 6): `Record captures the Painter`, `Edited from step 2`, `Your library is empty` | Macros | aberto |
| T | Amostra de cor pequena e alta na lista `1. Stroke 1 ▪ 0.50`; lista sem vão entre linhas | Macros | aberto |
| U | Título de sub-grupo (Blend, Morph, Envelope, Pattern on Path, Contour) a 4–8 do controlo de cima em vez de `GROUP_TOP` 14; chevron pequeno e acima do texto | Vector Advanced | aberto |
| V | Texto menor num irmão: «Pick Shapes» a corpo menor e com traço, ao lado de «Blend» | Vector Advanced | aberto |
| W | Abreviatura que cabia inteira: `Persp`, `Symm`, `Accel`, `Norm LUFS`, `Rm DC`, `Comp`, `Morph t` | Vector, Audio | aberto |
| X | Duas letras iguais lado a lado (D2 de cima): Size/Spacing «S S», Spacing/Slide, Width/Slant/Optical, Temperature/Tint, Cover/Dry cover, Fall-off/Full for, Random 2nd/Pressure 2nd «2» | Vector, Painter, Color Equalization | aberto |
| Y | Grelha partida: Align (3 ícones) mais estreito que a coluna, Width começa fora dela | Vector Paragraph | aberto |
| Z | Rótulo ou frase a 10 por ser `Whole canvas`/`no canvas` solto (`lei_do_ritmo`), Taper com 28 de vão | Painter | aberto |
| AA | Texto solto a fazer de campo, parece título: «Hello / world» sem caixa | Vector Text | dúvida |
| AB | Nome do botão repete o título: «Text on Path» no cartão Text on Path; «Morph», «Envelope» idem | Vector | baixo |
| AC | Linhas da lista a 2 de vão contra 6 | Tags (`All tags`) | aberto |
| AD | Caixa de marcar com a marca colada no nome, fora da coluna | Audio Mixer (Limiter, Ducking Enabled) | aberto |
| AE | Lista de botões de 1 coluna quando os nomes curtos cabem em 2: `All`, `Inverse`, `Color Range…`, `Load from MSK…`, `Rotate 90 CCW`, `Invert…`, `Insert Text` | Pixel (Select, Edit, Sprite) | aberto |
| AF | Selector de 5 opções longas em 3 filas | Pixel (Selector) | baixo |

## Decisões tomadas sem o dono

- 9-Slice: os dois atalhos saíram de ao lado da grade para uma fila própria nas colunas do cartão; ao lado da grade ficou a legenda, fila a fila. Rive edita N-Slice por eixos no canvas, sem grade: não serve de modelo aqui.
- «Remove X» sem o «x ». Direction com os nomes do Aseprite (Forward, Reverse, Ping-pong, Ping-pong Rev): a fila desce para duas linhas, sem corte.
- Frases com distância passam a seguir a unidade escolhida (px ou m), como o campo ao lado.
- O vão da grade 3×3 é o de toda linha (6), não `GRID_GAP` (4): a lei 3 de 05 §4 vale para ela.

## Ainda por fotografar

Forma (Fill, Stroke, Effects), Texto, e o cartão Objeto de cada tipo: outro retrato; cena `PH2D_VEC_CARTOES_SMOKE=1`. Render Source: só com estado semeado. Cartões nas outras janelas (Bones, Painter, Physics): `PENDENTES_RITMO` aponta os deles.
