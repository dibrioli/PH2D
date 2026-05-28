//! Tear-resistant stroke commit + crash recovery (ADR-0052).
//!
//! Pintura profissional dura horas. Esta camada garante que **stroke
//! perdido é IMPOSSÍVEL** mesmo sob crash / power-off / suspend mobile /
//! tablet unplug / storage full / kill explícito.
//!
//! ## Componentes
//!
//! - [`atomic_write`] — `atomic_write()` + `AtomicWriteError`: write-temp
//!   + fsync + rename + dir-fsync. POSIX clássico (PostgreSQL/SQLite).
//! - [`journal`] — `StrokeJournal` + `PartialStroke` + `FlushPolicy`:
//!   WAL append-only com CRC32 per entry. Flush default 8 samples ou 100ms.
//! - [`recovery`] — `CrashRecovery` + `RecoveredStroke` + `RecoveryState`:
//!   boot scan do WAL + replay seletivo.
//! - [`autosave`] — `AutoSave` + `AutoSavePolicy` + `AutoSaveState` +
//!   `AutoSaveError`: salva `.ph2d-painter` periodicamente via atomic_write.
//! - [`suspend`] — `SuspendHandler` + `SuspendState`: drena WAL + canon
//!   antes do force-kill OS (iOS ~5s budget).
//!
//! ## Drain orchestration (W11 helper pendente — audit M-15)
//!
//! ADR-0052 §2.5 especifica drain protocol em phases (WAL → canon → cache)
//! com budgets proporcionais ao `os_deadline_ms`. Helpers `within_wal_phase`
//! / `within_canon_phase` em [`SuspendHandler`] expõem o budget; **helper
//! `drain_for_suspend(handler, journal, autosave, canon_path, now_ms_fn)`
//! que coordena os 3 NÃO existe** — caller (W11 shell) implementa drain
//! manualmente. Risk: shell reinventa, esquece phase 1 (WAL flush) → stroke
//! perdido no force-kill. Helper canônico é W11 carry-over.
//!
//! ## Lifecycle stroke-in-progress (ADR-0052 §2.7)
//!
//! ```text
//! [idle] ──BeginStroke──► [stroke_active]
//!                            ├─AddSample──► WAL flush per policy
//!                            ├─CommitStroke──► WAL Commit ─► History append ─► AutoSave dispatch ─► [idle]
//!                            └─CancelStroke──► WAL Cancel ─► [idle]
//!
//! [crashed mid-stroke] ──boot──► CrashRecovery.recover()
//!                            ├─CommittedNotPersisted ─► silent replay
//!                            ├─InProgressAtCrash ─► UX prompt
//!                            └─Cancelled / Corrupted ─► discard
//! ```

pub mod atomic_write;
pub mod autosave;
pub mod journal;
pub mod recovery;
pub mod suspend;

pub use atomic_write::{AtomicWriteError, STORAGE_SAFETY_MULTIPLIER, atomic_write};
pub use autosave::{AutoSave, AutoSaveError, AutoSavePolicy, AutoSaveState};
pub use journal::{
    FlushPolicy, JOURNAL_MAGIC, JOURNAL_ROTATE_BYTES, JOURNAL_ROTATE_COMMITS, JournalError,
    MAX_RESYNC_SCAN_BYTES, MAX_WAL_FILE_BYTES, MAX_WAL_PAYLOAD, PartialStroke, SampleBatchPayload,
    StrokeJournal, WalEntryRaw, WalEntryType, read_journal,
};
pub use recovery::{
    CrashRecovery, MAX_RECOVERED_STROKES, MAX_SAMPLES_PER_RECOVERED_STROKE, RecoveredStroke,
    RecoveryError, RecoveryState,
};
pub use suspend::{
    DRAIN_DEADLINE_DEFAULT_MS, DRAIN_FLUSH_WAL_PHASE_MS, DRAIN_WRITE_CANON_PHASE_MS,
    SuspendHandler, SuspendState,
};
