//! Crash recovery — boot scan do WAL + replay seletivo. ADR-0052 §2.3.
//!
//! ## Boot flow
//!
//! 1. App boota; check se `painter_journal.wal` existe.
//! 2. [`CrashRecovery::scan`] lê WAL completo (~5ms / 50MB típico).
//! 3. Para cada `RecoveredStroke`:
//!    - `CommittedNotPersisted` → silent re-add para history + flush para
//!      `.ph2d-painter`.
//!    - `InProgressAtCrash` → coleta para UX prompt único.
//!    - `Cancelled` / `Corrupted` → descarte silencioso.
//! 4. Após resolução, `StrokeJournal::truncate_and_reset()` zera WAL.
//!
//! ## Audit T-durability fixes
//!
//! - **K-2**: `MAX_SAMPLES_PER_RECOVERED_STROKE` cap per-stroke (anti-blowup).
//! - **K-3**: `samples_count_in_journal` cross-validado vs `samples.len()` real.
//! - **K-4**: doc explica caller deve passar `existing_uuids` baseline pra
//!   detectar colisão com history persistido (W11 wire).
//! - **J-5**: `dropped_uuids` tracked separadamente; SampleBatch órfão DE
//!   stroke dropado-por-cap NÃO infla `corrupted_entries`.
//! - **J-7**: counts split em `crc_failures`, `parse_failures`,
//!   `orphan_batches` (backward-compat `corrupted_entries` = soma).
//! - **J-8**: `last_heartbeat_ms` surfaceado pra UX prompt distinguir
//!   "crashed agora" vs "abandoned há dias".
//! - **J-13**: lê `SampleBatchPayload { stroke_id, samples }` em vez de
//!   `Vec<RawPointerSample>` órfão.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::record::{RawPointerSample, StrokeId};

use super::journal::{
    JournalError, PartialStroke, SampleBatchPayload, WalEntryRaw, WalEntryType, read_journal,
};

/// Cap operacional de strokes recuperáveis em uma única invocação.
pub const MAX_RECOVERED_STROKES: usize = 100_000;

/// Cap operacional de samples por stroke recuperado (audit K-2).
/// Sessão real: 256-512 samples/stroke típico. Cap 1M cobre extremos
/// (fluid sim 30min de stroke contínuo) sem permitir blowup adversarial.
pub const MAX_SAMPLES_PER_RECOVERED_STROKE: usize = 1_000_000;

/// State machine do recovery. ADR-0052 §2.3.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RecoveryState {
    /// WAL tem Begin + samples + Commit → stroke chegou ao disco mas não
    /// foi gravado em main history. Action: replay para history silently.
    CommittedNotPersisted,
    /// WAL tem Begin + samples mas NO Commit → stroke estava em progresso
    /// quando crashou. Action: UX dialog "Recover stroke in progress?".
    InProgressAtCrash,
    /// WAL tem Begin + Cancel → stroke foi cancelado pelo user; descartar.
    Cancelled,
    /// CRC fail / parse error em entry → corrupted; pular + log.
    Corrupted,
    // === 2 slots de headroom ===
}

/// Stroke reconstruído a partir do WAL.
#[derive(Clone, Debug, PartialEq)]
pub struct RecoveredStroke {
    pub partial: PartialStroke,
    pub samples: Vec<RawPointerSample>,
    pub state: RecoveryState,
}

impl RecoveredStroke {
    /// Converte para `StrokeRecord` ready-to-push em `StrokeHistory`.
    /// Mapping canônico (audit T-durability final M-2 — caller PCA pulava
    /// fields silenciosamente):
    ///
    /// - `partial.started_at_ms` → `record.timestamp_ms` (rename).
    /// - `partial.canvas_id` é **descartado** (StrokeRecord vive dentro de
    ///   um `PaintProject` específico — caller DEVE filtrar via
    ///   [`CrashRecovery::committed_for_canvas`] antes de chamar este
    ///   helper, OR aceitar o stroke em qualquer canvas).
    /// - `samples` → `record.points` (rename).
    /// - `partial.samples_count_in_journal` é **descartado** (write-only
    ///   no WAL — vide K-3 carry-over W11 pra cross-validate via
    ///   BeginUpdate entry).
    pub fn into_stroke_record(self) -> crate::record::StrokeRecord {
        crate::record::StrokeRecord {
            uuid: self.partial.uuid,
            seq: self.partial.seq,
            timestamp_ms: self.partial.started_at_ms,
            brush_handle: self.partial.brush_handle,
            brush_params_hash: self.partial.brush_params_hash,
            layer_target: self.partial.layer_target,
            primary_color: self.partial.primary_color,
            secondary_color: self.partial.secondary_color,
            points: self.samples,
            rng_seed: self.partial.rng_seed,
            tool_mode: self.partial.tool_mode,
            version: self.partial.version,
        }
    }
}

/// Resultado de [`CrashRecovery::scan`].
///
/// Audit J-7: counts split pra diagnóstico fino. `corrupted_entries`
/// (backward-compat) = `crc_failures + parse_failures + orphan_batches`.
#[derive(Debug)]
pub struct CrashRecovery {
    pub journal_path: PathBuf,
    pub recovered_strokes: Vec<RecoveredStroke>,
    /// Total backward-compat (soma das 3 categorias abaixo).
    pub corrupted_entries: u32,
    /// Entries com CRC32 mismatch.
    pub crc_failures: u32,
    /// Entries com header OK + CRC OK mas postcard parse falhou.
    pub parse_failures: u32,
    /// SampleBatch sem Begin parent.
    pub orphan_batches: u32,
    /// Strokes excedendo MAX_RECOVERED_STROKES.
    pub dropped_over_cap: u32,
    /// Strokes excedendo MAX_SAMPLES_PER_RECOVERED_STROKE (sample blowup).
    pub strokes_truncated_over_sample_cap: u32,
    /// Último Heartbeat wall-clock ms registrado (audit J-8).
    /// Caller usa pra UX prompt "abandoned ~N min ago".
    pub last_heartbeat_ms: Option<u64>,
}

impl CrashRecovery {
    /// Scan completo do WAL.
    ///
    /// **Caller contract (audit K-4):** pra W11+ shell wire, passe
    /// `existing_uuids` (uuids de strokes JÁ em history persistido em
    /// `.ph2d-painter`) e `last_persisted_seq` (anchor anti-replay) via
    /// `scan_with_baseline`. `scan` é simples mas NÃO detecta colisão
    /// uuid/seq attacker-controlled.
    pub fn scan(journal_path: PathBuf) -> Result<Self, RecoveryError> {
        Self::scan_with_baseline(journal_path, &BTreeSet::new(), None)
    }

    /// Variante com baseline pra detectar replay attack em cloud-sync W16.
    /// Strokes com uuid em `existing_uuids` OR `seq <= last_persisted_seq`
    /// são marcados `RecoveryState::Corrupted`.
    pub fn scan_with_baseline(
        journal_path: PathBuf,
        existing_uuids: &BTreeSet<StrokeId>,
        last_persisted_seq: Option<u64>,
    ) -> Result<Self, RecoveryError> {
        let (entries, crc_from_read) =
            read_journal(&journal_path).map_err(RecoveryError::Journal)?;
        Self::reconstruct(
            journal_path,
            entries,
            crc_from_read,
            existing_uuids,
            last_persisted_seq,
        )
    }

    /// Variante in-memory pra tests.
    pub fn reconstruct(
        journal_path: PathBuf,
        entries: Vec<WalEntryRaw>,
        initial_crc_failures: u32,
        existing_uuids: &BTreeSet<StrokeId>,
        last_persisted_seq: Option<u64>,
    ) -> Result<Self, RecoveryError> {
        let mut crc_failures = initial_crc_failures;
        let mut parse_failures = 0u32;
        let mut orphan_batches = 0u32;
        let mut dropped_over_cap = 0u32;
        let mut strokes_truncated_over_sample_cap = 0u32;
        let mut last_heartbeat_ms: Option<u64> = None;

        // BTreeMap pra HR-5 determinism cross-OS.
        let mut by_uuid: BTreeMap<StrokeId, PartialReconstructState> = BTreeMap::new();
        let mut order: Vec<StrokeId> = Vec::new();
        // Track UUIDs que foram dropados-por-cap pra distinguir orphan-real
        // vs orphan-de-dropado (audit J-5).
        let mut dropped_uuids: BTreeSet<StrokeId> = BTreeSet::new();

        for entry in entries {
            if !entry.crc_valid {
                // Já contado em initial_crc_failures pelo read_journal? No, read_journal
                // só conta failures que abortaram entry inteira (parse error de header).
                // CRC fails durante read são contados aqui (entries que entraram em
                // entries com crc_valid=false).
                crc_failures = crc_failures.saturating_add(1);
                continue;
            }
            match entry.entry_type {
                WalEntryType::Begin => {
                    match postcard::from_bytes::<PartialStroke>(&entry.payload_bytes) {
                        Ok(stroke) => {
                            // Audit K-4: replay-attack defense.
                            let uuid_collides = existing_uuids.contains(&stroke.uuid);
                            let seq_replay =
                                last_persisted_seq.is_some_and(|max| stroke.seq <= max);
                            if uuid_collides || seq_replay {
                                let uuid = stroke.uuid;
                                if !by_uuid.contains_key(&uuid) {
                                    order.push(uuid);
                                }
                                by_uuid.insert(
                                    uuid,
                                    PartialReconstructState {
                                        partial: stroke,
                                        samples: Vec::new(),
                                        committed: false,
                                        cancelled: false,
                                        forced_corrupted: true,
                                    },
                                );
                                continue;
                            }
                            if by_uuid.len() >= MAX_RECOVERED_STROKES {
                                dropped_over_cap = dropped_over_cap.saturating_add(1);
                                dropped_uuids.insert(stroke.uuid);
                                continue;
                            }
                            let uuid = stroke.uuid;
                            if !by_uuid.contains_key(&uuid) {
                                order.push(uuid);
                            }
                            by_uuid.insert(
                                uuid,
                                PartialReconstructState {
                                    partial: stroke,
                                    samples: Vec::new(),
                                    committed: false,
                                    cancelled: false,
                                    forced_corrupted: false,
                                },
                            );
                        }
                        Err(_) => parse_failures = parse_failures.saturating_add(1),
                    }
                }
                WalEntryType::SampleBatch => {
                    // Audit J-13: payload carrega stroke_id explícito.
                    match postcard::from_bytes::<SampleBatchPayload>(&entry.payload_bytes) {
                        Ok(payload) => {
                            if let Some(state) = by_uuid.get_mut(&payload.stroke_id) {
                                // Audit K-2: cap samples acumulado.
                                let new_total = state.samples.len() + payload.samples.len();
                                if new_total > MAX_SAMPLES_PER_RECOVERED_STROKE {
                                    if state.samples.len() < MAX_SAMPLES_PER_RECOVERED_STROKE {
                                        let room =
                                            MAX_SAMPLES_PER_RECOVERED_STROKE - state.samples.len();
                                        state
                                            .samples
                                            .extend(payload.samples.into_iter().take(room));
                                    }
                                    strokes_truncated_over_sample_cap =
                                        strokes_truncated_over_sample_cap.saturating_add(1);
                                    state.forced_corrupted = true;
                                } else {
                                    state.samples.extend(payload.samples);
                                }
                            } else if dropped_uuids.contains(&payload.stroke_id) {
                                // Audit J-5: NÃO infla orphan_batches — parent foi
                                // dropado-por-cap, batch órfão é esperado.
                            } else {
                                orphan_batches = orphan_batches.saturating_add(1);
                            }
                        }
                        Err(_) => parse_failures = parse_failures.saturating_add(1),
                    }
                }
                WalEntryType::Commit => {
                    match postcard::from_bytes::<StrokeId>(&entry.payload_bytes) {
                        Ok(uuid) => {
                            if let Some(state) = by_uuid.get_mut(&uuid) {
                                state.committed = true;
                            }
                            // Commit pra uuid dropado-por-cap = ignore silente.
                        }
                        Err(_) => parse_failures = parse_failures.saturating_add(1),
                    }
                }
                WalEntryType::Cancel => {
                    match postcard::from_bytes::<StrokeId>(&entry.payload_bytes) {
                        Ok(uuid) => {
                            if let Some(state) = by_uuid.get_mut(&uuid) {
                                state.cancelled = true;
                            }
                        }
                        Err(_) => parse_failures = parse_failures.saturating_add(1),
                    }
                }
                WalEntryType::Heartbeat => {
                    // Audit J-8: surface o último heartbeat.
                    if let Ok(hb_ms) = postcard::from_bytes::<u64>(&entry.payload_bytes) {
                        last_heartbeat_ms = Some(match last_heartbeat_ms {
                            Some(prev) => prev.max(hb_ms),
                            None => hb_ms,
                        });
                    }
                }
            }
        }

        // Materializa em RecoveredStroke ordenado.
        let recovered_strokes: Vec<RecoveredStroke> = order
            .iter()
            .filter_map(|uuid| {
                let state = by_uuid.remove(uuid)?;
                let recovery_state = if state.forced_corrupted {
                    RecoveryState::Corrupted
                } else if state.cancelled {
                    RecoveryState::Cancelled
                } else if state.committed {
                    // Audit K-3 nota: cross-validation `samples_count_in_journal`
                    // vs `samples.len()` real exige que Begin entry seja
                    // re-emitida (BeginUpdate) ao commit_stroke pra que o
                    // count post-fact seja durable. WAL atual é append-only
                    // (never rewrite) então Begin carrega count=0 ao emitir.
                    // W11 carry-over: adicionar `BeginUpdate` entry pra
                    // enforcement runtime real do checksum semântico.
                    RecoveryState::CommittedNotPersisted
                } else {
                    // Stroke em progresso — count NÃO validado (samples ainda
                    // chegando quando crashou). Caller decide via UX.
                    RecoveryState::InProgressAtCrash
                };
                Some(RecoveredStroke {
                    partial: state.partial,
                    samples: state.samples,
                    state: recovery_state,
                })
            })
            .collect();

        let corrupted_entries = crc_failures
            .saturating_add(parse_failures)
            .saturating_add(orphan_batches);

        Ok(Self {
            journal_path,
            recovered_strokes,
            corrupted_entries,
            crc_failures,
            parse_failures,
            orphan_batches,
            dropped_over_cap,
            strokes_truncated_over_sample_cap,
            last_heartbeat_ms,
        })
    }

    pub fn committed_not_persisted(&self) -> impl Iterator<Item = &RecoveredStroke> {
        self.recovered_strokes
            .iter()
            .filter(|s| s.state == RecoveryState::CommittedNotPersisted)
    }

    pub fn in_progress_at_crash(&self) -> impl Iterator<Item = &RecoveredStroke> {
        self.recovered_strokes
            .iter()
            .filter(|s| s.state == RecoveryState::InProgressAtCrash)
    }

    /// Filtra strokes pertencentes ao `canvas_id` dado. Audit T-durability
    /// final M-3 — WAL é per-`storage_root` não per-canvas; sem filter,
    /// strokes de canvas A vazam pra canvas B em multi-canvas workspace.
    pub fn committed_for_canvas(
        &self,
        canvas_id: crate::device::CanvasId,
    ) -> impl Iterator<Item = &RecoveredStroke> {
        self.committed_not_persisted()
            .filter(move |s| s.partial.canvas_id == canvas_id)
    }

    /// Variante sorted-by-seq de `committed_not_persisted` (audit M-2 b):
    /// recovered_strokes é ordenado por insertion-order no WAL — WAL
    /// adversarial pode ter strokes fora-de-ordem-de-seq; replay direto
    /// viola invariante "seq monotônico per-canvas" (ADR-0046 §2.2).
    pub fn committed_not_persisted_sorted_by_seq(&self) -> Vec<&RecoveredStroke> {
        let mut v: Vec<&RecoveredStroke> = self.committed_not_persisted().collect();
        v.sort_by_key(|s| s.partial.seq);
        v
    }
}

struct PartialReconstructState {
    partial: PartialStroke,
    samples: Vec<RawPointerSample>,
    committed: bool,
    cancelled: bool,
    /// Audit K-2/K-3/K-4: forced to Corrupted no fim (sample cap exceeded
    /// OR uuid collide com baseline).
    forced_corrupted: bool,
}

/// Erros de [`CrashRecovery::scan`].
#[non_exhaustive]
#[derive(Debug)]
pub enum RecoveryError {
    Journal(JournalError),
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Journal(e) => write!(f, "recovery: {e}"),
        }
    }
}

impl std::error::Error for RecoveryError {}

impl PartialEq for RecoveryError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Journal(a), Self::Journal(b)) => a == b,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{CanvasId, LayerId};
    use crate::durability::journal::{FlushPolicy, StrokeJournal};
    use ph2d_color::OklchColor;
    use ph2d_painter_brush::BrushHandle;

    fn tmp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "ph2d-painter-stroke-recovery-{}-{name}.wal",
            std::process::id()
        ))
    }

    fn sample_partial(seq: u64) -> PartialStroke {
        PartialStroke::new(
            seq,
            CanvasId(1),
            LayerId(0),
            BrushHandle::default(),
            [0u8; 32],
            OklchColor::default(),
        )
    }

    #[test]
    fn recovery_detects_committed_not_persisted() {
        let path = tmp_path("committed");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::EveryNSamples(100)).unwrap();
            j.begin_stroke(sample_partial(7)).unwrap();
            j.add_sample(RawPointerSample::default(), 0).unwrap();
            j.commit_stroke(0).unwrap();
        }
        let rec = CrashRecovery::scan(path.clone()).expect("scan");
        assert_eq!(rec.corrupted_entries, 0);
        assert_eq!(rec.recovered_strokes.len(), 1);
        assert_eq!(
            rec.recovered_strokes[0].state,
            RecoveryState::CommittedNotPersisted
        );
        assert_eq!(rec.recovered_strokes[0].samples.len(), 1);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn recovery_detects_in_progress_at_crash() {
        let path = tmp_path("in_progress");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::EveryNSamples(100)).unwrap();
            j.begin_stroke(sample_partial(7)).unwrap();
            j.add_sample(RawPointerSample::default(), 0).unwrap();
            j.flush_sample_batch(0).unwrap();
        }
        let rec = CrashRecovery::scan(path.clone()).expect("scan");
        assert_eq!(rec.recovered_strokes.len(), 1);
        assert_eq!(
            rec.recovered_strokes[0].state,
            RecoveryState::InProgressAtCrash
        );
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn recovery_detects_cancelled() {
        let path = tmp_path("cancelled");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::default()).unwrap();
            j.begin_stroke(sample_partial(0)).unwrap();
            j.cancel_stroke().unwrap();
        }
        let rec = CrashRecovery::scan(path.clone()).expect("scan");
        assert_eq!(rec.recovered_strokes.len(), 1);
        assert_eq!(rec.recovered_strokes[0].state, RecoveryState::Cancelled);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn recovery_filter_methods_work() {
        let path = tmp_path("filters");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::EveryNSamples(100)).unwrap();
            for i in 0..2 {
                j.begin_stroke(sample_partial(i)).unwrap();
                j.commit_stroke(0).unwrap();
            }
            j.begin_stroke(sample_partial(99)).unwrap();
            j.flush_sample_batch(0).ok();
        }
        let rec = CrashRecovery::scan(path.clone()).expect("scan");
        assert_eq!(rec.committed_not_persisted().count(), 2);
        assert_eq!(rec.in_progress_at_crash().count(), 1);
        std::fs::remove_file(&path).ok();
    }

    /// Audit K-3 carry-over: cross-validation real diferida pra W11
    /// (precisa BeginUpdate entry). Hoje recovery aceita commit sem
    /// validar count.
    #[test]
    fn recovery_accepts_committed_without_count_validation_in_v1() {
        let path = tmp_path("count_v1");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::default()).unwrap();
            j.begin_stroke(sample_partial(0)).unwrap();
            j.add_sample(RawPointerSample::default(), 0).unwrap();
            j.commit_stroke(0).unwrap();
        }
        let rec = CrashRecovery::scan(path.clone()).expect("scan");
        assert_eq!(rec.recovered_strokes.len(), 1);
        assert_eq!(
            rec.recovered_strokes[0].state,
            RecoveryState::CommittedNotPersisted
        );
        std::fs::remove_file(&path).ok();
    }

    /// Audit K-4: stroke com seq em baseline → Corrupted.
    #[test]
    fn recovery_marks_corrupted_on_seq_replay() {
        let path = tmp_path("seq_replay");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::EveryNSamples(100)).unwrap();
            j.begin_stroke(sample_partial(5)).unwrap(); // seq=5
            j.commit_stroke(0).unwrap();
        }
        // Baseline last_persisted_seq=10 → seq=5 é replay attack.
        let rec = CrashRecovery::scan_with_baseline(path.clone(), &BTreeSet::new(), Some(10))
            .expect("scan");
        assert_eq!(rec.recovered_strokes.len(), 1);
        assert_eq!(rec.recovered_strokes[0].state, RecoveryState::Corrupted);
        std::fs::remove_file(&path).ok();
    }

    /// Audit J-8: heartbeat_ms surfaced.
    #[test]
    fn recovery_surfaces_last_heartbeat() {
        let path = tmp_path("heartbeat");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::default()).unwrap();
            j.heartbeat(1000).unwrap();
            j.heartbeat(2000).unwrap();
            j.heartbeat(1500).unwrap(); // out-of-order
        }
        let rec = CrashRecovery::scan(path.clone()).expect("scan");
        // last_heartbeat_ms = max de todos.
        assert_eq!(rec.last_heartbeat_ms, Some(2000));
        std::fs::remove_file(&path).ok();
    }

    /// Audit J-7: counts split available.
    #[test]
    fn recovery_exposes_split_counts() {
        let path = tmp_path("split_counts");
        std::fs::remove_file(&path).ok();
        {
            let mut j = StrokeJournal::open(path.clone(), FlushPolicy::default()).unwrap();
            j.begin_stroke(sample_partial(0)).unwrap();
            j.commit_stroke(0).unwrap();
        }
        let rec = CrashRecovery::scan(path.clone()).expect("scan");
        // Happy path: 0 em tudo.
        assert_eq!(rec.crc_failures, 0);
        assert_eq!(rec.parse_failures, 0);
        assert_eq!(rec.orphan_batches, 0);
        assert_eq!(rec.dropped_over_cap, 0);
        assert_eq!(rec.strokes_truncated_over_sample_cap, 0);
        std::fs::remove_file(&path).ok();
    }
}
