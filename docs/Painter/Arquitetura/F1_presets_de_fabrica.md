# F1 — Os pincéis de fábrica × os knobs (31/10)

> **Gerado** por `python3 -I docs/Painter/Arquitetura/ferramentas/presets_matriz.py /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` — não editar à mão; a fonte é o código e os PNG. Matriz completa: [`fotos/f1_presets/matriz.tsv`](fotos/f1_presets/matriz.tsv) · mapa de calor: [`fotos/f1_presets/mapa_de_calor.png`](fotos/f1_presets/mapa_de_calor.png).

**De onde vem cada número.** O JSON de cada preset está no chunk `iTXt` `ph2d-brush` do PNG (`crates/ph2d-brush-presets/src/png.rs:96-110`, zlib). Padrão de um knob do registo = o do **produto** (`WetKnobs::DEFAULT`, `tp/wetpaint_settings.rs` `painter_default_knobs`): é o que o preset recebe quando a chave falta (`tp/wetpaint_preset.rs:310-320` parte de `WetKnobs::DEFAULT`); difere da fábrica do motor (`wp/tuning/defs.rs`) em 7 knobs (Pigment 78, EdgeDarkening 12, PaperGate 0,4, Felt 0,03, BristleSize 2, BristleCount 2000, Emboss 0). Linhas da mão fora do registo: tuplas de `tp/mao.rs:90-125`. Chaves extra do mapa `engine` (`erase`, `glazeLayering`, `pigmentMixing`, `paintTexture`, `realOil`): `tp/wetpaint_preset.rs:97-109,325-392`. Campos `brush.*`: `BrushSpec::default` (`ph2d-painter-brush/src/spec_default.rs`) quando o campo é escalar de topo; nos aninhados, a MODA dos presets (marcado). «Painel não alcança» = o valor gravado sai de `min..max` (o `WetKnobs::set` PRENDE-O ao carregar — o motor nunca o recebe) ou o knob não tem escritor de UI (coluna «quem ESCREVE» do censo = «só preset»).

## (a) Pincéis por modo e categoria

| modo | n | categorias |
|---|---|---|
| Digital | **21** | Basic 8 · Effects 3 · Eraser 3 · Ink 2 · Marker 1 · Splatter 1 · Stamp 2 · Texture 1 |
| Watercolor | **1** | Watercolor 1 |
| WetPaint | **55** | Acrylic 1 · Airbrush 1 · Blend 4 · ExpressOil 1 · Ink 4 · Marker 2 · Oil 14 · Pastel 6 · Pencil 8 · Smear 5 · Water 1 · Watercolor 8 |
| Impasto | **0** | nenhum preset de fábrica (o `media` do formato só tem Digital/Watercolor/WetPaint em uso; o Impasto é `PaintMedia::Impasto` sem pasta em `assets/brushes`) |
| Composite | **0** | nenhum preset de fábrica (o `media` do formato só tem Digital/Watercolor/WetPaint em uso; o Composite não é um `media` de preset) |

Total: **77** ficheiros `*.ph2d-brush.png`.

## (b) Cada knob: quantos pincéis o tiram do padrão, faixa usada, alcance do painel

«fora» = pincéis cujo valor EFECTIVO difere do padrão (Wet Paint: ausente = padrão do produto; Digital/Watercolor só têm as linhas da mão no `engine`). «grav.» = presets que gravam a chave. Faixa = min..max dos valores gravados.

| chave | grupo | UI (censo) | faixa do painel | padrão | grav. | fora | faixa usada | painel não alcança |
|---|---|---|---|---|---|---|---|---|
| `pigmentPerDab` | Paint | Cartão: Pigment (`wetpaint_settings.rs:5 | 0..2000 | 78 | 55 | **44** | 2.8..2000 | — |
| `paperGate` | Paint | Tuning | 0..1.5 | 0.4 | 55 | **38** | 0..2.5 | FORA DA FAIXA (preso no load): Knife Stain=2.5 |
| `felt` | Hidden | só preset | 0..0.2 | 0.03 | 48 | **36** | 0..0.2 | sem painel (36 pincéis o põem) |
| `bristleCount` | Hidden | só preset | 0..4000 | 2000 | 48 | **14** | 300..4000 | sem painel (14 pincéis o põem) |
| `drag` | Paint | Tuning | 0..1 | 0.16 | 55 | **20** | 0..0.16 | — |
| `pickup` | Paint | Cartão: Pickup (`:571`) | 0..0.2 | 0.005 | 55 | **9** | 0..0.05 | — |
| `intensity` | Paint | Tuning | 0..4 | 1 | 55 | **0** | 1..1 | — |
| `bristleStrength` | Hidden | só preset | 0..4 | 1 | 49 | **16** | 0..2 | sem painel (16 pincéis o põem) |
| `bristleSize` | Hidden | só preset | 0.2..4 | 2 | 48 | **10** | 1..7 | FORA DA FAIXA (preso no load): Fan Brush=7, Express Oil=7; sem painel (10 pincéis o põem) |
| `spacing` | Hidden | só preset | 0.5..8 | 2 | 55 | **0** | 2..2 | — |
| `tipClean` | Paint | Tuning | 0..0.05 | 0.001 | 55 | **6** | 0.001..1 | FORA DA FAIXA (preso no load): Broad Nib=1, Marker Bullet=1, Marker Chisel=1 |
| `blendForce` | Hidden | Cartão (linha da ferramenta) (`tabelas.r | 0..1 | 0.8 | 55 | **11** | 0.3..1 | — |
| `gateSaturation` | Paint | Tuning | 100..6000 | 2000 | 55 | **0** | 2000..2000 | — |
| `waterPerDab` | Water | Tuning | 0..4 | 1 | 55 | **3** | 0.15..1.5 | — |
| `waterCap` | Water | Tuning | 1..30 | 8 | 55 | **1** | 8..30 | — |
| `evaporation` | Water | Cartão: Dry speed (`:572`) | 0..8 | 1 | 55 | **4** | 0.6..3 | — |
| `rewet` | Water | Tuning | 0..8 | 1 | 55 | **0** | 1..1 | — |
| `retention` | Water | Tuning | 0.9..1 | 0.995 | 55 | **1** | 0.97..0.995 | — |
| `edgeDarkening` | Water | Cartão: Edge (`:573`) | 0..200 | 12 | 55 | **42** | 0..200 | — |
| `baseEvaporation` | Water | Tuning | 0..20 | 2 | 55 | **0** | 2..2 | — |
| `leveling` | Hidden | Cartão: leveling (`tabelas.rs:129`) | 0..2 | 0.5 | 55 | **0** | 0.5..0.5 | — |
| `capillary` | Hidden | Cartão: capillary (`tabelas.rs:131`) | 0..1 | 0.2 | 55 | **2** | 0.05..0.2 | — |
| `brake` | Hidden | Cartão: brake (`tabelas.rs:132`) | 0..4 | 1.5 | 55 | **0** | 1.5..1.5 | — |
| `gravity` | Physics | Cartão: Gravity (`:574`) | 0..0.05 | 0.005 | 55 | **0** | 0.005..0.005 | — |
| `levelClamp` | Hidden | Cartão: level_clamp (`tabelas.rs:130`) | 0..1 | 0.2 | 55 | **0** | 0.2..0.2 | — |
| `viscosity` | Physics | Tuning | 0..2 | 0.2 | 55 | **0** | 0.2..0.2 | — |
| `maxVelocity` | Physics | Tuning | 0.2..3 | 1 | 55 | **0** | 1..1 | — |
| `projection` | Physics | Tuning | 0..1.5 | 0.7 | 55 | **0** | 0.7..0.7 | — |
| `brakeReach` | Physics | Tuning | 1..12 | 4 | 55 | **0** | 4..4 | — |
| `capillaryGate` | Physics | Tuning | 100..6000 | 2000 | 55 | **0** | 2000..2000 | — |
| `eraser` | Tools | Tuning | 0..1 | 0.4 | 55 | **0** | 0.4..0.4 | — |
| `dryer` | Hidden | Cartão (linha da ferramenta) (`tabelas.r | 0..1 | 0.32 | 55 | **0** | 0.32..0.32 | — |
| `blow` | Hidden | Cartão (linha da ferramenta) (`tabelas.r | 0..1 | 0.2 | 55 | **0** | 0.2..0.2 | — |
| `smear` | Hidden | Cartão (linha da ferramenta) (`tabelas.r | 0..1 | 0.8 | 55 | **6** | 0.2..1 | — |
| `paperContrast` | Paper | só preset (Paper fora do Tuning, `rows.r | 0.2..2.5 | 1 | 55 | **3** | 1..3 | FORA DA FAIXA (preso no load): Knife Stain=3; sem painel (3 pincéis o põem) |
| `paperFibres` | Paper | só preset (Paper fora do Tuning, `rows.r | 0..3 | 1 | 55 | **0** | 1..1 | — |
| `paperGrooves` | Paper | só preset (Paper fora do Tuning, `rows.r | 0..3 | 1 | 55 | **0** | 1..1 | — |
| `visualGrain` | Paper | só preset (Paper fora do Tuning, `rows.r | 0..3 | 1 | 55 | **0** | 1..1 | — |
| `emboss` | Paper | Tuning | 0..4 | 0 | 55 | **15** | 0..0.5 | — |
| `paperVisibility` | Paper | só preset (Paper fora do Tuning, `rows.r | 0..1.5 | 1 | 55 | **0** | 1..1 | — |
| `extDiffusion` | Hidden | Cartão: diffusion (`tabelas.rs:139`) | 0..0.25 | 0 | 55 | **1** | 0..0.01 | — |
| `extBackrun` | Hidden | Cartão: backrun (`tabelas.rs:134`) | 0..2 | 0 | 55 | **0** | 0..0 | — |
| `extFingering` | WetEffects | Tuning | 0..1 | 0 | 55 | **0** | 0..0 | — |
| `extRivulets` | WetEffects | Tuning | 4..60 | 15 | 55 | **0** | 15..15 | — |
| `extGranulation` | Hidden | só preset | 0..1 | 0 | 55 | **0** | 0..0 | — |
| `extStaining` | Hidden | Cartão: staining (`tabelas.rs:148`) | 0..1 | 0.5 | 55 | **0** | 0.5..0.5 | — |
| `extDryBrush` | Stroke | Tuning | 0..1 | 0 | 55 | **2** | 0..1 | — |
| `extWetSoften` | Hidden | Cartão: wet_soften (`tabelas.rs:153`) | 0..1 | 0 | 55 | **0** | 0..0 | — |
| `extRakeLight` | Hidden | só preset | 0..3 | 0 | 55 | **0** | 0..0 | — |
| `extValleyGran` | Hidden | só preset | 0..1 | 0 | 55 | **0** | 0..0 | — |
| `extWetSheen` | Hidden | só preset | 0..2 | 0 | 55 | **0** | 0..0 | — |
| `extGrainDither` | Hidden | só preset | 0..8 | 0 | 55 | **0** | 0..0 | — |
| `extEdgeTint` | Hidden | só preset | 0..1 | 0.5 | 55 | **0** | 0.5..0.5 | — |
| `wetLift` | Tools | Tuning | 0..1 | 0.25 | 55 | **1** | 0.25..0.7 | — |
| `glaze` | Hidden | Cartão: glaze (`tabelas.rs:149`) | 0..1 | 0 | 15 | **7** | 0..1 | — |
| `dust` | Hidden | Cartão: dust (`tabelas.rs:133`) | 0..1 | 0 | 1 | **1** | 0.8..0.8 | — |
| `pressureSize` | Paint | Tuning · mão (`mao.rs`) | 0..1 | 0.9 | 54 | **43** | 0..1 | — |
| `pressureOpacity` | Paint | Tuning · mão (`mao.rs`) | 0..1 | 0 | 66 | **57** | 0..1 | — |
| `pressureOpacityCurve` | Paint | Tuning · mão (`mao.rs`) | 0.25..4 | 1 | 62 | **34** | 0.5..4 | — |
| `pressureSizeCurve` | Paint | Tuning · mão (`mao.rs`) | 0.25..4 | 1 | 58 | **30** | 0.25..3 | — |
| `pressureFloor` | Paint | Tuning · mão (`mao.rs`) | 0..0.9 | 0 | 24 | **2** | 0..0.3 | — |
| `pressureCeiling` | Paint | Tuning · mão (`mao.rs`) | 0.1..1 | 1 | 22 | **1** | 0.8..1 | — |
| `pressureSmoothing` | Paint | Tuning · mão (`mao.rs`) | 0..0.9 | 0 | 22 | **0** | 0..0 | — |
| `pulledString` | Paint | Tuning · mão (`mao.rs`) | 0..400 | 0 | 22 | **0** | 0..0 | — |
| `dabRotation` | Paint | Tuning · mão (`mao.rs`) | 0..2 | 0 | 33 | **6** | 0..1 | — |
| `tiltStart` | Paint | Tuning · mão (`mao.rs`) | 0..90 | 37.5 | 27 | **5** | 37.5..39 | — |
| `tiltFull` | Paint | Tuning · mão (`mao.rs`) | 0..90 | 67.5 | 27 | **5** | 45..67.5 | — |
| `tiltWidth` | Paint | Tuning · mão (`mao.rs`) | 0.25..5 | 1 | 27 | **5** | 1..1.05 | — |
| `tiltLength` | Paint | Tuning · mão (`mao.rs`) | 0.25..5 | 1 | 27 | **5** | 1..3.3 | — |
| `tiltSmooth` | Paint | Tuning · mão (`mao.rs`) | 0..1 | 0 | 27 | **0** | 0..0 | — |
| `tiltLag` | Paint | Tuning · mão (`mao.rs`) | 0..500 | 0 | 27 | **5** | 0..170 | — |
| `bristlePressure` | Bristles | Tuning | 0..3 | 0 | 17 | **17** | 0.3..3 | — |
| `bristleNoiseLength` | Bristles | Tuning | 10..2000 | 150 | 2 | **2** | 30..2000 | — |
| `bristleLaneWidth` | Bristles | Tuning | 0.5..24 | 3 | 17 | **16** | 1..4 | — |
| `bristleWave` | Bristles | Tuning | 0..0.75 | 0 | 0 | **0** | —..— | — |
| `bristleWaveLength` | Bristles | Tuning | 50..2000 | 500 | 0 | **0** | —..— | — |
| `bristleOscillation` | Bristles | Tuning | 0..12 | 0 | 1 | **1** | 1..1 | — |
| `bristleSpan` | Bristles | Tuning | 0..12 | 0 | 1 | **1** | 2..2 | — |
| `bristleSpanLength` | Bristles | Tuning | 1..200 | 10 | 1 | **1** | 20..20 | — |
| `bristleNoiseHue` | Bristles | Tuning | 0..0.5 | 0 | 0 | **0** | —..— | — |
| `bristleNoiseSat` | Bristles | Tuning | 0..0.5 | 0 | 14 | **14** | 0.05..0.3 | — |
| `bristleNoiseLight` | Bristles | Tuning | 0..0.7 | 0 | 15 | **9** | 0..0.25 | — |
| `bristleLift` | Bristles | Tuning | 0..2 | 0 | 16 | **10** | 0..1.5 | — |
| `bristleHeight` | Bristles | Tuning | 0..8 | 0 | 14 | **11** | 0..0.75 | — |
| `bristleRemove` | Bristles | Tuning | 0..1 | 0.5 | 0 | **0** | —..— | — |
| `pigmentSpacing` | Hidden | só preset | 0..0.5 | 0.025 | 16 | **16** | 0..0.01 | sem painel (16 pincéis o põem) |
| `capsuleSpacing` | Hidden | só preset | 0.005..0.1 | 0.025 | 0 | **0** | —..— | — |
| `capsuleMaxPx` | Hidden | só preset | 0..64 | 0 | 14 | **14** | 2..4 | sem painel (14 pincéis o põem) |
| `capsuleRoundNearest` | Hidden | só preset | 0..1 | 0 | 15 | **15** | 1..1 | sem painel (15 pincéis o põem) |
| `embossSizeExp` | Hidden | só preset | 0..1 | 0 | 6 | **6** | 0.4..0.4 | sem painel (6 pincéis o põem) |
| `embossSizeRef` | Hidden | só preset | 1..1000 | 41 | 6 | **0** | 41..41 | — |
| `bristleTipRagged` | Bristles | Tuning | 0..1 | 0 | 8 | **7** | 0..1.6 | FORA DA FAIXA (preso no load): Filbert Dry=1.6, Filbert=1.6, Flat Bristle=1.6, Thick Impasto=1.6, Tilted Shade=1.6 |
| `levelingOwnStroke` | Stroke | Tuning | 0..1 | 0 | 15 | **15** | 1..1 | — |
| `tipReservoir` | Hidden | Cartão: tip_pickup (`tabelas.rs:141`) | 0..1 | 0 | 4 | **4** | 0.2..0.4 | — |
| `pressureGrain` | Stroke | Tuning | 0..1 | 0 | 5 | **3** | 0..0.5 | — |
| `strokeFilm` | Stroke | Tuning | 0..1 | 0 | 2 | **2** | 1..1 | — |
| `dustDrop` | Stroke | Tuning | 1..32 | 2 | 0 | **0** | —..— | — |
| `bristleStreak` | Bristles | Tuning | 0..1 | 0 | 0 | **0** | —..— | — |
| `bristleStreakLight` | Bristles | Tuning | 0..1 | 0 | 0 | **0** | —..— | — |
| `bristleStreakLength` | Bristles | Tuning | 10..4000 | 300 | 0 | **0** | —..— | — |
| `bristleStreakCut` | Bristles | Tuning | 0..0.95 | 0.5 | 0 | **0** | —..— | — |
| `thinStrokeFade` | Hidden | só preset | 0..4 | 0 | 0 | **0** | —..— | — |
| `thinStrokeRef` | Hidden | só preset | 1..400 | 40 | 0 | **0** | —..— | — |
| `waterPush` | Tools | Tuning | 0..1 | 0 | 1 | **1** | 0.35..0.35 | — |
| `dryerEdge` | Tools | Tuning | 0..1 | 0 | 0 | **0** | —..— | — |
| `wetOpen` | Hidden | Cartão: wet_bloom (`tabelas.rs:140`) | 0..1 | 0 | 2 | **2** | 0.15..0.42 | — |
| `wetOpenPush` | WetEffects | Tuning | 0..1 | 0.8 | 2 | **2** | 0.4..0.5 | — |
| `smearWater` | Hidden | Cartão (linha da ferramenta) (`tabelas.r | 0..1 | 0.5 | 1 | **1** | 0.9..0.9 | — |
| `knifeRidge` | Stroke | Tuning | 0..16 | 0 | 6 | **4** | 0..8 | — |
| `textureSizeExp` | Stroke | Tuning | 0..1.5 | 0 | 1 | **1** | 0.5..0.5 | — |
| `textureSizeRef` | Stroke | Tuning | 1..1000 | 41 | 0 | **0** | —..— | — |
| `metallic` | Material | Tuning | 0..1 | 0 | 1 | **1** | 0.2..0.2 | — |
| `roughness` | Material | Tuning | 0..1 | 0.5 | 1 | **1** | 0.4..0.4 | — |
| `iorLevel` | Material | Tuning | 0..1 | 0.5 | 0 | **0** | —..— | — |
| `coat` | Material | Tuning | 0..1 | 0 | 1 | **1** | 0.3..0.3 | — |
| `coatRoughness` | Material | Tuning | 0..1 | 0.03 | 0 | **0** | —..— | — |
| `sheen` | Material | Tuning | 0..1 | 0 | 0 | **0** | —..— | — |
| `subsurface` | Material | Tuning | 0..1 | 0 | 0 | **0** | —..— | — |
| `subsurfaceRadius` | Material | Tuning | 0..16 | 4 | 0 | **0** | —..— | — |
| `transparency` | Material | Tuning | 0..1 | 0 | 0 | **0** | —..— | — |
| `lockMaterial` | Material | Tuning | 0..1 | 0 | 0 | **0** | —..— | — |
| `knifeLevel` | Stroke | Tuning | 0..1 | 0 | 1 | **1** | 1..1 | — |
| `textureSizeExpAcross` | Stroke | Tuning | -0.01..1.5 | -0.01 | 0 | **0** | —..— | — |
| `transmission` | Material | Tuning | 0..1 | 0 | 0 | **0** | —..— | — |
| `paperRelief` | Material | Tuning | 0..1 | 0 | 0 | **0** | —..— | — |
| `transmissionMode` | Material | Tuning | 0..1 | 0 | 0 | **0** | —..— | — |
| `oilDepth` | Material | Tuning | 0..3 | 1 | 10 | **10** | 0.1..0.9 | — |
| `oilGloss` | Material | Tuning | 0..1 | 1 | 4 | **4** | 0.1..0.5 | — |
| `wetOpenEdge` | WetEffects | Tuning | 0..80 | 0 | 1 | **1** | 28..28 | — |
| `strokeFilmDome` | Stroke | Tuning | 0..1 | 0 | 2 | **2** | 0.3..0.5 | — |
| `strokeFilmEnds` | Stroke | Tuning | 0..1 | 0 | 2 | **2** | 1..1 | — |
| `dryerLevel` | Tools | Tuning | 0..2 | 0 | 0 | **0** | —..— | — |
| `blendReach` | Tools | Tuning | 0..1 | 0 | 0 | **0** | —..— | — |
| `wetCover` | Hidden | Cartão: cover (`tabelas.rs:150`) | 0..1 | 0 | 1 | **1** | 1..1 | — |
| `bristleLaneSizeExp` | Bristles | Tuning | 0..1.5 | 0 | 1 | **1** | 0.8..0.8 | — |
| `paperPress` | Material | Tuning | 0..1 | 1 | 0 | **0** | —..— | — |
| `rewetPaint` | Hidden | Cartão: rewet (`tabelas.rs:143`) | 0..1 | 0 | 0 | **0** | —..— | — |
| `solubility` | Hidden | Cartão: solubility (`tabelas.rs:145`) | 0..1 | 0 | 0 | **0** | —..— | — |
| `rewetDrag` | Hidden | Cartão: rewet_drag (`tabelas.rs:144`) | 0..1 | 0 | 0 | **0** | —..— | — |
| `edgeSoftMass` | Water | Tuning | 0..8 | 0 | 1 | **1** | 4..4 | — |
| `dryCover` | Hidden | Cartão: dry_cover (`tabelas.rs:152`) | 0..1 | 0 | 1 | **1** | 1..1 | — |
| `tuftArc` | Bristles | Tuning | 0..1 | 0 | 1 | **1** | 0.8..0.8 | — |
| `waterDrag` | Hidden | Cartão (linha da ferramenta) (`tabelas.r | 0..1 | 0 | 1 | **1** | 0.8..0.8 | — |
| `opaque` | Hidden | só preset | 0..1 | 0 | 3 | **3** | 1..1 | sem painel (3 pincéis o põem) |
| `tiltOpacity` | Mão (fora do registo) | cartão Mão | 0..1 | 0 | 21 | **0** | 0..0 | — |
| `speedSize` | Mão (fora do registo) | cartão Mão | 0..1 | 0 | 22 | **1** | 0..0.025 | — |
| `pressureSCurve` | Mão (fora do registo) | cartão Mão | 0..1 | 0 | 0 | **0** | —..— | — |
| `pressureJitter` | Mão (fora do registo) | cartão Mão | 0..1 | 0 | 1 | **1** | 0.35..0.35 | — |
| `speedLag` | Mão (fora do registo) | cartão Mão | 0..0.5 | 0 | 0 | **0** | —..— | — |
| `pressureHardness` | Mão (fora do registo) | cartão Mão | 0..1 | 0 | 1 | **1** | 0.15..0.15 | — |
| `speedHardness` | Mão (fora do registo) | cartão Mão | 0..0.3 | 0 | 0 | **0** | —..— | — |
| `speedOpacity` | Mão (fora do registo) | cartão Mão | 0..1 | 0 | 21 | **0** | 0..0 | — |
| `erase` | engine extra | slider Erase do cartão (WetKnobs::DEFAUL | 0..1 | 0.4 | 55 | **0** | 0.4..0.4 | — |
| `glazeLayering` | engine extra | bool; ausente = OFF | 0..1 | 0 | 0 | **0** | —..— | — |
| `pigmentMixing` | engine extra | bool; ausente = ON (K-M) | 0..1 | 1 | 3 | **3** | 0..0 | — |
| `paintTexture` | engine extra | bool; ausente = OFF | 0..1 | 0 | 0 | **0** | —..— | — |
| `realOil` | engine extra | bool; ausente = OFF | 0..1 | 0 | 14 | **14** | 1..1 | — |

### Campos fora do mapa `engine` (volumes, topo, `brush.*`)

| campo | padrão (fonte) | grav. | fora | faixa usada |
|---|---|---|---|---|
| `volumes.opacity` | 1 (ausente = o de agora; referência 1,0) | 76 | **10** | 0.26..1 |
| `brush.accumulate` | 0 (spec_default.rs) | 22 | **4** | 0..1 |
| `brush.airbrush_rate_s` | 0.1 (spec_default.rs) | 22 | **0** | 0.1..0.1 |
| `brush.blend` | Mix (moda (sem padrão escalar no código)) | 77 | **7** | —..— |
| `brush.blend_sobre_vazio` | 0 (spec_default.rs) | 22 | **3** | 0..1 |
| `brush.color_jitter_enabled` | 0 (spec_default.rs) | 77 | **0** | 0..0 |
| `brush.color_jitter_hue` | 0 (spec_default.rs) | 77 | **0** | 0..0 |
| `brush.color_jitter_sat` | 0 (spec_default.rs) | 77 | **0** | 0..0 |
| `brush.color_jitter_scope` | PerDab (moda (sem padrão escalar no código)) | 29 | **0** | —..— |
| `brush.color_jitter_val` | 0 (spec_default.rs) | 77 | **0** | 0..0 |
| `brush.continuous_spacing` | 0 (spec_default.rs) | 77 | **58** | 0..1 |
| `brush.cor_mao.pressure_hue` | 0 (moda (sem padrão escalar no código)) | 22 | **0** | 0..0 |
| `brush.cor_mao.pressure_sat` | 0 (moda (sem padrão escalar no código)) | 22 | **0** | 0..0 |
| `brush.cor_mao.pressure_secondary` | 0 (moda (sem padrão escalar no código)) | 22 | **0** | 0..0 |
| `brush.cor_mao.pressure_val` | 0 (moda (sem padrão escalar no código)) | 22 | **0** | 0..0 |
| `brush.cor_mao.secondary[0]` | 1 (moda (sem padrão escalar no código)) | 22 | **0** | 1..1 |
| `brush.cor_mao.secondary[1]` | 1 (moda (sem padrão escalar no código)) | 22 | **0** | 1..1 |
| `brush.cor_mao.secondary[2]` | 1 (moda (sem padrão escalar no código)) | 22 | **0** | 1..1 |
| `brush.cor_mao.secondary_jitter` | 0 (moda (sem padrão escalar no código)) | 22 | **0** | 0..0 |
| `brush.custom_falloff` | [{"handle": "Auto", "id": 0, " (moda (sem padrão escalar no código)) | 77 | **3** | —..— |
| `brush.dab_angle_deg` | 0 (spec_default.rs) | 77 | **19** | 0..172 |
| `brush.dab_curve` | [[0.0, 0.0, 0.0, 0.0, 0.0, 0.0 (moda (sem padrão escalar no código)) | 77 | **0** | —..— |
| `brush.dab_flatten` | 0 (spec_default.rs) | 77 | **19** | 0..0.9 |
| `brush.dab_jitter.across` | 0 (moda (sem padrão escalar no código)) | 26 | **1** | 0..0.9 |
| `brush.dab_jitter.angle_per_stroke` | 0 (moda (sem padrão escalar no código)) | 26 | **0** | 0..0 |
| `brush.dab_jitter.count` | 0 (moda (sem padrão escalar no código)) | 26 | **3** | 0..0.6 |
| `brush.dab_jitter.flip_x` | 0 (moda (sem padrão escalar no código)) | 26 | **1** | 0..0.5 |
| `brush.dab_jitter.flip_y` | 0 (moda (sem padrão escalar no código)) | 26 | **0** | 0..0 |
| `brush.dab_jitter.opacity` | 0 (moda (sem padrão escalar no código)) | 26 | **2** | 0..0.1 |
| `brush.dab_jitter.particle` | 1 (moda (sem padrão escalar no código)) | 26 | **3** | 0.25..1 |
| `brush.dab_jitter.roundness` | 0 (moda (sem padrão escalar no código)) | 26 | **0** | 0..0 |
| `brush.dab_quina` | 0 (spec_default.rs) | 43 | **20** | 0..1 |
| `brush.dash_ratio` | 1 (spec_default.rs) | 22 | **0** | 1..1 |
| `brush.dash_samples` | 20 (spec_default.rs) | 22 | **0** | 20..20 |
| `brush.edge_to_edge` | 0 (spec_default.rs) | 22 | **0** | 0..0 |
| `brush.falloff` | Smooth (moda (sem padrão escalar no código)) | 77 | **22** | —..— |
| `brush.flow` | 1 (spec_default.rs) | 22 | **5** | 0.012..1 |
| `brush.grain_depth` | 1 (spec_default.rs) | 77 | **1** | 0.3..1 |
| `brush.grid_cell_px[0]` | 32 (moda (sem padrão escalar no código)) | 22 | **0** | 32..32 |
| `brush.grid_cell_px[1]` | 32 (moda (sem padrão escalar no código)) | 22 | **0** | 32..32 |
| `brush.grid_fit` | 0 (spec_default.rs) | 22 | **0** | 0..0 |
| `brush.grid_offset_px[0]` | 0 (moda (sem padrão escalar no código)) | 22 | **0** | 0..0 |
| `brush.grid_offset_px[1]` | 0 (moda (sem padrão escalar no código)) | 22 | **0** | 0..0 |
| `brush.hardness` | 0 (spec_default.rs) | 77 | **48** | 0..1 |
| `brush.input_samples` | 1 (spec_default.rs) | 77 | **0** | 1..1 |
| `brush.jitter` | 0 (spec_default.rs) | 77 | **20** | 0..1 |
| `brush.jitter_absolute_px` | 0 (spec_default.rs) | 77 | **0** | 0..0 |
| `brush.jitter_rotate` | 0 (spec_default.rs) | 77 | **2** | 0..1 |
| `brush.jitter_scale` | 0 (spec_default.rs) | 77 | **4** | 0..0.9 |
| `brush.jitter_spacing` | 0 (spec_default.rs) | 77 | **0** | 0..0 |
| `brush.jitter_unit` | Brush (moda (sem padrão escalar no código)) | 77 | **0** | —..— |
| `brush.line_kind` | None (moda (sem padrão escalar no código)) | 22 | **0** | —..— |
| `brush.pigment` | 0 (spec_default.rs) | 77 | **1** | 0..1 |
| `brush.pigment_mix` | 0.5 (spec_default.rs) | 77 | **1** | 0.5..1 |
| `brush.ribbon_friction` | 0.3 (spec_default.rs) | 22 | **0** | 0.3..0.3 |
| `brush.ribbon_gravity` | 0 (spec_default.rs) | 22 | **0** | 0..0 |
| `brush.ribbon_rungs` | 0.5 (spec_default.rs) | 22 | **0** | 0.5..0.5 |
| `brush.ribbon_weight` | 0.45 (spec_default.rs) | 22 | **0** | 0.45..0.45 |
| `brush.rough_amount` | 0.4 (spec_default.rs) | 22 | **0** | 0.4..0.4 |
| `brush.rough_bowing` | 0.4 (spec_default.rs) | 22 | **0** | 0.4..0.4 |
| `brush.rough_passes` | 2 (spec_default.rs) | 22 | **0** | 2..2 |
| `brush.shape.angle_deg` | 0 (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.shape.flow` | False (moda (sem padrão escalar no código)) | 77 | **2** | 0..1 |
| `brush.shape.kind` | None (moda (sem padrão escalar no código)) | 77 | **15** | —..— |
| `brush.shape.mapping` | ViewPlane (moda (sem padrão escalar no código)) | 77 | **11** | —..— |
| `brush.shape.offset[0]` | 0 (moda (sem padrão escalar no código)) | 77 | **1** | 0..0.5 |
| `brush.shape.offset[1]` | 0 (moda (sem padrão escalar no código)) | 77 | **1** | 0..0.5 |
| `brush.shape.params` | [0.5, 0.5, 0.5, 0.5, 0.5, 0.5] (moda (sem padrão escalar no código)) | 77 | **12** | —..— |
| `brush.shape.rake` | False (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.shape.size[0]` | 1 (moda (sem padrão escalar no código)) | 77 | **13** | 0.02..16 |
| `brush.shape.size[1]` | 1 (moda (sem padrão escalar no código)) | 77 | **13** | 0.5..50 |
| `brush.shape.stencil_angle_deg` | 0 (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.shape.stencil_offset[0]` | 0 (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.shape.stencil_offset[1]` | 0 (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.shape.stencil_size[0]` | 0.5 (moda (sem padrão escalar no código)) | 77 | **0** | 0.5..0.5 |
| `brush.shape.stencil_size[1]` | 0.5 (moda (sem padrão escalar no código)) | 77 | **0** | 0.5..0.5 |
| `brush.sketchy_density` | 0.2 (spec_default.rs) | 22 | **0** | 0.2..0.2 |
| `brush.sketchy_magnetify` | 1 (spec_default.rs) | 22 | **0** | 1..1 |
| `brush.sketchy_reach` | 1 (spec_default.rs) | 22 | **0** | 1..1 |
| `brush.space_attenuation` | 0 (spec_default.rs) | 22 | **0** | 0..0 |
| `brush.spacing` | 0.1 (spec_default.rs) | 77 | **9** | 0.05..0.85 |
| `brush.spray_count` | 1 (spec_default.rs) | 77 | **3** | 1..6 |
| `brush.stabilizer` | 0.5 (spec_default.rs) | 77 | **3** | 0..0.5 |
| `brush.stroke_method` | Space (moda (sem padrão escalar no código)) | 22 | **0** | —..— |
| `brush.style_solid` | 0 (spec_default.rs) | 22 | **0** | 0..0 |
| `brush.taper.opacity` | 0 (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.taper.start` | 0 (moda (sem padrão escalar no código)) | 77 | **1** | 0..0.2 |
| `brush.taper.tip_start` | 0 (moda (sem padrão escalar no código)) | 77 | **1** | 0..0.8 |
| `brush.texture.angle_deg` | 0 (moda (sem padrão escalar no código)) | 77 | **1** | 0..45 |
| `brush.texture.flow` | False (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.texture.kind` | None (moda (sem padrão escalar no código)) | 77 | **24** | —..— |
| `brush.texture.mapping` | Tiled (moda (sem padrão escalar no código)) | 77 | **0** | —..— |
| `brush.texture.offset[0]` | 0 (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.texture.offset[1]` | 0 (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.texture.params` | [0.5, 0.5, 0.5, 0.5, 0.5, 0.5] (moda (sem padrão escalar no código)) | 77 | **8** | —..— |
| `brush.texture.rake` | False (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.texture.size[0]` | 1 (moda (sem padrão escalar no código)) | 77 | **24** | 1..256 |
| `brush.texture.size[1]` | 1 (moda (sem padrão escalar no código)) | 77 | **24** | 1..256 |
| `brush.texture.stencil_angle_deg` | 0 (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.texture.stencil_offset[0]` | 0 (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.texture.stencil_offset[1]` | 0 (moda (sem padrão escalar no código)) | 77 | **0** | 0..0 |
| `brush.texture.stencil_size[0]` | 0.5 (moda (sem padrão escalar no código)) | 77 | **0** | 0.5..0.5 |
| `brush.texture.stencil_size[1]` | 0.5 (moda (sem padrão escalar no código)) | 77 | **0** | 0.5..0.5 |
| `brush.thread_opacity` | 0.25 (spec_default.rs) | 22 | **0** | 0.25..0.25 |
| `brush.thread_width_px` | 1 (spec_default.rs) | 22 | **0** | 1..1 |
| `brush.wire_connection_line` | 1 (spec_default.rs) | 22 | **0** | 1..1 |
| `brush.wire_history` | 6 (spec_default.rs) | 22 | **0** | 6..6 |
| `ferramenta` | Eraser (moda (sem padrão escalar no código)) | 6 | **3** | —..— |
| `volumes.size_px` | 51 (moda (sem padrão escalar no código)) | 41 | **33** | 1..248 |
| `paint_mode` | Paint (moda (sem padrão escalar no código)) | 71 | **10** | —..— |
| `brush.borda_px` | 0 (spec_default.rs) | 7 | **4** | 0..3 |
| `brush.sub_passo_fino` | 0 (spec_default.rs) | 1 | **1** | 1..1 |
| `brush.pixel` | 2 (moda (sem padrão escalar no código)) | 1 | **0** | 2..2 |
| `brush.pigment_espectral` | 0 (spec_default.rs) | 1 | **1** | 1..1 |
| `brush.tecto_por_dab` | 0 (spec_default.rs) | 1 | **1** | 1..1 |
| `brush.dab_jitter.particle_max_px` | 72 (moda (sem padrão escalar no código)) | 2 | **1** | 56..72 |
| `brush.dab_jitter.veu` | 0.2 (moda (sem padrão escalar no código)) | 1 | **0** | 0.2..0.2 |
| `brush.dab_jitter.veu_escala` | 3 (moda (sem padrão escalar no código)) | 1 | **0** | 3..3 |
| `brush.dab_jitter.veu_opacidade` | 1 (moda (sem padrão escalar no código)) | 1 | **0** | 1..1 |
| `brush.passo_pela_pressao` | 0 (spec_default.rs) | 2 | **2** | 1..1 |
| `brush.depth` | 1.2 (spec_default.rs) | 1 | **1** | 2..2 |
| `brush.edge_flow.angle_deg` | 0 (moda (sem padrão escalar no código)) | 1 | **0** | 0..0 |
| `brush.edge_flow.flow` | False (moda (sem padrão escalar no código)) | 1 | **0** | 0..0 |
| `brush.edge_flow.kind` | None (moda (sem padrão escalar no código)) | 1 | **0** | —..— |
| `brush.edge_flow.mapping` | Tiled (moda (sem padrão escalar no código)) | 1 | **0** | —..— |
| `brush.edge_flow.offset[0]` | 0 (moda (sem padrão escalar no código)) | 1 | **0** | 0..0 |
| `brush.edge_flow.offset[1]` | 0 (moda (sem padrão escalar no código)) | 1 | **0** | 0..0 |
| `brush.edge_flow.params` | [0.5, 0.5, 0.5, 0.5, 0.5, 0.5] (moda (sem padrão escalar no código)) | 1 | **0** | —..— |
| `brush.edge_flow.rake` | False (moda (sem padrão escalar no código)) | 1 | **0** | 0..0 |
| `brush.edge_flow.size[0]` | 2 (moda (sem padrão escalar no código)) | 1 | **0** | 2..2 |
| `brush.edge_flow.size[1]` | 2 (moda (sem padrão escalar no código)) | 1 | **0** | 2..2 |
| `brush.edge_flow.stencil_angle_deg` | 0 (moda (sem padrão escalar no código)) | 1 | **0** | 0..0 |
| `brush.edge_flow.stencil_offset[0]` | 0 (moda (sem padrão escalar no código)) | 1 | **0** | 0..0 |
| `brush.edge_flow.stencil_offset[1]` | 0 (moda (sem padrão escalar no código)) | 1 | **0** | 0..0 |
| `brush.edge_flow.stencil_size[0]` | 0.5 (moda (sem padrão escalar no código)) | 1 | **0** | 0.5..0.5 |
| `brush.edge_flow.stencil_size[1]` | 0.5 (moda (sem padrão escalar no código)) | 1 | **0** | 0.5..0.5 |
| `brush.edge_gain` | 1.5 (spec_default.rs) | 1 | **1** | 0.2..0.2 |
| `brush.edge_spread` | 7 (spec_default.rs) | 1 | **1** | 4..4 |
| `brush.fill` | 0.12 (spec_default.rs) | 1 | **1** | 0.25..0.25 |
| `brush.granulation` | 0.3 (spec_default.rs) | 1 | **0** | 0.3..0.3 |
| `brush.granulation_use_paper` | 1 (spec_default.rs) | 1 | **0** | 1..1 |
| `brush.opacity` | 0.4 (spec_default.rs) | 1 | **0** | 0.4..0.4 |
| `brush.paper_edge` | 0 (spec_default.rs) | 1 | **0** | 0..0 |
| `brush.smooth_edges` | 1 (spec_default.rs) | 1 | **0** | 1..1 |
| `brush.warp` | 6 (spec_default.rs) | 1 | **1** | 12..12 |
| `brush.watercolor_shape_auto` | 1 (spec_default.rs) | 1 | **0** | 1..1 |
| `brush.wet_charge` | 1 (spec_default.rs) | 1 | **1** | 0.5..0.5 |
| `brush.wet_dilution` | 0 (spec_default.rs) | 1 | **1** | 0.2..0.2 |
| `brush.wet_pull` | 0 (spec_default.rs) | 1 | **1** | 0.6..0.6 |
| `brush.wet_rewet` | 0 (spec_default.rs) | 1 | **1** | 0.5..0.5 |
| `brush.wet_smudge` | 0 (spec_default.rs) | 1 | **1** | 0.5..0.5 |
| `volumes.water` | 1 (ausente = o de agora; referência 1,0) | 55 | **40** | 0..1 |
| `width_table` | [[1.5, 3.9791667], [1.7, 2.884 (moda (sem padrão escalar no código)) | 55 | **53** | —..— |
| `tilt.on` | 0 (ausente = OFF) | 55 | **1** | 0..1 |
| `tilt.ring` | 4 (moda (sem padrão escalar no código)) | 55 | **1** | 4..8 |
| `tilt.spoke` | 3 (moda (sem padrão escalar no código)) | 55 | **0** | 3..3 |
| `brush.size_max_px` | 60 (moda (sem padrão escalar no código)) | 14 | **11** | 50..140 |
| `brush.pontas_alem` | 0 (spec_default.rs) | 3 | **3** | 0.3..0.7 |
| `brush.pontas_esbate` | 0 (spec_default.rs) | 5 | **5** | -2.5..4 |
| `brush.pontas_rectas` | 0 (spec_default.rs) | 4 | **4** | 1..1 |
| `brush.dab_rotation_lag` | 0 (spec_default.rs) | 3 | **3** | 0.6..1.5 |
| `brush.dab_jitter.po` | 3.5 (moda (sem padrão escalar no código)) | 4 | **2** | 2.5..4 |
| `brush.dab_jitter.po_alcance` | 0.4 (moda (sem padrão escalar no código)) | 4 | **2** | 0.4..4 |
| `brush.dab_jitter.po_lado` | 0.43 (moda (sem padrão escalar no código)) | 4 | **2** | 0.35..0.43 |
| `brush.dab_jitter.po_opacidade` | 0.1 (moda (sem padrão escalar no código)) | 4 | **1** | 0.1..0.12 |
| `brush.dab_jitter.po_px` | 1.5 (moda (sem padrão escalar no código)) | 4 | **1** | 1.5..2 |
| `brush.dual.angulo` | 0 (moda (sem padrão escalar no código)) | 5 | **0** | 0..0 |
| `brush.dual.atraves` | 0.3 (moda (sem padrão escalar no código)) | 5 | **2** | 0.2..0.4 |
| `brush.dual.contagem` | 3 (moda (sem padrão escalar no código)) | 5 | **2** | 2..3 |
| `brush.dual.dureza` | 0 (moda (sem padrão escalar no código)) | 5 | **2** | 0..0.3 |
| `brush.dual.escala` | 0.45 (moda (sem padrão escalar no código)) | 5 | **2** | 0.45..0.6 |
| `brush.dual.espalhar` | 0.25 (moda (sem padrão escalar no código)) | 5 | **2** | 0.2..0.4 |
| `brush.dual.forma.angle_deg` | 0 (moda (sem padrão escalar no código)) | 5 | **0** | 0..0 |
| `brush.dual.forma.flow` | False (moda (sem padrão escalar no código)) | 5 | **0** | 0..0 |
| `brush.dual.forma.kind` | Noise (moda (sem padrão escalar no código)) | 5 | **2** | —..— |
| `brush.dual.forma.mapping` | Random (moda (sem padrão escalar no código)) | 5 | **2** | —..— |
| `brush.dual.forma.offset[0]` | 0 (moda (sem padrão escalar no código)) | 5 | **0** | 0..0 |
| `brush.dual.forma.offset[1]` | 0 (moda (sem padrão escalar no código)) | 5 | **0** | 0..0 |
| `brush.dual.forma.params` | [0.3, 0.5, 0.5, 0.5, 0.5, 0.5] (moda (sem padrão escalar no código)) | 5 | **2** | —..— |
| `brush.dual.forma.rake` | False (moda (sem padrão escalar no código)) | 5 | **0** | 0..0 |
| `brush.dual.forma.size[0]` | 32 (moda (sem padrão escalar no código)) | 5 | **2** | 1..32 |
| `brush.dual.forma.size[1]` | 32 (moda (sem padrão escalar no código)) | 5 | **2** | 1..32 |
| `brush.dual.forma.stencil_angle_deg` | 0 (moda (sem padrão escalar no código)) | 5 | **0** | 0..0 |
| `brush.dual.forma.stencil_offset[0]` | 0 (moda (sem padrão escalar no código)) | 5 | **0** | 0..0 |
| `brush.dual.forma.stencil_offset[1]` | 0 (moda (sem padrão escalar no código)) | 5 | **0** | 0..0 |
| `brush.dual.forma.stencil_size[0]` | 0.5 (moda (sem padrão escalar no código)) | 5 | **0** | 0.5..0.5 |
| `brush.dual.forma.stencil_size[1]` | 0.5 (moda (sem padrão escalar no código)) | 5 | **0** | 0.5..0.5 |
| `brush.dual.modo` | 0 (moda (sem padrão escalar no código)) | 5 | **0** | 0..0 |
| `brush.dual.passo` | 0.25 (moda (sem padrão escalar no código)) | 5 | **0** | 0.25..0.25 |
| `brush.dual.rotacao` | 0 (moda (sem padrão escalar no código)) | 5 | **0** | 0..0 |

## (c) Knobs que nenhum pincel usa e knobs de um só pincel

**Nenhum pincel os tira do padrão (75) — candidatos a sair ou a virar constante:**

`intensity`, `spacing`, `gateSaturation`, `rewet`, `baseEvaporation`, `leveling`, `brake`, `gravity`, `levelClamp`, `viscosity`, `maxVelocity`, `projection`, `brakeReach`, `capillaryGate`, `eraser`, `dryer`, `blow`, `paperFibres`, `paperGrooves`, `visualGrain`, `paperVisibility`, `extBackrun`, `extFingering`, `extRivulets`, `extGranulation`, `extStaining`, `extWetSoften`, `extRakeLight`, `extValleyGran`, `extWetSheen`, `extGrainDither`, `extEdgeTint`, `pressureSmoothing`, `pulledString`, `tiltSmooth`, `bristleWave`, `bristleWaveLength`, `bristleNoiseHue`, `bristleRemove`, `capsuleSpacing`, `embossSizeRef`, `dustDrop`, `bristleStreak`, `bristleStreakLight`, `bristleStreakLength`, `bristleStreakCut`, `thinStrokeFade`, `thinStrokeRef`, `dryerEdge`, `textureSizeRef`, `iorLevel`, `coatRoughness`, `sheen`, `subsurface`, `subsurfaceRadius`, `transparency`, `lockMaterial`, `textureSizeExpAcross`, `transmission`, `paperRelief`, `transmissionMode`, `dryerLevel`, `blendReach`, `paperPress`, `rewetPaint`, `solubility`, `rewetDrag`, `tiltOpacity`, `pressureSCurve`, `speedLag`, `speedHardness`, `speedOpacity`, `erase`, `glazeLayering`, `paintTexture`

**Usados por UM só pincel (26):**

| chave | pincel | valor | padrão |
|---|---|---|---|
| `waterCap` | WetPaint/Watercolor/Wet Liner | 30 | 8 |
| `retention` | WetPaint/Watercolor/Gouache Opaque | 0.97 | 0.995 |
| `extDiffusion` | WetPaint/Ink/Ink Brush | 0.01 | 0 |
| `wetLift` | WetPaint/Water/Water Brush | 0.7 | 0.25 |
| `dust` | WetPaint/Airbrush/Airbrush | 0.8 | 0 |
| `pressureCeiling` | Digital/Basic/Soft Round | 0.8 | 1 |
| `bristleOscillation` | WetPaint/Acrylic/Fan Brush | 1 | 0 |
| `bristleSpan` | WetPaint/Acrylic/Fan Brush | 2 | 0 |
| `bristleSpanLength` | WetPaint/Acrylic/Fan Brush | 20 | 10 |
| `waterPush` | WetPaint/Water/Water Brush | 0.35 | 0 |
| `smearWater` | WetPaint/Smear/Smear Wet | 0.9 | 0.5 |
| `textureSizeExp` | WetPaint/Oil/Dabber 4 | 0.5 | 0 |
| `metallic` | WetPaint/Oil/Metallic Flat | 0.2 | 0 |
| `roughness` | WetPaint/Oil/Metallic Flat | 0.4 | 0.5 |
| `coat` | WetPaint/Oil/Metallic Flat | 0.3 | 0 |
| `knifeLevel` | WetPaint/Oil/Knife Textured | 1 | 0 |
| `wetOpenEdge` | WetPaint/Watercolor/Wet Liner | 28 | 0 |
| `wetCover` | WetPaint/Watercolor/Gouache Opaque | 1 | 0 |
| `bristleLaneSizeExp` | WetPaint/Acrylic/Fan Brush | 0.8 | 0 |
| `edgeSoftMass` | WetPaint/Watercolor/Wet Wash | 4 | 0 |
| `dryCover` | WetPaint/Watercolor/Gouache Opaque | 1 | 0 |
| `tuftArc` | WetPaint/Acrylic/Fan Brush | 0.8 | 0 |
| `waterDrag` | WetPaint/Water/Water Brush | 0.8 | 0 |
| `speedSize` | Digital/Ink/Mapping Pen | 0.025 | 0 |
| `pressureJitter` | Digital/Ink/Ballpoint | 0.35 | 0 |
| `pressureHardness` | Digital/Ink/Ballpoint | 0.15 | 0 |

## (d) Famílias de duplicados D1..D22: que pincéis dependem de cada membro

Membro «usado» = valor efectivo fora do padrão. «≥ 2 membros» = pincéis que tiram do padrão MAIS DE UM membro da família — são eles que a migração para UMA lei tem de traduzir com conta, não com cópia.

### D1 — massa de pigmento por dab

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `pigmentPerDab` | 44 | Fan Brush, Airbrush, Express Oil, Ink Brush, Broad Nib, Dip Pen, Ink Bullet, Marker Bullet, Marker Chisel, Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat, Pastel Side, Soft Pastel, Square Hard, Oil Pastel, Wax Crayon, Wax Scrape, Charcoal, Compressed Charcoal, Colored… |
| `volumes.opacity` | 10 | Glaze Round, Smear Basic, Kneaded Eraser, Texture Eraser, Ballpoint, Mapping Pen, Bokeh, Stars, Fan Brush, Express Oil |
| `intensity` | 0 |  |
| `pressureOpacity` | 57 | Glaze Round, Soft Round, Burn, Color Glaze, Dodge, Kneaded Eraser, Ballpoint, Marker Liner, Splatter, Bokeh, Stars, Halftone, Fan Brush, Airbrush, Blender Hard, Express Oil, Ink Brush, Broad Nib, Dip Pen, Ink Bullet, Marker Bullet, Marker Chisel, Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knif… |

**≥ 2 membros: 46 pincéis**: Fan Brush, Airbrush, Express Oil, Ink Brush, Broad Nib, Dip Pen, Ink Bullet, Marker Bullet, Marker Chisel, Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat, Pastel Side, Soft Pastel, Square Hard, Oil Pastel, Wax Crayon, Wax Scrape, Charcoal, Compressed Charcoal, Colored 4B, Graphite 2B, Graphite 2H, Graphite 8B, Graphite HB, Scratch, Wet Flat, Wet Wash, Gouache Flat, Gouache Opaque, Glaze Round, Kneaded Eraser, Ballpoint, Bokeh, Stars

### D2 — arrasto ao longo do traço

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `drag` | 20 | Airbrush, Broad Nib, Marker Bullet, Marker Chisel, Pastel Side, Soft Pastel, Square Hard, Oil Pastel, Wax Crayon, Wax Scrape, Charcoal, Compressed Charcoal, Colored 4B, Graphite 2B, Graphite 2H, Graphite 8B, Graphite HB, Scratch, Wet Liner, Wet Wash |
| `smear` | 6 | Metallic Flat, Smear Flat, Smear Shattered, Smear Soft, Smear Spikey, Smear Wet |
| `rewetDrag` | 0 |  |
| `waterDrag` | 1 | Water Brush |
| `smearWater` | 1 | Smear Wet |

**≥ 2 membros: 1 pincéis**: Smear Wet

### D3 — recolha pela ponta

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `pickup` | 9 | Broad Nib, Dip Pen, Ink Bullet, Marker Bullet, Marker Chisel, Wet Drip, Wet Dry Brush, Wet Liner, Wet Wash |
| `tipReservoir` | 4 | Ink Brush, Broad Nib, Dip Pen, Ink Bullet |
| `tipClean` | 6 | Ink Brush, Broad Nib, Dip Pen, Ink Bullet, Marker Bullet, Marker Chisel |
| `wetLift` | 1 | Water Brush |

**≥ 2 membros: 6 pincéis**: Broad Nib, Dip Pen, Ink Bullet, Marker Bullet, Marker Chisel, Ink Brush

### D4 — água por dab

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `volumes.water` | 40 | Fan Brush, Airbrush, Express Oil, Ink Brush, Broad Nib, Dip Pen, Ink Bullet, Marker Bullet, Marker Chisel, Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat, Pastel Side, Soft Pastel, Square Hard, Oil Pastel, Wax Crayon, Wax Scrape, Charcoal, Compressed Charcoal, Colored… |
| `waterPerDab` | 3 | Water Brush, Wet Liner, Wet Wash |
| `smearWater` | 1 | Smear Wet |

**≥ 2 membros: 2 pincéis**: Wet Liner, Wet Wash

### D5 — secagem

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `evaporation` | 4 | Ink Brush, Wet Drip, Wet Dry Brush, Wet Wash |
| `baseEvaporation` | 0 |  |
| `retention` | 1 | Gouache Opaque |
| `dryer` | 0 |  |
| `dryerLevel` | 0 |  |
| `dryerEdge` | 0 |  |

**≥ 2 membros: 0 pincéis**

### D6 — absorção do papel

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `paperGate` | 38 | Fan Brush, Airbrush, Express Oil, Ink Brush, Broad Nib, Dip Pen, Ink Bullet, Marker Bullet, Marker Chisel, Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat, Soft Pastel, Square Hard, Oil Pastel, Wax Crayon, Wax Scrape, Charcoal, Compressed Charcoal, Colored 4B, Graphite… |
| `gateSaturation` | 0 |  |
| `capillary` | 2 | Wet Liner, Wet Wash |
| `capillaryGate` | 0 |  |

**≥ 2 membros: 2 pincéis**: Wet Liner, Wet Wash

### D7 — passo

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `spacing` | 0 |  |
| `capsuleSpacing` | 0 |  |
| `pigmentSpacing` | 16 | Fan Brush, Express Oil, Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat |
| `capsuleMaxPx` | 14 | Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat |
| `brush.spacing` | 9 | Blur Basic, Clone Basic, Smear Basic, Ballpoint, Mapping Pen, Splatter, Bokeh, Stars, Dabber 4 |

**≥ 2 membros: 14 pincéis**: Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat

### D8 — re-dissolver o assentado

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `rewet` | 0 |  |
| `rewetPaint` | 0 |  |
| `solubility` | 0 |  |

**≥ 2 membros: 0 pincéis**

### D9 — orla escura / sedimento

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `edgeDarkening` | 42 | Fan Brush, Airbrush, Express Oil, Ink Brush, Broad Nib, Dip Pen, Ink Bullet, Marker Bullet, Marker Chisel, Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat, Pastel Side, Soft Pastel, Square Hard, Oil Pastel, Wax Crayon, Wax Scrape, Charcoal, Compressed Charcoal, Colored… |
| `edgeSoftMass` | 1 | Wet Wash |
| `heavy_edge` | 0 | (nenhum preset grava o campo) |

**≥ 2 membros: 1 pincéis**: Wet Wash

### D10 — nivelamento

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `leveling` | 0 |  |
| `levelClamp` | 0 |  |
| `viscosity` | 0 |  |
| `brake` | 0 |  |
| `brakeReach` | 0 |  |
| `levelingOwnStroke` | 15 | Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat, Wet Liner, Wet Wash, Gouache Opaque |
| `knifeLevel` | 1 | Knife Textured |
| `dryerLevel` | 0 |  |

**≥ 2 membros: 1 pincéis**: Knife Textured

### D11 — força da borracha

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `erase` | 0 |  |
| `eraser` | 0 |  |

**≥ 2 membros: 0 pincéis**

### D12 — papel visto na tinta

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `paperVisibility` | 0 |  |
| `visualGrain` | 0 |  |
| `paintTexture` | 0 |  |
| `paperRelief` | 0 |  |
| `paperPress` | 0 |  |
| `extGranulation` | 0 |  |

**≥ 2 membros: 0 pincéis**

### D13 — altura do relevo

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `emboss` | 15 | Fan Brush, Express Oil, Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Stain, Knife Textured, Metallic Flat |
| `oilDepth` | 10 | Dabber 4, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Stain, Knife Textured, Metallic Flat |
| `knifeRidge` | 4 | Knife Grainy, Knife Oily, Knife Smooth 3, Knife Textured |
| `bristleHeight` | 11 | Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3 |
| `bristleRemove` | 0 |  |

**≥ 2 membros: 14 pincéis**: Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Stain, Knife Textured, Metallic Flat, Knife Smooth 3

### D14 — aderência do assentado

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `extStaining` | 0 |  |
| `solubility` | 0 |  |

**≥ 2 membros: 0 pincéis**

### D15 — pincel seco

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `extDryBrush` | 2 | Filbert Dry, Knife Dry |

**≥ 2 membros: 0 pincéis**

### D16 — velatura

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `glaze` | 7 | Ink Brush, Dip Pen, Ink Bullet, Marker Bullet, Marker Chisel, Wet Liner, Wet Wash |
| `glazeLayering` | 0 |  |
| `pigmentMixing` | 3 | Wet Liner, Wet Wash, Gouache Opaque |
| `live_water` | 0 | (nenhum preset grava o campo) |

**≥ 2 membros: 2 pincéis**: Wet Liner, Wet Wash

### D17 — estabilização

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `pulledString` | 0 |  |
| `brush.stabilizer` | 3 | Burn, Color Glaze, Dodge |

**≥ 2 membros: 0 pincéis**

### D18 — escala com o diâmetro

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `embossSizeExp` | 6 | Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Metallic Flat |
| `embossSizeRef` | 0 |  |
| `textureSizeExp` | 1 | Dabber 4 |
| `textureSizeRef` | 0 |  |
| `textureSizeExpAcross` | 0 |  |
| `bristleLaneSizeExp` | 1 | Fan Brush |

**≥ 2 membros: 0 pincéis**

### D19 — abertura do molhado alheio

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `wetOpen` | 2 | Wet Liner, Wet Wash |
| `wetOpenPush` | 2 | Wet Liner, Wet Wash |
| `wetOpenEdge` | 1 | Wet Liner |

**≥ 2 membros: 2 pincéis**: Wet Liner, Wet Wash

### D20 — material Principled (célula)

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `metallic` | 1 | Metallic Flat |
| `roughness` | 1 | Metallic Flat |
| `iorLevel` | 0 |  |
| `coat` | 1 | Metallic Flat |
| `coatRoughness` | 0 |  |
| `sheen` | 0 |  |
| `subsurface` | 0 |  |
| `subsurfaceRadius` | 0 |  |

**≥ 2 membros: 1 pincéis**: Metallic Flat

### D21 — cobertura sobre a de baixo

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `opaque` | 3 | Fan Brush, Gouache Flat, Gouache Opaque |
| `wetCover` | 1 | Gouache Opaque |
| `dryCover` | 1 | Gouache Opaque |
| `transparency` | 0 |  |
| `transmission` | 0 |  |
| `transmissionMode` | 0 |  |
| `realOil` | 14 | Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat |
| `pigmentPerDab` | 44 | Fan Brush, Airbrush, Express Oil, Ink Brush, Broad Nib, Dip Pen, Ink Bullet, Marker Bullet, Marker Chisel, Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat, Pastel Side, Soft Pastel, Square Hard, Oil Pastel, Wax Crayon, Wax Scrape, Charcoal, Compressed Charcoal, Colored… |
| `volumes.opacity` | 10 | Glaze Round, Smear Basic, Kneaded Eraser, Texture Eraser, Ballpoint, Mapping Pen, Bokeh, Stars, Fan Brush, Express Oil |

**≥ 2 membros: 18 pincéis**: Fan Brush, Gouache Flat, Gouache Opaque, Dabber 4, Dabber, Filbert Dry, Filbert, Flat Bristle, Thick Impasto, Tilted Shade, Knife Dry, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat, Express Oil

### D22 — brilho especular

| membro | pincéis fora do padrão | quais |
|---|---|---|
| `oilGloss` | 4 | Dabber 4, Thick Impasto, Knife Dry, Knife Stain |
| `coat` | 1 | Metallic Flat |
| `coatRoughness` | 0 |  |
| `roughness` | 1 | Metallic Flat |

**≥ 2 membros: 1 pincéis**: Metallic Flat

**Resumo:** D1 46 · D2 1 · D3 6 · D4 2 · D5 0 · D6 2 · D7 14 · D8 0 · D9 1 · D10 1 · D11 0 · D12 0 · D13 14 · D14 0 · D15 0 · D16 2 · D17 0 · D18 0 · D19 2 · D20 1 · D21 18 · D22 1

## (e) Pincéis de óleo: todos os knobs fora do padrão, lado a lado

Óleo = categoria `Oil`/`ExpressOil` ou `realOil = 1`. Célula vazia = no padrão.

| knob | padrão | Express Oil | Dabber 4 | Dabber | Filbert Dry | Filbert | Flat Bristle | Thick Impasto | Tilted Shade | Knife Dry | Knife Grainy | Knife Oily | Knife Smooth 3 | Knife Stain | Knife Textured | Metallic Flat |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `pigmentPerDab` | 78 | 75 | 2000 | 2000 | 90 | 250 | 200 | 600 | 250 | 400 | 600 | 150 | 600 | 1200 | 400 | 80 |
| `paperGate` | 0.4 | 0 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 1.2 | 0 | 1.2 | 2.5 | 1 | 0 |
| `felt` | 0.03 | 0 | 0.05 | 0.05 | 0.05 | 0.05 | 0.05 | 0.05 | 0.05 | 0.05 | 0.05 | 0.05 | 0.05 | 0.05 | 0.05 | 0.05 |
| `bristleCount` | 2000 | 300 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| `bristleStrength` | 1 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 0 | 0 | 0 |
| `bristleSize` | 2 | 7 |  |  | 1 | 1 | 1 | 1 | 1 |  |  |  |  |  |  | 1 |
| `blendForce` | 0.8 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.3 |
| `edgeDarkening` | 12 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `smear` | 0.8 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.2 |
| `paperContrast` | 1 |  |  |  |  |  |  |  |  |  | 2 |  | 2.2 | 3 |  |  |
| `emboss` | 0 | 0.2 | 0.5 | 0.5 | 0.4 | 0.4 | 0.4 | 0.3 | 0.4 | 0.1 | 0.1 | 0.1 |  | 0.1 | 0.1 | 0.2 |
| `extDryBrush` | 0 |  |  |  | 1 |  |  |  |  | 0.6 |  |  |  |  |  |  |
| `pressureSize` | 0.9 |  |  |  | 0.2 | 0.2 | 0.2 | 0.2 | 0.2 | 0.6 | 0.3 | 0.6 | 0.4 | 1 | 1 | 0.1 |
| `pressureOpacity` | 0 | 0.75 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 0.6 |
| `pressureOpacityCurve` | 1 | 1.5 | 3 | 3 | 2 | 2 | 2 | 2 | 2 |  |  |  |  |  |  | 2 |
| `pressureSizeCurve` | 1 | 0.5 | 0.25 | 0.25 | 0.25 | 0.25 | 0.25 | 0.25 | 0.25 |  |  |  |  | 1.5 | 1.5 | 0.25 |
| `dabRotation` | 0 |  |  |  | 1 |  | 1 |  | 1 |  | 1 |  | 1 | 1 |  |  |
| `bristlePressure` | 0 | 2.5 | 1.2 | 1.2 | 1.2 | 1.2 | 1.2 | 1.2 | 1.2 | 1.2 | 1.2 | 1.2 | 1.2 | 1.2 | 1.2 | 0.3 |
| `bristleNoiseLength` | 150 | 2000 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| `bristleLaneWidth` | 3 | 4 | 1.5 | 1.5 | 4 | 4 | 4 |  | 4 | 1.5 | 1.5 | 1.5 | 1.5 | 1.5 | 1.5 | 1.5 |
| `bristleNoiseSat` | 0 |  | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 | 0.3 | 0.05 |
| `bristleNoiseLight` | 0 |  | 0.1 | 0.1 |  |  |  |  |  | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 |  |
| `bristleLift` | 0 | 0.2 | 0.4 | 0.4 |  |  |  |  |  | 0.4 | 0.4 | 0.4 | 0.4 | 0.4 | 0.4 |  |
| `bristleHeight` | 0 |  | 0.75 | 0.75 | 0.75 | 0.75 | 0.75 | 0.75 | 0.75 | 0.75 | 0.75 | 0.75 | 0.75 |  |  |  |
| `pigmentSpacing` | 0.025 | 0.01 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `capsuleMaxPx` | 0 |  | 2 | 2 | 4 | 4 | 4 | 4 | 4 | 2 | 2 | 2 | 2 | 2 | 2 | 4 |
| `embossSizeExp` | 0 |  |  |  | 0.4 | 0.4 | 0.4 | 0.4 | 0.4 |  |  |  |  |  |  | 0.4 |
| `bristleTipRagged` | 0 |  |  |  | 1.6 | 1.6 | 1.6 | 1.6 | 1.6 |  |  |  |  |  |  |  |
| `levelingOwnStroke` | 0 |  |  |  | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| `knifeRidge` | 0 |  |  |  |  |  |  |  |  |  | 2 | 3 | 8 |  | 4 |  |
| `textureSizeExp` | 0 |  | 0.5 |  |  |  |  |  |  |  |  |  |  |  |  |  |
| `metallic` | 0 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.2 |
| `roughness` | 0.5 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.4 |
| `coat` | 0 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.3 |
| `knifeLevel` | 0 |  |  |  |  |  |  |  |  |  |  |  |  |  | 1 |  |
| `oilDepth` | 1 |  | 0.3 |  | 0.3 | 0.3 | 0.3 | 0.9 | 0.3 | 0.1 |  |  |  | 0.1 | 0.1 | 0.5 |
| `oilGloss` | 1 |  | 0.1 |  |  |  |  | 0.5 |  | 0.3 |  |  |  | 0.3 |  |  |
| `realOil` | 0 |  | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| `volumes.opacity` | 1 | 0.6 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| `brush.continuous_spacing` | 0 | True |  | True | True | True | True | True | True | True | True | True | True | True | True | True |
| `brush.dab_angle_deg` | 0 |  | 90 | 90 |  | 90 |  | 90 |  | 170 | 90 | 172 | 90 | 90 | 86 | 90 |
| `brush.dab_flatten` | 0 |  | 0.6 |  | 0.5 |  | 0.7 |  | 0.65 | 0.74 | 0.8 | 0.69 | 0.8 | 0.8 | 0.87 |  |
| `brush.dab_quina` | 0 |  |  |  | 0.4 |  | 1 |  | 0.5 | 1 | 1 | 1 | 1 | 1 | 0.5 | 0.75 |
| `brush.falloff` | Smooth |  |  |  |  |  |  |  |  |  | Constant |  | Constant | Constant | Constant |  |
| `brush.grain_depth` | 1 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.3 |
| `brush.hardness` | 0 | 0.9 | 0.6 | 0.6 | 0.9 | 0.9 | 0.9 | 0.3 | 0.9 | 0.4 | 1 | 0.5 | 1 | 1 | 1 | 1 |
| `brush.jitter` | 0 |  | 0.1 | 0.3 |  |  |  | 0.15 |  |  | 0.1 |  |  |  |  |  |
| `brush.shape.kind` | None |  |  |  |  |  |  |  |  |  |  | Gradient |  |  |  |  |
| `brush.shape.offset[0]` | 0 |  |  |  |  |  |  |  |  |  |  | 0.5 |  |  |  |  |
| `brush.shape.offset[1]` | 0 |  |  |  |  |  |  |  |  |  |  | 0.5 |  |  |  |  |
| `brush.shape.params` | [0.5, 0.5, 0 |  |  |  |  |  |  |  |  |  |  | [0.5, 0.5, 0 |  |  |  |  |
| `brush.shape.size[0]` | 1 |  |  |  |  |  |  |  |  |  |  | 0.5 |  |  |  |  |
| `brush.shape.size[1]` | 1 |  |  |  |  |  |  |  |  |  |  | 0.5 |  |  |  |  |
| `brush.spacing` | 0.1 |  | 0.3 |  |  |  |  |  |  |  |  |  |  |  |  |  |
| `brush.texture.kind` | None |  | Stucci |  |  |  |  | Clouds |  | Noise | Clouds |  | Noise | Noise | Noise | Noise |
| `brush.texture.params` | [0.5, 0.5, 0 |  | [0.5, 0.6, 0 |  |  |  |  |  |  |  |  |  |  | [1, 0.42, 0. | [0.25, 0.5,  |  |
| `brush.texture.size[0]` | 1 |  | 180 |  |  |  |  | 48 |  | 6 | 20 |  | 6 | 96 | 60 | 4 |
| `brush.texture.size[1]` | 1 |  | 180 |  |  |  |  | 48 |  | 6 | 20 |  | 6 | 96 | 2 | 4 |
| `volumes.size_px` | 51 | 64 |  |  | 34 | 64 | 46 | 64 | 40 |  |  |  |  |  |  | 40 |
| `volumes.water` | 1 | 0 | 0.1 | 0.1 | 0 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 |
| `brush.dab_rotation_lag` | 0 |  |  |  | 0.6 |  | 0.8 |  | 1.5 |  |  |  |  |  |  |  |

## Regerar

```
python3 -I /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor/docs/Painter/Arquitetura/ferramentas/presets_matriz.py /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
```

Saídas: este ficheiro, `fotos/f1_presets/matriz.tsv` (linha `#padrão` logo abaixo do cabeçalho; célula vazia = chave ausente no preset) e `fotos/f1_presets/mapa_de_calor.png`.
