---
name: vector-module-projeto
description: Especificação do **Vector Module** do PH2D — ferramenta de arte vetorial sucessora ambiciosa do Illustrator, totalmente integrada à game engine (animação, shaders, motion nodes, Luau, Painter, runtime/gameplay + physics colliders dinâmicos + dormant fractures + security hardening + cross-platform parity). **W0 RATIFICADA 2026-05-29 — 13 ADRs Accepted + amendments policy ativa.** W1 ABERTA.
status: **ACCEPTED — W0 RATIFIED 2026-05-29; W1 OPEN** (3 iterações Antigravity absorvidas; CONVERGENCE ~9.7/10; ENDORSEMENT 9.8/10)
data: 2026-05-29 (v4 — ratificada)
decisor: Enio (aprovação explícita 2026-05-29)
---

# Vector Module — Arte vetorial node-native do PH2D

> **Visão em uma frase:** uma ferramenta de **arte vetorial GPU-resident, node-native, com runtime de jogo first-class** — onde toda operação que o Illustrator destrói (boolean, offset, outline, contour, distort, scatter) é um **nó vivo, animável, scriptável, replayável em runtime**, com edição via stylus em iPad/Wacom e desktop a **≤9 ms** de latência, autoria assistida por LLM emitindo strokes editáveis (não SVG bake), e integração nativa com Painter (raster ↔ vector), motion nodes (Cavalry-style), shader nodes (Blender-style) e Luau/MCP.

---

## 0. Localização canônica e status

- **Doc canônico vivo:** este diretório (`docs/Vector Module/`). Spec **CONGELADA via 13 ADRs (0056..0068) Accepted em 2026-05-29** + amendments policy ativa para evolução pós-ratificação.
- **Estado em 2026-05-29 (v4 — RATIFICADA):** **W0 RATIFICADA por Enio em 2026-05-29 após 3 iterações Antigravity (Google DeepMind) absorvidas integralmente. W1 ABERTA.** Pesquisa de estado-da-arte feita; spec arquitetural delineada; **5 + 6 + 19 findings absorvidos** (vide §11.B + §11.C + §11.D); inovações expandidas de **5 → 8** (acrescidas §8.6 Tipografia Generativa + §8.7 Vector-SDF Hybrid GPU + §8.8 Dormant Fracture Edges); ADRs 9 → 13 + amendments policy ativa. **CRITICAL fixes ao longo das 3 iterações**: `AttributeEvaluator` retorna `AnimValue` typed enum + `t: f64`; sparse strips corrigido (Vello CPU); crate count 40 → 32 reais; `ph2d-brush-traits` quebra circular dep; `vector-trim-path` 18º node; Mobile Core tier <12 MB rival Rive; **shell iPad T0.14 predecessor task** (cross-platform desbloqueado pre-W1); **Metal Direct Overlay PlatformHost extension formal** (Modo B sub-9ms); **security sanitizers** LLM token injection + postcard bounds; **wgpu DeviceLost recovery** + emergency edit_log save; **CRDT timestamp validation + periodic integrity check**; **Vello encapsulation single-crate** (long-tail upgrade cost); **JBU 2-pass upsample**; **dynamic concave convex-hull fallback Tier 1**; **Reduced Motion runtime filter**; **Geometry Graph keyboard nav completo**; **fuzz testing T13.5** + criterion regression + a11y functional gates. **CONVERGENCE INDEX 9.2 → ~9.7 estimado** (Painter ratificou em 9.0); **ENDORSEMENT 9.8** (Antigravity 3ª iter). Pronto para ratificação Enio + abertura W1.
- **Família arquitetural:** subsistema novo — **NÃO é Image Tool** (não é one-shot raster como BgRemoval/Padding); **NÃO é só um Tool** (é um conjunto de tools + painéis + nodes + runtime + bridges). Família simétrica ao Painter (vide [`docs/Painter_projeto/`](../Painter_projeto/)): o Painter é o pilar **raster**, o Vector Module é o pilar **vetorial**. Ambos compartilham infra (input, color, layers, animation), mas têm dados e renderer próprios.
- **Mandato:** **padrão-ouro absoluto** ([feedback-perfection-no-deferrals](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md)). Tempo maior aceito como custo de superar Illustrator. Prior: [HANDOFF_node_system §0](../HANDOFF_node_system.md) + Painter W0.
- **Mandato adicional:** **integração total prevista, mesmo onde a outra ponta ainda não existe**. Shader nodes, animation system, motion nodes nó-graph (atualmente só `motion`), Painter (em W2+), Luau gameplay, MCP — todos são alvos de integração desenhados desde a arquitetura, ainda que cada bridge entre como ADR à medida que a outra ponta amadureça.

---

## 1. Os cinco eixos

### 1.1 Sabor Illustrator-superado (capability)
Tudo que Illustrator faz bem — pathfinder boolean, mesh gradients, variable-width strokes, art/pattern brushes, recolor artwork, blend tool, live effects — **o Vector Module faz, mas como nó vivo no graph**. Nada bake-and-discard. Pathfinder vira `vector.bool.{union,subtract,intersect,exclude,divide,trim,merge,crop,outline}` — todos como nodes editáveis para sempre, com as operandos preservadas, animáveis e replayáveis em runtime.

### 1.2 Sabor Figma (topologia)
**Data model = vector network**, não path. Vértices com N arestas incidentes; segmentos com handles tangentes por extremidade; regions indexam segmentos com winding rule. Um cubo é UM network com vértices compartilhados — não três subpaths separados. Crossings auto-inserem intersection vertices. Minimal-cycle-basis identifica regions preenchíveis. Cada region tem seu próprio fill. Decisão arquitetural já tomada — vide §7 + ADR-0056.

### 1.3 Sabor Cavalry/Houdini/Blender (procedural)
Toda operação geométrica é **node**: Boolean, Offset, Outline Stroke, Roughen, Twist, Mirror, Repeat, Scatter Along Path, Width Profile, Recolor, Perspective Warp, Custom-Shader-Fill. Stack ortogonal de modifiers, falloffs espaciais, duplicators distribuídos. **Live geometry graph** que Illustrator nunca teve. Vive em `crates/ph2d-node-vector-*` — drop-crate fan-out exatamente como motion nodes hoje.

### 1.4 Sabor Rive (runtime game)
**Runtime separado, opt-in via crate `ph2d-vector-runtime`**, consumido por jogos shipados. State machine animável que falaria com ECS/Luau. Stroke envelopes + variable-width + procedural fills funcionam no jogo distribuído, não só no editor (HR-7 espelhado). Boolean ops em runtime determinístico opt-in (HR-5). Vector como **asset de gameplay**, não decoração.

### 1.5 Sabor PH2D (LLM-first + multi-plataforma + determinístico)
- **MCP/Luau como graph node**: LLM emite vector network estruturado editável (não SVG dump), node `vector.llm.shape(prompt, constraints)` plugável no graph (HR-10 elevada).
- **iPad Apple Pencil first-class desde W1** (não port): predict+reconcile sub-9 ms ProMotion, Pencil Pro squeeze/barrel-roll, gestos canvas; idem Wacom/Huion/XP-Pen no desktop, S Pen no Android. UI design espelha 4-zonas Procreate-inspired ([ADR-0023](../architecture/decisions/0023-ui-ux-baseline.md)) já compatível com iPad.
- **Determinismo opt-in** (HR-5): vector edit log = event-sourced via `EditorAction`/`ActionBus` (já existe); replay bit-identical cross-platform; CRDT-ready para multi-agente co-edit no futuro.
- **Filtro minimalista-Blender**: um caminho canônico por feature; defaults excelentes; power escondido em sub-painéis (Geometry Graph, Stroke Studio, Fill Studio); atalhos de teclado primeira-classe em desktop.

---

## 2. Estado da arte — sumário da pesquisa (2026)

Pesquisa completa em [`14_estado_da_arte.md`](14_estado_da_arte.md) (a criar). Sumário dos pontos que viraram decisão arquitetural:

| Tool / Pesquisa | O que pegamos | O que descartamos / superamos |
|---|---|---|
| **Adobe Illustrator** | Vocab familiar (Pathfinder names, Pen/Direct Select, brushes nomenclature) | Boolean destrutivo, mesh gradient hand-author, ExtendScript, sem GPU compute, sem runtime, iPad port |
| **Affinity Designer** | Compound shapes não-destrutivas, Contour tool, dual persona (vector+pixel) | Sem node graph, sem animation, sem runtime, sem scripting decente |
| **Figma** | **Vector network como data model** (decisão fundamental), per-region fills, crossings auto | Boolean ainda bake, plugin sandbox, bezier-only |
| **Cavalry** | Generators / Modifiers / Behaviors / Falloffs / Duplicators / JSON data binding | Motion-first; static illustration awkward; sem runtime de jogo |
| **Linearity Curve / Vectornator** | Auto-trace por modo (Sketch/Illustration/Basic Shapes), Pencil-to-vector real-time, Magic Wand | Sem graph, sem scripting, sem animation |
| **Rive** | **Runtime de jogo** (state machine + bones + mesh deformation + shipping em Unity/Unreal/Bevy) | Data model path-only (não vector network), sem procedural modifiers, sem shader fills procedurais |
| **Cuttle** | **Parametric symbols** (sliders dirigem geometria, beats Figma components 10×) | UI laser-cutter-niche; sem GPU |
| **Inkscape LPE** | Catálogo de Live Path Effects (Pattern Along Path, Bend Path, Roughen, Width Path, Hatch Fill, Spiro/BSpline, Mirror/Rotate Copies, Boolean as LPE) | UX 2010, perf ruim com stack >5, sem graph view |
| **Blender Grease Pencil** | GP↔curves↔mesh dentro de Geometry Nodes; modifier stack (Array, Build, Lattice, Subdivide, Texture Mapping, Tint, Time Offset, Noise) | 3D-first (não nosso domínio) |
| **Houdini SOPs** | Non-destructive default; toda parâmetro animável; toda parâmetro com expressão | Software de aviação para uma planta de manjericão |
| **Vello (Linebender)** | **Renderer canônico** — GPU compute prefix-sum pipeline, sparse strips arch, infinite zoom | Já no stack PH2D ([§5 SKILL_Stack](../../SKILL_Stack_PH2D_Definitiva.md)); só consumir |
| **Linesweeper (Linebender)** | **Boolean robust** — duas-fases (ordering before intersection); robusto em degenerate cases reais | Substituí Clipper/Boost.Polygon (não-robust) |
| **GPU-friendly stroke expansion** (Levien+Uguray 2024) | Variable-width strokes em compute pass único; miter/bevel/round, dashes | CPU stroke (Illustrator) |
| **Spiro / Hyperbezier (Levien)** | **Authoring representation** — clothoid splines (Spiro) ou elastica-under-tension (hyperbezier); melhor default para pen tool | Cubic Bézier como única authoring |
| **Hobby's algorithm** | **Default fitter sob pencil tool** (minimum curvature variation, MetaPost) | Catmull-Rom, Schneider (Inkscape default) |
| **Diffusion curves** + **Gradient meshes (Poisson unification 2024)** | Mesh gradient via **diffusion curve** (curva carrega cor nos dois lados + blur, Poisson diffunde) — autor toca poucos pontos | Hand-author mesh patches |
| **LLM4SVG / SuperSVG / StarVector** | LLM-driven authoring **com editability mantida** (semantic tokens, structured output) | "Generate SVG dump" |
| **Differentiable Variable Fonts (2025)** | Stroke como 1D variable font: axes width / taper / contrast / jitter | Width Tool clássica de Illustrator |

Sources canônicas: vide [`13_referencias.md`](13_referencias.md) (a criar) — citações inline neste doc + bibliografia consolidada lá.

---

## 3. Escopo — IN (versão 1.0 do Vector Module)

Numerado por arquivo do spec (a criar conforme W0 fechar). Tudo aqui é **must-have antes de v1.0**.

### 3.1 Data model (`01_data_model.md`)
- **Vector Network** topology (Figma model): vertices[], segments[], regions[] com winding rules. Per-region fill.
- **Curve representation dual**: authoring = Spiro / hyperbezier (Levien); cooked / export = cubic Bézier (via `vello::kurbo`).
- **Edit log event-sourced**: toda mutação é uma `EditorAction::VectorOp(VectorOp)` no `ActionBus`; replay determinístico cross-platform; CRDT-ready.
- **Asset format**: `.ph2d-vector` (postcard binário, blake3-addressed, HR-6, com migrator HR-14).
- **Import/export**: SVG (subset rico — paths, gradients, masks, clip, text), AI (subset via internal PDF parser, lossy), PDF (read-only via lopdf), JSON (debug + LLM exchange).

### 3.2 Geometry node graph (`02_geometry_graph.md`)
- **Domain `vector` no `ph2d-nodegraph`** (irmão de `motion` que já existe). Crates `ph2d-node-vector-*` via fan-out drop-crate (DIRETRIZ §3.A).
- **Nodes v1.0 (canon)** — cada um vira crate independente:
  - `vector-source` — emite vector network primitivo (rect, ellipse, polygon, star, path)
  - `vector-boolean` — union / subtract / intersect / exclude / divide / trim / merge / crop / outline (via **linesweeper**)
  - `vector-offset` — paralelo / contour (live, via Euler-spiral approximation)
  - `vector-outline-stroke` — converte stroke em path filled
  - `vector-roughen` — perturbação organic parametrizada (frequência / amplitude / smoothness)
  - `vector-twist` — twist em torno de centro
  - `vector-bend-path` — bend ao longo de envelope path
  - `vector-pattern-along-path` — distribui pattern ao longo de path (Illustrator art brush + Inkscape Pattern Along Path)
  - `vector-scatter` — duplica + distribui (radial / grid / random / along-path)
  - `vector-width-profile` — variable width via 1D variable-font-style axes (width / taper / contrast / jitter / pressure)
  - `vector-hatch` — hatch fill parametrizada
  - `vector-mirror` — symmetry (V / H / Quadrant / Radial) — live, não destructive
  - `vector-corner-round` — per-node rounding (live; rounded-corner LPE Inkscape)
  - `vector-warp` — perspective / mesh warp / liquify-style (live)
  - `vector-recolor` — color harmony rules across the whole subgraph
  - `vector-llm-shape` ✨ — node MCP que LLM popula com vector network editável (LLM4SVG-style)
- **Stack ortogonal**: modifiers + falloffs espaciais + duplicators (Cavalry pattern).
- **Custom node via Luau**: `vector-luau-script(input, params) -> output_vector_network` — usuário avançado escreve modifier custom em Luau (HR-10 via MCP).

### 3.3 Renderer (`03_renderer.md`)
- **Vello 0.8 como renderer único** (GPU compute, prefix-sum pipeline, sparse strips). Já no stack PH2D (§5 SKILL_Stack).
- **GPU stroke expansion** (Levien+Uguray 2024 paper, já em Vello): variable width, miter/bevel/round, dashes, em compute pass único.
- **Editor + runtime compartilham renderer**: a tela do editor é uma instância do runtime renderer. WYSIWYG absoluto (HR-7).
- **Sub-pipeline procedural fill**: o fill de uma region pode ser um **shader graph** com **topologia compilada uma vez + UBOs animáveis** (vide §3.5).
- **Frame budget**: vetorial cabe em **3.5 ms** do sub-budget Render (HR-4); booleanos pesados re-rodam off-thread (vide pipeline boolean abaixo).

**Pipeline boolean — draft + reconcile (resolve crítica C):**
Linesweeper é exato mas pesado: redes vetoriais com 100+ segmentos não cabem em sub-ms na CPU móvel/tablet. Pipeline canônico em **três modos seletivos por contexto**:
1. **Draft preview (hot-path, ≤ 1 ms)** — boolean naive em CPU sobre subset reduzido (LOD da §3.10) ou approximation Bézier-cúbica clipping; resultado visualmente plausível para feedback de stylus.
2. **SDF hybrid (real-time, ≤ 0.5 ms compute pass, **§8.7**)** — rasteriza inputs a SDF 2D na GPU, boolean via `min/max` em shader (`min(d1, d2)` união, `max(d1, -d2)` corte, `max(d1, d2)` intersect). Modo ativo durante gameplay morphing e edição interativa. **Limite**: produz silhueta, não preserva topology editável.
3. **Linesweeper exato (async background worker, debounced)** — chamado em **commit do stroke** (mouse-up / pencil-lift) ou após N ms de inatividade. Resultado canônico que vira topology editável (regions / segments / vertices preservados). Cacheado por hash do graph input.

Decisão por-asset / por-tool: tools de edição (Pen / Direct Select) usam Linesweeper exato direto (não há lag de stylus); tools de gameplay morphing (`vector-runtime` em jogo, deformação live) usam SDF hybrid. UI mostra indicador discreto "boolean em commit…" quando worker está computando.

### 3.4 Tools (`04_tools.md`)
Cada um = crate `ph2d-tool-vector-*` (drop-crate fan-out, DIRETRIZ §3.A). Bridge no shell espelha `bgremoval_preview.rs`.

- **`vector-pen`** — Pen tool com **Bézier cúbico como representação default visível** (paridade Illustrator, sem fricção de muscle memory para profissionais vindos de AI/Affinity/Figma). Click adiciona vertex; click+drag estica tangentes cúbicas; close-path. **Assist Modes opcionais** (toggle no HUD `S` / `H`): `Spiro` (clothoid splines, Levien) e `Hyperbezier` (elastica-under-tension, Levien) — útil para letterforms, jewelry-style shapes, organic curves. Data model interno é dual-representation (vide §3.1 + ADR-0056); export para `.ph2d-vector` preserva ambos quando relevantes; export SVG cooka para cúbico (Levien Béz fitting). Bézier default resolve crítica D da avaliação Antigravity (§11.B).
- **`vector-pencil`** — Pencil/Freehand com **Hobby's algorithm** como default fitter (minimum curvature variation). Pressure/tilt → width via `width-profile`. Predict+reconcile loop (HR-1: via `PlatformHost::pencil_predicted_touches()`).
- **`vector-shape`** — Rect / Ellipse / Polygon / Star / Spiral primitivos (cada um é apenas uma `vector-source` pré-configurada).
- **`vector-select`** — Selection rectangle / lasso; trabalha em vertex + segment + region + path level.
- **`vector-direct-select`** — Manipulação direta de vertices / tangent handles / segments.
- **`vector-knife`** — corta segments (cria 2 vertices), preserva continuity.
- **`vector-eyedropper-vector`** — sample style (fill + stroke + effects) de outra path.
- **`vector-paint-bucket`** — fill por região + flood fill quando vector network tem hole.
- **`vector-symbol`** — instancia symbol parametrizado (Cuttle-style — sliders/colors/enums driving geometry; beats Figma components).
- **`vector-text-on-path`** — text rolling along path (parley + kurbo `nearest()` + flow).

### 3.5 Procedural fill — shader graph 2D (`05_procedural_fill.md`)
- **Fill graph** ortogonal ao geometry graph: cada region pode receber `Solid | LinearGradient | RadialGradient | MeshGradient | DiffusionCurve | Pattern | ProceduralShader | Image`.
- **ProceduralShader = node graph** (Blender-style texture nodes para 2D): nós {Noise, Voronoi, Ramp, Mix, Bump, Coord, Math, Image-sample, Time, ph2d-expr}.
- **Mesh gradient via diffusion curve** (Unified Smooth Vector Graphics 2024 paper): curva carrega cor nos dois lados + blur; Poisson diffunde no GPU (Monte Carlo Walk-on-Spheres ou multigrid). Auto-resolve, hand-author elimina mesh patches.
- **Variável de animação**: qualquer parâmetro de shader é animável (vide §3.6) ou recebe input de motion node (vide §3.7).

**Pipeline de compilação — topologia 1× + UBO por frame (resolve crítica B):**
WGSL é caro de compilar via `naga` + criar `wgpu::ComputePipeline` (10-100 ms stalls quebram HR-4 a 120 Hz). Pipeline canônico:
1. **Topologia do shader graph** (que nodes presentes + conexões entre eles) → hash + compile WGSL uma vez por (topology hash, target backend) → cacheado em memória de longo prazo + on-disk (`~/.cache/ph2d/shaders/<hash>.wgsl + .spv/.msl`).
2. **Parâmetros escalares animáveis** (cor, frequência de noise, posição de ramp, time, vetor de coordenada) → empacotados em `UniformBuffer` (UBO) atualizado por frame com zero alloc (HR-3).
3. **Topology change** (usuário pluga node novo no graph) → mostra spinner "compiling shader" no HUD, compila off-thread em background, swap atômico ao terminar; durante compile mostra resultado do template anterior.
4. **Variable Font Glyph como input shader** (vide §8.6): glifo é uma `Texture2D` SDF estática cached + axes (weight/width/slant/optical) entram como floats no UBO; mudança de axis = UBO update, não recompile.

### 3.6 Animation (`06_animation.md`)
- **Toda parâmetro do graph é animável** (Houdini paradigm). Curve per parameter; keyframes via timeline panel.
- **State machine** estilo Rive: estados = preset de parâmetros; transitions com blend (linear / ease / spring). State machine plugável em runtime via `EditorAction::ActivateState("hover")`.
- **Onion skin** quando vector network virou frame em Animation Assist (frame-based; espelha Painter §10).
- **Export**: GIF / APNG / MP4 (via `ph2d-imageio-*`); Lottie (subset, lossy); `.ph2d-vector-anim` postcard binário.

### 3.7 Integração com motion nodes (`07_motion_integration.md`)
- **Motion nodes (Cavalry-style) driving vector params**: motion graph `motion.wave` → param `vector-roughen.amplitude`. Conecta no editor de graph existente.
- **Reverso**: `vector-network` como input para motion nodes (e.g., `motion.scatter-along-path` consome path do graph vector).
- **Determinismo cascading**: se o motion graph é determinístico (`SimWorld`), a saída vector network também é determinística (HR-5).

### 3.8 Integração com Painter — vetor ↔ raster bridge (`08_painter_bridge.md`)
Esta é uma das inovações principais (§9.5).

- **Paint into vector**: usuário pinta com brush Painter dentro de canvas vetorial. Cada stroke vira `vector.pencil` path automaticamente (Hobby fitter), com pressure → width-profile, tilt → asymmetric envelope. Resultado é **vetor editável** com look de brush.
- **Vectorize raster**: comando "Vectorize layer" no Painter chama `vector-auto-trace` (modos Sketch / Illustration / Basic Shapes — Linearity Curve pattern, mas com ML via SuperSVG / LLM4SVG).
- **Vector com look de brush**: `vector-pattern-along-path` recebe stamp do `ph2d-painter-brush` library (qualquer brush canon do Painter) e aplica ao longo do vector path. Vector traçado parece pintado a mão sem perder editability.
- **Adjustment layers**: 12 adjustments do Painter (HSB, Curves, Gradient Map, Blur, etc., vide [ADR-0045](../architecture/decisions/0045-adjustment-layers.md) Painter) aplicáveis a vector layers via `vector-adjustment` node.

### 3.9 Integração com Luau + MCP (`09_scripting_mcp.md`)
- **Cada node expõe `#[lua_export]`** (HR-10): `ph2d.vector.boolean.union(a, b)`, etc.
- **Custom modifier em Luau**: `vector-luau-script` node carrega um Luau script (sandbox trusted/untrusted) que recebe vector network input e devolve output. Mesmo pattern de `ph2d-script` (M7 fechado).
- **MCP tools**:
  - Read-only: `vector_query`, `vector_inspect`
  - Mutative: `vector_create_path`, `vector_apply_node`, `vector_set_param`
  - Destructive (HR-11): `vector_delete_path`, `vector_clear_all`, `vector_flatten` (boolean apply destructive)
- **LLM-as-graph-node** ✨: `vector-llm-shape(prompt, constraints) → vector_network` — LLM emite vector network estruturado (LLM4SVG semantic tokens, **não SVG opaco**). Editável downstream. Re-promptable. Memory via `ph2d-bindgen` schema.

### 3.10 Integração com runtime / gameplay (`10_runtime_gameplay.md`)
- **Crate `ph2d-vector-runtime`**: subset do editor runnable em release de jogo (sem editor, sem Tool Studio). Aceita `.ph2d-vector` asset + state machine + run.
- **Live boolean em runtime** (a inovação central — vide §8.1): boolean ops podem rodar em runtime determinístico, em **simulation tick** ou **present tick** conforme decisão por-asset. Gameplay shape morphing (Mario eats mushroom; sword cuts shape) com geometry real, não sprite swap.
- **SDF hybrid mode em gameplay** (vide §8.7): morphing/cutting interativo a 120 FPS via boolean GPU SDF (`min/max` em shader compute); CPU Linesweeper reconcile apenas em frame de "commit" (e.g., quando a espada termina swing). Determinismo opt-in: SDF resolution + ordering of reductions documentados em ADR-0065.
- **State machine driven by ECS**: Luau emite `vector.state.set("hover")`, state machine transita, blend interpola params, renderer faz WYSIWYG.
- **LOD vetorial dinâmico (resolve Proposta 2 Antigravity)**: runtime aplica curve-aware path simplification antes do Vello sparse-strips. Algoritmo = Bézier-aware adaptive fitting (Levien `flatten_to_polyline` + RDP em 1ª pass, depois re-fit a poucos cúbicos). Threshold de detail driven pela câmera (distância world-space + cobertura em pixels da bbox); per-asset override (heroi sempre full detail, props distantes simplificam). Mantém frame budget 3.5 ms mesmo com 50+ elementos vetoriais em tela. Gate `tests/budget/vector_runtime_lod.rs`.
- **Physics collider integration (resolve Proposta 4 Antigravity — vide §8.5):** corpos rígidos Rapier 2D gerados automaticamente da `VectorNetwork` (decomp convex via earcut; ou direct rapier `SharedShape::convex_hull` por region). Quando `vector-boolean.subtract` runtime corta um asset (espada corta tábua), collider é **dividido em N corpos independentes** em tempo real (corte SDF GPU → vector network resultante via Linesweeper async → re-decomp). Joint constraints opcionais (cloth-like tearing). Mass derivada da área da region × density do material.
- **Memory budget**: 80 MB VRAM + 30 MB RAM padrão mobile / 200 MB + 100 MB desktop / 40 MB + 20 MB web (HR-13).

### 3.11 Input pipeline — Pencil/Wacom/Mouse multi-plat (`11_pencil_pipeline.md`)
- Apple Pencil (iPad / iPad Pro M-series), Wacom / Huion / XP-Pen (desktop), S Pen (Android), mouse fallback. Tudo via `ph2d-input` (HR-1).
- Pressure / tilt / azimuth / barrel-roll. Predict+reconcile loop sub-9 ms ProMotion.
- Pressure curve global em `Vector Preferences` + per-tool em Tool Studio (Pen / Pencil têm curves distintos).
- Palm rejection automática.
- Hover preview em devices que suportam (iPad M2+, Wacom hover-enabled). Mostra Spiro tangent preview antes do click.

### 3.12 UX chrome (`12_ux_chrome.md`)
- **Layout 4-zonas** existente em [ph2d-editor-core](../../crates/ph2d-editor-core/) (ADR-0023): Vector Module ocupa Center 100% canvas + sidebar esquerda (tool selector vertical) + top bar (File / Edit / Select / Object / Path / Effect / View) + bottom HUD (zoom / coord / status).
- **Painel docado canônico = Geometry Graph** (visualização do node graph ativo do layer selecionado, editável). Pode flotar via `FloatingPanel` (Procreate-style).
- **Painel docado secundário = Inspector** (params do node selecionado no graph).
- **HUD durante edit**: chrome esmaece quando ferramenta ativa toca canvas, números de slider flutuam perto.
- **Zen Mode** (4-finger tap em iPad / Tab em desktop) — só canvas + tool ativo.
- **Atalhos Blender-style em desktop** (primeira classe): P (pen), B (pencil), V (select), A (direct-select), N (knife), R (rotate), S (scale), G (move), Ctrl+J (join), Ctrl+G (group), Ctrl+Shift+G (ungroup), Tab (zen), Ctrl+Z/Y, 1-9 (layer pick), F (fill panel), L (stroke panel).
- **Gestos canvas (iPad)**: 2-finger undo, 3-finger redo, 4-finger zen, pinch zoom+rotate, tap-and-hold → eyedropper, draw-and-hold → QuickShape (line / arc / polygon / ellipse).
- **QuickMenu radial** (6 slots × 4 menus salváveis) — espelha Painter.

---

## 4. Escopo — OUT (não-objetivos explícitos v1.0)

Cada "não" abaixo é decisão consciente; reverter exige ADR. Detalhe em [`13_fora_de_escopo.md`](13_fora_de_escopo.md) (a criar).

- **3D vector / parametric 3D primitives.** PH2D é 2D engine ([§3 SKILL_Stack](../../SKILL_Stack_PH2D_Definitiva.md) Não-objetivo #1). Importar de glTF para 2.5D normal maps OK; criar 3D no Vector Module fora.
- **Print production CMYK first-class.** P3/sRGB internos (mesma decisão do Painter §03). CMYK só em export-side se demanda real surgir.
- **DTP / page layout (InDesign-class).** Multi-page documents, master pages, story flow — não-objetivo.
- **Live web-collab Figma-style.** CRDT data model é arquitetado, mas servidor de colaboração não é v1.0. Multi-agente local-only (sessions LLM paralelas) sim; multi-human cross-internet não.
- **Vector AI / motion-tween-by-prompt como única autoria.** LLM emite **strokes editáveis** (§9.4), não substitui artista. Sem "magic generate full illustration" sem editing.
- **ExtendScript / CEP / proprietary plugin SDK.** Plugins via Luau / WASM / MCP somente (HR-8 + HR-10 + HR-11).
- **Compatibility 100% com Illustrator AI nativo.** Import lossy documentado; round-trip não é objetivo.
- **PDF authoring (criar PDF arbitrário).** Export PDF subset OK (paths / gradients / text); criação arbitrária fora.
- **Mesh gradients hand-author (Illustrator-style mesh patches).** Substituído por **diffusion curves** (§3.5; Poisson PDE unification 2024).
- **SVG Filters DOM-level (feGaussianBlur etc).** Substituído por procedural shader graph (§3.5). Import de SVG com filters: degraded para "best effort raster" se filter não tem equivalente.

> **Removidos da OUT-list (agora IN, pós-crítica Antigravity 2026-05-27):**
>
> - ~~CRDT data model como infra "ready-but-not-implemented"~~ → **IN W1** (ADR-0057). LWW-Element-Set OR RGA OR CRDT custom estruturando `edit_log` desde dia 1 (Proposta 5 Antigravity). Habilita **multi-agente local agent ↔ designer** (LLM assistente + Enio editam mesmo canvas em paralelo) com resolução determinística de conflitos. Continua **OUT** a colaboração web cross-internet via servidor (out v1.0; arquitetura CRDT torna trivial em vN+1).
> - ~~Vector colliders dinâmicos como "futuro distante"~~ → **IN W10/W16** (ADR-0063 + ADR-0066; Proposta 4 Antigravity). Rapier 2D 0.28 + dynamic split de colliders em runtime boolean cut. Sword-cut → split físico real. Gameplay diferencial brutal.
> - ~~Tipografia generativa relegada a "v2.0"~~ → **IN W11+** (ADR-0066). Variable Fonts axes (weight/width/slant/optical) expostos como params do graph; motion fields + Luau deformam tipografia sem rasterizar (Proposta 3 Antigravity).
> - ~~Mesh gradient hand-author (Illustrator-style)~~ continua OUT — **substituído** por diffusion curve via Poisson PDE (vide §3.5 + §8.2).

---

## 5. Filtro minimalista-Blender — princípios operacionais

Quatro regras em toda decisão (igual Painter §5):

### 5.1 Um caminho canônico por feature
Se Illustrator tem 3 jeitos de unir 2 paths (Pathfinder / Compound Path / Shape Builder), Vector Module expõe **1 default** (`vector-boolean.union` node) **+ 1 customizable em Gesture Controls**. Sem 3 botões na UI primária.

### 5.2 Defaults excelentes; preferences escondidas
Pen tool default: Spiro authoring rep, Hobby fit no Pencil, Vello renderer, Linesweeper boolean — tudo sem o usuário tocar config. Tool Studio / Stroke Studio / Fill Studio existem mas usuário típico nunca abre.

### 5.3 Atalhos de teclado em desktop são primeira-classe
A grande lacuna do Illustrator é UX. Atalhos Blender-style canônicos (§3.12). Em iPad, gestos cobrem mesma função.

### 5.4 Power escondido atrás de sub-painéis especializados
Profundidade (Tool Studio, Stroke Studio, Fill Studio, Geometry Graph editor, Animation Curves) em sub-painéis docados. UI primária minúscula: 10 tools + sidebar 4 elementos + color thumb.

---

## 6. Multi-plataforma desde W1 (vs Illustrator iPad port)

| Plataforma | Input principal | Input secundário | Atalhos | Gestos multi-touch |
|---|---|---|---|---|
| Desktop (Mac/Win/Linux) | Wacom/Huion/XP-Pen tablet | Mouse | **Primeira-classe (Blender-style)** | Trackpad multi-touch |
| iPad / iOS | Apple Pencil (2/Pro) | Finger touch | Hardware keyboard quando presente | **Primeira-classe** |
| Android | S Pen / Wacom Android | Finger touch | Hardware keyboard quando presente | **Primeira-classe** |
| Web | Pointer Events API (touch/pen/mouse) | — | Browser-limited | Pointer Events |

Implicações concretas (espelha Painter §6):
- Apple Pencil Pro features detectadas runtime; sem build conditional.
- Hover preview funciona em qualquer device com pointer hover.
- Pressure normalizada `[0.0, 1.0]` em `ph2d-input` (HR-1); curves per-tool no espaço normalizado.
- **A UI Hero atual (4-zonas Procreate-inspired, ADR-0023)** já está preparada para iPad; Vector Module reusa o chrome como Painter reusa, com substituição de pills/sidebar via `ActivateTool("vector-*")`.

---

## 7. Mapping arquitetural ao PH2D

### 7.1 Crates novos (W1+)

**Crates existentes que o Vector Module consome** (não duplicar):
- [`ph2d-vector`](../../crates/ph2d-vector/) — Vello wrapper já existe; **expandido** para Vector Network data model + **ÚNICO PONTO de acoplamento físico com Vello/kurbo/peniko APIs** (Antigravity 3ª iteração L6F1 2026-05-29 — long-tail maintenance: Vello version churn cada quarter; encapsular em 1 crate central reduce N-crate upgrade cost). Outros 31 crates Vector Module consomem **PH2D-domain types** (e.g., `ph2d_vector::Pos2d`, `ph2d_vector::Network`) em vez de re-exporting `vello::kurbo::Vec2`. Isolation gate: arch-test `vello_kurbo_only_in_ph2d_vector` verifica nenhum crate fora de `ph2d-vector` importa `vello::*` ou `kurbo::*` direto.
- [`ph2d-nodegraph`](../../crates/ph2d-nodegraph/) — graph engine; domain `vector` é novo dominio.
- [`ph2d-tool-registry`](../../crates/ph2d-tool-registry/) — `ToolManifest`, `hash_node_id`, registry.
- [`ph2d-tool-runtime`](../../crates/ph2d-tool-runtime/) — 4 drivers helpers (Wave 10 infra), Vector tools consomem.
- [`ph2d-editor-core`](../../crates/ph2d-editor-core/) — `Tool` + `RasterEditTool` + `PanelEvent` traits + chrome + widgets.
- [`ph2d-tokens`](../../crates/ph2d-tokens/) — OKLCH colors, Spacing, Radius, TypeToken.
- [`ph2d-text`](../../crates/ph2d-text/) — parley wrapper para text on path.
- [`ph2d-a11y`](../../crates/ph2d-a11y/) — AccessKit wrapper.
- [`ph2d-gpu`](../../crates/ph2d-gpu/) — wgpu 28 wrapper para compute (boolean offline + procedural fill shaders).
- [`ph2d-input`](../../crates/ph2d-input/) — pencil/tablet/mouse abstração (HR-1).
- [`ph2d-script`](../../crates/ph2d-script/) — Luau runtime para custom modifiers + LLM bridge.
- [`ph2d-mcp`](../../crates/ph2d-mcp/) — MCP server skeleton, tools `vector_*`.
- [`ph2d-painter-brush`](../../crates/ph2d-painter-brush/) — brush library reusada em `vector-pattern-along-path` (§3.8).

**Crates NOVOS — família Vector Module** (drop-crate fan-out (A) per DIRETRIZ v7.0 §3.A).

**Decisão de granularidade (resolve crítica A Antigravity 2026-05-27 + L1F1/L3F1 2ª iteração 2026-05-28):** mantemos drop-crate fan-out como espinha (DIRETRIZ §3.A não negociável), mas **consolidamos seletivamente** via 4 merges (panels Studios → 1; utility tools → 1; transforms triviais → 1; llm+node wrapper → 1) + adição de `ph2d-brush-traits` (resolve circular dep Painter↔Vector). **Total: exatamente 32 crates** (lista completa em [17 §24](17_plano_de_implementacao.md) — Painter tem ~10; node domain `motion` tem 3). **Não monolítico** (proposta original Antigravity de colapsar pra 5-6 crates rejeitada novamente em 2ª iteração porque viola DIRETRIZ §3.A drop-crate fan-out + HR-18 god-file + bloqueia paralelismo multi-agente).

```
crates/
├── ph2d-vector-traits/          ✨ Mocks + abstrações + AnimValue enum (W1 day 1)
│   │                              destranca W1-W5 antes de Shader Graph /
│   │                              Animation System maduros (resolve crítica E)
│   ├── src/lib.rs
│   ├── src/anim_value.rs        AnimValue enum {Float, Vec2, Vec3, Color, Bool, Enum}
│   │                              + LinearInterp trait per-variant
│   │                              (resolve L1F4/L6F1 Antigravity 2ª iteração:
│   │                              f32 trait return type quebraria W10+)
│   ├── src/attribute_evaluator.rs   trait AttributeEvaluator → AnimValue (typed)
│   ├── src/procedural_fill_shader.rs trait ProceduralFillShader
│   ├── src/animation_curve.rs   trait AnimationCurveSampler → AnimValue
│   ├── src/mocks.rs             impls Mock para cada trait (linear interp
│   │                              per AnimValue variant, solid fill básico)
│   └── tests/
│
├── ph2d-brush-traits/           ✨ Contratos desacoplados Brush (W1 day 1; L6F2 fix)
│   │                              Resolve circular dep Painter↔Vector:
│   │                              ambos importam linearly daqui em vez de
│   │                              cross-importarem direto. BrushRef + StampSpec
│   │                              + interface BrushEngine abstraída
│   ├── src/lib.rs
│   ├── src/brush_ref.rs         BrushRef + BrushHandle types
│   ├── src/stamp_spec.rs        StampSpec interface (pos, tangent, pressure, ...)
│   ├── src/brush_engine.rs      trait BrushEngine (consumido por ambos)
│   └── tests/
│
├── ph2d-vector-doc/             ✨ Vector Network data model (vertices/segments/regions)
│   │                              + Spiro/hyperbezier authoring representation
│   │                              + Bézier cúbico (representação default visível)
│   │                              + serde/postcard schema (HR-14 versioned)
│   │                              + CRDT (LWW/RGA/custom — ADR-0057)
│   ├── src/lib.rs
│   ├── src/network.rs           VectorNetwork struct + invariants
│   ├── src/cubic.rs             Bézier cúbico (default visível ao user)
│   ├── src/spiro.rs             Spiro / hyperbezier (Assist Modes)
│   ├── src/cubic_fit.rs         conversão Spiro/hyper → cubic (Levien Béz fitting)
│   ├── src/edit_log.rs          event-sourced ops + CRDT replay determinístico
│   ├── src/crdt.rs              LWW-Element-Set / RGA / custom (vide ADR-0057)
│   └── tests/
│
├── ph2d-vector-runtime/         ✨ Game runtime (Rive-class)
│   │                              state machine + bones + mesh deform + animation
│   ├── src/lib.rs
│   ├── src/state_machine.rs
│   ├── src/animation.rs
│   ├── src/skeleton.rs          bones + vertex weighting
│   └── tests/                   determinism replay cross-platform
│
├── ph2d-vector-fill/            ✨ Procedural fill shader graph
│   │                              compile DAG → WGSL on-the-fly (cached by hash)
│   ├── src/lib.rs               FillGraph + Node enum (Noise/Voronoi/Ramp/Mix/...)
│   ├── src/wgsl_codegen.rs      DAG → WGSL string
│   ├── src/diffusion_curve.rs   Poisson PDE solver (compute pass)
│   ├── shaders/diffusion.wgsl   Walk-on-Spheres or multigrid Poisson
│   └── tests/                   golden shader output
│
├── ph2d-vector-llm/             ✨ LLM authoring node + MCP bridge
│   │                              vector-llm-shape node (LLM4SVG semantic tokens)
│   ├── src/lib.rs
│   ├── src/semantic_tokens.rs   parse structured LLM output → vector network
│   ├── src/mcp_tools.rs         MCP tool schemas + governance
│   └── tests/
│
├── ph2d-tool-vector-pen/        ✨ Pen tool (Bézier cúbico default + Spiro/Hyperbezier Assist toggle)
├── ph2d-tool-vector-pencil/     ✨ Pencil freehand (Hobby fitter, pressure→width)
├── ph2d-tool-vector-shape/      ✨ Rect/Ellipse/Polygon/Star/Spiral primitives
├── ph2d-tool-vector-select/     ✨ Marquee + lasso selection
├── ph2d-tool-vector-direct/     ✨ Direct vertex/tangent manipulation
├── ph2d-tool-vector-knife/      ✨ Knife (cuts segments preserving continuity)
├── ph2d-tool-vector-bucket/     ✨ Paint bucket (region fill + flood fill on holes)
├── ph2d-tool-vector-symbol/     ✨ Symbol instance (Cuttle-parametric)
├── ph2d-tool-vector-text-on-path/ ✨ Text on path (parley + kurbo nearest)
│
├── ph2d-node-vector-source/     ✨ emit primitive vector network (consolida 5 primitives:
│   │                              rect / ellipse / polygon / star / spiral — multi-variant
│   │                              em vez de 5 crates triviais, resolve crítica A Antigravity)
├── ph2d-node-vector-boolean/    ✨ union/subtract/intersect/exclude/divide/trim/merge/crop/outline
│   │                              (linesweeper exato no commit + SDF GPU draft real-time;
│   │                              vide §3.3 pipeline boolean draft+reconcile)
├── ph2d-node-vector-offset/     ✨ parallel/contour offset (GPU Euler-spiral)
├── ph2d-node-vector-outline-stroke/  ✨ stroke→filled path conversion
├── ph2d-node-vector-roughen/    ✨ organic perturbation (freq/amp/smooth)
├── ph2d-node-vector-twist/      ✨ twist around center
├── ph2d-node-vector-bend-path/  ✨ bend along envelope path
├── ph2d-node-vector-pattern-along-path/ ✨ distribute pattern (uses Painter brush library)
├── ph2d-node-vector-scatter/    ✨ duplicate + distribute (radial/grid/random/along-path)
├── ph2d-node-vector-width-profile/ ✨ variable width (variable-font 1D axes)
├── ph2d-node-vector-hatch/      ✨ parametric hatch fill
├── ph2d-node-vector-mirror/     ✨ live symmetry (V/H/Quad/Radial)
├── ph2d-node-vector-corner-round/ ✨ per-node rounding (live)
├── ph2d-node-vector-warp/       ✨ perspective/mesh warp/liquify
├── ph2d-node-vector-recolor/    ✨ color harmony across subgraph
├── ph2d-node-vector-llm-shape/  ✨ LLM-driven shape node (consumes ph2d-vector-llm)
├── ph2d-node-vector-luau-script/ ✨ custom modifier in Luau
│
├── ph2d-panel-vector-graph/     ✨ Geometry Graph editor panel
├── ph2d-panel-vector-inspector/ ✨ Node params Inspector panel
├── ph2d-panel-vector-symbol-lib/ ✨ Symbol library panel
└── ph2d-panel-vector-tool-studio/ ✨ Tool Studio panel (Pen/Pencil curves, etc.)
```

**Bridges no shell** (espelha Wave 10 padrão):

```
shells/desktop/src/render_loop/
├── vector_pen_bridge.rs ✨        (NEW, espelha bgremoval_preview.rs)
├── vector_pencil_bridge.rs ✨
├── vector_select_bridge.rs ✨
└── ...                            (um bridge por tool stateful)
```

**Wiring (codegen syncs já existentes):**
- `cargo run -p ph2d-tool-sync` regenera `ph2d-tool-registry-init` (todos tools, Vector Module incluso).
- `cargo run -p ph2d-node-sync` regenera `ph2d-node-registry-init` (todos nodes, domain `vector` incluso).
- `cargo run -p ph2d-panel-sync` regenera registry de painéis.
- `cargo run -p ph2d-chrome-sync` regenera chrome handlers.

### 7.2 Contratos congelados que o Vector Module respeita

- **`Tool` / `RasterEditTool` / `PanelEvent`** (ADR-0040 + ADR-0041, gate [`architecture_tool_contract_surface`](../../crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs); caps `Tool=10` / `RasterEditTool=5` / `PanelEvent=4`). **Vector Module NÃO implementa `RasterEditTool`** (não é raster); implementa `Tool` puro + downcast `as_any_mut` para state vector-specific. Pode ser que após W1 surja necessidade de `VectorEditTool` trait paralelo (5 métodos: `set_source / current_render / take_pending_commit / run_full / deactivate`) — se sim, é amendment de ADR-0040 (cap-bump) + ADR-0058 novo.
- **`NodeOp` / `OpResolver` / `NodeManifest`** (ADR-0039, caps 2/1/8). Domain `vector` adiciona nós; nenhum cap-bump previsto. Param vocabulary segue convenção existente (`f32` / `u32` / `String` / `Color` / `Path`).
- **`EditorAction`** (4 variants genéricos). Vector Module usa:
  - `ActivateTool("vector-pen")` etc.
  - `OneShotImageOp(...)` ZERO — vector é stateful, não one-shot.
  - `ToolPanelEvent(PanelEvent::SetValue|Click(id, ...))` para todos panel widgets.
  - `CancelActiveTool` ao trocar de ferramenta com edit em vôo.
  - **Vector-specific dispatch**: edit log lives no `ActionBus` como ToolPanelEvent payload OR via novo variant em `EditorAction` se surgir necessidade (ADR-0057).
- **Painter contracts** (ADR-0043..0053): Painter `PainterUiEdit` / `Brush` consumidos via `ph2d-painter-brush` API pública.

### 7.3 Hard Rules aplicadas

| HR | Aplicação no Vector Module |
|----|---------------------------|
| **HR-1** | Platform-agnostic; input via `ph2d-input` (pointer/pencil/touch); files via `PlatformHost`; zero `cfg(target_os)`. |
| **HR-3** | Hot path renderer: zero `Box::new` / `Vec::push`-realloc / `String::from`. Pool pré-alocado de path data, bump arena por frame, ring buffer pra undo. Gate em `tests/budget/vector_no_alloc.rs`. |
| **HR-4** | Vector renderer cabe no sub-budget **3.5 ms** (Render). Boolean ops pesados em compute pass off-thread com cache (resultado guardado por hash do graph input). Procedural shader compile fora de hot path. |
| **HR-5** | Vector Module vive em `PresentWorld` por default (ADR-0021). Determinismo opt-in via flag de asset (`.ph2d-vector` com `deterministic: true`): boolean ops usam fixed-point, ordering of reductions, Linesweeper deterministic mode. **CRDT replay** (`edit_log` event-sourced via `ph2d-vector-doc::crdt`) bit-identical cross-platform — gates `tests/determinism/vector_replay.rs` + `tests/determinism/vector_crdt_convergence.rs`. SDF Hybrid Pipeline (§8.7) marca `deterministic: false` por default (FMA + ordering); opt-in via fixed SDF resolution + ordered reductions documentado em ADR-0065. |
| **HR-6** | Assets blake3-addressed: `.ph2d-vector` (document), `.ph2d-vector-anim` (animation), `.ph2d-vector-symbol` (symbol). Strokes content-addressed; refactor preserva referências. |
| **HR-7** | Vector Module compilado só com feature `editor` em modo full; runtime crate `ph2d-vector-runtime` compilável em release de jogo distribuído. |
| **HR-10** | Cada node + cada tool param + cada state machine state exposto via `#[lua_export]` → MCP toolset `vector_*`. |
| **HR-11** | `vector_delete_path`, `vector_clear_all`, `vector_flatten` (apply boolean destructive) exigem confirmation token MCP. |
| **HR-12** | Todo widget (vertex handle, segment, tangent, slider, node param, symbol thumbnail) emite `accesskit::Node`. Node graph com role `Graph`, vertex handle com role `Slider2D`. |
| **HR-13** | `Plugin::init` declara `MemoryBudget`: desktop 200 MB VRAM (renderer + cache de boolean) + 100 MB RAM (edit log + undo); mobile 80 MB + 30 MB; web 40 MB + 20 MB. |
| **HR-14** | `.ph2d-vector` versionado com migrator obrigatório (`migrate_v1_to_v2`). Schema FREEZE em W2; cada bump gera ADR. |
| **HR-15** | Strings via Fluent (`t!`); locale bundles em `crates/ph2d-tool-vector-*/locales/`. |
| **HR-16** | Vector edit log entries são POD-like; sem closures, sem userdata; iteração ordenada para serialização. |
| **HR-18** | Nenhum arquivo ≥ 600 LOC; funções ≤ 200 LOC. Geometry Graph editor decomposto em sub-módulos (`graph_editor/{layout, edge_drag, node_pick, ...}.rs`). |
| **Práticas W1** | **Traits abstratas + mocks** (`ph2d-vector-traits` crate, resolve crítica E Antigravity): `AttributeEvaluator` (animation curve), `ProceduralFillShader` (shader graph mock), `AnimationCurveSampler` (timeline mock). Implementações Mock simples (linear interp, solid fill) destrancam testes fim-a-fim em W1-W5 antes que Shader Graph / Animation System reais amadureçam. Real impls substituem mocks via trait-object swap em W6+ / W10+. |

### 7.4 Dependências externas (além do stack PH2D pinado)

Reutiliza tudo o que já está em §5 SKILL_Stack. Possíveis deps NOVAS a justificar em PR:

| Crate | Versão alvo | Para quê | Justificativa |
|---|---|---|---|
| `linesweeper` | beta (já listado em SKILL §5 como "não wired") | Boolean ops robustos | Substitui Clipper/Boost.Polygon; criado pelo Joe Neeman (Linebender ecosystem); cobertura de degenerate cases reais |
| `usvg` | 0.43+ | SVG import | Já listado em SKILL §11.10 (asset importers v1) |
| `lopdf` ou `pdf-rs` | recente | PDF read | Investigar W1; se ambos morenos, write-own minimal parser |
| `spiro` (libspiro Rust port) | 0.x | Spiro authoring | Alternativa: port direto a partir de [libspiro C](https://github.com/fontforge/libspiro) — ~500 LOC de port, vale a pena Rust-puro |
| Hobby's algorithm | inline em `ph2d-vector-doc` | Pencil fitter | Sem crate canon — ~200 LOC inline a partir do paper |

Texture nodes / shader graph compilation usa `naga` (já pinado, acompanha wgpu 28).

---

## 8. Inovações extraordinárias — onde o Vector Module supera Illustrator

Cinco propostas que **nenhuma ferramenta vetorial mainstream entrega hoje**. Detalhe técnico em [`14_inovacoes_extraordinarias.md`](14_inovacoes_extraordinarias.md) (a criar).

### 8.1 ✨ Live Boolean Graph — toda operação destrutiva do Illustrator vira nó vivo
**Problema:** Illustrator Pathfinder bake-and-discard; Affinity compound shapes só boolean (não offset/distort/scatter/etc.); Figma boolean ainda bake.
**Solução:** **TODA operação geométrica do Vector Module é um nó no graph** (`ph2d-nodegraph` já existente, domain `vector`). Boolean, offset, outline, roughen, twist, bend, scatter, mirror, corner-round, warp, recolor — todos editáveis para sempre, com operandos preservados, animáveis em curve, replayáveis em runtime determinístico (HR-5).
**Tech:** Linesweeper (robust boolean) + Levien GPU stroke expansion (offset/outline live em compute) + node graph cache por hash.
**Impacto:** trabalho que leva minutos no Illustrator (apertar Pathfinder, perceber erro, undo, reorganizar layers, re-apertar) leva segundos (mexe slider no nó). Em runtime de jogo, gameplay morph de shape real é trivial (espada corta inimigo, formato muda — sem sprite swap).

### 8.2 ✨ Mesh gradient via diffusion curve — autor toca pontos, GPU resolve Poisson
**Problema:** Illustrator mesh gradient hand-author é doloroso (mesh patches manual); export rasteriza.
**Solução:** [Unified Smooth Vector Graphics (2024)](https://arxiv.org/pdf/2408.09211) — mesh gradients e diffusion curves são duas formas da mesma Poisson PDE. Autor desenha curva, marca cor em ambos lados (opcional + blur), GPU difunde o resto. Live, GPU-resident, infinite zoom.
**Tech:** Walk-on-Spheres Monte Carlo OR multigrid Poisson em compute pass (`ph2d-vector-fill/shaders/diffusion.wgsl`). 2026 hardware torna real-time tractable que não era em 2008.
**Impacto:** "Photoreal vector" sem mesh patches. Não existe em ferramenta mainstream.

### 8.3 ✨ Painter ↔ Vector bridge bidirecional
**Problema:** Procreate é raster-only, Illustrator é vector-only, nenhum oferece autoria fluida cruzada.
**Solução tripla:**
1. **Paint into vector** — usuário pinta com Painter brush dentro de canvas vetorial; stroke vira `vector.pencil` path automaticamente (Hobby fitter; pressure → width-profile). Resultado é vetor editável com look pintado.
2. **Vector com look brush** — `vector-pattern-along-path` consome qualquer brush do `ph2d-painter-brush` library e aplica ao longo de path vetorial. Vector traçado parece pintado a mão, sem perder editability.
3. **Auto-trace ML** — comando "Vectorize layer" no Painter chama node `vector-auto-trace` (modos Sketch / Illustration / Basic Shapes — Linearity Curve pattern, mas com ML backbone via SuperSVG / LLM4SVG embedded).
**Impacto:** Painter+Vector juntos = sucessor unificado de Procreate + Illustrator, com transição zero-fricção entre raster e vetor.

### 8.4 ✨ LLM-as-graph-node — IA emite vetor editável, não SVG opaco
**Problema:** "Generate SVG with AI" hoje = colar grande blob de SVG estranho não editável. Tools como Inkscape AI SVG Generator (2026) ainda emitem grupos opacos.
**Solução:** node `vector-llm-shape(prompt, constraints, style_ref) → vector_network`. LLM emite **semantic tokens estruturados** (LLM4SVG pattern) que o parser converte para Vector Network nativo. Resultado **100% editável downstream do node** (pode dar slider de roughness ao output do LLM!). Re-promptable. Memory via `ph2d-bindgen`.
**Tech:** MCP tool `painter_paint_strokes` precedente (Painter W13 ADR-0047) — espelha pattern. Outputs schema = Vector Network postcard + LLM context preserved.
**Impacto:** primeira ferramenta vetorial onde "AI gera arte" = "AI ajuda artista", não "AI substitui artista".

### 8.5 ✨ Vector Runtime de jogo determinístico + Physics Colliders Dinâmicos
**Problema:** Rive prova o mercado, mas data model path-only (não vector network), sem procedural modifiers, sem shader fills, sem determinismo cross-platform garantido, **sem integração física**.
**Solução tripla:**
1. **`ph2d-vector-runtime` crate ship-em-jogo** com: vector network completo + node graph modifier stack + state machine + shader graph fills + opt-in determinism (HR-5 + ADR-0021 SimWorld). Boolean ops em sim tick = fixed-point + ordered reductions. Replay determinístico cross-platform (Linux/Mac/Win) testado em CI.
2. **Physics Collider Integration** (Proposta 4 Antigravity 2026-05-27): corpos rígidos Rapier 2D 0.28 gerados automaticamente da `VectorNetwork` (decomp convex via earcut OR direct `SharedShape::convex_hull` por region). Mass derivada de `area_region × material.density`. Joints opcionais entre regions.
3. **Dynamic Split em runtime**: quando boolean cut runtime corta um asset (espada corta tábua, projétil perfura escudo), pipeline é (a) corte SDF GPU produz silhueta imediata, (b) Linesweeper async produz topology exata, (c) `Vector → Rapier collider re-decomp` divide o corpo rígido em N corpos independentes, (d) momento linear + angular preservado por cada split. Gameplay morphing real, não sprite swap.

**Impacto:** **vector arte em gameplay como first-class asset**, não decoration. Sword-cut em árvore → árvore quebra em 2 pedaços com física correta; explosion deforma terreno; bala atravessa shape morphing collider em tempo real. Nenhuma engine 2D mainstream entrega isso hoje.

### 8.6 ✨ Tipografia Generativa via Variable Fonts axes como graph inputs (NEW — Proposta 3 Antigravity)
**Problema:** tipografia em motion graphics é estática (After Effects deforma rasterizando), em vector graphics é estática (Illustrator não anima axes de variable fonts), em game engines é texto-em-imagem.
**Solução:** **glifo individual = vector network nativo**. Eixos OTF de variable font (`weight` / `width` / `slant` / `optical-size` / `GRAD` / qualquer axis custom) expostos como **parâmetros dinâmicos do graph**, animáveis em curve, atualizáveis por motion fields ou Luau scripts. O renderer (skrifa + Vello) consome o glifo+axes e renderiza sem rasterizar a fonte intermediária.
**Tech:** [Differentiable Variable Fonts 2025 paper](https://arxiv.org/html/2510.07638v1) → gradients de glyph shape w.r.t. axis values; spec habilita gradient descent em axis space. Trait `VariableFontAxis` expõe (name, min, max, default, current) como input/output do graph.
**Exemplos concretos:**
- Logo do jogo deforma weight a cada batida da música via motion node `motion-wave` → `variable-font.weight`.
- HUD do gameplay: número de munição fica mais grosso (weight) e mais largo (width) conforme aproxima do max → param attached via Luau.
- Letterform morphs por proximidade do mouse (falloff radial driving `slant` axis).

**Impacto:** **primeira ferramenta vetorial onde tipografia É vetor animável** — não substituto rasterizado.

### 8.7 ✨ Vector-SDF Hybrid GPU Pipeline (NEW — Proposta 1 Antigravity 1ª iteração)
**Problema:** Linesweeper exato é robusto mas pesado (não cabe em sub-ms na CPU móvel com 100+ segmentos). Gameplay morphing 120 FPS exige boolean ops triviais.
**Solução:** **Boolean ops em compute shader via SDF 2D** com `min(d1, d2)` união, `max(d1, -d2)` corte, `max(d1, d2)` intersect, `abs(d) - r` arredondamento. Pipeline em três modos seletivos por contexto (vide §3.3):
1. **Draft preview hot-path** (Bézier-cúbico clipping naive em CPU) — feedback de stylus.
2. **SDF Hybrid real-time** (compute pass ≤ 0.5 ms, modo gameplay morphing + edição interativa pesada) — `min/max` em shader, custo constante por pixel.
3. **Linesweeper exato async** (background worker debounced, modo commit) — produz vector network editável final.
**Limites do SDF mode:** produz silhueta, **não preserva topology editável** downstream do nó. Resolução SDF determina precisão (default 2× canvas DPI). Determinismo opt-in via ADR-0065 (fixed SDF resolution + ordered reductions).
**Impacto:** **morphing/cutting interativo a 120 FPS** em runtime e editor, sem o gargalo CPU clássico do Illustrator. Sword-cut (§8.5) usa SDF immediate + Linesweeper async para split de collider real.

### 8.8 ✨ Dormant Fracture Edges (NEW — Antigravity L7F1 2ª iteração 2026-05-28)
**Problema:** SDF Hybrid (§8.7) acelera VISUAL cut a sub-ms, mas physics collider split via Linesweeper async produz **descompasso temporal** (objeto visualmente cortado age fisicamente como peça única por até ~50ms). Antigravity catch L1F5 + L7F1.
**Solução:** **Pré-computar fracture lines no editor** (Voronoi sample OR artist-painted breakaway paths) e salvá-las como `DormantFractureSet` no `.ph2d-vector` asset. Em runtime cut:
1. Player swing/projétil impacta asset.
2. Runtime escolhe `DormantFractureEdge` mais próxima do impact point (O(log N) via spatial index).
3. **Instantâneo (sub-ms)**: collider Rapier já pré-decomposto em sub-bodies dormant; activation atômica + momentum applied.
4. Linesweeper async opcional para refinar topology se artista pediu (default: use dormant).
**Tech:** `DormantFractureSet { fracture_edges: Vec<VectorNetwork>, fracture_regions: Vec<RegionId>, sub_bodies: Vec<RuntimePhysicsBody> }` em asset; runtime activation via flag.
**Authoring**: artist desenha breakaway paths OU `vector-voronoi-fracture` node pré-computa N fracture variants automaticamente.
**Impacto:** **gameplay action sword-cut e destruction com ZERO custo CPU no tick de colisão**. Combina com §8.5 dynamic split runtime (fallback se impact não casa com fracture pré-computado). **Resolve descompasso temporal L1F5 simultaneamente.**

---

## 9. Roadmap por waves (expandido — 20 waves, padrão-ouro)

Sequência de fan-out, cada wave entregável e auditada (loop §1 do [HANDOFF_node_system.md](../HANDOFF_node_system.md)). Roadmap **maior que Painter (17 waves) pelo tamanho do escopo**.

| Wave | Conteúdo | Critério de fechamento |
|------|----------|------------------------|
| **W0** | **Spec freeze** — este doc + 9 ADRs (0056..0064). | Todos ADRs Accepted + arch test `vector_contract_surface` ativo. |
| **W1** | **Neck — Vector Network data model + Vello renderer integration**: `ph2d-vector-doc` com VectorNetwork (vertices/segments/regions), Spiro authoring rep, cubic export. Vello pipeline integration. Single `vector-pen` tool. Smoke "click 3 points, see closed path". | Smoke do Enio: troca para Vector Module, clica 3 pontos com Pen tool, vê triângulo fechado renderizado. Cross-platform smoke OK. |
| **W2** | **Vertical Vector MVP**: Pen + Pencil (Hobby fitter) + Shape (rect/ellipse/polygon/star) + Select + Direct Select tools. Color picker (reusa Painter §03). Stroke + Fill básico (solid + linear gradient). Undo via edit log. Schema `.ph2d-vector` FREEZE (HR-14). | Pen + Pencil + Shape funcionam; user pode desenhar logo simples; export PNG via Vello rendering; undo/redo. |
| **W3** | **Geometry Graph foundation** — node graph editor panel (`ph2d-panel-vector-graph`) + 3 nodes pilot: `vector-source` (consolida 5 primitives — rect/ellipse/poly/star/spiral em multi-variant single crate), `vector-boolean` (Linesweeper exato), `vector-offset`. Live edit OK (mover slider → re-render). | User clica 2 paths, adiciona Boolean Union node no graph, vê resultado live; mexe slider → atualiza. |
| **W4** | **Fan-out de geometry nodes** — 12 nodes restantes (`outline-stroke`, `roughen`, `twist`, `bend-path`, `pattern-along-path`, `scatter`, `width-profile`, `hatch`, `mirror`, `corner-round`, `warp`, `recolor`). | Cada node tem crate isolado + golden test; node graph com 5+ nodes renderizam < 3.5 ms (HR-4). |
| **W5** ✨ | **Pencil GPU stroke expansion + Vector-SDF Hybrid Pipeline ativo** (§8.7, ADR-0065): integração com Levien+Uguray paper (já em Vello); pressure/tilt → width-profile real-time. Pipeline SDF GPU ativo como modo alternativo de preview real-time (`min/max` em compute shader); Linesweeper exato continua canônico em commit. | Stroke de pencil tem width variável smooth via GPU expansion, latência sub-9 ms ProMotion. Boolean ops em modo SDF mostram 120 FPS estável com 50+ paths simultaneamente. |
| **W6** | **Procedural fill foundation** — `ph2d-vector-fill` shader graph crate; nodes pilot (Noise, Voronoi, Ramp, Mix, Image-sample); WGSL codegen. | User aplica fill com noise + ramp + voronoi, vê resultado live; cache de shader compile evita stall. |
| **W7** ✨ | **Mesh gradient via diffusion curve** (§8.2, ADR-0058) — Poisson PDE compute pass; UI para autor desenhar curve + cores. | Diffusion curve com 3 cores produz mesh smooth pixel-perfect; perf < 5 ms / canvas 1080p; cross-platform golden test. |
| **W8** | **Pattern Along Path + Painter brush reuse** (§8.3.2) — `vector-pattern-along-path` consome `ph2d-painter-brush` library. | Path traçado com `pencil_2b` brush parece pintado mas é vetor; edit vertex → re-renderiza. |
| **W9** | **Symbol system parametric (Cuttle-style)** — `ph2d-tool-vector-symbol` com sliders typed (number/color/enum/vector) driving geometry. | Symbol "snowflake" com slider `arms = 6` e `roughness = 0.3` se atualiza live em todas instâncias. |
| **W10** | **Animation foundation + Variable Fonts axes como graph inputs** (§8.6, ADR-0066) — toda param do graph animável; timeline panel; curve editor; state machine (presets); glifo individual = vector network nativo; axes OTF (weight/width/slant/optical) expostos como params dinâmicos animáveis. | User anima `vector-roughen.amplitude` de 0 → 1 em 2 seg; preview no canvas. Variable font axis `weight` animado por curve via motion node `motion-wave`. |
| **W11** | **Motion nodes integration** (§3.7) — domain `motion` driving vector params + reverse (vector path como input pra motion). | `motion-wave` driving `vector-roughen.amplitude` produz oscilação visível; cross-domain validation. |
| **W12** ✨ | **Painter ↔ Vector bridge** (§8.3) — paint-into-vector + vector-with-brush-look + auto-trace ML. | Painter brush stroke virou vector network editável; vector path renderiza com look "pencil_2b"; comando Vectorize Layer funciona. |
| **W13** ✨ | **LLM-as-graph-node** (§8.4, ADR-0061) — `vector-llm-shape` consome `ph2d-vector-llm`; LLM4SVG semantic tokens; editability preserved. | Prompt "spiral with 8 arms golden ratio" → vector network editável; user re-prompts ou move slider downstream. |
| **W14** | **Selection variants completo** — Marquee + Lasso + Magic Wand (color-based) + Group Select. Knife + Bucket tools. Text on path. | All selection modes funcionam; knife corta segments preserving continuity; text on path com parley. |
| **W15** | **Stroke Studio + Fill Studio + Tool Studio** painéis docados — full Brush Studio (Painter pattern) editor para customizar stroke / fill / tool curves. | Power user customiza Spiro tension, Hobby weight, pressure curve per-tool; save como `.ph2d-vector-tool` preset. |
| **W16** ✨ | **Vector Runtime crate `ph2d-vector-runtime` + Dynamic Physics Colliders** (§8.5, ADR-0063 + integration ADR-0067 se vier amendment Rapier) — subset runnable em release de jogo; state machine; bones + mesh deform; opt-in determinism; **Rapier 2D 0.28 collider gen automático da VectorNetwork + dynamic split em runtime boolean cut** (Proposta 4 Antigravity). | Smoke: game-shell desktop carrega `.ph2d-vector` asset com state machine; ECS dispara state transition; render WYSIWYG vs editor; **espada gameplay corta tábua vetor → collider Rapier divide em 2 corpos rígidos com momento preservado**; CI cross-OS hash test passa. |
| **W17** | **Multi-plataforma input** — iPad Apple Pencil (predict+reconcile sub-9 ms), Wacom hover, Pencil Pro squeeze/barrel-roll, Android S Pen. | Smoke do Enio em iPad Pro: Pencil 2 funciona, hover preview funciona, latência subjetiva indistinguível de Procreate. |
| **W18** | **Export interop** — SVG export (round-trip lossless v1.0 subset), PDF export (paths + gradients), AI import (lossy via PDF subset), `.ph2d-vector` v1 FREEZE com migrator (HR-14). | SVG output abre em browser idêntico; AI file de teste importa com layers + paths preservados. |
| **W19** | **Animation export** — GIF / APNG / MP4 (via `ph2d-imageio-*`) / Lottie subset / `.ph2d-vector-anim`. | Animation 2-seg exporta em todos formatos; Lottie roda em After Effects básico. |
| **W20** | **Final polish + bug bash + perf tuning + i18n bundles complete + a11y review + WCAG audit**. | Vector Module v1.0 declarado. Memory/perf budgets bate em CI baseline. Todos widgets emitem `Node`. Strings 100% i18n. WCAG 2.2 AA verde. |

**Frequência de FREEZE:**
- **W0** congela 9 contratos (ADRs 0056..0064).
- **W2** congela schema de `.ph2d-vector` v1 (HR-14: campo `version: u32`).
- **W3** congela `NodeOp` domain `vector` (caps no `architecture_contract_surface`).
- **W7** congela diffusion curve schema (Poisson solver + UI authoring).
- **W12** congela Painter bridge ABI.
- **W16** congela `ph2d-vector-runtime` API pública.
- **W18** congela export interop matrix.

---

## 10. ADRs a aprovar antes de W1 (13 total)

| ADR | Título | Conteúdo |
|-----|--------|----------|
| **ADR-0056** | Vector Network data model | VectorNetwork struct (vertices / segments / regions); **Bézier cúbico como representação default visível** (resolve crítica D Antigravity); Spiro/hyperbezier como Assist Modes; cubic export; edit log event-sourced; postcard schema versionado (HR-14). |
| **ADR-0057** | Vector edit dispatch + CRDT data model (LWW vs RGA vs custom) | Decidir se `EditorAction::VectorOp(VectorOp)` é variant novo (cap-bump) ou cabe em `ToolPanelEvent`. **CRDT estruturando `edit_log`** (LWW-Element-Set OR RGA OR CRDT custom) desde W1 — destranca multi-agente local agent ↔ designer (Proposta 5 Antigravity). |
| **ADR-0058** | Vector geometry graph (domain `vector` no `ph2d-nodegraph`) | `NodeOp` para boolean/offset/etc.; caps `NodeOp=2/OpResolver=1/NodeManifest=8` continuam válidos OR cap-bump (ADR-0039 amendment se sim). |
| **ADR-0059** | Vector renderer pipeline | Vello integration; GPU stroke expansion; Linesweeper boolean offline-cached; pipeline boolean **draft + reconcile** (3 modos selectivos: naive CPU draft / SDF GPU real-time / Linesweeper exato async — resolve crítica C Antigravity); frame budget 3.5 ms. |
| **ADR-0060** | Procedural fill shader graph | `ph2d-vector-fill` DAG → WGSL codegen; **topologia compilada 1× + UBO update por frame** (resolve crítica B Antigravity, sem compile-stutter on animate); cache strategy on-disk; diffusion curve Poisson PDE solver. |
| **ADR-0061** | Vector LLM authoring (MCP tools + LLM4SVG) | `vector-llm-shape` node; semantic tokens; HR-10 + HR-11 governance. |
| **ADR-0062** | Painter ↔ Vector bridge | API entre `ph2d-painter-brush` e `ph2d-node-vector-pattern-along-path`; paint-into-vector pipeline; auto-trace ML node. |
| **ADR-0063** | Vector runtime (game shipping) + Dynamic Physics Colliders | `ph2d-vector-runtime` crate; state machine; bones; mesh deform; opt-in determinism (HR-5 + ADR-0021 SimWorld); **Rapier 2D collider gen + dynamic split em runtime boolean cut** (Proposta 4 Antigravity); momentum preservation policy; mass derivada de `area_region × material.density`. |
| **ADR-0064** | Vector multi-platform input (Pencil/Wacom/S Pen) | Predict+reconcile sub-9 ms loop; `PlatformHost::pencil_predicted_touches()` extension; per-device feature detect. |
| **ADR-0065** ✨ | **Vector-SDF Hybrid GPU Pipeline** (Proposta 1 Antigravity) | SDF resolution (default 2× canvas DPI); compute shader algorithm (`min(d1, d2)` união, `max(d1, -d2)` corte, etc.); ordering of reductions; determinism opt-in policy (fixed SDF resolution + ordered reductions); fallback graceful para Linesweeper se compute unavailable. |
| **ADR-0066** ✨ | **Variable Font Glyph as Vector Network** (Proposta 3 Antigravity) | Glifo individual = vector network nativo; eixos OTF (weight / width / slant / optical / GRAD / custom axes) expostos como params do graph; trait `VariableFontAxis`; render path via skrifa + Vello sem rasterizar; animation hook via curves + Luau; HR-15 i18n locale-aware font fallback. |
| **ADR-0067** ✨ | **`ph2d-brush-traits` decoupling crate** (Antigravity L6F2 2ª iteração) | Quebra potencial circular dep Painter↔Vector. Crate foundational expõe `BrushRef`, `StampSpec`, `BrushEngine` trait; importável linearly por `ph2d-painter-brush` (impl) E `ph2d-node-vector-pattern-along-path` (consume). Pattern espelhado de `ph2d-vector-traits` (mocks foundation). |
| **ADR-0068** ✨ | **DeviceTier Mobile Core (<12 MB Rival Rive)** (Antigravity L5F4 2ª iteração) | Tier System Vector Module espelhando Painter ADR-0053 (Cross-platform tier policy). 5 tiers: Heavy / Standard / Lite / **Mobile Core (<12 MB VRAM rival Rive)** / Web. Mobile Core variant tree-shakes caches robustos + shader graph compiler + fluid sim + diffusion curves; preserves apenas state machine + bones + LOD + SDF rendering + solid/gradient fills. Asset cooked Mobile Core pré-renderiza diffusion curves e shaders complex para texture atlas estático. |

**Próximos passos concretos (ordem):**
1. **Spec review com Enio** — pegar feedback sobre escopo IN/OUT, número de waves, eixos.
2. **Critique adversarial via outra LLM** — prompt no §12 deste doc.
3. **Iteração padrão-ouro** — pós-critique, 2-3 rounds de audit multi-agente (vide [feedback-audit-lens-diversity](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md)).
4. **Os 11 ADRs (0056–0066) escritos e aprovados** — espelha Painter W0 (11 ADRs Accepted via 4 audits adversariais).
5. **W1 começa**: neck `ph2d-vector-doc` + Vello integration + `vector-pen` tool + smoke "click 3 points = closed triangle".

**ADR Amendments policy (Antigravity 3ª iteração L6F2 2026-05-29)**: ADRs ratificadas em W0 são **congeladas**. Alterações pós-W0 NÃO editam o ADR original; criam **amendment doc** numerado: `0056-amendment-1.md` (próximo seria `-2.md`, etc.). Amendments referenciam ADR base + estabelecem o que muda + status próprio (Proposed → Accepted). Vantagens: histórico de decisão preserved; engenheiros 2031 leem ADR + amendments sequencialmente para understand evolution. Pattern espelha Painter (informal pre-W0; agora explicit canon).

---

## 11. Índice dos arquivos do spec (a criar conforme W0 fechar)

| Arquivo | Conteúdo |
|---------|----------|
| `01_data_model.md` | Vector Network topology (Figma model), Spiro/hyperbezier authoring, cubic export, edit log event-sourced, `.ph2d-vector` schema. |
| `02_geometry_graph.md` | Domain `vector` no `ph2d-nodegraph`; lista canônica de 17 nodes v1.0; modifier stack pattern; Cavalry-inspired generators/modifiers/behaviors/falloffs/duplicators. |
| `03_renderer.md` | Vello integration; GPU stroke expansion; Linesweeper boolean; frame budget 3.5 ms; cache by hash. |
| `04_tools.md` | Pen / Pencil / Shape / Select / Direct Select / Knife / Bucket / Symbol / Text on Path / Eyedropper. Drop-crate fan-out. |
| `05_procedural_fill.md` | Shader graph 2D (Blender-style); WGSL codegen; diffusion curves; gradient meshes unified via Poisson. |
| `06_animation.md` | Toda param animável (Houdini paradigm); state machine (Rive-style); timeline panel; curve editor. |
| `07_motion_integration.md` | Domain `motion` driving vector params + reverse; cascading determinism. |
| `08_painter_bridge.md` | Paint-into-vector; vector-with-brush-look; auto-trace ML; bidirectional flow. |
| `09_scripting_mcp.md` | Cada node `#[lua_export]`; custom modifier em Luau; MCP tools `vector_*`; governance HR-11. |
| `10_runtime_gameplay.md` | `ph2d-vector-runtime` crate; state machine; bones + mesh deform; opt-in determinism; memory budget. |
| `11_pencil_pipeline.md` | Apple Pencil / Wacom / S Pen / mouse; predict+reconcile sub-9 ms; pressure curves; palm rejection. |
| `12_ux_chrome.md` | Layout 4-zonas; Geometry Graph panel; Inspector; Tool Studio; HUD; Zen Mode; atalhos Blender-style. |
| `13_fora_de_escopo.md` | Detalhe das OUT-lists (§4) com razões. |
| `14_inovacoes_extraordinarias.md` | Detalhe técnico das 5 inovações (§8). |
| `15_estado_da_arte.md` | Pesquisa completa de Illustrator/Affinity/Figma/Cavalry/Curve/Rive/Cuttle/Inkscape/Blender GP/Houdini + papers (Vello / Linesweeper / GPU stroke / Spiro / Hobby / diffusion curves / unified Poisson / variable fonts / ML vectorization). |
| `16_referencias.md` | Bibliografia consolidada com links canônicos. |
| `17_plano_de_implementacao.md` | Plano executável: 20 waves × N tasks granulares (T-W.N) com critérios concretos. **Fonte de verdade da implementação** — começar aqui ao executar. |

---

## 11.B Inovações extraordinárias absorvidas (2026-05-27, expansão do spec)

Após análise do doc [`avaliacao_e_melhorias.md`](avaliacao_e_melhorias.md) (Antigravity / Google DeepMind) com **5 críticas técnicas (A-E) + 5 propostas extraordinárias**, Enio decidiu **absorver integralmente** — mandato padrão-ouro absoluto. Detalhe técnico em [14_inovacoes_extraordinarias.md](14_inovacoes_extraordinarias.md). Resumo:

### Críticas técnicas absorvidas

| # | Crítica | Decisão | Onde se aplica | ADR |
|---|---------|---------|----------------|-----|
| **A** | Crate bloat (~40 crates) | **Aceito parcial** — consolidação seletiva (5 primitives source → 1 crate multi-variant); mantém drop-crate fan-out como espinha (DIRETRIZ §3.A). Total: ~30-32 crates. | §7.1 | — |
| **B** | Compile stutter de WGSL on-animate | **Aceito integral** — topologia compilada 1× + UBO update por frame; topology change off-thread com swap atômico. | §3.5 | ADR-0060 |
| **C** | Linesweeper síncrono no hot-path ProMotion sub-9 ms | **Aceito integral** — pipeline boolean draft + reconcile (3 modos): draft naive CPU / SDF GPU real-time / Linesweeper exato async. | §3.3 | ADR-0059 + ADR-0065 |
| **D** | Pen Tool Spiro default = rejeição profissional | **Aceito com ajuste** — Bézier cúbico como default visível; Spiro / hyperbezier como Assist Modes (toggle HUD S/H). | §3.4 | ADR-0056 |
| **E** | Vaporware coupling (Shader Graph / Animation System) | **Aceito integral** — traits abstratas `AttributeEvaluator` / `ProceduralFillShader` / `AnimationCurveSampler` + Mocks simples (linear interp, solid fill) em `ph2d-vector-traits` crate (W1). | §7.1 + §7.3 | — |

### Propostas extraordinárias absorvidas

| # | Proposta | Decisão | Wave | ADR |
|---|----------|---------|------|-----|
| **P1** | **Booleans Híbridos via Vector-SDF GPU** | **Aceito integral** — modo alternativo de preview + gameplay morphing; Linesweeper exato no commit (também resolve crítica C). | W5 | ADR-0065 (NEW §8.7) |
| **P2** | **LOD Vetorial dinâmico** (Bézier-aware adaptive fit) | **Aceito** — runtime aplica curve-aware simplification antes do Vello sparse-strips; threshold driven pela câmera. | W16 | ADR-0063 expansion (§3.10) |
| **P3** | **Tipografia Generativa + Variable Fonts axes como graph inputs** | **Aceito integral** — glifo = vector network nativo; eixos OTF expostos como params dinâmicos animáveis. | W10 + W11 | ADR-0066 (NEW §8.6) |
| **P4** | **Dynamic Rigid-Body Physics Vector Colliders** | **Aceito com euforia** — Rapier 2D collider gen automático + dynamic split em runtime boolean cut; sword-cut → 2 corpos rígidos com momento preservado. | W16 | ADR-0063 expansion (§8.5) |
| **P5** | **CRDT nativo edit_log (LWW-Element-Set OR RGA OR custom)** | **Aceito integral** — multi-agente local agent ↔ designer destrancado desde W1; CRDT replay determinístico cross-platform. | W1 | ADR-0057 |

### Mudanças resultantes no spec (1ª iteração)

- **Inovações extraordinárias**: 5 → **7** (§8.1..§8.7).
- **ADRs**: 9 → **11** (ADR-0056..ADR-0066).
- **Waves**: continua **20**, com ajustes pontuais em W3 (consolidação primitives), W5 (SDF GPU ativo), W10 (Variable Fonts), W16 (physics colliders).
- **OUT-list**: 3 itens migram para IN (CRDT local, colliders dinâmicos, tipografia generativa).
- **README**: ~500 → ~700 linhas pós-absorção.

---

## 11.C Crítica Antigravity 2ª iteração absorvida (2026-05-28)

Pós-1ª integração, o Vector Module recebeu **2ª rodada de auditoria adversarial** (8 lentes paralelas, 23 findings totais: 1 CRITICAL + 10 HIGH + 8 MEDIUM + 4 LOW). Absorção integral aplicada espelhando padrão da 1ª iteração. Detalhe técnico em [14_inovacoes_extraordinarias.md](14_inovacoes_extraordinarias.md).

### Críticas técnicas (2ª iteração) absorvidas

| # | Crítica (lente.finding) | Severidade | Decisão | Onde aplicado |
|---|--------------------------|------------|---------|---------------|
| **L1F4** | `AttributeEvaluator` retorna `f32` (quebra W10+ retroativamente) | **CRITICAL** | **ACEITO INTEGRAL** — corrigir trait W1 para `AnimValue` typed enum {Float, Vec2, Vec3, Color, Bool, Enum} | §01 §1.11.1 + §17 T1.1 |
| **L1F1+L3F1** | 30-32 crates declarado vs 40 listado real | HIGH | **ACEITO** — consolidação real seletiva (4 merges) para 32 reais. Lista atualizada em §17 §24. | §17 §24 + README §7.1 |
| **L1F2** | Compile stutter em enum control (NoiseKind switching) | HIGH | **ACEITO INTEGRAL** — codegen com switch interno + enum value via UBO. Gate `procedural_fill_enum_change_no_recompile`. | §05 §5.4.4 |
| **L1F3** | Hobby na hot-path stylus | MEDIUM | **ACEITO PARCIAL** — Hobby incremental fit em hot-path; full re-fit async em commit. | §04 §4.2.2 |
| **L1F5** | Collider split temporal gap (visual <0.5ms vs Rapier ~50ms) | HIGH | **ACEITO INTEGRAL** — 3-tier pipeline (Tier 0 Dormant Fracture, Tier 1 CPU fast-slice sub-ms, Tier 2 Linesweeper async). | §10 §10.9.4 |
| **L1F6** | CRDT testing limitado (5 fixtures) | MEDIUM | **ACEITO INTEGRAL** — proptest 256+ random cases obrigatório. Gate `vector_crdt_proptest_convergence`. | §01 §1.5.4 |
| **L2F1** | Sparse strips misattribution (CPU, não GPU) | HIGH | **ACEITO — EU ALUCINEI** — corrigido: Vello GPU = prefix-sum; sparse strips = Vello CPU. | §03 §3.1.1 + §15 §15.2.1 |
| **L2F2** | Vello 0.8 / wgpu 28 vs upstream 0.9 / 29 | MEDIUM | **REJEITO PARCIAL** — pin deliberado SKILL_Stack §5; nota explícita "upgrade plano W18 FREEZE". | §03 §3.1.1 (nota) |
| **L2F3** | Linesweeper deterministic mode não existe na API | MEDIUM | **ACEITO INTEGRAL** — determinismo é app-layer (Q16.16 pré-ordenar + ordered reductions PH2D side). | §03 §3.3.4 |
| **L3F3** | Task count 102 vs 140 summary | LOW | **ACEITO** — corrigido para 102. | §17 fim |
| **L4F1** | WoS Poisson 64spp @ 1080p inviable mobile | HIGH | **ACEITO INTEGRAL** — tier-aware resolution (Heavy 1080p/64spp; Mobile Core CPU multigrid fallback) + bilateral filter upscale. | §05 §5.6.5 |
| **L4F2** | Sub-9ms ProMotion via Bevy ECS inviável | HIGH | **ACEITO INTEGRAL** — 2-mode pipeline (Modo A Bevy/wgpu standard sub-12ms / Modo B Metal Direct Overlay sub-9ms M-series). | §11 §11.3.3 + ADR-0064 amend |
| **L4F3** | Topology change Luau hot-path | MEDIUM | **ACEITO** — restringir Luau gameplay a UBO mutations; topology change apenas editor. | §05 §5.4.5 (policy) |
| **L5F1** | Persona Illustrator pro rejeita Geometry Graph | MEDIUM | **ACEITO INTEGRAL** — Pathfinder Studio UX layer (botões clássicos sobre graph silently). | §12 §12.1.3 |
| **L5F2** | Trim Path missing (motion designers ex-AE/Cavalry/Rive) | HIGH | **ACEITO INTEGRAL** — `vector-trim-path` 18º node. | §02 §2.2.16-bis |
| **L5F3** | Hover preview limitado | MEDIUM | **ACEITO INTEGRAL** — elipse oriented por tilt/azimuth/pressure. | §11 §11.6.2 |
| **L5F4** | 80MB mobile vs Rive <10MB | HIGH | **ACEITO INTEGRAL** — Tier System com Mobile Core <12MB rival Rive. | §10 §10.10 + ADR-0068 |
| **L6F2** | Dependência circular Painter↔Vector | HIGH | **ACEITO INTEGRAL** — `ph2d-brush-traits` crate desacoplado. | README §7.1 + §17 §24 + ADR-0067 |
| **L8F1** | Audit criteria vagos | MEDIUM | **ACEITO INTEGRAL** — critérios quantificáveis (dhat-rs, p99 latency, 500+ ops stress). | §17 §0.8 |
| **L8F2** | Wave 7 21 dias otimista | HIGH | **ACEITO** — aumentado 35 dias + CPU multigrid fallback prototype. | §17 §10 (W7) |

### Propostas extraordinárias (2ª iteração) absorvidas

| # | Proposta (lente.finding) | Decisão | Wave | ADR |
|---|---------------------------|---------|------|-----|
| **L7F1** | **Dormant Fracture Edges** (pré-compute Voronoi/breakaway no editor; activate sub-µs runtime) | **ACEITO COM EUFORIA** — vira **Inovação #8** (§8.8). Combina com §8.5 split runtime e RESOLVE temporal gap L1F5. | W16 expandida | ADR-0063 amend |
| **L7F2** | Neural shader compute (mini-UNet diffusion curves) | **ACEITO COMO V2.0 STRETCH** — documentar future direction; ML model embed ~50MB não cabe v1.0. | V2.0 future | — |
| **L7F3** | 2.5D Parallax via stylus pressure (vertex depth → normal map) | **ACEITO** — opt-in feature em W17+. | W17+ | — |
| **L7F4** | Haptic Path Feedback (curvature derivatives → vibration) | **ACEITO** — Pencil Pro tem haptics; adicionar `HapticElement` API leve. | W17 | — |

### Mudanças resultantes (2ª iteração)

- **Inovações extraordinárias**: 7 → **8** (§8.1..§8.8; acrescida §8.8 Dormant Fracture Edges).
- **ADRs**: 11 → **13** (acrescidos ADR-0067 `ph2d-brush-traits` + ADR-0068 Mobile Core tier).
- **Crates**: 40 (listado real) → **32 reais** (4 consolidações + 1 brush-traits + 1 trim-path).
- **Nodes geométricos**: 17 → **18** (acrescido `vector-trim-path`).
- **Waves**: continua **20**; Wave 7 estimate 21 → 35 dias + CPU multigrid prototype; W16 expandida com Dormant Fracture pipeline.
- **README**: ~700 → ~900 linhas pós-2ª absorção.
- **Total spec**: ~8650 → ~10500 linhas pós-2ª absorção.

### Verdict da 2ª iteração

**ACEITO INTEGRAL** com 1 rejeição parcial (L2F2 — Vello 0.9 upgrade): pin deliberado preserved até W18 FREEZE event. Spec amadureceu de **v2 (pós-1ª iteração) → v3 (pós-2ª iteração)**. Pronto para **3ª iteração de audit** (recomendado mas opcional) OU ratificação Enio + abertura W1.

---

## 11.D Crítica Antigravity 3ª iteração absorvida (2026-05-29)

Pós-2ª integração, Vector Module recebeu **3ª rodada de auditoria adversarial** com **lentes rotacionadas** (rotação canônica per memory `feedback-audit-lens-diversity` — não repetir lentes anteriores). 19 findings totais: **0 CRITICAL** + 13 HIGH + 6 MEDIUM + 0 LOW. **CONVERGENCE INDEX 9.2/10** (Painter ratificou em 9.0). **ENDORSEMENT 9.8/10** (Antigravity). **GO CONDICIONADO** com 3 emendas críticas (todas absorvidas integralmente abaixo + 16 outras).

### Críticas técnicas (3ª iteração) absorvidas

| # | Crítica (lente.finding) | Severidade | Decisão | Onde aplicado |
|---|--------------------------|------------|---------|---------------|
| **L1F1** | `t: f32` perde precision sessões >4h ProMotion | MEDIUM | **ACEITO** — `t: f64` simpler que TimeContext struct para v1.0; TimeContext typed documentado future V2.0. | §17 T1.1 + §01 §1.11 |
| **L1F2** | Cargo.toml lock contention em crates consolidadas | HIGH | **ACEITO** — policy `git-stage-guard.sh` ext + `CARGO_LOCK_POLICY.md`. | §17 §24 |
| **L1F3** | Tier 1 fast-slice falha para dynamic concave sem cached decomp | HIGH | **ACEITO INTEGRAL** — outer convex hull approximation fallback sub-ms + async exact reconcile. | §10 §10.9.4 |
| **L1F4** | Bilateral filter 21×21 kernel @ 1080p estoura 0.3ms budget | HIGH | **ACEITO INTEGRAL** — JBU multi-pass (3×3 low-res denoise + 3×3 guided upscale). | §05 §5.6.5 |
| **L1F5** | Metal Direct Overlay quebra ADR-0020 surface lifecycle | HIGH | **ACEITO INTEGRAL** — PlatformHost::register_metal_overlay() + Metal Shared Events; documentado em ADR-0020-amendment-1.md. | §11 §11.3.3 |
| **L1F6** | Mobile Core crash em asset dinâmico com unavailable features | MEDIUM | **ACEITO INTEGRAL** — `degrade_fills_to_solid_avg` runtime fallback graceful + editor build-time validator. | §10 §10.10 |
| **L2F1** | Windows MAX_PATH 260-char + UNIX-style cache path | HIGH | **ACEITO INTEGRAL** — `directories` crate cross-platform + UNC paths `\\?\`. | §05 §5.5.1 |
| **L2F2** | Linux multi-arch SIMD non-determinism em Vello CPU | HIGH | **ACEITO** — disable auto-vec em deterministic profile + integer-only deterministic resolver. | §03 §3.3.4 |
| **L2F3** | Shell iPad não existe; sub-9ms Pencil pipeline bloqueado | **CRITICAL** | **ACEITO INTEGRAL CRITICAL** — T0.14 shell iPad scaffold como pre-W1 task. | §17 §3 T0.14 |
| **L3F1** | Fuzz testing missing (WGSL + LLM parser) | HIGH | **ACEITO INTEGRAL** — T13.5 cargo-fuzz targets + daily CI. | §17 T13.5 + §25 gates |
| **L3F2** | Criterion perf regression gate missing | MEDIUM | **ACEITO** — `vector_criterion_perf_regression` gate em §25. | §17 §25 |
| **L3F3** | A11y testing apenas presence de nodes (não functional) | MEDIUM | **ACEITO** — `vector_a11y_functional_traversal` gate; traversal smoke screen reader. | §17 §25 |
| **L4F1** | LLM token injection (OOM/infinite loop via params) | HIGH | **ACEITO INTEGRAL** — sanitizer com bounds rigorosos pré-alocação. | §09 §9.5.2-bis |
| **L4F2** | Postcard deser unsafe (heap overflow malicious file) | HIGH | **ACEITO INTEGRAL** — bounded_decode + size caps + adversarial fixtures. | §01 §1.6.5 |
| **L4F3** | CRDT timestamp forgery attack | MEDIUM | **ACEITO** — validation window 30s vs SimWorld clock + clamp. | §01 §1.5.3-bis |
| **L5F1** | Timeline a11y semantic readout missing | MEDIUM | **ACEITO** — auto-description generator + AccessKit nodes Timeline/Track/Keyframe/StateGraph. | §06 §6.7-bis |
| **L5F2** | Reduced Motion runtime filter missing | MEDIUM | **ACEITO INTEGRAL** — `VectorRuntime::tick` consulta `PlatformHost::reduced_motion_active()`; snap immediate. | §10 §10.2.3-bis |
| **L5F3** | Geometry Graph keyboard nav missing | HIGH | **ACEITO INTEGRAL** — Tab/Shift+Tab/Ctrl+Arrow/Enter/Edge creation mode + AccessKit announcements. | §12 §12.2.5 |
| **L6F1** | Vello spread em multi-crates → upgrade cost | HIGH | **ACEITO INTEGRAL** — Vello/kurbo/peniko encapsulado em `ph2d-vector` ÚNICO; outros consomem PH2D domain types; arch-gate `vello_kurbo_only_in_ph2d_vector`. | README §7.1 |
| **L6F2** | ADR amendments policy missing | MEDIUM | **ACEITO** — pattern `NNNN-amendment-N.md` canônico. | README §10 (próximos passos) |
| **L6F3** | LLM4SVG spec obsolescence (hardcoded tokens) | MEDIUM | **ACEITO** — JSON Schema em `ph2d-vector-llm/resources/`; injected dinamicamente em MCP context. | §09 §9.5.8-bis |
| **L7F1** | wgpu DeviceLost = data loss + editor freeze | HIGH | **ACEITO INTEGRAL** — emergency edit_log save + Vello CPU SIMD fallback. | §03 §3.9-bis |
| **L7F2** | LLM timeout = UI block infinito | HIGH | **ACEITO INTEGRAL** — 15s hard timeout + graceful cache fallback + UI toast. | §09 §9.5.7 |
| **L7F3** | CRDT silent divergence em multi-agent | HIGH | **ACEITO** — periodic blake3 integrity check (30s interval) + rollback to LCS. | §01 §1.5.3-bis |
| **L8F1** | Pathfinder Studio + manual edit divergence silent | HIGH | **ACEITO v1.0** — observable hint + "Open Geometry Graph" CTA; full bidirectional validation V2.0. | §12 §12.1.4 |
| **L8F2** | Mobile Core asset compiler missing static validator | HIGH | **ACEITO INTEGRAL** — editor build-time validator + `vector_mobile_core_asset_compat` gate. | §10 §10.10 + §17 §25 |

### Verdict da 3ª iteração

**ACEITO INTEGRAL** das 19 findings (paridade Painter 4-iter cascade). 0 CRITICAL na entrada vs 1 na 2ª iteração; convergência clara. Spec amadureceu de **v3 → v4** com:
- **8 novos arch-gates CI** (fuzz WGSL + fuzz LLM + fuzz postcard + criterion perf + a11y functional + Linux multiarch determinism + Mobile Core asset compat + Metal overlay no flicker).
- **CRITICAL T0.14 shell iPad** pre-W1 destranca cross-platform.
- **5 amendments ADR pattern formalizado** (ADR-0020-amendment-1, ADR-0050-amendment-N, etc.).
- **Security hardening completo** (sanitizer + bounded deser + timestamp validation + integrity check).

**CONVERGENCE INDEX projetado pós-3ª iteração**: ~9.7/10. **ENDORSEMENT 9.8/10** (Antigravity 3ª iter).

**Recomendação**: ratificar 13 ADRs + 1-2 amendments + abrir W1. 4ª iteração opcional (diminishing returns acima de 9.5).

---

## 12. Prompt de crítica adversarial — chame outra LLM

> Cole o bloco abaixo numa LLM forte (Gemini 2.5 Pro, GPT-5, Claude Opus 4.X de outro contexto, Grok 4). O objetivo é **busca do padrão-ouro absoluto** — adversarial review que encontre alucinação, lacuna técnica, gap de ambição, riscos arquiteturais não-considerados, alternativa superior abandonada por inércia, e oportunidade de inovação que nem este spec capturou.

```text
═══════════════════════════════════════════════════════════════════
PROMPT — CRÍTICA ADVERSARIAL DA SPEC DO "VECTOR MODULE" PH2D
═══════════════════════════════════════════════════════════════════

CONTEXTO

Você é um arquiteto sênior de software gráfico 2D e motion graphics,
com profundo conhecimento do estado-da-arte de ferramentas vetoriais
(Illustrator, Affinity Designer, Figma, Cavalry, Linearity Curve,
Rive, Cuttle, Inkscape, Blender Grease Pencil, Houdini SOPs) e
pesquisa moderna (Vello/Linebender, Linesweeper, GPU-friendly stroke
expansion 2024, Spiro/hyperbezier, Hobby's algorithm, diffusion
curves, Unified Smooth Vector Graphics Poisson 2024, LLM4SVG,
differentiable variable fonts 2025).

Você está avaliando o spec inicial (W0) de um módulo de arte vetorial
("Vector Module") para a game engine PH2D — uma engine 2D em Rust com
renderer Vello/wgpu, projetada para superar Godot/Unity em 2D e tendo
LLM como first-class user (MCP). PH2D já tem um Painter (sucessor do
Procreate) em construção (W0 ratificado, 11 ADRs aprovados, W1.T1.5
em andamento). O Vector Module pretende ser **o sucessor do Illustrator**
mas integrado à game engine: nodes geométricos, runtime de jogo,
shader fills procedurais, animação first-class, bridges com Painter,
motion nodes (Cavalry-style) e Luau scripting. Multi-platform desde
W1 (desktop, iPad com Apple Pencil, Android com S Pen, web).

O spec está em [doc canônico anexo / colado abaixo].

MANDATO ABSOLUTO

Padrão-ouro. Tempo é custo aceito; entrega medíocre é inaceitável.
A ferramenta deve **superar Adobe Illustrator** em capacidade, UX,
performance, e potencial artístico, com peso adicional de integração
total à game engine. Você está procurando **aquilo que nem o autor
do spec viu**.

SUA TAREFA — 6 LENTES ADVERSARIAIS PARALELAS

Aplique CADA lente independentemente. Não combine. Não suavize.
Adversarial = procure falha; se não achar, diga "não achei" — não
invente para preencher quota.

LENTE 1 — ALUCINAÇÃO TÉCNICA / FATOS INVERIFICÁVEIS
─────────────────────────────────────────────────
Para CADA claim técnico no spec (papers citados, crates externos,
features de tools, performance numbers, "Vello faz X", "Linesweeper
robusto em degenerate cases", "diffusion curves resolvíveis em real-
time em 2026"):
 • Verifique se o fato é real (cargo search, web search, arxiv ID).
 • Aponte claim que parece soundbite mas não tem fonte rastreável.
 • Aponte versão errada (e.g., "Vello 0.8" — confira está mesmo em 0.8).
 • Aponte feature que NÃO EXISTE em Vello/Linesweeper/etc. e foi
   assumida (e.g., "Vello suporta X" — Vello suporta?).
 • Aponte ADR fantasma referenciado mas que não existe ainda.

LENTE 2 — GAP DE AMBIÇÃO / O QUE NEM O SPEC OUSOU
─────────────────────────────────────────────────
O spec promete superar Illustrator. Onde ele se assemelha demais a
"Illustrator + nodes"? Que oportunidade radical o autor abandonou
por inércia ou pelo conforto de imitar tools existentes?
 • Vector AI generativo end-to-end (não só LLM-as-graph-node) —
   considerou? Por que descartou ou por que não chegou na ambição
   plena?
 • Multiplayer co-edit real-time CRDT (Figma model em open-source) —
   é OUT de v1.0. Justificado? Ou medo arquitetural?
 • Vector-as-input-to-physics (rigid body / cloth / fluid sim
   recebendo vector network como colliders) — PH2D tem rapier;
   spec menciona? Falta?
 • Generative procedural symbol library (1000 simbolos parametricos
   pre-treinados, biblioteca embutida estilo Adobe Stock mas live-
   parametric) — considerou?
 • Vector com profundidade Z (2.5D auto-parallax driven by node) —
   tipo Toonboom Storyboard Pro?
 • Algo que VOCÊ acharia o "moonshot extraordinário"?

LENTE 3 — ARQUITETURA / TRADE-OFFS / RISCO TÉCNICO
─────────────────────────────────────────────────
 • Tamanho da família de crates (~40 crates novos): viable
   mentalmente? Build time? Workspace bloat?
 • Vector network mutável + edit log event-sourced + CRDT-ready
   simultaneamente — é prematuro? Pode escolher só 1?
 • Sub-9 ms ProMotion latência: o spec promete mas predict+reconcile
   é caro. Reservou frame budget suficiente? Mediu hardware-em-mão?
 • Live boolean graph com Linesweeper: perf é viável quando o graph
   tem 50+ nodes com boolean encadeado? Spec menciona cache by hash
   mas isso é hand-wave. Sustenta um teste?
 • Vello 0.8 "alpha churn per quarter" (vide ADR-0004) — spec assume
   wgpu 28 + Vello 0.8 estáveis por anos. Plano B se Vello quebra?
 • SimWorld determinismo de boolean: linesweeper é determinístico
   bit-identical cross-platform (Win/Mac/Linux/iOS/Android)? Verifi-
   cou ou é wishful thinking?
 • 20 waves: cronograma realista ou marketing? Painter teve 17.
 • Shader graph compilation hot-path: cache by hash mas se o usuário
   anima param em curva = N shaders por frame? Plano?

LENTE 4 — UX / WORKFLOW DE ARTISTA
─────────────────────────────────────────────────
Imagine 3 personas: (a) artista vetorial profissional vindo do
Illustrator; (b) motion designer vindo do After Effects + Cavalry;
(c) artista digital vindo do Procreate; (d) game dev vindo do Rive.
Para cada:
 • Spec mostra workflow concreto fim-a-fim? Ou só features avulsas?
 • Onde Vector Module ainda é PIOR que a ferramenta de origem do
   artista? (Ser honesto. Illustrator vence em quê?)
 • "1 caminho canônico por feature" (filtro minimalista-Blender) —
   ferra o profissional que valoriza alternativas?
 • Pen tool com Spiro authoring rep: artista veterano de Illustrator
   vai aprender uma nova "physics" de pen tool? Ou vai detestar?
 • Geometry Graph é poder + cognitive load. Sub-painel docado é
   suficiente OU vai assustar a maioria? Onde escondê-lo melhor?
 • LLM-as-graph-node: artista quer? Ou é vantagem do marketing que
   o artista nunca usa?

LENTE 5 — INTEGRAÇÕES PROMETIDAS — REALIDADE OU VAPOR
─────────────────────────────────────────────────
Spec promete integração com:
 • Painter (em desenvolvimento, W1.T1.5)
 • Motion nodes (W2 fechada, fan-out aberto)
 • Shader nodes (NÃO EXISTE ainda em PH2D!)
 • Animation system (NÃO EXISTE ainda!)
 • Luau gameplay (M7 implementado, mas vector specifics?)
 • MCP (M9 skeleton, vector tools são novos)
 • Game runtime determinístico (HR-5 / ADR-0021 — vetor entra como?)

Para cada integração:
 • É vapor? "Quando a outra ponta amadurecer" tem timing realista?
 • O contrato da bridge está definido OU é "TBD"?
 • Algum risco de o Vector Module fechar W1-W5 e ficar com bridges
   mortas porque outra ponta não veio?

LENTE 6 — INOVAÇÃO QUE NEM O SPEC IMAGINOU
─────────────────────────────────────────────────
Você é um arquiteto criativo. Olhe o spec inteiro e pergunte:
 • Que feature radical existe em pesquisa recente (2024–2026) que
   o spec NÃO menciona?
 • Que combinação non-obvious de duas features do spec produz uma
   3ª feature emergente que o autor não viu?
 • Que ferramenta obscura (1 estrelinha no GitHub mas brilhante) o
   autor não conhece?
 • Que feature "anti-mainstream" (radicalmente diferente, contra-
   intuitiva) o autor poderia adicionar e que nenhum competitor
   tem?

FORMATO DE RESPOSTA OBRIGATÓRIO

Para CADA lente, devolva:
  HEADING: LENTE N — <nome da lente>
  FINDINGS: numerados 1..K
    Cada finding:
      • SEVERIDADE: CRITICAL / HIGH / MEDIUM / LOW
      • CLAIM (1 frase do que está errado/faltando/melhorável)
      • EVIDENCE (citação literal do spec OR URL/source da realidade)
      • RECOMMENDATION (1-3 sentenças concretas)
  Se zero findings na lente: "FINDINGS: nenhum encontrado em <X min
  de busca>".

Depois, AO FINAL:
  TOP-3 INSIGHTS — os 3 findings que MAIS levam o spec ao padrão-
  ouro. Justifique por que esses 3.
  GAP MÁXIMO — uma frase: qual a maior fraqueza?
  AMBIÇÃO PERDIDA — uma frase: qual o moonshot que o autor não
  ousou propor?

REGRAS

 • Não invente paper / crate / fact. Cite source OU diga "não
   verificado".
 • Não suavize. "Boa ideia mas talvez..." é ruído. Diga "isso falha
   porque X". Ou diga "não achei falha".
 • Não repita o spec de volta. Achados novos apenas.
 • Não economize. Spec tem 50+ páginas; achados sub-página são bem-
   vindos.
 • Português brasileiro está OK; inglês está OK. Mas escolha um e
   mantenha.

═══════════════════════════════════════════════════════════════════
SPEC EM AVALIAÇÃO ABAIXO
═══════════════════════════════════════════════════════════════════

[Cole aqui o conteúdo de docs/Vector Module/README.md — este arquivo]
```

---

## 13. Referências canônicas (pré-bibliografia consolidada)

**Tools (state of the art):**
- Adobe Illustrator 2026 — [TechRadar review](https://www.techradar.com/pro/software-services/adobe-illustrator-2026-review) · [ExtendScript limits](https://mapsoft.com/posts/extendscript-usage.html) · [Perceptual gradients export](https://helpx.adobe.com/illustrator/desktop/paint-and-fill/create-and-edit-gradients/control-perceptual-interpolation-in-gradients.html)
- Affinity Designer — [Boolean ops tutorial](https://designbundles.net/design-school/how-to-use-boolean-operations-in-affinity-designer) · [Contour tool docs](https://www.affinity.studio/help/tools-tools-contour/)
- Figma vector networks — [Alex Harri deep-dive](https://alexharri.com/blog/vector-networks) · [Figma blog intro](https://www.figma.com/blog/introducing-vector-networks/) · [Plugin API VectorNetwork](https://developers.figma.com/docs/plugins/api/VectorNetwork/)
- Cavalry — [SuperRenders 2026 review](https://superrendersfarm.com/article/cavalry-motion-design-review-2026) · [Grokipedia entry](https://grokipedia.com/page/Cavalry_animation_software)
- Linearity Curve — [linearity.io product](https://www.linearity.io/curve/) · [Auto Trace](https://www.linearity.io/features/auto-trace/)
- Rive — [rive.app/runtimes](https://rive.app/runtimes) · [Rive renderer open-source](https://rive.app/blog/rive-renderer-now-open-source-and-available-on-all-platforms) · [Bones docs](https://help.rive.app/editor/manipulating-shapes/bones)
- Cuttle / Boxy SVG / Paragraphic — [Cuttle HN](https://news.ycombinator.com/item?id=41674677) · [Paragraphic](https://paragraphic.design/)
- Inkscape LPE — [Inkscape Manuals LPE](https://inkscape-manuals.readthedocs.io/en/latest/live-path-effects.html) · [Tavmjong Bah LPE chapter](http://tavmjong.free.fr/INKSCAPE/MANUAL/html/Paths-LivePathEffects.html)
- Blender Grease Pencil — [Blender 4.3 release notes GP](https://developer.blender.org/docs/release_notes/4.3/grease_pencil/) · [CGChannel coverage](https://www.cgchannel.com/2024/10/blender-4-3-lets-you-control-grease-pencil-with-geometry-nodes/)
- Houdini SOPs — [SideFX docs](https://www.sidefx.com/docs/houdini/nodes/sop/index.html) · [Poly Expand 2D](https://www.sidefx.com/docs/houdini/nodes/sop/polyexpand2d.html)

**Research / papers:**
- Vello — [Linebender Vello GitHub](https://github.com/linebender/vello) · [Linebender Dec 2025 blog](https://linebender.org/blog/tmil-24/) · [Sparse strips thesis ETH 2025](https://ethz.ch/content/dam/ethz/special-interest/infk/inst-pls/plf-dam/documents/StudentProjects/MasterTheses/2025-Laurenz-Thesis.pdf)
- Linesweeper — [Joe Neeman blog](https://joe.neeman.me/posts/linesweeper/) · [GitHub](https://github.com/jneem/linesweeper)
- GPU-friendly stroke expansion (Levien+Uguray 2024) — [ACM paper](https://dl.acm.org/doi/10.1145/3675390) · [arXiv](https://arxiv.org/pdf/2405.00127) · [project page](https://linebender.org/gpu-stroke-expansion-paper/)
- Spiro / Hyperbezier (Levien) — [Spiro](https://levien.com/spiro/) · [Hyperbezier blog](https://www.cmyr.net/blog/hyperbezier.html) · [Béz fitting](https://raphlinus.github.io/curves/2021/03/11/bezier-fitting.html)
- Diffusion curves — [Orzan SIGGRAPH 2008](https://dl.acm.org/doi/10.1145/1360612.1360691) · [Monte Carlo 2026](https://arxiv.org/abs/2602.05492)
- Unified Smooth Vector Graphics (Poisson 2024) — [arXiv](https://arxiv.org/pdf/2408.09211)
- Hobby's algorithm — [PGF hobby package](https://ctan.math.washington.edu/tex-archive/graphics/pgf/contrib/hobby/hobby.pdf) · [implementation walkthrough](http://hz2.org/blog/hobby_curve.html)
- Variable fonts as primitive — [Differentiable Variable Fonts arXiv 2510.07638](https://arxiv.org/html/2510.07638v1)
- ML vectorization — [StarVector](https://arxiv.org/html/2312.11556v4) · [SuperSVG](https://arxiv.org/pdf/2406.09794) · [LLM4SVG](https://ximinng.github.io/LLM4SVGProject/)

**PH2D internal:**
- [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) — HR-1..HR-18, arquitetura, stack pinado.
- [`CLAUDE.md`](../../CLAUDE.md) — workflow operacional.
- [`docs/IntegracaoMultiAgente/DIRETRIZ.md`](../IntegracaoMultiAgente/DIRETRIZ.md) — triagem, fan-out drop-crate, contratos congelados.
- [`docs/Painter_projeto/README.md`](../Painter_projeto/README.md) — Painter spec (gêmeo raster — pattern de referência).
- [`docs/HANDOFF_node_system.md`](../HANDOFF_node_system.md) — tracker fan-out de nodes.
- ADRs canônicos: [0019 Luau](../architecture/decisions/0019-spike-scripting-output.md) · [0020 Surface lifecycle](../architecture/decisions/0020-surface-lifecycle.md) · [0021 SimWorld/PresentWorld](../architecture/decisions/0021-simulation-presentation-boundary.md) · [0023 UI baseline](../architecture/decisions/0023-ui-ux-baseline.md) · [0024 Editor input](../architecture/decisions/0024-editor-input-and-widget-state.md) · [0039 Nodegraph contract freeze](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md) · [0040 Tool isolation](../architecture/decisions/0040-tool-as-isolated-feature-crate.md) · [0041 RasterEdit rename](../architecture/decisions/0041-rasteredit-rename-and-deactivate.md) · [0043..0053 Painter cascade](../architecture/decisions/).

---

**Fim do W0 study.** Próximo passo: critique adversarial via §12 + 9 ADRs (0056..0064) escritos pós-iteração. **Sem ratificação, W1 fica bloqueada** — espelhando Painter W0 freeze que destrancou W1.T0.8 (homestead arch-gate).
