# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, P8a (núcleo + UI dos menus) — 2026-11-02

> Leitor: o agente que assume a linha na janela seguinte. Plano: [P8_plano.md](../P8_plano.md) (§9: o dono delegou as
> 4 perguntas — valem as recomendações). Anterior: [P7 integração](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-02_P7.md)
> (P7 aprovada no smoke, AINDA NÃO integrada). Merge-base `3df5894489`; tudo comitado em `line/PixelArt`, árvore limpa.

## 1. O que existe (não reconstruir)

**Núcleo `ph2d-pixel-core`** — leis do Aseprite Trial 1.3.18 (fixturas `tests/fixtures/oracle_trial_*.json`):
- `filter/mod.rs`: `Source`, `Filter`, `Channels`, `Edge {Clamp, Wrap=Tiled}`, `apply()` = um passo, lê o original, só dentro da selecção (gate `filter_target`).
- `filter/adjust.rs`: `Invert`; `BrightnessContrast` (contraste em ½ ANTES do brilho multiplicativo, `⌊y·255,5⌋` — exacto nos 9 casos da rodada 1); `Curve` (rectas, `⌊y⌋` — exacto); `HueSaturation` 4 modos (lei do livro, `l=(max+min)/510`; só os empates `k+½` diferem ±1: o Trial desempata por ruído de vírgula flutuante — o gate aceita SÓ isso).
- `filter/median.rs` (Despeckle, 0 desvios), `filter/outline.rs` (dentro/fora, 4 presets, Mask/cor, Tiled; a casa (dx,dy) da matriz testa o vizinho (−dx,−dy)), `filter/smudge.rs` (Blur `⌊média 3×3⌋` carimbo a carimbo; Jumble = vizinho 3×3 sorteado, semente do gesto).
- `gradient.rs`: Linear (projecção), Radial (elipse inscrita na caixa do arrasto), Solid `⌊a+(b−a)t⌋` por canal, Bayer com `t·(n²+1) ≥ M+1` (em `t`, não em `v`); região = `bucket_cells` a partir de `from`, dentro da selecção; `Gradient::values()` → `Stroke::with_values`. 32 arrastos, 0 células.
- `selection::Mask::edge4()` (Edit ▸ Stroke). Gates: `filter_oracle`, `gradient_oracle`, `fx_oracle`.

**Crate NOVA `ph2d-pixel-text`**: leitores BDF/HEX/FNT/FON (porte do MVP W27), fontes de fábrica Cozette/Tom Thumb/Unscii com licenças, rasterizador Mono/AA sobre `skrifa` 0.44 COM hinting (o Aseprite usa FreeType hinted, AA desligado por omissão). Trial mono: 7/9 casos célula a célula (`trial_text`), FreeType hinted 5 células (`freetype_oracle`).

**UI (comitada, seam tests verdes)**: Edit ▸ Fill (F), Stroke (S), Invert…, secção Adjustments (Brightness/Contrast…, Hue/Saturation… Ctrl+U, Color Curve… Ctrl+M), secção FX (Outline… Shift+O, Despeckle…). Diálogos = `PixelEditDialog` da P6 com canais, Preview pelo caminho da moldura solta (`Stroke::with_values`, sem gravar), OK = um passo, Cancel/Esc intactos. Widget NOVO `ph2d-editor-core/src/widget/curve_editor.rs` (+ gate). Ficheiros: `ph2d-app-pixel/src/{filtros,ajustes}.rs`, `ph2d-editor-core/src/pixel_panel/filters.rs`, `interaction/state/pixel_filter_ops.rs`, `ids/chrome/pixel_filter.rs`; testes `shells/pixel-scenes/tests/it/{filtros,ajustes}.rs`.

## 2. Aberto, na ordem de ataque (um bloco por família, §0.10)

1. **Convolution Matrix** (núcleo + diálogo F9): lei medida na rodada 2 — `clamp(trunc_a_zero(Σw·v/div) + bias)`, borda clamp, Tiled = wrap; matrizes recuperadas em `oracle_trial_convolution.json` (`round2_*`: blur, sharpen 3/5/7, misc-contour, rock-edges, brightness, contrast, negative, drunk…). Recusas: `misc-texturize/-emboss/-rock` (~1 000–3 300 matrizes explicam igual), `edges-find*` (128 constante). Kernels = DADOS nossos com o nome medido. Decidir com o dono se os kernels não recuperáveis ficam fora da lista.
2. **Brightness/Contrast com os 43 casos da rodada 2**: a fórmula `⌊y·255,5⌋` foi achada com 9; a rodada 2 diz «contraste primeiro deixa 1 613 ±1 em 11 008». Correr `docs/PixelArt/ferramentas/fit_ajustes.py` sobre os 43 e alargar o gate `brightness_contrast_gives_the_trials_tables` a todos; se a fórmula cair, re-ajustar (é offline).
3. **Ferramentas** Gradient (sub-ferramenta do balde, Shift+G; barra Linear/Radial + No Dithering/Bayer 2/4/8, estado PRÓPRIO, não o da dinâmica), Blur/Jumble (grupo novo; a tecla R alterna no Aseprite), e a opção «Palette ramp» do Gradient (decisão do dono §9.1, lei do MVP §13.3). Mapa de onde mexer: `PixelTool` em `ph2d-editor-core/src/pixel_panel/tools.rs`, grupos `PixelToolGroup`, `input_teclas.rs`, gesto em `input.rs`/`gesture.rs`.
4. **Text** (ferramenta T + Edit ▸ Insert Text): caixa no canvas editável até Enter (decisão §9.3), fonte (sistema via `ph2d-system-fonts::library` + as de fábrica), tamanho, B/I, antialias, fill/stroke. Abertos do núcleo: corte da caixa a 16 px (4 células/fonte), stubs do dropout (2 células em Sans 8), tom do AA (curva do Trial ≈ `1−(1−a)^0,62`, sem lei exacta).
5. **Por medir** (uma rodada só, se fizer falta): Invert em Indexed (o OK não muda nada no Trial — é o filtro que ignora Indexed?), Gradient em Indexed (o arrasto não pintou), Outline com Background = cor, Hue/Sat «Alpha» (fora da UI por não medido), alvo *all frames/layers*.
6. **Cena 19** (`PH2D_PIXEL_SMOKE=19`, `max_level` 18 → 19 em `crates/ph2d-app-pixel/src/smoke.rs:32`), sonda headless, fotos; mutações (P8_plano §6); fecho (DIRETRIZ §1.5.9) e handoff de integração.

## 3. Avisos

- `the_pixel_edit_dialogs_are_alive.rs` está em 796 linhas — o próximo diálogo abre ficheiro irmão.
- `PIXEL_CURVE_MAX = 32` pontos (o pedido do diálogo é copiado por valor; a caixa pintada não comporta mais — o teste confere).
- A caixa da curva é rectangular; conferir com a foto `aseprite/83_color_curve.png`. No Aseprite, o clique direito num ponto abre «Point Properties» (X/Y exactos, foto 87) — nós temos duplo-clique para tirar.
- Hook `prazo-do-agente` do main ainda tem o defeito do caminho relativo (curado só nesta linha, `288d03792f`): chame agentes com o cwd na RAIZ da worktree.
