#![forbid(unsafe_code)]
//! `ph2d-painter-stroke` — Stroke Vector History (ADR-0046).
//!
//! O **vetor oculto do canvas**. Cada pincelada vira [`StrokeRecord`] com
//! path bruto em fixed-point Q16.16, brush handle + params hash, RNG seed e
//! tool mode. [`StrokeHistory`] é a fonte de verdade — habilita:
//!
//! - **Undo profundo** sem cap arbitrário (Procreate ring 250 → Full).
//! - **Reproject to Resolution (W12)** — re-pinta canvas em resolução nova
//!   sem blur de upscale.
//! - **Stroke Inspector retroativo (W14)** — lasso temporal + trocar brush/
//!   color de strokes passados.
//! - **MCP Stroke Engine (W13)** — LLM consome/modifica strokes editáveis.
//! - **Replay determinístico** — HR-5 opt-in via feature `det-painter`,
//!   bit-identical cross-OS (Linux x86_64 / macOS arm64 / Windows /
//!   Web wasm32).
//!
//! ## Persistence
//!
//! - [`persistence::PaintProject`] (canon `.ph2d-painter`, HR-14 versionado,
//!   forward-compat obrigatório).
//! - [`persistence::PaintProjectCache`] (sidecar `.ph2d-painter-cache`,
//!   regenerável, sem migration chain — `source_blake3` invalida cache se
//!   canon mudar).
//!
//! Separação congelada pelo audit 2026-05-26 C-3 — snapshots/spatial_index
//! são cache derivada (não-essencial pra reproduzir o canvas).
//!
//! ## Caps congelados (`painter_contract_surface::stroke_history`)
//!
//! - [`StrokeRecord`](record::StrokeRecord) ≤ 16 fields
//! - [`RawPointerSample`](record::RawPointerSample) ≤ 12 fields (com
//!   `device_source: PointerSource` obrigatório, ADR-0050 §2.7)
//! - [`ToolMode`](record::ToolMode) ≤ 6 variants
//! - [`StrokeHistory`](history::StrokeHistory) ≤ 4 variants
//! - [`LayerSnapshot`](snapshot::LayerSnapshot) ≤ 8 fields
//! - [`SnapshotStorage`](snapshot::SnapshotStorage) ≤ 4 variants
//! - [`PaintProject`](persistence::PaintProject) ≤ 12 fields
//! - [`CanvasInfo`](persistence::CanvasInfo) ≤ 8 fields
//! - [`PaintProjectCache`](persistence::PaintProjectCache) ≤ 8 fields
//! - [`ReprojectMode`](reproject::ReprojectMode) ≤ 4 variants
//! - [`PainterMemoryBudget`](budget::PainterMemoryBudget) ≤ 8 fields
//!   (gate lê `ph2d-host::PainterMemoryBudget` quando o move pra ph2d-host
//!   fechar — vide [`budget`] docstring)
//! - `pub type StrokeId = Uuid` (literal needle)
//!
//! ## Dependências cross-crate (ADR-0046 §2.1)
//!
//! ```text
//! ph2d-painter-stroke  ──→  ph2d-painter-brush  (BrushHandle, BrushParamsHash)
//! ph2d-painter-brush   ──→  (nada do Painter — folha da brush engine)
//! ```
//!
//! Stroke é NÃO-folha (depende de brush); brush é folha. Resolução
//! `BrushHandle → Brush` runtime (lookup em `Library`) acontece em
//! `ph2d-tool-painter` (consumer de ambos).
//!
//! ## Stubs temporários
//!
//! - [`device::PointerSource`] — stub `Unknown` enquanto `ph2d-painter-input`
//!   (T-input, ADR-0050) não nasce.
//! - [`device::PersistLayerId`] + [`device::PersistLayerStack`] — stubs até W3 layers crate.
//! - [`persistence::ColorProfile`] — stub `Srgb` enquanto T-color (ADR-0051)
//!   não materializa `ph2d-color::ColorProfile`.
//! - [`budget::PainterMemoryBudget`] — standalone até Coord-A move pra
//!   `ph2d-host::MemoryBudget { painter: PainterMemoryBudget }`.
//!
//! Cada stub documenta o owner e a migração esperada.

pub mod budget;
pub mod determinism;
// Audit T1.8 L4-H2 — `device` é módulo de stubs temporários (PointerSource +
// PersistLayerId + PersistLayerStack + CanvasId) que migrarão para `ph2d-painter-input`
// (T-input) e `ph2d-painter-layers` (W3). `#[doc(hidden)]` esconde de
// rustdoc public pra que consumers prefiram o caminho top-level re-exportado.
#[doc(hidden)]
pub mod device;
/// **T-durability (ADR-0052):** WAL + crash recovery + auto-save + suspend.
pub mod durability;
pub mod history;
pub mod persistence;
pub mod record;
pub mod reproject;
pub mod snapshot;

pub use budget::PainterMemoryBudget;
#[cfg(feature = "det-painter")]
pub use determinism::{DetReplayError, replay_stroke_det};
#[allow(deprecated)] // f32_to_q1616 alias preservado pra migração T1.9+
pub use determinism::{
    f32_to_q88, f32_to_q1616, f32_to_q1616_checked, f32_to_q1616_saturating, q88_to_f32,
    q1616_to_f32,
};
pub use device::{
    CanvasId, LAYER_FLAG_ACTIVE, LAYER_FLAG_ALPHA_LOCKED, LAYER_FLAG_CLIPPING,
    LAYER_FLAG_IS_REFERENCE, LAYER_FLAG_LOCKED, LAYER_FLAG_VISIBLE, LayerNode, LayerNodeKind,
    LayerStackEntry, MAX_LAYER_NODE_DESERIALIZE_DEPTH, PersistLayerId, PersistLayerStack,
    PointerSource,
};
pub use durability::{
    AtomicWriteError, AutoSave, AutoSaveError, AutoSavePolicy, AutoSaveState, CrashRecovery,
    DRAIN_DEADLINE_DEFAULT_MS, DRAIN_FLUSH_WAL_PHASE_MS, DRAIN_WRITE_CANON_PHASE_MS, FlushPolicy,
    JOURNAL_MAGIC, JOURNAL_ROTATE_BYTES, JOURNAL_ROTATE_COMMITS, JournalError, PartialStroke,
    RecoveredStroke, RecoveryError, RecoveryState, STORAGE_SAFETY_MULTIPLIER, StrokeJournal,
    SuspendHandler, SuspendState, WalEntryRaw, WalEntryType, atomic_write, read_journal,
};
pub use history::{
    FULL_HISTORY_MIN_BUDGET_MB, RING_CAP_LOW_TIER, RING_CAP_MID_TIER, RING_MIN_CAP, StrokeHistory,
    StrokeHistoryIter,
};
pub use persistence::{
    CanvasInfo, ColorProfile, LoadError, MAX_BRUSH_SNAPSHOTS, MAX_CANON_BYTES, MAX_GROUP_DEPTH,
    MAX_LAYER_NAME_BYTES, MAX_LAYERS, MAX_RESERVED_PAYLOAD, MAX_SNAPSHOTS_PER_CACHE,
    MAX_STROKES_PER_CANON, PAINT_PROJECT_CACHE_MAGIC, PAINT_PROJECT_MAGIC, PaintProject,
    PaintProjectCache, SaveError, SerializedRTree, apply_migrations, load, save,
    validate_caps_post_deserialize,
};
pub use record::{
    CapExceeded, MAX_SAMPLES_PER_STROKE, RawPointerSample, SAMPLE_FLAG_AZIMUTH_UNAVAILABLE,
    SAMPLE_FLAG_BARREL_ROLL_UNAVAILABLE, SAMPLE_FLAG_STROKE_SPLIT_BOUNDARY,
    SAMPLE_FLAG_TILT_UNAVAILABLE, SAMPLE_FLAG_TIMESTAMP_UNAVAILABLE, StrokeId, StrokeRecord,
    ToolMode,
};
pub use reproject::{ProgressEvent, ReprojectError, ReprojectMode, reproject_canvas};
pub use snapshot::{
    LayerSnapshot, SNAPSHOT_STROKE_INTERVAL, SNAPSHOT_VERSION, SnapshotPathError, SnapshotStorage,
};

/// HR-14 schema version do canon savefile (`.ph2d-painter`).
/// - v1 = stub `LayerStackEntry::Reserved` (sem modelo de layer).
/// - **v2 (W3, ADR-0046-amendment-1)** = `LayerStackEntry::Node` real (layer
///   stack persistível). Files v1 migram via [`persistence::migrate_v1_to_v2`].
/// - **v3 (ADR-0096)** = remoção da aquarela/fluido do `RenderingParams`
///   (`watercolor`/`fluid_enabled`/`wash_enabled` saíram do `Brush` serializado).
///   QUEBRA DURA: postcard é posicional, logo projetos pré-v3 com brush não-default
///   NÃO carregam (falham no deserialize/checksum). Sem migration possível — não há
///   como re-ler os bytes antigos sem o struct antigo. Decisão "começar do zero".
///
/// Bump quando schema do `PaintProject` mudar; cada bump exige um migration helper
/// sequencial em `persistence` (exceto quebras duras documentadas, como o v3).
pub const SCHEMA_VERSION: u32 = 3;
