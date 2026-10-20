//! **A água depois do fim de um clipe** (W14, `docs/Macros/12_W14_a_agua_depois_do_parar.md`): o tocador de
//! uma macro corre a água da gravação no relógio fixo até assentar, ou até ao estado em que o editor a
//! parou ([`MarcaDaAgua`], guardado na marca do artista) — e pára como o editor parou: a tela composta
//! nesse estado e a sessão acabada. ⛔ Sem recompor a tela inteira: o editor só compõe o que a água
//! sujou, e a inteira diferia em 1/3 dos texels (medido, W14).

use super::fita::MarcaDaAgua;
use super::offthread::STEP_S;
use crate::tool::PainterTool;

impl PainterTool {
    /// A água da gravação ainda corre na tela (a sessão nascida sem a marca está viva): o estado que a
    /// tela mostra dela agora.
    pub(crate) fn wet_agua_da_gravacao_viva(&self) -> Option<MarcaDaAgua> {
        let s = self.paint.wetpaint.session.as_ref()?;
        s.marca.is_none().then_some(self.paint.wetpaint.composta)?
    }

    /// A água anda `dt_ms`. Com `ate`, nunca passa dele. `None` = já não anda (assentada, sem sessão, ou
    /// parada em `ate`); `Some(mudou)` = ainda anda, e se a tela mudou neste quadro (um quadro mais curto
    /// que um passo da água não a muda).
    pub(crate) fn wet_cauda(&mut self, dt_ms: f32, ate: Option<MarcaDaAgua>) -> Option<bool> {
        if self.wet_assentada() {
            return None;
        }
        let Some(alvo) = ate else {
            // A MESMA porta da imagem do fim (`imagens_da_gravacao`): assenta nela ao byte.
            let antes = self.paint.wetpaint.composta;
            self.drena_sem_compor();
            ph2d_editor_core::tool::Tool::on_tick(self, dt_ms);
            return Some(self.paint.wetpaint.composta != antes);
        };
        let divida = self.paint.wetpaint.relogio_fixo.get_or_insert(0.0);
        *divida += f64::from(dt_ms) / 1000.0 / STEP_S;
        let passos = divida.floor();
        *divida -= passos;
        let facts = self.wet_facts();
        let s = self.paint.wetpaint.session.as_mut()?;
        s.bring_home();
        s.reconcile_facts(facts);
        let (mut restam, mut andou) = (passos as usize, false);
        loop {
            let agora = MarcaDaAgua::de(&s.engine);
            if agora.ordem() >= alvo.ordem() {
                // Como o editor: a tela é o composite DESTE estado e a sessão acaba nele.
                if andou {
                    self.wetpaint_composite();
                }
                self.wetpaint_end_session();
                return Some(andou);
            }
            if restam == 0 || (agora.estagio.is_none() && !s.engine.sim_should_run()) {
                break;
            }
            if s.engine.step_stage().is_some() {
                restam -= 1;
            }
            andou = true;
        }
        if andou {
            self.wetpaint_composite();
        }
        Some(andou)
    }
}
