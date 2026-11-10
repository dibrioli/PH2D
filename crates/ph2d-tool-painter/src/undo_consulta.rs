//! **O QUE SE PERGUNTA AO HISTÓRICO SEM O MUDAR** — filho de [`super`] (`#[path]`, enxerga os privados),
//! partido pelo tecto de LOC (R4): o pai é o que o controller FAZ, aqui o que ele RESPONDE.

use super::{UndoController, UndoEntry};

impl UndoController {
    /// `true` if there is at least one edit to undo (drives the `undo_enabled`
    /// affordance).
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// `true` if there is at least one undone edit to redo (drives the
    /// `redo_enabled` affordance).
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// A transação do passo que um desfazer (`redo == false`) ou um refazer aplicaria agora.
    #[must_use]
    pub fn transacao_do_topo(&self, redo: bool) -> Option<u64> {
        if redo {
            self.redo.last()
        } else {
            self.undo.last()
        }
        .and_then(|e| e.transacao)
    }

    /// Retained undo depth (for tests / memory introspection).
    #[must_use]
    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    /// Retained redo depth.
    #[must_use]
    pub fn redo_depth(&self) -> usize {
        self.redo.len()
    }

    /// **Os bytes que o histórico de fato retém** — o número que o cap conta e que o gate de memória
    /// mede. Ele soma as duas pilhas: uma sessão de undos não devolve memória, ela a move de lado.
    #[must_use]
    pub fn retained_bytes(&self) -> usize {
        self.bytes
    }

    /// **Os bytes que CLONAR o histórico aloca** (as duas pilhas): as janelas de pixels dos passos. Os planos
    /// inteiros são `Arc` partilhados com o original — `retained_bytes` os conta (é o teto do cap), este não.
    #[must_use]
    pub fn bytes_copiados(&self) -> usize {
        self.undo
            .iter()
            .chain(&self.redo)
            .map(UndoEntry::copiado_bytes)
            .sum()
    }

    /// A chave ABSOLUTA do próximo passo (= profundidade + passos já descartados por baixo): não se repete quando o
    /// tecto tira o passo mais velho nem depois de um [`Self::clear`].
    #[must_use]
    pub fn chave_do_topo(&self) -> usize {
        self.descartados + self.undo.len()
    }
}
