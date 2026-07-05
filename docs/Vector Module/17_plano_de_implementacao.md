# 17 — Plano de implementação executável (tasks granulares + smokes)

> ⛔ **SUPERSEDED / PARKEADO 2026-07-05 — HISTÓRICO.** A ambição de 20 waves deste plano foi **estacionada**
> por decisão do Enio ([ADR-0108](../architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md)):
> alvo "estratosfera" sem kill-criteria = a doença que custou as semanas do Painter. O plano **canônico agora** é
> [18_plano_reposicionamento_rive_native.md](18_plano_reposicionamento_rive_native.md) — editor vetorial nativo
> (ECS/kurbo/Vello), referenciado no runtime Rive (MIT), boolean edit-time, skinning por bones, animação futura.
> Este doc fica como **registro histórico da pesquisa** — não é mais o roteiro.

> **W0 RATIFICADA 2026-05-29 — 13 ADRs Accepted + amendments policy ativa. W1 ABERTA.**
> Plano detalhado, à prova de falhas, com smokes visuais o mais cedo possível para o Enio aprovar/corrigir/melhorar antes de seguir. **20 waves × N tasks cada**. **Mandato §0 do HANDOFF_node_system + memory `feedback-perfection-no-deferrals`:** padrão-ouro absoluto, sem gambiarras, sem economias.

> **3 iterações Antigravity (Google DeepMind) absorvidas integralmente 2026-05-27/28/29.** Pipeline boolean draft+reconcile, topology vs UBO split em shader compile, Bézier cúbico default + Spiro Assist, traits + mocks W1, CRDT local nativo, physics colliders dinâmicos, Variable Fonts axes como graph inputs, Vector-SDF Hybrid GPU, Dormant Fracture Edges, shell iPad scaffold pre-W1, security sanitizers, wgpu DeviceLost recovery, Mobile Core tier <12MB. Detalhe em [`14_inovacoes_extraordinarias.md`](14_inovacoes_extraordinarias.md) e [`README §11.B + §11.C + §11.D`](README.md). **CONVERGENCE ~9.7/10 / ENDORSEMENT 9.8/10 (Antigravity 3ª iter).**

---

## §0. Princípios operacionais do plano

Esses 8 princípios governam toda task abaixo. Violar qualquer um = task NÃO está pronta.

### 0.1 Visual-first em cada wave

Cada wave entrega **pelo menos 1 smoke visual** que o Enio testa via `./play.command` (ou `PH2D_HERO_SCREEN=1 cargo run -p ph2d-host-desktop`). Implementação sem smoke não fecha. "Testes passam" não é suficiente — feature precisa ser visível.

### 0.2 Smoke intra-wave em waves grandes

Waves com >5 tasks (W1, W3, W4, W5, W6, W7, W10, W11, W12, W15, W16, W17, W18) ganham **smokes intermediários** — não esperar fim da wave para o Enio ver primeiro pixel. Smokes intermediários são marcados `🟦 SMOKE-INTRA` no plano.

### 0.3 Primeira linha vetorial no W1, day ~7

W1 não fica "duas semanas de infra invisível para Enio". Tasks T1.1..T1.7 entregam **clica Pen tool, 3 pontos = triângulo fechado renderizado via Vello** no day ~7. Polish (Pencil tool, undo via CRDT, sidebar) acontece em W1 day 8-14 e W2.

### 0.4 Tasks numeradas + dependências explícitas

Cada task tem ID `T-W.N` (W = wave, N = task index). Cada task lista **deps** (tasks anteriores que precisam estar verdes antes). Sem deps implícitas — se T2.3 precisa de T1.4, está escrito.

### 0.5 Critérios de aceitação concretos

Cada task fecha com **≥3 critérios** verificáveis. "Implementar X" não é critério. "X faz Y; teste Z passa; smoke W mostra V" é.

### 0.6 Anti-padrões listados por task

Cada task referencia explicitamente bugs já catalogados em [`docs/UI_Bugs/README.md`](../UI_Bugs/README.md) + [`docs/Image Tools Bugs/README.md`](../Image%20Tools%20Bugs/README.md) + DIRETRIZ v7.0 §5.3 + memory feedback files que **deve evitar**. "Não faça assim" é tão importante quanto "faça assim".

### 0.7 Triagem antes de cada task

Cada task começa com **triagem do caminho** per [DIRETRIZ v7.0 §2](../IntegracaoMultiAgente/DIRETRIZ.md): (A) drop-crate, (B) scaffold central, (C) Coord-only. Sem triagem, task não começa.

### 0.8 Auditoria adversarial obrigatória ≥2 lentes paralelas + critérios quantificáveis

Cada wave fecha com **≥2 agentes auditores em paralelo** (lente distinta: corretude/edge-cases · paridade/determinismo · consistência docs↔código · qualidade gold-standard · perf/budget · cross-platform). Findings → fix → re-audit → erro-zero. Espelha loop §1 do [`HANDOFF_node_system.md`](../HANDOFF_node_system.md). Memory: rotacionar lentes per [`feedback-audit-lens-diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md).

**Critérios QUANTIFICÁVEIS obrigatórios** (revisado Antigravity L8F1 2ª iteração 2026-05-28) — em vez de "findings → fix" genérico, cada audit wave fechamento precisa de **gates programáticos**:

- **Memory ceiling via dhat-rs**: zero alloc em hot path render gate (HR-3); peak memory dentro do tier budget per platform.
- **Latency thresholds**: percentile p99 < tier budget (e.g., Heavy 3.5 ms; Mobile Core 5 ms).
- **Stress test**: cada wave fechamento roda 500+ operations sequential (e.g., 500 boolean ops, 500 vertex moves, 500 brush stamps) sem leak, sem stall, sem crash.
- **Bit-identical replay**: hash blake3 cross-OS (Linux + Mac + Windows) match em fixture replay.
- **Property-based testing**: proptest 256+ random cases per gate critical.

Gates ativos por wave listados em §25.

---

## §1. Definition of Done (DoD) global por wave

Toda wave fecha quando **TODOS** os itens abaixo estão verdes:

- ☑ Todas as tasks da wave verdes (critérios de aceitação concretos cumpridos).
- ☑ Arch-gates pertinentes verdes (lista em DIRETRIZ v7.0 §5.1 + Vector-specific gates).
- ☑ `cargo clippy --workspace --all-targets -- -D warnings` verde.
- ☑ `cargo fmt --check` verde.
- ☑ Tests da wave: `cargo test -p <crates afetados>` 100% green.
- ☑ Memória dentro do budget (HR-13, vide [08 §… memory](08_performance_memory.md) quando criada; baseline desktop 200 MB VRAM + 100 MB RAM / mobile 80 + 30 / web 40 + 20).
- ☑ Frame budget dentro do sub-budget Vector (HR-4 — 3.5 ms render).
- ☑ A11y nodes presentes para widgets novos (HR-12, gate `hr12_widgets_a11y`).
- ☑ Strings via i18n Fluent (HR-15, gate `hr15_no_hardcoded_ui_strings` + `no_literal_color` + `no_magic_numeric`).
- ☑ ≥2 auditorias adversariais paralelas → findings remediadas → re-audit erro-zero.
- ☑ **Smoke visual do Enio aprovado** (explicit; cada wave lista o que ele testa).
- ☑ Crítica Antigravity (vide README §11.B) — gates aplicáveis por wave verdes (e.g., W3 boolean draft+reconcile presente; W5 shader compile cache hit-rate >95%; W10 CRDT convergence test cross-OS).

Wave NÃO fecha se algum item está vermelho. Implementador NÃO move para wave seguinte sozinho — Coord ou Enio confirma fechamento.

---

## §2. Tabela de smokes — visão de helicóptero (Enio bate o olho aqui)

Smokes do Enio por wave. Cada smoke é o **único critério que vale** para a wave ser declarada pronta visualmente.

| Wave | Smoke principal (o que o Enio testa) | Smokes intra-wave |
|------|---------------------------------------|---------------------|
| **W0** | Spec final + **11 ADRs Accepted** (0056..0066; cascata pós-crítica Antigravity). Sem ratificação Enio, W1 fica bloqueada. | — |
| **W1** | **Abre app, clica Vector pill, escolhe sprite/canvas vazio, clica Pen tool, 3 pontos na tela = triângulo fechado renderizado via Vello.** Sai do tool, commita (asset salva como `.ph2d-vector`). | 🟦 Day 3: Pen pill aparece + clica = `println!("Pen activated")`. 🟦 Day 5: 1 vertex aparece visualmente como ponto azul. 🟦 Day 7: triângulo fechado renderizado. 🟦 Day 10: persiste em `.ph2d-vector`. 🟦 Day 12: re-load mostra o triângulo. |
| **W2** | Pencil tool (freehand com Hobby fitter) + Shape tool (rect/ellipse/poly/star/spiral) + Select tool + Direct Select tool. Color picker funciona (reusa Painter `ph2d-painter-color`). Stroke + Fill básico (solid + linear gradient). Undo via CRDT edit_log (Ctrl+Z). | 🟦 Day 4: Pencil traçado smoothed. 🟦 Day 8: 5 primitives shapes funcionam. 🟦 Day 11: Select + Direct Select. 🟦 Day 14: undo via CRDT. |
| **W3** | **Cria 2 paths, adiciona Boolean Union node no Geometry Graph panel, vê resultado live; mexe slider de offset, vê resultado atualizar real-time. SDF Hybrid Pipeline (§8.7) ativo em background para preview interativo.** | 🟦 Day 4: Geometry Graph panel skeleton. 🟦 Day 8: `vector-source` consolidado (5 primitives). 🟦 Day 12: `vector-boolean` com Linesweeper. 🟦 Day 16: SDF Hybrid draft preview. 🟦 Day 20: live edit slider → re-render. |
| **W4** | Fan-out de 12 geometry nodes (`outline-stroke`, `roughen`, `twist`, `bend-path`, `pattern-along-path`, `scatter`, `width-profile`, `hatch`, `mirror`, `corner-round`, `warp`, `recolor`). Aplica `vector-roughen` num retângulo, vê perturbação live. | 🟦 Day 5: 3 nodes paralelos (roughen / mirror / corner-round). 🟦 Day 10: 6 nodes (acrescenta twist / scatter / hatch). 🟦 Day 15: 12 nodes completos. |
| **W5** | **Stroke de Pencil com pressure/tilt → variable-width real-time via GPU stroke expansion (Levien+Uguray); SDF Hybrid GPU pipeline (§8.7) ativo para boolean preview a 120 FPS; 50+ paths boolean simultaneamente sem stutter.** | 🟦 Day 4: GPU stroke expansion básico (constant width). 🟦 Day 8: pressure → width. 🟦 Day 12: tilt asymmetric envelope. 🟦 Day 16: SDF GPU compute pass ativo. 🟦 Day 20: 50+ paths boolean smooth. |
| **W6** | Procedural fill foundation — `ph2d-vector-fill` shader graph crate; nodes pilot (Noise, Voronoi, Ramp, Mix, Image-sample); user aplica fill com noise + ramp + voronoi, vê resultado live; topology compile cache evita stall (resolve crítica B). | 🟦 Day 5: 3 fill nodes. 🟦 Day 10: WGSL codegen funcional. 🟦 Day 14: cache hit-rate >95%. |
| **W7** ✨ | **Mesh gradient via diffusion curve** (§8.2, ADR-0060) — Poisson PDE compute pass; UI para autor desenhar curve + cores; diffusion curve com 3 cores produz mesh smooth pixel-perfect; perf < 5 ms / canvas 1080p. | 🟦 Day 7: solver Poisson baseline (single curve). 🟦 Day 14: 3 curves combined. 🟦 Day 21: cross-platform golden test verde. |
| W8 | Pattern Along Path + Painter brush reuse — `vector-pattern-along-path` consome `ph2d-painter-brush` library; path traçado com `pencil_2b` brush parece pintado mas é vetor; edit vertex → re-renderiza. | 🟦 Day 6: integração brush library. 🟦 Day 12: live edit vertex. |
| W9 | Symbol system parametric (Cuttle-style) — `ph2d-tool-vector-symbol` com sliders typed (number/color/enum/vector) driving geometry; Symbol "snowflake" com `arms=6` e `roughness=0.3` updates live em todas instâncias. | 🟦 Day 4: symbol struct + 1 slider. 🟦 Day 10: typed slider variants. 🟦 Day 14: multi-instance sync. |
| **W10** ✨ | **Animation foundation + Variable Fonts axes como graph inputs (§8.6, ADR-0066)** — toda param do graph animável; timeline panel; curve editor; state machine (presets); user anima `vector-roughen.amplitude` 0→1 em 2 seg; **glifo individual = vector network nativo; eixos OTF (weight/width/slant) animados via curve**. | 🟦 Day 5: timeline panel UI. 🟦 Day 10: curve editor funcional. 🟦 Day 15: state machine (3 states + blend). 🟦 Day 20: variable font axis animado por motion node. |
| W11 | Motion nodes integration — domain `motion` driving vector params + reverse (vector path como input pra motion); `motion-wave` driving `vector-roughen.amplitude` produz oscilação visível; cross-domain validation determinístico. | 🟦 Day 5: motion-wave → vector-param. 🟦 Day 10: vector path → motion-scatter-along-path. |
| **W12** ✨ | **Painter ↔ Vector bridge** (§8.3) — paint-into-vector + vector-with-brush-look + auto-trace ML. Painter brush stroke virou vector network editável; vector path renderiza com look "pencil_2b"; comando Vectorize Layer funciona. | 🟦 Day 5: paint-into-vector (Painter brush → vector.pencil). 🟦 Day 10: vector-with-brush-look. 🟦 Day 15: auto-trace ML (Sketch mode). 🟦 Day 21: 3 modos de auto-trace. |
| **W13** ✨ | **LLM-as-graph-node** (§8.4, ADR-0061) — `vector-llm-shape` consome `ph2d-vector-llm`; LLM4SVG semantic tokens; editability preserved downstream. Prompt "spiral with 8 arms golden ratio" → vector network editável; user re-prompts ou move slider downstream. | 🟦 Day 4: MCP tool esqueleto. 🟦 Day 10: semantic tokens parser. 🟦 Day 14: editable downstream. |
| W14 | Selection variants completo — Marquee + Lasso + Magic Wand (color-based) + Group Select. Knife + Bucket tools. Text on path. All selection modes funcionam; knife corta segments preserving continuity; text on path com parley. | 🟦 Day 5: Lasso + Magic Wand. 🟦 Day 10: Knife + Bucket. 🟦 Day 14: Text on path. |
| W15 | Stroke Studio + Fill Studio + Tool Studio painéis docados — full Studio editors para customizar stroke / fill / tool curves; Power user customiza Spiro tension, Hobby weight, pressure curve per-tool; save como `.ph2d-vector-tool` preset. | 🟦 Day 5: Tool Studio (Pen + Pencil curves). 🟦 Day 10: Stroke Studio (variable width axes). 🟦 Day 15: Fill Studio (shader graph editor). |
| **W16** ✨ | **Vector Runtime crate `ph2d-vector-runtime` + Dynamic Physics Colliders** (§8.5, ADR-0063) — subset runnable em release de jogo; state machine; bones + mesh deform; opt-in determinism; **Rapier 2D collider gen automático + dynamic split em runtime boolean cut**. Smoke: game-shell desktop carrega `.ph2d-vector` asset; ECS dispara state transition; espada vetor corta tábua vetor → 2 corpos rígidos com momento preservado; CI cross-OS hash test passa. | 🟦 Day 5: Runtime crate skeleton + asset loading. 🟦 Day 10: state machine + ECS bridge. 🟦 Day 15: Rapier collider gen. 🟦 Day 21: dynamic split em runtime boolean cut. 🟦 Day 28: LOD vetorial dinâmico. |
| **W17** | **Multi-plataforma input** — iPad Apple Pencil (predict+reconcile sub-9 ms), Wacom hover, Pencil Pro squeeze/barrel-roll, Android S Pen. Smoke do Enio em iPad Pro: Pencil 2 funciona, hover preview funciona, latência subjetiva indistinguível de Linearity Curve / Illustrator iPad. | 🟦 Day 7: predict+reconcile loop. 🟦 Day 14: hover preview. 🟦 Day 21: Pencil Pro features. |
| **W18** | Export interop — SVG export (round-trip lossless v1.0 subset), PDF export (paths + gradients), AI import (lossy via PDF subset), `.ph2d-vector` v1 FREEZE com migrator (HR-14). SVG output abre em browser idêntico; AI file de teste importa com layers + paths preservados. | 🟦 Day 5: SVG export básico. 🟦 Day 10: SVG round-trip. 🟦 Day 15: PDF export. 🟦 Day 21: AI import. |
| W19 | Animation export — GIF / APNG / MP4 (via `ph2d-imageio-*`) / Lottie subset / `.ph2d-vector-anim`. Animation 2-seg exporta em todos formatos; Lottie roda em After Effects básico. | 🟦 Day 5: GIF + APNG. 🟦 Day 10: MP4. 🟦 Day 14: Lottie subset. |
| **W20** | **Smoke full v1.0**: cria ilustração completa em 1 sessão (paths + boolean + procedural fill + animation + symbol + LLM-shape + auto-trace de raster + commit em `.ph2d-vector` v1). Cross-platform smoke (Mac + Linux + iPad pelo menos). Memory/perf budgets bate em CI baseline. WCAG 2.2 AA verde. | — (polish, sem smokes intermediários) |

---

## §3. Pré-W1 — 11 ADRs (W0)

**Caminho:** **(C) Coord-only**. Foundational + contratos congelados.

### T0.1 — ADR-0056 Vector Network data model

**Arquivo:** `docs/architecture/decisions/0056-vector-network-data-model.md`

**Conteúdo obrigatório:**
- `VectorNetwork` struct (vertices: SmallVec / Vec; segments com tangent handles per-end; regions com winding rule).
- **Decisão D Antigravity absorvida**: `RepresentationMode { Cubic /*default visível*/, SpiroAssist, HyperbezierAssist }` — Bézier cúbico = primário (paridade Illustrator, sem fricção muscle memory); Spiro / hyperbezier = Assist Modes opt-in (toggle HUD `S` / `H`).
- Cubic export (Levien Béz fitting).
- Edit log event-sourced — `VectorOp` payloads.
- `.ph2d-vector` postcard schema versionado (HR-14: campo `version: u32` no início).
- Determinismo opt-in (`deterministic: bool` flag).
- Caps congelados: `VectorOp ≤ N variants` (definir N), `Vertex ≤ N campos`, `Segment ≤ N campos`, `Region ≤ N campos`.
- Arch-gate `vector_contract_surface` ativo.
- Status: **Proposed** → ratificado em commit final.

**Critérios de aceitação:**
1. ADR escrito segundo template em [`docs/architecture/decisions/`](../architecture/decisions/) (Contexto / Decisão / Consequências / Alternativas).
2. Caps numéricos definidos (não "será definido depois").
3. Linka ADR-0039 (Nodegraph contract freeze) + ADR-0040 (Tool isolation) + ADR-0021 (SimWorld/PresentWorld) como pré-requisitos.
4. Decisão D Antigravity (Bézier cúbico default + Spiro Assist) explicitamente documentada com razão.
5. Enio aprova explicitamente.

**Deps:** nenhuma.

**Risco/edge case:** caps muito apertados → futuro precisa amendment. Caps muito largos → "spec do papel" sem efetividade. Mediar: pegue tamanho-real do Painter `PainterUiEdit` contract (ADR-0043, cap 24 variants) como baseline.

---

### T0.2 — ADR-0057 Vector edit dispatch + CRDT data model

**Arquivo:** `docs/architecture/decisions/0057-vector-edit-crdt.md`

**Conteúdo obrigatório:**
- Decisão: `EditorAction::VectorOp(VectorOp)` é variant novo no `EditorAction` (cap-bump) OR cabe em `ToolPanelEvent::SetValue|Click` existing?
  - **Recomendação**: variant novo `VectorOp` por payload size (paths complexos não cabem em PanelEvent simple key-value) — cap-bump `EditorAction = 5` (de 4) via amendment de ADR-0040 §7.
- **CRDT structuring `edit_log`** (resolve Proposta 5 Antigravity):
  - Trade-off analysis: **LWW-Element-Set** (Shapiro 2011 — simples, mas conflict resolution last-writer-wins pode descartar edição válida) vs **RGA** (Replicated Growable Array — preserva intent ordering, mais complexo) vs **custom CRDT** (intent-preserving para vector graph topology: vertices têm ordering em regions, tangentes conflitam).
  - **Recomendação**: hybrid — RGA para vertex/segment ordering em regions; LWW-Element-Set para set membership (which vertices in which region); custom merge para tangent handles (per-component LWW).
  - Multi-agente local (agent ↔ designer) desde W1; web cross-internet via servidor continua OUT v1.0.
- Replay determinístico cross-platform (`tests/determinism/vector_crdt_convergence.rs`).

**Critérios de aceitação:**
1. CRDT specific (LWW vs RGA vs custom) escolhido com justificativa.
2. `EditorAction::VectorOp` cap-bump aceito OR rejeitado (uso de ToolPanelEvent) com razão.
3. Convergence proof simples (2 sites editando paralelamente, merge converge).

**Deps:** T0.1.

**Risco/edge case:** LWW perde edições; RGA é overkill para single-user (>90% caso); custom CRDT debugging painful. Iteração: começar com **LWW** + testes que falham com edição concorrente → migrar para RGA/custom em W2 se necessário.

---

### T0.3 — ADR-0058 Vector geometry graph (domain `vector` no `ph2d-nodegraph`)

**Conteúdo obrigatório:**
- Domain `vector` adicionado a `ph2d-nodegraph::NodeOp` enum.
- 17 nodes canon: `vector-source` (multi-variant: rect/ellipse/poly/star/spiral, **resolve crítica A Antigravity**), `vector-boolean`, `vector-offset`, `vector-outline-stroke`, `vector-roughen`, `vector-twist`, `vector-bend-path`, `vector-pattern-along-path`, `vector-scatter`, `vector-width-profile`, `vector-hatch`, `vector-mirror`, `vector-corner-round`, `vector-warp`, `vector-recolor`, `vector-llm-shape`, `vector-luau-script`.
- Cada node: `NodeManifest` (id, name, inputs, outputs, effect [Pure|Temporal|Stateful], clock, params, lowerings).
- Caps congelados (continua usar caps ADR-0039 OR cap-bump): `NodeOp=2/OpResolver=1/NodeManifest=8` — esperar manter; cap-bump apenas se claramente necessário.
- Performance gate: graph com 50+ nodes < 3.5 ms.

**Critérios de aceitação:**
1. 17 nodes documentados com NodeManifest skeletal.
2. Caps congelados verificados (não precisa cap-bump OR cap-bump justificado).
3. Performance gate scenario documentado.

**Deps:** T0.1.

---

### T0.4 — ADR-0059 Vector renderer pipeline

**Conteúdo obrigatório:**
- Vello 0.8 integration (re-uses já existing `ph2d-vector` crate, expanded).
- GPU stroke expansion (Levien+Uguray 2024 paper, já em Vello).
- **Pipeline boolean draft+reconcile (resolve crítica C Antigravity):**
  1. Draft naive CPU (Bézier-cúbico clipping, ≤1 ms) — hot-path stylus.
  2. SDF Hybrid GPU (compute pass ≤ 0.5 ms) — real-time interactivity + gameplay morphing.
  3. Linesweeper exato async (background worker debounced, on commit) — topology canônica.
- Editor + runtime sharing renderer (HR-7).
- Frame budget 3.5 ms (sub-budget Render). Breakdown por sub-stage.
- Cache strategy (path data hash, boolean result hash).

**Critérios de aceitação:**
1. 3 modos boolean documented com timing budget.
2. Cache strategy especificada (per-modo).
3. Cross-platform target matrix (Mac/Win/Linux/iPad/Android/Web).

**Deps:** T0.1.

---

### T0.5 — ADR-0060 Procedural fill shader graph

**Conteúdo obrigatório:**
- `ph2d-vector-fill` DAG model.
- 17 fill nodes (Solid / Linear / Radial / Mesh / Diffusion / Pattern / ProceduralShader / Image / Noise / Voronoi / Ramp / Mix / Bump / Coord / Math / Image-sample / Time).
- WGSL codegen (DAG → WGSL string).
- **Topologia vs Params split (resolve crítica B Antigravity):**
  - Topology hash → compile WGSL 1× → cache on-disk (`~/.cache/ph2d/shaders/<hash>.wgsl + .spv/.msl`).
  - Params escalares → UBO atualizado por frame (zero-alloc HR-3).
  - Topology change → off-thread compile + swap atômico; durante compile usa template anterior.
- Diffusion curve solver (Walk-on-Spheres Monte Carlo OR multigrid).
- Cache hit-rate gate >95%.

**Critérios de aceitação:**
1. WGSL codegen schema definido.
2. Topology vs UBO split explicitamente documentado (resolve crítica B).
3. Cache hit-rate baseline scenario documentado.

**Deps:** T0.1, T0.3.

---

### T0.6 — ADR-0061 Vector LLM authoring (MCP tools + LLM4SVG)

**Conteúdo obrigatório:**
- Tools MCP: `vector_paint_shape`, `vector_modify_shape`, `vector_query_shape`, `vector_inspect_shape`.
- Schemas: `VectorShapeSpec`, `ShapeMod`, `ShapeFilter`, `ShapeRef`.
- LLM4SVG semantic tokens parser (não SVG opaco dump).
- Editability preserved downstream (LLM output editable like any other node).
- HR-11 governance (destructive flag + confirmation token).
- Audit log format (JSON Lines).

**Critérios de aceitação:**
1. 4 tools MCP especificadas com input/output schemas.
2. Governance HR-11 ratificada.
3. Editability preservation testada (output edita downstream e re-renderiza).

**Deps:** T0.1, T0.3.

---

### T0.7 — ADR-0062 Painter ↔ Vector bridge

**Conteúdo obrigatório:**
- API entre `ph2d-painter-brush` (existing) e `ph2d-node-vector-pattern-along-path`.
- Paint-into-vector pipeline: Painter brush stroke → Hobby fitter → vector.pencil path.
- Auto-trace ML node (`vector-auto-trace`) com 3 modos: Sketch / Illustration / Basic Shapes.
- Bidirectional flow (vector → Painter raster bake; raster → vector via auto-trace).
- Adjustment layers shared (12 Painter adjustments aplicáveis a vector layers).

**Critérios de aceitação:**
1. API surface entre crates documented (bridges não-circular).
2. Auto-trace 3 modes documentados (algorithm reference: SuperSVG / LLM4SVG / Sketch).
3. Adjustment layer sharing protocol documentado.

**Deps:** T0.1, T0.3.

---

### T0.8 — ADR-0063 Vector runtime + Dynamic Physics Colliders

**Conteúdo obrigatório:**
- `ph2d-vector-runtime` crate API.
- State machine model (Rive-inspired).
- Bones + vertex weighting.
- Mesh deformation hybrid.
- ECS integration.
- **Physics Collider Integration (Proposta 4 Antigravity):**
  - Rapier 2D 0.28 collider gen automático da VectorNetwork (decomp convex via earcut OR direct `SharedShape::convex_hull` por region).
  - **Dynamic split em runtime boolean cut**: pipeline (a) SDF GPU silhueta, (b) Linesweeper async topology, (c) collider re-decomp + split em N corpos, (d) momento preservado por split.
  - Mass derivada de `area_region × material.density`.
  - Joints opcionais entre regions.
- **LOD vetorial dinâmico (Proposta 2 Antigravity):** Bézier-aware adaptive fit pré-Vello sparse-strips; threshold driven by camera distance + bbox coverage in pixels.
- Memory budget per platform (desktop 200 MB / mobile 80 MB / web 40 MB).
- Asset loading (`.ph2d-vector` postcard parsing + caching).

**Critérios de aceitação:**
1. Runtime API documented.
2. Physics integration policy ratificada (split protocol + momentum preservation).
3. LOD threshold heuristic documented.
4. Memory budgets cross-platform.

**Deps:** T0.1, T0.3, T0.4.

---

### T0.9 — ADR-0064 Vector multi-platform input

**Conteúdo obrigatório:**
- PointerEvent unified model (vide Painter `07_pencil_pipeline.md` + ADR-0050).
- Device capability matrix (iPad Apple Pencil 2/Pro, Wacom/Huion/XP-Pen, S Pen, Mouse).
- Predict+reconcile sub-9 ms loop (ProMotion).
- `PlatformHost::pencil_predicted_touches()` extension (parallel a Painter T-input).
- Pressure / tilt / azimuth / barrel curves (per-tool override em Tool Studio).
- Palm rejection automática.
- Hover preview policy per-device.

**Critérios de aceitação:**
1. Device capability matrix completo.
2. Predict+reconcile loop documented com timing budget.
3. Palm rejection policy documented.

**Deps:** T0.1.

---

### T0.10 — ADR-0065 ✨ Vector-SDF Hybrid GPU Pipeline (Proposta 1 Antigravity)

**Conteúdo obrigatório:**
- SDF resolution policy (default 2× canvas DPI; per-asset override; per-zoom adaptive).
- Compute shader algorithm: `min(d1, d2)` união, `max(d1, -d2)` corte, `max(d1, d2)` intersect, `abs(d) - r` arredondamento.
- Vector network → SDF 2D rasterization (compute pass).
- Ordering of reductions (determinismo opt-in).
- Determinismo opt-in policy (fixed SDF resolution + ordered reductions + FMA off).
- Fallback graceful: compute shader unavailable (e.g., older WebGPU) → cai para Linesweeper síncrono com warning.
- Frame budget breakdown (SDF rasterize ≤ 0.2 ms, boolean compute ≤ 0.3 ms; total ≤ 0.5 ms).
- Limites documentados: SDF produz silhueta, **não preserva topology editável** downstream.

**Critérios de aceitação:**
1. Shader algorithm com pseudocode + WGSL snippet.
2. SDF resolution heuristic documented.
3. Determinismo opt-in policy ratificada.
4. Fallback policy explícita.

**Deps:** T0.1, T0.4.

---

### T0.11 — ADR-0066 ✨ Variable Font Glyph as Vector Network (Proposta 3 Antigravity)

**Conteúdo obrigatório:**
- Glifo individual = `VectorNetwork` nativo (cada contour vira regions; tangentes preservadas).
- Eixos OTF expostos como params: `weight` / `width` / `slant` / `optical-size` / `GRAD` / custom axes.
- Trait `VariableFontAxis { name, min, max, default, current }` — input/output do graph.
- Render path via skrifa (font parsing) + Vello (rasterization GPU compute) — sem rasterizar fonte intermediária.
- Animation hook: axes animáveis via curves + Luau scripts + motion nodes.
- HR-15 i18n locale-aware font fallback (consulta `PlatformHost::system_fonts()`).
- Differentiable Variable Fonts paper reference (2025 arXiv 2510.07638) → gradients w.r.t. axis values.

**Critérios de aceitação:**
1. Trait `VariableFontAxis` documented com 4 fields.
2. Render path skrifa + Vello documented (sem font rasterization intermediária).
3. Animation hook via motion node example documentado.
4. Font fallback chain policy documented.

**Deps:** T0.1, T0.3.

---

### T0.12 — Ativar arch-gate vazio `vector_contract_surface`

**Local:** novo arquivo `crates/ph2d-vector-doc/tests/architecture_vector_contract_surface.rs` (criado em W1).

**Conteúdo (stub W0):** arquivo presente com test `architecture_vector_contract_surface_placeholder` que sempre passa. Activa em T1.1 quando crate criado.

**Critérios:**
1. Gate stub presente.
2. Build verde (placeholder test passa).
3. Documentação inline indicando "completar em T1.1+".

**Deps:** T0.1, T0.3.

---

### T0.14 — ✨ Shell iPad scaffold (Antigravity 3ª iteração L2F3 CRITICAL 2026-05-29)

**Caminho:** **(C) Coord-only**. Foundational predecessor — shell iPad atualmente não existe em PH2D (SKILL_Stack §7 lista shell iPad ⏳ "não criada").

**Justificativa**: spec Vector Module Wave 1 + Wave 17 dependem de shell iPad existing para sub-9ms Apple Pencil + Metal Direct Overlay (Modo B §11.3.3). Sem shell, todas tasks iPad-specific bloqueiam.

**Conteúdo:**
- `shells/ipad/` directory (NEW).
- Xcode project skeleton (SwiftUI + MTKView).
- Wrapper para `ph2d-host-ffi` chamadas.
- `UIPencilInteraction` integration baseline.
- `UIAccessibility` tree publication.
- Bridge crate `ph2d-host-ios` (Rust side).
- `.cargo/config.toml` aarch64-apple-ios target.
- Build script `scripts/build-ipad.sh` (cargo lipo + Xcode).
- TestFlight staging hook (futuro).

**Critérios de aceitação:**
1. `cd shells/ipad && xcodebuild -scheme Ph2dIpad -sdk iphonesimulator15.0` produz simulator build verde.
2. Empty app launches em iPad simulator showing "PH2D Vector Module" placeholder text.
3. `ph2d-host-ffi` chamada round-trip (Swift → Rust core → Swift callback) verde.
4. Smoke do Enio em iPad físico: app launches, registra event de touch.

**Estimativa**: 5-7 dias (1 implementador full-time).

**Deps**: ADR-0064 ratificada (T0.9).

**Risco/edge case**: code signing setup em macOS para iOS dev requer Apple Developer Account + certificates. Setup inicial 1 dia. Documentar em scripts/.

**Anti-padrão**: NÃO criar shell iPad como part de Wave 1 implementer task — é foundational requisito Wave 0. Sem ela, smoke W1 day 7 em iPad ProMotion fica blocked.

---

### T0.15 — Aprovação dos ADRs pelo Enio

**Smoke W0:** Enio lê os 11 ADRs (0056..0066) + crítica Antigravity absorvida via README §11.B + ratifica.

**Cascata final ratificada (11 ADRs):**

| ADR | Título | Status alvo |
|---|---|---|
| 0056 | Vector Network data model (Bézier default + Spiro Assist) | Accepted |
| 0057 | Vector edit dispatch + CRDT (LWW/RGA/custom) | Accepted |
| 0058 | Vector geometry graph (domain `vector`) | Accepted |
| 0059 | Vector renderer pipeline (draft+reconcile boolean) | Accepted |
| 0060 | Procedural fill shader graph (topology vs UBO split) | Accepted |
| 0061 | Vector LLM authoring (MCP + LLM4SVG) | Accepted |
| 0062 | Painter ↔ Vector bridge | Accepted |
| 0063 | Vector runtime + Dynamic Physics Colliders | Accepted |
| 0064 | Vector multi-platform input | Accepted |
| 0065 ✨ | Vector-SDF Hybrid GPU Pipeline | Accepted |
| 0066 ✨ | Variable Font Glyph as Vector Network | Accepted |

**Critérios:**
1. 11 ADRs com status `Accepted`.
2. Commits locais de todos 11 ADRs em main (não pushados; push é fim-de-jornada sob ordem Enio).
3. Enio aprova explicitamente.

**Deps:** T0.1..T0.12.

**Próximo:** W1 abre.

---

## §4. W1 — Neck: Vector Network data model + Vello + first tool

**Objetivo:** primeira linha vetorial no PH2D. `VectorTool` ativa o modo, `vector-pen` deposita 3 vertices → triângulo fechado renderiza via Vello.

**Caminho:** **(A) Drop-crate fan-out** per DIRETRIZ v7.0 §3.A.

**Crates a criar (W1):**
- **Sequenciais (T1.1..T1.8):** `ph2d-vector-traits` (mocks W0), `ph2d-vector-doc` (data model), expansão de `ph2d-vector` (Vello integration), `ph2d-tool-vector-pen` (Pen tool), bridge no shell.
- **Paralelos (T-crdt / T-input):** materializam ADRs ratificados W0 que **precisam estar disponíveis ANTES do smoke Day 7**.

**Estimativa total:** ~14-18 dias (1 implementador full-time).

### T1.1 — Crate `ph2d-vector-traits` (mocks foundation)

**Caminho:** (A) drop-crate. **Resolve crítica E Antigravity.**

**Conteúdo:**
- `crates/ph2d-vector-traits/Cargo.toml` (deps mínimas: serde, glam, ph2d-color para ColorOklch).
- `src/lib.rs` com `#![forbid(unsafe_code)]` PRIMEIRO (per DIRETRIZ v7.0 §3.A.2).
- `src/anim_value.rs`: **`pub enum AnimValue { Float(f32), Vec2(Vec2), Vec3(Vec3), Color(ColorOklch), Bool(bool), Enum(u32) }`** + `Trait LinearInterp { fn lerp(a: Self, b: Self, t: f64) -> Self; }` impl per variant. **CRITICAL fix Antigravity 2ª iteração 2026-05-28** — trait original `fn sample(&self, t: f32) -> f32` quebraria animação de Vec2/Color/Bool em W10+; `AnimValue` typed enum preserva todas variantes desde W1 sem retrabalho destrutivo.
- `src/attribute_evaluator.rs`: trait `AttributeEvaluator { fn sample(&self, t: f64) -> AnimValue; }` (retorno `AnimValue`; **`t: f64` Antigravity 3ª iteração L1F1 2026-05-29** — preserve precision em sessões > 4 horas a 120Hz; `TimeContext` typed struct documentado como future V2.0 path se sub-microsecond precision necessária).
- `src/procedural_fill_shader.rs`: trait `ProceduralFillShader { fn compile(&self) -> WgslSource; }`.
- `src/animation_curve.rs`: trait `AnimationCurveSampler { fn at(&self, t: f64) -> AnimValue; }` (`t: f64`).
- `src/mocks.rs`: impl Mock para cada trait (linear interp per variant via `LinearInterp::lerp`, solid fill básico).
- Documentação inline indicando "swap mocks por real impls em W6+ / W10+".

**Critérios de aceitação:**
1. `cargo check -p ph2d-vector-traits` verde.
2. `cargo test -p ph2d-vector-traits` (5+ tests; cada trait + mock + cada variant de `AnimValue::lerp` testados) verde.
3. Mocks usáveis em W1-W5 antes de Shader Graph / Animation System maduros.
4. **`AnimValue` enum cobre todas variantes necessárias para W10+** (Float / Vec2 / Vec3 / Color / Bool / Enum); test em `tests/anim_value_coverage.rs` valida que cada variant é interpolável.

**Deps:** nenhuma (foundational). Depende implicitamente de `ph2d-color` (existing crate) para `ColorOklch`.

**Anti-padrão:** não criar traits "for the sake of abstraction" — só os 3 que destrancam W1-W5 (vide crítica E). **NÃO** restringir return type a escalar `f32` (crítica L1F4 Antigravity 2ª iteração — quebra W10+ retroativamente).

---

### T1.2 — Crate `ph2d-vector-doc` skeleton (VectorNetwork + Bézier cúbico)

**Caminho:** (A) drop-crate.

**Conteúdo:**
- `crates/ph2d-vector-doc/Cargo.toml` (deps: serde, postcard, smallvec, kurbo via `ph2d-vector::kurbo` re-export, glam, blake3).
- `src/lib.rs` com `#![forbid(unsafe_code)]` + `#![deny(missing_docs)]`.
- `src/network.rs`: `VectorNetwork { vertices: SmallVec<[Vertex; 32]>, segments: SmallVec<[Segment; 64]>, regions: SmallVec<[Region; 8]> }`.
- `src/cubic.rs`: `Vertex { pos: Vec2, in_tangent: Vec2, out_tangent: Vec2, kind: VertexKind }` — Bézier cúbico **default visível** (decisão D Antigravity).
- `src/region.rs`: `Region { segments: SmallVec<[SegmentRef; 16]>, winding: WindingRule }`.
- `src/edit_log.rs`: `VectorOp` enum + `EditLog { ops: Vec<VectorOp> }` (sem CRDT ainda; T1.6 adiciona).
- Schema versionado `version: u32` (HR-14).
- `src/cubic_fit.rs` STUB (será populated em T1.4 + W2).
- `src/spiro.rs` STUB (Assist mode, populated em W2).
- Arch-gate `vector_contract_surface` active.

**Critérios de aceitação:**
1. `cargo check -p ph2d-vector-doc` verde.
2. Caps documented (vertices ≤ 32 inline; cresce no heap acima): Painter parallel.
3. `cargo test -p ph2d-vector-doc` (smoke initial: criar triangle network, serialize, deserialize, hash match) verde.
4. Arch-gate ativo (`architecture_vector_contract_surface.rs` test passa).

**Deps:** T0.13 (W0 ratificada).

**Anti-padrão:**
- Não usar `HashMap` em sim path (HR-5 + ADR-0022): use `BTreeMap` ou `Vec` indexed.
- Não alocar em hot path (HR-3): SmallVec inline; bump arena por frame.

---

### T1.3 — Expansão `ph2d-vector` para Vello pipeline integration

**Caminho:** (A) drop-crate (expansão; arquivo existing).

**Conteúdo:**
- Em `crates/ph2d-vector/src/scene.rs` (existing): adicionar `pub fn draw_vector_network(&mut self, network: &VectorNetwork, transform: Affine)`.
- Conversão `VectorNetwork → kurbo::BezPath` per region.
- Fill (solid color baseline; gradient + procedural em W6).
- Stroke (constant width baseline; variable em W5).
- Re-export `VectorNetwork` from `ph2d-vector-doc` (transitive dep).

**Critérios de aceitação:**
1. `ph2d-vector` builda com novo método.
2. Smoke offline: `draw_vector_network(triangle)` produz Vello scene válido (renderizável).
3. Cross-platform render output bit-identical em det-mode (W17+ valida; W1 confia em Vello).

**Deps:** T1.2.

---

### T1.4 — Cubic fitting (Levien Béz fitting)

**Caminho:** (A) drop-crate (em `ph2d-vector-doc/src/cubic_fit.rs`).

**Conteúdo:**
- Implementar Bézier fitting Levien-style (referência: <https://raphlinus.github.io/curves/2021/03/11/bezier-fitting.html>).
- Converte poly-segment para single cubic best-fit.
- Usado para export SVG + canonicalization.

**Critérios de aceitação:**
1. Fit em 5 fixtures conhecidas: triangle, circle (4 cubics), spiral, hand-drawn pencil, complex polygon.
2. Max error < 0.5 px em todas as 5.
3. Performance < 1 ms para path de 100 segments.

**Deps:** T1.2.

**Anti-padrão:** não Schneider's algorithm (Inkscape default, sub-ótimo) — Levien é o canônico.

---

### T1.5 — Crate `ph2d-tool-vector-pen` (Pen tool com Bézier cúbico default)

**Caminho:** (A) drop-crate.

**Conteúdo:**
- `crates/ph2d-tool-vector-pen/Cargo.toml`.
- `src/lib.rs` PRIMEIRO com `pub const MANIFEST: ToolManifest { id: "vector_pen", cluster: "vector_tools", label_key: "tool.vector_pen.label" }` + `pub fn register(reg: &mut Registry)` + `pub fn make() -> Box<dyn Tool>`. **Naming**: id usa snake_case (`vector_pen`) para HR-15 i18n gate (`tool.<id>.label` literal) + convenção projeto (`color_equalization`, `equalize_sizes`). Icon slug usa hyphen (`vector-pen`) por SVG/Lucide convention — mesmo split de `bgremoval` (id) / `bg-removal` (slug). Atualizado pós-T1.5 audit R1.
- `src/tool.rs` impl Tool: `id() -> ToolId::new("vector_pen")`, `icon_slug() -> "vector-pen"`, `label() -> "Vector Pen"`, `as_any_mut() -> Some(self)`. label_key fica no manifest, não na trait.
- Click adiciona vertex; click+drag estica tangentes cúbicas; close-path detection (proximidade de start vertex).
- **Bézier cúbico = default visível** (decisão D); Spiro/Hyperbezier Assist toggle (HUD `S` / `H`) — UI stub em W1, lógica em W2.
- `src/icon.rs`: BezPath placeholder (pen-icon Lucide 24×24).
- IconId variant `VectorPen` em `ph2d-editor-core/src/icons.rs` em ordem alfabética.
- SVG `docs/design/icons/vector-pen.svg`.

**Critérios de aceitação:**
1. `cargo check -p ph2d-tool-vector-pen` verde.
2. `cargo run -p ph2d-tool-sync` rodado; `cargo test -p ph2d-tool-registry-init` verde.
3. Gate `enum_order_matches_svgs` verde (IconId alfabético).
4. Pen tool aparece na sidebar/topbar do app + clica = ativa modo.

**Deps:** T1.2, T1.3.

**Anti-padrão:** não inventar variant novo em `EditorAction` (HR §7.2 + memory `feedback_new_tool_icon_needs_iconid`).

---

### T1.6 — CRDT data model em `ph2d-vector-doc/src/crdt.rs`

**Caminho:** (A) drop-crate (em existing crate).

**Conteúdo:**
- Implementar **LWW-Element-Set** baseline (Shapiro 2011) para set membership.
- **RGA** (Replicated Growable Array) para vertex/segment ordering em regions.
- Custom merge para tangent handles (per-component LWW por axis).
- `CrdtReplay::apply(EditLog)` produz state convergente.
- `tests/crdt_convergence.rs`: 2-site simulation, paralelo edits, merge converge.

**Critérios de aceitação:**
1. CRDT spec implementada (LWW + RGA + custom).
2. Convergence test 2-site passa (5+ scenarios).
3. Replay determinístico cross-platform (gate `tests/determinism/vector_crdt_convergence.rs`).

**Deps:** T1.2.

**Anti-padrão:** não over-engineer CRDT — LWW suficiente para single-user; RGA habilitado quando concurrent edit possible (LLM agent + user).

---

### T1.7 — Bridge no shell — `vector_pen_bridge.rs` (smoke Day 7 alvo)

**Caminho:** (A) drop-crate (shell file).

**Conteúdo:**
- `shells/desktop/src/render_loop/vector_pen_bridge.rs` (NEW, espelha `bgremoval_preview.rs`).
- `refresh_vector_pen_preview(state, target)`: chamado a cada frame quando Pen tool ativo. Lê `VectorTool::current_network()`, gera Vello scene preview, deposita em layer overlay.
- `commit_vector_pen(state, asset)`: chamado em close-path. Salva `.ph2d-vector` em assets via `ph2d-asset`.
- Smoke: clica Pen pill, 3 pontos no canvas, vê triângulo fechado renderizado, sai do tool, asset salva.

**Critérios de aceitação:**
1. Bridge file segue padrão `bgremoval_preview.rs` (HR-1 + DIRETRIZ §3.A.4).
2. Smoke Day 7 verde: triângulo aparece como vetor.
3. Persiste em disk (asset save).

**Deps:** T1.5, T1.3.

**Smoke W1 Day 7 (visual_first_in_each_wave §0.1):** Enio clica Pen tool, 3 pontos → triângulo. ✓

---

### T1.8 — Tests, audit, fechamento W1

**Conteúdo:**
- Unit tests + integration tests em todas crates W1.
- Auditoria adversarial ≥2 lentes paralelas:
  - Lente A — **corretude/edge-cases**: vértices coincidentes, self-intersecting paths, regions degeneradas (zero area).
  - Lente B — **determinismo replay**: serialize/deserialize idempotente; hash blake3 estável cross-OS.
  - Lente C (opcional) — **multi-agent CRDT**: 2 sites editing paralelo, merge converge.
- Findings → fix → re-audit → erro-zero.
- Smoke do Enio confirmed.

**Critérios:**
1. ≥2 auditorias paralelas concluídas com findings remediados.
2. Smoke Enio Day 7 + Day 14 (close-path + save) ambos verde.
3. CI cross-OS hash test stable.

**Deps:** T1.1..T1.7.

---

## §5. W2 — Pencil + Shapes + Select + Color picker + Undo via CRDT

**Objetivo:** suite de tools básicos funcionais; user pode desenhar logo simples.

**Caminho:** **(A) Drop-crate fan-out** paralelizando 5 tools.

**Tasks (paralelas onde possível):**

### T2.1 — `ph2d-tool-vector-pencil` (Hobby fitter)

**Conteúdo:** Pencil tool com **Hobby's algorithm** (minimum curvature variation, MetaPost). Stroke recording (pressure/tilt/azimuth via `ph2d-input`). Auto-smooth on commit.

**Critérios:** pencil fits → 1 cubic per 10 input samples; smooth visually; pressure → width hooks (real em W5).

**Anti-padrão:** Schneider's algorithm (Inkscape default, ringi).

**Deps:** T1.5, T1.2.

---

### T2.2 — `ph2d-tool-vector-shape` (rect/ellipse/poly/star/spiral)

**Conteúdo:** Shape tool com 5 sub-modes (toggle no panel). Cada um emite primitivo configurado de `vector-source` (que será consolidado em W3 T3.2).

**Critérios:** 5 shapes funcionam; live preview enquanto drag.

**Deps:** T1.5, T1.2.

---

### T2.3 — `ph2d-tool-vector-select` + `ph2d-tool-vector-direct`

**Conteúdo:** Select (marquee + click) trabalha em network/region level. Direct Select trabalha em vertex/tangent level (drag move; alt-drag breaks tangent).

**Critérios:** marquee select multi-network; direct select move vertices live; tangent break funciona.

**Deps:** T1.2.

---

### T2.4 — Color picker reuse (`ph2d-painter-color`)

**Conteúdo:** Vector Module reusa o color picker do Painter (`ph2d-panel-vector-inspector` consome `ph2d-painter-color::ClassicPicker`). Solid fill + linear gradient para regions.

**Critérios:** color picker abre; user pinta fill region; gradient editor 2-stop linear funciona.

**Deps:** T1.2; depende de Painter `ph2d-painter-color` existing (vide HANDOFF_painter).

---

### T2.5 — Undo via CRDT edit_log

**Conteúdo:** Ctrl+Z em chrome handler invoca `VectorTool::undo()` → `EditLog::revert_last_op()` → re-render via dirty rect propagation.

**Critérios:** undo funciona 50+ ops sem corruption; redo (Ctrl+Y) recoloca; smoke Enio com 10 undo/redo sequence.

**Deps:** T1.6.

---

### T2.6 — Audit + fechamento W2

**Conteúdo:** ≥2 auditorias paralelas (lentes: UX consistency · perf undo with 1000 ops · CRDT convergence em multi-agent simulation).

---

## §6. W3 — Geometry Graph foundation + 3 nodes pilot + SDF Hybrid draft

**Objetivo:** Geometry Graph panel funcional; user clica 2 paths, adiciona Boolean node, vê resultado live com SDF preview real-time.

**Caminho:** **(A) Drop-crate fan-out** para nodes + **(B) Scaffold central** para panel.

### T3.1 — Crate `ph2d-panel-vector-graph` (B — Coord scaffold)

**Coord-B** plumba o panel scaffold (registra em `register_all_panels`, adiciona ao `chrome/mod.rs::dispatch_all` se chrome handler novo) ANTES de delegar a implementação.

**Conteúdo:** panel docado para visualizar/editar Geometry Graph; node placement; edge drag; param sliders.

**Critérios:** panel registra; abre quando vector layer selecionado; node placement funciona.

**Deps:** T1.2, T1.3.

---

### T3.2 — Crate `ph2d-node-vector-source` (consolida 5 primitives — **resolve crítica A Antigravity**)

**Conteúdo:**
- Multi-variant: `vector-source.rect`, `vector-source.ellipse`, `vector-source.polygon` (sides param), `vector-source.star` (points + inner/outer radius), `vector-source.spiral` (turns + radius).
- Cada variant: NodeManifest + algorithm (emit VectorNetwork) + golden test.

**Critérios:** 5 primitives renderizam corretos; golden tests bit-identical cross-OS.

**Deps:** T0.3 (W0 ADR-0058 ratificada).

---

### T3.3 — Crate `ph2d-node-vector-boolean` (Linesweeper exato + SDF draft)

**Conteúdo:**
- Pipeline boolean draft+reconcile (resolve crítica C):
  - SDF GPU compute pass (draft real-time, ≤ 0.5 ms) via shader em `crates/ph2d-vector/shaders/boolean_sdf.wgsl`.
  - Linesweeper exato async background worker (com debounce) para topology canônica.
- 9 variants: union / subtract / intersect / exclude / divide / trim / merge / crop / outline.
- Cache by hash (graph input hash → result network).

**Critérios:** todas 9 variants funcionam com Linesweeper; SDF draft visível durante slider drag; commit refresh para topology exata.

**Deps:** T0.10 (ADR-0065 SDF Hybrid), T1.3, T3.1.

---

### T3.4 — Crate `ph2d-node-vector-offset`

**Conteúdo:** parallel/contour offset live via Euler-spiral approximation (GPU stroke expansion technique).

**Critérios:** offset funciona; live edit slider; results bit-identical cross-OS.

**Deps:** T3.1, T1.3.

---

### T3.5 — Audit + fechamento W3

**Lentes paralelas:**
- Lente A: **edge cases boolean** (coincident edges, tangent contact, shared vertices) — onde Clipper falha, Linesweeper deve resistir.
- Lente B: **SDF vs Linesweeper consistency** — visual match na maioria dos casos (silhueta concordante).
- Lente C: **perf** — 100 paths boolean cabe em frame budget?

---

## §7. W4 — Fan-out 12 geometry nodes paralelos

**Caminho:** **(A) Drop-crate fan-out** — 12 Implementadores em paralelo (slots isolados via `scripts/slot-env.sh`).

### T4.1..T4.12 (paralelas)

Cada uma:
- `ph2d-node-vector-{outline-stroke, roughen, twist, bend-path, pattern-along-path, scatter, width-profile, hatch, mirror, corner-round, warp, recolor}`.
- Conteúdo: NodeManifest + algorithm + golden test.

**Critérios por node:**
1. `cargo check -p <crate>` verde.
2. `cargo run -p ph2d-node-sync` regenera registry.
3. `cargo test -p ph2d-node-registry-init` (staleness) verde.
4. Golden test bit-identical cross-OS.
5. Node aparece no Geometry Graph panel UI.

**Smoke W4:** Enio aplica `vector-roughen` num retângulo, vê perturbação live; aplica `vector-mirror` Quad, vê 4 cópias.

**Deps:** T3.1, T0.3.

---

### T4.13 — Audit + fechamento W4

**Lentes:** corretude per-node · perf agregado (graph com 6 nodes encadeados) · consistency (panel + render + edit_log).

---

## §8. W5 — GPU stroke expansion + variable-width + SDF Hybrid full

**Objetivo:** pressure/tilt → width-profile real-time via Vello GPU; SDF Hybrid GPU pipeline ativo para preview interativo.

### T5.1 — Variable-width stroke via Vello

**Conteúdo:** consome Vello GPU stroke expansion (paper Levien+Uguray 2024, já integrado em Vello). `WidthProfile` (1D variable-font-style axes: width / taper / contrast / jitter / pressure).

**Critérios:** stroke smooth com pressure variation; sub-9 ms ProMotion latency.

**Deps:** T1.3, T2.1.

---

### T5.2 — SDF Hybrid Pipeline ativo full

**Conteúdo:** SDF GPU compute pass para preview real-time de N boolean ops concorrentes. Activate via flag em asset OR per-tool mode.

**Critérios:** 50+ paths boolean simultaneamente a 120 FPS; gate `vector_sdf_real_time` verde.

**Deps:** T3.3.

---

### T5.3 — Audit + fechamento W5

---

## §9. W6 — Procedural fill foundation (shader graph)

**Objetivo:** `ph2d-vector-fill` shader graph crate; user aplica fill com noise + ramp + voronoi, vê live; topology compile cache funciona.

### T6.1 — Crate `ph2d-vector-fill` skeleton

**Conteúdo:** FillGraph DAG model. Node enum (Solid / Linear / Radial / Noise / Voronoi / Ramp / Mix / Bump / Coord / Math / Image-sample / Time).

**Critérios:** DAG parse + valida; nodes pilot (Noise + Ramp + Mix) funcionam offline.

---

### T6.2 — WGSL codegen (DAG → WGSL string)

**Conteúdo:** codegen visitor que percorre DAG, emite WGSL function chain; cache por hash (topology hash → WGSL).

**Critérios:** 5 fixtures codegen-gold-test passa; cache hit-rate >95% em scenario animation 60 frames.

---

### T6.3 — Topology vs UBO split (**resolve crítica B Antigravity**)

**Conteúdo:**
- Topology hash → compile WGSL 1× via naga → cache on-disk (`~/.cache/ph2d/shaders/<hash>.{wgsl,spv,msl}`).
- Params escalares → UBO atualizado por frame (zero-alloc HR-3).
- Topology change → off-thread compile + swap atômico; durante compile usa cached anterior.

**Critérios:** animate 60 frames de param escalar = 0 recompilations; gate `procedural_fill_no_recompile_on_animate` verde.

---

### T6.4 — Audit + fechamento W6

**Lentes:** **shader compile stutter** (crítica B) · cross-platform shader output bit-identical · cache invalidation correctness.

---

## §10. W7 — Mesh gradient via diffusion curve (Poisson PDE)

**Objetivo:** mesh gradient via diffusion curve (Unified Smooth Vector Graphics 2024 paper); autor desenha curva, marca cores, GPU resolve Poisson.

**Estimativa revisada Antigravity L8F2 2ª iteração 2026-05-28**: **35 dias** (era 21; aumentado por reconhecimento de research-grade complexity). Inclui:
- 7 dias prototipagem CPU multigrid baseline (validar matemática + boundary conditions).
- 14 dias GPU compute Walk-on-Spheres / multigrid implementation.
- 7 dias adaptive resolution + bilateral filter upscale tier-aware.
- 4 dias CPU SIMD fallback (Mobile Core entry).
- 3 dias cross-platform golden tests.

### T7.0 — Prototipagem CPU multigrid baseline (NEW, blindagem)

**Conteúdo:**
- `crates/ph2d-vector-fill/src/poisson_cpu.rs` — multigrid CPU SIMD baseline.
- 4 levels (1080p → 540p → 270p → 135p → solve → upscale).
- Validar boundary conditions (curve cores + blur radius).

**Critérios:**
1. 3-curve fixture converge < 50 iterations.
2. CPU SIMD ~15-25 ms / 1080p single-core (acceptable Mobile Core fallback).
3. Visual quality match GPU baseline (SSIM ≥ 0.995).

**Deps:** T0.5 (ADR-0060 ratificada).

### T7.1 — Poisson PDE solver compute pass GPU

**Conteúdo:**
- `crates/ph2d-vector-fill/shaders/diffusion.wgsl` — Walk-on-Spheres Monte Carlo OR multigrid GPU.
- Boundary: curva com cor em ambos lados + opcional blur.
- Iteration count adaptive (convergência threshold).
- **Tier-aware resolution** (vide [05 §5.6.5](05_procedural_fill.md)): Heavy 1080p × 64spp / Standard 540p × 32spp / Lite 270p × 16spp / Mobile Core CPU fallback.
- Bilateral filter upscale single-pass.

**Critérios:** 3 cores em 1 diffusion curve produz smooth mesh; perf within tier budget (vide [05 §5.6.5](05_procedural_fill.md) gate `vector_diffusion_curve_tier_budget`); cross-platform golden test SSIM ≥ 0.995.

---

### T7.2 — UI para autor diffusion curve

**Conteúdo:** Diffusion Curve tool — desenha curva, click side esquerdo/direito para set cor, slider para blur radius.

**Critérios:** UX intuitivo (vide [pesquisa Linearity Curve pencil tool]).

---

### T7.3 — Audit + fechamento W7

---

## §11. W8 — Pattern Along Path + Painter brush reuse

**Objetivo:** path traçado com brush Painter parece pintado mas é vetor editável.

### T8.1 — `ph2d-node-vector-pattern-along-path` consome `ph2d-painter-brush`

**Conteúdo:** node distribui stamps Painter ao longo do path; spacing / jitter / scatter params.

**Critérios:** path desenhado com `pencil_2b` parece pintado; edit vertex → re-renderiza imediato.

**Deps:** Painter `ph2d-painter-brush` existing.

---

### T8.2 — Audit + fechamento W8

---

## §12. W9 — Symbol system parametric (Cuttle-style)

**Objetivo:** Symbol "snowflake" com `arms=6` + `roughness=0.3` updates live em todas instâncias.

### T9.1 — `ph2d-tool-vector-symbol`

**Conteúdo:** Symbol = parametric vector network template; instances bind params via graph; typed slider variants (number / color / enum / vector).

**Critérios:** symbol cria + N instâncias; edit param master → sync todas; smoke "snowflake".

---

### T9.2 — Symbol Library panel

**Conteúdo:** `ph2d-panel-vector-symbol-lib` — browse symbols, drag-and-drop pro canvas.

---

### T9.3 — Audit + fechamento W9

---

## §13. W10 — Animation + Variable Fonts axes (NEW §8.6)

**Objetivo:** toda param animável; timeline panel; state machine; **glifo individual = vector network nativo; eixos OTF animados**.

### T10.1 — Animation foundation

**Conteúdo:** consome `ph2d-vector-traits::AnimationCurveSampler` (mock em W1, real em W10). Timeline panel com tracks por param. Curve editor (Bézier handles).

**Critérios:** animate `vector-roughen.amplitude` 0→1 em 2 seg; preview no canvas; export GIF baseline.

---

### T10.2 — State machine (Rive-style)

**Conteúdo:** state = preset de params; transition com blend (linear / ease / spring). State machine plugável em runtime W16.

**Critérios:** 3 states + blend funcionam; smoke "hover → press → release".

---

### T10.3 — Variable Fonts axes como graph inputs (**§8.6, ADR-0066**)

**Conteúdo:**
- `crates/ph2d-vector-font/` (NEW crate) — consome `skrifa` para parse variable fonts; glifo individual → VectorNetwork.
- Trait `VariableFontAxis` exposto como graph input/output.
- 4 axes default: weight / width / slant / optical-size.
- Render path via `skrifa` → `kurbo::BezPath` per glyph → Vello.

**Critérios:** variable font carregada; axes expostos no Inspector; animate `weight` via curve → letterform deforma live.

---

### T10.4 — Audit + fechamento W10

**Lentes:** animation perf · variable font axis interpolation correctness · cross-platform font rendering.

---

## §14. W11 — Motion nodes integration

**Objetivo:** `motion-wave` driving `vector-roughen.amplitude`; reverse (vector path → motion-scatter-along-path).

### T11.1 — Motion nodes → vector params

**Conteúdo:** ph2d-nodegraph já suporta cross-domain (motion.wave outputs f32, vector-roughen.amplitude accepts f32). Validar end-to-end.

**Critérios:** motion-wave output animado → vector-roughen visual updates.

---

### T11.2 — Vector path → motion-scatter-along-path

**Conteúdo:** motion-scatter-along-path consome `VectorNetwork::path()` como input.

**Critérios:** sprites scattered along vector path live.

---

### T11.3 — Determinismo cascading

**Conteúdo:** se motion graph é determinístico (SimWorld), output vector também é (HR-5). Validate em cross-OS test.

---

### T11.4 — Audit + fechamento W11

---

## §15. W12 — Painter ↔ Vector bridge (§8.3)

**Objetivo:** paint-into-vector + vector-with-brush-look + auto-trace ML.

### T12.1 — Paint into vector

**Conteúdo:** Painter brush stroke → Hobby fitter → vector.pencil path. Pressure → width-profile. Tilt → asymmetric envelope.

**Critérios:** pintar com Painter brush, ver vector path nascer editável.

---

### T12.2 — Vector with brush look

**Conteúdo:** já feito em W8 via vector-pattern-along-path consumindo ph2d-painter-brush. Reusar.

---

### T12.3 — Auto-trace ML (Sketch / Illustration / Basic Shapes)

**Conteúdo:**
- `crates/ph2d-node-vector-auto-trace/` (NEW).
- 3 modos: Sketch (line detection), Illustration (color region quantize), Basic Shapes (primitive fit).
- Algorithm: SuperSVG / LLM4SVG (embed model se necessário) ou external lib (potrace fallback).

**Critérios:** raster image → auto-trace produz vector network editável; 3 modos visualmente distintos.

---

### T12.4 — Audit + fechamento W12

---

## §16. W13 — LLM-as-graph-node (§8.4, ADR-0061)

**Objetivo:** prompt "spiral with 8 arms golden ratio" → vector network editável; user re-prompts ou move slider downstream.

### T13.1 — `crates/ph2d-vector-llm/` skeleton

**Conteúdo:**
- MCP tool `vector_paint_shape(prompt, constraints) -> VectorNetwork`.
- Semantic tokens parser (LLM4SVG-style structured output).
- Editability preservation: output é VectorNetwork standard, editável downstream do node.

**Critérios:** prompt → vector aparece; edit vertex pos downstream funciona.

---

### T13.2 — Node `vector-llm-shape`

**Conteúdo:** node graph wrapper para `ph2d-vector-llm`. Params: prompt (String), seed (u64), style_ref (opcional).

**Critérios:** node aparece no graph; re-prompt + re-bake; output editável.

---

### T13.3 — HR-11 governance

**Conteúdo:** `vector_paint_shape` é mutative (não destructive). `vector_delete_path` é destructive — confirmation token + audit log.

---

### T13.4 — Audit + fechamento W13

---

### T13.5 — ✨ Fuzz testing targets (Antigravity 3ª iteração L3F1 + L4F1/F2 2026-05-29)

**Caminho:** **(A) drop-crate** — `crates/ph2d-vector-fuzz/` (NEW).

**Conteúdo:**
- `crates/ph2d-vector-fuzz/Cargo.toml` com `libfuzzer-sys` dep + bin targets.
- `fuzz_targets/wgsl_codegen.rs` — input arbitrary DAG; assert codegen output válido (não panic, não recursion infinite, ≤ 50 KB output).
- `fuzz_targets/llm_semantic_tokens.rs` — input arbitrary semantic token JSON; assert parser handles gracefully (bounded vertices/segments, no panic).
- `fuzz_targets/postcard_asset.rs` — input arbitrary bytes; assert `load_vector_asset()` panic-free, bounded memory.
- Daily CI dedicated workflow (`.github/workflows/vector-fuzz.yml`) roda fuzz 1 hora per target; report findings.

**Critérios de aceitação:**
1. 3 fuzz targets compilam + rodam sem crash em smoke (60s each).
2. Daily CI workflow ativo; results published em GitHub Actions artifacts.
3. Zero panics em 24h fuzz run (gate `vector_fuzz_24h_no_panic`).
4. Coverage report (per `cargo fuzz coverage`) ≥ 70% nas funções target.

**Deps:** T6.2 (WGSL codegen exists), T13.1 (LLM parser exists), T1.2 (postcard schema).

**Anti-padrão**: NÃO usar fuzz targets como gate W1-W3 (premature — nada para fuzz ainda). Fuzz começa W6+ quando codegen + parser maduros.

---

## §17. W14 — Selection variants + Knife + Bucket + Text on path

### T14.1 — Selection completo

**Conteúdo:** Marquee (rect) + Lasso (freehand) + Magic Wand (color-based) + Group Select (path → all in group).

---

### T14.2 — Knife tool

**Conteúdo:** corta segments preservando continuity; ray-cast hit-test + segment split em duas BezPath.

---

### T14.3 — Bucket tool

**Conteúdo:** region fill + flood fill quando network tem hole.

---

### T14.4 — Text on path

**Conteúdo:** parley layout + kurbo `ParamCurveNearest::nearest()` per segment → glyph positioning. Reusa `ph2d-text`.

---

### T14.5 — Audit + fechamento W14

---

## §18. W15 — Stroke Studio + Fill Studio + Tool Studio panels

### T15.1 — Tool Studio panel

**Conteúdo:** edit pressure curves per-tool, Spiro tension, Hobby weight, etc.

---

### T15.2 — Stroke Studio panel

**Conteúdo:** edit variable width axes, stroke profile envelope, dashes, caps.

---

### T15.3 — Fill Studio panel

**Conteúdo:** shader graph editor (drag-and-drop fill nodes; reusa `ph2d-panel-vector-graph` pattern).

---

### T15.4 — Save/load `.ph2d-vector-tool` preset

**Conteúdo:** export user-defined tool preset; share-able.

---

### T15.5 — Audit + fechamento W15

---

## §19. W16 — Vector Runtime + Dynamic Physics Colliders (§8.5)

**Objetivo:** crate ship-em-jogo; sword cut → 2 corpos rígidos.

### T16.1 — `crates/ph2d-vector-runtime/` skeleton

**Conteúdo:** subset runnable em release de jogo (sem editor). `.ph2d-vector` asset loader. State machine runner. ECS integration.

---

### T16.2 — State machine + ECS bridge

**Conteúdo:** `EditorAction::ActivateState("hover")` triggera transition; blend interpola params; render WYSIWYG vs editor.

---

### T16.3 — Rapier 2D collider generation

**Conteúdo:** VectorNetwork → Rapier `Collider`. Decomp convex via earcut OR direct `SharedShape::convex_hull` por region. Mass = area × density.

---

### T16.4 — Dynamic split em runtime boolean cut (**Proposta 4 Antigravity**)

**Conteúdo:**
- Pipeline cut: (a) SDF GPU silhueta imediata, (b) Linesweeper async topology, (c) collider re-decomp + split em N corpos, (d) momento linear+angular preservado por split.
- Tests: sword cuts plank → 2 RigidBody com mass + velocity conservation.

**Critérios:** smoke do Enio "espada corta tábua → 2 pedaços caem com física correta".

---

### T16.5 — LOD vetorial dinâmico (**Proposta 2 Antigravity**)

**Conteúdo:** runtime aplica Bézier-aware adaptive fit (Levien flatten + RDP em 1ª pass, re-fit a poucos cúbicos). Threshold via camera distance + pixel coverage.

**Critérios:** 50+ vector elements em tela cabe no frame budget 3.5 ms.

---

### T16.6 — Audit + fechamento W16

**Lentes:** runtime perf · physics correctness (momentum conservation) · LOD visual quality.

---

## §20. W17 — Multi-plataforma input (Pencil/Wacom/S Pen)

### T17.1 — Predict+reconcile sub-9 ms loop

**Conteúdo:** prediction com extrapolation linear/cubic; render speculative; reconcile on input arrival. Espelha Painter T-input (ADR-0050).

**Critérios:** latência subjetiva indistinguível de Linearity Curve em iPad Pro.

---

### T17.2 — Hover preview

**Conteúdo:** Spiro tangent preview + variable width thickness preview antes do click.

---

### T17.3 — Apple Pencil Pro features

**Conteúdo:** squeeze → QuickMenu radial; barrel-roll → tool rotate ou stroke twist.

---

### T17.4 — Wacom hover + S Pen + Mouse fallback

**Conteúdo:** PlatformHost extensions per device.

---

### T17.5 — Audit + fechamento W17

---

## §21. W18 — Export interop (SVG / PDF / AI / native FREEZE)

### T18.1 — SVG export (round-trip lossless v1.0 subset)

**Conteúdo:** VectorNetwork → SVG paths + gradients + masks + clip + text-on-path. Round-trip: SVG output → SVG import → bit-identical.

---

### T18.2 — PDF export

**Conteúdo:** subset (paths + gradients + text). Reusa `lopdf` ou write-own minimal.

---

### T18.3 — AI import (lossy via PDF subset)

**Conteúdo:** Adobe Illustrator nativo é PDF wrapped; parse subset via `lopdf`. Log gaps documentados.

---

### T18.4 — `.ph2d-vector` v1 FREEZE com migrator (HR-14)

**Conteúdo:** schema v1 freeze; `migrate_v1_to_v2` stub para futuro.

---

### T18.5 — Audit + fechamento W18

---

## §22. W19 — Animation export

### T19.1 — GIF + APNG export

**Conteúdo:** consome `ph2d-imageio-gif` + `ph2d-imageio-apng` (existing).

---

### T19.2 — MP4 export

**Conteúdo:** via ffmpeg subprocess OR `mp4-rust` crate.

---

### T19.3 — Lottie subset export

**Conteúdo:** subset (paths + transforms + opacity + masks). Roda em After Effects básico.

---

### T19.4 — `.ph2d-vector-anim` schema FREEZE

**Conteúdo:** native animation format; postcard binário versionado.

---

### T19.5 — Audit + fechamento W19

---

## §23. W20 — Final polish + bug bash + v1.0 declaração

### T20.1 — Memory budget audit per platform

---

### T20.2 — Frame budget audit (all 20 waves cross-tested)

---

### T20.3 — A11y review WCAG 2.2 AA

---

### T20.4 — i18n bundles complete (pt-BR + en-US)

---

### T20.5 — Cross-platform smoke (Mac + Linux + iPad pelo menos)

---

### T20.6 — Smoke v1.0 full do Enio

**Conteúdo:** Enio cria ilustração completa em 1 sessão (paths + boolean + procedural fill + animation + symbol + LLM-shape + auto-trace de raster + commit em `.ph2d-vector` v1).

**Critério:** Enio sai satisfeito ✓ → Vector Module v1.0 declarado.

---

## §24. Crates audit — exatamente 32 crates novos (pós-2ª iteração Antigravity 2026-05-28)

**Crítica L1F1 + L3F1 absorvida**: contagem real era 40 (não 30-32). Consolidação real aplicada via 3 merges (`-3 panels Studios → 1`, `-2 utility tools → 1`, `-2 transforms triviais → 1`, `-1 llm+node-llm-shape → 1`) para chegar honestamente em **32 crates**.

| Crate | Wave | Responsabilidade | Pattern |
|-------|------|------------------|---------|
| `ph2d-vector-traits` | W1 T1.1 | Mocks + trait abstrações + `AnimValue` enum | foundation |
| **`ph2d-brush-traits`** | W1 T1.1b | **Contratos desacoplados Brush** (resolve circular dep Painter↔Vector, crítica L6F2 Antigravity) — importável linearly por Painter E Vector | foundation |
| `ph2d-vector-doc` | W1 T1.2 | Data model VectorNetwork + CRDT | foundation |
| `ph2d-vector` (existing, expanded) | W1 T1.3 | Vello pipeline integration | expansão |
| `ph2d-tool-vector-pen` | W1 T1.5 | Pen tool Bézier default + Spiro Assist | tool |
| `ph2d-tool-vector-pencil` | W2 T2.1 | Pencil Hobby fitter | tool |
| `ph2d-tool-vector-shape` | W2 T2.2 | 5 primitives (delega ao node `vector-source` em commit) | tool |
| `ph2d-tool-vector-select` | W2 T2.3 | Marquee + click select | tool |
| `ph2d-tool-vector-direct` | W2 T2.3 | Direct vertex/tangent + Inspector panel | tool |
| **`ph2d-tool-vector-utilities`** | W14 | **Consolida 3 tools** (Knife + Bucket + Eyedropper) em sub-modules; pequenos isolados não-justificam crates separados | tool |
| `ph2d-tool-vector-symbol` | W9 T9.1 | Parametric symbols Cuttle-style | tool |
| `ph2d-tool-vector-text-on-path` | W14 T14.4 | Text on path layout | tool |
| `ph2d-panel-vector-graph` | W3 T3.1 | Geometry Graph editor panel | panel (Coord-B) |
| `ph2d-panel-vector-inspector` | W2 T2.3 | Inspector node/vertex params | panel |
| `ph2d-panel-vector-pathfinder-studio` | W3 | **NEW** Pathfinder Studio (UX layer Illustrator-style sobre Geometry Graph, crítica L5F1) | panel |
| **`ph2d-panel-vector-studios`** | W15 | **Consolida 3 panels Studios** (Tool Studio + Stroke Studio + Fill Studio) em sub-modules; UI patterns shared, fits well combined | panel |
| `ph2d-panel-vector-symbol-lib` | W9 T9.2 | Symbol library browser | panel |
| `ph2d-node-vector-source` | W3 T3.2 | 5 primitives consolidated multi-variant | node |
| `ph2d-node-vector-boolean` | W3 T3.3 | 9 boolean variants + SDF + Linesweeper draft+reconcile | node |
| `ph2d-node-vector-offset` | W3 T3.4 | parallel/contour offset | node |
| `ph2d-node-vector-outline-stroke` | W4 T4.1 | stroke → filled path | node |
| `ph2d-node-vector-roughen` | W4 T4.2 | organic perturbation | node |
| **`ph2d-node-vector-transforms`** | W4 | **Consolida 3 transforms triviais** (Twist + Mirror + Corner-Round) em sub-modules; pure math triviais, fit combined | node |
| `ph2d-node-vector-bend-path` | W4 T4.4 | bend along envelope | node |
| `ph2d-node-vector-pattern-along-path` | W4 / W8 T8.1 | distribute pattern (consome `ph2d-brush-traits`) | node |
| `ph2d-node-vector-scatter` | W4 T4.6 | duplicate + distribute (radial/grid/random/along-path) | node |
| `ph2d-node-vector-width-profile` | W4 T4.7 | variable width 1D axes | node |
| `ph2d-node-vector-hatch` | W4 T4.8 | parametric hatch fill | node |
| `ph2d-node-vector-warp` | W4 T4.11 | perspective/mesh warp/liquify | node |
| `ph2d-node-vector-recolor` | W4 T4.12 | color harmony rules across subgraph | node |
| **`ph2d-node-vector-trim-path`** | W4 | **NEW 18º node** — Trim Path (`trim_start`, `trim_end`, `offset`) — essential motion designers ex-AE/Cavalry/Rive, crítica L5F2 Antigravity | node |
| `ph2d-node-vector-auto-trace` | W12 T12.3 | ML raster→vector (Sketch/Illustration/Basic Shapes) | node |
| `ph2d-node-vector-luau-script` | W4 (opt-in W9) | Custom modifier em Luau | node |
| **`ph2d-vector-llm`** | W13 | **Consolida** crate de tools MCP + node wrapper (`vector-llm-shape`) em single crate (data + node fit combined) | foundation+node |
| `ph2d-vector-fill` | W6 | Procedural fill shader graph + diffusion curve solver | foundation |
| `ph2d-vector-font` | W10 T10.3 | Variable fonts axes (skrifa + glyph → VectorNetwork) | foundation |
| `ph2d-vector-runtime` | W16 T16.1 | Game runtime ship-able + physics colliders + dormant fractures | foundation |

**Total: 32 crates** ✓ (anteriormente 40; consolidação real per Antigravity 2ª iteração).

Padrão de consolidação: crates consolidados (`utilities`, `studios`, `transforms`, `llm`) usam sub-modules isolados (`src/knife.rs`, `src/bucket.rs`, etc.) com workspace.member entry único — preserve drop-crate isolation para multi-agente per DIRETRIZ §3.A enquanto reduz overhead Cargo. **Não é monolitização** (proposta original Antigravity de 5-6 crates rejeitada porque viola DIRETRIZ §3.A drop-crate fan-out + HR-18 god-file).

**Linker optimization (recomendação L1F1 Antigravity)**: `.cargo/config.toml` workspace pin lld (Linux) / mold (Linux opt-in) / ld-prime (macOS) — já configurado em PH2D desde 2026-05-13 (vide memory `feedback_codificacao_rapida`). Build cold time tracked em CI.

**Cargo.toml lock contention policy (Antigravity 3ª iteração L1F2 2026-05-29)**: crates consolidadas (`ph2d-tool-vector-utilities`, `ph2d-panel-vector-studios`, `ph2d-node-vector-transforms`, `ph2d-vector-llm`) têm `Cargo.toml` único compartilhado entre múltiplos sub-modules. Risco: dois agentes paralelos editam Cargo.toml mesma crate em features distintas → conflict no merge. **Policy enforced via `scripts/git-stage-guard.sh`** (PH2D já tem desde 2026-05-15 per memory `feedback_destructive_git_outside_pasta`): edição em `Cargo.toml` de crate consolidada deve serializar via `COORD_OVERRIDE=1` env flag OR PR comments coordenam ordem. Adicionar entrada explícita em `crates/<consolidated>/CARGO_LOCK_POLICY.md` documentando "no concurrent Cargo.toml edits".

---

## §25. Cross-cutting checks — gates de CI ativos por wave

### Gates W0 + persistentes

- `architecture_vector_contract_surface` — caps congelados de VectorOp / Vertex / Segment / Region (T0.12).
- `vector_register_all_alphabetical` — order em `register_all_nodes` / `register_all_tools`.
- `no_hashmap_in_vector_sim` — ADR-0022 enforce no domain `vector` em sim path.

### Gates W1-W3

- `vector_no_alloc_hot_path` (T1.3) — dhat-rs gate, zero alloc em render hot path.
- `vector_crdt_convergence` (T1.6) — multi-site replay determinístico.
- `vector_persist_roundtrip` (T1.2) — serialize → deserialize → hash match.

### Gates W4-W6

- `vector_node_golden_<name>` (per node) — bit-identical cross-OS.
- `procedural_fill_no_recompile_on_animate` (T6.3) — animate 60 frames = 0 recompilations.
- `vector_sdf_real_time` (T5.2) — 50+ paths boolean a 120 FPS.

### Gates W10+

- `variable_font_axis_interpolation_smooth` (T10.3) — no glitches durante axis animation.
- `vector_animation_export_pixel_match` (T19.x) — GIF/APNG/MP4 visual diff < threshold.

### Gates W16

- `vector_runtime_physics_momentum_conservation` (T16.4) — split de collider preserva momento.
- `vector_runtime_lod_frame_budget` (T16.5) — 50+ elements cabe em 3.5 ms.

### Gates W18-W20

- `vector_svg_roundtrip_lossless` (T18.1) — SVG export → import → bit-identical.
- `vector_v1_schema_freeze` (T18.4) — migrator chain v1 → vN.
- `vector_wcag_aa` (T20.3) — WCAG 2.2 AA compliance test.

### Gates novos (Antigravity 3ª iteração 2026-05-29)

- **`vector_fuzz_wgsl_codegen`** (L3F1, T13.5) — `cargo-fuzz` target alimenta `ph2d-vector-fill::wgsl_codegen` com DAGs aleatórios (10k random cases per run, daily CI dedicated). Captures naga panic, unbounded recursion, malformed shader output. Falha em primeira regression.
- **`vector_fuzz_llm_semantic_tokens`** (L3F1, T13.5) — `cargo-fuzz` target alimenta `ph2d-vector-llm::semantic_tokens_parser` com structured JSON malicious payloads (turns=1000000, etc.). Captures OOM, infinite loop, bounded validation correctness.
- **`vector_fuzz_postcard_asset`** (L4F2) — fuzz `.ph2d-vector` postcard deserializer com adversarial byte streams. Captures heap overflow, malformed SmallVec lengths.
- **`vector_criterion_perf_regression`** (L3F2) — `criterion` benchmarks per crate; CI gate compara contra `main` baseline; >5% regression em hot path = falha. Suite: render frame, boolean ops, scatter, hobby fit, CRDT apply, asset load.
- **`vector_a11y_functional_traversal`** (L3F3) — não apenas presence de AccessKit nodes; simula screen reader passa por canvas + Graph panel + Inspector + timeline; valida readout coherent. Uses test harness around `accesskit-tester` crate.
- **`vector_linux_multiarch_determinism`** (L2F2) — CPU fallback bit-identical em Linux x86_64 (AVX2 + non-AVX) + aarch64 NEON.
- **`vector_mobile_core_asset_compat`** (L8F2) — editor cooker recusa assets com unavailable features marcados para Mobile Core tier sem pre-render atlas.
- **`vector_metal_overlay_no_flicker`** (L1F5) — visual test em iPad simulator + Mac M1; 60 sec stylus stroke; zero flickering frame detected.

---

## Fim do plano

20 waves × ~5-12 tasks cada = **~102 tasks T-W.N total** ratificadas e prontas para execução (medido via `grep -c "^### T"` em 2026-05-28; corrigido de ~140 estimativa inicial). Audit adversarial obrigatório por wave (≥2 lentes paralelas) com **critérios quantificáveis** (§0.8). **13 ADRs (0056..0068)** pré-aprovados em W0 destrancam todo o plano:
- 11 originais (0056..0066) + 2 absorvidos Antigravity 2ª iteração: ADR-0067 (`ph2d-brush-traits` desacoplamento) + ADR-0068 (DeviceTier `Mobile Core` Vector Module).

**Smoke W1 ready:** clica Pen tool (Bézier cúbico default) → 3 pontos no canvas → triângulo fechado renderizado via Vello GPU prefix-sum pipeline. Day ~7.
