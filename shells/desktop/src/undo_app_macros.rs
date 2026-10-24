//! O que o `undo_app` pede às macros (W8/W14): a transação da reescrita, as edições que esperam, o
//! passo que o baseline vira. Irmão de [`super::app`] por RESPONSABILIDADE (o tecto de 600 do HR-18).

impl crate::App {
    /// A reescrita de uma etapa editada de uma macro escreveu (W8): o passo global regista-se já com a
    /// transação dela.
    pub(super) fn reescrita_aterrou(&self) -> bool {
        self.gfx
            .as_ref()
            .is_some_and(|g| g.painter_macros.transacao().is_some())
    }

    /// **Os objectos (`StableId`) cuja edição de etapa espera a conta dela** (W14): a imagem nova da
    /// reescrita ou a conferência da água gravada.
    pub(super) fn macros_que_esperam(&self) -> Vec<u64> {
        self.gfx.as_ref().map_or_else(Vec::new, |g| {
            let bits = g.painter_macros.a_reescrever_em();
            ph2d_app_macros::espera::os_que_esperam(g.sim.world(), bits)
        })
    }

    /// O baseline vira um passo (com a transação de uma reescrita que aterrou) e `atual` o baseline novo.
    pub(super) fn empurra(&mut self, atual: super::ProjectState) {
        if let Some(base) = self.undo_baseline.replace(atual) {
            let agora = self.undo_baseline.as_ref().map(|a| &a.world);
            ph2d_app_macros::com(|m| agora.map(|a| m.mundo_mudou(&base.world, a)));
            // ⚠️ **A marca que vai com o passo é a do BASELINE**, não a de agora — ela descreve o
            // estado que está a ser empurrado. E a de agora passa a ser a do baseline novo, pela
            // mesma mecânica do próprio `undo_baseline`.
            let agora = self.field_selection_mark();
            let mark = std::mem::replace(&mut self.undo_baseline_selection, agora);
            let t = self
                .gfx
                .as_mut()
                .and_then(|g| g.painter_macros.toma_a_transacao());
            let link = self.undo_links().and_then(|l| l.pending.take());
            self.undo.push_linked(base, mark, t, link);
            if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                h.documents.scene_stepped();
            }
            self.undo.espera.registou();
        }
    }
}
