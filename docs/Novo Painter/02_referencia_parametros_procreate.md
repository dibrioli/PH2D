# 02 — Referência definitiva: parâmetros do Brush Studio (Procreate)

> A **fonte definitiva** contra a qual implementamos. Os 14 painéis do Brush Studio e **cada controle**,
> com: nome exato (label do Procreate) · o que faz · tipo/range · a **chave do `Brush.archive` plist**
> (ground-truth do formato; do `procreate-brush-decoder` v1.7) · o **campo correspondente no nosso `Brush`**
> ([`crates/ph2d-painter-brush/src/`](../../crates/ph2d-painter-brush/src/)).
>
> Coluna **Modelo**: `✅` campo existe / `⚠️` existe mas diverge do Procreate / `❌` falta no nosso modelo.
> (Avaliação no dab pipeline e UI são rastreadas no [plano](03_plano_implementacao.md), não aqui.)

## Fatos estruturais

- **14 categorias** (a sidebar do Brush Studio): 1. Stroke Path · 2. Stabilization · 3. Taper · 4. Shape ·
  5. Grain · 6. Rendering · 7. Wet Mix · 8. Color Dynamics · 9. Dynamics · 10. Apple Pencil · 11. Properties ·
  12. Materials · 13. Preview · 14. About. **StreamLine vive em Stabilization, não em Stroke Path.**
- **Rendering = exatamente 6 modos** (4 Glaze + 2 Blending). Os sliders de Wet Mix são *gated* atrás dos 2 modos Blending.
- **Stamp vs Stroke jitter:** Stamp = por-dab (intra-pincelada); Stroke = por-pincelada (inter-pincelada).
- O `Brush.archive` é um **NSKeyedArchiver binary plist**; chaves reais vivem em `$objects[1]`. PNGs irmãos:
  `Shape.png`, `Grain.png` (Grain é invertido na importação). Container `.brush`/`.brushset` = ZIP.

---

## 1. Stroke Path
*"Spacing, jitter, e quão rápido a pincelada some."*

| Controle | O que faz | Tipo/range | plist key | Nosso campo | Modelo |
|---|---|---|---|---|---|
| Spacing | Quantas vezes o shape se carimba ao longo do path | slider 0–100% | `plotSpacing` (0–2.0) | `stroke_path.spacing` | ✅ |
| Spacing Jitter | Variabilidade do spacing (caótico em valores altos) | slider 0–100% | `plotSpacingJitter` (0–10) | `stroke_path.spacing_jitter` | ✅ |
| Jitter (Lateral) | Desloca carimbos perpendicular à pincelada | slider 0–200% | `plotJitter` | `stroke_path.jitter_lateral` | ✅ |
| Jitter (Linear) | Desloca carimbos na direção da pincelada | slider 0–200% | `plotJitterLongitudinal` | — | ❌ **falta** |
| Fall Off | Começa opaco e some ao longo da pincelada | slider | `dynamicsFalloff` (0–1) | `stroke_path.falloff` | ✅ |

---

## 2. Stabilization
*"Suaviza pinceladas enquanto você desenha."*

| Controle | O que faz | Tipo/range | plist key | Nosso campo | Modelo |
|---|---|---|---|---|---|
| StreamLine — Amount | Suavização forte e uniforme (inking/caligrafia) | slider 0–100% | `plotSmoothing` | `stabilization.streamline_amount` | ✅ |
| StreamLine — Pressure | Liga a suavização à pressão da caneta | slider 0–100% | `dynamicsPressureSmoothing` | `stabilization.streamline_pressure` | ✅ |
| Stabilization — Amount | % de estabilização (média móvel) | slider 0–100% | `plotMovingAverageStabilization` | `stabilization.stabilization` | ✅ |
| Motion Filtering — Amount | % de filtragem de movimento (FFT) | slider 0–100% | `plotFFTSmoothingAmount` | `stabilization.motion_filtering_amount` | ✅ |
| Motion Filtering — Expression | Devolve expressividade aos traços | slider 0–100% | `plotFFTSmoothingBias` | `stabilization.motion_filtering_expression` | ✅ |

---

## 3. Taper
*"Espessura e opacidade no início e fim da pincelada."* Procreate tem **dois sistemas independentes**:
**Pressure Taper** (Apple Pencil) e **Touch Taper** (dedo), + toggle Classic.

| Controle | O que faz | Tipo/range | plist key | Nosso campo | Modelo |
|---|---|---|---|---|---|
| Pressure Taper — comprimento (início/fim) | Slider duplo do comprimento do taper | slider duplo | `pencilTaperStartLength`/`pencilTaperEndLength` | `taper.taper_length_start`/`_end` | ✅ |
| Link Tip Sizes (pencil) | Espelha o ajuste ao mover qualquer handle | toggle | `pencilTaperSizeLinked` | `taper.taper_link_sizes` | ✅ |
| Size (pencil) | Severidade da transição grosso→fino | slider 0–100% | `pencilTaperSize` | `taper.taper_size_start`/`_end` | ✅ |
| Opacity (pencil) | Transparência nas pontas do taper | slider 0–100% | `pencilTaperOpacity` | `taper.taper_opacity_start`/`_end` | ✅ |
| Pressure | Usa o feedback de pressão p/ taper mais responsivo | slider 0–100% | `taperPressure` | `taper.taper_pressure_link` (bool) | ⚠️ é bool, Procreate é slider |
| Tip | Ponta fina (low) → ponta grossa (high) | slider | `pencilTaperShape` | — | ❌ **falta** |
| Tip Animation | Preview do efeito do taper | toggle | `pencilTipAnimation` | — | ❌ (preview-only) |
| Touch Taper — comprimento/size/opacity/tip | Equivalente p/ dedo (sem Pressure, sem Tip Animation) | sliders | `taperStartLength`/`taperEndLength`/`taperSize`/`taperOpacity`/`taperShape` | (compartilha os campos acima) | ⚠️ split pencil/touch não modelado |
| Classic | Volta ao taper de versões antigas | toggle | `taperVersion` (0/1) | — | ❌ **falta** |

> **Gap de modelo:** unificamos pencil+touch num só conjunto. O Procreate os mantém **separados**
> (a única diferença estrutural: touch não tem Pressure nem Tip Animation). + faltam `tip/shape` e `classic`.

---

## 4. Shape

| Controle | O que faz | Tipo/range | plist key | Nosso campo | Modelo |
|---|---|---|---|---|---|
| Shape Source | Imagem-base carimbada (importável; Shape Editor) | image source | `bundledShapePath` (`$null`=embedded) | `shape.shape_source` | ✅ |
| Input Style | Touch only / Azimuth / Azimuth+barrel roll | dropdown | `shapeAzimuth`+`shapeRoll` | `shape.shape_input_style` | ✅ |
| Rotation (Touch) | Rotação do shape vs direção do traço (−100..+100%) | slider ±100% | `shapeRotation` (−1..1) | `shape.shape_rotation_follow` (bool) | ⚠️ é bool, Procreate é slider ±100% |
| Scatter | Randomiza rotação a cada carimbo (independe da direção) | slider 0–200% | `shapeScatter` (0–2) | `shape.shape_scatter` | ✅ |
| Count | Nº de carimbos por ponto (até 16) | slider 1–16 | `shapeCount` | `shape.shape_count` (u32) | ✅ |
| Count Jitter | Varia o Count por ponto | slider 0–100% | `shapeCountJitter` | `shape.shape_count_jitter` | ✅ |
| Randomised | Randomiza a rotação no início da pincelada | toggle | `shapeRandomise` | `shape.shape_randomized` | ✅ |
| Flip X / Flip Y | Espelha o shape h/v | toggles | `shapeFlipXJitter`/`shapeFlipYJitter` | `shape.shape_flip_x`/`_y` | ✅ |
| Roundness (graph) — rotação base | Nó verde: rotação base do shape | graph 2D | `shapeAngle` (rad) | — | ❌ **falta** (ângulo base) |
| Roundness (graph) — squash | Nós azuis: achatamento (elipse) | graph 2D | `shapeRoundness` (0–1) | `shape.shape_roundness` | ✅ |
| Pressure (roundness) | Achata o shape pela pressão | slider 0–100% | `dynamicsPressureShapeRoundness` | `shape.shape_pressure_roundness` | ✅ |
| Tilt (roundness) | Achata o shape pelo tilt | slider 0–100% | `dynamicsTiltShapeRoundness` | `shape.shape_tilt_roundness` | ✅ |
| Roundness Vertical Jitter | Compressão vertical aleatória ao longo do path | slider 0–100% | `jitterShapeRoundness` | `shape.shape_vertical_jitter` | ✅ |
| Roundness Horizontal Jitter | Compressão horizontal aleatória | slider 0–100% | `jitterShapeRoundnessX` | `shape.shape_horizontal_jitter` | ✅ |
| Shape Filtering | No Filtering / Classic / Improved (antialias da borda) | dropdown | `shapeFilter`+`shapeFilterMode` | `shape.shape_filtering` | ✅ |

---

## 5. Grain
*Textura cinza sobreposta por pincelada, dentro do Shape.* Controles mudam com **Movement**.

| Controle | O que faz | Tipo/range | plist key | Nosso campo | Modelo |
|---|---|---|---|---|---|
| Grain Source | Bitmap de textura (cinza; Grain Editor + tiling) | image source | `bundledGrainPath` | `grain.grain_source` | ✅ |
| Movement (mode) | **Moving** (arrasta a textura) vs **Texturized** (estática atrás) | dropdown | `textureApplication` (0/1) | `grain.grain_behavior` | ✅ |
| Movement (slider) | *Só Moving:* carimbado/borrado → rolando/detalhado | slider 0–100% | `textureMovement` | `grain.grain_movement` | ✅ |
| Scale | Tamanho do grain dentro do shape | slider 0–100% | `textureScale` (0–16) | `grain.grain_scale` | ✅ |
| Zoom | *Só Moving:* Follow Size ↔ Cropped | slider | `textureZoom` | `grain.grain_zoom` (enum) | ✅ |
| Rotation | *Só Moving:* esfrega o grain pela mudança de direção | slider | `textureRotation` (−1..1) | `grain.grain_rotation` | ✅ |
| Depth | Força da textura sobre a cor-base | slider 0–100% | `grainDepth` | `grain.grain_depth` | ✅ |
| Minimum (Depth Min) | Piso de contraste (ativo com Depth Jitter) | slider 0–100% | `grainDepthMinimum` | `grain.grain_depth_min` | ✅ |
| Depth Jitter | *Só Moving:* oscila textura↔cor aleatoriamente | slider 0–100% | `grainDepthJitter` | `grain.grain_depth_jitter` | ✅ |
| Offset Jitter | *Só Moving:* desloca o ponto de aplicação (quebra tiling) | slider 0–100% | `textureOffsetJitter` | `grain.grain_offset_jitter` | ✅ |
| Blend Mode | Como o grain mistura com a cor-base (enum de blend) | dropdown | `grainBlendMode`+`grainBlendModeExtended` | `grain.grain_blend_mode` | ✅ |
| Brightness | Clareia/escurece o grain | slider ±75% | `textureBrightness` (−0.75..0.75) | `grain.grain_brightness` | ✅ |
| Contrast | Aumenta/diminui contraste do grain | slider ±100% | `textureContrast` (−1..1) | `grain.grain_contrast` | ✅ |
| Grain Filtering | No Filtering / Classic / Improved | dropdown | `textureFilter`+`textureFilterMode` | `grain.grain_filtering` | ✅ |

---

## 6. Rendering
*"Como as pinceladas interagem entre si."*

**Rendering Mode (dropdown — leve→pesado):** Light Glaze · Uniform Glaze · Intense Glaze · Heavy Glaze ·
Uniform Blending · Intense Blending. plist: `renderingRecursiveMixing`+`renderingModulatedTransfer`+`renderingMaxTransfer`.
Nosso campo: `rendering.rendering_mode` (`RenderingMode=6` **CONGELADO**, paridade exata). ✅

| Controle | O que faz | Tipo/range | plist key | Nosso campo | Modelo |
|---|---|---|---|---|---|
| Flow | Quanto de cor/textura flui do brush p/ o canvas | slider 0–100% | `dynamicsGlazedFlow` | `rendering.flow` | ✅ |
| Wet Edges | Suaviza/borra as bordas (sangramento de pigmento) | slider 0–100% | `wetEdgesAmount` (0–1) | `rendering.wet_edges` (bool) | ⚠️ é bool, Procreate é slider 0–100% |
| Burnt Edges | Efeito "color burn" nas bordas ao sobrepor | slider 0–100% | `burntEdgesAmount` | `rendering.burnt_edges` (bool) | ⚠️ é bool, Procreate é slider |
| Burnt Edges Mode | Blend mode do burnt edges | dropdown | `burntEdgesBlendMode`+ext | `rendering.burnt_edges_mode` | ✅ |
| Blend Mode (stroke) | Blend mode da pincelada inteira | dropdown | `blendMode`+`extendedBlend2` | `rendering.stroke_blend_mode_index` | ✅ |
| Luminance Blending | Mistura luminância em vez de cor (gamma correct) | toggle | `blendGammaCorrect` | `rendering.luminance_blending` | ✅ |
| Alpha Threshold | Pixel vira 100% opaco ou 100% transparente | toggle + amount | `alphaThreshold`+`alphaThresholdAmount` | `rendering.alpha_threshold` (f32) | ✅ |
| Classic (Combine) | Reverte o blend de dual brushes ao modo "Normal" (5.3) | toggle | (combine flag) | — | ❌ (dual brush não modelado) |

> `rendering.pigment_mode` (`PigmentMode`) e `rendering.accumulate`/`edge_intensity` são **nossos** (mistura
> de pigmento Mixbox + build-up vs wash); não têm chave Procreate 1:1 — são a base do nosso Wet Mix de cor.

---

## 7. Wet Mix *(núcleo do mixer-brush)*
Modelo: **Charge** = reservatório depositado no início (esgota ao arrastar; recarrega ao levantar/retocar).
**Dilution** = afina com água (transparência). **Attack** = quanto da tinta carregada gruda. **Pull** =
esfrega pigmento já no canvas. **Grade** = chunkiness da textura. Todos 0–100%.

| Controle | O que faz | plist key | Nosso campo | Modelo |
|---|---|---|---|---|
| Dilution | Água misturada na tinta (transparência) | `dynamicsMix` | `wet_mix.dilution` | ✅ |
| Charge | Tinta carregada no início (esgota ao arrastar) | `dynamicsLoad` | `wet_mix.load` | ✅ |
| Attack | Quanto de tinta gruda no canvas | `dynamicsPressureMix` | `wet_mix.attack` | ✅ |
| Pull | Força com que o brush puxa/esfrega tinta do canvas | `dynamicsWetAccumulation` | `wet_mix.pull` | ✅ |
| Grade | Chunkiness/contraste da textura do brush | `dynamicsMixSoftening` (−1..1) | `wet_mix.grade` | ✅ |
| Blur | Borra a tinta + espalhamento da pincelada | `dynamicsBlur` | `wet_mix.blur` | ✅ |
| Blur Jitter | Randomiza o blur por carimbo | `dynamicsBlurJitter` | `wet_mix.blur_jitter` | ✅ |
| Wetness Jitter | Randomiza quanta água mistura ao longo da pincelada | `dynamicsWetnessJitter` | `wet_mix.wetness_jitter` | ✅ |

> + `wet_mix.wet_mix_enabled` (nosso gate; no Procreate é implícito pelos 2 modos Blending).

---

## 8. Color Dynamics
*Muda cor aleatoriamente, ou desloca H/S/B por pressão/tilt.* % slider; 0% = off.

**Distinção-chave: Stamp** (randomiza cada carimbo, intra-pincelada) **vs Stroke** (uma mudança por
pincelada inteira). O brilho é **split em Lightness + Darkness** em Stamp/Stroke; Pressure usa "Brightness"
único; Tilt/Barrel usam "Lightness". Secondary Color = blend Primária↔Secundária (não shift HSB).

| Sub-seção | Eixos | plist keys | Nossos campos | Modelo |
|---|---|---|---|---|
| Stamp Color Jitter | Hue · Saturation · Lightness · Darkness · Secondary | `dynamicsJitterHue/Saturation/Lightness/Darkness`, `jitterSecondary` | `color_dynamics.stamp_hue_jitter`/`_saturation_jitter`/`_lightness_jitter`/`_darkness_jitter`/`_secondary_color`+`_secondary_amount` | ✅ |
| Stroke Color Jitter | Hue · Saturation · Lightness · Darkness · Secondary | `dynamicsJitterStroke*`, `jitterStrokeSecondary` | `color_dynamics.stroke_*` (idem) | ✅ |
| Color Pressure | Hue · Saturation · Brightness · Secondary | `dynamicsPressureHue/Saturation/Brightness/SecondaryColor` | `color_dynamics.color_pressure_hue/_saturation/_brightness/_secondary` | ✅ |
| Color Tilt | Hue · Saturation · Lightness · Secondary | `dynamicsTiltHue/Saturation/Brightness/SecondaryColor` | `color_dynamics.color_tilt_hue/_saturation/_brightness/_secondary` | ✅ |
| Color Barrel Roll *(Pencil Pro)* | Hue · Saturation · Lightness · Secondary | `dynamicsRollHue/Saturation/Brightness/SecondaryColor` | `color_dynamics.color_barrel_hue/_saturation/_brightness/_secondary` | ✅ |

> + `hue_dynamic_amount`/`saturation_dynamic_amount`/`brightness_dynamic_amount`/`secondary_dynamic_amount`
> (nossos amplificadores globais por-eixo).

---

## 9. Dynamics
*Mudanças pela velocidade do traço + jitter aleatório de size/opacity.*

| Controle | O que faz | Tipo/range | plist key | Nosso campo | Modelo |
|---|---|---|---|---|---|
| Speed — Size | Velocidade varia o tamanho (−100..+100%) | slider ±100% | `dynamicsSpeedSize` (−1..1) | `dynamics.speed_size` | ✅ |
| Speed — Opacity | Velocidade varia a opacidade | slider ±100% | `dynamicsSpeedOpacity` | `dynamics.speed_opacity` | ✅ |
| Speed — Spacing | Velocidade varia o spacing | slider 0–100% | `plotSpacingSpeed` | `dynamics.speed_spacing` | ✅ |
| Jitter — Size | Altera o size do carimbo ao acaso | slider 0–100% | `dynamicsJitterSize` | `dynamics.jitter_size` | ✅ |
| Jitter — Opacity | Altera a opacidade do carimbo ao acaso | slider 0–100% | `dynamicsJitterOpacity` | `dynamics.jitter_opacity` | ✅ |

---

## 10. Apple Pencil

| Controle | O que faz | Tipo/range | plist key | Nosso campo | Modelo |
|---|---|---|---|---|---|
| Pressure — curva de resposta | Editor de curva (x=pressão, y=efeito; até 6 nós) | curve | `dynamicsPressureOverall` (pts) | `pencil.pressure_curve` `[(f32,f32);8]` | ✅ |
| Pressure — Size | Pressão → tamanho da ponta | slider 0–100% | `dynamicsPressureSize` | `pencil.pressure_targets` (bitmask) | ⚠️ amount per-target não modelado |
| Pressure — Opacity | Pressão → opacidade | slider 0–100% | `dynamicsPressureOpacityTransfer` | `pencil.pressure_targets` | ⚠️ idem |
| Pressure — Flow | Pressão → quanta tinta deposita | slider 0–100% | `dynamicsPressureOpacity` | `pencil.pressure_targets` | ⚠️ idem |
| Pressure — Bleed | Pressão → sangramento nas bordas | slider 0–100% | `dynamicsPressureBleed` | `pencil.pressure_targets` | ⚠️ idem |
| Tilt — curva | (nós/curva do tilt) | curve | (tilt curve) | `pencil.tilt_curve` `[(f32,f32);8]` | ✅ |
| Tilt — Angle | Threshold em que o tilt engaja (0–90°, default 9°) | dial 0–90° | `dynamicsTiltAngle` | — | ❌ **falta** |
| Tilt — Opacity | Tilt → opacidade | slider 0–100% | `dynamicsTiltOpacity` | `pencil.tilt_targets` (bitmask) | ⚠️ amount per-target não modelado |
| Tilt — Gradation | Suaviza ao sombrear na diagonal | slider 0–100% | `dynamicsTiltGradation` | `pencil.tilt_targets` | ⚠️ idem |
| Tilt — Bleed | Tilt → sangramento | slider 0–100% | `dynamicsTiltBleed` | `pencil.tilt_targets` | ⚠️ idem |
| Tilt — Size | Tilt → espessura | slider 0–100% | `dynamicsTiltSize` | `pencil.tilt_targets` | ⚠️ idem |
| Tilt — Size Compression | Impede a textura crescer junto com o brush | toggle | `dynamicsTiltCompression` | — | ❌ **falta** |
| Barrel Roll *(Pencil Pro)* — Size/Opacity/Bleed | Rotação do Pencil Pro → size/opacity/bleed | sliders | `dynamicsRollSize/Opacity/Bleed` | `pencil.barrel_roll_curve`+`barrel_targets` | ⚠️ amount per-target não modelado |
| Hover — Outline/Pressure/Fill | Comportamento do hover (Pencil 2/Pro) | enums/toggles | `hoverOutline`/`hoverPressure`/`hoverFill` | `pencil.cursor_outline`/`hover_estimated_pressure`/`hover_fill` | ✅ |

> **Gap de modelo (Apple Pencil):** nosso `pencil` usa **bitmask de targets + curva compartilhada**, enquanto o
> Procreate tem **amount individual por target** (Size/Opacity/Flow/Bleed etc.) + **Tilt Angle** + **Size
> Compression**. Decisão de design a tomar no plano (manter encoding compacto vs expandir p/ paridade exata de sliders).

---

## 11. Properties

| Controle | O que faz | Tipo/range | plist key | Nosso campo | Modelo |
|---|---|---|---|---|---|
| Use Stamp Preview | Preview como carimbo único em vez de traço | toggle | `stamp` | (ver Preview §13) | ⚠️ vive em Preview |
| Orient to Screen | Orientação consistente com canvas vs tela | toggle | `oriented` | `properties.orient_to_screen` | ✅ |
| Preview Size | Tamanho do traço/carimbo na Brush Library | slider | `previewSize` | (ver Preview §13) | ⚠️ vive em Preview |
| Smudge Pull | Força default do smudge quando usado como Smudge tool | slider 0–100% | `dynamicsSmudgeAccumulation` | `properties.smudge_pull` | ✅ |
| Maximum Size | Limite superior do slider de size da sidebar | slider | `maxSize` (0–16 → 0–1600%) | `properties.max_size_px` | ✅ |
| Minimum Size | Limite inferior | slider | `minSize` | `properties.min_size_px` | ✅ |
| Maximum Opacity | Limite superior do slider de opacity | slider | `maxOpacity` | `properties.max_opacity` | ✅ |
| Minimum Opacity | Limite inferior | slider | `minOpacity` | `properties.min_opacity` | ✅ |

---

## 12. Materials *(3D / PBR — fora do escopo 2D, Procreate 5.2)*
Metallic (Amount/Source/Scale) + Roughness (Amount/Source/Scale) + Height. plist: `metallic*`/`roughness*`/`height*`.
**Não modelado** no nosso `Brush` (correto — é p/ pintura de modelo 3D). Deixar como deferred fora do escopo do Painter 2D.

---

## 13. Preview *(chrome do preview — não modelado como struct)*

| Controle | O que faz | plist key | Modelo |
|---|---|---|---|
| Use stamp preview | Mostra um carimbo em vez de traço | `stamp` | ❌ (chrome de preview) |
| Size | Tamanho do preview | `previewSize` | ❌ |
| Pressure Minimum | Eleva o piso de pressão mostrado | `previewPressureMinimum` | ❌ |
| Pressure Scale | Expande/contrai o range de pressão mostrado | `previewPressureScale` | ❌ |
| Wet Mix | Liga efeitos de Wet Mix no preview | `previewWetMixEnabled` | ❌ |
| Tilt Angle | Ângulo p/ efeitos de azimuth/barrel no preview | `previewTiltAngleOffset` | ❌ |

> Baixa prioridade: é só a renderização da miniatura na Brush Library. Avaliar uma 13ª sub-struct `PreviewParams`
> (ou dobrar em `properties`) quando a UI do Brush Studio existir.

---

## 14. About this Brush

| Controle | O que faz | plist key | Nosso campo | Modelo |
|---|---|---|---|---|
| Brush Title | Nome (display-only desde 5.4) | `name` | `about.name` | ✅ |
| Author / Made by | Nome do autor | `authorName` | `about.author` | ✅ |
| Signature | Assinatura à mão | (signature) | `about.signature_uri` | ✅ |
| Date Created | Auto-preenchido | `creationDate` | `about.created_at_ms` | ✅ |
| Create New Reset Point / Reset Brush | Checkpoint/baseline do brush | (reset) | `about.reset_points` | ✅ |

---

## Resumo dos gaps de modelo (o que falta/diverge no nosso `Brush`)

**Faltam (`❌`):**
1. `stroke_path` — **Jitter Linear** (`plotJitterLongitudinal`).
2. `taper` — **Tip/Shape** (`pencilTaperShape`), **Classic** (`taperVersion`), **split pencil↔touch**, Tip Animation.
3. `shape` — **Angle base** (`shapeAngle`, o nó de rotação do Roundness graph).
4. `pencil` — **Tilt Angle** (threshold), **Tilt Size Compression**; e **amounts per-target** (hoje bitmask+curva).
5. `rendering` — **Classic/Combine** de dual brush (dual brush inteiro não modelado).
6. **Preview** (categoria 13) — nenhuma struct.

**Divergem (`⚠️`):**
- `rendering.wet_edges`/`burnt_edges` são **bool**, Procreate são **sliders 0–100%** (`wetEdgesAmount`/`burntEdgesAmount`).
- `shape.shape_rotation_follow` é **bool**, Procreate é **slider ±100%** (`shapeRotation`).
- `taper.taper_pressure_link` é **bool**, Procreate é **slider** (`taperPressure`).
- `pencil` usa **bitmask de targets + curva compartilhada** em vez de sliders individuais por target.

**Mudar qualquer um desses no `Brush` toca o contrato congelado `Brush≤168`** → ADR-amendment + gate
`architecture_painter_contract_surface` (CLAUDE.md §6). Há **1 slot top-level de headroom** (cap ≤ 14, hoje 13).

## O que é distintivo do Procreate vs commodity (do conversor Procreate→Krita)

- **Commodity** (qualquer engine tem): size, spacing, scatter/count, roundness/angle/flip, dual/masked brush,
  grain com blend modes & scale, a matriz pressão/tilt/jitter, blend modes, color jitter.
- **Distintivo** (o conversor marca como `unsupported`): os **6 Rendering Modes** (Glaze/Blending), **Wet Mix
  como modelo de 6 eixos** com mistura **Mixbox**, **Moving/rolling grain**, **Burnt Edges**, o **sistema de
  taper duplo**, **StreamLine + FFT motion filtering**, **Luminance Blending**. **É aqui que mora o trabalho
  diferenciador — e é exatamente o que o nosso modelo já reservou campos.**

---

### Fontes
- Procreate Handbook — *Brush Studio / Brush Studio Settings*: https://help.procreate.com/procreate/handbook/brushes/brush-studio-settings
- `aumlette-lab/procreate-brush-decoder` — `procreate-brush-decoder-v1.7.json` (chaves do plist + fórmulas/ranges)
- `freyalupen/procreate-to-krita-brush-converter` — `procreate_to_kpp.py` (mapa supported/unsupported)
- Nosso modelo: [`crates/ph2d-painter-brush/src/`](../../crates/ph2d-painter-brush/src/) (`brush.rs` + os 12 sub-structs), ADR-0044.
