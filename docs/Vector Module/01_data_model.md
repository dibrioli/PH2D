# 01 — Data Model (Vector Network + Curve Representations + CRDT Edit Log)

> Spec técnico denso do data model do Vector Module. Vector Network (Figma model) como topologia primária. Bézier cúbico como representação **default visível** (paridade Illustrator + decisão D Antigravity); Spiro / hyperbezier como Assist Modes opt-in. Edit log event-sourced + CRDT (LWW-Element-Set + RGA + custom) para multi-agente local. Schema `.ph2d-vector` versionado (HR-14).
>
> **ADRs ratificadores:** ADR-0056 (Vector Network data model) + ADR-0057 (Edit dispatch + CRDT).
> **Spec gêmeo:** [`02_geometry_graph.md`](02_geometry_graph.md) (operações sobre o data model).

## 1.1 Vector Network — topologia primária

### 1.1.1 Por que vector network (não path)

**Path model clássico** (Illustrator, Affinity, Inkscape pre-2024):
- Subpath sequencial: `M x0,y0 L x1,y1 C cx,cy ...`.
- Cubo modelado como **três subpaths separados**, cada um com seus próprios vertices.
- Vertex compartilhado entre subpaths é **invisível** ao modelo → mover um vertice exige editar 3 subpaths.
- Adicionar shared edge entre 2 paths exige delete + redraw.

**Vector Network model** ([Figma 2017](https://www.figma.com/blog/introducing-vector-networks/)):
- **Graph não-direcionado** de vertices + segments + regions.
- **Vertex pode ter N incidências.** Cubo é UM network com vertices compartilhados; deletar 1 edge não destrói o resto.
- **Crossings auto-inserem** intersection vertices (auto-resolve em time real via Linesweeper).
- **Minimal cycle basis** identifica regions preenchíveis automaticamente.
- **Per-region fills**: mesma network, múltiplas regions, cada uma com seu fill independente.

**Decisão:** vector network. **Não negociável** — diferencial competitivo central.

### 1.1.2 Estrutura canônica

```rust
/// Topologia primária do Vector Module. Graph não-direcionado de
/// vertices + segments + regions, com per-region fills e crossings
/// auto-resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorNetwork {
    /// Schema version (HR-14). Bump em quebra de layout.
    pub version: u32,

    /// Vertices indexados por VertexId (u32 stable across edits).
    /// SmallVec inline 32 para evitar alloc em shapes simples.
    pub vertices: SmallVec<[Vertex; 32]>,

    /// Segments indexados por SegmentId (u32 stable). Cada segment
    /// referencia 2 vertices + 4 tangent handles (in/out per end).
    pub segments: SmallVec<[Segment; 64]>,

    /// Regions com winding rule + segment refs. SmallVec inline 8.
    pub regions: SmallVec<[Region; 8]>,

    /// Style refs (stroke + fill per region/segment).
    pub style_refs: StyleRefMap,

    /// Authoring representation hint (cubic vs spiro vs hyper).
    /// Apenas hint para Pen tool re-abrir no modo correto; render
    /// usa cubic sempre (vide §1.3).
    pub authoring_hint: RepresentationMode,

    /// Determinismo opt-in (ADR-0056 §3.4). Default false (PresentWorld).
    /// Quando true, garante fixed-point coords + ordered reductions.
    pub deterministic: bool,
}
```

```rust
/// Identifier estável cross-edits. Usado em edit log + CRDT references.
/// u32 = 4 bilhões max per network — over-engineered? não, evita renumber.
pub type VertexId = u32;
pub type SegmentId = u32;
pub type RegionId = u32;
```

### 1.1.3 Vertex

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub id: VertexId,

    /// World coords em pixels (canvas-relative).
    pub pos: Vec2,

    /// Tangent kind determina behavior em edição:
    /// - Mirror: in_tan e out_tan são reflection (smooth).
    /// - Aligned: in_tan e out_tan colineares mas magnitudes independentes.
    /// - Free: in_tan e out_tan independentes (corner).
    /// - Auto: tangentes derivadas automaticamente (smooth fitter aplicado).
    pub kind: VertexKind,

    /// Tangent handles per incident edge — armazenado no Segment (vide §1.1.4),
    /// não aqui, para suportar vertex compartilhado entre múltiplas edges.
    /// (Vertex Network model: tangent é per-edge-end, não per-vertex.)
    _phantom: PhantomData<()>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VertexKind {
    /// Mirror tangents (smooth, default em Pen tool click).
    Mirror,
    /// Aligned tangents (colineares, magnitudes diferentes).
    Aligned,
    /// Free corner.
    Free,
    /// Auto (smooth fitter aplicado).
    Auto,
}
```

### 1.1.4 Segment

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: SegmentId,

    /// Endpoints — VertexId references.
    pub start: VertexId,
    pub end: VertexId,

    /// Tangent handles per-end. Bézier cúbico standard.
    /// out_at_start: control point saindo de start.
    /// in_at_end: control point entrando em end.
    /// Para straight line: ambos = pos do endpoint correspondente.
    pub out_at_start: Vec2,
    pub in_at_end: Vec2,

    /// Style: stroke + fill são per-region/per-segment (style_refs no parent).
    /// Aqui apenas indices via StyleRef.
    pub style_ref: Option<StyleRef>,
}
```

### 1.1.5 Region

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: RegionId,

    /// Ordered segments com direction flag (true = forward, false = reverse).
    /// Direction permite shared edges entre regions.
    pub segments: SmallVec<[(SegmentId, bool); 16]>,

    /// Winding rule: EvenOdd ou NonZero.
    pub winding: WindingRule,

    /// Fill style ref.
    pub fill: Option<FillRef>,

    /// Z-order dentro do network (regions empilham em order de criação por default).
    pub z: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WindingRule {
    EvenOdd,
    NonZero,
}
```

### 1.1.6 Invariants

1. **Toda Vertex no `vertices[]` tem id único.** Verificado em `VectorNetwork::validate()`.
2. **Todo Segment referencia VertexIds existentes.**
3. **Toda Region referencia SegmentIds existentes em ordem que forma cycle (start de N = end de N-1).**
4. **Crossings auto-resolved:** dois segments que se intersectam geram intersection vertex (auto-split via Linesweeper). Test: `tests/crossing_auto_resolve.rs`.
5. **Determinismo opt-in:** quando `deterministic=true`, vertices stored em fixed-point Q16.16; reductions ordered.

### 1.1.7 Performance budget

- Hot path: render N=1000 segments em ≤ 1.5 ms (sub-budget Render = 3.5 ms).
- Hit-test: query "qual segment está sob mouse" em ≤ 100 µs via spatial index (BVH ou grid).
- Memory: 1000 segments network ≈ 200 KB (vertices ~32B × 500 + segments ~48B × 1000 + regions ~96B × 50).

---

## 1.2 Spiro / Hyperbezier — Authoring representations (Assist Modes)

### 1.2.1 Por que dual-representation

**Bézier cúbico tem fraquezas como representação de autor:**
- Curva "perfeita" exige fine-tuning manual de tangentes.
- Smoothness não é parameter (G2 continuity não é trivial de manter).
- Aproximar circle perfeito requer 4 cubics com magic ratio.

**Spiro (clothoid splines, Raph Levien):**
- Curva determinada por **2 control points por segment** (não 4 tangent handles).
- Smoothness é parameter intrínseco do modelo (clothoid = curvature linear in arc-length).
- Excellent para typeface design (usado em Inconsolata).
- Refs: <https://levien.com/spiro/> + libspiro <https://github.com/fontforge/libspiro>.

**Hyperbezier (Levien newer family):**
- Elastica-under-tension behavior (smooth defaults para "pen tool that doesn't fight you").
- Single param `tension` controla suavidade.
- Refs: <https://www.cmyr.net/blog/hyperbezier.html>.

### 1.2.2 Quando usar Spiro/Hyperbezier

**Default da Pen Tool = Bézier cúbico** (decisão D Antigravity, vide §1.3). Spiro/Hyper como **Assist Modes opt-in** (toggle HUD `S` / `H`):

- **Letterforms / typeface design** → Spiro shines (clothoid suporta calligraphy curves).
- **Jewelry / organic shapes** → Hyperbezier (elastica produces visually pleasing defaults).
- **Logos / geometric shapes** → cubic Bézier (precise control).

### 1.2.3 Estrutura

```rust
/// Authoring representation alternativa, opt-in via Assist Mode.
/// Stored APENAS quando authoring_hint != Cubic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiroPath {
    /// Control points (2 per segment, não 4 como cubic).
    pub points: SmallVec<[SpiroPoint; 16]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiroPoint {
    pub pos: Vec2,
    /// Spiro type: G2 corner, smooth, etc.
    pub kind: SpiroKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SpiroKind {
    Corner,        // {} hard corner
    Smooth,        // [] G2 smooth
    LeftConstraint,  // [[ asymmetric left
    RightConstraint, // ]] asymmetric right
}
```

### 1.2.4 Conversion cubic ↔ Spiro

- **Spiro → Cubic** (sempre needed para render via Vello): Spiro solver produz polyline; cubic fit via **Levien Béz fitting** ([raphlinus.github.io/curves/2021/03/11/bezier-fitting.html](https://raphlinus.github.io/curves/2021/03/11/bezier-fitting.html)). Max error < 0.5 px.
- **Cubic → Spiro** (apenas quando user troca Assist mode mid-edit): approximação inversa, lossy. Documented em ADR-0056 §5.2.

### 1.2.5 Determinismo

Spiro solver é iterativo (Newton-Raphson para satisfazer constraints). Determinismo opt-in usa fixed iteration count + fixed-point arithmetic.

---

## 1.3 Bézier cúbico — Representação default visível

### 1.3.1 Decisão D Antigravity (2026-05-27)

Spec original W0 propôs Spiro como **default**. Antigravity alertou: "designers profissionais têm décadas de muscle memory em tangentes Bézier; forçar Spiro causará rejeição imediata."

**Decisão final: Bézier cúbico = default visível** (paridade Illustrator). Spiro / Hyperbezier = Assist Modes opt-in.

### 1.3.2 Render path

- Vello (GPU compute) consome cubic Bézier (`kurbo::CubicBez`).
- Não há render path direto para Spiro / Hyperbezier — sempre convert via `spiro_to_cubic()` antes de render.
- Cache da conversion (Spiro path → cubic) por hash do SpiroPath; invalidate em edit.

### 1.3.3 Export

- SVG export: **sempre cubic** (Levien Béz fitting de Spiro/Hyper).
- `.ph2d-vector` export: preserva ambos (cubic always; spiro/hyper se authoring_hint != Cubic).

---

## 1.4 Edit log event-sourced

### 1.4.1 Por que event-sourced

- **Imperative mutate** (`network.vertices[3].pos = new_pos`) perde history → undo impossível sem snapshot.
- **Snapshot-based undo** = pesado em memory (1 snapshot por op).
- **Event-sourced log** = lightweight (1 entry por op); replay rápido; trivialmente CRDT-able.
- Padrão consagrado em Figma, Notion, qualquer collab tool moderno.

### 1.4.2 Estrutura

```rust
/// Event-sourced edit log. Toda mutação é uma VectorOp gravada
/// aqui; replay reproduz state from scratch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditLog {
    /// Ops em order temporal.
    pub ops: Vec<VectorOp>,

    /// Snapshots periódicos (a cada 100 ops) para acelerar undo
    /// muito longe sem replay full.
    pub snapshots: Vec<(usize, NetworkSnapshot)>,
}

/// Mutações canônicas do VectorNetwork. Cap ≤ N variants (ADR-0056
/// definirá N final; estimativa atual = 16).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VectorOp {
    /// Adiciona vertex.
    AddVertex { id: VertexId, pos: Vec2, kind: VertexKind },
    /// Move vertex (changes pos).
    MoveVertex { id: VertexId, new_pos: Vec2 },
    /// Deleta vertex (e segments incidentes, propagado).
    RemoveVertex { id: VertexId },
    /// Adiciona segment.
    AddSegment { id: SegmentId, start: VertexId, end: VertexId, tangents: TangentsCubic },
    /// Move tangent (changes out_at_start ou in_at_end).
    MoveTangent { seg: SegmentId, which: TangentSide, new_pos: Vec2 },
    /// Deleta segment (não deleta vertices).
    RemoveSegment { id: SegmentId },
    /// Adiciona region (cycle of segments).
    AddRegion { id: RegionId, segments: SmallVec<[(SegmentId, bool); 16]>, winding: WindingRule },
    /// Set region fill.
    SetRegionFill { id: RegionId, fill: Option<FillRef> },
    /// Boolean op result baked (quando user "applies" boolean destructively;
    /// raro — most boolean ops são live node graph, NÃO destructive).
    ApplyBoolean { op: BooleanOp, regions: SmallVec<[RegionId; 4]>, result_id: RegionId },
    /// CRDT merge marker (quando 2 sites merging concurrent edits).
    CrdtMerge { peer_id: u64, peer_seq: u64 },
    /// Style ops (stroke/fill changes).
    SetStrokeStyle { seg: SegmentId, style: StrokeStyle },
    /// Authoring mode change (cubic → spiro Assist).
    SetAuthoringHint { mode: RepresentationMode },
}
```

### 1.4.3 Undo / Redo

- Undo = `EditLog::revert_last_op()` + re-apply de snapshot if conveniently next snapshot.
- Redo = re-apply ops popped to redo stack.
- Memory budget: 10k ops ≈ 1 MB; trivial.

---

## 1.5 CRDT (LWW-Element-Set + RGA + custom)

### 1.5.1 Por que CRDT

**Resolve Proposta 5 Antigravity** — multi-agente local (LLM assistente + designer humano) editing mesmo canvas em paralelo. CRDT garante:
- **Convergence**: 2 sites editing → merge converge para mesmo state.
- **Sem servidor**: peer-to-peer local (ou file-based merge).
- **Sem locks**: editing concurrent é safe.

### 1.5.2 Trade-off LWW vs RGA vs custom

Vector network tem **3 categorias de state com semantics distintas**:

1. **Set membership** (which vertices in which region): **LWW-Element-Set** (Shapiro 2011). Adicionar/remover vertex de region; conflict resolution simples last-writer-wins por timestamp.
2. **Ordering** (segments dentro de region; order matters para winding direction): **RGA** (Replicated Growable Array) — preserva intent ordering across concurrent inserts.
3. **Continuous values** (vertex.pos, tangent handles): **per-component LWW** com tolerância — duas tangentes movendo concurrentemente colide; LWW por axis com semantics "last writer per axis" (não whole vector).

### 1.5.3 Implementation strategy

```rust
/// CRDT replay engine. Aplica ops via merging que converge mesmo
/// com edição concurrent.
pub struct CrdtReplay {
    /// Site ID local (estável per session).
    pub site_id: u64,
    /// Sequence local counter.
    pub seq: u64,
    /// LWW-Element-Set para region membership.
    pub region_members: HashMap<RegionId, LwwSet<(SegmentId, bool)>>,
    /// RGA para segment ordering em regions.
    pub segment_order: HashMap<RegionId, Rga<SegmentId>>,
    /// Per-component LWW para vertex positions / tangents.
    pub continuous: HashMap<(VertexOrSegmentRef, ComponentAxis), LwwRegister<f32>>,
}

impl CrdtReplay {
    /// Apply op com CRDT semantics. Convergence garantida em test
    /// tests/crdt_convergence.rs (5+ scenarios).
    pub fn apply(&mut self, op: VectorOp, ts: Timestamp) -> Result<()> {
        // (...)
    }

    /// Merge de outro site's log. Idempotent + commutative.
    pub fn merge(&mut self, other: &EditLog) -> Result<()> {
        // (...)
    }
}
```

### 1.5.3-bis Timestamp validation window + periodic integrity check (Antigravity 3ª iteração L4F3 + L7F3 2026-05-29)

**Timestamp forge attack (L4F3)**: agente local malicioso (LLM agent compromised OR adversarial Luau script) pode forjar timestamps far-future ("my edits always win LWW") OU far-past ("rollback your edits"). Solução: validation window clamped contra SimWorld clock.

```rust
impl CrdtReplay {
    pub fn apply(&mut self, op: VectorOp, claimed_ts: Timestamp) -> Result<()> {
        let local_clock = self.sim_world_clock_now();
        let max_drift = Duration::from_secs(30);  // window tolerance
        
        if claimed_ts > local_clock + max_drift {
            return Err(Error::TimestampFromFuture { claimed: claimed_ts, local: local_clock });
        }
        if claimed_ts < local_clock - max_drift {
            return Err(Error::TimestampTooOld { claimed: claimed_ts, local: local_clock });
        }
        
        // Accept op com timestamp clamped
        let safe_ts = claimed_ts.clamp(local_clock - max_drift, local_clock + max_drift);
        self.apply_with_validated_ts(op, safe_ts)
    }
}
```

**Periodic integrity check (L7F3)**: silent CRDT divergence em multi-agent local (rare but real) corrupts state. Mitigation:
- Cada N seconds (default 30s), todos sites computam `blake3(state.serialize())` e exchange hash.
- Discrepância detectada → trigger rollback to last consensual snapshot (LCS) + replay logs from there.
- LCS = `state` em `snapshot_idx` periódico (cada 100 ops).

Gate CI `vector_crdt_silent_divergence_recovery` simula divergence + valida recovery converge.

### 1.5.4 Convergence test + property-based testing (revisado Antigravity L1F6 2ª iteração 2026-05-28)

**Fixture tests** (`tests/determinism/vector_crdt_convergence.rs`):
- 2 sites A + B editando paralelamente (10 ops cada com timestamps overlapping).
- A merges B; B merges A.
- Resulting state idêntico em A e B (hash blake3 match).
- Repeat com 5+ scenarios (region creation, vertex move, tangent edit, boolean apply, simultaneous delete).

**Property-based testing OBRIGATÓRIO** (`tests/determinism/vector_crdt_proptest.rs`, crítica L1F6):

Usar crate [`proptest`](https://crates.io/crates/proptest) para simular **milhares de sequências aleatórias** de mutações concorrentes:

```rust
proptest! {
    #[test]
    fn crdt_converges_under_random_concurrent_edits(
        site_a_ops in prop::collection::vec(arb_vector_op(), 1..200),
        site_b_ops in prop::collection::vec(arb_vector_op(), 1..200),
    ) {
        let mut crdt_a = CrdtReplay::new(site_id_a());
        let mut crdt_b = CrdtReplay::new(site_id_b());
        
        // Apply ops independently
        for op in &site_a_ops { crdt_a.apply(op.clone(), arb_timestamp()).unwrap(); }
        for op in &site_b_ops { crdt_b.apply(op.clone(), arb_timestamp()).unwrap(); }
        
        // Cross-merge
        let log_b: EditLog = crdt_b.export_log();
        let log_a: EditLog = crdt_a.export_log();
        crdt_a.merge(&log_b).unwrap();
        crdt_b.merge(&log_a).unwrap();
        
        // Convergence assertion
        let hash_a = blake3::hash(&postcard::to_allocvec(&crdt_a.state()).unwrap());
        let hash_b = blake3::hash(&postcard::to_allocvec(&crdt_b.state()).unwrap());
        prop_assert_eq!(hash_a, hash_b);
        
        // Topology invariants — proteção contra anomalias L1F6
        prop_assert!(crdt_a.state().has_no_orphan_segments());
        prop_assert!(crdt_a.state().has_no_self_intersecting_regions());
        prop_assert!(crdt_a.state().region_cycles_close_properly());
    }
}
```

**Configuration**: `proptest!{}` runs default 256 cases; CI override `PROPTEST_CASES=10000` para nightly stress test.

**Gate CI**: `vector_crdt_proptest_convergence` valida zero failures em 10k random sequences cross-OS.

### 1.5.5 Performance

- Apply op: O(1) amortized.
- Merge log: O(N) onde N = ops in other log.
- Memory overhead vs imperative: ~30% (CRDT bookkeeping). Aceito.

---

## 1.6 `.ph2d-vector` postcard schema (versionado, HR-14)

### 1.6.1 Layout v1

```rust
/// Top-level asset format. Postcard binário, blake3-addressed (HR-6),
/// versioned (HR-14).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ph2dVectorAsset {
    /// Schema version. Migrator chain obrigatório (HR-14).
    pub version: u32,

    /// Network principal.
    pub network: VectorNetwork,

    /// Edit log (event-sourced, vide §1.4).
    pub edit_log: EditLog,

    /// Style table (stroke + fill definitions, referenced via StyleRef).
    pub styles: StyleTable,

    /// Authoring metadata (creator, timestamps, app version).
    pub metadata: AuthoringMetadata,

    /// Embed assets (referenced images, fonts, etc.).
    pub embedded_assets: SmallVec<[EmbeddedAsset; 4]>,

    /// CRDT state (opt-in, presente quando multi-agente flag ativo).
    pub crdt_state: Option<CrdtReplay>,
}
```

### 1.6.2 Migrator policy

```rust
/// Migrator entry point. Chamado em `load_ph2d_vector()` quando
/// asset version != CURRENT_VERSION.
fn migrate(asset: &mut Ph2dVectorAsset) -> Result<()> {
    while asset.version < CURRENT_VERSION {
        match asset.version {
            1 => migrate_v1_to_v2(asset)?,
            // 2 => migrate_v2_to_v3(asset)?,
            // 3 => migrate_v3_to_v4(asset)?,
            n => return Err(Error::UnsupportedVersion(n)),
        }
    }
    Ok(())
}

fn migrate_v1_to_v2(asset: &mut Ph2dVectorAsset) -> Result<()> {
    // v2 adicionou (hipotetico) `winding_rule` em Region.
    // Default existing regions para EvenOdd.
    for region in &mut asset.network.regions {
        if region.winding == WindingRule::default() {
            region.winding = WindingRule::EvenOdd;
        }
    }
    asset.version = 2;
    Ok(())
}
```

### 1.6.3 Size budget

- Network 100 vertices / 200 segments / 20 regions: ~50 KB binário.
- Edit log 1000 ops: ~200 KB.
- Embedded image 512×512 PNG: ~150 KB.
- Total tipico asset: ~500 KB.

### 1.6.4 Content addressing (HR-6)

Asset identidade = blake3 do binário cooked. Rename de arquivo não invalida referências (handle = hash, não path).

### 1.6.5 Postcard deserialization bounds (Antigravity 3ª iteração L4F2 2026-05-29)

Asset `.ph2d-vector` parse via postcard é **input não confiável** se vem de network ou shared workspace. Adversarial file pode declarar SmallVec length gigante → heap overflow / OOM / potential RCE.

**Pipeline canônico**:

```rust
pub fn load_vector_asset(bytes: &[u8]) -> Result<Ph2dVectorAsset> {
    // 1. File size cap absoluto — reject before parse
    const MAX_ASSET_SIZE: usize = 100 * 1024 * 1024;  // 100 MB
    if bytes.len() > MAX_ASSET_SIZE {
        return Err(Error::AssetTooLarge);
    }

    // 2. Header validate antes de full decode
    let header: Ph2dVectorHeader = postcard::from_bytes_partial(bytes)?;
    if header.magic != PH2D_VECTOR_MAGIC {
        return Err(Error::InvalidMagic);
    }
    if header.version > CURRENT_VERSION + 1 {  // future asset rejeita
        return Err(Error::FutureVersion(header.version));
    }

    // 3. Bounds em cada collection length DURANTE decode
    let mut decoder = postcard::Deserializer::from_bytes(bytes);
    let asset: Ph2dVectorAsset = bounded_decode(&mut decoder, AssetBounds {
        max_vertices: 100_000,
        max_segments: 200_000,
        max_regions: 10_000,
        max_edit_log_ops: 1_000_000,
        max_embedded_assets: 64,
        max_embedded_asset_size: 16 * 1024 * 1024,  // 16 MB
    })?;

    // 4. Validate post-decode invariants
    asset.network.validate()?;

    // 5. Migrate if needed
    if asset.version < CURRENT_VERSION {
        migrate(&mut asset)?;
    }

    Ok(asset)
}
```

**Adversarial test fixtures**: `tests/security/malicious_assets/`:
- `oversized_smallvec.ph2d-vector` (claims 1B vertices) → reject.
- `truncated_header.ph2d-vector` → reject.
- `invalid_winding.ph2d-vector` → reject post-validate.
- `nested_embedded_recursion.ph2d-vector` → reject (max nesting depth 4).

Gate CI `vector_postcard_security_bounds` valida 100% recusados.

---

## 1.7 Hit-testing API

### 1.7.1 Per-segment hit test

```rust
impl Segment {
    /// Nearest point on this segment (parametric t + distance).
    /// Usa kurbo::ParamCurveNearest::nearest() — NUNCA rasterize-then-pick.
    pub fn nearest(&self, network: &VectorNetwork, query: Vec2) -> NearestResult {
        let cubic = self.as_kurbo_cubic_bez(network);
        let n = cubic.nearest(query.into(), 1e-6);
        NearestResult {
            t: n.t,
            point: cubic.eval(n.t),
            distance: (query - cubic.eval(n.t).into()).length(),
        }
    }
}
```

### 1.7.2 Network-level spatial index

- BVH (bounding volume hierarchy) ou grid hash para acelerar "qual segment está sob mouse" em network com 1000+ segments.
- Updated incrementalmente em `VectorOp::MoveVertex` / `AddSegment` / etc.
- Memory: ~16 bytes / segment.

### 1.7.3 Region hit-test

Point-in-region usando ray casting + winding rule (Even-Odd ou Non-Zero). O(N segments / region).

---

## 1.8 Determinismo opt-in

### 1.8.1 Quando ativar

```rust
let asset = Ph2dVectorAsset {
    network: VectorNetwork {
        deterministic: true,  // opt-in
        ..
    },
    ..
};
```

### 1.8.2 Garantias

Quando `deterministic=true`:
- **Coordenadas em fixed-point Q16.16** (32-bit, 16 bits integer + 16 bits fraction). 1 unit fixed = 1/65536 px.
- **Ordered reductions**: qualquer operação que reduz (sum, min, max) usa order canônico (e.g., by vertex_id sorted).
- **Sem FMA** (Fused Multiply-Add varia entre archs). Compute shaders incluem `#pragma fma_off`.
- **Sem `dpdx`/`dpdy`** em pipelines determinísticos (vary by GPU).
- **Linesweeper deterministic mode** (versão sorted output).

### 1.8.3 Gate CI

`tests/determinism/vector_replay.rs`:
- Fixture com 600 ops + multi-agent merges.
- Roda em Linux + Mac + Windows.
- Hash blake3 do network resultante deve ser idêntico cross-OS.

### 1.8.4 Performance impact

Determinismo opt-in custa ~3-5× mais (fixed-point arithmetic + ordered reductions). Aceito para use cases de replay / multi-agent / Future-proof multiplayer-co-edit.

---

## 1.9 Winding rules

### 1.9.1 EvenOdd vs NonZero

- **EvenOdd**: ray casting count odd → inside. Útil para "holes" trivially (inner region cancels outer).
- **NonZero**: ray casting signed count != 0 → inside. SVG default; usado em fontes (glyph holes correctly handled).

### 1.9.2 Default

`WindingRule::NonZero` (SVG default + paridade com Vello / kurbo defaults).

### 1.9.3 User toggle

Inspector panel mostra dropdown per-region. Memorizado em `Region::winding`.

---

## 1.10 Imports / Exports

### 1.10.1 SVG (round-trip lossless v1.0 subset)

**Import:**
- Paths (M/L/C/Z) → segments.
- `<polygon>`, `<polyline>`, `<rect>`, `<ellipse>`, `<circle>` → regions de primitives equivalentes.
- Gradients (linear / radial) → FillRef.
- `<g>` groups → vector layer hierarchy.
- `<defs>` + `<use>` → symbol instances (W9).
- `<text>` → text-on-path (W14).
- Clip paths, masks → region operations.

**Export:**
- VectorNetwork → SVG paths (cubic Bézier cooked).
- Round-trip test: SVG → import → export → diff < 0.01 px.

**Não suportado (v1.0):**
- Filters DOM (`feGaussianBlur`, `feTurbulence`) — substituído por procedural shader graph (§05). Import: melhor esforço raster.
- SMIL animation — substituído por animation system (§06).

### 1.10.2 Adobe Illustrator (.ai) — lossy via PDF subset

Adobe Illustrator nativo é PDF wrapped com extensions proprietárias. Parser via `lopdf` (read-only):
- Extract paths + gradients + text + layers.
- Log gaps documentados (mesh gradient → raster fallback; symbols → flatten).

### 1.10.3 PDF (read-only via lopdf)

Subset de paths + gradients + text. Documentation em [`docs/Vector Module/15_estado_da_arte.md`](15_estado_da_arte.md).

### 1.10.4 `.ph2d-vector` native (canon)

Postcard binário. Schema versionado HR-14. blake3-addressed HR-6.

### 1.10.5 JSON (debug + LLM exchange)

Serde JSON apenas para development + LLM input/output (§09). Não shipping.

---

## 1.11 Structs Rust skeleton

### 1.11.1 Tipos completos

```rust
// (...completed structs above...)

/// Stroke style — colocalizado com Region/Segment via StyleRef.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub width: WidthProfile,
    pub color: Color,  // OKLCH from ph2d-tokens (HR-1 + ph2d-color).
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    pub miter_limit: f32,
    pub dashes: Option<DashPattern>,
}

/// Width profile — 1D variable-font-style axes (Levien stroke expansion).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidthProfile {
    pub base_width: f32,
    pub pressure_weight: f32,  // [0..1]
    pub taper_start: f32,
    pub taper_end: f32,
    pub contrast: f32,
    pub jitter_amount: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StrokeCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StrokeJoin {
    Miter,
    Round,
    Bevel,
}

/// Fill style — Solid / Gradient / Pattern / Procedural Shader (vide §05).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FillStyle {
    Solid(Color),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    MeshGradient(MeshGradient),  // Diffusion curve (§05 + §14.3).
    Pattern(PatternRef),
    ProceduralShader(ShaderGraphRef),  // ph2d-vector-fill DAG.
    Image(ImageRef),
}
```

### 1.11.2 IDs e refs

```rust
/// Style table — indexed via StyleRef. Permite estilo compartilhado
/// entre segments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleTable {
    pub strokes: SmallVec<[StrokeStyle; 8]>,
    pub fills: SmallVec<[FillStyle; 8]>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StrokeRef(pub u16);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FillRef(pub u16);
```

---

## 1.12 Memory layout invariants

### 1.12.1 HR-3 zero-alloc no hot path

- **Render**: VectorNetwork → Vello scene SEM Box::new / Vec::push-realoc / String::from.
- **Edit log apply**: usa pool pré-alocado de VectorOp.
- **Bump arena** por frame para temp data (kurbo BezPath flatten, etc.).

### 1.12.2 Gate dhat-rs

```rust
#[cfg(test)]
mod no_alloc_tests {
    use dhat::Profiler;

    #[test]
    fn render_vector_network_zero_alloc() {
        let mut profiler = Profiler::new_heap();
        let network = fixture_complex_network();
        render_to_scene(&network);  // hot path
        let stats = profiler.stats();
        assert_eq!(stats.total_blocks, 0);
    }
}
```

### 1.12.3 SmallVec caps

- Vertices inline 32 (cresce no heap acima).
- Segments inline 64.
- Regions inline 8.
- Segments per Region inline 16.

Justificativa: shapes simples (rect / ellipse / star) cabem inline; complex (typeface glyph) heap mas raro.

---

## 1.13 Performance budget

### 1.13.1 Frame budget (3.5 ms render sub-budget HR-4)

- **Network → BezPath conversion**: ≤ 0.3 ms / 1000 segments.
- **Vello rasterization**: ≤ 1.5 ms / 1000 segments (já em GPU compute).
- **Procedural fill shader**: ≤ 1.0 ms (depende do shader; vide §05).
- **Buffer total**: 0.7 ms folga para spikes.

### 1.13.2 Edit responsiveness

- **MoveVertex op apply**: ≤ 50 µs.
- **CRDT merge 100 ops**: ≤ 5 ms.
- **Boolean draft preview (SDF GPU)**: ≤ 0.5 ms.
- **Boolean exact reconcile (Linesweeper async)**: 1-50 ms (off-thread, debounced).

### 1.13.3 Memory budget per platform (HR-13)

| Platform | VRAM Vector | RAM Vector | Notes |
|----------|-------------|-----------|-------|
| Desktop (Mac/Win/Linux) | 200 MB | 100 MB | Includes shader compile cache + boolean result cache |
| iPad Pro M-series | 150 MB | 80 MB | Editor mode |
| iPad regular | 80 MB | 60 MB | Light mode (LOD aggressive) |
| Android top-tier | 80 MB | 50 MB | |
| Android entry | 40 MB | 30 MB | Light mode (LOD aggressive) |
| Web (WebGPU) | 40 MB | 20 MB | |

---

## 1.14 Cache strategy

### 1.14.1 Render cache

- **Network hash → Vello scene cached** (re-use entre frames quando network não muda).
- Invalidate on `VectorOp::*` que touch network.

### 1.14.2 Boolean cache

- **Boolean node hash (inputs + op type) → result network cached**.
- LRU eviction (cap 50 MB).
- Cross-frame stable.

### 1.14.3 Shader cache (procedural fill, §05)

- **Topology hash → WGSL compiled** + cached on-disk (`~/.cache/ph2d/shaders/<hash>.{wgsl,spv,msl}`).
- Cache hit-rate >95% target (gate `procedural_fill_no_recompile_on_animate`).

---

## 1.15 Innovations: Vector Network como differentiator + CRDT como differentiator

### 1.15.1 Vector Network vs path-based — diferenças observáveis

| Operação | Path model (Illustrator) | Vector Network (PH2D) |
|----------|--------------------------|------------------------|
| Cubo com shared edges | 3 subpaths separados | 1 network com vertices compartilhados |
| Mover vertex compartilhado | 3 edits sincronos manuais | 1 edit (auto-propagado) |
| Crossings auto-resolve | ❌ manual split | ✓ auto via Linesweeper |
| Multiple fills em 1 path | ❌ (1 fill per path) | ✓ (1 fill per region) |
| Edit topology destructive? | ✓ | ❌ (vive em edit log) |

### 1.15.2 CRDT vs imperative-mutate — diferenças observáveis

| Operação | Imperative-mutate (most tools) | CRDT (PH2D) |
|----------|--------------------------------|--------------|
| Multi-agent local concurrent edit | ❌ overwrite | ✓ converges |
| Replay determinístico cross-OS | difícil (snapshot snapshots) | trivial (replay log) |
| Multiplayer co-edit-ready | exige rewrite | trivial (already CRDT) |
| LLM agent co-editing canvas | ❌ collision | ✓ converges |

### 1.15.3 PH2D ahead of curve

- Vector Network adotado por Figma 2017 — único tool mainstream com.
- CRDT data model é estado da arte (Figma usa internamente; Notion/Linear/Linear/Excalidraw adotam).
- Combo Vector Network + CRDT + node graph + GPU compute + runtime de jogo = **PH2D Vector Module único no mundo**.

---

## Fim do data model

Spec técnico completo do Vector Module data model. Cap-bumps e ratificação via ADR-0056 (Vector Network) + ADR-0057 (CRDT) na ratificação W0.

**Next:** [02_geometry_graph.md](02_geometry_graph.md) (operações sobre o data model — boolean / offset / scatter / etc.).
