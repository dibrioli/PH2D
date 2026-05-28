# 14 — Inovações extraordinárias (superando o Illustrator)

> Doc dedicado às **7 inovações extraordinárias** + **5 críticas técnicas (A-E)** absorvidas após análise do feedback Antigravity / Google DeepMind ([`avaliacao_e_melhorias.md`](avaliacao_e_melhorias.md)). Mandato Enio 2026-05-27: **padrão-ouro absoluto, sem medo, sem economias**.

## 14.0 Princípio operacional

O Vector Module PH2D não é "clone multiplataforma do Illustrator" — é o **sucessor definitivo** da arte vetorial integrada à game engine. Cada inovação aqui foi escolhida porque:

1. **Nenhuma ferramenta mainstream entrega hoje** (ou tem versão fundamentalmente inferior — Illustrator boolean bake-and-discard; Figma boolean ainda bake; Rive runtime sem procedural modifiers; Affinity sem node graph; Cavalry sem runtime de jogo).
2. **É viável** dentro da infraestrutura Rust + wgpu + Vello + compute shaders do PH2D.
3. **Cabe nos princípios** PH2D (HR-1..HR-18, multi-plataforma desde W1, LLM-first, padrão ouro).
4. **Tem integração total prevista** com Painter (raster ↔ vector), motion nodes (Cavalry-style), shader nodes futuros, animation system, Luau gameplay, MCP, runtime de jogo com physics.

Esta página é a fonte de verdade técnica para **Coordenadores escrevendo ADRs 0056..0066** e **Implementadores construindo as waves específicas**.

## 14.1 Mapa rápido (referência)

| # | Inovação / Crítica | Origem | Wave | ADR | Veredicto |
|---|--------------------|--------|------|-----|-----------|
| **§14.2 (P1)** | **Live Boolean Graph** (Linesweeper + node graph + cache by hash) | Spec original PH2D | W3-W4 | ADR-0058 + ADR-0059 | Aceito integral (toda op destrutiva do Illustrator vira nó vivo) |
| **§14.3 (P2)** | **Mesh gradient via Diffusion Curve** (Poisson PDE, Unified Smooth Vector Graphics 2024) | Spec original PH2D | W7 | ADR-0060 | Aceito integral (substitui mesh-patches hand-author) |
| **§14.4 (P3)** | **Painter ↔ Vector Bridge bidirecional** (paint-into-vector + brush-look + auto-trace ML) | Spec original PH2D | W12 | ADR-0062 | Aceito integral (sucessor unificado Procreate + Illustrator) |
| **§14.5 (P4)** | **LLM-as-graph-node** (LLM4SVG semantic tokens, output editável) | Spec original PH2D | W13 | ADR-0061 | Aceito integral (primeira ferramenta onde IA = ajuda, não substitui) |
| **§14.6 (P5)** | **Vector Runtime + Dynamic Physics Colliders** | Spec PH2D + Proposta 4 Antigravity | W16 | ADR-0063 | Aceito integral (sword-cut → 2 corpos rígidos com momento preservado) |
| **§14.7 (P6) ✨** | **Tipografia Generativa via Variable Fonts axes** | Proposta 3 Antigravity | W10 | ADR-0066 | Aceito integral (glifo = vector network nativo) |
| **§14.8 (P7) ✨** | **Vector-SDF Hybrid GPU Pipeline** | Proposta 1 Antigravity | W5 | ADR-0065 | Aceito integral (boolean 120 FPS via min/max compute) |
| **§14.9 (A-E)** | **5 críticas técnicas absorvidas** | Antigravity 2026-05-27 | W1+ | múltiplos | Crate consolidation seletiva + UBO + draft+reconcile + Bézier default + traits/mocks |

---

## 14.2 Inovação 1 — Live Boolean Graph

### 14.2.1 Origem da decisão

O spec original W0 já listou em §8.1 do README. Antigravity adicionou crítica C (sync vs async hot-path) que reformulou a implementação.

### 14.2.2 Por que muda o jogo

- **Illustrator Pathfinder = bake-and-discard.** Click Unite → operandos perdidos; mudou de ideia? Refaz do zero.
- **Affinity Compound Shapes = boolean-only não-destructive.** Sem offset, sem outline, sem distort no graph.
- **Figma Boolean = ainda bake.** Suaviza algumas operações via "Boolean Group" mas não é editável downstream.
- **Vector Module:** **TODA operação geométrica é um nó vivo** no `ph2d-nodegraph` domain `vector`. Boolean, offset, outline, roughen, twist, bend, scatter, mirror, corner-round, warp, recolor — todos com operandos preservados, sliders editáveis para sempre, animáveis em curve, replayáveis em runtime determinístico.

### 14.2.3 Escopo nas waves

#### W3 (T3.3) — Boolean foundation

- `vector-boolean` node com 9 variants: union / subtract / intersect / exclude / divide / trim / merge / crop / outline.
- Pipeline draft+reconcile (resolve crítica C):
  1. **Draft naive CPU** (≤ 1 ms) — Bézier-cúbico clipping aproximado, hot-path stylus.
  2. **SDF Hybrid GPU** (≤ 0.5 ms compute pass) — real-time slider drag + gameplay morphing.
  3. **Linesweeper exato async** (background worker debounced) — topology canônica em commit.
- Cache by hash do graph input.

#### W4 (T4.1..T4.12) — 12 nodes restantes em fan-out paralelo

- Cada node em crate próprio (`ph2d-node-vector-*`).
- Implementadores em paralelo via `scripts/slot-env.sh` isolation.

#### W5 (T5.2) — SDF Hybrid ativo full

- Habilita preview em tempo real para 50+ paths boolean simultaneamente.
- Linesweeper exato continua canônico em commit.

### 14.2.4 Custo

- **Linesweeper** = beta no momento; risco mitigação via fallback Clipper em emergência.
- **Cache strategy** crítico: hash do graph input + invalidation on edit. Memória ~30-50 MB.
- **Frame budget**: SDF GPU ≤ 0.5 ms / pass; Linesweeper async em worker dedicado.

### 14.2.5 Diferencial competitivo

| Ferramenta | Boolean é vivo? | Procedural ops vivos? | GPU draft? | Async exact? |
|------------|------------------|------------------------|------------|--------------|
| Illustrator | ❌ bake | ❌ flat Effects panel | ❌ | ❌ |
| Affinity | ✓ compound shapes only | ❌ | ❌ | ❌ |
| Figma | ❌ bake | ❌ | ❌ | ❌ |
| Inkscape | ✓ LPE boolean | ✓ LPE stack (perf falha >5) | ❌ | ❌ |
| **PH2D Vector** | **✓ full** | **✓ all 17 nodes** | **✓ SDF Hybrid** | **✓ Linesweeper async** |

---

## 14.3 Inovação 2 — Mesh gradient via Diffusion Curve (Poisson PDE)

### 14.3.1 Origem da decisão

Spec original PH2D §8.2; pesquisa identificou paper [Unified Smooth Vector Graphics: Modeling Gradient Meshes and Curve-based Approaches Jointly as Poisson Problem (arXiv 2408.09211, 2024)](https://arxiv.org/pdf/2408.09211) que **mostra mesh gradients e diffusion curves como duas formas da mesma PDE**.

### 14.3.2 Por que muda o jogo

- **Illustrator mesh gradient** = hand-author de mesh patches (penoso); export rasteriza (não preserva vector).
- **Diffusion curves** = autor desenha **curva** com cor nos dois lados (opcionalmente + blur); GPU diffunde Poisson no resto do canvas. Poucos toques produzem photoreal smooth shading.
- Em 2008 (Orzan SIGGRAPH original) o solver era CPU-bound → off-line apenas. Em 2026 hardware torna real-time tractable via Walk-on-Spheres Monte Carlo ou multigrid compute pass.

### 14.3.3 Escopo nas waves

#### W7 (T7.1) — Poisson PDE solver compute pass

- Shader `crates/ph2d-vector-fill/shaders/diffusion.wgsl`.
- **Walk-on-Spheres Monte Carlo** (estocastic, embarrassingly parallel — ideal para GPU) **OR** **multigrid** (iterativo, melhor convergence).
- Adaptive iteration count com convergência threshold (`max_residual < 1e-3`).
- Boundary curva com cor `(rgba_left, rgba_right, blur_radius)` em ambos lados.

#### W7 (T7.2) — UI para autor diffusion curve

- Diffusion Curve tool — desenha curva (Spiro / cubic), click side esquerdo/direito para set cor, slider blur.
- Reusa color picker do Painter.

### 14.3.4 Custo

- **Perf** alvo: < 5 ms / canvas 1080p (com 5 curves). Devices entry-level podem precisar fallback para `pre-bake offline → texture sample`.
- **Memória**: solver intermediate state ~20 MB para canvas 1080p.
- **Determinismo**: Monte Carlo é estocastic; det-mode opt-in usa fixed seed + ordered reductions (mais lento, ~3-5×).

### 14.3.5 Diferencial competitivo

Nenhum competitor mainstream entrega mesh gradient via diffusion curve em runtime GPU. Único próximo é trabalho de pesquisa ([ETH ray-traced diffusion curves 2013](https://igl.ethz.ch/projects/diffusion-curves/)), nunca commercializada.

---

## 14.4 Inovação 3 — Painter ↔ Vector Bridge bidirecional

### 14.4.1 Origem da decisão

Spec original PH2D §8.3. PH2D já tem Painter em desenvolvimento (W0 fechado 2026-05-26 com 11 ADRs Accepted; W1.T1.5 em curso). Vector Module é o irmão vetorial natural.

### 14.4.2 Por que muda o jogo

- **Procreate = raster only**, vector "support" é import + rasterize.
- **Illustrator = vector only**, raster é objeto opaco.
- **Photoshop = vector pobre** (paths sim, mas sem live boolean / sem procedural).
- **Vector Module + Painter unidos** = sucessor unificado de Procreate + Illustrator com transição zero-fricção.

### 14.4.3 Escopo nas waves — três bridges

#### Bridge 1: Paint-into-vector (W12 T12.1)

- Usuário pinta com Painter brush dentro do canvas Vector Module.
- Cada Painter stroke → Hobby fitter → `vector.pencil` path automaticamente.
- Pressure → `width-profile.pressure` axis; tilt → asymmetric envelope.
- Resultado: vetor editável com look pintado.

#### Bridge 2: Vector com look de brush (W8 T8.1 / W12 T12.2)

- `vector-pattern-along-path` consome qualquer brush do `ph2d-painter-brush` library.
- Distribui stamps (Painter brush stamps) ao longo do vector path com spacing / jitter / scatter params.
- Vector path traçado parece pintado a mão, sem perder editability (mover vertex → re-renderiza brush stamps automaticamente).

#### Bridge 3: Vectorize raster (W12 T12.3)

- Comando "Vectorize layer" no Painter chama node `vector-auto-trace`.
- 3 modos (Linearity Curve pattern):
  - **Sketch**: line detection (Sobel + Canny + path tracing).
  - **Illustration**: color region quantize + Potrace-style boundary extraction.
  - **Basic Shapes**: ML primitive fit (rect / ellipse / poly detection).
- Backbone ML opcional: SuperSVG / LLM4SVG (embed se necessário) ou Potrace fallback.

### 14.4.4 Custo

- Bridge 1: leve (Hobby fitter já existe em pesquisa; ~300 LOC inline).
- Bridge 2: leve (já feito em W8 T8.1 como parte do `pattern-along-path` node).
- Bridge 3: pesado se embed ML (modelo SuperSVG ~50 MB; LLM4SVG via API call). Potrace fallback CPU-only (~500 LOC port).

### 14.4.5 Diferencial competitivo

Nenhuma ferramenta entrega os 3 bridges simultaneamente. Linearity Curve tem só Bridge 3 (auto-trace). Procreate tem zero. Affinity Designer tem dual persona mas sem bridge nativo brush ↔ vector.

---

## 14.5 Inovação 4 — LLM-as-graph-node

### 14.5.1 Origem da decisão

Spec original PH2D §8.4. PH2D tem MCP server skeleton (M9) + Luau (M7) — destranca LLM como first-class user (HR-10 + HR-11).

### 14.5.2 Por que muda o jogo

- **"Generate SVG with AI" hoje = colar grande blob de SVG estranho, não editável.**
- **Inkscape AI SVG Generator (2026)** ainda emite grupos opacos.
- **Vector Module:** node `vector-llm-shape(prompt, constraints, style_ref) → VectorNetwork`. LLM emite **semantic tokens estruturados** (LLM4SVG pattern — vide §15 referências) que o parser converte para Vector Network nativo. Resultado **100% editável downstream** (slider de roughness no output do LLM funciona!). Re-promptable.

### 14.5.3 Escopo nas waves

#### W13 (T13.1) — `crates/ph2d-vector-llm/` skeleton

- MCP tools:
  - `vector_paint_shape(prompt, constraints, style_ref) → VectorNetwork` (mutative).
  - `vector_modify_shape(shape_ref, mod_prompt) → VectorNetwork` (mutative).
  - `vector_query_shape(shape_ref) → ShapeMetadata` (read-only).
  - `vector_inspect_shape(shape_ref) → semantic_tokens` (read-only).
- Semantic tokens parser (LLM4SVG-style structured output).
- HR-11 governance (`vector_delete_path` destructive → confirmation token).

#### W13 (T13.2) — Node `vector-llm-shape`

- Node graph wrapper para `ph2d-vector-llm`.
- Params: `prompt` (String), `seed` (u64), `style_ref` (opcional reference image).
- Output: VectorNetwork plugável downstream.

### 14.5.4 Custo

- Modelo LLM externo (Claude / GPT / Gemini via MCP) — sem ship local model.
- Latência: prompt → tokens → parse → VectorNetwork ~2-10 segundos (depende do LLM). Async com spinner UI.
- Governance audit log em JSONL.

### 14.5.5 Diferencial competitivo

- **Inkscape AI SVG Generator** (2026 plugin): output não editável estruturado.
- **StarVector / SuperSVG**: pesquisa, sem product UX integrado.
- **Vector Module**: primeiro tool onde LLM = node-graph node fluido, editable downstream, integrado a MCP governance (HR-11).

---

## 14.6 Inovação 5 — Vector Runtime + Dynamic Physics Colliders

### 14.6.1 Origem da decisão

Spec original PH2D §8.5 (runtime determinístico). **Proposta 4 Antigravity adicionou physics colliders** — lacuna que o spec original tinha (Lente 2 do próprio prompt de crítica do autor previu: "Vector-as-input-to-physics — PH2D tem rapier; falta?").

### 14.6.2 Por que muda o jogo

- **Rive prova o mercado** (Spotify, Duolingo, Disney shipam) mas:
  - Data model path-only (não vector network).
  - Sem procedural modifiers / falloffs / generators (Cavalry-class).
  - Sem shader fills procedurais.
  - Sem determinismo cross-platform garantido.
  - **Sem integração física**.
- **Vector Module Runtime:** Rive-class **mais** Cavalry-class **mais** Houdini-determinism opt-in **mais** Rapier 2D physics integration.

### 14.6.3 Escopo nas waves

#### W16 (T16.1) — `ph2d-vector-runtime` crate

- Subset runnable em release de jogo (sem editor, sem Studio panels).
- `.ph2d-vector` asset loader (postcard parse + cache).
- State machine model (Rive-inspired): states / transitions / blending.
- Bones + vertex weighting (Rive-class skeletal deformation).
- Mesh deformation hybrid (path moves + raster UV warp).
- ECS integration via `EditorAction::ActivateState("hover")`.
- Luau bridge: `ph2d.vector.state.set("press")`.
- Opt-in determinism (HR-5 + ADR-0021 SimWorld): boolean ops em sim tick = fixed-point + ordered reductions. Replay determinístico cross-platform (Linux/Mac/Win) testado em CI.

#### W16 (T16.3-T16.4) — Rapier 2D collider gen + Dynamic Split (**Proposta 4 Antigravity**)

- VectorNetwork → Rapier `Collider`. Decomp convex via earcut OR direct `SharedShape::convex_hull` por region.
- Mass = `area_region × material.density`.
- Joints opcionais entre regions (cloth-like, breakable).
- **Dynamic split em runtime boolean cut**: pipeline (a) SDF GPU silhueta imediata, (b) Linesweeper async topology exata, (c) `Vector → Rapier collider re-decomp` divide corpo rígido em N corpos independentes, (d) momento linear + angular preservado por cada split.

#### W16 (T16.5) — LOD vetorial dinâmico (**Proposta 2 Antigravity**)

- Runtime aplica curve-aware adaptive fit pré-Vello sparse-strips.
- Threshold de detail driven pela câmera (distância world-space + cobertura em pixels da bbox).
- Per-asset override (heroi sempre full detail, props distantes simplificam).
- Mantém frame budget 3.5 ms mesmo com 50+ elementos vetoriais em tela.

### 14.6.4 Custo

- Runtime crate isolado é leve (~5k LOC estimado).
- Rapier integration: collider gen + re-decomp em runtime é O(N segments) — viable para shapes < 200 segments.
- LOD adaptive fit: Bézier-aware (Levien flatten + RDP) ~50-200 µs / shape.
- Memory: 200 MB VRAM + 100 MB RAM padrão desktop / 80 + 30 mobile / 40 + 20 web (HR-13).

### 14.6.5 Diferencial competitivo

| Ferramenta | Runtime ship? | Procedural modifiers? | Shader fills? | Determinism? | Physics integration? |
|------------|----------------|------------------------|---------------|--------------|----------------------|
| Rive | ✓ (Unity/Unreal/Bevy) | ❌ | ❌ | weak | ❌ |
| Lottie | ✓ (everywhere) | ❌ | ❌ | weak | ❌ |
| Spine | ✓ (skeletal only) | ❌ | ❌ | weak | ❌ |
| **PH2D Vector Runtime** | **✓** | **✓ 17 nodes** | **✓ procedural shader graph** | **✓ opt-in HR-5** | **✓ Rapier 2D dynamic split** |

---

## 14.7 ✨ Inovação 6 — Tipografia Generativa via Variable Fonts axes (NEW — Proposta 3 Antigravity)

### 14.7.1 Origem da decisão

Proposta 3 Antigravity. Spec original já mencionou en passant (§3.4 text-on-path + Diff Variable Fonts paper) — Antigravity expandiu a ambição ao tratar glifo como vector network nativo.

### 14.7.2 Por que muda o jogo

- **Motion graphics tipográfica hoje** = After Effects deforma rasterizando (perde editability).
- **Vector graphics tipográfica hoje** = Illustrator não anima axes de variable fonts; expansion → outlines bake.
- **Game engines** = texto-em-imagem ou bitmap font.
- **Vector Module:** **glifo individual = vector network nativo**. Eixos OTF de variable font (`weight` / `width` / `slant` / `optical-size` / `GRAD` / qualquer axis custom) expostos como **parâmetros dinâmicos do graph**, animáveis em curve, atualizáveis por motion fields ou Luau scripts.

### 14.7.3 Escopo nas waves

#### W10 (T10.3) — Variable Fonts axes integration

- Novo crate `ph2d-vector-font` consome `skrifa` (font parsing canon Linebender).
- Glifo → VectorNetwork (cada contour → region; tangentes preservadas via Levien Béz fitting).
- Trait `VariableFontAxis { name, min, max, default, current }` expõe axes como graph input/output.
- 4 axes default suportados: weight / width / slant / optical-size. Custom axes via OTF feature table.
- Render path: skrifa parse → kurbo BezPath per glyph → Vello rasterize. **Sem rasterizar a fonte intermediária**.

### 14.7.4 Exemplos concretos

- Logo do jogo deforma weight a cada batida da música: motion node `motion-wave` → `variable-font.weight`.
- HUD do gameplay: número de munição fica mais grosso (weight ↑) e mais largo (width ↑) conforme aproxima do max → bound via Luau script.
- Letterform morphs por proximidade do mouse (falloff radial driving `slant` axis em real-time).

### 14.7.5 Custo

- `skrifa` crate (Linebender) já maduro.
- Glyph → VectorNetwork conversion: ~100 µs / glyph (cache).
- Animation hook via UBO (vide §14.9 crítica B): zero recompile on axis change.

### 14.7.6 Diferencial competitivo

- **After Effects + variable fonts plugins** = workflow externo, lossy export.
- **Adobe Illustrator + variable fonts** = nenhuma animação de axes nativa.
- **Vector Module**: primeira ferramenta vetorial onde **tipografia É vetor animável** — não substituto rasterizado.

---

## 14.8 ✨ Inovação 7 — Vector-SDF Hybrid GPU Pipeline (NEW — Proposta 1 Antigravity)

### 14.8.1 Origem da decisão

Proposta 1 Antigravity. Resolve **dois problemas simultâneos**:
1. Latência síncrona Linesweeper no hot-path (crítica C).
2. Gameplay morphing 120 FPS impraticável com Linesweeper exato.

### 14.8.2 Por que muda o jogo

- **Linesweeper exato** = robusto (catches degenerate cases que Clipper falha) mas pesado (100+ segments em < 1 ms na CPU móvel? não vai).
- **SDF 2D em compute shader** = boolean ops triviais: `min(d1, d2)` união, `max(d1, -d2)` corte, `max(d1, d2)` intersect, `abs(d) - r` arredondamento. Custo constante O(pixels), não O(segments × segments).
- **Combo**: SDF Hybrid para preview real-time + Linesweeper exato no commit.

### 14.8.3 Escopo nas waves

#### W3 (T3.3) — SDF Hybrid draft preview

- Pipeline 3 modos (vide §14.2.3 boolean foundation).
- Shader `crates/ph2d-vector/shaders/boolean_sdf.wgsl`.
- VectorNetwork → SDF 2D rasterization (compute pass).
- Boolean compute via `min/max` (≤ 0.3 ms).

#### W5 (T5.2) — SDF Hybrid full ativo

- 50+ paths boolean simultaneamente a 120 FPS.
- Modo ativo durante edição interativa + gameplay morphing.
- Linesweeper exato continua no commit (mouse-up / pencil-lift / após N ms inatividade).

#### W16 (T16.4) — SDF runtime gameplay morphing

- Espada corta tábua: SDF silhueta immediate + Linesweeper async para split de collider real.
- 120 FPS estável em frame budget.

### 14.8.4 Custo

- SDF resolution default 2× canvas DPI → memory ~ 4× canvas area.
- Compute shader cross-platform: WebGPU minimal (Vello requires compute anyway).
- Fallback graceful: compute unavailable → Linesweeper síncrono com warning UI.
- Limites documentados: SDF produz silhueta, **não preserva topology editável** downstream — Linesweeper continua único caminho canônico.

### 14.8.5 Determinismo

- Default `deterministic: false` (FMA + ordering em SDF compute).
- Opt-in `deterministic: true` (ADR-0065): fixed SDF resolution + ordered reductions + FMA off. ~3-5× mais lento mas bit-identical cross-platform.

### 14.8.6 Diferencial competitivo

Nenhuma ferramenta vetorial mainstream entrega SDF boolean compute pass. Único próximo é uso de SDF em renderers de jogos (Valve's distance field text generalizado) mas não para boolean ops em vector graph.

---

## 14.9 Críticas técnicas absorvidas (A-E Antigravity)

### 14.9.1 Crítica A — Crate bloat (~40 crates) → consolidar

#### Análise Antigravity

Spec original W0 propôs ~40 crates novos. Antigravity alertou: "overhead intolerável Cargo, build time CI sofrerá severamente". Proposta original: colapsar pra 2 crates monolíticos (`crates/ph2d-tool-vector/` + `crates/ph2d-node-vector/`).

#### Decisão final

**Aceito parcial. Solução proposta é arquiteturalmente regressiva:**
- ❌ Viola DIRETRIZ §3.A — fan-out drop-crate é a coluna do multi-agente (paraleliza Implementadores).
- ❌ Viola HR-18 — monolítico vira god-file.
- ❌ Ignora precedente Painter (~10 crates próprios funcionando).
- ❌ PH2D já tem 60+ crates com build OK.

**Contra-proposta:**
- **Manter** drop-crate fan-out para tools e nodes não-triviais.
- **Consolidar seletivamente** primitives triviais: 5 source shapes (rect / ellipse / polygon / star / spiral) em 1 crate multi-variant `ph2d-node-vector-source`. Reduz fan-out de ~40 para ~30-32.
- Aplica regra: **drop-crate quando o trabalho exige Implementador dedicado**; **consolidar quando o trabalho é mecânica/manifesto-like trivial**.

### 14.9.2 Crítica B — Compile stutter em shaders procedurais

#### Análise Antigravity

Compilar WGSL via naga + criar `wgpu::ComputePipeline` por frame = 10-100 ms stall, **quebra HR-4 a 120 Hz**. Animar param escalar em curva 60 Hz = N shaders por segundo.

#### Decisão final — **Aceito integral**

**Pipeline canônico (ADR-0060):**
1. **Topologia do shader graph** (que nodes presentes + conexões) → hash + compile WGSL **uma vez por (topology hash, target backend)** → cacheado em memória de longo prazo + on-disk (`~/.cache/ph2d/shaders/<hash>.{wgsl,spv,msl}`).
2. **Parâmetros escalares animáveis** (cor, frequência de noise, posição de ramp, time, vetor coord) → empacotados em `UniformBuffer` (UBO) atualizado por frame com zero alloc (HR-3).
3. **Topology change** (usuário pluga node novo no graph) → spinner "compiling shader" no HUD, compila off-thread em background, swap atômico ao terminar. Durante compile mostra resultado do template anterior.
4. **Variable Font axes** (§14.7): axes como floats no UBO; mudança de axis = UBO update, não recompile.

**Gate CI**: `procedural_fill_no_recompile_on_animate` — animate 60 frames de param escalar = 0 recompilations.

### 14.9.3 Crítica C — Linesweeper síncrono no hot-path ProMotion sub-9 ms

#### Análise Antigravity

Varredura geométrica em 100+ segments **não** completa em sub-ms na CPU móvel/tablet. Síncrono no input thread = engasgo imediato + perda de latência visual.

#### Decisão final — **Aceito integral**

**Pipeline boolean draft+reconcile (ADR-0059, vide §14.2 + §14.8):**
1. **Draft preview hot-path** (≤ 1 ms): boolean naive CPU sobre subset reduzido OR Bézier-cúbico clipping aproximado.
2. **SDF hybrid GPU** (≤ 0.5 ms compute pass): real-time slider drag + gameplay morphing.
3. **Linesweeper exato async**: background worker debounced, chamado em commit do stroke (mouse-up / pencil-lift) ou após N ms inatividade. Resultado canônico que vira topology editável.

UI mostra indicador discreto "boolean em commit…" quando worker está computando.

### 14.9.4 Crítica D — Rejeição profissional ao Pen Tool Spiro

#### Análise Antigravity

Spec original W0 adotou Spiro/hyperbezier como **default** da Pen Tool. Antigravity alertou: "designers profissionais têm décadas de muscle memory em tangentes Bézier cúbicas; forçar Spiro causará rejeição imediata."

#### Decisão final — **Aceito com ajuste**

**Bézier cúbico = representação default visível** (paridade Illustrator, zero fricção):
- Click adiciona vertex; click+drag estica tangentes cúbicas; close-path.
- Default ao abrir Pen tool.

**Spiro / Hyperbezier como Assist Modes opt-in** (toggle HUD `S` / `H`):
- Útil para letterforms, jewelry-style shapes, organic curves.
- Data model interno é dual-representation (cubic + spiro stored when relevant).
- Export para `.ph2d-vector` preserva ambos; export SVG cooka para cubic (Levien Béz fitting).

**Hobby fitter no Pencil tool** continua canônico (sem alternativa superior).

### 14.9.5 Crítica E — Vaporware coupling (Shader Graph / Animation System)

#### Análise Antigravity

Vector Module assume acoplamento direto com Shader Graph e Animation System — sistemas que ainda não existem no PH2D. Risco: W1-W5 bloqueado por outra ponta não amadurecer.

#### Decisão final — **Aceito integral**

**Traits abstratas + mocks no crate foundational `ph2d-vector-traits`** (W1 T1.1):
- `AttributeEvaluator { fn sample(&self, t: f32) -> f32 }` — animation curve interp (mock = linear).
- `ProceduralFillShader { fn compile(&self) -> WgslSource }` — shader graph mock (= solid fill básico).
- `AnimationCurveSampler { fn at(&self, t: f32) -> AnimValue }` — timeline mock.

**W1-W5 usam mocks** → Vector Module testável fim-a-fim antes que Shader Graph / Animation System reais amadureçam. **W6+ / W10+ substitui mocks** via trait-object swap.

PH2D precedent: `ph2d-script::ScriptHost` foi definido antes do Luau real wire (M7).

---

## 14.10 Riscos cross-inovações

### 14.10.1 Compute pressure budget

7 inovações com compute shaders concorrentes em frame budget 3.5 ms:
- Vello stroke expansion: ~0.5 ms.
- SDF Hybrid boolean: ~0.5 ms (W5 ativo).
- Diffusion curve Poisson: ~5 ms / canvas 1080p (W7) — off-budget? Pode rodar off-thread com cache by hash.
- Procedural fill shader: variável (Noise leve ~0.1 ms; Voronoi pesada ~1 ms).
- LOD adaptive fit (runtime W16): ~0.2 ms / shape.

**Mitigação:**
- Gate CI `vector_compute_budget_aggregate` mede pressão agregada em scenario worst-case.
- Diffusion curve sai do hot-path em W7+ (resultado cacheado por hash).
- LOD threshold ajustável por device tier (`DeviceTier` da ADR-0053 Painter).

### 14.10.2 Determinismo cascading

Múltiplas inovações com determinismo opt-in (Linesweeper, SDF, CRDT, fluid sim runtime).

**Mitigação:**
- Gate CI `vector_determinism_cross_os_replay` rodando fixture com todas opt-ins ativas.
- ADRs documentam ordering of reductions explicitly per inovação.
- Memory `feedback-no-industrial-claims-without-verification` reforça: nenhum claim "determinístico" sem teste cross-OS funcionando.

### 14.10.3 LLM dependency externa

LLM-as-graph-node (W13) depende de modelo externo via MCP.

**Mitigação:**
- Fallback graceful: LLM offline → node mostra "LLM unavailable; cached result" usando última output válida.
- Audit log de prompts para reprodução.
- Embed model opcional (W13.x stretch goal — SuperSVG ~50 MB embed para offline auto-trace).

### 14.10.4 Cross-platform shader bit-identity

WGSL compila para SPIR-V (Vulkan / Windows D3D12) / MSL (Metal) / HLSL — runtime escolhe backend.

**Risco:** mesma WGSL produz output ligeiramente diferente cross-backend (rounding, FMA, ordering).

**Mitigação:**
- Gate CI `vector_shader_cross_backend_diff_threshold` mede SSIM ≥ 0.999 entre Linux / Mac / Win.
- Det-mode opt-in: forces no-FMA shader prelude.

---

## Fim das 7 inovações + 5 críticas

**Síntese:** Vector Module se posiciona como **sucessor definitivo do Illustrator** ao oferecer simultaneamente:
- Vector network topology (sucessor Figma).
- Live node graph com 17 nodes (sucessor Cavalry).
- Runtime de jogo determinístico + physics (sucessor Rive, superior).
- Mesh gradient via diffusion curve (substitui hand-author).
- LLM authoring editável (primeiro do mundo).
- Painter ↔ Vector bridge bidirecional (sucessor Procreate + Illustrator unificados).
- Tipografia generativa via variable fonts axes (primeiro do mundo).
- SDF Hybrid GPU pipeline (boolean 120 FPS).

**Mandato Enio**: padrão-ouro absoluto, tempo é custo aceito, **superar** Illustrator e Rive simultaneamente, **integrar-se** totalmente à game engine.
