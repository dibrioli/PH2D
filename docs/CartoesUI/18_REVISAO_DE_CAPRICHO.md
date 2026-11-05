# 18 — Revisão de capricho, cartão a cartão (Inspector)

Dono 04/11, foto do 9-Slice: «Faltou conferência e capricho em alguns lugares. Veja como o espaçamento é ruim». Os portões passavam: medem regras, não capricho. Esta lista saiu de OLHAR cada cartão aberto e ampliado (`PH2D_FOTO_DIR=<pasta> cargo test -p ph2d-panel-registry-init --profile smoke --test it q_foto -- --ignored`; arnês `tests/it/q_fotos.rs`, 54 cartões do objecto impossível `o_inspector_armado`, tema `Dark`).

Estados: **curado** · **aberto** · **dúvida** (decisão para o dono). Lei dos vãos: [05 §4 «Ritmo vertical»](05_ESPECIFICACAO_UI_COMPLETA.md); portão `lei_do_ritmo`.

## Famílias (a cura é uma, os cartões são muitos)

| # | Família | Cartões | Estado |
|---|---|---|---|
| A | Fila «+ Add X · x Remove X» no estilo antigo (fundo escuro, traço), colada na lista e sem vão, «x » em ASCII, texto cortado («+ Add Transiti…») | Timers, Signal Actions, State Machine (2), Counter Watch, Trigger, Tween, Shake Emitter, Health/Resistances | aberto |
| B | Título de grupo colado ao controlo de cima (vão 0–5) | Particles (Emission, Velocity, Forces, Look, Signals), Animation (Signals, idle, Playback), Sockets (This object's anchors), Health (Resistances), State Machine (Now), Instance | aberto |
| C | Frase de ajuda colada (vão 2–4) | 9-Slice (duas), Counter Watch (Now) | aberto |
| D | Letra que não diz o que é | 9-Slice X/Y em Left/Top e Right/Bottom; Image H Frames/V Frames com F e F; Nav Agent Repath At e Stuck After com A e A; State Machine From/To/Initial state com S | aberto |
| E | Nome repete o que as letras já dizem | 9-Slice «Size X / Y»; Sockets «Position X / Y (px)», «Bounds X / Y / W / H (px)», «Center X / Y / W / H (px)» | aberto |
| F | Número cortado com «…» em 4 colunas | Sockets: Bounds e Center («0....») | aberto |
| G | Legenda ou nome críptico | 9-Slice (legenda), Sampling (Near+Mip, Lin+Mip, Lin+Aniso), Animation (Fwd, Rev, PP, PP Rev), Texture & Atlas («Reimport at current px/m») | aberto |
| H | Unidade escrita de duas maneiras | «(deg)» × «(°)»; «px/s2» × «px/s²»; «m» em frases com o campo em px (Ray Sensor, Nav Agent, Platform Player, Light 2D) | aberto |
| I | Botões de acção de linha inteira com vão de 9 em vez de 6, e irmãos que dividiriam a linha | Physics Body, Joint (Copy/Paste), Platform Player (Fit to Collider/Fit Crouch), Animation (Add/Remove), Constraints (Bake/Add) | aberto |
| J | Chip sem rótulo, 22 de altura contra 28 de todo controlo, marca menor | Physics Body («Only for tag…»), Tags («Pick a tag…»), Script («Choose a project script…», Browse, Edit script) | aberto |
| K | Corpo antigo dentro do cartão (caixa dentro de caixa, botão sem face) | Instance/Variant, Properties | aberto |
| L | Conta-gotas pequeno, com traço, longe da caixa | Joint (Body A/B), Pulley Wheel (Rope, Mounted On), Game Camera (Target) | aberto |

## Cartão a cartão

| Cartão | Defeito | Família | Estado |
|---|---|---|---|
| 9-Slice | «Per-region tiling» colado na grade e na caixa de cima | B | aberto |
| 9-Slice | Dois botões à direita não acompanham a 3.ª fila da grade | — | aberto |
| 9-Slice | Legenda «Corners F fixed (on/off)…» críptica e colada | C, G | aberto |
| 9-Slice | «Left / Top» com X/Y, «Right / Bottom» com X/Y (deviam ser L/T, R/B) | D | aberto |
| 9-Slice | «Size X / Y» com W/H (é largura e altura finais; 0 = tamanho do sprite) | E | aberto |
| 9-Slice | Frase do «Whole» colada no segmentado e longa | C | aberto |
| Texture & Atlas | Storage e Source podiam dividir linha; «Reimport at current px/m» é jargão | G | aberto |
| Color & Tint | «Equalize Corners» sem rótulo mas claro: ok. Sem defeito novo | — | — |
| Ordering | Nascia dobrado e fora de todo censo e foto: o arnês não o abria | — | aberto |
| Sampling | 6 filtros em 2 filas de larguras diferentes, nomes crípticos | G | aberto |
| Material & Blend | Idem Ordering | — | aberto |
| Visibility | Sem defeito de vão. Grade de 32: células lado 20, vão 2, contra `GRID_GAP` 4 | — | aberto |
| Transform | Sem defeito (aprovado) | — | — |
| Image | H Frames e V Frames com a mesma letra F | D | aberto |
| Animation | «idle» e «Playback» em branco, sem vão; «Signals (empty = silent)» colado; Fwd/Rev/PP | B, G | aberto |
| Animation | Add, Remove, Edit em 3 linhas | I | aberto |
| Physics Body | «+ 2 more shapes» sem vão acima; chip sem rótulo; 3 botões | B, I, J | aberto |
| Joint | Copy e Paste em duas linhas; conta-gotas | I, L | aberto |
| Pulley Wheel | Radius e Out Radius com a mesma letra; conta-gotas; «30.000» × «40» | D, L | aberto |
| Platform Player | «1.00, 0.00 m/s» em m; Fit to Collider e Fit Crouch em duas linhas | H, I | aberto |
| Tags | Chip sem rótulo | J | aberto |
| Constraints | Move Up/Down/Delete na fila antiga; «Glue · Pin — Mão» num corpo maior | A | aberto |
| Particles | Títulos de grupo colados; cor «FF0000» sem amostra visível | B | aberto |
| Macro | Sem defeito de vão | — | — |
| Sockets / Anchors | «This object's anchors» colado; nomes com X/Y; números cortados; «x Remove Anchor» | B, E, F | aberto |
| Top-Down Player | «(deg)» e «px/s2» | H | aberto |
| Projectile Motion | Max Bounces e Bounciness com B e B; px/s2 | D, H | aberto |
| Ray Sensor | «4.50 m» numa frase, campo em px | H | aberto |
| Nav Region | Sem defeito de vão | — | — |
| Nav Agent | Repath At e Stuck After com A; muitas frases em fila; «12.50 m» | D, H | aberto |
| Nav Cost Area | Sem defeito | — | — |
| Nav Link | Sem defeito | — | — |
| Timers | Fila antiga colada; linha cortada com «…» | A | aberto |
| Signal Actions | Fila antiga | A | aberto |
| Factory | Sem defeito de vão | — | — |
| Lifecycle | Sem defeito | — | — |
| Path Follow | Cycle numa célula ao lado de Autostart: alturas boas | — | — |
| Weapon | Sem defeito | — | — |
| State Machine | Fila antiga, com corte; «Now: Closed» sem vão; From/To state com S | A, B, D | aberto |
| HUD | Reference Width e Height: ok. Sem defeito de vão | — | — |
| Sequence Player | «Play nothing» diz mal o que faz | G | dúvida |
| Counter Watch | Fila antiga; «Now: 42» colado | A, C | aberto |
| Trigger | Fila antiga | A | aberto |
| Tween | Fila antiga | A | aberto |
| Health | Resistances com vão pequeno; fila antiga | A, B | aberto |
| Damage | Lasting e Lasts com L | D | aberto |
| Health Bar | Sem defeito | — | — |
| Motion Input | Sem defeito | — | — |
| Audio Source 2D | Browse, Preview, Stop na fila antiga, colados na caixa | A | aberto |
| Game Camera | Conta-gotas dentro da caixa, com traço | L | aberto |
| Camera Shake | Sem defeito | — | — |
| Shake Emitter | Fila antiga; «12 m» na linha | A, H | aberto |
| Parallax | Sem defeito de vão | — | — |
| Light 2D | Quatro frases seguidas: longas para estarem sempre à vista | G | dúvida |
| Light Occluder, Unlit | «x Remove …» em ASCII | A | aberto |
| Script | Chip, Browse e Edit a 22; «greeting» com a caixa antiga e traço; «legacy_speed…» cortado | J | aberto |
| Instance / Variant | Botões sem face; caixa dentro de caixa | K | aberto |
| Properties | Segmentado antigo azul; caixa dentro de caixa | K | aberto |

## Ainda por fotografar

Forma (Fill, Stroke, Effects), Texto, e o cartão Objeto de cada tipo: outro retrato; cena `PH2D_VEC_CARTOES_SMOKE=1`. Render Source: só com estado semeado.
