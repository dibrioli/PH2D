# ADR-0046 — Stroke Vector History format (`.ph2d-painter` v1)

**Status:** Proposed (2026-05-26)
**Decisor(es):** Enio + Claude (Coord-A, sessão Painter W0).
**Pré-requisitos:** [ADR-0043 — Painter contract](0043-painter-contract.md), [ADR-0044 — Brush Engine GPU](0044-brush-engine-gpu.md).
**Spec normativa:** [`docs/Painter_projeto/01_brush_engine.md`](../../Painter_projeto/01_brush_engine.md) §1.14 + [`08_performance_memory.md`](../../Painter_projeto/08_performance_memory.md) §8.2 (memory budget).
**Relaciona:** [ADR-0047 — MCP Stroke Engine](0047-painter-mcp-stroke-engine.md) (consome `StrokeRecord`), [ADR-0048 — Stroke Inspector](0048-stroke-inspector.md) (consome history + queries).
**Tags:** painter, wave-0, contract, stroke-history, determinism, hr-14-serialization

---

## 1. Contexto

Stroke history vetorial é o **vetor oculto do canvas** — o que faz o Painter PH2D ter capacidades que Procreate não tem:

1. **Reproject to Resolution (W12)** — re-pinta canvas em resolução nova sem blur de upscale.
2. **Stroke Inspector retroativo (W14)** — selecionar lasso temporal, trocar brush/color depois.
3. **Replay determinístico** — CI cross-OS verifica bit-identical.
4. **MCP Stroke Engine (W13)** — LLM consome/modifica strokes editáveis.
5. **Undo profundo** — sem cap arbitrário (Procreate ring 250).

Sem contrato congelado:

- **`StrokeRecord` cresce ad-hoc.** Cada feature adiciona campo "para fora" do schema; `.ph2d-painter` v1 fica incompatível com v1.x.
- **Determinismo é wishful thinking.** Sem fixed-point coords + RNG seed gravada por stroke, replay diverge cross-platform.
- **Memory budget vira pesadelo.** Sessões longas (12h ≈ 20k strokes ≈ 200 MB) sem cap explícito derrubam mobile.
- **Reproject vira código por-platform.** Sem protocolo (det vs GPU), W12 reimplementa per-flow.

ADR-0043 §2.5 + ADR-0044 cederam esse território a esta ADR. ADRs irmãs 0047 e 0048 **dependem** desta (compartilham fonte de verdade).

---

## 2. Decisão

### 2.1 Crate `ph2d-painter-stroke` (W1 T1.X) — leaf crate

```
crates/ph2d-painter-stroke/
  Cargo.toml         # deps: serde, postcard, blake3, uuid, ph2d-color, ph2d-painter-brush
                     # (DEP em brush APENAS para os tipos opacos BrushHandle + BrushParamsHash;
                     #  brush NÃO dep stroke — sentido único.)
  src/lib.rs         # #![forbid(unsafe_code)] PRIMEIRO
  src/record.rs      # StrokeRecord + RawPointerSample + ToolMode + pub type StrokeId
  src/history.rs     # StrokeHistory enum (Full | Ring) + invariants
  src/persistence.rs # .ph2d-painter postcard schema v1 (HR-14)
  src/snapshot.rs    # SnapshotBuilder (every-N-strokes layer texture cache)
  src/reproject.rs   # Reproject protocol (W12 implementation entry point)
  src/determinism.rs # fixed-point Q16.16/Q8.8 helpers + RNG-per-stroke
  tests/             # persistence round-trip + det-mode bit-identical
```

**Dependência cross-crate (autoridade direção):**

```
ph2d-painter-stroke  ──→  ph2d-painter-brush  (tipos opacos: BrushHandle, BrushParamsHash)
ph2d-painter-brush   ──→  (nada do Painter — é o leaf da brush engine)
```

`ph2d-painter-stroke` é **NÃO-folha** (dep `ph2d-painter-brush`). A audit 2026-05-26 (A1) flagged inversão prévia. Correção: `BrushHandle` é tipo opaco simples (ADR-0044 §2.8); importar do crate-fonte é canon. Resolução `BrushHandle → Brush` runtime (lookup em `Library`) acontece em `ph2d-tool-painter` (consumer de ambos).

### 2.2 `StrokeRecord` schema — cap **≤ 16 fields**

Baseline §1.14.1: 11 campos. Cap = **16** (+ 5 slots de headroom para evolução W12-W15).

```rust
/// Type alias canônico — referenciado por ADRs 0047, 0048.
pub type StrokeId = Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StrokeRecord {
    pub uuid: StrokeId,                      // identidade global do stroke
    pub seq: u64,                            // monotônico per-canvas (ordem cronológica)
    pub timestamp_ms: u64,                   // wall-clock approx (NÃO determinístico)
    pub brush_handle: BrushHandle,           // resolvido contra Library + atlas (ADR-0044)
    pub brush_params_hash: BrushParamsHash,  // blake3 do Brush snapshot (Reproject anchor)
    pub layer_target: LayerId,               // qual raster layer recebe os stamps
    pub primary_color: OklchColor,
    pub secondary_color: Option<OklchColor>, // long-press color slot
    pub points: Vec<RawPointerSample>,       // path BRUTO (pré-stabilization; cap §2.3)
    pub rng_seed: u64,                       // RNG per-stroke (det-mode anchor)
    pub tool_mode: ToolMode,                 // §2.4
    pub version: u32,                        // HR-14 — v1 = 1
    // === 4 slots de headroom (e.g., MCP origin tag, audit-log ref, time-lapse marker) ===
}
```

**`BrushParamsHash`:** type alias `pub type BrushParamsHash = [u8; 32];` definido em `ph2d-painter-brush` (computed via `Brush::params_blake3()` impl method). Se a `Brush` em si fosse copiada por stroke, o `.ph2d-painter` explodiria (12k strokes × ~5KB Brush ≈ 60 MB só de duplicação). Hash refere-se a tabela deduplicada `brush_snapshots: BTreeMap<BrushParamsHash, Brush>` na persistence (§2.7).

**Cap reserva `points: Vec<RawPointerSample>` — `len() ≤ 65535` (u16)** (audit B-3). Strokes muito longos (>22 minutos sem release) ficam impossíveis pelo type system. Caso edge gritante (fluid sim watercolor pintando 30min): split implícito no commit do stroke (não nessa ADR; W1 implementation detail).

### 2.3 `RawPointerSample` — cap **≤ 12 fields**

Pontos brutos pré-stabilization (Streamline / Stabilization aplicados em runtime, não gravados):

```rust
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RawPointerSample {
    pub x_q1616: i32,                        // fixed-point Q16.16 (det-mode)
    pub y_q1616: i32,                        // idem
    pub pressure_q88: u16,                   // Q8.8 [0, 1.0)
    pub tilt_q88: u16,                       // Q8.8 [0, π/2)
    pub azimuth_q88: u16,                    // Q8.8 [0, 2π)
    pub barrel_roll_q88: u16,                // Q8.8 [0, 2π); 0 se device não suporta
    pub timestamp_delta_us: u32,             // microseconds since previous sample (replay tempo correto)
    pub flags: u32,                          // bit 0: stylus_button1, bit 1: stylus_button2, …
    // === 4 slots de headroom ===
}
```

**Tamanho fixo:** 24 bytes / sample. 256 samples por stroke típico → 6 KB / stroke + ~200 B metadata = ~6.2 KB / stroke.

**Razão fixed-point (não f32):**
- **Determinismo cross-platform** (HR-5 opt-in). f32 não é IEEE-bit-identical entre ARM/x86 (FMA, denormals, sin/cos).
- **Reproject scaling** sem perda — re-multiplica Q16.16 por scale_factor e re-quantiza.
- **Compressão** — fixed-point comprime ~30% melhor em zstd que f32 (postcard com `optimize-for-storage` flag em W16).

Conversão runtime: helpers em `determinism.rs` (`fn q1616_to_f32(v: i32) -> f32 { v as f32 / 65536.0 }`).

### 2.4 `ToolMode` enum — cap **≤ 6 variants** (v1 usa 3)

```rust
#[repr(u32)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolMode {
    Paint  = 0,
    Smudge = 1,
    Erase  = 2,
    // === 3 slots de headroom (e.g., Mask, Lasso-temporal-mark) ===
}
```

`#[repr(u32)]` para serialização estável em postcard + alinhamento bytemuck se chegar a virar GPU buffer.

### 2.5 `StrokeHistory` enum — cap **≤ 4 variants** (v1 usa 2)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum StrokeHistory {
    /// Default desktop / iPad Pro M1+ / Android top-tier (memory_budget ≥ 150 MB).
    /// Sem cap arbitrário; cresce com sessão.
    Full(Vec<StrokeRecord>),
    /// Fallback low-end (Web 200 MB total, Android entry, iPad <= 2018).
    /// Ring buffer com cap N (default 1000). Warning UX:
    /// "Stroke history limited on this device; export to keep full history."
    Ring { records: VecDeque<StrokeRecord>, cap: usize },
    // === 2 slots de headroom (e.g., Branched para experiments, External para offload disk) ===
}
```

**Politica de escolha:** runtime detecta via `MemoryBudget::Painter.stroke_history_mb` ([`08 §8.2`](../../Painter_projeto/08_performance_memory.md)):

| Budget | Modo | Cap |
|---|---|---|
| ≥ 150 MB | `Full` | sem cap (subject a budget; eviction de snapshots é primeira linha) |
| 50-149 MB | `Ring { cap: 5000 }` | ~30 MB |
| < 50 MB | `Ring { cap: 1000 }` | ~6 MB |

Decisão automática + override manual via Settings (W17 polish).

### 2.6 Snapshots (otimização de undo)

Per §1.14.3 da spec: a cada N strokes (default 50), snapshot da layer texture é tirado. Undo de 1 stroke = load snapshot + re-run ≤ 50 strokes ≤ 200 ms p99.

```rust
pub struct LayerSnapshot {
    pub at_seq: u64,                         // após qual stroke seq
    pub layer_id: LayerId,
    pub texture_blake3: [u8; 32],            // content-addressed (dedup)
    pub texture_data: SnapshotStorage,       // InMemory(Vec<u8>) | OnDisk(PathBuf)
    pub timestamp_ms: u64,
    pub version: u32,                        // sidecar version v1 = 1 (HR-14 NOT obrigatório; regenerável)
}

pub enum SnapshotStorage {
    InMemory(Vec<u8>),                       // RGBA8 ou RGBA16F per canvas profile
    OnDisk(PathBuf),                         // offload LRU sob memory pressure
    // === 2 slots de headroom (e.g., Compressed zstd in-mem) ===
}
```

**Cap politica:**
- `LayerSnapshot ≤ 8 fields` (v1 usa 6)
- `SnapshotStorage ≤ 4 variants` (v1 usa 2)
- `PaintProjectCache ≤ 8 fields` (v1 usa 6)
- Snapshots **evictáveis sob memory pressure** (LRU em offload disk; metadata fica em RAM).

Snapshots NÃO entram em `StrokeHistory` — vivem em `Snapshots` struct paralelo. Razão: history é semântica de **input** (o que usuária fez); snapshots são **cache** (otimização derivada). Separar permite invalidar cache sem tocar history.

### 2.7 Persistence — `PaintProject` (canon) + `PaintProjectCache` (sidecar)

**Audit 2026-05-26 (C-3) flag:** snapshots são **cache** (não-essencial pra reproduzir o canvas), gravar dentro do canon força migração HR-14 quando snapshot schema mudar. **Separação congelada:**

#### 2.7.1 `.ph2d-painter` — canon savefile (HR-14 versionado)

```
PaintProject (v1)
├── magic: [u8; 12]                          // "PH2D-PAINTER"
├── version: u32                             // = 1
├── canvas: CanvasInfo { width, height, color_profile, ppm }
├── layer_stack: LayerStack                  // raster + adjustment layers (ADR-0045)
├── history: StrokeHistory                   // §2.5 — fonte de verdade vetorial
├── brush_snapshots: Vec<(BrushParamsHash, Brush)>  // dedup table (§2.2)
├── created_at: u64                          // ms since epoch
├── modified_at: u64                          // idem
└── checksum: blake3([magic..modified_at])   // integrity guard
```

**Top-level cap:** ≤ 12 fields (v1: 9). `CanvasInfo` cap ≤ 8 fields. Tudo aqui é **essencial** para reconstruir o canvas — replay de history sobre layer_stack vazio produz pixel-perfect output.

#### 2.7.2 `.ph2d-painter-cache` — sidecar cache (NÃO HR-14-versionado obrigatório)

Arquivo separado, ao lado do `.ph2d-painter`. Pode ser **deletado/regenerado** a qualquer momento sem perda de dados:

```
PaintProjectCache (v1)
├── magic: [u8; 18]                          // "PH2D-PAINTER-CACHE"
├── version: u32                             // = 1
├── source_blake3: [u8; 32]                  // hash do .ph2d-painter sibling — invalida se desbatido
├── snapshots: Vec<LayerSnapshot>            // §2.6
├── spatial_index: Option<SerializedRTree>   // R-tree de bboxes (ADR-0048 §2.4); pode estar vazia
└── generated_at: u64
```

**Top-level cap:** ≤ 8 fields (v1: 6). Não exige migration chain — versions futuras podem só ignorar sidecars antigos e regenerar.

**Cleanup policy:** se `source_blake3` não bate com `.ph2d-painter` atual, cache descartado silenciosamente (regenera on-demand). Sidecar nunca "atrasa" o save do canon.

**HR-14 migration policy (canon `.ph2d-painter`):**
- v1 → v2 quando schema mudar. v2 reader **deve** ler v1 (forward-compat is mandatory).
- Writer sempre emite latest version.
- Migration helpers em `persistence.rs::migrate_v{N}_to_v{N+1}`.

**HR-14 NOT obrigatório (sidecar `.ph2d-painter-cache`):** sidecar é regenerável. v1 → v2 = bump version, old readers regeneram. Sem migration chain.

### 2.8 Determinismo opt-in (`--features det-painter`)

```rust
#[cfg(feature = "det-painter")]
pub fn replay_stroke_det(stroke: &StrokeRecord, layer: &mut LayerTexture, brush: &Brush) {
    // Tudo CPU. Sem GPU. Sem fast-math. Sem FMA.
    // Stamps gerados em ordem fixa: por seq do stroke + ordem em points[].
    // RNG: ChaCha8Rng::seed_from_u64(stroke.rng_seed).
    // Math: f64 ops via `compiler_fence` para impedir FMA-fusion auto.
    // Output: bit-identical cross-OS (Linux x86_64 / macOS arm64 / Windows x86_64 / Web wasm32).
}
```

**Test gates (W11+):**
- `painter_replay_determinism` — replay no Linux + macOS + Windows + Web devolve blake3 idêntico para mesma `.ph2d-painter`.
- `painter_replay_q1616_roundtrip` — `q1616_to_f32` ∘ `f32_to_q1616` é identidade (no ULP drift).

**Modo padrão (GPU):** replay aproximado (mesma estética; pode diferir em alguns ULPs). Suficiente para Reproject visual mas **não para verificação cross-platform** — disclaimer no doc.

### 2.9 Reproject protocol (W12)

```rust
pub fn reproject_canvas(
    src: &PaintProject,
    target_size: (u32, u32),
    mode: ReprojectMode,                     // Det | Gpu
    progress: &mut dyn FnMut(ProgressEvent),
) -> Result<PaintProject, ReprojectError> { /* … */ }

pub enum ReprojectMode {
    /// CPU fallback bit-identical (HR-5 det-painter). ~5 strokes/s.
    Det,
    /// GPU stamp pipeline reaplicado. ~50 strokes/s. Aproximado (estética idêntica).
    Gpu,
    // === 2 slots de headroom ===
}
```

**Algoritmo:**
1. Criar canvas novo com `target_size` + mesmo color profile.
2. Para cada `stroke` em `history` (ordem `seq` crescente):
   1. Resolver `brush_params_hash` → `Brush` (lookup em `brush_snapshots`).
   2. Escalar `points[].x_q1616 / y_q1616` linearmente para `target_size` (multiplicação fixed-point exata).
   3. Re-pintar via stamp pipeline (`PainterTool::ingest_stroke_det()` em det mode).
3. Aplicar adjustment layers + masks no compositor final.

**Off-thread:** opera em thread dedicada (worker) com progress callback; canvas alvo só fica visível ao final.

**Cap:**
- `ReprojectMode` ≤ 4 variants
- `ReprojectError` ≤ 8 variants

### 2.10 Memory budget + extension em `ph2d-host::MemoryBudget` (autorizado por ADR-0043 §2.5)

Esta ADR amenda `ph2d-host::MemoryBudget` com um campo agregado de buckets Painter:

```rust
pub struct MemoryBudget {
    // ... campos existentes ...
    pub painter: PainterMemoryBudget,
}

pub struct PainterMemoryBudget {
    pub stroke_history_mb: u32,              // §2.5 tier selection input
    pub snapshot_cap_mb: u32,                // §2.6 LRU cap
    pub atlas_shape_mb: u32,                 // ADR-0044 §1.8.1
    pub atlas_grain_mb: u32,                 // idem
}
```

Cap: `PainterMemoryBudget ≤ 8 fields` (v1: 4). Plataformas runtime instanciam via host-side detection (memory probing + tier classification).

| Bucket | Budget default | Notas |
|---|---|---|
| Stroke history (`Vec<StrokeRecord>`) | 150 MB | ~20k strokes default; eviction NÃO automática (é history, não cache) |
| Snapshots (`LayerSnapshot`) | 200 MB (cap LRU) | offload disk sob pressure; metadata permanece RAM |
| Brush snapshots (dedup table) | 10 MB | ~1k brush variants no extremo; typically <100 KB |
| Atlas Shape (R8) | 4 MB | static após boot |
| Atlas Grain (R8) | 32 MB | static após boot (Procedural Grain reduziu de 64 MB; ADR-0044) |

Total Painter overhead estável: ~400 MB no extremo. Cabe em macOS desktop (16 GB) com folga; iPad Pro 8 GB OK; iPad 2018 (3 GB) → fallback `Ring { cap: 1000 }` + snapshot cap mais agressivo.

### 2.11 Arch-gate `painter_contract_surface::stroke_history`

Adicionado ao arquivo único `crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs` (homestead congelado per ADR-0043 §2.4; sem deps de runtime; audita texto cross-crate):

```rust
mod stroke_history {
    #[test] fn stroke_record_field_count_is_capped()        { /* ≤ 16 */ }
    #[test] fn raw_pointer_sample_field_count_is_capped()   { /* ≤ 12 */ }
    #[test] fn stroke_record_points_len_capped_at_u16()     { /* len ≤ 65535 — runtime + textual */ }
    #[test] fn tool_mode_variant_count_is_capped()          { /* ≤ 6 */ }
    #[test] fn stroke_history_variant_count_is_capped()     { /* ≤ 4 */ }
    #[test] fn layer_snapshot_field_count_is_capped()       { /* ≤ 8 */ }
    #[test] fn snapshot_storage_variant_count_is_capped()   { /* ≤ 4 */ }
    #[test] fn paint_project_field_count_is_capped()        { /* ≤ 12 */ }
    #[test] fn canvas_info_field_count_is_capped()          { /* ≤ 8 (§2.7.1) */ }
    #[test] fn paint_project_cache_field_count_is_capped()  { /* ≤ 8 — sidecar (§2.7.2) */ }
    #[test] fn reproject_mode_variant_count_is_capped()     { /* ≤ 4 */ }
    #[test] fn painter_memory_budget_field_count_is_capped() { /* ≤ 8 */ }
    #[test] fn stroke_id_is_uuid()                          { /* StrokeId == Uuid alias */ }
}
```

Localização **congelada** em `ph2d-painter-contracts` (audit C3, 2026-05-26): sem migração futura para outro crate; o homestead independente é o canon. Auditoria textual cross-crate via `walkdir + grep estrutural` — gate não importa tipos dos crates filhos em runtime.

### 2.12 Gates de comportamento

| Gate | Crate | Valida |
|---|---|---|
| `painter_persistence_roundtrip_v1` | ph2d-painter-stroke | Save → load → save produz `.ph2d-painter` byte-identical (postcard determinístico). |
| `painter_persistence_forward_compat_v1_to_v2` | idem | Quando v2 nascer, reader v2 lê v1 sem erro. |
| `painter_replay_determinism` | idem (feature `det-painter`) | Cross-OS blake3 idêntico para mesma fonte. |
| `painter_replay_q1616_roundtrip` | idem | Conversão fixed-point round-trip ULP-zero. |
| `painter_snapshot_lru_evicts_correctly` | idem | LRU eviction prioriza snapshots não-acessados. |
| `painter_undo_p99_under_200ms` | idem | Undo único @ 5000 strokes em layer 4K com snapshot a cada 50 ≤ 200 ms p99. |

---

## 3. Consequências

### Positivas

- **History completa habilita Reproject (W12), Inspector (W14), MCP (W13), replay determinístico.** Cada uma dessas features pesa muito; sem history vetorial, são impossíveis ou caras.
- **Schema congelado em postcard v1.** Migration chain documentada. `.ph2d-painter` v1 lê em v2+ for forever (HR-14).
- **Fixed-point Q16.16/Q8.8 desbloqueia HR-5 det-mode.** Replay cross-OS bit-identical é o que diferencia "engine séria" de "engine arrebatada".
- **Brush snapshots dedup** evita o `.ph2d-painter` explodir (60 MB → ~5 MB). Hash-addressing é canon (blake3 já no projeto).
- **`StrokeHistory::Ring` fallback gracioso** em low-end com warning UX honesto.

### Negativas / Custos

- **150 MB stroke history budget é caro.** Comparado ao ring 250 do design original (25 MB), 6× mais. Mitigação: É **history**, não cache — não evicta automaticamente. Usuária esperando isso (cf. Procreate `.procreate` chega a 150 MB facilmente).
- **Fixed-point Q16.16 limita coords a ±32768 px.** Cap suficiente para canvases até 16384×8192 (max suportado spec §2.5). Se algum dia precisar > 32k px, amend para Q24.8 (3 bytes coords) ou f64 quantizado.
- **`brush_params_hash` exige `Brush::params_blake3()` impl em `ph2d-painter-brush`.** Cross-crate dep documentada (`ph2d-painter-stroke → ph2d-painter-brush` para `BrushHandle` resolution at runtime, mas não para schema). Acceptable.
- **Snapshot LRU em disco precisa filesystem permission no Web/iOS.** Mitigação: IndexedDB no Web (≤ 1 GB típico); iOS sandbox app dir (multi-GB OK). Edge case Web private-browsing: cair pra ring 1000 + warning.
- **Q88 pressure 0..255 / 256.** Resolução de 0.0039 — perceptualmente abaixo de 1% pressure JND (Pencil 4096-level pressure tem JND ~0.025). Aceito.

### Neutras

- **`timestamp_ms` é wall-clock approx (não determinístico).** Usado só para time-lapse + audit log; **NÃO** entra no det-mode replay. Documentado em §2.2.
- **`SnapshotStorage::OnDisk` adiciona dep filesystem.** Já presente via `ph2d-host::storage_root()`; sem custo arquitetural novo.

---

## 4. Alternativas consideradas

### 4.1 f32 coords (sem fixed-point)

**Rejeitada.** HR-5 det-mode exige reproducibilidade cross-OS; f32 sin/cos/FMA divergem entre ARM/x86. Q16.16 + replay CPU = bit-identical. Custo de conversão runtime é desprezível (~ns por sample).

### 4.2 Ring buffer 250 strokes (design original Procreate-style)

**Rejeitada.** Mata Reproject, Inspector, MCP. Sabor Procreate é desejável em UX **não em arquitetura**. Spec [§1.14](../../Painter_projeto/01_brush_engine.md) já justifica o full-history como vantagem competitiva.

### 4.3 SQLite local em vez de postcard

**Rejeitada.** SQLite adiciona ~700 KB libs + thread-safety complexity. Postcard é stdlib-only, ~50 KB lib, e bate Procreate em I/O perf (medido em benches Wave 10). Indexed queries (Inspector W14) viram BTree em RAM com lookup direto.

### 4.4 Sem snapshots (undo re-run completo)

**Rejeitada.** Re-run 20k strokes para 1 undo = ~6 segundos (50 strokes/s GPU). Inaceitável. Snapshot a cada 50 strokes = ≤ 1 segundo p99. Custo: ~200 MB cache LRU. Aceito.

### 4.5 Brush copy-by-value em cada StrokeRecord

**Rejeitada.** ~5 KB / stroke × 20k strokes = 100 MB só de duplicação. Hash-dedup em tabela paralela reduz a ~50 KB no típico.

### 4.6 Schema JSON (legível human)

**Rejeitada.** `.ph2d-painter` é trafficked binary (mobile sync, cloud, share). Tamanho importa (JSON ~5× postcard). Legibilidade é commodity via tool `ph2d-painter-inspect` (W13+).

---

## 5. Verificação

```sh
cargo test -p ph2d-painter-stroke
# 8 caps + 6 behavior gates.

cargo test -p ph2d-painter-stroke --features det-painter
# Replay cross-OS determinismo (CI roda em Linux+macOS+Windows+Web).

cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
# 16-24 sub-tests cumulativos (ADR-0043 + 0044 + 0045 + 0046).
```

### 5.1 Definição de "Accepted"

Esta ADR transita `Proposed → Accepted` no mesmo evento T0.9.

---

## 6. Tracking

- Plano operacional: [§15 do plano §3 (T0.4) + §4 (W1) + §12 (W12)](../../Painter_projeto/15_plano_de_implementacao.md).
- Spec normativa: [`01_brush_engine.md §1.14`](../../Painter_projeto/01_brush_engine.md) + [`08 §8.2`](../../Painter_projeto/08_performance_memory.md) + [`09 export`](../../Painter_projeto/09_export_interop.md).
- ADRs irmãs dependentes: 0047 (MCP Stroke Engine) + 0048 (Stroke Inspector) consomem este schema.
- Próxima ADR na cascata W0: [ADR-0047 — Painter MCP Stroke Engine](0047-painter-mcp-stroke-engine.md) (T0.5).
