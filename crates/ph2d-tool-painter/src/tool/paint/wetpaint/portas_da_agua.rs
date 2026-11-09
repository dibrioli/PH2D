//! As portas da água para gates, miniaturas e sondas (relógio fixo, lei da tinta, semente, fita,
//! largura crua) — filho de [`super`] (`wetpaint.rs`), partido pelo tecto de LOC (movimento puro).

use super::*;

impl PainterTool {
    /// **A água num relógio FIXO** — 40 passos por segundo de `on_tick`, síncronos, pela mesma porta
    /// dos gates de física (`wet_step_sync`). Só para quem pinta FORA do ecrã e precisa da mesma tinta
    /// duas vezes (os gates, a miniatura de um preset); a água do ecrã anda ao relógio real, numa
    /// thread própria (`wetpaint/offthread.rs`).
    pub fn set_wet_relogio_fixo(&mut self, ligado: bool) {
        self.paint.wetpaint.relogio_fixo = ligado.then_some(0.0);
    }

    /// **A lei da tinta** (F4-2 §5): `Velha` (a de sempre) ou `Pigmento` (volumes e somas ópticas). Vive no
    /// `Sim::tinta` da sessão (criada aqui), fora do `Sim::lei` que cada preset reescreve — sobrevive a
    /// `wet_preset_aplicar`, e uma sessão NOVA (desfazer, guarda, grid ratio) herda-a (`ensure_wet_session`).
    /// Só as sondas e os testes a escolhem até a E12 decidir a lei do produto.
    #[cfg(test)]
    pub(crate) fn set_wet_lei_da_tinta(&mut self, lei: ph2d_wet_paint::sim::LeiDaTinta) {
        self.paint.wetpaint.lei_da_tinta = lei;
        if !self.ensure_wet_session() {
            return;
        }
        if let Some(s) = self.paint.wetpaint.session.as_mut() {
            s.bring_home();
            s.engine.set_lei_da_tinta(lei);
        }
    }

    /// A lei da tinta da sessão viva (`None` = sem sessão).
    #[cfg(test)]
    pub(crate) fn wet_lei_da_tinta(&mut self) -> Option<ph2d_wet_paint::sim::LeiDaTinta> {
        let s = self.paint.wetpaint.session.as_mut()?;
        s.bring_home();
        Some(s.engine.sim.tinta)
    }

    /// **A semente do PRÓXIMO traço** (cada traço tira a sua e avança-a): os gates que medem um meio SORTEADO (o
    /// pó do Airbrush) pela mediana de várias tiragens — uma só mede a sorte (doc 51 §4: as pontas 0,54 a 1,03).
    pub fn set_semente_dos_tracos(&mut self, semente: u64) {
        self.paint.seed = semente;
    }

    /// **O worker da água num ritmo escrito** ([`fita::RitmoDaAgua`]) — para os gates que gravam a água
    /// NO worker e precisam da mesma gravação duas vezes. Vale para as sessões que nascerem.
    pub fn set_wet_ritmo_de_teste(&mut self, ritmo: Option<fita::RitmoDaAgua>) {
        self.paint.wetpaint.ritmo = ritmo;
    }

    /// **A água parou** — sem sessão molhada, ou o motor sem fluido nem passo em voo: o passo seguinte
    /// já não corre (`sim_step_stage` devolve sem mexer). A régua do tutorial mede aqui (W7).
    pub(crate) fn wet_assentada(&mut self) -> bool {
        let Some(s) = self.paint.wetpaint.session.as_mut() else {
            return true;
        };
        s.bring_home();
        !s.engine.sim.step_pending() && !s.engine.active_grid().has_fluid
    }

    /// **SONDA:** o motor da sessão, depois de os factos do painel lá chegarem — para medir uma lei
    /// candidata que o painel ainda não escreve (`tests/it/sonda_midias.rs`, `PH2D_SONDA_LEI`). Os
    /// factos seguintes só reescrevem o motor se mudarem.
    #[doc(hidden)]
    pub fn wet_sonda_motor(&mut self, f: impl FnOnce(&mut ph2d_wet_paint::painter::Engine)) {
        if !self.ensure_wet_session() {
            return;
        }
        let facts = self.wet_facts();
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            sess.bring_home();
            sess.reconcile_facts(facts);
            f(&mut sess.engine);
        }
    }

    /// Instala (ou tira) a fita da macro: no estado, para as sessões que nascerem, e na sessão viva.
    pub(crate) fn wet_instala_fita(&mut self, f: Option<fita::FitaPartilhada>) {
        if let Some(s) = self.paint.wetpaint.session.as_mut() {
            s.fita.clone_from(&f);
        }
        self.paint.wetpaint.fita = f;
    }

    /// **A água de uma reprodução passa ao relógio fixo** e larga a fita: depois de uma etapa editada,
    /// as respostas gravadas do worker já não são as desta tinta. O worker corre `STEP_S` de sim por
    /// `STEP_S` de parede — o relógio fixo é a cadência que ele perseguia.
    pub(crate) fn wet_passa_ao_relogio_fixo(&mut self) {
        if let Some(s) = self.paint.wetpaint.session.as_mut() {
            s.solta_o_roteiro();
        }
        self.paint.wetpaint.fita = None;
        if self.paint.wetpaint.relogio_fixo.is_none() {
            self.paint.wetpaint.relogio_fixo = Some(0.0);
        }
    }

    /// O motor da água volta para casa (bloqueia ≤ um estágio) — o fim de uma gravação, nos dois lados.
    pub(crate) fn wet_traz_para_casa(&mut self) {
        if let Some(s) = self.paint.wetpaint.session.as_mut() {
            s.bring_home();
        }
    }

    /// **A lei CRUA da largura** — o carimbo chega ao motor com `escala ×` o raio pedido, sem a promessa
    /// da W2 (`escala` > 1 mede raios do motor além da faixa do slider, `BRUSH_SIZE_MAX_PX`). Para a
    /// sonda que MEDE a tabela de um preset (`tests/it/presets_de_fabrica.rs`) e para os portões da
    /// FÍSICA, que medem uma lei num carimbo do motor de tamanho fixo (a W2 só re-rotula o número do
    /// painel). Aplicar um preset volta a ligar a promessa.
    pub fn set_wet_largura_crua(&mut self, escala: f32) {
        self.paint.wetpaint.largura =
            (escala != 1.0).then(|| largura::LeiDaLargura::escala(escala));
    }

    /// A lei da largura do pincel de agora, como um preset a guarda (`None` = sem promessa).
    #[must_use]
    pub fn wet_largura_tabela(&self) -> Option<Vec<[f64; 2]>> {
        self.paint
            .wetpaint
            .largura
            .as_ref()
            .map(largura::LeiDaLargura::tabela)
    }
}
