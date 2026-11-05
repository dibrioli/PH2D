# 18 — Revisão de capricho, cartão a cartão (Inspector)

Dono 04/11, foto do 9-Slice: «Faltou conferência e capricho em alguns lugares. Veja como o espaçamento é ruim». Os portões passavam: medem regras, não capricho. Esta lista saiu de OLHAR cada cartão aberto e ampliado (`PH2D_FOTO_DIR=<pasta> cargo test -p ph2d-panel-registry-init --profile smoke --test it q_foto -- --ignored`; arnês `tests/it/q_fotos.rs`, 54 cartões, todos olhados, do objecto impossível `o_inspector_armado`, tema `Dark`).

Estados: **curado** · **aberto** · **dúvida** (decisão para o dono). Lei dos vãos: [05 §4 «Ritmo vertical»](05_ESPECIFICACAO_UI_COMPLETA.md); portão `lei_do_ritmo`.

## Famílias (a cura é uma, os cartões são muitos)

| # | Família | Cartões | Estado |
|---|---|---|---|
| A | Fila «+ Add X · x Remove X» colada na lista, peças unidas, «x » em ASCII, texto cortado («+ Add Transiti…») | Timers, Signal Actions, State Machine (2), Counter Watch, Trigger, Tween, Shake Emitter, Health/Resistances, Audio Source | **curado**: `rows::fila_de_botoes` (colunas do cartão, `FIELD_H`, vão 6), «x » saiu de 12 textos |
| A2 | Move Up/Down/Delete da pilha de Constraints ainda na fila antiga | Constraints | **curado**: `rows::fila_de_botoes` |
| B | Título de grupo colado ao controlo de cima (vão 0–5) | Particles, Animation (Signals, idle, Playback), Sockets, Health, State Machine | **curado**: `rows::titulo_de_grupo`, `rotulo_do_bloco`, `antes_da_lista` |
| C | Frase de ajuda colada (vão 2–4) | 9-Slice (duas), Counter Watch | **curado**: `rows::aviso` |
| D | Letra que não diz o que é | 9-Slice; H Frames/V Frames; Repath At/Stuck After; From/To/Initial state | **curado**: `Eixos::Letras`, regra de `letra_da_grandeza` |
| D2 | Duas letras iguais lado a lado | Max Bounces e Bounciness; Lasting e Lasts; Radius e Out Radius | aberto (rótulo diferente em cima: baixo) |
| E | Nome repete as letras | 9-Slice Size; Sockets Position, Bounds, Center | **curado** |
| F | Número cortado com «…» em 4 colunas | Sockets Bounds e Center; Visibility Enabler Rect | **curado**: `Eixos::Retangulo` em 2×2, como o Figma |
| G | Nome críptico | 9-Slice (legenda), Sampling, Animation Direction, Texture & Atlas, Sequence | **curado**; «Play nothing» passa a «Clear Cutscene» |
| H | Unidade de duas maneiras | «(deg)» × «(°)»; «px/s2» × «px/s²»; «m» em frases ao lado de campos em px (Ray, Nav Agent, Player, Light 2D, Shake Emitter) | **curado**: `Unit::no_rotulo`, `distancia_na_frase`, `velocidade_na_frase` |
| H2 | «Radius 30.000» × «40»: casas diferentes no mesmo cartão | Pulley Wheel, Particles (Gravity), Player | **curado**: `card_field/valor.rs`, o número só mostra as casas que tem, mesmo com ruído de `f32` (`casas.rs` estava livre: nada largado a meio) |
| I | Botões de acção com vão de 9 em vez de 6; irmãos que dividiriam a linha | todos os 46 de `caixa_do_botao`; Joint Copy/Paste, Player Fit, Animation Add/Remove, Constraints Bake/Add | **curado**: `abaixo_do_botao`; pares em `fila_de_botoes`; Bake e Add em duas colunas, o menu do Add abre por baixo dos dois |
| J | Chip sem rótulo a 22 contra 28 | Physics Body, Tags, Script | **curado**: `rows::caixa_do_chip`; Script com chip, Browse e Edit a `FIELD_H` |
| K | Corpo antigo no cartão: caixa dentro de caixa, botão sem face, vãos 0–4 | Instance/Variant, Properties | **curado**: `sections/instance_cartao.rs`, mesmos ids; `PH2D_CARDS=0` continua no cartão de antes. `PENDENTES_RITMO` do `inspector`: 6 → 0 |
| L | Conta-gotas pequeno, longe da caixa | Joint (Body A/B), Pulley Wheel, Game Camera | **curado**: `rows::caixa_de_alvo`, nome e conta-gotas (`GRID_CELL`) dentro da caixa, como o campo de objecto da Unity |
| M | Listas: linha quadrada, cortada com «…» no resumo | Timers («2.50s · repeats · → resp…»), Script (órfão) | **curado**: o resumo que não cabe perde o sinal inteiro, o órfão perde a razão que a nota de cima já diz; a letra da linha ao meio (`rows::folga_da_linha`, 15 sítios). Canto quadrado fica: é a lei de `paint_row_highlight` |
| N | Cor com alfa 0: «FF0000» sem amostra visível | Particles (Color at Death) | **curado**: `card_field::paint_amostra`, metade sem alfa e metade como é, à Figma |
| O | Chip de tag (`Enemy ×`) medido como rótulo: 2 pendentes falsos | Tags, régua | **curado**: `tags.rs` diz à régua que o chip é um controlo |

A primeira ronda de fotos saiu em `Forge` (fundos bordô, botões com traço): artefacto do `MockPanelHost::new()`, não do app. Daí em diante, `in_theme(Dark)`.

## Cartão a cartão (o que ficou de cada um)

| Cartão | Olhado | Estado |
|---|---|---|
| Object, Transform, Color & Tint, Ordering, Material & Blend, Macro, Top-Down Player, Ray Sensor, Nav Region, Nav Cost Area, Nav Link, Factory, Lifecycle, Path Follow, Weapon, HUD, Health Bar, Motion Input, Game Camera (salvo L), Camera Shake, Parallax, Light Occluder, Unlit, Damage, Projectile Motion | sim | sem defeito de vão (D2 baixo) |
| 9-Slice, Texture & Atlas, Sampling, Image, Animation, Physics Body, Joint (salvo L), Platform Player, Tags, Particles (salvo N), Sockets/Anchors, Nav Agent, Timers, Signal Actions, State Machine, Counter Watch, Trigger, Tween, Health, Audio Source 2D, Shake Emitter | sim | curados por família |
| Pulley Wheel | sim | curado: H2, L |
| Constraints | sim | curado: A2, I |
| Sequence Player | sim | decidido: «Clear Cutscene» |
| Light 2D | sim | decidido: as duas frases mais longas encurtadas; as outras duas dizem o que só elas dizem |
| Script | sim | curado: J, M |
| Instance/Variant, Properties | sim | curado: K |
| Visibility | sim | grade de 32 com `GRID_CELL`, vão menor de propósito (05 §4) |

## Os outros painéis (fatia q3, 04/11)

Arnês: `PH2D_FOTO_DIR=<pasta> PH2D_FOTO_PAINEIS=vector,physics cargo test -p ph2d-panel-registry-init --profile smoke --test it q_foto_paineis -- --ignored` (de fábrica, armado e em cada estado de `censo_estados`). Olhados: Vector (6 estados), Painter (`molhado`, 3 fatias de 6), Physics, Flip, Pixel, Tags, Audio Mixer, Audio Editor, Macros, Padding, Upscale, Equalize, Background Removal, Color Equalization, Bones. Fatias recortadas ao que foi pintado: um painel flutuante (Wet Tuning, Grid, Assets, Tokens) não mora na coluna.

### Famílias

| # | Família | Onde | Estado |
|---|---|---|---|
| P | Grupo de botões colados em peça única (vão 0, cantos só nas pontas) num cartão | Audio Editor (Transport, Loop, Edit, Effects, Markers, Variations, Spectral) | **curado**: `cards_grupo` (FIELD_GAP, FIELD_ROW_GAP, cantos inteiros). `PH2D_CARDS=0` continua encostado |
| Q | Texto a sair por baixo da caixa do nome; fileira vazia em Delivery (`stepper_row` já somava, somava-se outra vez: o painel de antes também a tinha) | Audio Editor (Transport, Delivery) | **curado** (`text_input` centra a linha medida num cartão; `y += gap`) |
| Q2 | `◀ ▬ ▶` sem texto: em Effects e Delivery o nome vem do motor; na foto sem motor fica vazio | Audio Editor | **curado**: o estado do arnês publica os nomes mais compridos, nada corta. Só na tabela de textos: Compressor, Multiband Compressor, Transient Shaper, Stereo Delay, Room Reverb, Comb Filter, Ring Modulator; Sharpness (Q), Frequency, Sensitivity, Damping; Final Mix, Sci-Fi Radio; Load/Change Room…; Measuring the export. Ids e ficheiros v1 iguais |
| R | Slider com rótulo à esquerda, fora da grelha; valor 3 px abaixo da barra | Audio Editor (Crossfade, Amount, Pitch/Gain jitter, Quality) | **curado**: rótulo em cima, barra e valor na mesma linha |
| R2 | «Add clips to build a set», tempo do clip, «RAM —» | Audio Editor | **curado**: `grupo::linha_de_texto` |
| R4 | A linha escolhida da Chain saía 4 px da coluna de cada lado, com o texto baixo | Audio Editor | **curado** (foto `q4_audio_efeitos.png`) |
| R3 | Ícones de Chain e o de repor sem face e maiores que os dos cartões aprovados | Audio Editor (Effects) | **curado**: `grupo::glifo` (ICON_GLYPH), face e vão de todo cartão |
| S | Frase indentada (x do corpo + 8) | Macros | **curado**: `bloco`/`linha` sem recuo num cartão |
| T | Lista `1. Stroke 1 ▪ 0.50`: linha baixa, texto abaixo da amostra | Macros | **curado**: `FIELD_H`, texto no meio |
| U | Título de sub-grupo a 8 do controlo de cima | Vector Advanced | **curado**: 14 até às letras (`o_advanced_separa…`) |
| U2 | Chevron «⌄» 3 px acima do texto. O glifo está certo (centrado na caixa): quem descia era o título. Num cartão, `paint_section_header` sai cedo para `grupo::paint_group_header`, por isso a cura em `section_header` nunca corria (saiu). O grupo usa `folga_do_texto`, como todo texto de cartão | Vector Advanced, grupos de todo cartão | **curado** (`as_maiusculas_do_grupo_ficam_no_meio_do_chevron`, foto) |
| AG | Faixa inteira no idioma antigo: botões cinzentos (Ghost, Pin, Auto…), números com setinhas pretas, rótulos à esquerda, texto de lista abaixo da caixa | Flip Frames | **curado** na q4 (`barra_cartoes.rs`): moldura sem cartão, como a Timeline; célula de 78 para 118 px (98 no painel de antes) |
| AH | Nome da camada 2 px abaixo dos ícones da linha | Pixel Timeline, Pixel | **curado**: `folga_do_texto` em cartões |
| V | «Pick Shapes» com a letra menor que «Blend»: é um segmentado de uma peça (o modo acende) ao lado de um botão | Vector Advanced | dúvida (Widget::Choice × Button; o tom do modo é decisão) |
| W | Abreviatura que cabia inteira: `Persp`, `Symm`, `Accel`, `Morph t`, `Norm LUFS`, `Rm DC`, `Comp`, `Iso`, `Stag Sq`, `Stag Hex`, `Tri`, `Von4`, `Moore8` | Vector, Audio, Grid | **curado** |
| W2 | Ainda crípticos: `Offset (1/n)`, `Jetty`, `+M`, `DTCG`, `Cross` (Snap) | Vector, Timeline, Design Tokens | dúvida: sem nome melhor sem o dono (`Jetty` é o nome do Visio) |
| X | Duas letras iguais lado a lado (D2): Size/Spacing «S S», Spacing/Slide, Temperature/Tint, Cover/Dry cover, Fall-off/Full for, Random 2nd/Pressure 2nd «2» | Vector, Painter, Color Equalization, e o D2 do Inspector | **curado** de uma vez, na fila (`panel::cards::letras`): o campo da direita mostra a letra que o distingue (inicial de outra palavra, senão a letra seguinte): S·P, T·I, F·U. Letra na fila de baixo pode repetir: não estão lado a lado |
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
| Painter | Wet Paint e, na q4, Layers, Deform, Select, Composite, Impasto, Watercolor, abertos | curado. E10 intocado. Sem defeito novo. Por olhar: Mask (Clone, Inpaint e Sculpt fotografados e não lidos cartão a cartão) |
| Wet Tuning | as 4 fatias | sem defeito novo. Dúvida: `Smooth wash body (off · on)` e `Lock material (off · on)` são números de 0 a 1 que agem como caixa; mudar de widget é mudar o controlo |
| Physics, Flip, Tags, Assets, Padding, Upscale, Equalize, Background Removal, Color Equalization, Bones, Design Tokens, Timeline (barra) | sim, com as fatias seguintes na q4 | sem defeito de vão |
| Grid | inteiro (`PH2D_FOTO_ROLA=700`, a janela rola em vez de crescer) | sem defeito novo; H2 curado na q2 |
| Audio Mixer, Audio Editor, Macros | sim | curados salvo R3, Q2 |
| Pixel | Pencil, Colors, Layers, Palette, Selector, Symmetry, Select, Edit, Sprite | curado salvo AF |
| Motion Graph (barra) | sim, no app (`PH2D_GPU_COOK_DEMO=1`) | sem defeito novo no cromo; nós coloridos e pílula como o doc 14 |
| Flip Frames, Pixel Timeline | de fábrica e armado; Frames também no app | AG e AH curados |

### Fatia q4 (04/11): o plano acima, feito

Tudo fotografado depois das curas: [Frames, arnês](maquete/q4_frames.png) · [Frames, no app](maquete/q4_frames_no_app.png) · [Effects](maquete/q4_audio_efeitos.png). Ficou por olhar: Mask do Painter, cartão a cartão; a lista aberta de Frames e o Audio Editor com clip real no app (a cena `PH2D_AUDIO_DELIVERY_SMOKE` não abre o painel: pede o HOME semeado da receita 11; o arnês viu os nomes).

## Decisões tomadas sem o dono

- Sequence: o botão que tira a cutscene chama-se «Clear Cutscene» («Play nothing» parecia uma ordem de tocar).
- Light 2D: «Too small to light anything at this zoom. Zoom in or make the lamp bigger.» e «{n} of {total} sprites that glow or cast shadows use their box, not their pixels.». A explicação de malha e tiling saiu da frase.
- Properties: a escolha de variante não leva marca de animação (o cartão de antes também não a tinha).
- Hierarchy: sem lugar para o nome e o selo juntos, o selo sai. O nome é por onde se acha a linha.
- Tokens e Wet na coluna de 220 px: os ícones descem para a linha de baixo, e os números vão um por linha (lei 5 de 05 §4).
- R6, coluna de 220 px: os pares de botões do Audio Editor, Delay, Foreground/Background e Sync descem de linha em vez de cortar o nome; o painel fica mais alto nessa largura. As abas num cartão usam o corpo do segmentado (11 px).
- R6, Pixel: o resumo no título de Colors mostra 6 dígitos; o alfa fica na linha da cor.
- R6, Render Source: a frase do preço do RGBA16 também quebra com `PH2D_CARDS=0`, em vez de cortar. É a única mudança de R6 no painel de antes.

- Audio Editor: o dono pediu em 2026-09 botões encostados («tudo o que puder ser ajuntado, ajunte»), no painel antigo. Num cartão vale a lei de 05 §4 e o padrão aprovado de Physics e Flip: soltos. `PH2D_CARDS=0` fica como ele pediu.
- Slider do Audio Editor com rótulo em cima, como todo campo (02 §2); o valor continua a ser a leitura do motor, sem caixa.
- «Morph t» virou «Amount», «Von4/Moore8» viraram «4 Neighbors/8 Neighbors», «Comp» virou «Compressor». Os ids não mudam.
- 9-Slice: os dois atalhos saíram de ao lado da grade para uma fila própria nas colunas do cartão; ao lado da grade ficou a legenda, fila a fila. Rive edita N-Slice por eixos no canvas, sem grade: não serve de modelo aqui.
- Frames: o painel que era um cartão único passa a moldura sem cartão: a altura é da tira, como na Timeline. Campos com o rótulo ao lado e a mesma coluna para os cinco. Auto, Falloff, Additive, Pin, Ghost, Fade, Pairs viraram caixas (o id continua Button, emite o mesmo Click), `Add` botão. Dica em cada ícone e caixa.
- Letras iguais: regra na fila, não letra declarada campo a campo: cobre os ~45 sítios de uma vez. Só muda o da direita, e só se repete.
- Efeitos: abreviaturas e nomes soltos viram o nome por extenso. `Peak EQ`, `Low Shelf`, `De-Esser`, `Gate / Expander`, codecs e `Bitrate` ficam: são o nome que o artista encontra em qualquer outro programa. `Haas` virou `Stereo Delay`, `Q` virou `Sharpness (Q)`.
- «Remove X» sem o «x ». Direction com os nomes do Aseprite (Forward, Reverse, Ping-pong, Ping-pong Rev): a fila desce para duas linhas, sem corte.
- Frases com distância passam a seguir a unidade escolhida (px ou m), como o campo ao lado.
- O vão da grade 3×3 é o de toda linha (6), não `GRID_GAP` (4): a lei 3 de 05 §4 vale para ela.

## Ainda por fotografar

Forma (Fill, Stroke, Appearance, Transform): olhada, [q2_forma.png](maquete/q2_forma.png), sem defeito. R6, arnês (`PH2D_FOTO_OBJETO=vetor|imagem|outro`): [Text](maquete/r6_texto.png), [Effects com duas camadas](maquete/r6_effects.png), [Object de cada tipo](maquete/r6_objeto_de_cada_tipo.png), [Render Source](maquete/r6_render_source.png). Object: ícone do tipo, nome e olho, igual nos três; o cadeado só nasce com linha na Hierarchy, e a foto não a tem. Cartões nas outras janelas: `PENDENTES_RITMO` aponta os deles.

**Curado, achado pela foto do Render Source.** O arnês publicava `source_precision: None`, estado que a app só tem sem imagem, e escondia o caminho real: com `Rgba8` a frase do preço separava `Format` de `Emissive` (`lei_da_compactacao`: `Source + Format`), e cortava a 268 px com `PH2D_CARDS=0`. Num cartão a frase vem agora depois da fila dos dois; fora dele, quebra em vez de cortar (a única mudança no painel de antes). A fixtura passa a `Rgba8`, sem subir catraca. Contraprova: a frase de volta antes do `Emissive` ⇒ vermelho.

**Curado (F4):** o resumo do Variant quebrava com o «·» no início da linha 2. `paint_text_block` cola o « · » à palavra de antes (NBSP), para toda frase que quebra; teste de 40 a 320 px.
