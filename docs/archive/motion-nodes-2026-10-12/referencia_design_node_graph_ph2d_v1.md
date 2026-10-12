# ARQUIVO — referencia_design_node_graph_ph2d_v1.md (história, 228 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`referencia_design_node_graph_ph2d_v1.md`](../../Motion%20Nodes/referencia_design_node_graph_ph2d_v1.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-178,257-647,798-918` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### 5.1 Tipos canônicos (em `ph2d-nodegraph-core`)

```rust
/// Identificador estável de um nó dentro de um grafo. Hash do (graph_id, node_local_id).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Reflect, Saveable)]
pub struct NodeId(u64);

/// Identificador estável de um tipo de nó. Hash FNV-1a 64-bit do "scene.pulsar".
/// Mesma estrutura que ph2d-tool-registry::node_id (ADR-0027).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeTypeId(u64);

/// Identificador de socket dentro de um nó.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PinId { pub node: NodeId, pub local: u16 }

/// Aresta entre dois pinos. Direção: from.output → to.input.
#[derive(Copy, Clone, Reflect, Saveable)]
pub struct Edge { pub from: PinId, pub to: PinId }

/// Manifest declarativo de um tipo de nó. Espelha ToolManifest (ADR-0027).
/// Cada tipo de nó exporta `pub const MANIFEST: NodeManifest`.
pub struct NodeManifest {
    pub type_id: NodeTypeId,            // hash de "scene.pulsar"
    pub label_key: LabelKey,            // "node.scene.pulsar.label" — HR-15
    pub category: NodeCategory,         // header color via this
    pub silhouette: NodeShape,          // Rect/Circle/Diamond/Cigar/TrapezoidDown/TrapezoidUp/Tabbed
    pub icon_fn: fn() -> BezPath,       // Vello path
    pub inputs: &'static [PinSpec],
    pub outputs: &'static [PinSpec],
    pub params: &'static [ParamSpec],
    pub context: Context,               // Scene/Shader/Logic — wire compat enforced
    pub sim_safe: bool,                 // P6 — gate para grafos SimWorld
    pub reads_attrs: &'static [AttrId], // P-influence layer: o que esse nó lê
    pub writes_attrs: &'static [AttrId],// o que esse nó escreve
    pub a11y_role: Role,                // HR-12
    pub memory_budget: MemoryBudget,    // HR-13
    pub mcp: McpExposure,               // HR-10/HR-11
    pub eval_fn: fn(&EvalCtx, &Params, &Inputs) -> Outputs,
    pub emit_luau_fn: Option<fn(&LuauEmitter, &Params) -> ()>,    // None para Scene/Shader
    pub emit_wgsl_fn: Option<fn(&WgslEmitter, &Params) -> ()>,    // None para Scene/Logic
}

/// Especificação de um socket.
pub struct PinSpec {
    pub name: &'static str,
    pub label_key: LabelKey,
    pub data_type: DataType,            // float, shape, trigger, etc.
    pub cardinality: PinCardinality,    // Single (círculo) ou Stream (diamante) — P-visual
    pub side: PinSide,                  // Left/Right/Top (Nuke junctions)
}

/// O grafo em si — vive como Asset, hash blake3 (HR-6).
#[derive(Reflect, Saveable)]
#[saveable(version = 1)]
pub struct Graph {
    pub context: Context,
    pub nodes: Vec<NodeInstance>,
    pub edges: Vec<Edge>,
    pub default_params: BTreeMap<NodeId, ParamMap>,  // BTreeMap, não HashMap — ADR-0022
    pub timeline: Option<Handle<Timeline>>,          // Ω3 — companheiro opcional
}

/// Instância de grafo no ECS. Component em PresentWorld (ou SimWorld para Logic).
#[derive(Component, Reflect, Saveable)]
#[saveable(version = 1)]
pub struct GraphInstance {
    pub asset_id: Handle<Graph>,
    pub param_overrides: SmallVec<[(NodeId, ParamName, Value); 8]>,
    pub runtime_state: GraphRuntimeState,  // HR-16 POD-only
}
```

**Notas:**
- `BTreeMap` substitui `HashMap` em todo state simulado (ADR-0022). Iteração determinística cross-OS.
- `SmallVec<[...; 8]>` permite 8 overrides inline sem heap (caso comum); spillover heap apenas em casos extremos.
- `version: u32` em `Graph` e `GraphInstance` para migração HR-14.

## 11. Estrutura de crates

Decisão M3: **3 infra crates + 1 init + 3 category crates** = 7 crates totais (extensível).

```
crates/
├── ph2d-nodegraph-core/           ~800 LOC
│   ├── lib.rs                     (exports)
│   ├── graph.rs                   (Graph, NodeInstance, Edge)
│   ├── manifest.rs                (NodeManifest, PinSpec, ParamSpec)
│   ├── context.rs                 (Context enum extensível)
│   ├── data_type.rs               (DataType enum: float, vec3, shape, trigger…)
│   ├── attr_id.rs                 (AttrId enum — Camada 2)
│   ├── registry.rs                (NodeRegistry: BTreeMap<NodeTypeId, &'static Manifest>)
│   └── node_id.rs                 (NodeTypeId hash FNV-1a — espelha ADR-0027)
│
├── ph2d-nodegraph-eval/           ~600 LOC
│   ├── lib.rs
│   ├── eval_ctx.rs                (EvalCtx<'a> com bumpalo arena — P7)
│   ├── scene_evaluator.rs         (pull lazy DAG, cache por frame)
│   ├── luau_emitter.rs            (Logic — compile-to-Luau D1)
│   ├── wgsl_emitter.rs            (Shader — compile-to-WGSL)
│   ├── probe_sink.rs              (Camada 5)
│   └── hot_reload.rs              (reset+restore Defold-style — Ω5)
│
├── ph2d-nodegraph-editor/         ~2000 LOC (decomposto em mod, HR-18 cap 600/file)
│   ├── lib.rs
│   ├── canvas.rs                  (Vello canvas, pan/zoom)
│   ├── node_paint.rs              (silhuetas, header colorido, pin shapes — §6)
│   ├── wire_paint.rs              (espessura, cor, dashed — §6.5)
│   ├── zone_paint.rs              (If/Repeat/Forever brackets — P4)
│   ├── influence_highlight.rs     (Camada 1)
│   ├── heatmap.rs                 (Camada 4)
│   ├── probe.rs                   (Camada 5)
│   ├── overlay.rs                 (Camada 6 — espacial por nó-tipo)
│   ├── backdrop.rs                (Camada 7)
│   ├── sidebar.rs                 (Camada 8 — co-author + attrs)
│   ├── breadcrumb.rs              (subnet navigation — Ω9)
│   ├── interaction.rs             (drag/connect, espelha ADR-0024 WidgetStore pattern)
│   ├── mcp_actions.rs             (Camada 8 + Ω1 — graph.* tools)
│   └── a11y.rs                    (HR-12 — Node/Pin/Edge → accesskit::Node)
│
├── ph2d-nodegraph-init/           ~100 LOC
│   └── lib.rs                     (register_all_nodes — append-only)
│
├── ph2d-nodes-scene/              ~3000 LOC, 1 file/nó
│   ├── lib.rs                     (pub mod pulsar; pub mod wave; ... — append-only)
│   ├── pulsar.rs                  (1 nó por arquivo, ~60 LOC: const MANIFEST + eval)
│   ├── wave.rs
│   ├── falloff.rs
│   ├── spin.rs
│   ├── wiggle.rs
│   ├── stagger.rs
│   ├── bloom.rs
│   ├── render.rs                  (terminal output)
│   ├── ... (~50 nós seed na v1+v1.5)
│   └── bridges/
│       ├── commit_transform_sim.rs
│       └── material.rs
│
├── ph2d-nodes-shader/             ~1500 LOC (v1.5, M17)
│   └── ... (15 nós seed)
│
└── ph2d-nodes-logic/              ~1500 LOC (v2, M18)
    └── ... (10 nós seed + compiler)
```

**Por que não A (1 crate por nó):** workspace overhead massivo (50+ crates), build time degrada, `register_all_nodes` vira 50 linhas com conflito de merge constante.

**Por que B (agrupado):** 1 arquivo por nó dentro do crate de categoria. 2 agentes em nós diferentes = zero conflito. `pub mod X;` em lib.rs é append-only (mesmo problema, mesma solução do ADR-0027).

**Quando promover para crate próprio:** se um nó precisa de dep pesada (ex: FFT lib em AudioReact), promove para `crates/ph2d-node-audioreact/`. Pattern híbrido — convention-by-discovery permite.

---

## 12. Plano de implementação — marcos M14-M18

### M14 — Infra core + LLM-co-author foundation

**Crates:** `ph2d-nodegraph-core`, `ph2d-nodegraph-eval`, `ph2d-nodegraph-editor`, `ph2d-nodegraph-init`.
**Escopo:**
- Tipos canônicos (Graph, NodeManifest, NodeInstance, Edge, PinSpec, AttrId)
- Pull lazy evaluator com bumpalo (P7 zero-alloc bench)
- Editor canvas Vello + drag/connect (reusa ADR-0024 input pipeline)
- 7 silhuetas + header colorido + pin shapes + wire thickness (§6)
- A11y day-1: `Node`/`Pin`/`Edge` → `accesskit::Node` (HR-12)
- **MCP tools `graph.*`** — 13 ferramentas (§8.2)
- Subnet/composite (Ω9) com asset Handle
- Hot reload reset+restore (Ω5)
- Save format versionado + migrator stub v1→v2 (HR-14)
- Lint-as-spec: `node_pin_color_canonical.rs`, `node_pin_shape_consistency.rs` (Ω8)

**Gate:** abre canvas vazio; usuário arrasta 2 nós (Source + Render) e conecta; MCP `graph.add_node` cria nó programaticamente; subnet collapse/expand funciona; cross-OS replay hash idêntico em fixture sintético.
**Tempo:** ~4-6 semanas. Single agente sequencial.

### M15 — 30 Scene nodes seed + Camadas 1-3 de Inteligibilidade

**Crate:** `ph2d-nodes-scene/` (30 nós seed).
**Escopo:**
- 30 nós motion design canônicos (Pulsar, Onda, Brilho, Falloff, Spin, Wiggle, Stagger, Wave, Move, Transform, Color, ColorArray, NumberRange, Modulate, Noise, Random, Oscillator, Distribute*, Duplicator, Group, Mixer, Bloom, Blur, Glow, HueShift, Sepia, Vignette, Render, Source primitivos)
- Cada nó com `reads_attrs`/`writes_attrs` declarados (Camada 2)
- Camada 1: Influence Highlight on-demand ★★★★★
- Camada 3: attribute tags inline (mini chips F/T/C/P/R/S/α)

**Gate:** tutorial Mini Cavalry "01-estrela-giratoria.md" reprodutível em PH2D editor; LLM via MCP cria grafo "make sprite breathe" em ≤ 5 tool calls.
**Tempo:** ~6-8 semanas. Sequencial inicialmente; paralelizável após primeiros 10 nós (template estabilizado).

### M16 — Camadas 4-8 + 20 Scene nodes adicionais

**Crate:** `ph2d-nodes-scene/` cresce.
**Escopo:**
- 20 nós a mais (Particles emitter, Instancer, Path follow, FollowTarget, LookAt, Snap, Clamp, Delay, Spring, easing curves, time sources)
- Camada 4: heatmap por instância para Focus nodes
- Camada 5: probe em qualquer fio
- Camada 6: overlays espaciais (DistributePath, VectorField, Attractor, Pivot, FollowTarget)
- Camada 7: backdrops com emoji
- Camada 8: sidebar Quem-escreve-quem-lê + co-author log

**Gate:** todos os 5 tutoriais Mini Cavalry reprodutíveis; canvas com 50+ nós permanece legível.
**Tempo:** ~4 semanas. Paralelizável (cookbook estabilizado).

### M17 — Shader context

**Crates:** `ph2d-nodes-shader/`, expande `ph2d-nodegraph-eval` (WgslEmitter).
**Escopo:**
- 15 nós shader (Sample, UV, Combine vec3, Split, Swizzle, Math {Mul/Add/Lerp/Smoothstep}, Noise, Voronoi, Gradient, Step, Fresnel-2D, Mix, Material Output, Custom WGSL)
- Compilador WGSL → naga → MSL/SPIR-V/HLSL
- Preview por nó (Substance Designer pattern) renderizando em mini sphere/plane no canto
- Bridge node `Material` aceita subnet shader; integrado a `Render` em Scene

**Gate:** cooked WGSL asset blake3-hash-stable cross-OS; tonemap shader existente (sprite.wgsl) reproduzível como grafo.
**Tempo:** ~3-4 semanas.

### M18 — Logic context + Luau compiler

**Crates:** `ph2d-nodes-logic/`, `ph2d-nodegraph-eval::luau_emitter`.
**Escopo:**
- 10 nós Logic high-level domain (`OnClick`, `OnFrame`, `OnKey`, `OnCollision`, `If` zona, `Repeat N` zona, `Wait`, `Spawn`, `Trigger Animation`, `Set Property`)
- Compilador `LuauEmitter` (AST→Luau source→mlua bytecode)
- `sim_safe: bool` enforced em compile time para SimWorld target
- Audit log HR-11 em modos Rollback/Lockstep
- Hot reload reset+restore validado (Defold-style herdado)

**Gate:** "On click sprite, spawn 10 particles, after 2s fade out" em 5 nós; LLM via MCP cria grafo equivalente em ≤ 7 tool calls; replay determinístico em fixture multiplayer mockada.
**Tempo:** ~6-8 semanas.

**Total M14-M18:** ~6 meses calendário. Cada marco mergeável independentemente.

---

