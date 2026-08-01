//! **A superfície que só a REDE e as SONDAS usam** — filho de [`super`] (`#[path]`, então ele enxerga
//! os privados), split pelo cap de LOC e pela linha de corte natural: o pai é *o que o controller
//! FAZ*, aqui mora *o que se OLHA (e, na sonda, o que se ABLACIONA) nele*.
//!
//! ⚠️ **Nenhum caminho de produto chama o que está aqui.** Os dois membros são gateados
//! (`cfg(test, debug_assertions)` e `cfg(test)`), e é isso que os torna seguros de expor: um acessor
//! que dá `&mut` ao cursor num build de release seria uma segunda porta para *"quem é o estado
//! adjacente?"* — a pergunta que o [`set_cursor`](super::UndoController::set_cursor) responde uma vez.

use super::{ModelSnapshot, UndoController};

impl UndoController {
    /// O **cursor** — só para a rede de verificação do S3 (ver
    /// [`crate::undo_planes::PlaneDeltas::divergences`]). Ele é a base de todo delta, e a pergunta que a
    /// rede faz é se o estado VIVO do tool poderia sê-lo no lugar dele.
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn cursor_for_audit(&self) -> Option<&ModelSnapshot> {
        self.cursor.as_deref()
    }

    /// **A ABLAÇÃO do degrau 4: o cursor larga os três planos de relevo.**
    ///
    /// Só para SONDA — ela simula o EFEITO da elisão sem construir a correção dela, que é como se mede
    /// o prêmio antes de pagar o desenho (doc 28 §5.60). O degrau 3 já fez a materialização do relevo
    /// partir do plano VIVO, então o cursor não é lido para essas três famílias e o estado resultante é
    /// coerente; o que ainda falta para a elisão ser produto é o lado `before` — e a §5.60 mede que
    /// elidi-lo hoje **não remove um dono, troca o dono de lugar** (`OnlyAfter` guarda o plano vivo).
    #[cfg(test)]
    pub(crate) fn probe_drop_cursor_relief(&mut self) {
        if let Some(c) = self.cursor.as_deref_mut() {
            c.heights.clear();
            c.covers.clear();
            c.mats.clear();
        }
    }
}
