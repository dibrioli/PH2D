# 15 — Estado da arte (pesquisa 2026)

> Compilação da **pesquisa de estado-da-arte** de ferramentas vetoriais 2D + papers acadêmicos modernos. Base técnica das decisões arquiteturais do Vector Module. Source survey completed 2026-05-27 via WebSearch + arxiv + technical docs.
>
> **Sumário executivo:** Vector Module se posiciona ahead do mainstream em **vector network topology** (Figma), **live boolean graph** (sem competitor mainstream), **GPU compute renderer** (Vello/Linebender), **integrated runtime para jogos** (sucessor Rive), **mesh gradients via diffusion curve** (paper Poisson unification 2024), e **LLM authoring editável** (LLM4SVG).

## 15.0 Estrutura

- §15.1 Tools mainstream (Illustrator, Affinity, Figma, Cavalry, Linearity Curve, Rive, Cuttle, Inkscape, Blender Grease Pencil, Houdini).
- §15.2 Research papers + GPU primitives.
- §15.3 Frontier of innovation (2026).
- §15.4 Gaps to exploit — opinionated.
- §15.5 Recommendations específicas para PH2D.

---

## 15.1 Tools mainstream

### 15.1.1 Adobe Illustrator — incumbent, weighed down

**What it does well:**
- **Pathfinder**: gold-standard boolean ops UX (Unite, Minus Front/Back, Intersect, Exclude, Divide, Merge, Crop, Outline, Trim). Decades of "do what I mean" tuning.
- **Mesh gradients**: ainda o most expressive smooth-shading primitive commercial.
- **Variable-width strokes** via Width Tool — interactive per-point width.
- **Art / Pattern / Bristle / Scatter brushes**: path = carrier para arbitrary art.
- **Live Effects** (Roughen, Pucker & Bloat, Zig Zag, Free Distort, Warp, Offset Path) — non-destructive mas locked into appearance panel order.
- **Recolor Artwork** — color harmony rules across illustration.
- **Blend Tool** — distribui shapes/colors between two anchors along spine.

**Where it fails:**
- **Boolean perf terrível** em complex paths; pathfinder bake-and-discard.
- **Mesh gradient hand-author painful**: hand-placing mesh patches é chore.
- **No real GPU compute path rendering** — CPU rasterization + GPU compositing on top.
- **ExtendScript awful**: ES3, single-thread, no async, freezes host UI.
- **No animation, no runtime, no graph**. Effects panel é flat list, não DAG.
- **iPad version** é permanent feature-trailing port.

### 15.1.2 Affinity Designer 2 — disciplined challenger

- **Dual personas** (Designer / Pixel / Export) — same document, switch toolset; pixel layers e vector layers coexist.
- **Compound shapes** via Alt-click boolean — true non-destructive boolean tree.
- **Contour tool** — non-destructive offset path; tweakable or removable.
- **Symbol system** — local symbols com cascading edits, sem parametric inputs.

**Failures:** no node graph, no animation, no scripting, no runtime, no GPU compute.

### 15.1.3 Figma — vector network topology reform

**The vector network is the only fundamental data-model innovation any mainstream vector tool shipped this decade.**

- **Vertex pode ter N incidências** (cubo é UM network com vertices compartilhados).
- **Minimal cycle basis algorithm** identifies fillable regions auto.
- **Graph expansion at edge crossings** auto-inserts intersection vertices.
- **Per-region fills** — multiple closed regions em one network.
- **Stroke distribution** — inside / outside / center on region boundary.

**Failures:** bezier-only (no Spiro/Hobby), boolean ainda bake, Auto Layout não fala com vector networks, plugin API stripped sandbox.

### 15.1.4 Cavalry — node graph applied to 2D motion

What After Effects would be if Adobe had hired Houdini engineers.

- **Generators** (shape, text, image) → **Modifiers** (Trim Path, Round Corners, Wiggle, Noise Deformer) → **Behaviors** (rigid body, attractor, spring) → **Falloffs** (radial, linear, noise, shape-based).
- **Duplicators** distribute shapes into grids / circles / along paths.
- **JSON data binding** — any animatable property pode sip from external feed.
- **Expressions everywhere**.

**Failures:** motion-first; static illustration UX awkward; no runtime for games.

### 15.1.5 Linearity Curve / Vectornator — iPad-native

- **Auto Trace** com 3 modos (Sketch / Illustration / Basic Shapes).
- **Pencil Tool**: freehand auto-smoothed em vector paths em real time, com Apple Pencil pressure + tilt driving width + opacity.
- **Magic Wand** para marquee-by-color selection.
- Metal-based; pencil latency rides Apple sub-9ms ProMotion path.

**Failures:** proprietary file format, no scripting, no live graph, weak Illustrator compatibility, no animation.

### 15.1.6 Rive — runtime-first vector tool

Most relevant existing reference for "game-engine vector".

- **Authoring tool + runtime + open-source renderer** em one stack.
- **State machines** com blend states para animation control from game code.
- **Bones + vertex weighting** — skeletal deformation of vector paths + image meshes.
- **Mesh deformation of both vector paths and raster textures** — hybrid model.
- **Rive Renderer**: novel triangulation that converts antialiased Bézier paths to triangle patches; targets 120fps.
- Shipping em production: Spotify, Duolingo, Disney; integrated em Unity, Unreal, Defold, Bevy.

**Failures:** authoring tool weaker que Illustrator for static work; data model mais limited que Figma's (still path-based); no live boolean graph; no procedural modifiers / falloffs / generators like Cavalry; rendering é fixed-function — no procedural shader fills.

### 15.1.7 Cuttle / Boxy SVG / Paragraphic — parametric niche

[Cuttle](https://cuttle.xyz) é production-ready example of **parameters-driving-geometry**: define numeric parameters (material thickness, count, spacing), bind throughout project, sliders update geometry live.

Built-in parametric templates show real strength: distribute customizable design to many users.

### 15.1.8 Inkscape — open-source reference + LPE catalog

**Live Path Effects** são most underappreciated non-destructive system em mainstream tools:
- **Pattern Along Path** — distort pattern to follow path.
- **Bend Path** — bend any shape along custom envelope.
- **Pattern from Nodes** — distribute pattern at every node.
- **Width Path** — per-position stroke width handles.
- **Roughen** — organic imperfection (parametric — frequency, displacement, smoothness).
- **Corner Rounding** — per-node rounding.
- **Hatch Fill** — fully parametrized hatching.
- **Spiro / Sketch / BSpline** — alternate curve interpretations.
- **Mirror Symmetry / Rotate Copies** — live symmetry.
- **Boolean** as LPE — non-destructive boolean preservando operand editability.

LPEs **stack**, and that compositional power differentiates Inkscape from Illustrator. Catch: UX genuinely 2010-era, performance falls off cliff at LPE stack depth ≥ 5, no graph view.

### 15.1.9 Blender Grease Pencil — 2D vector inside 3D pipeline

Most interesting cross-pollination: as of Blender 4.3 (Oct 2024) e matured em 4.5 LTS, **Grease Pencil works inside Geometry Nodes**.

- GP é "flat list of layers com curves"; geometry-node operations (Set Position, Resample Curve, Capture Attribute, Sample Index, Geometry Proximity) work per-layer on curve data.
- Conversion nodes go GP↔curves↔mesh.
- Modifier stack (Array, Build, Hook, Lattice, Subdivide, Texture Mapping, Tint, Time Offset, Hue/Saturation, Multiply, Noise) é all non-destructive.
- 2.5D pipelines compose GP on 3D — strokes projected to camera, attached to 3D anchors, lit, shaded.

Result: vector primitive flowing through procedural graph que rest of engine already uses. DCC industry's closest reference for unified node-graph + vector + raster + 3D + animation model.

### 15.1.10 Houdini SOPs — what "node graph for vector" looks like at scale

Houdini's SOP context already supports 2D-ish workflows: Bézier curve drawing, Poly Expand 2D para offset/inset paths, attribute-driven everything, CHOPs para channel-driven animation.

Lesson for new tool: copy Houdini's **non-destructive default**: every node has parameters, every parameter is animatable, every parameter accepts expression que references any other parameter ou attribute em scene.

### 15.1.11 Procreate — raster pretending it doesn't need vector

Counter-example. Raster-dominant; "vector" support is limited to import + rasterize. Procreate's UX é best-in-class for stylus-driven painting, but they refuse to add vector primitives — exactly the gap a new tool can exploit (paint into vector — Inovação #3).

---

## 15.2 Research papers + GPU primitives

### 15.2.1 Vello (Linebender) — vector renderer (two-layer architecture)

- **3 implementations**:
  - **Vello GPU** (compute via WebGPU) — **prefix-sum stage pipeline** (coarse → fine → ratification → fine rasterize). Primary path em todos targets com WebGPU compute.
  - **Vello CPU** (multi-threaded SIMD: SSE2/AVX/AVX2/AVX512/NEON) — **sparse strips arch** (CPU-only). Fallback path para devices sem compute support.
  - **Vello Hybrid** — combina GPU + CPU em casos específicos.
- **Sparse strips arch** ([Laurenz Stampfl ETH master thesis 2025](https://ethz.ch/content/dam/ethz/special-interest/infk/inst-pls/plf-dam/documents/StudentProjects/MasterTheses/2025-Laurenz-Thesis.pdf)): literally "High-performance 2D graphics rendering on the **CPU** using sparse strips" — run-length-compressed antialiased boundaries + sparsely represented solid interiors via Rust SIMD. **Não é arquitetura GPU.**

**Decisão PH2D**: Vello é renderer backbone (já pinado em SKILL_Stack §5 em 0.8). Pipeline GPU primário; CPU fallback graceful via sparse strips para edge cases.

### 15.2.2 Linesweeper — robust boolean ops

Joe Neeman's library ([blog](https://joe.neeman.me/posts/linesweeper/)) — modern answer to "why is path boolean broken in every tool".

Key insight: **prioritize orderings over intersections**. Two-phase sweep:
1. Approximate horizontal ordering of segments sem computing exact intersection points.
2. Approximate all segments com Béziers sharing y-coordinates, subdivide as needed.

Makes live boolean graph realistic for first time. **Decisão PH2D**: Linesweeper para exact boolean (commit path).

### 15.2.3 GPU-friendly stroke expansion (Levien + Uguray, 2024)

[ACM paper 10.1145/3675390](https://dl.acm.org/doi/10.1145/3675390): Approximates parallel-curve offset of cubic Béziers com **Euler spirals** (clothoids), depois flattens. Fully parallel — single compute shader pass.

Robust em extreme zoom, handles variable-width strokes, miter/bevel/round joins, dashes. Já integrated em Vello. **Decisão PH2D**: consume direto via Vello API.

### 15.2.4 Hyperbezier + Spiro (Raph Levien)

- **Spiro**: clothoid splines, constant-curvature joints. Used by Inkscape e font designers. Two-control-point alternative to cubic Bézier.
- **Hyperbezier**: Levien's newer family, two-control-point com elastica-under-tension behavior.
- **Béz fitting**: converting Euler spirals / hyperbeziers back to cubic Bézier for SVG export.

**Decisão PH2D**: **Bézier cúbico default** (decisão D Antigravity — paridade Illustrator); Spiro / hyperbezier como Assist Modes opt-in (vide [04 §4.1.2](04_tools.md)).

### 15.2.5 Diffusion curves

[Orzan et al. SIGGRAPH 2008](https://dl.acm.org/doi/10.1145/1360612.1360691); revived 2013-2026 via [Monte Carlo extraction (2026)](https://arxiv.org/abs/2602.05492). Curva carrega cores em ambos lados, com optional blur; Poisson PDE diffunde no resto do canvas.

**Decisão PH2D**: diffusion curves substituem mesh gradient hand-author (Inovação #2; [05 §5.6](05_procedural_fill.md)).

### 15.2.6 Unified Smooth Vector Graphics (Poisson 2024)

[arXiv 2408.09211](https://arxiv.org/pdf/2408.09211): mesh gradients e diffusion curves são **two boundary-value forms of the same PDE**. Treat as one primitive. Unifying paper que justifica diffusion curve approach.

### 15.2.7 SDF-based vector

Multi-channel SDFs ([msdfgen](https://github.com/Chlumsky/msdfgen)) generalizam Valve text technique to arbitrary 2D shapes. **Decisão PH2D**: SDF-based boolean ops via compute shader (Inovação #7, ADR-0065; vide [03 §3.4](03_renderer.md)).

### 15.2.8 Hobby's algorithm

[John Hobby's MetaPost algorithm](http://hz2.org/blog/hobby_curve.html): computes "most pleasant" cubic Bézier chain through point sequence by **minimizing total curvature variation**.

Vastly better que Catmull-Rom ou basic smoothing. **Decisão PH2D**: Hobby fitter default sob Pencil tool (no commercial vector tool ships it; Inkscape uses Schneider's, inferior).

### 15.2.9 Variable fonts as vector primitive

[Differentiable Variable Fonts (Oct 2025 arXiv 2510.07638)](https://arxiv.org/html/2510.07638v1) formaliza variable font interpolation como differentiable function. Implication: **strokes get axes**, not just width.

**Decisão PH2D**: variable font glyph = vector network nativo (Inovação #6, ADR-0066).

### 15.2.10 ML vectorization

- **[StarVector](https://arxiv.org/html/2312.11556v4)** — multimodal LLM trained on image+text → SVG code.
- **[VectorArk](https://arxiv.org/html/2605.24398)** — fine-tunes MLLMs on artist-designed vector data.
- **[SuperSVG](https://arxiv.org/pdf/2406.09794)** — first deep-learning method para complex-detail vectorization via superpixel decomposition.
- **[LLM4SVG](https://ximinng.github.io/LLM4SVGProject/)** — learnable semantic tokens encoding SVG components. **Maintains editability** (output é structured, não opaque pixels).

**Decisão PH2D**: LLM4SVG-style semantic tokens em `vector-llm-shape` node (Inovação #4, ADR-0061).

---

## 15.3 Frontier of innovation (2026)

What is possible em 2026 que wasn't em 2021:

1. **GPU-resident vector rendering em infinite zoom.** Vello + GPU-friendly stroke expansion. Linebender ships em Rust today.
2. **Non-destructive boolean ops as live graph.** Linesweeper's robustness makes boolean a real-time node.
3. **Vector ↔ raster bridges.** Paint with stylus em vector path that auto-traces em real time. Vector strokes que **look like brush strokes** via per-stroke shader fills.
4. **Procedural fills via shader nodes.** Treat fill as fragment shader; node graph builds shader. Blender's shader nodes for 2D.
5. **Runtime vector animation em games.** Rive class, but com richer authoring (vector network + procedural modifiers), procedural shader fills, determinismo cross-platform.
6. **ML-assisted vectorization, perspective correction, auto-symmetry.** SuperSVG handles complex images; perspective-aware vectorization em research today.
7. **LLM-driven authoring.** LLM4SVG semantic tokens — generate **editable structured strokes**, not "SVG dump".
8. **Stylus latency targets sub-9ms** em iPad ProMotion. Predict + reconcile.
9. **Parametric / scripted vector.** Cuttle proves market wants sliders.
10. **Variable fonts ↔ variable strokes as unified concept.** Stroke is 1D variable font: continuous axes for width, taper, contrast, jitter, pressure-response.

---

## 15.4 Gaps to exploit — opinionated

Ranked by "biggest expected payoff for new tool com Rust + node-graph + game-engine backbone":

1. **Boolean / offset / outline / stroke-to-fill as live graph nodes.** Illustrator e Figma bake; Affinity does compound shapes mas no full DAG. Linesweeper makes this finally robust. **Win condition**: every "destructive" Illustrator command becomes node with editable inputs forever. → **Inovação #1**.

2. **First-class runtime vector for games.** Rive proves market is real but path-only data model and no procedural modifiers. Vector network (Figma-class) + Cavalry-class modifier stack + Rive-class runtime + state machine = no existing competitor occupies that quadrant. → **Inovação #5**.

3. **GPU compute as only renderer.** Vello-based. Path-pure data flows to GPU each frame. Infinite zoom. 4K+ canvases em 120fps. Editor shares renderer com runtime — same WGSL pipeline draws ícone em editor e em shipped game. → **Inovação #5 (renderer shared)**.

4. **Procedural fills via shader nodes.** Fill node graph (noise, ramp, Voronoi, gradient mesh, diffusion curve, image sample, shader expression) producing procedural texture sampled by path. Outputs to WGSL. → **Inovação #2 + procedural fill spec [§05]**.

5. **Pencil-into-vector com real stylus physics.** Pressure → variable-width stroke. Tilt → asymmetric envelope. Velocity → smoothing weight. Hobby fitter on commit. Apple Pencil sub-9ms loop com predict + reconcile. → **Inovação #3 (Painter ↔ Vector bridge) + [§11]**.

6. **Deterministic replay.** Vector edit log (CRDT or event-sourced) replays bit-identical across machines / platforms. Trivial for testing, version control, multiplayer co-edit, AI assistance traces. No commercial tool has this. → **[§01 §1.5]**.

7. **Symbol system that's parametric.** Cuttle proves it works. Symbols expose typed parameters (number, color, vector, enum); symbol instances bind via graph. Beats Figma components, beats Illustrator symbols by 10×. → **[§04 §4.8]**.

8. **Animation native, not bolted on.** Every parameter is curve em time. Rive-style state machine drives transitions. Cavalry-style behaviors and falloffs work same em editor preview and em shipped runtime. No "export to Lottie" — `.ph2d-vector-anim` file is runtime asset. → **[§06]**.

9. **Scripting that doesn't suck.** Embed Luau / WASM. Every node exposes parameters and outputs. Scripts run async, don't freeze UI, can make HTTP calls, can be hot-reloaded. Bar to clear é ExtendScript. → **[§09]**.

10. **Mesh gradients done right.** Author from single shape by clicking color points on boundary + interior, solve Poisson on GPU as diffusion curve. Math é one PDE; user never sees mesh. → **Inovação #2 + [§05 §5.6]**.

11. **iPad-class touch UX.** Not port. Same `wgpu` renderer runs on iPad — Rust-on-iOS via Bevy/winit shipping em 2026. Apple Pencil + ProMotion é design target. → **Cross-platform principle + [§11]**.

12. **LLM-driven authoring as graph operation.** Not "generate SVG", but node `LLMShape(prompt, context, schema) → vector_network`. LLM emits structured paths typed against data model. User edit output downstream. → **Inovação #4 + [§09 §9.5]**.

13. **Vector + raster + 3D unified by node graph.** Blender Grease Pencil proves this. Curves and meshes and textures and 3D geometry interoperate via shared geometry-node semantics. Vector Module em 2D engine não needs 3D — but should let raster, vector, and procedural fills coexist as nodes. → **[§08 Painter bridge] + [§07 motion integration]**.

---

## 15.5 Recommendations específicas para PH2D Vector Module

Stack canon decidido pela pesquisa:

- **Renderer**: Vello 0.8 + GPU stroke expansion (Levien+Uguray 2024). Backend único.
- **Boolean**: Linesweeper exato (commit) + SDF GPU Hybrid (real-time, ADR-0065).
- **Authoring**: Bézier cúbico default + Spiro/Hyperbezier Assist Modes.
- **Pencil fitter**: Hobby's algorithm (minimum curvature variation).
- **Data model**: Vector Network (Figma topology) + edit log event-sourced + CRDT (LWW + RGA hybrid).
- **Procedural fill**: shader graph DAG + WGSL codegen + UBO update per frame.
- **Mesh gradient**: diffusion curve via Poisson PDE (unified com gradient mesh).
- **Runtime**: `ph2d-vector-runtime` crate ship-able + Rapier 2D physics colliders (dynamic split).
- **Variable fonts**: glyph = vector network nativo; axes como graph params.
- **LLM**: `vector-llm-shape` node + LLM4SVG semantic tokens.
- **Multi-platform**: Vello cross-backend (Metal/Vulkan/D3D12/WebGPU); predict+reconcile sub-9ms.

---

## Fim do estado da arte

Pesquisa completa. Source survey, tools mainstream surveyed, papers acadêmicos reviewed, frontier mapped, gaps identified, recommendations explícitas.

**Próxima leitura:** [`16_referencias.md`](16_referencias.md) (bibliografia consolidada com links).
