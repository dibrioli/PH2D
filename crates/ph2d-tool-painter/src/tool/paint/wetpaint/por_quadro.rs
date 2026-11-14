//! **A tela do Wet Paint compõe-se UMA vez por quadro** (R3, 31/10) — a lei da pilha
//! ([`super::super::composite_por_quadro`]) levada ao fluido. Medição: `docs/Painter/49_realshader/R3_PINTAR_RELATORIO.md`.
//!
//! Cada lote de dabs recompunha a região suja do motor no próprio evento; com o Óleo Real ela cresce pelo
//! alcance da luz (~32 células de cada lado) e um traço de faca de 3 px por evento refazia a mesma área dezenas
//! de vezes. O depósito continua dab a dab, no evento; só a escrita na tela espera pela drenagem, e o sujo do
//! motor ACUMULA a união até lá. A composição é uma função das grades sobre a região (crescida pelo alcance de
//! quem a lê), logo compor a união uma vez dá os bytes que a última composição de cada píxel daria.
//!
//! Adia só com o hospedeiro a drenar por quadro e sem leitor da tela no caminho do carimbo — o MESMO predicado
//! da pilha ([`PainterTool::pilha_pode_adiar`]). As portas que compõem o pendente são as da pilha
//! ([`PainterTool::compoe_o_pendente`]: as drenagens, o pen-down seguinte, o fecho do traço…) e o tique, antes
//! de o motor ir para o worker. Qualquer composição limpa o pendente (`wetpaint_composite_veiled`).

use super::*;

impl PainterTool {
    /// O fim do lote: compor já, ou deixar para a drenagem do quadro.
    pub(in crate::tool::paint) fn wet_compoe_ou_adia(&mut self) {
        if self.pilha_pode_adiar() {
            self.paint.wetpaint.composite_pendente = true;
            self.preview_dirty = true;
        } else {
            self.wetpaint_composite();
        }
    }

    /// Compor o que os lotes deste quadro deixaram por compor. Com o motor no worker não espera: a composição do
    /// tique que o trouxer leva a união (o sujo do motor não se perde).
    pub(in crate::tool::paint) fn wet_compoe_o_pendente(&mut self) {
        if !std::mem::take(&mut self.paint.wetpaint.composite_pendente) {
            return;
        }
        // O lote ainda com o operário (F2d): o quadro não espera, o seguinte compõe.
        if !self.paint.wetpaint.session.assentou() {
            self.paint.wetpaint.composite_pendente = true;
            return;
        }
        if self
            .paint
            .wetpaint
            .session
            .as_mut()
            .is_some_and(WetSession::try_bring_home_now)
        {
            let t0 = std::time::Instant::now();
            self.no_pool(Self::wetpaint_composite);
            super::sessao_do_wet::diag_espera("compoe", t0, std::panic::Location::caller());
        }
    }

    /// **Corre `f` num fio do pool do rayon** (R3): o lote e a composição lançam dezenas de passes paralelos
    /// pequenos, e cada um lançado de FORA do pool entra pela fila global, acorda os fios e adormece quem
    /// chamou no trinco — medido nas facas 4K: o fio do evento trabalhava 2,9 s em 13,6 s de parede. Dentro do
    /// pool os passes dividem-se pela fila do próprio fio e quem chama trabalha. As contas são as mesmas, no
    /// mesmo fio ou noutro (as linhas de cada passe são independentes por contrato), logo os bytes também.
    /// Nos testes corre no fio do teste (os interruptores de teste são `thread_local!`).
    ///
    /// ⛔ **`f` nunca espera o motor** (`WetSession::bring_home` recusa-o): um operário em `wait_until` rouba este job
    /// por cima de um pedaço do estágio do worker, e o worker só devolve o motor quando esse pedaço acabar.
    pub(in crate::tool::paint) fn no_pool<R: Send>(
        &mut self,
        f: impl FnOnce(&mut Self) -> R + Send,
    ) -> R {
        if cfg!(test)
            || rayon::current_thread_index().is_some()
            || !crate::wet_diag::NO_POOL.load(std::sync::atomic::Ordering::Relaxed)
        {
            return f(self);
        }
        rayon::join(|| f(self), || ()).0
    }
}
