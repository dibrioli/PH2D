═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Painter (NOVO) · W4 Adjustment Layers — fan-out
Autor: Implementador Painter (sessão 2026-06-02/03) · você roda em CONTEXTO SEPARADO
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ W3 FECHADO. W4 EM ANDAMENTO: o contrato + compositor (Coord) e DOIS ║
║ kinds (HSB + Brightness/Contrast) + a FUNDAÇÃO GENÉRICA de slider    ║
║ estão prontos e verdes. Tua missão: T4.15 (menu de kinds) → destravar║
║ a criação de todos os kinds → fan-out dos 22 restantes.             ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
►► SESSION UPDATE 2026-06-03 (impl seguinte) — LOCAL, não pushado
───────────────────────────────────────────────────────────────────
**T4.15 FEITO + 5 kinds per-pixel.** Tudo verde nos gates §5 (check 4-crates,
test brush/tool/contracts/editor-core, clippy 3-crates, host-desktop). 1 commit
local pronto p/ ship do Coord. **PEDIDO DE SMOKE ao Enio** (ver fim do report).

  - **T4.15 — menu "+ Adjustment"** (opção (a) do §2): o botão `+ Adj` virou um
    `Dropdown` (registra `InteractiveState::Dropdown` em `populate.rs`; o dispatch
    genérico faz toggle open/close — espelho exato do blend chip). Abre um popover
    full-width (cai reto pela coluna do panel — nomes longos cabem em 1 linha) com
    os 24 kinds de `AdjustmentKind::ALL`. Pintura deferred em `paint.rs`
    (`adjust_menu.rs`, novo) + stash `PENDING_ADJ_MENU` (state.rs). Click num kind
    → `event.rs` fecha o dropdown + forward `SelectOption(ADD_ADJUSTMENT, idx)` →
    tool mapeia `idx`→`AdjustmentKind::ALL[idx]`→`add_adjustment_layer(kind)`.
    Ícone fica Accent enquanto aberto. Helper id novo `painter_adjustment_kind_
    option_id(index)` (editor-core ids.rs, aditivo).
  - **`AdjustmentKind::ALL` (24) + `display_name()`** (inglês) em adjustments.rs;
    o `ALL_KINDS` do teste agora referencia `ALL` (DRY, fonte única).
  - **5 kinds per-pixel landados** (compute OKLab/linear/display + descritores de
    slider + golden invariantes): **Invert** (negativo display-space, 0 sliders),
    **Exposure** (EV/offset/gamma, 3), **Vibrance** (chroma OKLab low-sat-weighted,
    2; gray-safe), **Posterize** (quantize display, 1), **Threshold** (luma display
    → P&B, 1). Todos: alpha preservado + neutral early-return (onde aplicável).
  - **PERF (lição — aplique nos próximos display-space kinds):** o smoke do Enio
    pegou queda de FPS no drag dos kinds display-space (Invert/Posterize/Threshold/
    Exposure faziam até **6 `powf`/pixel** — round-trip sRGB por canal — vs o
    `cbrt` único dos kinds OKLab). Como o recompose é canvas-inteiro por frame (o
    `CompositorCache` é lever do Coord, §3), `powf`/pixel domina. **Fix:** todo op
    per-canal display-space é função 1-D do input → `build_lut`/`sample_lut`
    (N=1024) UMA vez por call, inner loop = clamp+index+lerp, **0 transcendentais/
    pixel**. Posterize mantém `round()` exato no índice de banda. **Curves/Levels/
    GradientMap (display-space) DEVEM usar o mesmo LUT** — não chame `powf` por
    pixel. commit `9e12b31`.
    **►► RE-SMOKE (Enio): FPS AINDA cai — agora em TODOS os kinds, incl. HSB/B-C.**
    Isso CONFIRMA que o compute nunca foi o gargalo dominante: é o **recompose de
    canvas INTEIRO por frame de drag** (bandwidth-bound — lê todas as layers +
    reupload). **= o `CompositorCache` (ADR-0045 §2.7/§2.11), que segue SKELETON**
    (`compositor.rs` `CompositorCache::invalidate_from` = "skeleton: clears"; gate
    `adjustment_layer_recomposition_perf_4k` `#[ignore]` "W5 wires CompositorCache").
    **=> ESCALADO AO COORD (W5).** Fora da minha pasta (inegociável #2 + §3/§5). O
    compute (LUT + OKLab) está no orçamento; o lever estrutural é do Coord.
  - **AINDA NÃO** (próximo impl): **espaciais** Gaussian/Motion/Sharpen/Bloom/
    ChromaticAberration + **Noise** (precisam vizinhança/seed-por-posição — §3 é
    Coord-boundary, PARE e alinhe). **Bespoke-UI** Curves/GradientMap/ColorLookup/
    SelectiveColor/Levels/ChannelMixer/ShadowsHighlights/BlackAndWhite/ColorBalance/
    PhotoFilter (controles próprios em paint_adjust.rs). **= 13 kinds restantes.**

───────────────────────────────────────────────────────────────────

───────────────────────────────────────────────────────────────────
§0 — ESTADO ATUAL (confirme no git; NÃO refaça)
───────────────────────────────────────────────────────────────────
W0-W3 ratificados. **W4:**
  - **Coord landou a fundação:** T4.1 contrato `adjustments` (commit `051455b`,
    ADR-0045 + `0045-amendment-1.md`) + T4.2 `LayerKind::Adjustment` + compositor
    arm + `CompositorCache` skeleton (`d97f906`).
  - **Eu (impl anterior) landei, LOCAL (não pushado):**
    - `c49f768` HSB compute · `8b3f9de` HSB UI (create + sliders) ·
      `47601ef` HSB em OKLab (fix de ruído do smoke do Enio) ·
      `13b710b` Brightness/Contrast + **plumbing genérico de slider**.
  - Há `be1d0f5` (W3 close) também local. **5 commits painter locais** entram no
    ship do Coord. Confirme: `git log --oneline | grep painter`.
  - **MULTI-AGENTE:** o agente Vector commita por cima da mesma history (HEAD pode
    ser um commit `fix(vector): …`). Meus commits painter seguem ancestrais de HEAD
    (verificado) — `git log -8` pode NÃO mostrá-los (janela curta); use
    `git merge-base --is-ancestor 13b710b HEAD`.

**Smoke Enio (Day-4):** HSB em OKLab confirmado limpo. FPS no drag ainda cai →
ver §3 (lever é do Coord, não teu).

───────────────────────────────────────────────────────────────────
§1 — A FUNDAÇÃO GENÉRICA (entenda isto ANTES de codar — é tua alavanca)
───────────────────────────────────────────────────────────────────
Adicionar um kind **baseado em slider** agora custa **3 coisas, todas em UM
arquivo** (`crates/ph2d-painter-brush/src/adjustments.rs`). ZERO mudança de
panel/tool/editor-core:

  1. **A compute fn** `fn apply_<kind>(p: &<Kind>Params, acc: &mut [[f32;4]])`.
  2. **Um arm** em `pub fn apply_adjustment(kind, params, acc)` (o dispatch).
  3. **Os descritores de slider**, dois arms espelhados:
     - `adjustment_slider_params(params) -> Vec<(&'static str /*label*/, f32 /*0..1*/)>`
     - `set_adjustment_slider_param(params, slot, value01)` (o inverso)
     (até 6 slots → widget ids `AdjParam0..5` já existem e são genéricos).

O resto já é genérico e NÃO precisa tocar: `paint_adjustment_params`
(`paint_adjust.rs`) renderiza um slider por slot; `PainterTool::
set_adjustment_param(id, slot, v)` roteia; `handle_panel_event` mapeia
`AdjParam{N}→slot N`. **Espelhe HSB/B-C** que já estão lá como exemplo.

**Contrato do hook de compute (NÃO viole):**
  - `acc` é **straight LINEAR f32 RGBA** (R,G,B,A em 0..=1, **A = cobertura —
    PRESERVE, só transforme RGB**). Mask/opacity/blend o compositor faz EM VOLTA
    da call (copy→apply→blend por mask×opacity); teu compute é puro kind+params→pixel.
  - **Neutral → early-return identidade EXATA** (perf no drag + exatidão).
  - **Espaço de cor = OKLab** (`ph2d_color::oklab::OklabColor::{from_linear,
    to_linear}`). **NÃO use HSL pra hue** — hue HSL é instável em quase-neutro →
    arco-íris no cinza (o bug que o Enio pegou; ver `47601ef`). Hue = rotação
    rígida do vetor (a,b). Brightness "exato nos extremos" = lerp linear p/
    preto/branco.

───────────────────────────────────────────────────────────────────
§2 — TUA ORDEM DE TRABALHO
───────────────────────────────────────────────────────────────────
1. **T4.15 — menu "+ Adjustment" (UNBLOCKER #1).** Hoje o botão "+ Adj"
   (`PAINTER_LAYERS_ADD_ADJUSTMENT`, em `paint.rs::paint_action_toolbar`) cria
   SÓ HSB (`handle_panel_event` → `add_adjustment_layer(HueSaturationBrightness)`).
   Precisa de um submenu dos 24 kinds → cria o escolhido. **Espelhe o blend
   dropdown** (`blend.rs::paint_blend_popover` + `PENDING_BLEND_DD` em `state.rs`,
   deferred-paint no fim do `paint`, on-top). Opções de design (tua escolha):
   (a) "+ Adj" abre popover de 24 kinds → `add_adjustment_layer(kind)`; ou
   (b) um chip de kind na row do adjustment (espelho do blend chip) p/ trocar o
   kind de um adjustment existente. Recomendo (a) p/ criar + (b) depois se quiser.
   `AdjustmentKind::ALL` (ou itere as 24) + nomes legíveis (em INGLÊS).
   **Sem isso, o Enio só smoca HSB.** Faça PRIMEIRO + peça smoke (B-C já anda).

2. **Fan-out dos 22 kinds restantes** (T4.4-T4.14 + Tier-2). Os de **slider** são
   triviais (§1, ~30min cada): GaussianBlur(1), Brightness/Contrast✓, Exposure(3),
   Vibrance(2), PhotoFilter(2), Posterize(1), Threshold(1), Sharpen, MotionBlur(2),
   ChromaticAberration(4), Bloom(4), Noise. **Invert(0 params)** = arm de compute
   só (sem slider). Cada um: compute OKLab/linear + golden de invariante.
   Os de **UI bespoke** (precisam controle próprio em `paint_adjust.rs`, não
   slider): **Curves** (editor de curva), **GradientMap** (editor de stops),
   **ColorLookupLut** (.cube), **SelectiveColor/Levels/ChannelMixer/
   ShadowsHighlights/BlackAndWhite** (multi-control). Deixe esses por último.
   SMOKE-INTRA Day-8 = HSB+ColorBalance+Curves+GradientMap (plano §7).

3. **Blur/convolução (Gaussian/Motion/Sharpen/Bloom)** são **espaciais** — o
   `apply_adjustment(acc)` recebe só a janela de pixels, sem vizinhança fora dela.
   Cuidado: a `Region`/dirty-rect quebra blur de raio grande. Pra v1 CPU,
   provavelmente full-canvas (sem dirty-rect) pra esses; **PARE e alinhe com o
   Coord** se precisar de halo/borda fora da janela (é do compositor = Coord).

───────────────────────────────────────────────────────────────────
§3 — FPS (NÃO é teu fix — é do Coord)
───────────────────────────────────────────────────────────────────
Cada drag de slider → recompose de canvas INTEIRO por frame. O lever real é o
**cut-point `CompositorCache`** (skeleton em `compositor.rs`, ADR-0045 §2.7) —
cachear o composite ABAIXO do adjustment; só apply+blend re-rodam. **É
foundational = Coord (C); o perf-gate `adjustment_layer_recomposition_perf_4k` é
SOFT no W4 de propósito.** NÃO construa cache nem mexa no arm Adjustment do
compositor. Eu já flaguei pro Coord (`HANDOFF_painter_w4_triage_coord.md`). Teu
único dever de perf: **neutral early-return** + manter o compute barato (OKLab
3 cbrt > 6 powf; evite transcendental redundante).

───────────────────────────────────────────────────────────────────
§4 — ARMADILHAS QUE JÁ QUEIMARAM (reuse)
───────────────────────────────────────────────────────────────────
  - **Group/Adjustment NUNCA é paint target** (não têm pixel buffer → canvas
    branco + strokes engolidos). `set_active_layer` resolve grupo→1º descendente
    pintável; `add_adjustment_layer` mantém o raster anterior ativo. Se criar
    helper que muda active, respeite isso.
  - **Inner vs outer (amendment-1):** o compositor lê os campos INNER do
    `AdjustmentLayer` (visible/opacity/blend/mask). `LayerStack::set_visible/
    opacity/blend_mode` já SINCRONIZAM o inner. Setter novo de layer → sincronize
    o inner também, senão o controle fica morto no adjustment.
  - **`cargo fmt` reflua arms de match** → ao usar Edit, RELEIA o trecho exato
    antes (meus old_strings quebraram 2×).
  - **zsh NÃO faz word-split de `$VAR` sem aspas** → liste os paths do git
    explícitos no `git add/commit`, não via variável.
  - **Botão fixo novo** exige register em `populate.rs` + forward `event.rs` +
    route `handle_panel_event` ([[feedback-panel-populate-register]]).
  - **Slider novo:** `register_if_absent` (não register) + `slider.value` do param
    a cada frame (espelho do opacity slider).
  - **IconId:** reuse um existente (ex.: `ColorEqualization` pro "+ Adj"); SVG novo
    sem variant IconId em ordem alfabética quebra TODOS os ícones.
  - **UI em INGLÊS** ([[feedback-app-ui-english-only]]).

───────────────────────────────────────────────────────────────────
§5 — TUA PASTA + GATES (isolamento DIRETRIZ §1.4)
───────────────────────────────────────────────────────────────────
EDITE: `crates/ph2d-painter-brush/src/adjustments.rs` (compute + descritores) ·
`crates/ph2d-tool-painter/` (tool/layers, NÃO o `compositor.rs` arm Adjustment —
Coord) · `crates/ph2d-panel-painter-layers/` (paint_adjust.rs + paint.rs +
event.rs + populate.rs) · `crates/ph2d-editor-core/src/ids.rs` (SÓ aditivo).
NÃO TOQUE (PARE/reporte ao Coord): `compositor.rs` arm `Adjustment` + o
`CompositorCache` (foundational/perf = Coord); `ph2d-painter-contracts` (gates de
contrato = Coord); contratos congelados (§6 CLAUDE.md — `AdjustmentKind≤32`).
GATES (batched no fim do bloco; `cargo check` ESCONDE):
  cargo test -p ph2d-painter-brush --lib adjustments
  cargo test -p ph2d-tool-painter --lib
  cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
  cargo test -p ph2d-editor-core --test architecture_panel_loc_cap --test node_id_collisions
  cargo clippy -p ph2d-painter-brush -p ph2d-tool-painter -p ph2d-panel-painter-layers --all-targets --no-deps -- -D warnings
  cargo check -p ph2d-host-desktop   # 1× no fim (mudou API pública)
Slot CoW: `bash scripts/slot-seed.sh painter` → prefixe `CARGO_TARGET_DIR=…`.

───────────────────────────────────────────────────────────────────
§6 — CARRY-OVERS / COORD-SCOPE
───────────────────────────────────────────────────────────────────
  - **Golden Photoshop-SSIM ≥0.999** (plano §7) precisa de fixture PS-exportado
    real (asset do Enio/Coord). Eu NÃO fakeei — só testes de invariante
    (neutral/extremos/alpha/round-trip). Use o mesmo padrão até existir fixture.
  - **node_id_collisions `kinds` array** (hand-maintained) não inclui os
    `PainterLayerWidget` novos (MaskInvert/MaskApply/AdjParam0..5) nem os
    CHROME_IDS novos — follow-up Coord aditivo (passa, só não-coberto).
  - **5 commits painter locais** → ship do Coord. Você NÃO pusha.
  - Detalhe do contrato/decisões: `HANDOFF_painter_w4_triage_coord.md`
    (TRIAGEM + DECISÕES DO COORD + T4.1/T4.2 LANDADO + feedback do smoke).
  - Plano: `docs/Painter_projeto/15_plano_de_implementacao.md` §7 (W4) ·
    ADR-0045 + `0045-amendment-1.md`.
═══════════════════════════════════════════════════════════════════
