# 40 — O **Shape**: a casa dos pincéis, em todas as mídias (estado de 15/10)

> Para o próximo leitor: o que a secção Shape É depois do [plano 27](27_plano_shape_casa_dos_pinceis.md) (W1–W5) e dos
> prompts [36](36_PROMPT_janela_cor_do_shape_e_do_grain.md)/[38](38_PROMPT_rampa_do_shape_como_a_do_grain.md). O
> porquê medido de cada decisão vive no plano 27 §6 e nos handoffs; aqui fica a LEI e o portão que a prova.

## §1 — O que o Shape descreve

**A ponta**: o contorno (falloff, Taper, Flatten & Rotate, imagem) e a textura dentro dela. Vale igual nas cinco
mídias; o que muda por mídia é o LEITOR de cada linha (§4). A física (água, pigmento, papel) fica no Tuning do Wet Paint
e nas secções próprias — o Shape não a toca.

## §2 — As 30 formas (`TextureKind`, `ph2d-painter-brush/src/texture/kind.rs`, `COUNT = 30`)

| classe | tipos | parâmetros na secção |
|---|---|---|
| nenhuma | **None** | — (no Wet Paint = a ponta LISA, §3) |
| padrão procedural (24) | Noise · Checker · Voronoi · Stripes · Clouds · Distorted Noise · Magic · Marble · Musgrave · Wood · Stucci · Gradient · Grain · Crosshatch · Dots · Grid · Bricks · Waves · Chevron · Diamonds · Triangles · Hexagons · Scales · Weave | Contrast · Brightness + até 4 do tipo (`patterns/specs.rs`) |
| papel procedural (3) | Paper Cold Press · Paper Rough · Paper Hot Press | Contrast · Brightness |
| imagem | **Image** (PNG; «Use as Brush Shape» da Hierarchy) | Contrast · Brightness; silhueta pelo alfa |
| cerdas | **Bristles** (o 30.º, W1) | Count · Strength · Size · Felt |

- **Bristles é UM gerador só**: `ph2d_wet_paint::brush::build_bristle_texture` (o `ph2d-painter-brush` depende do
  motor), amostrado em **píxeis** (ladrilho 128 px que não escala com o tamanho — por desenho, plano 27 §4), nunca no
  carimbo cacheado. Disponível em todas as mídias: no Digital/Impasto é uma ponta de cerdas.
- A ordem do dropdown é a do discriminante (`shape_kind_options`, `paint_shape.rs`); os nomes, `ph2d-i18n`
  `paint_brush.texture_kind.*`.

## §3 — O papel de cada slot no **Wet Paint** (a rota `wetpaint/dab_route.rs`)

O motor recebe a ponta por três canais (`ph2d-wet-paint` `stamp_row_shaped`):

| canal | o quê | quem o enche |
|---|---|---|
| `sil` (silhueta) | contorno: falloff × pegada (Flatten & Rotate) | sempre; com **Image** também a imagem |
| `texv` (textura) | o que o dente do papel corta: `stamp = fall · texv · I`, `depósito = stamp − (1 − tooth)·gate` | **Bristles** → as cerdas (`texture::bristles_at`, em células, só o rotor por dab) · **Grain** armado → o grão (SUBSTITUI as cerdas, D7) · senão `1` = ponta **LISA** |
| `padrao` (lei B) | o padrão PROCEDURAL do Shape, multiplicado DEPOIS do portão: `depósito = (min(env·texv·I, 1) − g)·sv` | os 27 procedurais (`shape_value_at`); só a ferramenta Paint |

- **A lei B** (W3) fecha o Bug #47: o padrão deixa de ser comido pelo portão do papel; `None` no canal = `× 1.0`
  exacto ⇒ os redondos ao bit. A lei A (portão proporcional) está **recusada** (muda os redondos 25–190 %).
- A borracha apaga pela silhueta inteira; a máscara do Solid (C2) pousa pela União das silhuetas dos carimbos.
- O pincel Wet Paint de fábrica nasce com o Shape em **Bristles** e as cerdas do Wet Round
  (`WetKnobs::cerdas_no_shape`); um preset antigo (cerdas no mapa `engine`) chega com o Shape em Bristles; gravar já
  não escreve o mapa. O Tuning esconde os 4 knobs das cerdas (`KnobGroup::Hidden`).
- O motor tem UMA pegada, o disco: os `BrushShape::{Flat, Fan}` do porte JS saíram na W5 (sem chamador no produto).
- ⚠️ Aberto (lista 26 **B6**): a ponta lisa deposita 4,7–6,2× a tinta das cerdas de fábrica; sobre o grão dos
  pastéis o padrão rende 4–5× menos que sobre a lisa. Decisão do dono, com a prancha.

## §4 — O painel por mídia (`ph2d-panel-painter-layers/src/paint_shape*.rs`, lei em `inercia.rs`)

| linha | Digital | Impasto | Watercolor | Wet Paint |
|---|---|---|---|---|
| Automatic | — | — | ✓ (ligado esconde o resto) | — |
| Falloff · Taper · Flatten & Rotate · Texture | ✓ | ✓ | ✓ | ✓ |
| Use Texture Colors | ✓ | ✓ | esmaece (`CorDaTexturaSemLeitor`) | esmaece e recusa o clique |
| Follow (Off/Rake/Flow) | ✓, esmaece sem nada a girar (`FollowSemNada`) | idem | idem | idem |
| Angle · Offset · Size · parâmetros | com tipo ≠ None | idem | idem | Bristles esmaece com Grain armado ou ferramenta que não deposita (`CerdasDaAgua`) |
| Relief · Shine | ✓ (`impasto_applies`) | ✓ | — | — |
| **Shape Color** (§5) | ✓ | ✓ | ✓ | oculta (`shape_ramp_offered`; lista 26 B9) |

## §5 — Shape Color (prompts 36/38) — uma porta: `ph2d-painter-brush/src/ramp_alpha.rs`

- **Duas camadas**, as duas ligadas por omissão: **Texture Color** (em cima) = `rampa[valor da textura do Shape]` — um
  padrão procedural vira COR e deixa de recortar (a lei do Grain em cor); a imagem continua ponta — e **Falloff
  Color** = `rampa[cobertura]` (a lei do P36, ao byte). «Move Up» troca a ordem; **Blend** (24 modos) na de cima;
  **Opacity** por camada.
- **A lei do alfa**: sem menu Alpha — o alfa de cada cor da rampa É o alfa da tinta (`BRUSH_RAMP_ALPHA =
  RampAlphaMode::Strength`); as camadas empilham sobre o TRANSPARENTE (`compose_shape_ramp_layers`, alfa = alfa da cor ×
  Opacity): cor transparente em cima mostra a de baixo; onde nenhuma tem cor não há tinta.
- **Texture Color sem textura não pinta**: com o Shape em None o cartão esmaece e não aceita clique
  (`Dependente::TexturaSemForma`); a prévia da ponta pinta a mesma pilha que o motor.
- **B&W**: a rampa é o TOM; os dois cartões ficam e o tom é a luminância da pilha (`ensure_shape_ramp_lut`).
- Quem manda na rampa é quem a indexa (`RampIndex::{Grain, Shape}`, P36): a cor do Shape não desliga o Grain.

## §6 — Os portões (cada lei, a prova)

| lei | portão |
|---|---|
| W1: um gerador, o ladrilho do motor | `ph2d-painter-brush` `texture/bristles.rs` `o_ladrilho_do_shape_e_o_do_motor_ao_bit` |
| W2: o Wet Paint lê as cerdas do Shape | `ph2d-tool-painter` `tool/paint/tests/shape_bristles.rs` `o_wet_paint_le_as_cerdas_do_shape` |
| W2: migração dos presets | idem `os_presets_antigos_chegam_com_as_cerdas_no_shape` |
| fábrica ao byte (W2, W3, W5) | `tests/it/presets_de_fabrica.rs` `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` · `ph2d-wet-paint` `tests/it/fingerprint.rs` |
| W3: lei B | `wetpaint/lei_b_tests.rs` `o_padrao_do_shape_pinta_sobre_o_grao_dos_pasteis` · `…_sobre_a_ponta_lisa_nao_perde_os_vales` |
| W4 C2: Solid de carimbos na água | `wetpaint/mascara_tests.rs` `com_ponta_de_imagem_a_mancha_da_agua_e_de_carimbos` |
| W4 C1: painel por mídia | `ph2d-panel-painter-layers` `tests/it/seam_texture_colors.rs` `a_texture_colour_line_the_medium_never_reads_refuses_the_click` · `tests/it/censo_dos_controlos.rs` |
| Shape Color: segue a textura, camadas, alfa, B&W | `tool/paint/tests/rampa_do_shape.rs` (`a_texture_color_segue_a_textura_do_shape`, `as_camadas_misturam_na_ordem_escolhida`, `o_alfa_de_cada_cor_e_a_opacity_sao_a_tinta`, `com_bw_as_duas_camadas_fazem_o_tom`) · `tool/paint/tests/cor_do_shape.rs` `a_rampa_do_shape_em_cor_nao_desliga_o_grain` |
| prévia e cartão sem textura | `ph2d-panel-painter-layers` `src/paint_shape_ramp_tests.rs` `a_previa_da_ponta_pinta_as_camadas_sem_textura` · `as_duas_camadas_pintam_os_controlos_de_cada_estado` |

Sondas (`#[ignore]`, `--profile smoke`): `wetpaint/sonda_shape_casa.rs` — tinta por tipo e por preset (W0/W3), a
prancha do passo (P32) e a do B6 (`sonda_prancha_b6_lisa_e_cerdas`, composta por `ferramentas/prancha_b6.py`).

## §7 — Abertos que tocam o Shape (lista [26](26_ABERTOS.md))

B6 (a ponta lisa) · B9 (Shape Color no Wet Paint) · B12 (que rampa ganha) · C16 («Layer N Color» não esmaece na
água) · C23 (a imagem indexa o alfa, o Krita a luminância) · C24 (Continuous lê o valor no centro do dab) · biblioteca
de pontas em PNG (D5, depois).
