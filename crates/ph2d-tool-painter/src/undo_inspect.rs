//! **A superfície que só a REDE e as SONDAS usam** — filho de [`super`] (`#[path]`, então ele enxerga
//! os privados), split pelo cap de LOC e pela linha de corte natural: o pai é *o que o controller
//! FAZ*, aqui mora *o que se OLHA (e, na sonda, o que se ABLACIONA) nele*.
//!
//! ⚠️ **Nenhum caminho de produto chama o que está aqui.** O acessor é gateado
//! (`cfg(test, debug_assertions)`), e é isso que o torna seguro de expor: um que desse `&mut` ao
//! cursor num build de release seria uma segunda porta para *"quem é o estado adjacente?"* — a
//! pergunta que o [`set_cursor`](super::UndoController::set_cursor) responde uma vez.
//!
//! ⚠️ **E o `probe_drop_cursor_relief` MORREU aqui**: ele simulava o EFEITO da elisão do cursor para
//! medir o prêmio antes de o desenho existir. O degrau 4 construiu a coisa, e a ablação passou a ser
//! pela ENTRADA (`UndoController::elide_cursor`) — uma sonda que finge o que o produto agora faz é
//! uma segunda resposta esperando alguém chamá-la.

use super::{ModelSnapshot, UndoController};

impl UndoController {
    /// O **cursor** — só para a rede de verificação do S3 (ver
    /// [`crate::undo_planes::PlaneDeltas::divergences`]). Ele é a base de todo delta, e a pergunta que a
    /// rede faz é se o estado VIVO do tool poderia sê-lo no lugar dele.
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn cursor_for_audit(&self) -> Option<&ModelSnapshot> {
        self.cursor.as_deref()
    }

    /// **Derruba as ENTRADAS e deixa o cursor de pé** — a ablação que separa os dois donos que o
    /// `undo.clear()` remove de uma vez (doc 28 §5.66 §3).
    ///
    /// ⚠️ Ela não finge nada que o produto faça: é a metade de um `clear()`, e existe porque a pergunta
    /// *"quem segura o plano vivo?"* tem duas respostas possíveis no controller e a contagem de donos
    /// sozinha não as distingue.
    #[cfg(test)]
    pub(crate) fn probe_drop_entries(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.bytes = 0;
    }

    /// **A variante que o TOPO da história guardou por plano** — ver
    /// [`PlaneDeltas::variant_report`](crate::undo_planes::PlaneDeltas::variant_report).
    #[cfg(test)]
    pub(crate) fn probe_top_variants(&self) -> Option<String> {
        Some(self.undo.last()?.planes.variant_report())
    }
}
