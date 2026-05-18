//! `EventOutcome` — tripartite return for panel `apply_event` thunks.
//!
//! ADR-0029 Phase B.1: moved from `ph2d-editor::panel_registry` to
//! editor-core so panel crates (`ph2d-panel-*`) can consume it
//! without depending on `ph2d-editor`. Wave 8 Phase 4 originally
//! defined it (audit B2 + A4 closure).

/// Outcome of a panel's `apply_event` call. Three variants make the
/// dispatcher's job explicit:
///
/// - **Consumed**: the panel handled the event exclusively. Dispatcher
///   stops iterating; no other panel needs to see it.
/// - **Ignored**: the panel did nothing. Dispatcher continues to next.
/// - **Observed**: the panel did a side-effect (state mutation, buffer
///   commit, etc.) but doesn't claim exclusivity. Dispatcher continues
///   iterating; the orchestrator OR's the `observed` flag into the
///   final return so the caller knows *something* happened.
///
/// `Observed` exists because of the audit B2 anti-pattern: hierarchy's
/// `Blur(HIER_RENAME_INPUT)` stages a `HierRenameCommit` AND used to
/// return false (so chrome fallthrough still ran). Tripartite makes
/// the intent loud + impossible to introduce silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOutcome {
    Consumed,
    Ignored,
    Observed,
}

impl EventOutcome {
    /// Migration helper: convert a legacy `-> bool` return. `true` →
    /// Consumed, `false` → Ignored. Panels that have side-effects
    /// without exclusivity (rare — currently just hierarchy Blur)
    /// return `Observed` directly without going through this helper.
    pub fn from_bool(consumed: bool) -> Self {
        if consumed {
            EventOutcome::Consumed
        } else {
            EventOutcome::Ignored
        }
    }
}
