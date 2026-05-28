//! Auto-save policy + state machine. ADR-0052 §2.4.
//!
//! `.ph2d-painter` (canon) é salvo periodicamente via [`atomic_write`].
//! Worker thread; UI nunca bloqueada. Storage-full handling explícito.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::device::CanvasId;

use super::atomic_write::AtomicWriteError;

/// Default `EveryNStrokes` count. ADR-0052 §2.4 — mobile-friendly.
pub const AUTOSAVE_DEFAULT_N_STROKES: u32 = 25;

/// Default `EveryMinutes` interval. ADR-0052 §2.4 — 5 minutos wall-clock.
pub const AUTOSAVE_DEFAULT_MINUTES: u32 = 5;

/// Política de auto-save. ADR-0052 §2.4.
///
/// `#[non_exhaustive]` permite policies futuras (`OnIdle`, `Adaptive`, etc.).
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AutoSavePolicy {
    /// Salva após N strokes commitados. Mobile-friendly (memory pressure
    /// dispara antes de wall-clock chegar).
    EveryNStrokes(u32),
    /// Salva a cada M minutos wall-clock. Latency-bounded.
    EveryMinutes(u32),
    /// Hybrid (recomendado): qualquer um dos dois primeiro.
    Hybrid { n: u32, minutes: u32 },
    /// Manual only — power user.
    Manual,
    // === 4 slots de headroom ===
}

impl Default for AutoSavePolicy {
    fn default() -> Self {
        Self::Hybrid {
            n: AUTOSAVE_DEFAULT_N_STROKES,
            minutes: AUTOSAVE_DEFAULT_MINUTES,
        }
    }
}

/// Estado do auto-save scheduler. ADR-0052 §2.4.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AutoSaveState {
    /// Idle / nothing dirty.
    #[default]
    Idle,
    /// Dirty mas dentro da janela de policy (não disparou).
    Pending,
    /// I/O em worker thread; estado intermediário.
    SavingInBackground,
    /// Save completo; timestamp em wall-clock ms.
    SavedAt(u64),
    /// Save falhou; diagnostic + UX toast.
    Failed(AutoSaveError),
    // === 1 slot de headroom (e.g., RetryScheduled) ===
}

/// Coordinator do auto-save. NÃO contém worker thread aqui — é state
/// machine pura; caller (shell W11+) wire o worker.
///
/// **Audit T-durability K-6 + O-3 + N-6 — thread-safety + Clone removido:**
/// NÃO thread-safe + NÃO `Clone`. Caller DEVE garantir single-writer per
/// canvas (e.g., `Mutex<AutoSave>` por `CanvasId`, OR canal SPSC entre
/// MCP thread e UI thread). Clone foi removido pra evitar split-brain
/// onde 2 threads escrevem mesmo `tmp_path` determinístico em
/// `atomic_write` → canon corruption (O-3).
///
/// **Spec correction (audit N-6 / M-15):** ADR-0052 §2.4 prometia
/// "AutoSave em worker thread; UI nunca bloqueada" — implementação é
/// state machine pura; caller (W11 shell) é responsável por dispatch
/// em worker thread. ADR amendment pendente.
#[derive(Debug)]
pub struct AutoSave {
    pub canvas_id: CanvasId,
    pub path: PathBuf,
    pub policy: AutoSavePolicy,
    /// Wall-clock ms do último save bem-sucedido.
    pub last_save_ms: u64,
    /// Strokes commitados desde o último save.
    pub strokes_since_last: u32,
    pub state: AutoSaveState,
}

impl AutoSave {
    pub fn new(canvas_id: CanvasId, path: PathBuf, policy: AutoSavePolicy) -> Self {
        Self::with_clock_start(canvas_id, path, policy, 0)
    }

    /// Construtor que aceita `start_ms` inicial (audit T-durability final M-10).
    /// `EveryMinutes(m)` calcula `now - last_save_ms`; com `last_save_ms = 0`
    /// caller que passa wall-clock real no primeiro commit dispara save
    /// IMEDIATAMENTE (foot-gun timing). Use este construtor passando o
    /// wall-clock no boot pra evitar.
    pub fn with_clock_start(
        canvas_id: CanvasId,
        path: PathBuf,
        policy: AutoSavePolicy,
        start_ms: u64,
    ) -> Self {
        Self {
            canvas_id,
            path,
            policy,
            last_save_ms: start_ms,
            strokes_since_last: 0,
            state: AutoSaveState::Idle,
        }
    }

    /// Caller chama após cada `commit_stroke` em history. Returns `true` se
    /// AutoSave decide disparar agora.
    ///
    /// **Audit K-6:** `#[must_use]` força caller checar bool (esquecer =
    /// save nunca dispara em policy `EveryNStrokes`).
    #[must_use]
    pub fn on_stroke_committed(&mut self, now_ms: u64) -> bool {
        self.strokes_since_last = self.strokes_since_last.saturating_add(1);
        // Audit T-durability final M-9: Manual policy NÃO deve marcar
        // Pending — caller observando state pra wire worker save dispara
        // acidentalmente em Manual. Mantém Idle em Manual; mudança só
        // via `request_manual_save()`.
        if !matches!(self.policy, AutoSavePolicy::Manual) {
            self.state = AutoSaveState::Pending;
        }
        self.should_save(now_ms)
    }

    /// `true` se policy diz pra salvar agora.
    pub fn should_save(&self, now_ms: u64) -> bool {
        match self.policy {
            AutoSavePolicy::Manual => false,
            AutoSavePolicy::EveryNStrokes(n) => self.strokes_since_last >= n,
            AutoSavePolicy::EveryMinutes(m) => {
                now_ms.saturating_sub(self.last_save_ms) >= u64::from(m) * 60 * 1000
            }
            AutoSavePolicy::Hybrid { n, minutes } => {
                self.strokes_since_last >= n
                    || now_ms.saturating_sub(self.last_save_ms) >= u64::from(minutes) * 60 * 1000
            }
        }
    }

    /// Caller marca início de save (worker thread spawned).
    pub fn mark_saving(&mut self) {
        self.state = AutoSaveState::SavingInBackground;
    }

    /// Caller chama após save bem-sucedido (worker completou).
    pub fn mark_saved(&mut self, now_ms: u64) {
        self.last_save_ms = now_ms;
        self.strokes_since_last = 0;
        self.state = AutoSaveState::SavedAt(now_ms);
    }

    /// Caller chama após save falho.
    pub fn mark_failed(&mut self, err: AutoSaveError) {
        self.state = AutoSaveState::Failed(err);
    }

    /// Force-save manual (Cmd+S). Bypass policy.
    pub fn request_manual_save(&mut self) {
        self.state = AutoSaveState::Pending;
    }
}

/// Erros de auto-save. ADR-0052 §2.8.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AutoSaveError {
    /// I/O genérico (permissions, FS readonly, etc.).
    Io(std::io::ErrorKind),
    /// Storage check falhou; libere espaço.
    InsufficientStorage { needed_bytes: u64 },
    /// `atomic_write` falhou em alguma fase específica.
    AtomicWriteFailed(AtomicWriteError),
    /// fsync falhou — disco pode estar corrupto / removido.
    FsyncFailed(std::io::ErrorKind),
    // === 4 slots de headroom ===
}

impl From<AtomicWriteError> for AutoSaveError {
    fn from(e: AtomicWriteError) -> Self {
        match e {
            AtomicWriteError::InsufficientStorage { needed_bytes } => {
                Self::InsufficientStorage { needed_bytes }
            }
            AtomicWriteError::FsyncFailed(k) => Self::FsyncFailed(k),
            other => Self::AtomicWriteFailed(other),
        }
    }
}

impl std::fmt::Display for AutoSaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(k) => write!(f, "auto-save I/O: {k:?}"),
            Self::InsufficientStorage { needed_bytes } => write!(
                f,
                "auto-save: insufficient storage (need ≥ {needed_bytes} bytes)"
            ),
            Self::AtomicWriteFailed(e) => write!(f, "auto-save: {e}"),
            Self::FsyncFailed(k) => write!(f, "auto-save: fsync failed: {k:?}"),
        }
    }
}

impl std::error::Error for AutoSaveError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_default_is_hybrid_25_5() {
        assert_eq!(
            AutoSavePolicy::default(),
            AutoSavePolicy::Hybrid { n: 25, minutes: 5 }
        );
    }

    #[test]
    fn should_save_triggers_after_n_strokes() {
        let mut a = AutoSave::new(
            CanvasId(1),
            PathBuf::from("/tmp/x.ph2d-painter"),
            AutoSavePolicy::EveryNStrokes(3),
        );
        assert!(!a.on_stroke_committed(0));
        assert!(!a.on_stroke_committed(0));
        assert!(a.on_stroke_committed(0)); // 3rd
    }

    #[test]
    fn should_save_triggers_after_minutes() {
        let mut a = AutoSave::new(
            CanvasId(1),
            PathBuf::from("/tmp/x.ph2d-painter"),
            AutoSavePolicy::EveryMinutes(5),
        );
        assert!(!a.on_stroke_committed(0));
        // 4 minutos: ainda não.
        assert!(!a.should_save(4 * 60 * 1000));
        // 5 minutos: dispara.
        assert!(a.should_save(5 * 60 * 1000));
    }

    #[test]
    fn manual_policy_never_auto_triggers() {
        let mut a = AutoSave::new(
            CanvasId(1),
            PathBuf::from("/tmp/x.ph2d-painter"),
            AutoSavePolicy::Manual,
        );
        for _ in 0..1000 {
            assert!(!a.on_stroke_committed(0));
        }
        // Mas request_manual_save muda state.
        a.request_manual_save();
        assert_eq!(a.state, AutoSaveState::Pending);
    }

    #[test]
    fn mark_saved_resets_strokes_counter() {
        let mut a = AutoSave::new(
            CanvasId(1),
            PathBuf::from("/tmp/x.ph2d-painter"),
            AutoSavePolicy::EveryNStrokes(2),
        );
        let _ = a.on_stroke_committed(0);
        let _ = a.on_stroke_committed(0);
        assert_eq!(a.strokes_since_last, 2);
        a.mark_saving();
        a.mark_saved(123);
        assert_eq!(a.strokes_since_last, 0);
        assert_eq!(a.last_save_ms, 123);
        assert_eq!(a.state, AutoSaveState::SavedAt(123));
    }

    #[test]
    fn autosave_error_from_atomic_maps_correctly() {
        let e = AtomicWriteError::InsufficientStorage { needed_bytes: 1024 };
        let mapped: AutoSaveError = e.into();
        assert!(matches!(
            mapped,
            AutoSaveError::InsufficientStorage { needed_bytes: 1024 }
        ));
    }
}
