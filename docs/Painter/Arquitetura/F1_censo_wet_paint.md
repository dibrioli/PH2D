# F1 — Censo dos controlos do Wet Paint (conferido no CÓDIGO, 31/10)

> Leitor: a próxima LLM (F2/F3 da refatoração, [`PROMPT_REFATORACAO_WETPAINT_E_AJUDA.md`](../49_realshader/PROMPT_REFATORACAO_WETPAINT_E_AJUDA.md) §F1).
> Árvore: `line/PainterWatercolor` em `935f650ff8`. Os docs 45/46 NÃO foram usados como fonte — tudo daqui sai de grep no código.
> Abreviaturas de caminho: `wp/` = `crates/ph2d-wet-paint/src/`, `tp/` = `crates/ph2d-tool-painter/src/tool/paint/`, `gpu/` = `crates/ph2d-wet-gpu/src/`.
> Meio: **A** aquarela (lei `Cobre`, sem óleo) · **G** guache (Opaque/DryCover/WetCover) · **O** óleo «normal» (presets Oil sem a caixa) · **OR** Óleo Real (caixa Real Oil, `sim.oleo`). ⚠️ O motor NÃO tem enum de meio: o «meio» é uma combinação de knobs + 3 caixas; a coluna diz onde o knob muda o píxel na prática.

## 0. Números

| | n |
|---|---|
| knobs do registo (`wp/tuning/defs.rs:45`, `KNOB_COUNT`) | **144** |
| …visíveis no Tuning (`ph2d-panel-wet-tuning/src/rows.rs:96-100`, contagem `rows.rs:343-351`) | 95 |
| …no cartão Wet Paint (`tp/wetpaint_settings/tabelas.rs:128` 18 + `:83` 6 + 5 sliders de topo `tp/wetpaint_settings.rs:570-574`) | 29 |
| …só no preset (Hidden fora do cartão, e Paper ≠ emboss) | 30 |
| controlos FORA do registo de knobs (cartão + Real Oil + caixas, §2) | 33 |
| campos do preset fora de `engine` (`ph2d-brush-presets/src/modelo.rs:63-116` + 42 de `BrushSpec`, `tp/wetpaint_preset/campos.rs:5`) | 10 + 42 |
| **knobs MORTOS no produto** (só lidos por `render_region`, que só testes chamam) | **5** |
| knobs sem escritor de UI que DECIDEM (só o preset os põe) | 5 (Paper*, VisualGrain, PaperVisibility) + 22 Hidden do pincel |
| famílias de duplicados (§4) | **22** (D1..D22) |

**Escritores comuns a TODOS os knobs:** o preset (`tp/wetpaint_preset.rs:313-320`, mapa `engine` pela `key`), `WetKnobs::set` (`tp/wetpaint_settings.rs:160`, prende a `min..max`), o Tuning (`tp/wetpaint_settings/tuning.rs:30-34`), o reset (`tuning.rs:23`, `wetpaint_settings.rs:611`). A ponte para o motor é UMA: `tp/wetpaint/session.rs:236-247` (`e.set_knob` / `e.tuning.set` p/ `Rebuild::Render`). A coluna «quem ESCREVE» abaixo dá só o widget que o artista toca.

## 1. Os 144 knobs do registo

«decide?» = há um leitor de produção que muda o píxel (lido no código; os do grupo Material só com o Real Oil ligado ou com a célula a ter material). Readers listados sem testes/sondas/medidas (máx. 4).

| # | knob (`key`) | nome na tela | grupo | faixa · passo · fábrica motor | quem ESCREVE (UI) | quem LÊ (produção) | decide? | meio | grandeza aparente | dup / morto |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | `PigmentPerDab` (`pigmentPerDab`) | Pigment per dab | Paint | 0.0..2000.0 · 10.0 · 600.0 | Cartão: Pigment (`wetpaint_settings.rs:570`) | `tp/wetpaint_preset/preview.rs:488` `wp/tuning.rs:150` `wp/sim.rs:88` `wp/trail/deposit.rs:123` | sim | todos | massa de pigmento por dab (carga) | D1 (× Opacity `sliders.pigmento` × Intensity) |
| 1 | `PaperGate` (`paperGate`) | Paper gate | Paint | 0.0..1.5 · 0.01 · 0.6 | Tuning | `wp/tools.rs:96` `wp/trail.rs:596` `wp/trail/deposit.rs:124` | sim | todos | limiar de absorção do dente (quanto pigmento o papel prende) | D6 (com GateSaturation, CapillaryGate) |
| 2 | `Felt` (`felt`) | Felt (pores) | Hidden | 0.0..0.2 · 0.001 · 0.01 · rebuild Brush | só preset | `wp/painter.rs:224` `wp/brush/unidade.rs:17` | sim | todos (ponta) | forma da ponta (feltro) | Shape › Bristles (grupo Hidden, viaja pelo Shape) |
| 3 | `BristleCount` (`bristleCount`) | Bristle count | Hidden | 0.0..4000.0 · 10.0 · 950.0 · rebuild Brush | só preset | `wp/painter.rs:225` `wp/brush/unidade.rs:14` | sim | todos (ponta) | nº de cerdas | idem |
| 4 | `Drag` (`drag`) | Drag | Paint | 0.0..1.0 · 0.01 · 0.16 | Tuning | `wp/trail/transfer.rs:115` | sim | todos | fracção da janela arrastada (inércia do rasto), sem unidade | D2 (com Smear, RewetDrag, WaterDrag) |
| 5 | `Pickup` (`pickup`) | Pickup | Paint | 0.0..0.2 · 0.001 · 0.005 | Cartão: Pickup (`:571`) | `wp/trail/bico.rs:31` `wp/trail/bico.rs:184` | sim | todos | taxa de recolha do molhado pela ponta | D3 (com TipReservoir, TipClean, WetLift) |
| 6 | `Intensity` (`intensity`) | Intensity | Paint | 0.0..4.0 · 0.05 · 1.0 | Tuning | `wp/painter/doors.rs:24` | sim | todos | ganho do dab pós-clamp (sem unidade) | D1 (multiplica a carga) |
| 7 | `BristleStrength` (`bristleStrength`) | Bristle strength | Hidden | 0.0..4.0 · 0.05 · 1.0 · rebuild Brush | só preset | `wp/painter.rs:226` `wp/brush/unidade.rs:15` | sim | todos (ponta) | rigidez/contraste das cerdas | Shape |
| 8 | `BristleSize` (`bristleSize`) | Bristle size | Hidden | 0.2..4.0 · 0.05 · 1.0 · rebuild Brush | só preset | `wp/painter.rs:227` `wp/brush/unidade.rs:16` | sim | todos (ponta) | calibre da cerda | Shape |
| 9 | `Spacing` (`spacing`) | — | Hidden | 0.5..8.0 · 0.5 · 2.0 | só preset | `wp/painter.rs:394` `wp/painter.rs:425` `wp/painter/doors.rs:124` | sim | todos | passo do traço sintético (px) | D7 (com CapsuleSpacing, PigmentSpacing, `brush.spacing`) |
| 10 | `TipClean` (`tipClean`) | Tip clean | Paint | 0.0..0.05 · 0.0005 · 0.001 | Tuning | `wp/trail.rs:307` `wp/trail/bico.rs:21` | sim | todos | limpeza da ponta por dab | D3 |
| 11 | `BlendForce` (`blendForce`) | Blend force | Hidden | 0.0..1.0 · 0.01 · 0.8 | Cartão (linha da ferramenta) (`tabelas.rs:98`) | `wp/trail.rs:598` | sim | todas (ferr. Blend) | força do Blend | rótulo «Strength» igual a Smear/Blow |
| 12 | `GateSaturation` (`gateSaturation`) | Gate saturation | Paint | 100.0..6000.0 · 50.0 · 2000.0 | Tuning | `wp/tools.rs:97` `wp/trail.rs:597` `wp/trail/deposit.rs:125` | sim | todos | massa que satura o dente | D6 |
| 13 | `WaterPerDab` (`waterPerDab`) | Water per dab | Water | 0.0..4.0 · 0.05 · 1.0 | Tuning | `wp/tools.rs:184` `wp/painter/doors.rs:29` | sim | A/G | volume de água por dab (multiplicador) | D4 (× Water do cartão) |
| 14 | `WaterCap` (`waterCap`) | Water cap | Water | 1.0..30.0 · 0.5 · 8.0 | Tuning | `wp/tools.rs:185` `wp/tuning.rs:230` `wp/trail/transfer.rs:118` `wp/tools/smear.rs:74` | sim | A/G | tecto de água por célula | — |
| 15 | `Evaporation` (`evaporation`) | Evaporation | Water | 0.0..8.0 · 0.05 · 1.0 | Cartão: Dry speed (`:572`) | `wp/sim.rs:434` `wp/sim.rs:536` `gpu/passo.rs:692` | sim | A/G/O | taxa de evaporação | D5 (com BaseEvaporation, Retention, Dryer) |
| 16 | `Rewet` (`rewet`) | Re-wet | Water | 0.0..8.0 · 0.05 · 1.0 | Tuning | `wp/sim.rs:435` `wp/sim.rs:537` `wp/sim.rs:577` `gpu/passo.rs:693` | sim | A/G | taxa de re-molha do assentado pelo filme | D8 (rótulo vs `Dissolve` «Re-wet») |
| 17 | `Retention` (`retention`) | Retention | Water | 0.9..1.0 · 0.0005 · 0.995 | Tuning | `wp/drying.rs:121` `gpu/seca.rs:55` | sim | A/G | fracção de água retida por passo | D5 |
| 18 | `EdgeDarkening` (`edgeDarkening`) | Edge darkening | Water | 0.0..200.0 · 1.0 · 50.0 | Cartão: Edge (`:573`) | `wp/drying.rs:122` `gpu/seca.rs:56` | sim | A | deposição na orla (efeito café) | D9 (com EdgeSoftMass, Heavy Edge) |
| 19 | `BaseEvaporation` (`baseEvaporation`) | Base evaporation | Water | 0.0..20.0 · 0.1 · 2.0 | Tuning | `wp/drying.rs:123` `gpu/seca.rs:57` | sim | A/G/O | evaporação de base | D5 |
| 20 | `Leveling` (`leveling`) | Leveling | Hidden | 0.0..2.0 · 0.01 · 0.5 | Cartão: leveling (`tabelas.rs:129`) | `wp/solver.rs:93` `wp/solver/flow_jacobi.rs:123` `gpu/lib.rs:253` | sim | A/G/O | nivelamento da superfície de água (difusão de altura) | D10 (com LevelClamp, Viscosity, Brake) |
| 21 | `Capillary` (`capillary`) | Capillary | Hidden | 0.0..1.0 · 0.01 · 0.2 | Cartão: capillary (`tabelas.rs:131`) | `wp/solver.rs:96` `wp/solver/flow_jacobi.rs:127` `gpu/lib.rs:255` | sim | A | fluxo capilar pelo dente | D6 |
| 22 | `Brake` (`brake`) | Brake | Hidden | 0.0..4.0 · 0.05 · 1.5 | Cartão: brake (`tabelas.rs:132`) | `wp/solver.rs:99` `wp/solver/flow_jacobi.rs:130` `gpu/lib.rs:258` | sim | A | travão do fluxo na frente seca | D10 |
| 23 | `Gravity` (`gravity`) | Gravity | Physics | 0.0..0.05 · 0.0005 · 0.005 | Cartão: Gravity (`:574`) | `wp/sim.rs:321` | sim | A | aceleração da gota | — (Tilt + Heavy Edge correlatos) |
| 24 | `LevelClamp` (`levelClamp`) | Level clamp | Hidden | 0.0..1.0 · 0.01 · 0.2 | Cartão: level_clamp (`tabelas.rs:130`) | `wp/solver.rs:94` `wp/solver/flow_jacobi.rs:124` `gpu/lib.rs:254` | sim | A/G/O | tecto do nivelamento | D10 |
| 25 | `Viscosity` (`viscosity`) | Viscosity | Physics | 0.0..2.0 · 0.01 · 0.2 | Tuning | `wp/solver.rs:98` `wp/solver/flow_jacobi.rs:129` `gpu/lib.rs:257` | sim | A/G/O | viscosidade da água | D10 |
| 26 | `MaxVelocity` (`maxVelocity`) | Max velocity | Physics | 0.2..3.0 · 0.05 · 1.0 | Tuning | `wp/solver.rs:74` `wp/solver.rs:453` `wp/solver/advect.rs:24` `wp/solver/flow_jacobi.rs:122` +3 | sim | A | tecto de velocidade (CFL) | numérico, sem grandeza de produto |
| 27 | `Projection` (`projection`) | Projection | Physics | 0.0..1.5 · 0.01 · 0.7 | Tuning | `wp/solver/project.rs:44` `gpu/lib.rs:261` | sim | A | peso da projecção (incompressibilidade) | numérico |
| 28 | `BrakeReach` (`brakeReach`) | Brake reach | Physics | 1.0..12.0 · 0.5 · 4.0 | Tuning | `wp/solver.rs:100` `wp/solver/flow_jacobi.rs:131` `gpu/lib.rs:259` | sim | A | alcance do travão (células) | D10 |
| 29 | `CapillaryGate` (`capillaryGate`) | Capillary gate | Physics | 100.0..6000.0 · 50.0 · 2000.0 | Tuning | `wp/solver.rs:97` `wp/solver/flow_jacobi.rs:128` `gpu/lib.rs:256` | sim | A | limiar de massa do capilar | D6 |
| 30 | `Eraser` (`eraser`) | Eraser | Tools | 0.0..1.0 · 0.01 · 0.4 | Tuning | `wp/tools.rs:98` | sim | todos (borracha) | força da borracha molhada | **D11 duplicado** do slider `erase` do cartão (produto `force = Eraser × erase`, `tools.rs:98`) |
| 31 | `Dryer` (`dryer`) | Dryer | Hidden | 0.0..1.0 · 0.01 · 0.32 | Cartão (linha da ferramenta) (`tabelas.rs:104`) | `wp/tools.rs:267` | sim | todas (ferr. Dry) | absorção do secador | D5 |
| 32 | `Blow` (`blow`) | Blow | Hidden | 0.0..1.0 · 0.01 · 0.2 | Cartão (linha da ferramenta) (`tabelas.rs:110`) | `wp/tools.rs:320` | sim | ferr. Blow | força do sopro | rótulo «Strength» |
| 33 | `Smear` (`smear`) | Smear | Hidden | 0.0..1.0 · 0.01 · 0.8 | Cartão (linha da ferramenta) (`tabelas.rs:86`) | `wp/tools/smear.rs:48` `wp/tools/smear.rs:68` | sim | ferr. Smear | força do Smear | D2 |
| 34 | `PaperContrast` (`paperContrast`) | — | Paper | 0.2..2.5 · 0.05 · 1.0 · rebuild Paper | só preset (Paper fora do Tuning, `rows.rs:99`) | `wp/painter.rs:233` | sim (só papel do motor) | todos | contraste do papel gerado | sem escritor de UI; só com o papel do próprio motor (`session.rs` reconcile) |
| 35 | `PaperFibres` (`paperFibres`) | — | Paper | 0.0..3.0 · 0.05 · 1.0 · rebuild Paper | só preset (Paper fora do Tuning, `rows.rs:99`) | `wp/painter.rs:234` | sim (idem) | todos | fibras do papel gerado | idem |
| 36 | `PaperGrooves` (`paperGrooves`) | — | Paper | 0.0..3.0 · 0.05 · 1.0 · rebuild Paper | só preset (Paper fora do Tuning, `rows.rs:99`) | `wp/painter.rs:235` | sim (idem) | todos | sulcos do papel gerado | idem |
| 37 | `VisualGrain` (`visualGrain`) | — | Paper | 0.0..3.0 · 0.05 · 1.0 · rebuild Render | só preset (Paper fora do Tuning, `rows.rs:99`) | `wp/render.rs:67` `wp/grid/aparencia.rs:17` `wp/grid/aparencia.rs:52` | sim (na célula, `aparencia.rs:52`) | todos | grão impresso na tinta | D12 (com PaperVisibility, Paint Texture, PaperRelief); sem escritor de UI |
| 38 | `Emboss` (`emboss`) | Emboss | Paper | 0.0..4.0 · 0.05 · 1.0 · rebuild Render | Tuning | `wp/render.rs:68` `wp/trail/transfer.rs:120` | sim | todos | altura do relevo da tinta | D13 (com OilDepth, KnifeRidge, BristleHeight) |
| 39 | `PaperVisibility` (`paperVisibility`) | — | Paper | 0.0..1.5 · 0.01 · 1.0 · rebuild Render | só preset (Paper fora do Tuning, `rows.rs:99`) | `wp/render.rs:66` `wp/grid/aparencia.rs:15` `wp/grid/aparencia.rs:51` | sim (na célula) | todos | papel impresso na tinta | D12; sem escritor de UI |
| 40 | `ExtDiffusion` (`extDiffusion`) | Pigment diffusion | Hidden | 0.0..0.25 · 0.005 · 0.0 | Cartão: diffusion (`tabelas.rs:139`) | `wp/sim.rs:459` `wp/sim.rs:545` `wp/solver.rs:558` `gpu/passo.rs:26` | sim | A | difusão de pigmento | — |
| 41 | `ExtBackrun` (`extBackrun`) | Backruns (blooms) | Hidden | 0.0..2.0 · 0.05 · 0.0 | Cartão: backrun (`tabelas.rs:134`) | `wp/solver.rs:87` `wp/solver.rs:88` `wp/solver/flow_jacobi.rs:140` `wp/solver/flow_jacobi.rs:141` +1 | sim | A | flores/backrun | — |
| 42 | `ExtFingering` (`extFingering`) | Drip fingers | WetEffects | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/solver.rs:76` `wp/solver.rs:89` `wp/solver/flow_jacobi.rs:109` `wp/solver/flow_jacobi.rs:139` +1 | sim | A | instabilidade da frente (dedos) | — |
| 43 | `ExtRivulets` (`extRivulets`) | Number of fingers | WetEffects | 4.0..60.0 · 1.0 · 15.0 | Tuning | `wp/solver.rs:83` `wp/solver/flow_jacobi.rs:117` | sim (só se Fingering>0) | A | comprimento de onda dos dedos (px) | dependente de ExtFingering |
| 44 | `ExtGranulation` (`extGranulation`) | Granulation | Hidden | 0.0..1.0 · 0.01 · 0.0 | só preset | `wp/drying.rs:124` `gpu/seca.rs:58` | sim | A | granulação na secagem | D12 parcial (grão) |
| 45 | `ExtStaining` (`extStaining`) | Staining | Hidden | 0.0..1.0 · 0.01 · 0.5 | Cartão: staining (`tabelas.rs:148`) | `wp/tools.rs:33` `wp/tools.rs:104` `wp/drying.rs:125` `wp/tools/smear.rs:75` +1 | sim | A/G | aderência do assentado (resiste ao levantar) | D14 (com Solubility: inversos) |
| 46 | `ExtDryBrush` (`extDryBrush`) | Dry brush | Stroke | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/trail/deposit.rs:126` | sim | A/O | pincel seco (falha no dente) | D15 (com `dry_gate` derivado da água, `doors.rs:40`) |
| 47 | `ExtWetSoften` (`extWetSoften`) | Softer on wet paint | Hidden | 0.0..1.0 · 0.01 · 0.0 | Cartão: wet_soften (`tabelas.rs:153`) | `wp/trail/deposit.rs:127` | sim | A | amolecimento em molhado | — |
| 48 | `ExtRakeLight` (`extRakeLight`) | — | Hidden | 0.0..3.0 · 0.05 · 0.0 · rebuild Render | só preset | `wp/render.rs:70` | **não** (só `render_region`, chamado só por testes) | — | luz rasante do papel | **MORTO no produto** |
| 49 | `ExtValleyGran` (`extValleyGran`) | — | Hidden | 0.0..1.0 · 0.01 · 0.0 · rebuild Render | só preset | `wp/render.rs:71` | **não** (idem) | — | grão nos vales | **MORTO no produto** |
| 50 | `ExtWetSheen` (`extWetSheen`) | — | Hidden | 0.0..2.0 · 0.05 · 0.0 · rebuild Render | só preset | `wp/render.rs:72` | **não** (idem) | — | brilho do molhado | **MORTO no produto** |
| 51 | `ExtGrainDither` (`extGrainDither`) | — | Hidden | 0.0..8.0 · 0.1 · 0.0 · rebuild Render | só preset | `wp/render.rs:73` | **não** (idem) | — | dither do grão | **MORTO no produto** |
| 52 | `ExtEdgeTint` (`extEdgeTint`) | — | Hidden | 0.0..1.0 · 0.01 · 0.5 · rebuild Render | só preset | `wp/render.rs:75` | **não** (idem) | — | tinta da orla | **MORTO no produto** |
| 53 | `WetLift` (`wetLift`) | Rewet lift | Tools | 0.0..1.0 · 0.01 · 0.25 | Tuning | `wp/tools.rs:33` | sim | ferr. Water | levantar molhado pela água | D3 |
| 54 | `Glaze` (`glaze`) | Brush glaze | Hidden | 0.0..1.0 · 0.01 · 0.0 | Cartão: glaze (`tabelas.rs:149`) | `wp/trail/transfer.rs:128` | sim | A | fracção do filme que FILTRA o assentado (`vela`) | **D16** (com a caixa Glaze layering `km_glaze` → `vidrado`, e `glaze`/`glaze_filme` de sessão) |
| 55 | `Dust` (`dust`) | Brush dust | Hidden | 0.0..1.0 · 0.01 · 0.0 | Cartão: dust (`tabelas.rs:133`) | `tp/wetpaint/pousa_o_dab.rs:237` `wp/tuning.rs:191` `wp/sim.rs:158` `wp/trail/deposit.rs:58` +2 | sim | A (pó/pastel) | fracção do pigmento em grãos | — |
| 56 | `PressureSize` (`pressureSize`) | Pressure size | Paint | 0.0..1.0 · 0.01 · 0.9 | Tuning · mão (`mao.rs`) | `tp/mao.rs:40` `tp/mao.rs:249` `tp/mao.rs:289` `tp/mao.rs:366` +1 | sim (dinâmica) | todos | pressão → tamanho | mão |
| 57 | `PressureOpacity` (`pressureOpacity`) | Pressure opacity | Paint | 0.0..1.0 · 0.01 · 0.0 | Tuning · mão (`mao.rs`) | `tp/mao.rs:42` `tp/mao.rs:228` `tp/mao.rs:255` `tp/mao.rs:297` +2 | sim (dinâmica) | todos | pressão → carga | D1 (escala a carga) |
| 58 | `PressureOpacityCurve` (`pressureOpacityCurve`) | Pressure opacity curve | Paint | 0.25..4.0 · 0.05 · 1.0 | Tuning · mão (`mao.rs`) | `tp/mao.rs:43` `tp/mao.rs:228` `tp/mao.rs:261` `tp/mao.rs:304` +2 | sim | todos | curva | — |
| 59 | `PressureSizeCurve` (`pressureSizeCurve`) | Pressure size curve | Paint | 0.25..4.0 · 0.05 · 1.0 | Tuning · mão (`mao.rs`) | `tp/mao.rs:41` `tp/mao.rs:253` `tp/mao.rs:296` `tp/mao.rs:367` | sim | todos | curva | — |
| 60 | `PressureFloor` (`pressureFloor`) | Pressure floor | Paint | 0.0..0.9 · 0.01 · 0.0 | Tuning · mão (`mao.rs`) | `tp/mao.rs:44` `tp/mao.rs:262` `tp/mao.rs:305` `tp/mao.rs:370` | sim | todos | piso da caneta | — |
| 61 | `PressureCeiling` (`pressureCeiling`) | Pressure ceiling | Paint | 0.1..1.0 · 0.01 · 1.0 | Tuning · mão (`mao.rs`) | `tp/mao.rs:45` `tp/mao.rs:263` `tp/mao.rs:306` `tp/mao.rs:371` | sim | todos | tecto da caneta | — |
| 62 | `PressureSmoothing` (`pressureSmoothing`) | Pressure smoothing | Paint | 0.0..0.9 · 0.01 · 0.0 | Tuning · mão (`mao.rs`) | `tp/mao.rs:46` `tp/mao.rs:264` `tp/mao.rs:307` `tp/mao.rs:372` | sim | todos | suavização | — |
| 63 | `PulledString` (`pulledString`) | Pulled string | Paint | 0.0..400.0 · 1.0 · 0.0 | Tuning · mão (`mao.rs`) | `tp/mao.rs:47` `tp/mao.rs:265` `tp/mao.rs:308` `tp/mao.rs:373` | sim | todos | estabilizador (px) | D17 (com `stabilizer` do BrushSpec) |
| 64 | `DabRotation` (`dabRotation`) | Dab rotation (none · follow · pen tilt) | Paint | 0.0..2.0 · 1.0 · 0.0 | Tuning · mão (`mao.rs`) | `tp/mao.rs:48` `tp/mao.rs:215` `tp/mao.rs:266` `tp/mao.rs:309` +1 | sim | todos | modo de rotação (enum 0..2) | — |
| 65 | `TiltStart` (`tiltStart`) | Tilt start | Paint | 0.0..90.0 · 0.5 · 37.5 | Tuning · mão (`mao.rs`) | `tp/mao.rs:49` `tp/mao.rs:267` `tp/mao.rs:314` | sim | todos | ângulo (°) | — |
| 66 | `TiltFull` (`tiltFull`) | Tilt full | Paint | 0.0..90.0 · 0.5 · 67.5 | Tuning · mão (`mao.rs`) | `tp/mao.rs:50` `tp/mao.rs:268` `tp/mao.rs:315` | sim | todos | ângulo (°) | — |
| 67 | `TiltWidth` (`tiltWidth`) | Tilt width | Paint | 0.25..5.0 · 0.05 · 1.0 | Tuning · mão (`mao.rs`) | `tp/mao.rs:51` `tp/mao.rs:269` `tp/mao.rs:316` | sim | todos | ganho de largura | — |
| 68 | `TiltLength` (`tiltLength`) | Tilt length | Paint | 0.25..5.0 · 0.05 · 1.0 | Tuning · mão (`mao.rs`) | `tp/mao.rs:52` `tp/mao.rs:270` `tp/mao.rs:317` | sim | todos | ganho de comprimento | — |
| 69 | `TiltSmooth` (`tiltSmooth`) | Tilt smooths the grain | Paint | 0.0..1.0 · 0.01 · 0.0 | Tuning · mão (`mao.rs`) | `tp/mao.rs:53` `tp/mao.rs:271` `tp/mao.rs:318` | sim | todos | suavização | — |
| 70 | `TiltLag` (`tiltLag`) | Tilt lag | Paint | 0.0..500.0 · 1.0 · 0.0 | Tuning · mão (`mao.rs`) | `tp/mao.rs:54` `tp/mao.rs:272` `tp/mao.rs:319` | sim | todos | atraso (ms) | — |
| 71 | `BristlePressure` (`bristlePressure`) | Pressure per bristle | Bristles | 0.0..3.0 · 0.01 · 0.0 | Tuning | **nenhum** | sim | O/todos | pressão → abertura dos fios | — |
| 72 | `BristleNoiseLength` (`bristleNoiseLength`) | Pressure change length | Bristles | 10.0..2000.0 · 1.0 · 150.0 | Tuning | **nenhum** | sim | O | comprimento do ruído (px) | — |
| 73 | `BristleLaneWidth` (`bristleLaneWidth`) | Bristle width | Bristles | 0.5..24.0 · 0.1 · 3.0 | Tuning | **nenhum** | sim | O | largura do fio (px) | — |
| 74 | `BristleWave` (`bristleWave`) | Width wave | Bristles | 0.0..0.75 · 0.01 · 0.0 | Tuning | **nenhum** | sim | O | ondulação | — |
| 75 | `BristleWaveLength` (`bristleWaveLength`) | Width wave length | Bristles | 50.0..2000.0 · 1.0 · 500.0 | Tuning | **nenhum** | sim | O | comprimento da onda (px) | — |
| 76 | `BristleOscillation` (`bristleOscillation`) | Brush wobble | Bristles | 0.0..12.0 · 0.1 · 0.0 | Tuning | **nenhum** | sim | O | oscilação (px) | — |
| 77 | `BristleSpan` (`bristleSpan`) | Random span | Bristles | 0.0..12.0 · 0.1 · 0.0 | Tuning | **nenhum** | sim | O | abertura (px) | — |
| 78 | `BristleSpanLength` (`bristleSpanLength`) | Random span length | Bristles | 1.0..200.0 · 0.5 · 10.0 | Tuning | **nenhum** | sim | O | comprimento da abertura (px) | — |
| 79 | `BristleNoiseHue` (`bristleNoiseHue`) | Hue jitter per bristle | Bristles | 0.0..0.5 · 0.005 · 0.0 | Tuning | **nenhum** | sim | O | ruído de matiz (voltas) | cor, não física |
| 80 | `BristleNoiseSat` (`bristleNoiseSat`) | Saturation jitter per bristle | Bristles | 0.0..0.5 · 0.005 · 0.0 | Tuning | **nenhum** | sim | O | ruído de saturação | cor |
| 81 | `BristleNoiseLight` (`bristleNoiseLight`) | Lightness jitter per bristle | Bristles | 0.0..0.7 · 0.005 · 0.0 | Tuning | **nenhum** | sim | O | ruído de luminosidade | cor |
| 82 | `BristleLift` (`bristleLift`) | Light touch lifts bristles | Bristles | 0.0..2.0 · 0.01 · 0.0 | Tuning | **nenhum** | sim | O | levantar dos fios | — |
| 83 | `BristleHeight` (`bristleHeight`) | Bristle relief | Bristles | 0.0..8.0 · 0.01 · 0.0 | Tuning | **nenhum** | sim | O/OR | altura do sulco da cerda (px) | D13 |
| 84 | `BristleRemove` (`bristleRemove`) | Bristles that dig | Bristles | 0.0..1.0 · 0.01 · 0.5 | Tuning | **nenhum** | sim | O/OR | escavação do sulco | D13 |
| 85 | `PigmentSpacing` (`pigmentSpacing`) | — | Hidden | 0.0..0.5 · 0.005 · 0.025 | só preset | `tp/wetpaint/dab_route.rs:213` | sim | todos | passo do pigmento (fracção do diâmetro) | D7 |
| 86 | `CapsuleSpacing` (`capsuleSpacing`) | — | Hidden | 0.005..0.1 · 0.0025 · 0.025 | só preset | `tp/wetpaint/lote.rs:87` `wp/trail.rs:483` | sim | todos | passo da cápsula | D7 |
| 87 | `CapsuleMaxPx` (`capsuleMaxPx`) | — | Hidden | 0.0..64.0 · 0.5 · 0.0 | só preset | `tp/wetpaint/lote.rs:88` | sim | todos | tecto da cápsula (px) | D7 |
| 88 | `CapsuleRoundNearest` (`capsuleRoundNearest`) | — | Hidden | 0.0..1.0 · 1.0 · 0.0 | só preset | `tp/wetpaint/lote.rs:102` | sim | todos | arredondar (bool) | numérico |
| 89 | `EmbossSizeExp` (`embossSizeExp`) | — | Hidden | 0.0..1.0 · 0.01 · 0.0 | só preset | `tp/wetpaint/lote.rs:132` `tp/wetpaint/lote.rs:143` `wp/sim.rs:161` | sim | todos | escala do relevo com o diâmetro | D18 (com TextureSizeExp, BristleLaneSizeExp) |
| 90 | `EmbossSizeRef` (`embossSizeRef`) | — | Hidden | 1.0..1000.0 · 0.5 · 41.0 | só preset | `tp/wetpaint/lote.rs:146` | sim | todos | diâmetro de referência (px) | D18 |
| 91 | `BristleTipRagged` (`bristleTipRagged`) | Ragged bristle tips | Bristles | 0.0..1.0 · 0.01 · 0.0 | Tuning | `tp/wetpaint/ponta_das_cerdas.rs:2` `tp/wetpaint/pousa_o_dab.rs:359` | sim | O | ponta esfiapada | — |
| 92 | `LevelingOwnStroke` (`levelingOwnStroke`) | Strokes keep apart (off · on) | Stroke | 0.0..1.0 · 1.0 · 0.0 | Tuning | `wp/sim.rs:168` `wp/grid/traco.rs:2` `wp/trail/transfer.rs:123` | sim | O/G | o nivelamento isola o traço (bool) | D10 |
| 93 | `TipReservoir` (`tipReservoir`) | Tip picks up and smears | Hidden | 0.0..1.0 · 0.01 · 0.0 | Cartão: tip_pickup (`tabelas.rs:141`) | `wp/trail/bico.rs:32` `wp/trail/bico.rs:108` `wp/trail/bico.rs:183` `wp/trail/bico.rs:211` +1 | sim | todos | reservatório da ponta (apanha o molhado de outra cor) | D3 |
| 94 | `PressureGrain` (`pressureGrain`) | Light pressure breaks into grain | Stroke | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/trail/deposit.rs:116` `wp/trail/deposit.rs:135` | sim | todos | pressão → grão | — |
| 95 | `StrokeFilm` (`strokeFilm`) | Smooth wash body (off · on) | Stroke | 0.0..1.0 · 1.0 · 0.0 | Tuning | `tp/wetpaint/pousa_o_dab.rs:474` `tp/wetpaint/dab_route.rs:190` `wp/trail.rs:22` `wp/trail.rs:143` +7 | sim | O/G | modo «filme do traço» (bool): a água é a do filme | ramo por meio (§2) |
| 96 | `DustDrop` (`dustDrop`) | Dust grain size | Stroke | 1.0..32.0 · 0.5 · 2.0 | Tuning | `wp/trail/deposit.rs:58` `wp/trail/deposit.rs:133` | sim | A (pó) | tamanho do grão (px) | — |
| 97 | `BristleStreak` (`bristleStreak`) | Dark streaks | Bristles | 0.0..1.0 · 0.005 · 0.0 | Tuning | `wp/bristles_fios.rs:47` | sim | O | veio claro/escuro | — |
| 98 | `BristleStreakLight` (`bristleStreakLight`) | Light streaks | Bristles | 0.0..1.0 · 0.005 · 0.0 | Tuning | **nenhum** | sim | O | luz do veio | — |
| 99 | `BristleStreakLength` (`bristleStreakLength`) | Streak length | Bristles | 10.0..4000.0 · 1.0 · 300.0 | Tuning | **nenhum** | sim | O | comprimento do veio (px) | — |
| 100 | `BristleStreakCut` (`bristleStreakCut`) | Streak rarity | Bristles | 0.0..0.95 · 0.01 · 0.5 | Tuning | **nenhum** | sim | O | corte do veio | — |
| 101 | `ThinStrokeFade` (`thinStrokeFade`) | — | Hidden | 0.0..4.0 · 0.01 · 0.0 | só preset | `tp/wetpaint/rota_da_lane.rs:17` `tp/wetpaint/rota_da_lane.rs:22` | sim | todos | esbater do traço fino | — |
| 102 | `ThinStrokeRef` (`thinStrokeRef`) | — | Hidden | 1.0..400.0 · 0.5 · 40.0 | só preset | `tp/wetpaint/rota_da_lane.rs:26` `wp/tuning.rs:206` | sim | todos | diâmetro de referência (px) | — |
| 103 | `WaterPush` (`waterPush`) | Water pushes pigment | Tools | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/painter/doors.rs:367` `wp/tools/bordas.rs:2` `wp/tools/bordas.rs:26` `wp/tools/bordas.rs:34` | sim | ferr. Water | empurrar água na borda | — |
| 104 | `DryerEdge` (`dryerEdge`) | Dryer leaves an edge | Tools | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/painter/doors.rs:381` `wp/tools/bordas.rs:2` `wp/tools/bordas.rs:199` `wp/tools/bordas.rs:206` | sim | ferr. Dry | orla do secador | D5 |
| 105 | `WetOpen` (`wetOpen`) | Wet-in-wet bloom | Hidden | 0.0..1.0 · 0.01 · 0.0 | Cartão: wet_bloom (`tabelas.rs:140`) | `wp/trail.rs:25` `wp/trail.rs:145` `wp/tuning.rs:227` `wp/tuning.rs:292` +5 | sim | A/G/O | abertura do molhado alheio (Wet bloom) | D19 (com WetOpenPush, WetOpenEdge) |
| 106 | `WetOpenPush` (`wetOpenPush`) | Bloom pushes paint away | WetEffects | 0.0..1.0 · 0.01 · 0.8 | Tuning | `wp/trail/molhado.rs:3` `wp/trail/molhado.rs:554` | sim | idem | empurrão da abertura | D19 |
| 107 | `SmearWater` (`smearWater`) | — | Hidden | 0.0..1.0 · 0.01 · 0.5 | Cartão (linha da ferramenta) (`tabelas.rs:92`) | `wp/tools/smear.rs:70` `wp/tools/smear.rs:71` | sim | ferr. Smear | água do Smear | D2/D4 |
| 108 | `KnifeRidge` (`knifeRidge`) | Knife ridge | Stroke | 0.0..16.0 · 0.01 · 0.0 | Tuning | `tp/wetpaint/dab_route.rs:200` | sim | O/OR (faca) | crista da faca (px) | D13 |
| 109 | `TextureSizeExp` (`textureSizeExp`) | Texture follows brush size | Stroke | 0.0..1.5 · 0.01 · 0.0 | Tuning | `tp/wetpaint/lote.rs:153` `tp/wetpaint/lote.rs:160` `wp/tuning.rs:273` | sim | todos | escala da textura com o diâmetro | D18 |
| 110 | `TextureSizeRef` (`textureSizeRef`) | Texture size reference | Stroke | 1.0..1000.0 · 0.5 · 41.0 | Tuning | `tp/wetpaint/lote.rs:163` `tp/wetpaint/lote.rs:171` `tp/wetpaint/lote.rs:177` `tp/wetpaint/lote.rs:218` | sim | todos | diâmetro de referência (px) | D18 |
| 111 | `Metallic` (`metallic`) | Metallic | Material | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/grid/material.rs:20` `wp/grid/material.rs:25` | sim (OR) | OR | metalicidade (Principled) | **D20** (= cartão Real Oil Metallic, `max`, `material.rs:146`) |
| 112 | `Roughness` (`roughness`) | Roughness | Material | 0.0..1.0 · 0.01 · 0.5 | Tuning | `wp/tuning.rs:374` `wp/grid/material.rs:26` | sim (OR) | OR | rugosidade | **D20** (× cartão, `material.rs:147`) |
| 113 | `IorLevel` (`iorLevel`) | IOR Level | Material | 0.0..1.0 · 0.01 · 0.5 | Tuning | `wp/tuning.rs:375` `wp/grid/material.rs:27` | sim (OR) | OR | nível do IOR | **D20** |
| 114 | `Coat` (`coat`) | Coat | Material | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/tuning.rs:376` `wp/grid/material.rs:28` | sim (OR) | OR | verniz | **D20** |
| 115 | `CoatRoughness` (`coatRoughness`) | Coat Roughness | Material | 0.0..1.0 · 0.01 · 0.03 | Tuning | `wp/tuning.rs:377` `wp/grid/material.rs:29` | sim (OR) | OR | rugosidade do verniz | **D20** |
| 116 | `Sheen` (`sheen`) | Sheen | Material | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/grid/material.rs:30` | sim (OR) | OR | veludo | **D20** |
| 117 | `Subsurface` (`subsurface`) | Subsurface | Material | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/tuning.rs:378` `wp/grid/material.rs:31` | sim (OR) | OR | subsuperfície | **D20** |
| 118 | `SubsurfaceRadius` (`subsurfaceRadius`) | Subsurface Radius | Material | 0.0..16.0 · 0.1 · 4.0 | Tuning | `wp/grid/material.rs:32` | sim (OR) | OR | raio SSS (px) | **D20** |
| 119 | `Transparency` (`transparency`) | Transparency | Material | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/grid/material.rs:33` | sim (OR) | OR (e cobertura) | transparência da tinta | D21 (com Transmission, TransmissionMode, Opaque, WetCover) |
| 120 | `LockMaterial` (`lockMaterial`) | Lock material (off · on) | Material | 0.0..1.0 · 1.0 · 0.0 | Tuning | `wp/grid/material.rs:41` `wp/grid/material.rs:44` | sim | OR | trava o material pousado (bool) | — |
| 121 | `KnifeLevel` (`knifeLevel`) | Knife leveling | Stroke | 0.0..1.0 · 0.01 · 0.0 | Tuning | `tp/wetpaint/dab_route.rs:206` `wp/lamina.rs:69` `wp/lamina.rs:85` `wp/oleo_vista_prova.rs:22` +1 | sim | O/OR (faca) | nivelamento pela lâmina | D10/D13 |
| 122 | `TextureSizeExpAcross` (`textureSizeExpAcross`) | Texture follows size across | Stroke | -0.01..1.5 · 0.01 · -0.01 | Tuning | `tp/wetpaint/lote.rs:195` `tp/wetpaint/lote.rs:205` `tp/wetpaint/lote.rs:213` | sim | todos | idem através (−0,01 = desligado) | D18 (sentinela −0,01) |
| 123 | `Transmission` (`transmission`) | Transmission | Material | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/grid/material.rs:34` | sim (OR) | OR | transmissão | D21 |
| 124 | `PaperRelief` (`paperRelief`) | Paper Relief | Material | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/grid/material.rs:35` | sim (OR) | OR | relevo do papel visto através da tinta | D12 |
| 125 | `TransmissionMode` (`transmissionMode`) | Transmission mode | Material | 0.0..1.0 · 1.0 · 0.0 | Tuning | `wp/grid/material.rs:36` | sim (OR) | OR | vidro claro (bool) | D21 |
| 126 | `OilDepth` (`oilDepth`) | Depth | Material | 0.0..3.0 · 0.01 · 1.0 | Tuning | `wp/grid/oleo_celula.rs:19` `wp/grid/oleo_celula.rs:55` | sim (só com Real Oil) | OR | ganho de altura do óleo | D13 (`ganho = Depth × oleo`, `oleo_celula.rs:65`) |
| 127 | `OilGloss` (`oilGloss`) | Gloss | Material | 0.0..1.0 · 0.01 · 1.0 | Tuning | `wp/grid/oleo_celula.rs:21` `wp/grid/oleo_celula.rs:56` | sim (só com Real Oil) | OR | força especular | D22 (com Coat/Roughness) |
| 128 | `WetOpenEdge` (`wetOpenEdge`) | Bloom edge softness | WetEffects | 0.0..80.0 · 0.5 · 0.0 | Tuning | `wp/sim.rs:165` `wp/trail/molhado.rs:19` `wp/trail/molhado.rs:36` `wp/trail/molhado.rs:319` +2 | sim | A/G/O | orla da abertura (px) | D19 |
| 129 | `StrokeFilmDome` (`strokeFilmDome`) | Wash body: loaded centre, soft sides | Stroke | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/trail.rs:485` `wp/trail/filme.rs:110` | sim (só com StrokeFilm) | O/G | cúpula do filme | dependente de StrokeFilm |
| 130 | `StrokeFilmEnds` (`strokeFilmEnds`) | Wash body: soft ends | Stroke | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/trail.rs:486` `wp/trail/filme.rs:112` | sim (só com StrokeFilm) | O/G | pontas do filme | idem |
| 131 | `DryerLevel` (`dryerLevel`) | Dryer evens the paint (brush radii) | Tools | 0.0..2.0 · 0.01 · 0.0 | Tuning | `wp/painter/doors.rs:381` `wp/tools/secador.rs:1` `wp/tools/secador.rs:16` `wp/tools/secador.rs:20` | sim | ferr. Dry | nivelar ao secar | D5/D10 |
| 132 | `BlendReach` (`blendReach`) | Blender reach (brush radii) | Tools | 0.0..1.0 · 0.01 · 0.0 | Tuning | `wp/trail/transfer.rs:12` `wp/trail/transfer.rs:469` `wp/trail/transfer.rs:560` `wp/trail/transfer/perto.rs:1` | sim | ferr. Blend | alcance do Blend | — |
| 133 | `WetCover` (`wetCover`) | Covers wet paint underneath | Hidden | 0.0..1.0 · 0.01 · 0.0 | Cartão: cover (`tabelas.rs:150`) | `wp/tuning.rs:345` `wp/trail/transfer.rs:12` `wp/trail/transfer.rs:129` `wp/trail/transfer/cobre.rs:1` +1 | sim | G/O | cobertura do molhado (guache sobre molhado) | **D21** (cobertura: WetCover, DryCover, Opaque, esconde de sessão) |
| 134 | `BristleLaneSizeExp` (`bristleLaneSizeExp`) | Bristles follow brush size | Bristles | 0.0..1.5 · 0.01 · 0.0 | Tuning | `tp/wetpaint/lote.rs:169` `tp/wetpaint/lote.rs:174` `wp/bristles_fios.rs:75` | sim | O | escala do fio com o diâmetro | D18 |
| 135 | `PaperPress` (`paperPress`) | Paper Press | Material | 0.0..1.0 · 0.01 · 1.0 | Tuning | `wp/material.rs:48` `wp/grid/material.rs:37` | sim | O/OR | a tinta amassa o dente do papel | D12 |
| 136 | `Dissolve` (`rewetPaint`) | — | Hidden | 0.0..1.0 · 0.01 · 0.0 | Cartão: rewet (`tabelas.rs:143`) | `wp/trail.rs:152` `wp/tuning.rs:332` `wp/trail/transfer.rs:12` `wp/trail/transfer.rs:133` +2 | sim | A/G | água do pincel que dissolve tinta solúvel («Re-wet») | D8 (com Rewet, Solubility) |
| 137 | `Solubility` (`solubility`) | — | Hidden | 0.0..1.0 · 0.01 · 0.0 | Cartão: solubility (`tabelas.rs:145`) | `wp/tuning.rs:328` `wp/grid/oleo_celula.rs:23` `wp/grid/oleo_celula.rs:46` | sim | A/G | quanto a tinta pousada se dissolve depois | D8/D14 |
| 138 | `RewetDrag` (`rewetDrag`) | — | Hidden | 0.0..1.0 · 0.01 · 0.0 | Cartão: rewet_drag (`tabelas.rs:144`) | `wp/painter.rs:320` `wp/painter/doors.rs:216` `wp/tools/smear.rs:31` `wp/tools/smear.rs:66` | sim | A/G | arrasto da tinta re-molhada | D2 |
| 139 | `EdgeSoftMass` (`edgeSoftMass`) | Edge darkening fades with the paint | Water | 0.0..8.0 · 0.1 · 0.0 | Tuning | `wp/drying.rs:300` `wp/drying.rs:483` | sim | A | borda graduada (massa) | D9 |
| 140 | `DryCover` (`dryCover`) | — | Hidden | 0.0..1.0 · 0.01 · 0.0 | Cartão: dry_cover (`tabelas.rs:152`) | `wp/trail/transfer.rs:131` `wp/trail/transfer/cobre.rs:2` `wp/trail/transfer/cobre.rs:30` | sim | G/O | cobertura sobre o assentado | **D21** |
| 141 | `TuftArc` (`tuftArc`) | Tufts fan out in an arc | Bristles | 0.0..1.0 · 0.01 · 0.0 | Tuning | `tp/wetpaint/pousa_o_dab.rs:555` | sim | O | arco do tufo | — |
| 142 | `WaterDrag` (`waterDrag`) | — | Hidden | 0.0..1.0 · 0.01 · 0.0 | Cartão (linha da ferramenta) (`tabelas.rs:117`) | `wp/painter/doors.rs:362` `wp/tools/arrasto_agua.rs:1` `wp/tools/arrasto_agua.rs:6` `wp/tools/arrasto_agua.rs:21` | sim | ferr. Water | arrasto pela água | D2 |
| 143 | `Opaque` (`opaque`) | — | Hidden | 0.0..1.0 · 0.01 · 0.0 | só preset | `wp/assenta.rs:180` `wp/grid/aparencia.rs:25` `wp/grid/aparencia.rs:55` `gpu/passo.rs:18` +1 | sim | G/O | prende o ponto ao segmento (mistura opaca K-M) | **D21** (= ramo `com_opaca`) |

## 2. Controlos fora do registo de knobs

| nome na tela | id / campo | quem ESCREVE | quem LÊ | decide? | faixa | meio | grandeza | dup / nota |
|---|---|---|---|---|---|---|---|---|
| Water | `PAINTER_WETPAINT_WATER` → `knobs.water` → `sliders.water` | `tp/wetpaint_settings.rs:569`; preset `volumes.water` `tp/wetpaint_preset.rs:447` | `wp/painter/doors.rs:25-29` (`water_amount = 2·(0,35w)²/(p+0,01)·WaterPerDab`) | sim | 0..1 | A/G/O | volume de água por dab | **D4** com `WaterPerDab`; quadrático e INVERSO da pressão |
| Opacity (do pincel) | `wetpaint.opacidade` → `sliders.pigmento` | `tp/wetpaint_preset.rs:444`; snapshot `tp/snapshot.rs:465` | `wp/painter.rs:287` (`com_pigmento`), `doors.rs:174,467`, `painter/po.rs:28` | sim | 0..1 | todos | fracção de pigmento | **D1** com `PigmentPerDab`, `Intensity`, `PressureOpacity` |
| Erase | `PAINTER_WETPAINT_ERASE` → `knobs.erase` → `sliders.erase` | `tp/wetpaint_settings.rs:581` | `wp/painter/doors.rs:254` → `wp/tools.rs:98` | sim | 0..1 | todos | força da borracha | **D11** com `Knob::Eraser` (produto) |
| Pigment mixing (K-M) | `km_mixing` | `tp/wetpaint_settings.rs:661`; preset `:385` | `session.rs:258` → `sim.km_mixing` + `sim.glaze = VELATURA_MEDIDA` se OFF; `wp/sim.rs:329` (`ColorMix::Km/Plain`), `wp/solver/advect.rs:26` | sim | bool | todos | lei de mistura (K-M × RGB) | **ramo** (§3 R9); OFF = mistura RGB + velatura |
| Glaze layering (K-M) | `km_glaze` (`WET_TUNING_KM_GLAZE`) | `tp/wetpaint_settings.rs:667`; preset `:383` | `session.rs:269` → `sim.camadas_transparentes` → `aparencia.vidrado` → `Ecra::com_vidrado` (`wp/assenta.rs:122`) | sim | bool | A | `esconde` → `TINTA_ESCONDE_EM_VELATURA` | **D16** com `Knob::Glaze` (vela) |
| Paint Texture | `paper_visual` (`PAINTER_WETPAINT_PAPER_VISUAL`) | `tp/wetpaint_settings.rs:638`; preset `:386` | `sim.textura` → `wp/grid/aparencia.rs:54,98`, `wp/render.rs:530` | sim | bool | todos | imprime o papel na tinta | **D12** |
| Real Oil | `oleo_real` (`PAINTER_WETPAINT_OLEO_REAL`) → `sim.oleo` | `tp/wetpaint_settings.rs:643`; preset `CHAVE_OLEO` `tp/wetpaint_preset.rs:388` | `wp/sim.rs:349-353` (`PINCEL_DE_OLEO`), `sim.rs` `Params.oleo` → `grid/oleo_celula.rs:47`, `trail/transfer.rs:174`, `solver.rs:104`, `painter/doors.rs:85`; óptica `tp/wetpaint/composite_oleo.rs:161` | sim | bool | OR | «este traço é óleo» | **o maior ramo** (§3 R3–R6) |
| Shadow / Light height / Light angle / Crevice shadow / Crevice reach / Environment light | `OleoReal::{sombra,altitude,azimute,fenda,alcance_fenda,forca_ambiente}` (`tp/ids/painter_wetpaint.rs:116-160`) | `tp/wetpaint_settings/tela_e_oleo.rs:81-` | `wp/oleo_real.rs`, `oleo_sombra.rs`, `oleo_luz.rs`, `oleo_ambiente.rs` (render) | sim (só vista) | 0..1 · 5..90° · 0..360° · 0..`FENDA_MAX` · 1..`ALCANCE_FENDA_MAX` · 0..1 | OR | luz da CENA, não da tinta | globais da tela, NÃO viajam no preset |
| Environment | `OleoReal::ambiente` | `tp/ids/painter_wetpaint.rs:186` | `wp/oleo_ambiente.rs` | sim | enum 4 | OR | céu reflectido | cena |
| Metallic…Subsurface Radius (8, cartão Real Oil) | `MaterialCartao` (`wp/oleo_material.rs:24`) | `tela_e_oleo.rs` (ids `PAINTER_WETPAINT_OLEO_MATERIAL`) | `wp/material.rs:138-165` `efectivo(célula, cartão)` | sim | 0..1; raio 0..`RAIO_SUBSUPERFICIE_MAX` | OR | Principled | **D20**: a MESMA grandeza que os knobs Material da célula, combinada por `max` (metal, verniz, veludo, SSS) ou por razão (rugosidade, IOR, raio) |
| Live Water | `agua_viva` | `tp/wetpaint_settings.rs:653`; preset `live_water` `:409` | `session.rs:279` → `sim_while_painting` + `glaze_filme = VELATURA_DO_FILME(_VIVA)` | sim | bool | A | modo de agendamento — e MUDA a velatura do filme | **ramo** de cor escondido num controlo de desempenho (§3 R10) |
| Heavy Edge | `borda_que_pesa` | `tp/wetpaint_settings.rs:559`; preset `heavy_edge` `:411` | `session.rs:285` → `sim.sedimenta` → `wp/solver/sedimenta.rs:78-92` | sim só com Tilt | 0..1 | A | sedimentação na orla de baixo | D9 |
| Tilt (toggle/ring/spoke) | `tilt_on/ring/spoke` | `tp/wetpaint_settings.rs:585-592,615` | `session.rs:248-254` → `sim.tilt_*` | sim | ring 0..8, spoke 0..11 | A | inclinação da tela | — |
| Grid Size (px) | `PAINTER_WETPAINT_GRID` | `tp/wetpaint_settings.rs:544` | `tp/wetpaint/grid_map.rs` | sim (resolução) | 1..30 | todos | resolução numérica | sem grandeza física |
| Flow Grid (×) | `PAINTER_WETPAINT_FLOW` | `tp/wetpaint_settings.rs:548` | `grid_map.rs` | sim | 1..16 | A | resolução do fluxo | sem grandeza física |
| Smooth Edges | `PAINTER_WETPAINT_SMOOTH_EDGES` | `tp/wetpaint_settings.rs:552` | `tp/wetpaint/smooth_edges.rs` | sim (render) | 0..1 | todos | antisserrilhado da silhueta | sem grandeza física |
| Wet canvas / Dry canvas / Fast dry | `PAINTER_WETPAINT_{WETCANVAS,DRYCANVAS,FASTDRY}` | `tp/wetpaint_settings.rs:620-631` | acções na grade | sim (acção) | botão | A/G | acção, não parâmetro | — |
| Show wet | `PAINTER_WETPAINT_SHOWWET` | `:632` | `tp/wetpaint/veu.rs:9,69` | só vista | bool | — | sobreposição | — |
| Tuning / Reset | `PAINTER_WETPAINT_TUNING/RESET` | `:657`, `:611` | painel | — | — | — | UI | — |

**Campos do preset fora de `engine`** (`ph2d-brush-presets/src/modelo.rs`): `volumes.{size_px,opacity,water}` (lidos, `tp/wetpaint_preset.rs:439-447`), `volumes.{pressure,length}` (**não lidos pelo Wet Paint** — nenhum escritor em `wet_preset_aplicar_volumes_corpo`), `paint_mode` (a ferramenta Paint/Smear/…, `tp/wetpaint_preset.rs:460`), `tilt`, `live_water`, `heavy_edge`, `shape_color`, `grain_color`, `width_table`, `ferramenta`, e o `brush` (42 campos de `BrushSpec`, `tp/wetpaint_preset/campos.rs:5`; os 80 que o Wet Paint NÃO lê estão em `campos.rs:91` `CAMPOS_FORA`, com o porquê). O mapa `engine` leva as 144 `key` + `erase` + `glaze`/`km`/`textura`/`realOil` (`tp/wetpaint_preset.rs:95-107,504-528`) + as 15 linhas da mão (`tp/mao.rs:39`).

## 3. Inventário dos MODOS / sentinelas / ramos por meio (lei de camada e de assentar)

| # | onde | o que faz |
|---|---|---|
| R1 | `wp/assenta.rs:22-34` `enum Lei {Porto, PorCima, Camada, Cobre}` | 4 leis do ecrã; o produto fixa `Cobre` (`tp/wetpaint/authored_actions.rs:64`); `Camada` só como controlo de portões; `Porto` é a lei do port JS (ainda usada por `remolha.rs:68`, `grid/traco.rs:77`, `oleo_borda.rs:21`). |
| R2 | `wp/assenta.rs:39-66` `Ecra::Cobre {filme, esconde, molhada, vela, oleo, opaca}` | 6 parâmetros que se SOMAM numa célula; cada `com_*` é um modificador encadeado (`render.rs:494-498`, `drying.rs:169-174`, `transfer.rs:255-258,663-666`, `tools.rs:246-248`, `smear.rs:274-276`, `canvas_ops.rs:59-61`). |
| R2a | `assenta.rs:69` `Ecra::seco()` | força `molhada = 1`: o render desenha o molhado com a lei seca (`render.rs:463`). |
| R2b | `assenta.rs:97` `com_vela` | `vela` = Glaze da célula (`Knob::Glaze` fixado no pouso): fracção do filme que FILTRA (multiplica) o assentado (`ponto.rs` `self.vela > 0`, `assenta/ponto.rs:121-124`). |
| R2c | `assenta.rs:122` `com_vidrado` | interpola `esconde` → `TINTA_ESCONDE_EM_VELATURA` pela fracção `vidrado` da célula (caixa Glaze layering). |
| R3 | `assenta.rs:154-178` `com_oleo` | **dois ramos pelo SINAL do `oleo` da sessão**: `o < 0` (= `PINCEL_DE_OLEO`, pincel com Real Oil) ⇒ vira `com_opaca(oleo_célula/OLEO_PLENO)`; `o ≥ 0` (aguada) ⇒ `oleo = min(oleo_célula/OLEO_PLENO, 1)` que corta o tapar extra. |
| R4 | `assenta.rs:208` `PINCEL_DE_OLEO = -1.0` | sentinela de «o pincel é óleo» dentro de um campo que noutros ramos é fracção `0..1`; posto em `wp/sim.rs:349-353`; `ponto.rs:229` lê `oleo.max(0.0)` para o neutralizar. |
| R5 | `assenta.rs:212` `OLEO_PLENO = 0.04` | rampa: 4 % de óleo na célula já conta como óleo pleno (calibrada na orla do Dabber). |
| R6 | `assenta/ponto.rs:229` `a_e = a_t + (h − a_t)·d·(1 − oleo)` | o **tapar extra** (`h = 1 − (1 − a_f)^esconde`, `assenta.rs:226`) ponderado pela diferença de pigmento `d` e cortado sobre óleo. |
| R7 | `assenta.rs:184-202` `com_opaca` + `ponto.rs:246-255` + `ponto.rs:98-106` `mistura_opaca` (`ponto.rs:141`) | a tinta OPACA (`Knob::Opaque` na célula, ou o óleo via R3): o canal vai para uma mistura K-M `ks` entre `sc` e `uc` pela concentração `concentracao_opaca(parte)`, fracção `opaca·d`. Uma 2.ª lei de cor DENTRO da 1.ª. |
| R8 | `ponto.rs:139` caixa das tintas `r.min(t_s.max(uc))` + `na_caixa_das_tintas` (render) | clamp por canal à mais clara das duas tintas — remendo do branco do tapar extra (Bug #50). |
| R9 | `wp/sim.rs:329-334` `ColorMix::Km` × `ColorMix::Plain`; `session.rs:258-266` `glaze = VELATURA_MEDIDA` se K-M OFF | a **mistura RGB** (Plain) é o ramo com K-M desligado, e liga uma velatura extra; `grid/traco.rs:75` usa `Plain` fixo. |
| R10 | `session.rs:279-283` `glaze_filme = VELATURA_DO_FILME_VIVA` × `VELATURA_DO_FILME` | a velatura do filme (`Ecra::Cobre.filme`) depende do Live Water (agendamento). |
| R11 | `wp/sim.rs:357` `vidrado`, `:358` `textura`, `:359` `oleo` (`Params`) | três caixas viram fracções `0/1` que a célula guarda (`grid/aparencia.rs:9-60`, `grid/oleo_celula.rs:14-57`) e se misturam pela massa. |
| R12 | `wp/grid/oleo_celula.rs:45-57` `do_pincel` | sem Real Oil pousa `NEUTRA` (Depth/Gloss ignorados); com ele, `oleo = p.oleo`, `OilDepth`, `OilGloss`; `Solubility` viaja na mesma struct. |
| R13 | `wp/trail/transfer.rs:174` `if p.oleo > 0 { espalha.primeiro_toque }` | o óleo molhado espalha o relevo de baixo no 1.º toque — só com Real Oil. |
| R14 | `wp/solver/ladeira_do_oleo.rs:14-25` `LADEIRA_ROTA` (AtomicU8, 0 ⇒ 7) e rotas 1–7 (`:79-102,161,178`) | **8 rotas** da «ladeira do óleo» (altura água+óleo no nivelamento): 1 sem peso, 2 desligada, 3 massa inteira, 4 peso assente de agora, 5 da fotografia, 6 = 3+5, 7 = 5 + presa a `|dfilm|` (produto). Estado global mutável de sonda dentro do solver. `FILME_POR_MASSA = 1/1000` (`:30`), `FILM_CHEIO = 0.983` (`:61`). |
| R15 | `wp/solver.rs:104`, `solver/flow_jacobi.rs:126,192`, `painter/doors.rs:85` | `oleo_na_mao` escolhe a ladeira do óleo de ANTES do traço (`Grid::oleo_antes`, `ladeira_do_oleo.rs:172`). |
| R16 | `wp/drying.rs:54,216`, `grid/canvas_ops.rs:104` `if ecra != Ecra::Porto` | ramos de secagem/levantar que só correm fora da lei do port. |
| R17 | `wp/trail.rs:22,143,480`, `trail/filme.rs`, `trail/transfer.rs:146-148`, `sim.rs:70` (`StrokeFilm > 0`) | o «filme do traço» (óleo/guache): a água da célula é a do filme, sem 2.º gole nem tecto — um modo booleano por knob. |
| R18 | `wp/lamina.rs:69,85`, `grid/nivelado.rs:57`, `tp/wetpaint/dab_route.rs:200-206` (`KnifeLevel`, `KnifeRidge`) | a faca: nivela e marca `nivelado`; `transfer.rs:139` `apaga_marca` desfaz pela massa. |
| R19 | `wp/trail/transfer/cobre.rs:1-30` (`WetCover`, `DryCover`) | guache: cobre o molhado / o assentado — terceira e quarta leis de cobertura, fora do `Ecra`. |
| R20 | `wp/render.rs:463-465` | o render troca `PorCima(glaze)` por omissão quando a lei não é `Cobre`/`Camada`. |
| R21 | `wp/render.rs:487` `w_oleo = c.oleo_em(cx,cy)` + `render_pigment_row_oleo*` (`render.rs:378,407,722`) | o píxel final mistura a óptica do óleo com a da tinta sem óleo pela fracção de óleo da célula — dois pipelines de cor. |
| R22 | `wp/colorops.rs:101,169-182` `TINTA_ESCONDE`, `TINTA_ESCONDE_EM_VELATURA`, `VELATURA_MEDIDA`, `VELATURA_DO_FILME(_VIVA)`, `molhada_escurece` | constantes calibradas contra o Rebelle que fazem de parâmetros escondidos da lei (`authored_actions.rs:59-65` fixa `pickup_assentado=0`, `lei=Cobre`, `esconde=TINTA_ESCONDE`). |
| R23 | `wp/tuning/defs.rs:306` `TextureSizeExpAcross` mín. `-0.01` = «desligado» | sentinela numérica num knob. **FEITA 04/11:** seletor `textureAcrossOwn` (0 Same as along · 1 Own exponent, `Knob::TextureAcrossOwn`, `KNOB_COUNT` 143) + `textureAcrossExp` 0..1,5; chave antiga por `tuning::chave_antiga_atraves`; gate `nenhum_knob_guarda_uma_sentinela_negativa`. |

**Leitura:** a cor de uma célula sai de `Lei` × `ColorMix` × (`filme`,`esconde`,`molhada`,`vela`,`oleo` com sinal,`opaca`) × caixa das tintas × (pipeline óleo | sem óleo); a aguarela, a guache, o óleo e o Óleo Real são combinações destes ramos, não pontos de uma escala.

## 4. Pares/famílias de controlos que mexem na MESMA grandeza

| D | grandeza | controlos | onde se juntam |
|---|---|---|---|
| D1 | massa de pigmento por dab | Pigment (`PigmentPerDab`) · Opacity (`sliders.pigmento`) · `Intensity` · `PressureOpacity` | `wp/sim.rs:88` `com_pigmento`, `doors.rs:24`, `tp/mao.rs:255` |
| D2 | arrasto de tinta ao longo do traço | `Drag` · `Smear` · `RewetDrag` · `WaterDrag` · `SmearWater` | `transfer.rs:115`, `tools/smear.rs:48-71`, `tools/arrasto_agua.rs` |
| D3 | recolha pela ponta | `Pickup` · `TipReservoir` · `TipClean` · `WetLift` | `trail/bico.rs:21-269`, `tools.rs:33` |
| D4 | água por dab | Water (`sliders.water`) · `WaterPerDab` · `SmearWater` | `doors.rs:25-29` (produto) |
| D5 | secagem | Dry speed (`Evaporation`) · `BaseEvaporation` · `Retention` · `Dryer` · `DryerLevel` · `DryerEdge` · Fast dry | `drying.rs:121-125`, `sim.rs:434`, `tools.rs:267` |
| D6 | absorção do papel | `PaperGate` · `GateSaturation` · `Capillary` · `CapillaryGate` | `trail/deposit.rs:124-125`, `solver.rs:96-97` |
| D7 | passo | `Spacing` · `CapsuleSpacing` · `PigmentSpacing` · `CapsuleMaxPx` · `brush.spacing` | `painter.rs:394`, `trail.rs:483`, `dab_route.rs:213`, `lote.rs:87-102` |
| D8 | re-dissolver o assentado | `Rewet` (Water) · `Dissolve` (cartão «Re-wet») · `Solubility` | `sim.rs:435`, `transfer/remolha.rs`, `oleo_celula.rs:46` — dois knobs com o MESMO rótulo de produto |
| D9 | orla escura / sedimento | `EdgeDarkening` · `EdgeSoftMass` · Heavy Edge | `drying.rs:122,300`, `solver/sedimenta.rs` |
| D10 | nivelamento | `Leveling` · `LevelClamp` · `Viscosity` · `Brake` · `BrakeReach` · `LevelingOwnStroke` · `KnifeLevel` · `DryerLevel` | `solver.rs:93-100`, `lamina.rs`, `secador.rs` |
| D11 | força da borracha | Erase (cartão) · `Eraser` (Tuning) | `tools.rs:98` `force = Eraser × erase_slider` |
| D12 | papel visto na tinta | `PaperVisibility` · `VisualGrain` · Paint Texture · `PaperRelief` · `PaperPress` · `ExtGranulation` | `grid/aparencia.rs:51-54`, `grid/material.rs:35-37` |
| D13 | altura do relevo | `Emboss` · `OilDepth` · `KnifeRidge` · `BristleHeight` · `BristleRemove` | `transfer.rs:120`, `oleo_celula.rs:65`, `dab_route.rs:195-200` |
| D14 | aderência do assentado | `ExtStaining` · `Solubility` (inversos) | `tools.rs:104`, `oleo_celula.rs:46` |
| D15 | pincel seco | `ExtDryBrush` · `dry_gate` derivado da água | `deposit.rs:126`, `doors.rs:40` |
| D16 | filtro (velatura) | `Glaze` (`vela`) · Glaze layering (`vidrado`) · `glaze` da sessão (K-M OFF) · `glaze_filme` (Live Water) | R2b, R2c, R9, R10 |
| D17 | estabilização | `PulledString` · `BrushSpec.stabilizer` | `tp/mao.rs:265` |
| D18 | escala com o diâmetro | `EmbossSizeExp/Ref` · `TextureSizeExp/Ref/ExpAcross` · `BristleLaneSizeExp` | `tp/wetpaint/lote.rs:132-218` |
| D19 | abertura do molhado alheio | `WetOpen` · `WetOpenPush` · `WetOpenEdge` | `trail/molhado.rs` |
| D20 | material Principled | 8 knobs Material (célula) · 8 linhas do cartão Real Oil (global) | `wp/material.rs:138-165` |
| D21 | **cobertura de uma tinta sobre a de baixo** | `esconde` (sessão, `TINTA_ESCONDE`) · `Opaque` · `WetCover` · `DryCover` · `Transparency` · `Transmission/Mode` · `PINCEL_DE_OLEO`+`OLEO_PLENO` · `Pigment`/Opacity (via `a_f`) | R2, R3, R6, R7, R19 — **o núcleo do defeito**: ≥ 7 caminhos para UMA grandeza |
| D22 | brilho especular | `OilGloss` · `Coat` · `CoatRoughness` · `Roughness` | `oleo_celula.rs:56`, `grid/material.rs:26-29` |

## 5. Controlos sem grandeza física clara

- **Numéricos/de solver:** `MaxVelocity`, `Projection`, `CapsuleRoundNearest`, `CapsuleMaxPx`, Grid Size, Flow Grid, Smooth Edges, `ThinStrokeFade/Ref`, `PressureSmoothing`.
- **Modos booleanos disfarçados de knob:** `StrokeFilm`, `LevelingOwnStroke`, `TransmissionMode`, `LockMaterial`, `TextureSizeExpAcross = −0,01`.
- **Ganhos sem unidade que multiplicam outro:** `Intensity`, `WaterPerDab` (× Water), `Eraser` (× Erase), `ExtRivulets` (só com `ExtFingering`), `StrokeFilmDome/Ends` (só com `StrokeFilm`), `OilDepth/OilGloss` (só com Real Oil).
- **Ruído de cor (estética, não física):** `BristleNoiseHue/Sat/Light`, `BristleStreak*`.
- **Constantes calibradas que agem como parâmetros escondidos:** `TINTA_ESCONDE`, `OLEO_PLENO`, `FILME_POR_MASSA`, `FILM_CHEIO`, `VELATURA_*`, `PINCEL_DE_OLEO` (R22, R4, R5, R14).
- **Mortos no produto:** `ExtRakeLight`, `ExtValleyGran`, `ExtWetSheen`, `ExtGrainDither`, `ExtEdgeTint` — lidos só em `wp/render.rs:70-75` (`render_region`), e `render_region` só é chamado por `crates/ph2d-wet-paint/tests/it/{measure,perf}_experimental.rs`; o produto compõe por `render_pigment_row_*` (`tp/wetpaint/composite.rs:263,279`). `volumes.pressure`/`volumes.length` do preset: sem leitor no Wet Paint.

## 6. Limites deste censo

- «decide?» foi conferido por existência de leitor de produção que entra na conta do píxel; não foi MEDIDO por varredura (isso é a F0). Os knobs Paper (`PaperContrast/Fibres/Grooves`) só decidem quando o papel é o do próprio motor (`tp/wetpaint/session.rs:211-217`).
- Não varri quais knobs os presets de fábrica põem fora do default (os presets são PNG com JSON embutido, `ph2d-brush-presets/src/png.rs`); é o próximo passo útil para saber que cantos do espaço os pincéis aprovados usam.
- Tabela §1 gerada por script a partir de `defs.rs`, das chaves i18n `panel.wet_tuning.knob.*` e de `grep Knob::X`; leitores via struct intermédia (ex.: `Fios`, `BristleKnobs`) aparecem como o sítio da struct.
