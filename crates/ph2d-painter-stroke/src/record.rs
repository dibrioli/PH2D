//! `StrokeRecord` — uma pincelada vetorial completa, gravada pra undo profundo,
//! Reproject, Inspector retroativo e MCP (ADR-0046 §2.2).
//!
//! Schema congelado por `painter_contract_surface::stroke_history`:
//! - [`StrokeRecord`] ≤ 16 fields
//! - [`RawPointerSample`] ≤ 12 fields (com `device_source` obrigatório, ADR-0050 §2.7)
//! - [`ToolMode`] ≤ 6 variants

use ph2d_color::OklchColor;
use ph2d_painter_brush::{BrushHandle, BrushParamsHash};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::device::{LayerId, PointerSource};

/// Identidade global de um stroke. ADR-0046 §2.2 — referenciado por
/// ADRs 0047 (MCP) e 0048 (Inspector).
pub type StrokeId = Uuid;

/// Pincelada vetorial completa.
///
/// Ordem cronológica é dada por `seq` (monotônico per-canvas);
/// `timestamp_ms` é wall-clock aproximado e **NÃO** entra em det-mode
/// replay (ADR-0046 §2.2).
///
/// Path cresce até [`MAX_SAMPLES_PER_STROKE`] (`u16::MAX`) — strokes mais
/// longos exigem split implícito no commit (T1.9+ implementation detail).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StrokeRecord {
    /// UUID v4 global do stroke. Estável pra MCP queries + Inspector refs.
    pub uuid: StrokeId,
    /// Monotônico per-canvas, ordem cronológica (replay/ undo anchor).
    pub seq: u64,
    /// Wall-clock approx em ms desde epoch. NÃO determinístico — só time-lapse + audit.
    pub timestamp_ms: u64,
    /// Brush opaco (built-in slot ou imported atlas layer) — ADR-0044 §2.8.
    pub brush_handle: BrushHandle,
    /// blake3 do `Brush` snapshot — dedup em `PaintProject::brush_snapshots`.
    ///
    /// **Audit T1.8 L4-H12 — type-alias warning:** `BrushParamsHash` é
    /// `pub type ... = [u8; 32]` em `ph2d-painter-brush` — zero type-level
    /// protection contra confusion com outros hashes `[u8; 32]`
    /// (`texture_blake3` em LayerSnapshot, `source_blake3` em cache).
    /// Caller DEVE passar `brush.params_blake3()` resultado — passar
    /// qualquer outro `[u8; 32]` compila silenciosamente mas quebra
    /// content-addressing. W11 polish: trocar alias por newtype
    /// `struct BrushParamsHash(pub [u8; 32])` cross-crate.
    pub brush_params_hash: BrushParamsHash,
    /// Qual raster layer recebe os stamps.
    pub layer_target: LayerId,
    /// Cor primária OKLCH no momento do stroke (ADR-0051 working space conversion runtime).
    pub primary_color: OklchColor,
    /// Cor secundária (Procreate long-press slot). `None` quando stroke não usou.
    pub secondary_color: Option<OklchColor>,
    /// Path BRUTO pré-stabilization — cap implícito `u16::MAX`. `Vec` não é
    /// typed-bounded; helper [`StrokeRecord::try_push_sample`] é o caminho
    /// oficial que enforça [`MAX_SAMPLES_PER_STROKE`]. **Audit T1.8 L3-G3:**
    /// não há gate runtime-enforcing em `Vec::push` direto; caller que
    /// pula `try_push_sample` é responsável por consultar `points_at_cap()`
    /// antes de empurrar (consumer code em T1.9+ MCP/dispatch DEVE usar
    /// o helper).
    pub points: Vec<RawPointerSample>,
    /// Seed RNG do stroke — anchor pra Color Dynamics + Procedural Grain det-mode.
    pub rng_seed: u64,
    /// Mode operacional (Paint/Smudge/Erase em v1).
    pub tool_mode: ToolMode,
    /// HR-14 schema version (v1 = 1).
    pub version: u32,
    // === 4 slots de headroom (MCP origin tag, audit-log ref, time-lapse marker, …) ===
}

/// Cap de amostras pré-stabilization em um stroke. Strokes > 65535 samples
/// exigem split (~22 min de fluid sim contínuo a 50 Hz). ADR-0046 §2.2.
pub const MAX_SAMPLES_PER_STROKE: usize = u16::MAX as usize;

impl StrokeRecord {
    /// Constrói um stroke vazio com defaults seguros — útil para tests e
    /// para o caminho MCP-streaming antes do primeiro sample chegar.
    pub fn new(
        seq: u64,
        brush_handle: BrushHandle,
        brush_params_hash: BrushParamsHash,
        layer_target: LayerId,
        primary_color: OklchColor,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            seq,
            timestamp_ms: 0,
            brush_handle,
            brush_params_hash,
            layer_target,
            primary_color,
            secondary_color: None,
            points: Vec::new(),
            rng_seed: 0,
            tool_mode: ToolMode::Paint,
            version: super::SCHEMA_VERSION,
        }
    }

    /// `true` se o stroke estouraria o cap quando o próximo sample for empurrado.
    /// Caller deve splitar antes (T1.9+ wires a policy de split).
    pub fn points_at_cap(&self) -> bool {
        self.points.len() >= MAX_SAMPLES_PER_STROKE
    }

    /// Empurra um sample com cap enforcement. Audit T1.8 L1-F3: `Vec::push`
    /// não respeita o cap u16; este helper é o caminho oficial pra
    /// adicionar samples (MCP, dispatch, replay). Caller que ignorar e usar
    /// `record.points.push(...)` direto torna-se responsável por checar
    /// [`Self::points_at_cap`].
    pub fn try_push_sample(&mut self, sample: RawPointerSample) -> Result<(), CapExceeded> {
        if self.points_at_cap() {
            return Err(CapExceeded::Samples {
                cap: MAX_SAMPLES_PER_STROKE,
            });
        }
        self.points.push(sample);
        Ok(())
    }
}

/// Erro de cap excedido em `StrokeRecord::try_push_sample` e operações
/// futuras que podem ultrapassar invariantes de tamanho.
///
/// **Audit T1.8 L4-H10:** virou `#[non_exhaustive] enum` (era struct) pra
/// permitir variants futuros (`BudgetMb`, `StrokeLengthSeconds`, etc.) sem
/// semver break em downstream.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CapExceeded {
    /// Excedeu [`MAX_SAMPLES_PER_STROKE`] em
    /// [`StrokeRecord::try_push_sample`].
    Samples { cap: usize },
}

impl CapExceeded {
    /// Helper retro-compat — `cap` antigo equivalia ao Samples::cap.
    pub const fn cap(&self) -> usize {
        match self {
            Self::Samples { cap } => *cap,
        }
    }
}

impl std::fmt::Display for CapExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Samples { cap } => write!(
                f,
                "StrokeRecord.points cap exceeded ({cap} samples max — split stroke or commit)"
            ),
        }
    }
}

impl std::error::Error for CapExceeded {}

/// Sample bruto do pointer pré-stabilization. Streamline / Stabilization
/// rodam runtime sobre estes samples; só os brutos são gravados (ADR-0046 §2.3).
///
/// Tamanho fixo 24 bytes — 256 samples = 6 KB / stroke típico.
///
/// **Fixed-point Q16.16/Q8.8**: HR-5 cross-OS bit-identical exige
/// representação que não dependa de IEEE754 fast-math drift. Conversão
/// roundtrip via [`crate::determinism`].
///
/// **Audit T1.8 L4-H17 — `Hash` derived:** habilita dedup de samples
/// consecutivos idênticos (Procreate-like sub-pixel delta filter) via
/// `HashSet` em W13 MCP path. Custo zero — POD 28 bytes.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RawPointerSample {
    /// x em Q16.16 (coord canvas, ±32768 px).
    pub x_q1616: i32,
    /// y em Q16.16 (coord canvas, ±32768 px).
    pub y_q1616: i32,
    /// Pressure em Q8.8, [0, 1.0).
    pub pressure_q88: u16,
    /// Tilt em Q8.8, [0, π/2).
    pub tilt_q88: u16,
    /// Azimuth em Q8.8, [0, 2π).
    pub azimuth_q88: u16,
    /// Barrel roll em Q8.8, [0, 2π); 0 se device não suporta.
    pub barrel_roll_q88: u16,
    /// Microseconds desde o sample anterior (replay tempo correto).
    pub timestamp_delta_us: u32,
    /// Bitset: bit0 = stylus_button1, bit1 = stylus_button2, bit2 = eraser_inverted, …
    pub flags: u32,
    /// Origem do pointer event (ADR-0050 §2.7). Stub `Unknown` enquanto
    /// `ph2d-painter-input` (T-input) não nasce; vide [`crate::device`].
    pub device_source: PointerSource,
    // === 3 slots de headroom (e.g. orientation, hover_distance, twist) ===
}

/// Mode operacional do stroke. `#[repr(u32)]` pra serialização postcard
/// estável + alinhamento bytemuck caso vire GPU buffer (W12 Reproject).
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ToolMode {
    #[default]
    Paint = 0,
    Smudge = 1,
    Erase = 2,
    // === 3 slots de headroom (Mask, LassoTemporalMark, …) ===
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_brush_hash() -> BrushParamsHash {
        let mut h = [0u8; 32];
        for (i, slot) in h.iter_mut().enumerate() {
            *slot = i as u8;
        }
        h
    }

    #[test]
    fn stroke_record_new_defaults_safe() {
        let rec = StrokeRecord::new(
            7,
            BrushHandle::default(),
            sample_brush_hash(),
            LayerId(3),
            OklchColor::opaque(0.6, 0.1, 200.0),
        );
        assert_eq!(rec.seq, 7);
        assert_eq!(rec.tool_mode, ToolMode::Paint);
        assert_eq!(rec.version, super::super::SCHEMA_VERSION);
        assert!(rec.points.is_empty());
        assert!(rec.secondary_color.is_none());
    }

    #[test]
    fn raw_pointer_sample_size_is_pinned_28_bytes() {
        // ADR-0046 §2.3 baseline: 24 bytes (sem device_source). T1.8 adiciona
        // device_source: PointerSource (`#[repr(u8)]` enum) — alinhamento
        // produz 28 bytes (u8 + 3 padding pra próximo i32 do struct
        // wrapper em testing / VEC layouts). Audit T1.8 L1-F7 exige pin
        // (em vez de `<= 32`) pra prevenir drift silencioso quando T-input
        // expandir PointerSource pra 16 variants reais — `#[repr(u8)]` mantém
        // o discriminant em 1 byte, padding preserva 28.
        //
        // Se este test falhar, AMEND ADR-0046 §2.3 explicitamente antes de
        // mudar o valor — memory budget ADR-0046 §2.10 ("20k strokes em
        // 150 MB" assume 24B; em 28B vira ~17k) precisa ser recalculado.
        assert_eq!(
            std::mem::size_of::<RawPointerSample>(),
            28,
            "RawPointerSample ABI drifted from 28 bytes — amend ADR-0046 §2.3 + §2.10 budget estimates"
        );
    }

    #[test]
    fn try_push_sample_rejects_at_cap() {
        // Audit T1.8 L1-F3: try_push_sample é o caminho oficial; Vec::push
        // direto ignora cap. Sentinel test guard a regressão.
        let mut rec = StrokeRecord::new(
            0,
            BrushHandle::default(),
            [0u8; 32],
            LayerId::default(),
            OklchColor::default(),
        );
        rec.points
            .resize_with(MAX_SAMPLES_PER_STROKE, RawPointerSample::default);
        let err = rec
            .try_push_sample(RawPointerSample::default())
            .unwrap_err();
        assert_eq!(err.cap(), MAX_SAMPLES_PER_STROKE);
        assert!(matches!(err, CapExceeded::Samples { .. }));
        assert_eq!(
            rec.points.len(),
            MAX_SAMPLES_PER_STROKE,
            "rejected sample must not append"
        );
    }

    #[test]
    fn try_push_sample_succeeds_below_cap() {
        let mut rec = StrokeRecord::new(
            0,
            BrushHandle::default(),
            [0u8; 32],
            LayerId::default(),
            OklchColor::default(),
        );
        for _ in 0..10 {
            rec.try_push_sample(RawPointerSample::default())
                .expect("below cap must succeed");
        }
        assert_eq!(rec.points.len(), 10);
    }

    #[test]
    fn points_at_cap_detects_boundary() {
        let mut rec = StrokeRecord::new(
            0,
            BrushHandle::default(),
            [0u8; 32],
            LayerId::default(),
            OklchColor::default(),
        );
        assert!(!rec.points_at_cap());
        // Sintetiza 1 sample shy do cap pra testar o boundary sem alloc gigante.
        rec.points.reserve(MAX_SAMPLES_PER_STROKE);
        // SAFETY: estamos só verificando contagem; preenchemos com defaults
        // (não há invariant runtime que exija conteúdo > Vec::len()).
        // Usamos resize_with pra evitar Vec::push N vezes (compile-time gate
        // sem rodar 65k iterations).
        rec.points
            .resize_with(MAX_SAMPLES_PER_STROKE, RawPointerSample::default);
        assert!(rec.points_at_cap());
    }

    #[test]
    fn tool_mode_repr_u32() {
        // ADR-0046 §2.4 — #[repr(u32)] estável pra postcard + bytemuck futuro.
        assert_eq!(std::mem::size_of::<ToolMode>(), 4);
        // Encoding discriminant fixo.
        assert_eq!(ToolMode::Paint as u32, 0);
        assert_eq!(ToolMode::Smudge as u32, 1);
        assert_eq!(ToolMode::Erase as u32, 2);
    }
}
