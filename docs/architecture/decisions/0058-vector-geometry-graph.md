# ADR-0058 — Vector geometry graph (domain `vector` em `ph2d-nodegraph`)

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [ADR-0039 — Nodegraph contract freeze](0039-nodegraph-contract-freeze-w2t4.md), [ADR-0056 — Vector Network data model](0056-vector-network-data-model.md), [ADR-0040 — Tool isolation](0040-tool-as-isolated-feature-crate.md).
**Spec normativa:** [`docs/Vector Module/02_geometry_graph.md`](../../Vector%20Module/02_geometry_graph.md).
**Tags:** vector, wave-0, contract, node-graph, drop-crate, fan-out

---

## 1. Contexto

O Vector Module entrega **Live Boolean Graph** (Inovação #1, §8.1 README) — toda operação que Illustrator destrói (boolean, offset, outline, contour, distort, scatter) vira **nó vivo** no `ph2d-nodegraph`. Sucessor de Cavalry/Houdini/Blender em 2D vetor.

`ph2d-nodegraph` já tem domain `motion` (ADR-0039, 3 crates Stateful: motion-grid, motion-clone, motion-transform). Vector é o **segundo domain** com fan-out drop-crate.

### 1.1 Por que domain `vector` separado

- Tipo de output canônico (`VectorNetwork`) é diferente de motion (`Vec2`/`Affine`/`Vec3`/Array).
- Cross-domain connections existem (motion → vector params, vector → motion paths input — vide ADR-0058 §2.9 cascading determinism) mas são **edges entre domains**, não fusão.
- Drop-crate fan-out (DIRETRIZ §3.A) destranca paralelismo W3+ (18 nodes em paralelo W4).

---

## 2. Decisão

### 2.1 Domain `vector` adicionado a `NodeOp` enum (sem cap-bump)

Caps ADR-0039 (`NodeOp=2 / OpResolver=1 / NodeManifest=8`) **continuam válidos**. Vector domain adiciona nodes sem mexer contrato base nodegraph.

### 2.2 18 nodes canônicos (cada um drop-crate `crates/ph2d-node-vector-<slug>/`)

Consolidação seletiva pós-Antigravity 2ª iter (5 primitives source → 1 multi-variant crate) + 1 trim-path absorvido 2ª iter:

| # | Crate | Wave | Effect | Função |
|---|-------|------|--------|--------|
| 1 | `ph2d-node-vector-source` | W3 | Pure | 5 primitives multi-variant (rect/ellipse/poly/star/spiral) |
| 2 | `ph2d-node-vector-boolean` | W3 | Stateful | 9 boolean variants + draft+reconcile pipeline |
| 3 | `ph2d-node-vector-offset` | W3 | Pure | Parallel/contour live (Euler-spiral GPU) |
| 4 | `ph2d-node-vector-outline-stroke` | W4 | Pure | Stroke → filled path |
| 5 | `ph2d-node-vector-roughen` | W4 | Pure | Perturbação organic |
| 6 | `ph2d-node-vector-transforms` (CONSOLIDATED 3) | W4 | Pure | Twist + Mirror + Corner-Round em sub-modules |
| 7 | `ph2d-node-vector-bend-path` | W4 | Pure | Bend along envelope |
| 8 | `ph2d-node-vector-pattern-along-path` | W4/W8 | Pure | Brush distribution (consome `ph2d-brush-traits`) |
| 9 | `ph2d-node-vector-scatter` | W4 | Pure | Duplicate + distribute (radial/grid/random/along-path) |
| 10 | `ph2d-node-vector-width-profile` | W4 | Pure | Variable width 1D axes |
| 11 | `ph2d-node-vector-hatch` | W4 | Pure | Parametric hatch fill |
| 12 | `ph2d-node-vector-warp` | W4 | Pure | Perspective / mesh warp / liquify |
| 13 | `ph2d-node-vector-recolor` | W4 | Pure | Color harmony rules |
| 14 | `ph2d-node-vector-trim-path` ✨ (NEW Antigravity 2ª iter L5F2) | W4 | Pure | Trim start/end/offset (essencial motion designers ex-AE/Cavalry/Rive) |
| 15 | `ph2d-node-vector-llm-shape` ✨ | W13 | Stateful | LLM authoring (ADR-0061) |
| 16 | `ph2d-node-vector-auto-trace` ✨ | W12 | Stateful | ML raster→vector (ADR-0062) |
| 17 | `ph2d-node-vector-luau-script` | W4 (opt-in W9) | Stateful | Custom modifier em Luau |
| 18 | `ph2d-node-vector-voronoi-fracture` ✨ (Inovação #8 dormant fractures) | W16 | Pure | Pré-compute fracture lines para Dormant Fractures (ADR-0063) |

### 2.3 Cavalry-inspired taxonomy

- **Generators**: emit VectorNetwork from scratch (`vector-source`, `vector-llm-shape`).
- **Modifiers**: transform input network (12 nodes).
- **Combiners**: multiple inputs → single output (`vector-boolean`, `vector-pattern-along-path`).
- **Distributors**: scatter / duplicate (`vector-scatter`).
- **Falloffs**: sub-graph para modular params espacialmente (radial / linear / noise / shape-based — vide ADR-0058 §2.5 spec).

### 2.4 Pipeline boolean draft+reconcile (resolve crítica C Antigravity 1ª iter)

3 modos selectivos por contexto (vide ADR-0059 §3.3 + ADR-0065 SDF Hybrid):

| Modo | Algorithm | Budget | Use case |
|------|-----------|--------|----------|
| 1. Draft preview | CPU naive cubic clipping | ≤ 1 ms | Hot-path slider drag |
| 2. SDF Hybrid GPU | `min/max` compute shader | ≤ 0.5 ms | Real-time interactivity + gameplay morphing |
| 3. Linesweeper exato | Async background worker debounced | 1-50 ms off-thread | Commit (mouse-up / pencil-lift) — topology canônica |

Cache by `(input_a_hash, input_b_hash, op_type)` LRU 50 MB.

### 2.5 Custom modifier em Luau (`vector-luau-script` node)

Power user escreve modifier custom em Luau (HR-8 + HR-10):
- API exposta: `network.vertices`, `network.segments`, `network.regions`, `ph2d.vector.{add_vertex, move_vertex, ...}`, `ph2d.vec2()`, `ph2d.lerp()`.
- Sandbox: Luau strict mode (ADR-0019); no `os.execute`/`io.*`; timeout 5s; memory cap.
- Determinismo: `pairs_sorted()` obrigatório em pipeline determinístico (HR-16).

### 2.6 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| Total nodes canônicos | **18** (era 17; +trim-path Antigravity 2ª iter) | Adição requer ADR amendment (`0058-amendment-N.md`) |
| Cap-bump nodegraph | **NÃO** (caps ADR-0039 preservados) | Vector domain cabe na contract surface existente |
| Boolean variants | **9** (union/subtract/intersect/exclude/divide/trim/merge/crop/outline) | Paridade Illustrator Pathfinder |
| Boolean cache LRU | **50 MB** | Balance entre re-render speed vs memory |
| Graph nodes simultaneos worst case | **50+** com frame budget 3.5 ms | Gate `vector_graph_50_nodes_budget` |
| Luau script timeout | **5 seconds** | Kill se exceeds; mesma semantics que ADR-0019 |

### 2.7 Cross-domain determinism cascading

Quando upstream domain (`motion`) é determinístico (SimWorld ADR-0021), output vector também é. Trait `Node::deterministic_capable() -> bool`; check em cross-domain eval. Test `tests/determinism/vector_cross_domain.rs`.

`vector-llm-shape` é **NÃO** deterministic_capable por default (LLM emisson varies); UI warns user "LLM node prevents deterministic replay; consider baking result first (right-click → Bake)".

---

## 3. Consequências

### 3.1 Positivas

- **Live Boolean Graph diferencial competitivo brutal** — toda operação destrutiva Illustrator vira nó vivo. Nenhuma ferramenta mainstream entrega (Affinity compound shapes só boolean; Figma boolean ainda bake).
- **Fan-out paralelo destrancado** — 18 Implementadores podem escrever 18 crates em paralelo (DIRETRIZ §3.A + memory `feedback-phase-cascade`).
- **Trim Path ✨ NEW absorvido** elimina gap óbvio para motion designers ex-AE/Cavalry/Rive (L5F2 catch certeiro).
- **Consolidação seletiva (transforms triviais)** reduz fan-out incidental sem ganho real.
- **Caps ADR-0039 preservados** — zero amendment necessário em contrato base nodegraph.

### 3.2 Negativas

- **18 crates** adiciona ~30s build time cold (~5min total já mod tempo PH2D). Aceito — mantém DIRETRIZ §3.A drop-crate.
- **Cross-domain edges** introduce type check overhead (motion `Vec2` → vector `Vec2` é trivial; motion `f32` → vector `path.amplitude` é nominal). Mitigação: edge validation em graph editor UI antes de commit.
- **Boolean cache LRU 50 MB** memory overhead per session. Trade-off worth perf gain.

### 3.3 Neutras

- Falloff sub-system não é cap-bumped — vive em `ph2d-vector-doc` como AnimValue subscripts.

---

## 4. Alternativas consideradas

### 4.1 Domain monolítico `image-tools-vector` (rejeitada — viola DIRETRIZ)

Colocar todos 18 nodes em 1 crate. **Por que rejeitada**: viola DIRETRIZ §3.A drop-crate fan-out (coluna multi-agente PH2D); HR-18 LOC caps geraria god-file; Painter precedent (~10 crates próprios funcionando).

### 4.2 Cap-bump `NodeOp = 3` (rejeitada — Vector cabe em existing cap)

Adicionar variant `NodeOp::Vector(VectorOp)`. **Por que rejeitada**: caps 2/1/8 cobrem vector via mesmas semantics que motion (Pure/Temporal/Stateful effects). Cap-bump exigiria amendment ADR-0039; sem benefício real.

### 4.3 Trim Path como modifier de `outline-stroke` (rejeitada — UX óbvia)

Trim como param do outline-stroke node. **Por que rejeitada**: trim é animação primitiva canônica (After Effects "Trim Paths" é discoverable feature top); merece próprio node. UX paridade.

### 4.4 Auto-trace via classical Potrace only (rejeitada — diminished ambition)

Auto-trace tradicional. **Por que rejeitada**: 2026 hardware suporta ML embed (SuperSVG ~50 MB) + LLM4SVG via MCP. Spec inclui 3 modos (Sketch/Illustration/Basic Shapes); Potrace é fallback.

---

## 5. Implementação (Wave 3+)

- **W3**: `ph2d-panel-vector-graph` (Coord-B scaffold) + 3 nodes pilot (`vector-source` consolidated, `vector-boolean`, `vector-offset`).
- **W4**: Fan-out paralelo 12 nodes restantes + trim-path.
- **W8 / W12 / W13 / W16**: nodes especializados via ADRs irmãs.

Gates ativos a partir de W3: `vector_node_golden_<name>` per-node + `vector_graph_50_nodes_budget` + `vector_cross_domain_deterministic_cascading`.

---

## 6. Referências

- Spec normativa: [`docs/Vector Module/02_geometry_graph.md`](../../Vector%20Module/02_geometry_graph.md) (576 linhas).
- ADR-0039 nodegraph contract base.
- Cavalry app: <https://cavalry.scenegroup.co/>
- Houdini SOPs: <https://www.sidefx.com/docs/houdini/nodes/sop/index.html>
- Blender 4.3 Grease Pencil em Geometry Nodes: <https://developer.blender.org/docs/release_notes/4.3/grease_pencil/>
- Antigravity L5F2 trim-path absorption em README §11.C.
