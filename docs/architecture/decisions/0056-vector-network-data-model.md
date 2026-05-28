# ADR-0056 — Vector Network data model (topologia + dual-representation + edit log)

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [ADR-0021 — SimWorld/PresentWorld boundary](0021-simulation-presentation-boundary.md), [ADR-0022 — No HashMap in simulation](0022-no-hashmap-in-simulation.md), [ADR-0039 — Nodegraph contract freeze](0039-nodegraph-contract-freeze-w2t4.md).
**Sub-contratos congelados por ADRs irmãs:** 0057 (Edit dispatch + CRDT), 0058 (Geometry graph domain `vector`), 0059 (Renderer pipeline), 0060 (Procedural fill shader graph), 0061 (LLM authoring), 0062 (Painter bridge), 0063 (Runtime + Physics), 0064 (Multi-platform input), 0065 (SDF Hybrid GPU), 0066 (Variable Font Glyph), 0067 (brush-traits decoupling), 0068 (Mobile Core tier).
**Spec normativa:** [`docs/Vector Module/01_data_model.md`](../../Vector%20Module/01_data_model.md) — esta ADR fixa o **contrato de superfície** do data model; a spec detalha o **comportamento e algoritmos**.
**Tags:** vector, wave-0, contract, drop-crate, padrão-ouro, foundational

---

## 1. Contexto

O Vector Module é o **sucessor ambicioso do Adobe Illustrator** integrado à game engine PH2D ([`docs/Vector Module/README.md`](../../Vector%20Module/README.md)). Mandato §0 [HANDOFF_node_system §0](../../HANDOFF_node_system.md) + memory `feedback-perfection-no-deferrals`: padrão-ouro absoluto, integração total prevista com Painter / motion nodes / shader nodes futuros / animation / Luau / MCP / runtime de jogo com physics colliders dinâmicos. 20 waves planejadas, 105 tasks T-W.N, 32 crates novos, 8 inovações extraordinárias (entre elas Live Boolean Graph + Mesh Gradient via diffusion curve + LLM-as-graph-node + Vector Runtime determinístico + Dormant Fracture Edges).

Antes de escrever uma linha de código (W1+), o data model precisa estar **congelado** porque:

1. **Drop-crate fan-out depende disso.** 32 crates Vector consomem `VectorNetwork`, `Vertex`, `Segment`, `Region`, `VectorOp`. Mudança post-W0 num desses tipos forçaria cap-bump em todos os 17 nodes (`ph2d-node-vector-*`) + 10 tools (`ph2d-tool-vector-*`) + 4 panels — fan-out paralelo colide.

2. **Múltiplas waves paralelas vão mutar o mesmo `VectorNetwork`.** W1 cria network básico; W3 adiciona boolean ops (mutate via node graph); W4 fan-out de 12 modifiers; W12 paint-into-vector bridge cria networks Hobby-fit; W13 LLM emite networks via semantic tokens. Sem contrato fixo, cada wave reinterpreta a API e duas sessões paralelas colidem.

3. **A spec do Vector Module já está madura.** 3 iterações Antigravity (1ª 2026-05-27, 2ª 2026-05-28, 3ª 2026-05-29) catalogaram 5+23+19 findings; CONVERGENCE 9.2/10 medido + ~9.7/10 estimado pós-3ª absorção. O risco "spec do papel" se materializa se esta ADR só repetir prosa sem **caps numéricos** — vide DIRETRIZ §3.A.3 "caps numéricos são o teste de cheiro do contrato".

4. **O contrato `NodeOp`/`OpResolver`/`NodeManifest` está congelado por ADR-0039** (caps 2/1/8). Esta ADR **não amenda** o contrato nodegraph — só fixa **o que `VectorNetwork`, `Vertex`, `Segment`, `Region` e `VectorOp` expõem** dentro do domain `vector` adicionado em ADR-0058.

### 1.1 O que diferencia o Vector Module data model de outros tools PH2D

Os 5 RasterEditTool de produção (bgremoval, color-equalization, padding, upscale, equalize-sizes) operam sobre raster bytes em runtime. Painter (ADR-0043) tem `Brush` + `Stamp` + stroke history vetorial mas continua raster final. Motion nodes (ADR-0039 + crates `ph2d-node-motion-*`) emitem `Vec2`/`Affine` outputs.

Vector Module é **único em duas dimensões fundamentais**:

| Eixo | Outros tools PH2D | Vector Module |
|---|---|---|
| **Tipo primário do output** | Bytes raster (RasterEditTool) OU primitivos `f32`/`Vec2`/`Affine` (motion) | `VectorNetwork` — graph topológico de vertices/segments/regions com per-region fills |
| **Mutabilidade em runtime** | Read-mostly assets carregados de disk | Edit-heavy: edit log event-sourced + CRDT multi-agent local desde W1 |

**Crucial:** ambas dimensões exigem data model novo. Não cabe estender `Brush` (Painter) nem `Stamp` (Painter). Não cabe reusar `Vec2`/`Affine` outputs de motion (precisa de topology — vertices têm N incidências). Não cabe path-based subpath model clássico de Illustrator/SVG (Figma's vector network ganha em editing UX, vide §15.1.3 estado da arte).

### 1.2 Por que sub-contratos separados (0057..0068)

Esta ADR fixa **só a superfície do data model** (structs + enum + serialization schema). Sub-sistemas adjacentes têm ADRs próprias:

- ADR-0057: como mutar o data model (edit log + CRDT semantics).
- ADR-0058: como compor operações sobre o data model (geometry graph domain `vector`).
- ADR-0059: como renderizar (Vello pipeline + draft+reconcile boolean).
- ADRs 0060-0068: subsistemas que consomem este data model como input/output canônico.

Razão: cada sub-contrato tem evolução distinta. Adicionar `vector-trim-path` node (ADR-0058 amendment) não pode forçar bump no `VectorOp` cap. Adicionar Mobile Core tier degradation (ADR-0068) não pode mexer schema `.ph2d-vector`. A modularidade do contrato espelha a modularidade dos crates.

---

## 2. Decisão

### 2.1 Crate foundational `ph2d-vector-doc`

O data model vive em crate isolado `crates/ph2d-vector-doc/` (drop-crate per DIRETRIZ §3.A). Estrutura canônica:

```
crates/ph2d-vector-doc/
├── Cargo.toml                    (deps: serde, postcard, smallvec, glam, blake3,
│                                   ph2d-color, ph2d-vector-traits)
├── src/
│   ├── lib.rs                    pub use exports + invariants doc
│   ├── network.rs                VectorNetwork struct + invariants + validate()
│   ├── cubic.rs                  Vertex + Bézier cubic (representação default visível)
│   ├── spiro.rs                  Spiro / hyperbezier (Assist Modes opt-in)
│   ├── cubic_fit.rs              Conversão Spiro/hyper → cubic (Levien Béz fitting)
│   ├── region.rs                 Region + WindingRule + winding semantics
│   ├── edit_log.rs               VectorOp enum + EditLog + event sourcing
│   ├── crdt.rs                   LWW-Element-Set + RGA + custom (vide ADR-0057)
│   ├── postcard_schema.rs        Ph2dVectorAsset + bounded_decode + migrators
│   ├── hit_test.rs               nearest() per segment + BVH spatial index
│   └── deterministic.rs          Q16.16 fixed-point opt-in + ordered reductions
└── tests/
    ├── architecture_vector_contract_surface.rs   (caps gate)
    ├── crossing_auto_resolve.rs                  (invariant test)
    ├── crdt_convergence.rs                       (fixture + proptest)
    └── security_postcard_bounds.rs               (adversarial fixtures)
```

Justificativa: **único crate** que define os tipos primários `VectorNetwork`/`Vertex`/`Segment`/`Region`/`VectorOp`. Outros 31 crates Vector importam transitivamente via `ph2d-vector::*` re-exports (ADR-0059 `ph2d-vector` crate encapsula Vello/kurbo/peniko — vide L6F1 long-tail maintenance).

### 2.2 Vector Network topology (Figma model)

**Decisão fundamental**: data model = **vector network**, NÃO path-based. Vertex pode ter N incidências; crossings auto-inserem intersection vertices; minimal cycle basis identifica regions; per-region fills (vide §15.1.3 referência paper Figma 2017 + alexharri.com deep-dive).

```rust
/// Topologia primária do Vector Module. Graph não-direcionado de
/// vertices + segments + regions, com per-region fills e crossings
/// auto-resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VectorNetwork {
    /// Schema version (HR-14). Bump em quebra de layout.
    pub version: u32,

    /// Vertices indexados por VertexId. SmallVec inline 32.
    pub vertices: SmallVec<[Vertex; 32]>,

    /// Segments indexados por SegmentId. SmallVec inline 64.
    pub segments: SmallVec<[Segment; 64]>,

    /// Regions com winding rule + segment refs. SmallVec inline 8.
    pub regions: SmallVec<[Region; 8]>,

    /// Style refs (stroke + fill per region/segment).
    pub style_refs: StyleRefMap,

    /// Authoring representation hint (cubic vs spiro vs hyper).
    pub authoring_hint: RepresentationMode,

    /// Determinismo opt-in (vide ADR-0021 SimWorld). Default false.
    pub deterministic: bool,
}

pub type VertexId = u32;
pub type SegmentId = u32;
pub type RegionId = u32;
```

### 2.3 Caps congelados (arch-gate `architecture_vector_contract_surface`)

| Cap | Valor | Razão |
|---|---|---|
| `VectorOp ≤ N variants` | **N = 16** | Cobre 14 ops básicas (`AddVertex`, `MoveVertex`, `RemoveVertex`, `AddSegment`, `MoveTangent`, `RemoveSegment`, `AddRegion`, `SetRegionFill`, `ApplyBoolean`, `CrdtMerge`, `SetStrokeStyle`, `SetAuthoringHint`, `BatchOp`, `Checkpoint`) + 2 reservados para expansão (até v2 schema). Bump exige amendment ADR-0056-amendment-N.md. |
| `Vertex` campos | **5** | `id` (u32), `pos` (Vec2), `kind` (VertexKind enum), reservado x2. Compact representation; tangentes vivem em `Segment` (per-edge, não per-vertex — vector network invariant). |
| `Segment` campos | **6** | `id` (u32), `start` (VertexId), `end` (VertexId), `out_at_start` (Vec2), `in_at_end` (Vec2), `style_ref` (Option<StyleRef>). |
| `Region` campos | **5** | `id` (u32), `segments` (SmallVec<[(SegmentId, bool); 16]>), `winding` (WindingRule), `fill` (Option<FillRef>), `z` (i32). |
| `VertexKind` variants | **4 FROZEN** | `Mirror`, `Aligned`, `Free`, `Auto`. Cobre paridade Illustrator/Affinity/Figma editing semantics. |
| `RepresentationMode` variants | **3 FROZEN** | `Cubic` (default visível), `SpiroAssist`, `HyperbezierAssist`. Decisão D Antigravity 1ª iteração — Bézier cúbico default; Spiro/hyper opt-in. |
| `WindingRule` variants | **2 FROZEN** | `EvenOdd`, `NonZero`. Default `NonZero` (SVG canon). |

Arch-gate `crates/ph2d-vector-doc/tests/architecture_vector_contract_surface.rs` (W1.T1.2 cria) força caps numéricos. Falha em build se cap excedido.

### 2.4 Dual-representation: Bézier cúbico default + Spiro/Hyperbezier Assist Modes (decisão D Antigravity)

**Decisão D Antigravity 1ª iteração 2026-05-27 absorvida em §1.3 spec**:

- **Bézier cúbico = representação default visível** ao usuário. Paridade Illustrator/Affinity/Figma — zero fricção muscle memory.
- **Spiro / Hyperbezier = Assist Modes opt-in** via toggle HUD (`S` / `H`). Útil para typeface design, jewelry, organic curves. Sem forçar ao usuário.
- **Data model interno é dual-representation**: cubic stored sempre; spiro/hyper stored quando `authoring_hint != Cubic`. Conversion Spiro→cubic via Levien Béz fitting é canônica (single direction) com max error < 0.5 px.

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RepresentationMode {
    Cubic,                  // Default visível (paridade Illustrator)
    SpiroAssist,            // Clothoid splines opt-in
    HyperbezierAssist,      // Elastica-under-tension opt-in
}
```

### 2.5 `AnimValue` typed enum (CRITICAL fix Antigravity 2ª iteração L1F4)

Animation infrastructure exige typed return — `f32` quebraria W10+ retroativamente:

```rust
// Em ph2d-vector-traits::anim_value
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum AnimValue {
    Float(f32),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Color(ph2d_color::ColorOklch),
    Bool(bool),
    Enum(u32),
}

pub trait LinearInterp: Sized {
    /// `t` é `f64` (decisão L1F1 Antigravity 3ª iteração — preserve precision
    /// em sessões > 4h a 120Hz; `TimeContext` typed documentado future V2.0).
    fn lerp(a: Self, b: Self, t: f64) -> Self;
}
```

`AttributeEvaluator::sample(t: f64) -> AnimValue` é canônico desde W1.

### 2.6 Schema `.ph2d-vector` postcard (HR-14 versioned)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ph2dVectorAsset {
    pub version: u32,                             // Schema version (HR-14)
    pub network: VectorNetwork,
    pub edit_log: EditLog,
    pub styles: StyleTable,
    pub metadata: AuthoringMetadata,
    pub embedded_assets: SmallVec<[EmbeddedAsset; 4]>,
    pub crdt_state: Option<CrdtReplay>,           // multi-agent opt-in
    pub dormant_fractures: Option<DormantFractureSet>,  // §8.8 inovação #8
}
```

**Bounded deserialization** (Antigravity 3ª iteração L4F2 security):

```rust
pub fn load_vector_asset(bytes: &[u8]) -> Result<Ph2dVectorAsset> {
    const MAX_ASSET_SIZE: usize = 100 * 1024 * 1024;  // 100 MB
    if bytes.len() > MAX_ASSET_SIZE { return Err(Error::AssetTooLarge); }
    bounded_decode(bytes, AssetBounds {
        max_vertices: 100_000,
        max_segments: 200_000,
        max_regions: 10_000,
        max_edit_log_ops: 1_000_000,
        max_embedded_assets: 64,
        max_embedded_asset_size: 16 * 1024 * 1024,
    })
}
```

Migrator chain HR-14 obrigatório: `migrate_v1_to_v2`, etc.

### 2.7 Determinismo opt-in (ADR-0021 SimWorld bridge)

Quando `VectorNetwork::deterministic = true`:
- Coordenadas em fixed-point Q16.16 (32-bit, 1 unit = 1/65536 px).
- Ordered reductions (sum/min/max via canonical iter order — by id sorted, não hash).
- Sem FMA (compute shaders incluem `#pragma fma_off`).
- Sem `dpdx`/`dpdy` em pipelines determinísticos.
- Linesweeper boolean determinístico via app-layer pré-ordenação (L2F3 Antigravity 3ª iter — vide ADR-0059 §2.4).

Gate `tests/determinism/vector_replay.rs` (fixture 600 ops) + cross-OS Linux x86_64 (AVX2 + non-AVX) + aarch64 NEON + macOS + Windows: hash blake3 bit-identical.

### 2.8 Edit log event-sourced (foundational para ADR-0057)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditLog {
    pub ops: Vec<VectorOp>,
    pub snapshots: Vec<(usize, NetworkSnapshot)>,  // a cada 100 ops
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VectorOp {
    AddVertex { id: VertexId, pos: Vec2, kind: VertexKind },
    MoveVertex { id: VertexId, new_pos: Vec2 },
    RemoveVertex { id: VertexId },
    AddSegment { id: SegmentId, start: VertexId, end: VertexId, tangents: TangentsCubic },
    MoveTangent { seg: SegmentId, which: TangentSide, new_pos: Vec2 },
    RemoveSegment { id: SegmentId },
    AddRegion { id: RegionId, segments: SmallVec<[(SegmentId, bool); 16]>, winding: WindingRule },
    SetRegionFill { id: RegionId, fill: Option<FillRef> },
    ApplyBoolean { op: BooleanOp, regions: SmallVec<[RegionId; 4]>, result_id: RegionId },
    CrdtMerge { peer_id: u64, peer_seq: u64 },
    SetStrokeStyle { seg: SegmentId, style: StrokeStyle },
    SetAuthoringHint { mode: RepresentationMode },
    BatchOp { ops: SmallVec<[Box<VectorOp>; 8]> },  // atomic transaction
    Checkpoint { hash: blake3::Hash },              // sync point CRDT
    // 2 variants reservados (até cap=16).
}
```

CRDT semantics (LWW + RGA + custom) detalhadas em ADR-0057.

---

## 3. Consequências

### 3.1 Positivas

- **32 crates Vector compartilham mesmo data model invariante** — fan-out paralelo W2+ não colide.
- **Vector network topology** (Figma model) destranca features impossíveis em path-based: shared edges, per-region fills, auto-crossings, minimal cycle basis. Diferencial competitivo central.
- **Dual-representation cubic+spiro** preserva muscle memory profissional (Bézier default) sem ferir power user (Spiro/Hyper opt-in).
- **`AnimValue` typed enum** elimina retrabalho W10+ (vide §11.C absorção Antigravity 2ª iter L1F4 CRITICAL).
- **Determinismo opt-in via Q16.16** habilita replay cross-platform + rollback netcode + multiplayer co-edit-ready (CRDT data model já preparado).
- **Bounded deserialization** previne RCE via malicious `.ph2d-vector` files (segurança vide ADR-0057 + §11.D L4F2).
- **Schema versionado HR-14** garante migrator chain manutenível por anos (long-tail L6F2 mitigation).

### 3.2 Negativas

- **Memory overhead vs path-based**: vector network adiciona ~30% memória vs path simples (vertices têm id, segments têm both endpoints separados). Aceito — caps SmallVec inline cobrem shapes simples sem heap.
- **Conversion overhead Spiro/Hyper → cubic** em render path (~50-200 µs per path). Cached por hash; W1 prototype valida custo real.
- **CRDT bookkeeping** adiciona ~30% memory overhead em sessões multi-agent. Disabled em single-user mode (default).
- **18 nodes geométricos** (ADR-0058) precisam respeitar mesmo data model — drift entre node implementations seria bug (mitigação: arch-gate `vector_contract_surface` + golden tests per-node).

### 3.3 Neutras

- Memory budget per asset desktop: ~5-15 MB típico; Mobile Core tier limit ~1-2 MB (ADR-0068).
- Spec gêmea (`docs/Vector Module/01_data_model.md`, ~914 linhas) preserve sincronizada com este ADR via cross-references; divergência detectada via review em PRs futuros.

---

## 4. Alternativas consideradas

### 4.1 Path-based subpath model (rejeitada)

Adobe Illustrator / SVG path model. Cada `Path` = sequência de subpaths; vertices isolated por subpath; crossings manuais.

**Por que rejeitada**: vector network resolve dores estruturais que path-based herda (vide §1.1 do README): shared edges entre regions exige duplicate vertices; "boolean union" perde topology editável; per-region fills impossíveis. Figma 2017 demonstrou a topologia network como **única inovação fundamental de data model vetorial desta década** (vide §15 estado da arte).

### 4.2 Spiro como representação default visível (rejeitada — decisão D Antigravity 1ª iter)

Spec original W0 propunha Spiro/Hyperbezier como authoring representation default. Antigravity 1ª iteração L5F1 alertou: "designers profissionais têm décadas de muscle memory em tangentes Bézier; forçar Spiro causará rejeição imediata."

**Por que rejeitada**: muscle memory de 25+ anos de Adobe Illustrator não é overcome-able via "novo modelo melhor". Bézier cúbico = default visível; Spiro/Hyper opt-in. Hobby fitter no Pencil tool permanece canônico (sem alternativa superior). Dual-representation interna preserva ambos sem fricção UX.

### 4.3 `AnimValue` retornando apenas `f32` (rejeitada — CRITICAL L1F4 Antigravity 2ª iter)

Spec original W0 propunha `trait AttributeEvaluator { fn sample(&self, t: f32) -> f32; }`. Antigravity 2ª iteração L1F4 (severidade CRITICAL) alertou: "quebra W10+ retroativamente quando animação precisa interpolar Vec2/Color/Bool."

**Por que rejeitada**: animação de Color OKLCH, Vec2 vertex positions, Bool flag toggles é **caso comum** em vector graphics motion design. Retornar `f32` força workarounds em cada caller (multiple traits, double dispatch, etc.). `AnimValue` typed enum é canônico desde W1 — pequeno cost upfront, zero retrabalho future.

### 4.4 LWW-Element-Set puro (sem RGA) para CRDT (rejeitada — vide ADR-0057)

Proposta 5 Antigravity 1ª iteração sugeriu LWW. **Por que rejeitada parcialmente**: LWW perde ordering em region segments (winding direction matters); RGA preserve intent. Hybrid LWW+RGA+custom canônico (detalhe em ADR-0057).

### 4.5 Postcard sem bounds (rejeitada — L4F2 Antigravity 3ª iter)

Loading assets via `postcard::from_bytes` direto sem size caps. **Por que rejeitada**: malicious `.ph2d-vector` file pode declarar SmallVec length gigante → heap overflow / RCE potencial. `bounded_decode` + adversarial fixtures em `tests/security/` obrigatório.

---

## 5. Implementação (Wave 1)

Tasks T-W.N que materializam este ADR (vide `docs/Vector Module/17_plano_de_implementacao.md`):

- **T1.1** — `ph2d-vector-traits` crate (mocks + `AnimValue` enum).
- **T1.2** — `ph2d-vector-doc` skeleton (este ADR).
- **T1.4** — Cubic fitting (Levien Béz fitting).
- **T1.6** — CRDT data model (vide ADR-0057).

Arch-gate ativo a partir de T1.2 commit: `architecture_vector_contract_surface`.

---

## 6. Open questions resolved during W0

| Q | Resolução |
|---|-----------|
| `t` em traits animação: `f32` vs `f64` vs `TimeContext`? | **`f64`** (L1F1 Antigravity 3ª iter — preserve precision sessões >4h; `TimeContext` future V2.0). |
| Cubic vs Spiro como default? | **Cubic default visível, Spiro Assist opt-in** (decisão D Antigravity 1ª iter). |
| HashMap vs BTreeMap em sim path? | **BTreeMap OR Vec indexed** (ADR-0022). |
| Memory inline budget SmallVec? | **Vertices 32, Segments 64, Regions 8, Region.segments 16, EmbeddedAssets 4**. |

---

## 7. Referências

- Spec normativa: [`docs/Vector Module/01_data_model.md`](../../Vector%20Module/01_data_model.md) (914 linhas).
- §15 Estado da arte: [`docs/Vector Module/15_estado_da_arte.md`](../../Vector%20Module/15_estado_da_arte.md) §15.1.3 Figma vector networks + §15.2.4 Spiro/Hyperbezier Levien.
- Figma vector networks (2017): <https://www.figma.com/blog/introducing-vector-networks/>
- Alex Harri deep-dive: <https://alexharri.com/blog/vector-networks>
- Spiro (Raph Levien): <https://levien.com/spiro/>
- Hyperbezier (Levien): <https://www.cmyr.net/blog/hyperbezier.html>
- Béz fitting (Levien 2021): <https://raphlinus.github.io/curves/2021/03/11/bezier-fitting.html>
- ADR Painter precedente: [ADR-0043](0043-painter-contract.md).
- 3 iterações Antigravity (Google DeepMind) absorvidas: [`docs/Vector Module/README.md §11.B + §11.C + §11.D`](../../Vector%20Module/README.md).
