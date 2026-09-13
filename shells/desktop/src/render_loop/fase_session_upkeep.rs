//! **Fase do quadro: A MANUTENÇÃO DE SESSÃO** — o Shape Builder, os pares do tween, o Apply/Clear e o
//! ajuste vivo do Colorize e os helpers do Gap Closure (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **A última antes do empréstimo do `gfx`**, e é por isso que o Apply do Colorize diferido mora
//! aqui: o dreno de painel roda com o `gfx` preso, e aqui a `App` está livre.

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_session_upkeep(&mut self) {
        self.build_session_upkeep();
        // Tween v2: a sessão de correção de pares SEGUE o artista a um novo intervalo (no-op
        // se o intervalo é o mesmo, ou se a sessão está fechada).
        self.flip_tween_pairs_upkeep();
        // ADR-0114 C2: o Apply/Clear do Colorize marcado no frame anterior (o drain de painel
        // roda com `self.gfx` preso; aqui, antes de bindar `gfx`, `self` está livre).
        if std::mem::take(&mut self.flip_state.pending_colorize_apply) {
            self.flip_colorize_apply();
            // ⚠️ **Sem isto o Apply NÃO É DESFAZÍVEL.** O `post_frame_undo` só compara o
            // estado quando o frame teve INPUT (`had_input`) — é o proxy dele para "o
            // usuário fez algo que merece um passo". O clique aconteceu no frame ANTERIOR
            // (a deferral acima), então este frame normalmente não tem input nenhum: o diff
            // era pulado e a mutação nunca virava passo. O trabalho é do usuário e cai
            // NESTE frame, então o frame carrega trabalho — e o proxy volta a ser verdade.
            self.any_input_this_frame = true;
        }
        if std::mem::take(&mut self.flip_state.pending_colorize_clear) {
            self.flip_colorize_clear();
        }
        // ADR-0114 C2 (6º smoke): Trap/Bleed em tempo real depois do Apply. Se um dos dois
        // mudou desde a última rodada, re-roda o corte in-place (o "ajustar a última operação").
        self.flip_colorize_live_adjust();
        // Doc 06 §8: os helpers ao vivo do Gap Closure — mantém o worker sincronizado
        // com o desenho na tela e o alcance atual (só em modo Fill; no-op fora dele).
        self.flip_gap_helpers_tick();
    }
}
