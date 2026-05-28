//! `SuspendHandler` — iOS/Android lifecycle integration. ADR-0052 §2.5.
//!
//! iOS background task API dá **~5s** para gracefully shutdown antes do
//! force-kill OS. Android idem (com variações: Lite ~2-3s, Go ~1s). Esta
//! camada drena WAL + canon + cache priorizando WAL > canon > cache na
//! janela de budget que o OS impôs.
//!
//! ## Drain protocol (ADR-0052 §2.5)
//!
//! Por padrão 5000ms (iOS budget). Caller passa `os_deadline_ms` real
//! conforme device (Android Lite ~3000, Go ~1500, low-power-mode reduções).
//! **Phases proporcionais ao budget**, não hardcoded:
//!
//! - Phase 1 (WAL flush): 40% do budget. Stroke-in-progress preservado.
//! - Phase 2 (canon write): 70% acumulado do budget.
//! - Phase 3 (cache flush + final fsync): resto.
//!
//! Caller usa [`SuspendHandler::within_wal_phase`] / [`Self::within_canon_phase`]
//! pra abortar phases que já passaram.
//!
//! ## Audit T-durability fixes
//!
//! - J-1 CRITICAL: `os_deadline_ms` persistido em struct; phases calculadas
//!   proporcionalmente em vez de hardcoded `DRAIN_DEADLINE_DEFAULT_MS`.
//! - J-14 / K-8: `on_imminent` idempotente (preserva deadline existente se
//!   já em SuspendImminent — re-trigger não estende budget).
//! - K-7: `last_tick_ms` tracking pra defender contra clock regression.
//! - K-9: `confirm_active` state-guarded (só após Resuming).

/// Deadline default em ms — iOS background task budget. ADR-0052 §2.5.
pub const DRAIN_DEADLINE_DEFAULT_MS: u32 = 5_000;

/// Fração do budget alocada para phase 1 (WAL flush). 40% — preserva
/// stroke-in-progress (mais crítico).
pub const DRAIN_FLUSH_WAL_PHASE_FRAC: u32 = 40;

/// Fração ACUMULADA do budget alocada para fim da phase 2 (canon write).
/// 70% — canon dirty escrito antes do force-kill.
pub const DRAIN_WRITE_CANON_PHASE_FRAC: u32 = 70;

/// Backward-compat: budget em ms da phase 1 calculado com default deadline.
/// Use [`SuspendHandler::within_wal_phase`] pra checagem dynamic.
pub const DRAIN_FLUSH_WAL_PHASE_MS: u32 =
    DRAIN_DEADLINE_DEFAULT_MS * DRAIN_FLUSH_WAL_PHASE_FRAC / 100;

/// Backward-compat: budget em ms da phase 2 calculado com default deadline.
pub const DRAIN_WRITE_CANON_PHASE_MS: u32 =
    DRAIN_DEADLINE_DEFAULT_MS * DRAIN_WRITE_CANON_PHASE_FRAC / 100;

/// State machine do suspend handler. ADR-0052 §2.5.
///
/// `#[non_exhaustive]` permite states futuros (`Foreground`, `Throttled`).
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SuspendState {
    /// App em foreground; nothing pending.
    #[default]
    Active,
    /// OS sinalizou suspend; `remaining_ms` budget restante.
    SuspendImminent { remaining_ms: u32 },
    /// App voltou de suspend; aguarda `confirm_active` pra retomar.
    Resuming,
    // === 1 slot de headroom (e.g., Throttled — bg refresh limit) ===
}

/// Coordinator do suspend. State machine pura — caller (shell) é
/// responsável por executar I/O efetivo nas phases.
#[derive(Clone, Debug)]
pub struct SuspendHandler {
    pub state: SuspendState,
    /// Wall-clock deadline absoluta (caller traduz pra wait-budget per phase).
    pub drain_deadline_ms: u64,
    /// Budget que o OS deu na última `on_imminent`. Persistido pra phase
    /// calculations corretas (audit J-1).
    pub os_deadline_ms: u32,
    /// Wall-clock do último tick. Defensiva contra clock regression (K-7).
    pub last_tick_ms: u64,
}

impl Default for SuspendHandler {
    fn default() -> Self {
        Self {
            state: SuspendState::Active,
            drain_deadline_ms: 0,
            os_deadline_ms: DRAIN_DEADLINE_DEFAULT_MS,
            last_tick_ms: 0,
        }
    }
}

impl SuspendHandler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Caller invoca quando OS sinaliza suspend imminent. `now_ms` é
    /// wall-clock; `os_deadline_ms` é budget do OS (5000 iOS / 3000
    /// Android Lite / 1500 Android Go etc.).
    ///
    /// **Audit J-14 / K-8 idempotência:** se já em `SuspendImminent` e o
    /// deadline anterior ainda é futuro, NÃO sobrescreve — preserva
    /// progresso de drain. Re-trigger só efetiva mudança após `on_resume`
    /// OR se o deadline anterior já expirou.
    pub fn on_imminent(&mut self, now_ms: u64, os_deadline_ms: u32) {
        if matches!(self.state, SuspendState::SuspendImminent { .. })
            && now_ms < self.drain_deadline_ms
        {
            // Já em drain ativo; preserva.
            return;
        }
        self.os_deadline_ms = os_deadline_ms;
        self.drain_deadline_ms = now_ms.saturating_add(u64::from(os_deadline_ms));
        self.last_tick_ms = now_ms;
        self.state = SuspendState::SuspendImminent {
            remaining_ms: os_deadline_ms,
        };
    }

    /// Caller invoca conforme drain progride pra atualizar `remaining_ms`.
    /// Retorna `true` se ainda há budget pra próxima phase.
    ///
    /// **Audit K-7:** clock-regression-safe — usa `now_ms.max(last_tick_ms)`
    /// em vez de aceitar regressão silently. Se sistema NTP regrediu o
    /// clock mid-suspend, tratamos como "no time passou" (zero damage) em
    /// vez de "budget restaurado" (data loss potencial).
    pub fn tick(&mut self, now_ms: u64) -> bool {
        let safe_now = now_ms.max(self.last_tick_ms);
        self.last_tick_ms = safe_now;
        let remaining = self.drain_deadline_ms.saturating_sub(safe_now);
        if let SuspendState::SuspendImminent { remaining_ms } = &mut self.state {
            *remaining_ms = remaining.min(u64::from(u32::MAX)) as u32;
        }
        remaining > 0
    }

    /// Caller invoca quando OS sinaliza resume. Reseta state.
    pub fn on_resume(&mut self) {
        self.state = SuspendState::Resuming;
        self.drain_deadline_ms = 0;
        self.last_tick_ms = 0;
    }

    /// Caller invoca depois de `on_resume` quando estabilidade confirmada.
    ///
    /// **Audit K-9 state-guard:** silencia chamada se state != Resuming
    /// (`debug_assert!` em dev pra pegar bug, no-op em release pra
    /// preservar invariante "drain em curso não é silenciado").
    pub fn confirm_active(&mut self) {
        debug_assert!(
            matches!(self.state, SuspendState::Resuming),
            "confirm_active deve ser chamado apenas após on_resume (state atual: {:?})",
            self.state
        );
        if !matches!(self.state, SuspendState::Resuming) {
            return;
        }
        self.state = SuspendState::Active;
    }

    /// `true` se ainda dentro do budget de phase 1 (WAL flush — 40% do budget).
    /// Audit J-1: agora usa `self.os_deadline_ms` (não hardcoded default).
    pub fn within_wal_phase(&self, now_ms: u64) -> bool {
        let Some(elapsed) = self.elapsed_since_start(now_ms) else {
            return false;
        };
        let phase_budget =
            u64::from(self.os_deadline_ms) * u64::from(DRAIN_FLUSH_WAL_PHASE_FRAC) / 100;
        elapsed < phase_budget
    }

    /// `true` se ainda dentro do budget de phase 2 (canon write — 70% acumulado).
    /// Audit J-1: usa `self.os_deadline_ms`.
    pub fn within_canon_phase(&self, now_ms: u64) -> bool {
        let Some(elapsed) = self.elapsed_since_start(now_ms) else {
            return false;
        };
        let phase_budget =
            u64::from(self.os_deadline_ms) * u64::from(DRAIN_WRITE_CANON_PHASE_FRAC) / 100;
        elapsed < phase_budget
    }

    /// Helper interno: tempo decorrido desde `on_imminent`, ou `None` se
    /// não está em SuspendImminent.
    fn elapsed_since_start(&self, now_ms: u64) -> Option<u64> {
        if !matches!(self.state, SuspendState::SuspendImminent { .. }) {
            return None;
        }
        let start = self
            .drain_deadline_ms
            .saturating_sub(u64::from(self.os_deadline_ms));
        Some(now_ms.saturating_sub(start))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_active_zero_deadline_default_budget() {
        let s = SuspendHandler::default();
        assert_eq!(s.state, SuspendState::Active);
        assert_eq!(s.drain_deadline_ms, 0);
        assert_eq!(s.os_deadline_ms, DRAIN_DEADLINE_DEFAULT_MS);
    }

    #[test]
    fn on_imminent_sets_state_and_deadline() {
        let mut h = SuspendHandler::new();
        h.on_imminent(1000, 5000);
        assert!(matches!(
            h.state,
            SuspendState::SuspendImminent { remaining_ms: 5000 }
        ));
        assert_eq!(h.drain_deadline_ms, 6000);
        assert_eq!(h.os_deadline_ms, 5000);
    }

    #[test]
    fn tick_decrements_remaining_ms() {
        let mut h = SuspendHandler::new();
        h.on_imminent(0, 5000);
        assert!(h.tick(2_000));
        if let SuspendState::SuspendImminent { remaining_ms } = h.state {
            assert_eq!(remaining_ms, 3_000);
        } else {
            panic!("state mudou indevidamente");
        }
    }

    #[test]
    fn tick_returns_false_after_deadline() {
        let mut h = SuspendHandler::new();
        h.on_imminent(0, 5000);
        assert!(!h.tick(6_000), "passou deadline");
    }

    #[test]
    fn on_resume_resets_to_resuming() {
        let mut h = SuspendHandler::new();
        h.on_imminent(0, 5000);
        h.on_resume();
        assert_eq!(h.state, SuspendState::Resuming);
        assert_eq!(h.drain_deadline_ms, 0);
        h.confirm_active();
        assert_eq!(h.state, SuspendState::Active);
    }

    #[test]
    fn within_wal_phase_true_in_first_40pct_of_default_budget() {
        let mut h = SuspendHandler::new();
        h.on_imminent(0, 5000);
        // 40% de 5000 = 2000ms.
        assert!(h.within_wal_phase(0));
        assert!(h.within_wal_phase(1_500));
        assert!(!h.within_wal_phase(2_500));
    }

    #[test]
    fn within_canon_phase_true_in_first_70pct_of_default_budget() {
        let mut h = SuspendHandler::new();
        h.on_imminent(0, 5000);
        // 70% de 5000 = 3500ms.
        assert!(h.within_canon_phase(0));
        assert!(h.within_canon_phase(3_000));
        assert!(!h.within_canon_phase(4_000));
    }

    /// J-1 CRITICAL regression test: budget != 5000ms.
    #[test]
    fn within_wal_phase_scales_with_os_deadline_android_lite_3000ms() {
        let mut h = SuspendHandler::new();
        h.on_imminent(0, 3000);
        // 40% de 3000 = 1200ms.
        assert!(h.within_wal_phase(0));
        assert!(h.within_wal_phase(1_000));
        assert!(!h.within_wal_phase(1_500));
    }

    /// J-1 CRITICAL regression test: budget muito reduzido (Android Go).
    #[test]
    fn within_wal_phase_scales_with_os_deadline_android_go_1500ms() {
        let mut h = SuspendHandler::new();
        h.on_imminent(0, 1500);
        // 40% de 1500 = 600ms.
        assert!(h.within_wal_phase(0));
        assert!(h.within_wal_phase(500));
        assert!(!h.within_wal_phase(700));
    }

    /// J-14/K-8: on_imminent re-trigger durante drain ativo NÃO estende.
    #[test]
    fn on_imminent_idempotent_during_active_drain() {
        let mut h = SuspendHandler::new();
        h.on_imminent(0, 5000);
        let original_deadline = h.drain_deadline_ms;
        // Re-trigger no t=2000 com budget novo de 5000 — DEVE preservar.
        h.on_imminent(2_000, 5000);
        assert_eq!(h.drain_deadline_ms, original_deadline);
    }

    /// J-14/K-8: re-trigger após deadline expirado OK.
    #[test]
    fn on_imminent_re_trigger_after_deadline_expired_updates() {
        let mut h = SuspendHandler::new();
        h.on_imminent(0, 5000);
        // Deadline passou; re-trigger atualiza.
        h.on_imminent(10_000, 3000);
        assert_eq!(h.drain_deadline_ms, 13_000);
        assert_eq!(h.os_deadline_ms, 3000);
    }

    /// K-7: clock regression é tratada como "no time passou".
    #[test]
    fn tick_clock_regression_does_not_restore_budget() {
        let mut h = SuspendHandler::new();
        h.on_imminent(1000, 5000); // deadline = 6000
        h.tick(3000); // remaining = 3000
        h.tick(2000); // CLOCK REGRESSION — deveria manter remaining ≈ 3000, não 4000.
        if let SuspendState::SuspendImminent { remaining_ms } = h.state {
            assert!(
                remaining_ms <= 3000,
                "clock regression NÃO deve restaurar budget; got {remaining_ms}"
            );
        }
    }

    /// K-9: confirm_active fora de Resuming é no-op (no panic em release).
    #[test]
    #[cfg(not(debug_assertions))]
    fn confirm_active_outside_resuming_is_noop() {
        let mut h = SuspendHandler::new();
        h.on_imminent(0, 5000);
        h.confirm_active(); // Em SuspendImminent — deve ser no-op em release.
        assert!(matches!(h.state, SuspendState::SuspendImminent { .. }));
    }

    /// K-9: confirm_active em Resuming muda pra Active.
    #[test]
    fn confirm_active_after_resume_transitions_to_active() {
        let mut h = SuspendHandler::new();
        h.on_imminent(0, 5000);
        h.on_resume();
        h.confirm_active();
        assert_eq!(h.state, SuspendState::Active);
    }
}
