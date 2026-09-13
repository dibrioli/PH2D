//! **Fase do quadro: O PERFILADOR** (`PH2D_FLUID_PROFILE`) — o último passo do quadro: conta os quadros,
//! arma o relógio da janela no PRIMEIRO quadro e, a cada 120, chama o relatório da partição
//! (`fase_frame_profile_report`) (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Corre DEPOIS do `run_present_phase`: o `frame_ms_ewma` e o `frame_cpu_ms_ewma` que o relatório lê
//! são os deste quadro.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_frame_profile(&mut self) {
        // Frame-phase profiler (PH2D_FLUID_PROFILE): the `[fluid]` line proves the
        // fluid drive is ~2 ms, so a 6-fps stall lives elsewhere. This splits the
        // parent: total vs CPU-encode (raw) → the gap is the present/GPU acquire
        // stall; plus the painter dispatch (CPU preview produce + upload).
        if frame_prof_on() {
            let n = FRAME_PROF_N.with(|c| {
                let n = c.get().wrapping_add(1);
                c.set(n);
                n
            });
            // ⚠️ O relógio da janela ARMA no primeiro frame, não no primeiro
            // relatório: sem isto a primeira janela sairia com `span = 0` e
            // imprimiria uma partição de zeros, que se lê como *"o worker não
            // fez nada"* — a mentira oposta à que ele existe para evitar.
            FRAME_PROF_SINCE.with(|c| {
                let mut b = c.borrow_mut();
                if b.is_none() {
                    *b = Some(std::time::Instant::now());
                }
            });
            if n.is_multiple_of(120) {
                self.fase_frame_profile_report();
            }
        }
    }
}
