# 03 — Os 4 motores: o que exprimem HOJE e as LACUNAS que o catálogo exige (a fila da JANELA 2)

> Entrega E da JANELA 1 (19/10). Leitor: a JANELA 2 e os seus agentes. Fonte: o CÓDIGO da linha (`line/PainterWatercolor`),
> `arquivo:linha`; o censo dos 281 presets do Rebelle (`docs/Painter/NewFeaturesWetPaint/fixtures/rebelle_presets_281.json`,
> contado por script nesta entrega: «Rb N» = N presets que USAM o recurso); e o catálogo [`01_CATALOGO.md`](01_CATALOGO.md)
> (ids `A-1`…`M-7`, capacidades `CAP-01`…`CAP-27` do seu §1 — cruzadas aqui com o código).
> `PB` = `crates/ph2d-painter-brush/src/` · `TP` = `crates/ph2d-tool-painter/src/tool/paint/` · `WP` = `crates/ph2d-wet-paint/src/` ·
> `PL` = `crates/ph2d-panel-painter-layers/src/` · `AP` = `crates/ph2d-app-painter/src/`.

## §0 — Em três linhas

1. **Os 4 motores partilham UM carimbo** (`PB/stroke.rs` → `dab_build.rs::dab_from`, doc 47 §1): forma, grão, jitter, taper,
   cápsula, pressão, inclinação e rotação são do `BrushSpec` (`PB/spec.rs:62–593`) e valem nos quatro; muda o que cada meio
   faz com o dab (pousa cor · lavagem óptica · altura · fluido).
2. **Pergunta 2: NÃO.** O `.ph2d-brush.png`, o carregador e o navegador são só do Wet Paint (§2) — a lacuna nº 1 (CAP-22).
3. **Muita lei já existe sem porta**: a pressão/inclinação dos 3 meios antigos (W6; fora do Wet o traço lê
   `Dynamics::default()`), a rotação que segue o traço (`DabRotation::FollowTrajectory`, só pelo knob do Wet), os 24 modos de
   mistura (o Wet JÁ os lê por sessão), o HSB por dab/traço, as cores da imagem da ponta. **Duas linhas do §1 do catálogo
   estão erradas pelo código**: CAP-01 (o Wet TEM `dabRotation` = Follow; falta inércia/memória) e CAP-05 (o Wet TEM o modo
   do pincel, `TP/wetpaint.rs:608`).

## §1 — O que cada motor exprime hoje

### §1.1 Digital (`PaintMedia::Digital`, `TP/media.rs:28–39`) — o carimbo de `ph2d-painter-brush`

| parâmetro | o que faz · faixa | onde | UI |
|---|---|---|---|
| `radius_px`, `hardness`, `falloff`, `custom_falloff` | tamanho; dureza; 7 curvas + curva editável | `PB/spec.rs:62–85`, `falloff.rs:15`, `falloff_curve.rs:94` | Brush (`TP/brush_core_settings.rs:20–96`) |
| `shape` | ponta por padrão/imagem (30 `TextureKind`, doc 40 §2; Bristles; Image = silhueta); «Use Texture Colors» = a COR da imagem (`PB/stamp_color.rs:409`) | `PB/spec.rs:262` | secção Shape |
| `dab_flatten` · `dab_angle_deg` · `dab_quina` | elipse · ângulo fixo 0..360 · ponta RETA 0..1 (prompt 44) | `PB/spec.rs:290–297` | gizmo `PL/event/dab_gizmo.rs` |
| `pen_tilt.rotation` | `DabRotation` None · FollowTrajectory · PenTilt (0..2); sem barril («dono 08/10», `PB/pen.rs:9`) | `PB/pen.rs:11–31`, `PB/spec_frame.rs:12–30` | ⚠️ só o knob `dabRotation` do Wet |
| heading | EMA da tangente, `L = max(0,1·diâmetro, 3 px)` FIXO, recomeça a cada traço | `PB/heading.rs:23–68` | — |
| `spacing`, `continuous_spacing`, `stroke_method` | passo; dab-CÁPSULA (`PB/capsula.rs:21,208`); Dots/Airbrush/Anchored/Space/DragDot/Line/Arc/Ellipse | `PB/spec.rs:72,117`, `stroke_method.rs:17–40` | Stroke |
| `stabilizer`, `line_kind` | estabilizador; Speed (inércia) · Sketchy · Wire · Ribbon (mola: peso/atrito/gravidade `spec.rs:174–187`) · Rough · Dash | `PB/spec.rs:136–225`, `line_kind.rs:16` | Brush/Stroke |
| `jitter`, `jitter_scale/rotate/spacing`, unidade · `spray_count` | posição · tamanho · rotação · passo · N marcas/dab | `PB/spec.rs:80,212–216,320–344` | `PL/paint_stroke/jitter_card.rs:141–166` |
| `taper` | **só o INÍCIO** (`start`, `tip_start`, `opacity`), por caminho, ≤ 8 diâmetros | `PB/taper.rs:62–128` | `PL/paint_taper.rs` |
| `Dynamics` + `Pen` | pressão→tamanho (ON, piso 0,1) / →opacidade (OFF), curvas, piso/tecto, suavização, corda | `PB/dynamics.rs:11,30–44`, `PB/pen.rs:203` | ⚠️ **sem UI; fixo** (`TP/state_default.rs:77`); só o Wet o escreve (`TP/wetpaint_settings.rs:61–82`) |
| `pen_tilt` (Tilt) | lápis DEITADO: elipse no azimute, alisa o grão, atraso (37,5°→67,5°) | `PB/pen.rs:40–70` | ⚠️ só no Wet |
| `texture` + `grain_depth` | grão View (=«moving») · Tiled (=«texturized») · Random · Stencil; 7 tipos | `PB/texture.rs:56`, `spec.rs:248,252` | `PL/paint_texture.rs:138` |
| `color_jitter_*` | HSB por dab ou por traço | `PB/spec.rs:309–317`, `jitter.rs:86` | `PL/paint_brush_top.rs:223–241` |
| `blend` | **24 modos** (Mix…Luminosity, EraseAlpha, AddAlpha) | `PB/blend.rs:18–38` | `PL/paint_brush.rs:176,351` |
| `strength`, `flow`, `accumulate`, `airbrush_rate_s` · `grid_*` · `symmetry` | opacidade, fluxo, build-up, aerógrafo · grelha · simetria | `PB/spec.rs:68–123,228,350` | Brush/cartões |
| `PaintMode` | Paint · Smear · Blur · Clone · Mask · Knife | `TP/paint_mode.rs:6–60` | rádio |

Portões: `a_following_tip_turns_the_flattened_footprint_with_the_stroke`, `no_follower_ignores_the_heading_byte_identical`
(`PB/texture/tests.rs`), `jitter_rotate_makes_an_active_shape_rotate_per_dab`, `a_known_path_tapers_at_its_head_and_not_at_its_tail`,
`view_follows_the_dab_but_tiled_is_fixed_to_canvas`, `symmetry_disabled_paints_byte_identically`,
`a_taper_authored_in_one_medium_is_the_taper_in_all_four` (`TP/taper_media_tests.rs:26`).

### §1.2 Watercolor (lavagem óptica sobre o carimbo; campos em `PB/spec.rs`)

`depth` 0,1..8 (398, Beer–Lambert `Tᵢ = pigmentᵢ^(D·depth)`) · `fill` (394) · `opacity` (407) · orla `edge_gain` 0..8 /
`edge_spread` 1..48 px / `smooth_edges` (359–367) · `granulation` + papel `paper`/`paper_depth`/`paper_edge`/`granulation_use_paper`
(371, 419, 463–472) · K–M `pigment`/`pigment_mix` (385–388) · `warp` 0..48 (411) · `edge_flow` (416) · **`wet_smudge`, `wet_rewet`,
`wet_charge`, `wet_dilution`, `wet_pull`** (424–456: os nomes do Wet Mix do Procreate) · secagem 2..60 s
(`TP/watercolor_settings.rs:50–51`). Todos com knob (`ids/painter_watercolor.rs`); reserva, vidro K–M e fundo só código. A ponta de
imagem é cortada pelo disco (C3, mídia congelada). Portões: `watercolor_dry_tests`, `watercolor_fios_tests`, `vidro_tests`.

### §1.3 Impasto (a tinta tem ALTURA)

`impasto_depth` −1..1, `impasto_source` Uniform/Grain, `impasto_draw_to` Depth/Color/Both (`PB/spec.rs:499–526`, `height_modes.rs`) ·
`smoothing`, `body`, `plow`, `push` (530–560; Push = «fim da fila», B3) · luz ângulo/elevação 5..90/intensidade 0..2/cor
(`TP/impasto_settings.rs:19–26,381–404`) · material shine/roughness/**metallic**/wax (`PB/spec.rs:574–593`) · **10 ferramentas**
Deposit, Knife, Sculpt Smooth, Sharpen, Flatten, Scrape, Fill, Chisel, Layer, Inflate (`TP/impasto_tool.rs:41–53`) · settle e pool
só código. Não se achou cor recolhida pela Knife do Impasto (sonda antes de a contar).

### §1.4 Wet Paint (`ph2d-wet-paint`, ADR-0134; 109 knobs em `WP/tuning/defs.rs`, 44 no painel Tuning)

| grupo | knobs (`defs.rs`) |
|---|---|
| Paint | `pigmentPerDab` 0..2000 (47) · `paperGate` · `drag` · **`pickup`** 0..0,2 (97) · `tipClean` (149) · **`blendForce`** (159) · `wetLift` (591) · **`glaze`** (601, doc 42) · **`dust`** (611, doc 43) |
| Caneta (W6) | `pressureSize/Opacity/OpacityCurve` (621–641) · `pressureSizeCurve/Floor/Ceiling/Smoothing` · `pulledString` 0..400 · **`dabRotation` 0..2** (655) · `tiltStart/Full/Width/Length/Smooth/Lag` |
| Água/Física | `waterPerDab`, `waterCap`, `evaporation`, `rewet`, `retention`, `edgeDarkening`, `leveling`…`capillaryGate`, **`gravity`** (179–339) |
| Ferramentas | `eraser`, `dryer`, `blow`, `smear` (349–379); `WetTool` Paint/Smear/Blend/Wet/Dry/Blow (`TP/wetpaint_settings.rs:274`) — viaja no preset (`TP/wetpaint_preset.rs:180–195`) |
| Papel/relevo | `paperContrast/Fibres/Grooves`, `visualGrain`, **`emboss`**, `paperVisibility` (389–441) |
| Ext (neutras) | `extDiffusion`, **`extBackrun`**, `extFingering`, `extRivulets`, `extGranulation`, `extStaining`, `extDryBrush`… (452–577) |
| Fios (ocultos, no Shape) | `bristleCount` 0..4000, `bristleStrength/Size`, `felt`, `bristlePressure/Lift/Height/Remove/Wave*/Span*/Noise*/Oscillation/LaneWidth` (`WP/bristles_fios.rs`, `bristles_relevo.rs`) |
| Cápsula (ocultos) | `capsuleSpacing` 0,025, `capsuleMaxPx`, `capsuleRoundNearest`, `pigmentSpacing` |

O preset grava os 31 `CAMPOS_DO_PINCEL` (`TP/wetpaint_preset.rs:47`, incluindo `blend`, jitter×6, taper, shape, grão, quina) + o
mapa `engine`. **O `blend` é lido**: a sessão molhada compõe-se no modo do pincel (`TP/wetpaint.rs:608–616`,
`wetpaint/composite.rs:218`, `wetpaint/blend_tests.rs`). Fábrica: **28** presets depois da limpeza F (eram 31). Portões:
`cada_preset_parece_a_sua_midia`, `sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes`, `the_factory_presets_are_counted`,
`the_tilt_dial_drives_the_sim_vector`, `every_visible_knob_has_a_row`.

## §2 — Pergunta 2: os presets aceitam Digital/Watercolor/Impasto? **NÃO** (CAP-22)

- **Formato** (`crates/ph2d-brush-presets/src/`): PNG + `iTXt ph2d-brush` (JSON) + `phTx` (texturas), `SCHEMA = 1` (`modelo.rs:8`);
  `Parametros::media: String` (`modelo.rs:66`) é texto livre ⇒ **o formato aguenta outro meio sem subir de versão**.
- **Captura**: grava sempre `MEIO_DO_PRESET = "WetPaint"` (`TP/wetpaint_preset.rs:38,475`); `watercolor_*`/`impasto_*` estão em
  `CAMPOS_FORA` como `DO_MEIO` (`:105,136–137,154–168`).
- **Carga**: `wet_preset_aplicar_corpo` **força** `set_paint_media(PaintMedia::WetPaint)` (`TP/wetpaint_preset.rs:334–336`); o
  `Dynamics`/`PenTilt` do preset só é lido com o Wet armado (`TP/wetpaint_settings.rs:63,90`).
- **Navegador**: a biblioteca só abre com o Wet armado (`AP/painter_bridge_presets.rs:324–331`, de `AP/painter_bridge.rs:265`); a
  secção só se desenha no Wet — **a decisão mora em `PL/paint_brush_sections.rs:116`** (`if !meio(PaintMedia::WetPaint) { return y; }`).
  As pastas `assets/brushes/WetPainter/<Oil|Ink|Pencil|…>/` são sub-famílias do Wet, não `PaintMedia`.
- **Miniatura**: só no Wet (A15, «NA FILA», dono 17/10).

O que falta (L1): (a) `media` lido e respeitado, o desconhecido recusado com aviso (nunca cair no Digital calado); (b) captura e
aplicação POR MEIO: os campos do `BrushSpec` do meio + `PaintMode` + `Dynamics` + `PenTilt`, sem `engine`; (c) o navegador nos 4
meios (biblioteca aberta sem o Wet, agrupada por meio, clicar troca o meio); (d) a miniatura de cada meio (A15); (e) pastas irmãs de
`WetPainter/` (decisão técnica: nenhum caminho gravado muda).

## §3 — A lista canónica × o que temos (✅ há · ◐ parcial · ✗ falta · → lacuna)

| capacidade (origem) | | onde / o que falta |
|---|---|---|
| Spacing, streamline, jitter de posição (Pc Stroke path) | ✅ | Pulled String só no Wet →L2 |
| Fall-off: a tinta que acaba (Pc; Rb `paint_fadeout_length` 281/281) | ✗ | →L5 |
| Taper início · taper FIM / por pressão (Pc Taper, PS Fade) | ✅ · ✗ | →L5 |
| Roundness, ângulo, ponta reta, ponta de imagem | ✅ | |
| Rotação que SEGUE o traço (Pc; PS «Direction»; Rb FOLLOW_TRAJECTORY 97) | ◐ | instantânea, sem inércia nem memória, sem UI fora do Wet →L3 |
| Rotação de barril (Rb PEN_ROTATION 50; `force_rotation_mode` 135) | ✗ | →L3 |
| Azimute (Pc; PS Pen Tilt) | ◐ | `PenTilt` só no Wet →L2 |
| Scatter, count (Pc Shape) | ✅ | |
| Count jitter, flip X/Y, roundness jitter, «Randomized», scatter só perpendicular (PS) | ✗ | →L4 |
| Jitter de opacidade/fluxo (PS Transfer; Rb `splat_opacity_jitter` 14) · tamanho por partícula (Rb 33) | ✗ | →L4 |
| Grão moving × texturized, escala, rotação, profundidade (Pc Grain, PS Texture; CAP-27) | ✅ | |
| Modo do GRÃO, depth min/jitter, brilho/contraste (PS; Pc; Rb `texture_params`) | ✗ | →L9 |
| Dual brush (PS; Pc; Rb `dual_brush_*` 11; CAP-06) | ✗ | só uma chave num teste (`TP/wetpaint_preset/tests.rs:478`) →L6 |
| Modo do pincel (Pc; PS; Rb `glaze_rgb_blend_mode` ≠ NORMAL = 7; CAP-05) | ◐ | os 24 modos existem nos 2 meios; faltam presets e a medição →L7 |
| Glaze por traço: light/uniform/intense glaze (Pc; CAP-11) | ◐ | `accumulate`/`strength` são por dab; falta o tecto POR TRAÇO →L8 |
| Wet edges · burnt edges (Pc, PS) | ◐ · ✗ | a orla da Watercolor e o `edgeDarkening` do Wet; Digital nada →L8 |
| Blending com a cor de baixo · Wet mix (Pc; CAP-09/10) | ◐ | Wet (`pickup`, `blendForce`, Smear/Blend) ✅; Watercolor dilution/charge/pull ✅, **attack/grade** ✗; Digital ✗ →L10 |
| HSB por dab/traço (Pc; PS; CAP-07) | ✅ | |
| Cor secundária, pressão/tilt→cor (Pc; PS fg/bg) | ✗ | →L11 |
| Pressão→tamanho/opacidade, curvas (Pc Pencil; PS Transfer) | ◐ | lei W6 completa, fixa fora do Wet; pressão→FLUXO ✗ →L2 |
| Inclinação→tamanho/forma/grão · →opacidade (Pc; CAP-03) | ◐ · ✗ | só no Wet →L2 |
| Velocidade→tamanho/opacidade (Pc; Kr sensor; Rb 0/281; CAP-02) | ◐ | só `LineKind::Speed` →L12 |
| Build-up (PS airbrush) | ✅ | `StrokeMethod::Airbrush` |
| Mixer brush wet/load/mix/flow (PS) · Kr Color Smudge · Kr Bristle · CP Real Bristle | ✅ no Wet | Digital ✗ →L10 |
| Smudge/blender/borracha como pincel (Rb SMUDGE 6, BLEND 8, ERASER 10; CAP-09, CAP-23) | ✅ | Wet (`WetTool` no preset) e `EraseAlpha` com a ponta inteira; presets Digital →L1 |
| Ponta que se GASTA (PS erodible; CAP-04) | ✗ | →L17 |
| Pixel sem AA (CAP-16) | ✗ | →L18 |
| Ponta RGBA, image hose / sequência (Kr, PS; CAP-17) | ◐ | «Use Texture Colors» (cor da imagem) ✅; N imagens por sorteio/sequência ✗ →L19 |
| Escorridos (CAP-12) · capilaridade/franja (CAP-14) · backrun (CAP-15) · secar/levantar (CAP-26) | ✅ | Wet `gravity` (Wet Drip) · `capillary`/`extFingering`/orla · `extBackrun` (neutro — medir) · `WetTool::Dry`, `wetLift` |
| Poça onde a caneta abranda (CAP-13) | ◐ | por medir: o depósito é por passo, não por tempo (salvo Airbrush) →L12 |
| Espessura + mistura no MESMO traço (CAP-21; CP Thick Paint) | ◐ | o Wet tem relevo próprio (`emboss`, `bristleHeight`); Impasto e Wet são exclusivos (`set_paint_media`, `TP/media.rs:248`) →L20 |
| Metálico/brilho (CAP-18) | ✅ | Impasto `metallic`/`shine` |
| Sketch/Harmony (CAP-25) · padrão na tela (CAP-20) | ✅ | `LineKind::Sketchy/Wire` · grão Tiled |
| Hatching (Kr) · partículas com física (Kr, CP; CAP-24) · glitter (AR) · pattern pen (CP) · rolo (AR) | ◐/✗ | Stripes Tiled faz a trama fixa; a Ribbon é a mola; glitter = spray + cor + metallic por medir →L15 |
| Simetria, grelha, stencil, clone, blur | ✅ | |

## §4 — As LACUNAS

**Regra de todas (a cerca):** aditiva e desligada por omissão; no neutro os 3 meios antigos ficam ao byte
(`tests/it/os_outros_meios_nao_mudam.rs`, FNV da `sonda_gestos` a 28 px contra `ESPERADO`, fixtures
`docs/Painter/NewFeaturesWetPaint/fixtures/sonda_28/`), os presets também (`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`,
`sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes`), e `docs/Painter/NewFeaturesWetPaint/ferramentas/cerca.sh` verde (só linhas
ACRESCENTADAS em `watercolor_*`, `impasto_*`, `substrate_*`, `dynamics.rs`, `spec.rs`, `dab.rs`, `stamp_route.rs`…; isenção em
`cerca_isencoes.tsv` só com o número exacto). ⚠️ LOC: `PB/stroke.rs` **696/700**, `TP/stamp_color_cache.rs` 697 (C15) — tocar lá
começa por CORTE. Custo: **S** ≤ ~300 LOC/uma rota · **M** ~300–900 ou várias rotas/caches · **L** > 900 ou motor novo.

**L1 — Presets nos 4 meios + navegador e miniatura em todos** (CAP-22, §2). Pincéis: os ≈ 45 de motor ≠ Wet — A-1, A-2, A-3, A-5,
A-6, B-6, B-7, B-11, C-4, C-5, C-6, C-7, C-10, C-11, C-14, D-1, D-2, D-4, G-10, G-13, I-12, J-2, J-3, J-5, K-1…K-8, L-5, L-8, L-9, M-1,
M-4…M-7. Motor: `TP/wetpaint_preset.rs` partido num aplicador/capturador por meio (o do Wet ao bit), `AP/painter_bridge_presets.rs`,
`PL/paint_brush_sections.rs:116`, A15. **L** (~900–1 200 LOC). Portões: `um_preset_digital_volta_ao_bit` (capturar→gravar→carregar = a
cena da bateria ao byte), `o_meio_do_preset_e_o_meio_que_pinta` (desconhecido = recusa), `os_presets_do_wet_nao_mudam`, a contagem por
meio.

**L2 — A mão nos 3 meios antigos** (CAP-03 + pressão). A lei W6 existe; falta: `Dynamics` + `PenTilt` como campos do pincel em todos os
meios (gravados por L1), o MESMO cartão «Pen» do Wet nos 3 (a porta única `TP/wetpaint_settings.rs:61–100`, sem copiar a lei),
pressão→FLUXO e inclinação→opacidade. Pincéis: A-3, B-6, B-7, C-4, C-5, C-10, D-2 (e todo Dg com pressão). **S–M** (~350 LOC; o default
`size_pressure ON, size_min 0,1` = o neutro ao bit). Portões: `o_neutro_da_mao_e_o_traco_de_hoje` (rampa 0→1 da bateria nos 3 meios =
FNV de hoje), `a_pressao_digital_afina_como_a_do_wet`. ⛔ Recusa medida (doc 47): o piso que tira a mão do dono (0,24–0,86).

**L3 — A ponta que GIRA, com inércia e memória (C38) + barril** (CAP-01). Hoje Follow é instantâneo (L fixo 0,1·diâmetro), recomeça do
0° a cada traço e o P53 nem o usou (`dab_angle 0`). Rebelle: Flat Oily/Knife Smooth = PEN_ROTATION + `force_rotation_mode`; sem barril a
lâmina guarda a orientação entre traços e só roda com o arrasto — recto fino em qualquer direcção, curva larga (53 §4). Falta:
`rotation_lag` por pincel (o L do heading), a memória do último ângulo no tool, o evento `rotation` do `zwp_tablet_tool_v2` (ADR-0189)
quando a caneta o der, e a UI fora do Wet (com L2). Pincéis: A-4, B-9, D-3, E-3, F-2, F-6, G-5, G-6, G-7, G-8, H-2, I-8 (12; refaz o
Filbert e a Knife do veredito). **S** (~250 LOC; cuidado com as caches de orientação constante, `has_per_dab_rotation`, doc 47 §1).
Portões: `a_lamina_e_fina_no_recto_em_qualquer_direccao` (0/45/90/135° ± 5 %), `a_curva_alarga_com_a_inercia`,
`a_lamina_guarda_o_angulo_entre_tracos`; `no_follower_ignores_the_heading_byte_identical` intacto. Oráculo: Rebelle Flat Oily na bateria.

**L4 — O jitter completo** (CAP-08). Falta: opacidade e fluxo por dab, count jitter, flip X/Y, roundness jitter, ângulo inicial por traço,
scatter só perpendicular, tamanho por partícula do spray. Pincéis: I-11, J-2, J-3, J-5, K-1, K-2, K-3, K-5, G-4. Motor: `PB/jitter.rs`
(255 LOC, espaço) + `dab_build`; as caches constantes têm de declarar a variação. **S–M** (~400 LOC). Portões: neutro = ao bit por campo,
`o_jitter_de_opacidade_espalha_a_cobertura` (média preservada), mutação.

**L5 — O FIM do traço: taper de fim, taper por pressão, fall-off** (parte de CAP-19). O fim não se conhece em tempo real: lei do
Procreate (re-pintar a cauda curta ao levantar, dentro da transacção; prévia ≠ documento). Fall-off = a carga que se esgota pelo
comprimento (Digital: cobertura; Wet: `pigmentPerDab`). Pincéis: C-4, C-6, D-1, D-2 (fim) · G-1, G-8, G-9, G-11, I-11, C-11 (secar
ao longo). Motor: `PB/taper.rs` (`end`, `tip_end`), `PB/stroke.rs` (corte antes), o fim da transacção. **M** (~600 LOC: o desfazer e a
cápsula). Portões: `a_cauda_afila_como_a_cabeca`, `o_fall_off_esmaece_ao_longo`; no neutro o portão do taper de hoje intacto.
Antes: medir se `tipClean` do Wet já é um fall-off (recta longa da bateria).

**L6 — Dual brush** (CAP-06). Lei (PS/Pc/Rb, documentada): 2.ª ponta com tamanho/spacing/scatter/count/rotação próprios, MÁSCARA da 1.ª
por um modo (Multiply, Darken, Overlay, Height; Rb `dual_brush_layer_blend_mode`). Pincéis: C-9, C-11, C-14, D-4, F-3, G-11, K-9 (+ Rb
Dry Thin, Charcoal 2). Motor: `dual: Option<DualTip>` no `BrushSpec` (None = ao bit), ~10 rotas de carimbo + caches + GPU
(`crates/ph2d-paint-gpu/src/stamp.wgsl`) + secção nova. **L** (~1 000 LOC). Portões: `sem_dual_brush_o_carimbo_e_o_de_hoje`,
`o_dual_brush_so_tira` (Multiply: cobertura ≤ a da 1.ª em todo píxel), ms/quadro a 40 e 200 px.

**L7 — Modos de mistura nos pincéis de fábrica** (CAP-05). O motor JÁ mistura (§1.1, §1.4). Falta: medir o Marker/Effect do Rebelle
(Linear Burn, Color, Additive, Hue, Luminosity, Overlay, Saturation; a cena do 52 §A pintou 0 px sob o Wine ⇒ repetir sem o `DRY` entre
base e efeito) e fazer os presets. ⚠️ o modo é da SESSÃO molhada: medir o efeito sobre tinta ainda molhada. Pincéis: E-1…E-4, M-1, M-2,
M-3, K-4. **S** (presets + uma cena; motor só se a medição o pedir). Portão: `o_efeito_do_pincel_compoe_no_seu_modo` (base + efeito
contra a fórmula de cada modo).

**L8 — Tecto de opacidade POR TRAÇO, wet edges e burnt edges no Digital** (CAP-11). Um buffer do traço composto no fim (Pc
light/uniform/intense glaze), e as orlas como passagem sobre ele — a orla da Watercolor é a mesma conta: REUSAR a lei, não a copiar.
Pincéis: A-6, E-1…E-4, I-12, C-11. **M** (~700 LOC; rota nova no Digital, com o desfazer). Portões:
`o_tecto_do_traco_nao_escurece_no_proprio_cruzamento` (S que se cruza: máx = tecto), `sem_tecto_o_carimbo_e_o_de_hoje`.

**L9 — O grão: modo, profundidade mín/jitter, brilho/contraste.** Pincéis: B-6, B-7, B-11, F-3, F-7, K-8 (e os lápis/carvões Dg). Motor:
`PB/texture.rs` + rotas por píxel; LUT nas assadas. **S–M** (~400 LOC). Portões: o modo de hoje = neutro ao bit (nomeá-lo medindo),
`o_height_abre_o_grao_com_a_pressao`.

**L10 — Mistura com a cor de baixo fora do Wet** (CAP-09/10). Decisão técnica MEDINDO: os pincéis de mistura vão para o Wet (já pinta;
zero motor) e só se o EXAME §3.6 (ms/quadro a 200 px) o reprovar se faz um Color Smudge Digital (color rate + smudge length).
Attack/grade do Wet Mix faltam na Watercolor. Pincéis: G-12, G-13 (hoje Dg no catálogo ⇒ Wet), H-1, H-3. **S** (Wet) / **M** (Digital).

**L11 — Cor secundária e cor pela mão** (CAP-07). Pincéis: K-2, K-3, K-4, M-4, M-5. Motor: `PB/jitter.rs` + a cor do dab. **S** (~250
LOC). Portões: `sem_cor_secundaria_a_cor_e_a_de_hoje`, `a_pressao_leva_a_cor_a_secundaria`.

**L12 — Velocidade→tamanho/opacidade (e a poça onde abranda)** (CAP-02, CAP-13). Pincéis: C-4, C-6, D-1, D-2 · C-1, C-7, C-12. Motor:
`Dynamics` + a velocidade do `Stroke`. **S**. Portão: recto lento × rápido da bateria. Rb 0/281 ⇒ a lei vem do Krita/Procreate.

**L13 — As lanes do Jitter alto (C39)** — decisão de LEI medida no Dabber/Spatter do Rebelle (53 §6). Pincéis: G-4, I-11, J-3. **M**.
Portão: textura do Dabber + RSS a raio 300/1000.

**L15 — Nicho** (absorve a L14 «padrão ao longo do caminho», que nenhum pincel do catálogo pede): hatching (K-7; hoje Stripes Tiled), partículas com física (M-6, M-7; a Ribbon é a mola), glitter (M-5; spray + cor +
`metallic` — MEDIR a composição antes), pattern pen e rolo (fora do catálogo). **M** cada, só por prioridade do catálogo.

**L16 — As lâminas finas (C40)**: Cv da borda 0,004 → 0,056 a 4,8–6,0 px (a grelha do Wet). Pincéis: G-7, D-3, G-14. **M**. Depende de L3.

**L17 — A ponta que se GASTA** (CAP-04; PS erodible): raio/quina/achatamento que mudam com o caminho e a pressão, e se repõem.
Pincéis: B-11, F-4, F-7. **M** (estado da ponta no tool + campos no `BrushSpec`). Portão: `a_ponta_gasta_alarga_ao_longo`.

**L18 — Pixel sem AA** (CAP-16): carimbo binário alinhado à grelha de píxeis. Pincel: A-5. **S** (uma rota; o `hardness 1` de hoje ainda
tem AA). Portão: só valores 0/1 na camada; linha de 1 px sem degraus duplos (pixel-perfect).

**L19 — Ponta RGBA de N imagens** (CAP-17; image hose): sorteio/sequência entre N imagens, com a cor delas. Pincéis: K-2, K-3, K-4, K-5,
M-6. **M** (o `shape` é uma imagem; o formato já leva texturas `phTx`). Portão: `uma_imagem_e_o_carimbo_de_hoje`.

**L20 — Espessura + mistura no mesmo traço** (CAP-21): primeiro MEDIR se o relevo do Wet (`emboss`, `bristleHeight`) dá G-10/G-7; só se
não der, o Impasto passa a coexistir com o Wet (fim da exclusão em `TP/media.rs:248`). **S** (medir) / **L** (coexistir).

## §5 — A ORDEM (a fila da JANELA 2)

Critério: pincéis do catálogo que destrava × 1/custo, com a regra §0.10 (o equivalente num bloco).

| # | lacuna | CAP | custo | destrava | bloco |
|---|---|---|---|---|---|
| 1 | **L1** presets nos 4 meios + navegador + miniatura | 22 | L | ≈ 45 | **A** |
| 2 | **L2** a mão nos 3 meios | 03 | S–M | ≈ 10 + todo Dg com pressão | **A** (o mesmo campo e cartão que L1 grava) |
| 3 | **L3** a ponta que gira (C38) | 01 | S | 12 | — |
| 4 | **L7** modos de mistura nos presets | 05 | S | 8 | — |
| 5 | **L4** jitter completo | 08 | S–M | 9 | **C** |
| 6 | **L11** cor secundária e pela mão | 07 | S | 5 | **C** (`PB/jitter.rs`, o mesmo cartão) |
| 7 | **L6** dual brush | 06 | L | 7 | — |
| 8 | **L5** fim do traço + **L12** velocidade | 19, 02, 13 | M + S | 13 | **B** (um corte do `stroke.rs` serve os dois) |
| 9 | **L8** tecto por traço, wet/burnt edges | 11 | M | 6 | — |
| 10 | **L19** ponta RGBA de N imagens | 17 | M | 5 | — |
| 11 | L9 · L17 · L10 · L18 · L13 · L16 · L20 · L15 | — | S–L | por pedido | — |

**Antes de cada lacuna**: (1) o oráculo na bateria (`docs/Painter/Pinceis/ferramentas/bateria.json`); (2) CONFIRMAR NO CÓDIGO que falta
— esta leitura achou três «ausências» que existiam (`FollowTrajectory`, o `blend` do Wet, «Use Texture Colors»); (3) ler as `⛔ Recusas
MEDIDAS` dos docs 47 e 53 (o piso de pressão, `MAX_SMOOTHING` 0,95 = 95 ms, juntar as lanes por r + 2·alcance).

## §6 — Por confirmar (esta leitura NÃO provou)

- A Knife do Impasto recolhe cor? (só altura achada.) · `tipClean` do Wet já é um fall-off? · `extBackrun` dá o backrun de I-10/L-6?
- O custo de L1 na miniatura dos 3 meios (A15): a rota de cena existe na bancada (entrega D), não no painel.
- O `PenTilt` no Watercolor com «Automatic» (a elipse só com ele off, doc 47 §6) — L2 tem de o respeitar.
