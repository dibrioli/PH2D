═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Painter (NOVO) · W4 bespoke-UI kinds (Curves/Levels)
Autor: Implementador Painter (sessão 2026-06-03/04) · você roda em CONTEXTO SEPARADO
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ MANDATO NOVO (Enio): TODA implementação agora busca MÁXIMA          ║
║ PERFORMANCE EM TEMPO REAL. Curves/Levels NÃO são "mais um kind      ║
║ de CPU" — têm que ser GPU real-time (sub-ms), como filtro WebGL.    ║
║ A estratégia certa (LUT na GPU) está no §3. Leia ANTES de codar.    ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
§0 — INEGOCIÁVEIS DE PARTIDA (antes de qualquer edit)
───────────────────────────────────────────────────────────────────
- **REBASE/branch a partir de `98da9d3`** (pushado — aviso crítico do Coord).
  Houve refatoração god-module→submodules MASSIVA: vários arquivos MUDARAM DE
  LUGAR (§2). NÃO confie em paths antigos.
- **Isolamento (CLAUDE.md §0.2):** edite SÓ a sua pasta; foundational/GPU-shader/
  contrato = PARE e reporte ao Coord. Sua pasta aqui: `adjustments/` (compute +
  contrato), `ph2d-panel-painter-layers/` (UI bespoke), `tool/` (layers/params,
  NÃO o compositor arm/cache). GPU shader/binding (`ph2d-render`) = Coord.
- **Inner loop = `cargo check -p <crate>`** no slot CoW (`scripts/slot-seed.sh`);
  gates/clippy 1× no fim. `git commit --no-verify -- <seus paths>`; NUNCA `-A`.
- **GPU tests rodam no teu Mac (Metal):** `cargo test -p ph2d-render --test
  layer_compositor_gpu -- --ignored` — USE pra validar paridade GPU↔CPU de verdade
  (não só naga). Foi assim que a engine de adjustments foi provada.

───────────────────────────────────────────────────────────────────
§1 — ESTADO (o que já existe — NÃO refaça)
───────────────────────────────────────────────────────────────────
W4: menu "+ Adjustment" (24 kinds) + **7 kinds prontos** (HSB, Brightness/Contrast,
Invert, Posterize, Threshold, Exposure, Vibrance) — compute CPU + GPU + paridade.

**A engine GPU de adjustments está VIVA e PROVADA** (a grande peça desta sessão):
  - `ph2d-render` compositor GPU aplica adjustments num único compute pass
    (`OP_ADJUSTMENT` + `apply_adjustment` WGSL). Paridade GPU↔CPU (±4 bytes,
    opacity-parcial exata) + perf **base+HSB 1024² = 1.7ms vs 55ms CPU (~32×)**.
  - **Preview do painter JÁ ROTEIA pela GPU** (Phase 3 completo, commit `6691554`):
    `flatten_for_gpu` → `LayerCompositor::composite` → premul → `PreviewOverride`.
    Fallback CPU (com cut-cache) quando o stack não é GPU-representável (mask/clip/
    reference/adjustment-mascarado/kind-sem-gpu_code).
  → **Qualquer kind novo com `gpu_code()` vira real-time no preview automaticamente.**

Sua tarefa: **Curves + Levels** (os primeiros bespoke-UI), GPU-real-time. Depois,
os demais bespoke (GradientMap/ColorLookup/SelectiveColor/ChannelMixer/
ShadowsHighlights/BlackAndWhite/ColorBalance/PhotoFilter) + os espaciais
(Gaussian/Motion/Sharpen/Bloom/ChromaticAberration/Noise — esses são multi-pass GPU,
§3.C).

───────────────────────────────────────────────────────────────────
§2 — MAPA PÓS-REFATORAÇÃO (onde tudo está AGORA)
───────────────────────────────────────────────────────────────────
`crates/ph2d-painter-brush/src/adjustments/`  (era `adjustments.rs`)
  - `mod.rs` — `AdjustmentKind` (+`ALL`/`display_name`/`gpu_code`), `AdjustmentParams`
    (+`kind`/`neutral_for`/`gpu_params`), `AdjustmentLayer`, todos os `*Params`
    (incl. `CurvesParams{points_rgb/r/g/b: ControlPoints}`, `LevelsParams{black_point,
    gamma,white_point,output_black,output_white}`, `ControlPoints{points:Vec<[f32;2]>}`).
  - `compute.rs` — `apply_adjustment` (dispatch ~23), per-kind (`apply_hsb`~332 etc.),
    `adjustment_slider_params`~248, `set_adjustment_slider_param`~280, sRGB f32 +
    `build_lut`/`sample_lut` (utilitário LUT já existe!).
  - `tests.rs` — gates + golden.
`crates/ph2d-render/`
  - `src/layer_compositor/{mod.rs,compositor.rs,tests.rs}` (era `layer_compositor.rs`):
    `LayerOp::Adjustment{kind,params:[f32;3],blend,opacity}`~148, `AdjParamsGpu`~240,
    `OP_ADJUSTMENT`~233, bind/dispatch.
  - `src/shaders/layer_composite.wgsl` — `ADJ_*`~65, `AdjParams`~85, `apply_adjustment`
    WGSL~430, OKLab~399, `cs_flat`/`cs_grouped`.
  - `src/preview_premul.rs` + `shaders/preview_premul.wgsl` — o passe straight→premul
    (Coord, Phase 3.2). `tests/layer_compositor_gpu.rs` — paridade+perf GPU.
`crates/ph2d-tool-painter/src/`
  - `tool/{mod.rs,trait_impls.rs,...}` (era `tool.rs`): `add_adjustment_layer`~503,
    `set_adjustment_param`~528, `preview_layer_pixels` (provider GPU),
    `handle_panel_event` em `trait_impls.rs`~52.
  - `layers/` (era `layers.rs`): `add_adjustment`, `adjustment_mut`, `LayerKind::Adjustment`.
  - `compositor/{compose.rs,cache.rs}` (era `compositor.rs`) — CPU ref + cut-cache.
`crates/ph2d-panel-painter-layers/src/`  (NÃO refatorado — paths estáveis)
  - `paint_adjust.rs` — `paint_adjustment_params`~39 (renderiza sliders por slot).
    **É AQUI que mora o controle bespoke** (curve editor / levels).
  - `adjust_menu.rs` — popover dos 24 kinds. `event.rs` — `decode`/forward.
`crates/ph2d-editor-core/src/ids/`  (era `ids.rs` — split por domínio)
  - `PainterLayerWidget` (`AdjParam0..5`), `painter_layer_widget_id`. p/ um widget
    bespoke novo, adicione variant(s) aqui (aditivo).

───────────────────────────────────────────────────────────────────
§3 — A ESTRATÉGIA REAL-TIME (o coração — Curves/Levels = LUT na GPU)
───────────────────────────────────────────────────────────────────
**O problema:** `gpu_params()` é `[f32;3]`. Curves (8 pts × 4 canais) e Levels
(5 f32) NÃO cabem. O caminho ingênuo (gpu_code → [f32;3]) é impossível → cairia no
fallback CPU (~dezenas de ms) = viola o mandato real-time.

**A solução (jeito WebGL, real-time):** Curves E Levels são **transferências 1-D
por canal** (`out_ch = f(in_ch)` em display-space). Então **bake pra um LUT** e
amostre na GPU:
  1. **CPU (sua pasta):** de `CurvesParams`/`LevelsParams`, compute uma LUT por
     canal — ex. `[[u8;256];3]` ou `[f32;256*3]` (R/G/B; Curves = master∘per-canal).
     Só recomputa quando o param muda (drag do editor) — 768 evals, trivial.
     Implemente o `apply_curves`/`apply_levels` CPU (a referência canônica) + a fn
     que gera a LUT a partir deles.
  2. **GPU (foundational = COORD, coordene):** o compositor precisa de um binding de
     **LUT por-adjustment** (storage buffer `adj_luts` ou texture, indexado pelo op),
     análogo ao `srgb_lut` que já existe. O `apply_adjustment` WGSL, no case
     ADJ_CURVES/ADJ_LEVELS, amostra `lut[base + ch*256 + round(srgb(x)*255)]`. O
     `AdjParamsGpu`/`LayerOp::Adjustment` ganha um `lut_index` (ou reusa o params
     slot). É extensão da MINHA engine GPU (não quebra contrato congelado — o
     `gpu_params`/`AdjParams` são adições minhas, não ADR). **Peça ao Coord esse
     binding** (ele mantém o shader/compositor) — você entrega a matemática + a
     LUT-build + a UI.
  3. **Paridade:** gate GPU↔CPU (espelhe `gpu_adjustment_matches_cpu_reference_each_kind`
     em `tests/layer_compositor_gpu.rs`) — a LUT GPU tem que bater com `apply_curves`/
     `apply_levels` CPU dentro de ±tolerância. RODE no teu Mac.

**Faseamento pragmático (decida com o Coord):** (v1) Curves/Levels CPU-compute +
bespoke UI + `gpu_code()=None` → funciona, correto, fallback CPU. (v2 = o mandato) o
binding de LUT GPU → real-time. Recomendo fazer v1+v2 juntos pra HSB-style smoke,
mas v1 sozinho já destrava a UI. **NÃO** tente meter Curves em `[f32;3]`.

**.C — espaciais (Gaussian/Bloom/etc.), p/ depois:** multi-pass GPU (separável
ping-pong) — terreno natural da GPU, mas é outro mecanismo (não LUT 1-D). Fora do
escopo Curves/Levels; alinhe com o Coord quando chegar.

───────────────────────────────────────────────────────────────────
§4 — BESPOKE UI (o editor de curva / controles de levels)
───────────────────────────────────────────────────────────────────
O slider genérico (`AdjParam0..5`) NÃO serve pra Curves (precisa de um canvas de
curva arrastável) nem pra Levels (sliders triplos + histograma). Padrão (5 passos):
  1. `adjustment_slider_params()` retorna **vazio** pra Curves/Levels (sem sliders
     genéricos) — `compute.rs`.
  2. Pinte o controle bespoke num arm novo de `paint_adjust.rs` (após ~83): pro
     Curves, um quadrado com a curva + pontos de controle arrastáveis; pro Levels,
     3 handles (black/gamma/white) sobre um histograma.
  3. Registre os widget-ids no store em `paint_adjust.rs` (espelho do
     `register_slider`~89) — ids novos via `PainterLayerWidget` em `editor-core/ids/`.
  4. `event.rs`: decode o id bespoke → forward um `PanelEvent` (drag de ponto =
     `SetValue`/`SetVec`; pode precisar de uma forma nova no canal genérico — se o
     `PanelEvent` congelado não couber, PARE e reporte ao Coord, não renegocie).
  5. `tool/trait_impls.rs::handle_panel_event`: decode → método novo no tool
     (`set_curve_point`/`set_levels`) que muta `adjustment_mut(id).params` +
     `invalidate_composite`/arma o cache pending (como `set_adjustment_param`~528).
  **UI em INGLÊS** ([[feedback-app-ui-english-only]]); zero hex/f32-literal/string
  hardcoded (tokens/i18n, HR-15); botão/widget novo exige register
  ([[feedback-panel-populate-register]]).

───────────────────────────────────────────────────────────────────
§5 — GATES (batched no fim; `cargo check` ESCONDE)
───────────────────────────────────────────────────────────────────
  cargo test -p ph2d-painter-brush --lib adjustments
  cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
  cargo test -p ph2d-render --lib layer_compositor   # naga + coefficient + discriminant
  cargo test -p ph2d-render --test layer_compositor_gpu -- --ignored   # paridade+perf GPU (Metal)
  cargo test -p ph2d-tool-painter --lib
  cargo clippy -p ph2d-painter-brush -p ph2d-panel-painter-layers --all-targets --no-deps -- -D warnings
  # contrato: AdjustmentKind/Params já têm Curves/Levels (cap ≤32) — não estoure.
  # WGSL: literais OKLab/sRGB DEVEM ser os f32-arredondados do ph2d_color (NÃO
  #   full-precision) — gate shader_adjustment_coefficients_bit_identical_with_rust.
fmt: `rustfmt <seus arquivos>` (NÃO `cargo fmt -p` — reformata WIP alheio,
[[feedback-cargo-fmt-p-reformats-foreign-wip]]).

───────────────────────────────────────────────────────────────────
§6 — ARMADILHAS / CONTEXTO
───────────────────────────────────────────────────────────────────
  - **Multi-agente quente:** Vector + Coord commitam por cima. `git status` antes
    de stage; commit scoped cedo cria fence ([[feedback-destructive-reset-collision-2026-05-28]]).
    Crate alheio com `Cargo.toml` sem `lib.rs` quebra o workspace inteiro — se
    `cargo` falhar em crate que você não tocou, é isso (transitório, espere/reporte).
  - **GPU OKLab = `pow(x,1/3)`** (CPU = libm `cbrt`): paridade é ±tolerância (≤4
    bytes), não bit-exata. Curves/Levels são display-space (sRGB transfer) — idem.
  - **Adjustment é display-space?** Curves/Levels SIM (convertem linear↔sRGB
    internamente, como Invert/Posterize). A LUT é indexada em sRGB.
  - **A engine GPU/preview está PROVADA — não a refatore.** Você ESTENDE (novo kind
    + LUT binding via Coord). Adicione kinds; não mexa no que funciona.
  - **Não pusha.** Reporta commit local; Coord faz ship (§3 CLAUDE.md).

───────────────────────────────────────────────────────────────────
§7 — REFERÊNCIAS
───────────────────────────────────────────────────────────────────
  - ADR-0045 (+amendment-1) adjustments; `docs/Painter_projeto/15_plano §7` (W4).
  - Engine GPU + Phase 3: `docs/HANDOFF_painter_gpu_preview_coord.md` (a engine +
    o hookup que já landou) — leia pra entender o caminho GPU.
  - Perf/arquitetura: memória [[project-painter-composite-perf-2026-06-03]]
    (CPU-reference era a raiz; GPU é o real-time; literais f32-arredondados).
  - Commits-chave: `e0a81c9`/`afe210f`/`18a85a1` (engine+contrato), `6691554`
    (preview GPU), `6044cc1` (flatten/gate).
═══════════════════════════════════════════════════════════════════
