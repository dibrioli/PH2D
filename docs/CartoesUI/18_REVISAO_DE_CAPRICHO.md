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

## Decisões tomadas sem o dono

- 9-Slice: os dois atalhos saíram de ao lado da grade para uma fila própria nas colunas do cartão; ao lado da grade ficou a legenda, fila a fila. Rive edita N-Slice por eixos no canvas, sem grade: não serve de modelo aqui.
- «Remove X» sem o «x ». Direction com os nomes do Aseprite (Forward, Reverse, Ping-pong, Ping-pong Rev): a fila desce para duas linhas, sem corte.
- Frases com distância passam a seguir a unidade escolhida (px ou m), como o campo ao lado.
- O vão da grade 3×3 é o de toda linha (6), não `GRID_GAP` (4): a lei 3 de 05 §4 vale para ela.

## Ainda por fotografar

Forma (Fill, Stroke, Effects), Texto, e o cartão Objeto de cada tipo: outro retrato; cena `PH2D_VEC_CARTOES_SMOKE=1`. Render Source: só com estado semeado. Cartões nas outras janelas (Bones, Painter, Physics): `PENDENTES_RITMO` aponta os deles.
