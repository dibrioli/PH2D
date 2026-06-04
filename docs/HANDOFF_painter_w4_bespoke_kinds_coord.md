═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · W4 bespoke-UI kinds (Curves/Levels) — o que LANDOU
e o que precisa de você (foundational/contrato)
Autor: Implementador Painter (sessão 2026-06-04) · contexto separado
═══════════════════════════════════════════════════════════════════

Resposta ao brief `docs/HANDOFF_painter_w4_bespoke_kinds_impl.md`. Segui o
faseamento que ELE recomendou (§3 "Faseamento pragmático"): **v1 = CPU-compute +
bespoke UI + `gpu_code()=None` (fallback CPU), entregue agora**; **v2 = LUT na GPU
(o mandato real-time) = SEU**, porque o shader/binding vive em `ph2d-render`
(isolamento §0.2 do brief) e a engine GPU é sua. Entrego a MATEMÁTICA + a
LUT-build + a UI; você pluga o binding. Idem para o que precisa de dispatch
foundational (drag 2D livre de ponto) e de um canal de dados (histograma).

───────────────────────────────────────────────────────────────────
§1 — O QUE LANDOU (commit local; 100% na minha pasta; gates §5)
───────────────────────────────────────────────────────────────────
**Engine de compute (a referência canônica do mandato GPU)** —
`crates/ph2d-painter-brush/src/adjustments/`:
  - `compute.rs`: `apply_curves` + `apply_levels` (display-space, LUT-backed,
    zero transcendental/pixel no inner loop). Curves = spline cúbico MONOTONE
    (Fritsch–Carlson, sem overshoot) por canal, master∘per-canal. Levels =
    black/gamma/white in + output remap (PS-style). Wirados no dispatch
    `apply_adjustment` → já rodam no compositor CPU (`compose.rs:279`).
  - **Exporters de LUT display-space (= o que SUA binding GPU consome):**
    `pub fn curves_display_luts(&CurvesParams) -> [[f32; 256]; 3]` (R/G/B) e
    `pub fn levels_display_lut(&LevelsParams) -> [f32; 256]` (canal-uniforme).
    `pub const DISPLAY_LUT_N = 256`. **Curves E Levels são transferências 1-D
    em display-space → uma LUT 256/canal É a representação real-time.** CPU e GPU
    leem a MESMA tabela (gate de paridade = "leem a mesma LUT", ±tolerância).
  - `mod.rs`: `LevelsParams` ganhou `Default` MANUAL neutro
    `{black:0, gamma:1, white:1, out_black:0, out_white:1}` — o derive all-zero era
    DEGENERADO (white==black colapsa o range, gamma==0 inválido). Surface do
    contrato intacta (5 campos; cap não mexido).
  - `gpu_code()` de Curves/Levels = **`None`** (fallback CPU) — o gate
    `gpu_code_and_params_contract` ainda assere `None`. **VOCÊ flipa pra `Some(7)`
    / `Some(8)` quando o shader suportar** (§2) e atualiza esse assert.

**UI** — `crates/ph2d-panel-painter-layers/src/paint_adjust.rs` (+ re-export em
`ph2d-tool-painter/src/lib.rs`):
  - **Levels = COMPLETO e usável agora.** Mapeia no rack genérico de sliders
    (5 ≤ 6 slots: Black/Gamma/White/Out-Lo/Out-Hi). Zero UI nova, zero dispatch
    novo, interação nativa (snappy). Gamma é log-simétrico (neutro γ=1 no meio).
  - **Curves = editor bespoke (canvas) funcional**, dentro da máquina existente:
    canvas plotando a curva master viva (amostra o `curves_display_luts` — a mesma
    LUT que a GPU vai bindar) + handles arrastáveis. **Truque de isolamento:** os
    handles REUSAM os widgets genéricos `AdjParam0..4` como sliders VERTICAIS
    (1 por ponto), então o drag flui pelo MESMO caminho `SetValue →
    set_adjustment_param` que todo ajuste — **sem novo `InteractiveState`, sem
    novo PanelEvent, sem mexer em `event.rs`/`handle_panel_event`/ids.** v1 edita a
    curva master com X FIXO (5 handles semeados em `add_adjustment_layer`).

**Tool API** — `crates/ph2d-tool-painter/src/tool/layers.rs`:
  - `pub fn set_curve_point(id, channel, point_index, x01, y01)` — move um ponto
    (channel 0=master/1=R/2=G/3=B), re-sorta por x, mesma fast-lane de cut-cache
    do `set_adjustment_param`. **É a API que o editor 2D-livre (§3) vai chamar.**

───────────────────────────────────────────────────────────────────
§2 — [VOCÊ] BINDING DE LUT NA GPU = o mandato real-time (v2)
───────────────────────────────────────────────────────────────────
Hoje Curves/Levels caem no fallback CPU (gpu_code None) — o composite CPU lê o
canvas inteiro e faz sRGB round-trip/pixel = as "dezenas de ms" que o mandato
quer matar. A engine GPU já prova o caminho (HSB 1024² 1.7ms vs 55ms). Curves/
Levels viram real-time exatamente como os 7 escalares, MAS via LUT (não [f32;3]):

1. **`layer_composite.wgsl`** — nova binding + 2 cases:
   - `@group(0) @binding(6) var<storage, read> adj_luts: array<f32>;` (análogo ao
     `srgb_lut` binding 4). Stride por canal = `DISPLAY_LUT_N` (256).
   - `const ADJ_CURVES: u32 = 7u;` / `const ADJ_LEVELS: u32 = 8u;`
   - Cases em `apply_adjustment` (display-space, espelha o CPU):
     ```
     // base = índice do 1º float da LUT deste op (ver §2.3 como carregar)
     // canal c: s = linear_to_srgb(rgb[c]); out_s = adj_luts[base + c*256 + u32(s*255+0.5)];
     //          out[c] = srgb_to_linear_f32(out_s)
     ```
     Levels = canal-uniforme → use `base + 0*256` para R/G/B (1 tabela). Curves =
     `base + c*256` (3 tabelas). Nearest-lookup na GPU vs lerp no CPU = ±1-2 byte
     (dentro da tolerância; já documentado em `compute.rs`).
2. **`AdjParamsGpu` / WGSL `AdjParams`** (16 bytes): Curves/Levels não usam p0..p2,
   então **reuse `p0` como o índice-base no `adj_luts`** (bit-cast u32→f32 ou
   índice float — sua escolha; recomendo `p0 = base as f32`, `base` é múltiplo de
   256, exato em f32 até 2^24). Não precisa estourar os 16 bytes.
3. **flatten (lado tool, `compositor/` = sua área — §0 do brief diz "NÃO o
   compositor arm/cache" pra mim):** no `flatten_for_gpu`, para um layer
   Curves/Levels: chame `curves_display_luts`/`levels_display_lut` (já `pub`),
   `extend` a LUT no buffer `adj_luts`, set `p0 = base`, e emita
   `LayerOp::Adjustment { kind: 7|8, .. }`. Isso exige `gpu_code()` virar
   `Some(7|8)` — flip no `mod.rs` + atualizar o assert do gate.
   ⚠️ `ph2d-render` NÃO pode depender de `ph2d-painter-brush` (decoupling do
   header). Por isso a LUT é construída no TOOL (que já depende do brush) e
   passada como bytes pro compositor — exatamente como ele já recebe op-list.
4. **Paridade (RODE no Mac/Metal):** espelhe
   `gpu_adjustment_matches_cpu_reference_each_kind` pra Curves/Levels em
   `tests/layer_compositor_gpu.rs` — GPU LUT-output ≈ `apply_curves`/`apply_levels`
   CPU dentro de ±tolerância. (Sem WGSL OKLab novo aqui — Curves/Levels são
   srgb-transfer puro, igual Invert/Posterize; o gate de literais não muda.)

───────────────────────────────────────────────────────────────────
§3 — [VOCÊ] EDITOR DE CURVA 2D-LIVRE = upgrade foundational (dispatch)
───────────────────────────────────────────────────────────────────
v1 entrega handles X-FIXO (5 sliders verticais reusando AdjParam). O Photoshop-
grade (arrastar ponto em X+Y livre, add/remover ponto, abas R/G/B) precisa de
DISPATCH foundational em `ph2d-editor-core` (confirmei: não há acessor de pointer
(x,y) pra widget custom no painel; só `Slider` 1-D emite ValueChanged por-Move).
Padrão exato a copiar = o **BlenderColorPicker wheel** (a SV-rect 2-D):
  - `InteractiveState::CurvePoint { parent, index, x, y }` (ou um `BlenderHit`-style
    sub-control), + um arm em `interaction/dispatch/pointer.rs` que computa (x,y)
    do pointer+rect no Move (espelho de `apply_blender_hit`/`wheel_pick`).
  - A **API do tool já existe**: `PainterTool::set_curve_point(id, channel, idx,
    x01, y01)`. O painel decodifica o CurvePoint → forward. Se o PanelEvent
    congelado (4 variantes) não couber o (idx,x,y), encode em `SelectOption(id,
    "ch:idx:x:y")` (cabe no contrato) OU adicione a variante via ADR — sua decisão.
  - **Render:** o toolkit do painel só expõe fill/stroke de RECT (sem polyline);
    v1 plota a curva com dots densos. Pra um stroke liso, adicione um helper
    `stroke_polyline`/`fill_circle` em `editor-core/paint.rs` (foundational =
    você) ou exponha `VectorScene::inner_mut` ao painel.
  - Per-canal R/G/B: abas no canvas + `set_curve_point(channel=1..3)` (já aceita).

───────────────────────────────────────────────────────────────────
§4 — [VOCÊ] HISTOGRAMA do Levels = canal de dados (cosmético)
───────────────────────────────────────────────────────────────────
Levels já é funcional (5 sliders). O look bespoke "handles sobre histograma"
precisa das estatísticas de pixel que o painel NÃO tem. Publique um
`histogram: [u32; 256]` (ou per-canal) no snapshot de layer que o bridge já
publica, computado do composite ativo no tool. Aí o painel pinta o histograma
atrás dos handles (e troco os 5 sliders genéricos por handles bespoke). Baixa
prioridade — não bloqueia uso.

───────────────────────────────────────────────────────────────────
§5 — GATES
───────────────────────────────────────────────────────────────────
  cargo test -p ph2d-painter-brush --lib adjustments              → 36/36 ✅ (14 novos:
    curves/levels neutralidade, golden, monotonicidade, LUT-vs-apply paridade)
  cargo test -p ph2d-tool-painter --lib                           → 166/166 ✅
    (inclui set_curve_point move/no-op + os W3/W5 existentes)
  cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
                                                                  → 81/81 ✅
  cargo clippy -p ph2d-painter-brush -p ph2d-panel-painter-layers
    -p ph2d-tool-painter --all-targets --no-deps -- -D warnings   → limpo ✅
  fmt: `rustfmt --edition 2024` (toolchain pin 1.95) nos meus 7 arquivos
    (NÃO `cargo fmt -p` — reformataria WIP alheio).
  Nota: a bateria ficou ~minutos bloqueada por colisão transitória (o agente
  Vector criava `ph2d-vector-kurbo`/`ph2d-node-vector-offset` com `Cargo.toml`
  sem `lib.rs` → quebra o workspace inteiro, §6 do brief); rodou verde assim que
  estabilizou. NÃO toquei `ph2d-render` (sem WGSL novo); os gates de shader/GPU
  não são afetados por esta wave.

───────────────────────────────────────────────────────────────────
§6 — PRÓXIMOS bespoke (depois de Curves/Levels) — mesma receita
───────────────────────────────────────────────────────────────────
GradientMap/ColorLookup/SelectiveColor/ChannelMixer/ShadowsHighlights/
BlackAndWhite/ColorBalance/PhotoFilter: os que forem transferência 1-D
per-canal reusam a MESMA binding `adj_luts` (GradientMap, BlackAndWhite→tint,
PhotoFilter). Os espaciais (Gaussian/Motion/Sharpen/Bloom/ChromaticAberration/
Noise) = multi-pass GPU separável (§3.C do brief) — outro mecanismo, alinhe na vez.

───────────────────────────────────────────────────────────────────
§7 — REFERÊNCIAS
───────────────────────────────────────────────────────────────────
  - Brief de origem: `docs/HANDOFF_painter_w4_bespoke_kinds_impl.md`.
  - Engine GPU/preview: `docs/HANDOFF_painter_gpu_preview_coord.md`.
  - Exporters/compute: `crates/ph2d-painter-brush/src/adjustments/compute.rs`
    (`curves_display_luts`/`levels_display_lut`/`apply_curves`/`apply_levels`).
  - Editor: `crates/ph2d-panel-painter-layers/src/paint_adjust.rs`.
  - WGSL alvo: `crates/ph2d-render/src/shaders/layer_composite.wgsl`
    (`apply_adjustment` switch ~430, `adj_params` binding 5, `srgb_lut` binding 4).
═══════════════════════════════════════════════════════════════════
