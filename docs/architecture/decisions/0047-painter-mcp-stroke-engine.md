# ADR-0047 — Painter MCP Stroke Engine (4 tools + governance HR-11)

**Status:** Accepted (2026-05-26)
**Decisor(es):** Enio + Claude (Coord-A, sessão Painter W0).
**Pré-requisitos:** [ADR-0043 — Painter contract](0043-painter-contract.md), [ADR-0044 — Brush Engine GPU](0044-brush-engine-gpu.md), [ADR-0046 — Stroke Vector History](0046-stroke-vector-history.md).
**Spec normativa:** [`docs/Painter_projeto/01_brush_engine.md`](../../Painter_projeto/01_brush_engine.md) §1.13.
**Relaciona:** [ADR-0048 — Stroke Inspector](0048-stroke-inspector.md) (compartilha `StrokeRecord` source via ADR-0046).
**Tags:** painter, wave-0, contract, mcp, llm, hr-11-governance, audit-log

---

## 1. Contexto

Paint stacks tradicionais expõem APIs procedurais para LLMs gerarem **pixels colados como layer** (`Layer::set_pixels(png_bytes)`). Resultado: o artista humano herda um raster opaco — não-editável, não-restilizável, não-rebrush-able.

PH2D Painter eleva HR-10 (LLM friendly) ao expor **strokes reais**: o LLM constrói `Vec<StrokeSpec>` (path + pressure + brush ref + color) e o engine executa pelo **mesmo stamp pipeline** que o artista usa. Resultado: 100% editável traço-a-traço via Stroke Inspector (W14), via Reproject (W12), via undo/redo, via troca retroativa de brush.

Sem contrato congelado:

1. **MCP tool surface vira "qualquer parâmetro que pareça útil".** Caps amorfos.
2. **HR-11 governance fica implícita.** Cada implementação reinventa "como pedir token, como logar audit".
3. **`StrokeSpec` ≠ `StrokeRecord` divergem.** LLM gera input com schema A; history grava schema B; código de conversão vira opaco.
4. **Quality emerges via prompting (§1.13.5) ≠ "engine garante qualidade".** Sem contrato técnico, fica confuso onde reside cada responsabilidade.

ADR-0046 §2 mencionou esta ADR como consumer downstream — esta cobre o **contrato MCP**.

---

## 2. Decisão

### 2.1 Crate `ph2d-painter-mcp` (W13)

```
crates/ph2d-painter-mcp/
  Cargo.toml         # deps: ph2d-painter-stroke (schemas), ph2d-painter-brush (BrushHandle),
                     #        ph2d-color, serde, blake3, mcp-rs (TBD external crate)
  src/lib.rs         # #![forbid(unsafe_code)] PRIMEIRO
  src/tools.rs       # 4 #[mcp_tool] entrypoints
  src/spec.rs        # StrokeSpec + StrokePoint + StrokeMods + StrokeFilter + StrokeRef
  src/conversion.rs  # StrokeSpec -> StrokeRecord (the bridge)
  src/governance.rs  # HR-11 token + audit log
  tests/             # round-trip + governance + caps
```

**Dep isolation:** `ph2d-painter-mcp` é folha — nenhum outro crate depende dele. Isso permite incluir/excluir MCP do build via feature flag (`--features mcp` no shell). Mobile/Web buildings sem MCP shippable.

**Razão de crate separado:** §1.13.4 spec estima ~250 LOC v1. Não justifica fundir em `ph2d-tool-painter` nem `ph2d-painter-stroke` — MCP traz deps externas (`mcp-rs`, eventual JSON schema validation) que não pertencem ao hot path do brush engine.

### 2.2 4 MCP tools — cap **≤ 8 tools**

v1 ship 4 tools (vide §1.13.1 da spec). Cap = **8** (4 slots para evolução W13+).

```rust
// Pinta sequência de strokes reais na layer alvo. Cria N novos StrokeRecord
// e retorna seus StrokeId. Engine reaplica strokes pelo mesmo stamp pipeline
// que o artista usa (HR-10).
#[mcp_tool(name = "painter_paint_strokes", destructive = true)]
pub fn painter_paint_strokes(
    canvas_id: CanvasId,
    layer_id: LayerId,
    brush_handle: BrushHandle,
    strokes: Vec<StrokeSpec>,
    confirmation_token: Token,
) -> Result<Vec<StrokeId>, McpError>;

// Modifica stroke existente retroativamente (par com Inspector W14 — ADR-0048).
#[mcp_tool(name = "painter_modify_stroke", destructive = true)]
pub fn painter_modify_stroke(
    stroke_id: StrokeId,
    mods: StrokeMods,
    confirmation_token: Token,
) -> Result<(), McpError>;

// Query strokes na layer ativa. Read-only — sem token.
#[mcp_tool(name = "painter_query_strokes")]
pub fn painter_query_strokes(
    canvas_id: CanvasId,
    layer_id: LayerId,
    filter: StrokeFilter,
) -> Vec<StrokeRef>;

// Inspeção fina de um stroke. Read-only — sem token. Retorna o StrokeRecord
// canônico (schema ADR-0046).
#[mcp_tool(name = "painter_inspect_stroke")]
pub fn painter_inspect_stroke(stroke_id: StrokeId) -> Result<StrokeRecord, McpError>;
```

**Política `destructive: true`:**
- Acessível **só com** `confirmation_token` (HR-11 — single-use, 5 min TTL, gerado por flow humano fora-de-banda) **OU** flag servidor `--unsafe-mcp` (auditável; equivalente a "auto-aprovar tudo").
- Read-only tools (`query`, `inspect`) **não** exigem token.

### 2.3 `StrokeSpec` — cap **≤ 8 fields** (v1 usa 5)

Schema **input do LLM** — distinto de `StrokeRecord` (schema **interno**, ADR-0046). LLM trabalha em `f32` (mais simples); engine converte para fixed-point Q16.16 antes de gravar history (§2.7).

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StrokeSpec {
    pub points: Vec<StrokePoint>,            // path bruto (cap §2.4); len ≤ 65535 (ADR-0046 §2.2)
    pub primary_color: OklchColor,
    pub secondary_color: Option<OklchColor>,
    pub tool_mode: ToolMode,                 // Paint | Smudge | Erase (ADR-0046)
    pub rng_seed: Option<u64>,               // determinismo opt-in (None = engine escolhe)
    pub version: u32,                        // HR-14 schema versioning — v1 = 1
    // === 2 slots de headroom ===
}
```

**`version: u32` é OBRIGATÓRIO** (audit A3, 2026-05-26): LLM-facing schema serializável precisa de migration chain. Breaking changes futuros (e.g., adicionar `barrel_roll`) viram v2; reader engine sabe migrar v1 → v2.

### 2.4 `StrokePoint` — cap **≤ 8 fields** (v1 usa 5)

```rust
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StrokePoint {
    pub position: [f32; 2],                  // x, y in canvas coords (engine quantiza para Q16.16)
    pub pressure: f32,                       // [0, 1] (engine quantiza para Q8.8)
    pub tilt: f32,                           // [0, π/2]
    pub azimuth: f32,                        // [0, 2π)
    pub timestamp_ms: u32,                   // ms desde primeiro point do stroke (delta-encoded internally)
    // === 3 slots de headroom (e.g., barrel_roll, button bitmask) ===
}
```

**`f32` aqui é deliberado.** LLM-facing API; tradução para Q16.16 ocorre em `conversion.rs::spec_to_record(spec) -> StrokeRecord` antes de gravar `StrokeHistory` (ADR-0046 §2.3).

### 2.5 `StrokeMods` — cap **≤ 8 fields** (v1 usa 5)

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct StrokeMods {
    pub new_brush: Option<BrushHandle>,
    pub new_color: Option<OklchColor>,
    pub pressure_scale: Option<f32>,         // multiplica todas as pressures (escala global)
    pub path_offset: Option<[f32; 2]>,       // desloca todos os pontos por (dx, dy)
    pub path_replace: Option<Vec<StrokePoint>>, // substituição completa do path
    // === 3 slots de headroom (e.g., new_pressure_curve, new_tilt_remap) ===
}
```

**Semantic:** todos os fields são `Option`; `None` = "não modifica". Aplicação ordem fixa: `new_brush → new_color → pressure_scale → (path_offset XOR path_replace)`. `path_offset` e `path_replace` são mutex (XOR runtime check).

### 2.6 `StrokeFilter` — cap **≤ 8 fields** (v1 usa 5)

Filtros AND-combined para `painter_query_strokes`:

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct StrokeFilter {
    /// Bbox (canvas-space). Stroke matches se QUALQUER ponto cai dentro do bbox.
    pub bbox: Option<Rect>,
    /// Brush ref. Stroke matches se `brush_handle == this`.
    pub brush: Option<BrushHandle>,
    /// Color similarity threshold em OKLab (ΔE). Stroke matches se ΔE(primary, this.0) <= this.1.
    pub color_near: Option<(OklchColor, f32)>,
    /// Range de timestamps (ms desde canvas creation).
    pub timestamp_range: Option<(u64, u64)>,
    /// Tool mode.
    pub tool_mode: Option<ToolMode>,
    // === 3 slots de headroom (e.g., seq_range, layer_id filter, opacity_above) ===
}
```

### 2.7 `StrokeRef` (output leve de `painter_query_strokes`)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrokeRef {
    pub stroke_id: StrokeId,                 // = Uuid
    pub seq: u64,
    pub bbox: Rect,                          // computed na query (cached em StrokeRecord index)
    pub brush_handle: BrushHandle,
    pub primary_color: OklchColor,
    // 5 fields fixos — não cresce sem amendment.
}
```

Output denso (`StrokeRecord` completo) é via `painter_inspect_stroke`.

### 2.8 `McpError` enum — cap **≤ 12 variants** (v1 usa 8)

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum McpError {
    InvalidCanvas(CanvasId),
    InvalidLayer(LayerId),
    InvalidStroke(StrokeId),
    InvalidBrush(BrushHandle),
    MissingConfirmationToken,
    InvalidConfirmationToken,
    TokenExpired,
    StrokesEmpty,                            // strokes.is_empty()
    StrokesTooLarge { count: usize, cap: usize }, // batch cap (§2.10)
    PathReplaceAndOffsetConflict,
    LayerLocked(LayerId),
    Internal(String),                        // catch-all com contexto; reservado pra impl
    // === 4 slots de headroom ===
}
```

### 2.9 `Conversion` — `StrokeSpec ⇒ StrokeRecord`

```rust
pub fn spec_to_record(
    spec: StrokeSpec,
    layer_target: LayerId,
    brush_handle: BrushHandle,
    brush_params_hash: BrushParamsHash,
    seq: u64,
) -> StrokeRecord {
    StrokeRecord {
        uuid: Uuid::new_v4(),
        seq,
        timestamp_ms: now_ms(),
        brush_handle,
        brush_params_hash,
        layer_target,
        primary_color: spec.primary_color,
        secondary_color: spec.secondary_color,
        points: spec.points.into_iter().map(spec_point_to_raw).collect(),
        rng_seed: spec.rng_seed.unwrap_or_else(rand::random),
        tool_mode: spec.tool_mode,
        version: 1,
    }
}

fn spec_point_to_raw(p: StrokePoint) -> RawPointerSample {
    RawPointerSample {
        x_q1616: f32_to_q1616(p.position[0]),
        y_q1616: f32_to_q1616(p.position[1]),
        pressure_q88: f32_to_q88(p.pressure),
        tilt_q88: f32_to_q88(p.tilt),
        azimuth_q88: f32_to_q88(p.azimuth),
        barrel_roll_q88: 0,                  // MCP v1 não expõe (slot reservado)
        timestamp_delta_us: (p.timestamp_ms * 1000),
        flags: 0,
    }
}
```

**Determinismo:** se `spec.rng_seed = Some(s)`, replay HR-5 funciona; se `None`, engine usa `rand::random()` e replay diverge (documentado).

### 2.10 Governance HR-11

**Confirmation token:**

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token(pub Uuid);

pub struct TokenRegistry {
    issued: BTreeMap<Token, IssuedToken>,    // HR-5: BTreeMap, não HashMap
}

pub struct IssuedToken {
    pub agent_id: String,                    // qual LLM/MCP-client pediu
    pub purpose: String,                     // contexto humano: "hatching test"
    pub issued_at_ms: u64,
    pub expires_at_ms: u64,                  // issued_at + 5 min default
    pub single_use: bool,                    // default true
    pub consumed: bool,
}

impl TokenRegistry {
    pub fn issue(agent_id: &str, purpose: &str) -> Token { /* … */ }
    pub fn consume(token: Token) -> Result<(), McpError> { /* validates expiry + single-use */ }
}
```

Token issuance é **side-channel humano** (CLI prompt + UI dialog). Não é gerado por LLM. Aceitável também: flag `--unsafe-mcp` no servidor de desenvolvimento (auditável).

**Batch cap + streaming protocol:** `painter_paint_strokes` aceita batches grandes via **streaming chunk protocol** (sem cap arbitrário em strokes count; cap em **chunk latency**):

- **Per-chunk cap: ≤ 500 strokes** (latency-bounded, ~3-5s p99 paint pipeline em 4K canvas).
- **Sessão grande: chunks múltiplos** sequencial dentro do mesmo `painter_paint_strokes` call.
- **API atualizada:** `painter_paint_strokes` retorna `StreamHandle` em vez de `Vec<StrokeId>` síncrono:

```rust
#[mcp_tool(name = "painter_paint_strokes", destructive = true, streaming = true)]
pub fn painter_paint_strokes(
    canvas_id: CanvasId,
    layer_id: LayerId,
    brush_handle: BrushHandle,
    strokes: Vec<StrokeSpec>,                // qualquer tamanho; engine chunka
    confirmation_token: Token,
) -> StreamHandle<StrokeProgressEvent>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StrokeProgressEvent {
    ChunkStarted { chunk_index: u32, chunk_size: u32 },
    StrokeCompleted { stroke_id: StrokeId, chunk_index: u32 },
    ChunkCompleted { chunk_index: u32, strokes_committed: u32 },
    BatchCompleted { total_strokes: u32, total_duration_ms: u64 },
    Error { chunk_index: u32, error: McpError },
}
```

Audit M-10 (2026-05-26) flagged cap original 1000 arbitrário. Audit sense-check veterano flagged latency: "5000 strokes × 256 points × JSON-RPC = 30+s antes do paint pipeline tocar". **Solução streaming:** chunks ≤ 500 strokes; LLM e UI consomem progress events em real-time; UI never freezes.

**Backpressure:** se UI thread saturada (frame drops > 5 consecutivos), engine pausa chunking; resume quando UI responsiva. `StreamHandle` cancel-able via `painter_cancel_stroke_batch(stream_handle)` (5ª MCP tool).

### 2.11 Audit log — JSON Lines schema congelado

```jsonc
// audit.log entry (JSON line per call)
{
  "ts_ms": 1748208000000,
  "event": "painter_paint_strokes",
  "agent_id": "claude-opus-4-7",
  "purpose": "hatching from selection",
  "canvas_id": "01J3KR2P...",
  "layer_id": "...",
  "brush_handle": 17,
  "strokes_count": 87,
  "blake3_layer_before": "abcd...32 hex...",
  "blake3_layer_after":  "ef01...32 hex...",
  "stroke_ids": ["uuid1", "uuid2", "..."],
  "token_uuid": "uuid",
  "duration_ms": 421
}
```

**Cap:** `AuditLogEntry ≤ 16 fields`. JSON Lines (newline-delimited) — append-only, never rewrite. Localização: `<storage_root>/painter_audit.log`. Rotação: a cada 100 MB roll para `.1`, `.2` (cap 5 ⇒ ~500 MB).

### 2.12 Caps de tamanho

| Tipo | Cap |
|---|---|
| `StrokeSpec` | ≤ 8 fields |
| `StrokePoint` | ≤ 8 fields |
| `StrokeMods` | ≤ 8 fields |
| `StrokeFilter` | ≤ 8 fields |
| `StrokeRef` | = 5 fields (frozen, sem headroom) |
| `McpError` | ≤ 12 variants |
| `Token` | = 1 field (Uuid newtype) |
| `IssuedToken` | ≤ 8 fields |
| `AuditLogEntry` | ≤ 16 fields |
| MCP tools count | ≤ 8 (v1 = 5: paint/modify/query/inspect/cancel_batch) |
| **Chunk cap (`paint_strokes`)** | ≤ 500 strokes/chunk (latency-bounded; total batch sem cap arbitrário) |
| Chunk p99 latency target | ≤ 5s @ 4K canvas |
| `StrokeMods.path_replace` cap | 4096 points |
| `StrokeSpec.points.len()` | ≤ 65535 (ADR-0046 §2.2 — herda Q16.16 fixed-point limit) |
| `StreamHandle` fields | ≤ 4 (v1 = 3) |
| `StrokeProgressEvent` variants | ≤ 8 (v1 = 5) |

### 2.13 Quality emerges via prompting (responsabilidade-fora-do-contrato)

§1.13.5 da spec: **esta ADR cobre o contrato técnico**. Qualidade estética dos strokes que LLMs geram é trabalho contínuo de:

1. Prompts de sistema com exemplos de bom hachuring/washing/lineart.
2. Fine-tunes específicos para Painter (futuro).
3. System prompts que ensinam ao LLM os 12 default brushes (§1.6) e seus comportamentos.

ADR-0047 **não** garante qualidade — garante **superfície estável** para qualidade emergir.

### 2.14 Arch-gate `painter_contract_surface::mcp`

Adicionado ao arquivo compartilhado `crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs` (homestead congelado per ADR-0043 §2.4):

```rust
mod mcp {
    #[test] fn mcp_tools_count_is_capped()                  { /* ≤ 8 */ }
    #[test] fn stroke_spec_field_count_is_capped()          { /* ≤ 8 */ }
    #[test] fn stroke_point_field_count_is_capped()         { /* ≤ 8 */ }
    #[test] fn stroke_mods_field_count_is_capped()          { /* ≤ 8 */ }
    #[test] fn stroke_filter_field_count_is_capped()        { /* ≤ 8 */ }
    #[test] fn stroke_ref_field_count_is_exact_5()          { /* frozen */ }
    #[test] fn mcp_error_variant_count_is_capped()          { /* ≤ 12 */ }
    #[test] fn audit_log_entry_field_count_is_capped()      { /* ≤ 16 */ }
}
```

### 2.15 Gates de comportamento

| Gate | Crate | Valida |
|---|---|---|
| `mcp_paint_strokes_round_trip` | ph2d-painter-mcp | `StrokeSpec → record → query → ref → inspect` recupera spec equivalente (módulo timestamp e uuid). |
| `mcp_paint_strokes_requires_token` | idem | Call sem `confirmation_token` válido → `MissingConfirmationToken`. |
| `mcp_paint_strokes_chunks_at_500` | idem | Batch > 500 strokes é automaticamente chunked em ≤ 500/chunk. Sem `StrokesTooLarge` por tamanho de batch. |
| `mcp_paint_strokes_chunk_p99_under_5s` | idem | Cada chunk completa pipeline em ≤ 5s p99 @ 4K canvas. Hard desde W13. |
| `mcp_paint_strokes_stream_progress_events` | idem | Cliente MCP recebe `ChunkStarted` / `StrokeCompleted` / `ChunkCompleted` / `BatchCompleted` em ordem; UI consome em real-time. |
| `mcp_paint_strokes_backpressure_pauses_chunking` | idem | UI frame drops > 5 consecutivos → engine pausa próximo chunk; resume quando UI responsiva. |
| `mcp_cancel_stroke_batch_aborts_mid_stream` | idem | `painter_cancel_stroke_batch(handle)` interrompe stream; commits parciais permanecem em history. |
| `mcp_modify_stroke_path_offset_replace_mutex` | idem | `StrokeMods { path_offset: Some, path_replace: Some }` → `PathReplaceAndOffsetConflict`. |
| `mcp_audit_log_appended_per_call` | idem | Após call destructive, audit.log ganha 1 nova linha JSON válida. |
| `mcp_query_strokes_filter_combinations` | idem | Filtros AND-combined (bbox + brush + color_near) retornam ∩ correto. |
| `mcp_determinism_with_rng_seed` | idem (det-painter) | Mesmo `StrokeSpec` com `rng_seed=Some(s)` replay produz pixel-identical. |

---

## 3. Consequências

### Positivas

- **HR-10 elevada concretamente.** LLM gera strokes editáveis, não pixels colados. Diferenciação técnica genuína vs Procreate (§14.8.1).
- **Schemas `StrokeSpec` (input LLM, f32) vs `StrokeRecord` (interno, Q16.16) decouplados.** Conversion explícita em `conversion.rs`; LLM nunca mexe com fixed-point.
- **HR-11 governance explícita.** Token + audit log isolam destructive operations. Sem "qualquer call MCP modifica canvas".
- **`ph2d-painter-mcp` é folha + feature flag opcional.** Mobile/Web buildings sem MCP exclusos via `--no-default-features`.
- **Caps em todos os schemas.** Nenhum cresce sem amendment.
- **Quality emerge via prompting (responsabilidade fora-do-contrato).** Engineering separation clean.

### Negativas / Custos

- **Token issuance é human-in-the-loop.** Não automatável em CI. Mitigação: `--unsafe-mcp` para test envs; tokens via env var em dev runs.
- **Streaming protocol** elimina o trade-off batch-cap-arbitrário. Batches grandes (50k strokes) viram chunks ≤ 500 com progress events. UI nunca freeze; LLM workflow real-time. Audit sense-check 2026-05-26 closed.
- **Audit log cresce 5 MB/dia em uso pesado.** Rotação automática (100 MB → roll) limita a ~500 MB total. Aceito.
- **Conversion `f32 → Q16.16` introduz quantização determinística.** ULP drift possível em coords não-grid-aligned. Garantido só para coords < 32768 (limite Q16.16). Spec já cap canvas em 16384×8192 (§2.5 02_layers.md).
- **`StrokeMods.path_replace` cap 4096 points.** Pode bater em strokes muito longos (~30 segundos sem release). Bypass: split em múltiplas operações `mod_stroke`. Edge case raríssimo.

### Neutras

- **`StrokeRef` é frozen em 5 campos.** Sem headroom = sem evolução sem amendment explícita. Trade-off: estabilidade externa do schema serializável (LLM consome `StrokeRef` JSON; mudar é breaking change).
- **Crate separado `ph2d-painter-mcp`.** +1 crate na fan-out matrix, +0 dep no core. Aceito.

---

## 4. Alternativas consideradas

### 4.1 Fundir `ph2d-painter-mcp` em `ph2d-tool-painter`

**Rejeitada.** MCP traz `mcp-rs` (TBD external crate) + JSON Schema validator. Mobile/Web buildings sem MCP precisam excluir essas deps; cleanest via crate boundary + feature flag no shell.

### 4.2 `StrokeSpec` usa `Q16.16` direto (sem conversion)

**Rejeitada.** LLM nativos trabalham em `f32` em prompts; força LLM a quantizar é fricção sem ganho — engine já quantiza no `spec_to_record`. Decoupling input vs internal é canon.

### 4.3 Token automático (LLM gera próprio token)

**Rejeitada.** Subverte HR-11 inteiramente. Token humano é o **mecanismo** de governance. Caso CI: `--unsafe-mcp` flag explícita é a porta auditável.

### 4.4 8 tools no v1 (em vez de 4)

**Rejeitada.** §1.13.4 spec estima 250 LOC para os 4. Mais ferramentas sem casos de uso mapeados = surface bloat. Cap 8 deixa 4 slots; expansão por amendment quando necessidade emergir.

### 4.5 Audit log binário (postcard)

**Rejeitada.** JSON Lines é grep-able + tail-able + ferramenta-able (jq, splunk, datadog). Audit log é texto humano por contrato. Tamanho não é problema (rotação).

### 4.6 `painter_paint_strokes` retorna `Vec<StrokeRecord>` completo (não `Vec<StrokeId>`)

**Rejeitada.** Retorno grande (10 KB/record × 1000 = 10 MB) em response MCP. Caller usa `painter_inspect_stroke(id)` se precisar detalhe. Round-trip mais limpo.

---

## 5. Verificação

```sh
cargo test -p ph2d-painter-mcp
# 8 caps + 7 behavior gates.

cargo test -p ph2d-painter-mcp --features det-painter
# Determinismo end-to-end com rng_seed.

cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
# Caps cumulativos (ADRs 0043+0044+0045+0046+0047).
```

### 5.1 Definição de "Accepted"

Esta ADR transita `Proposed → Accepted` no mesmo evento T0.9.

---

## 6. Tracking

- Plano operacional: [§15 do plano §3 (T0.5) + §13 (W13)](../../Painter_projeto/15_plano_de_implementacao.md).
- Spec normativa: [`01_brush_engine.md §1.13`](../../Painter_projeto/01_brush_engine.md).
- HR-11 governance reference: [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) §9 (Hard Rules).
- Próxima ADR na cascata W0: [ADR-0048 — Stroke Inspector retroativo](0048-stroke-inspector.md) (T0.6).
