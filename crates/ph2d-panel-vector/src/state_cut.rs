//! **A LINHA DE CORTE existe?** — o único fato que o painel precisa saber sobre o corte.
//!
//! Irmão de [`crate::state_counts`] e pela mesma razão: é a pergunta que decide se um gesto é
//! **oferecido**. Os dois botões do corte (`Cut` · `Discard Cut Line`) só fazem sentido com uma
//! lâmina desenhada — pintados sem ela, seriam dois botões mudos no estado em que o artista
//! passa a maior parte do tempo (a lei que o Average e os dois pills desta mesma wave já
//! pagaram, no mesmo dia).
//!
//! ⚠️ **A verdade mora no ECS** (`ph2d_ecs::VecCutPath`), não aqui: isto é a projeção que a
//! shell publica por frame, como toda a fronteira deste painel. O painel não pode consultar o
//! mundo — e não deve: se pudesse, haveria duas respostas para *"há lâmina?"*.

use std::cell::Cell;

thread_local! {
    /// Há uma linha de corte na cena neste frame?
    static CUT_LINE_EXISTS: Cell<bool> = const { Cell::new(false) };
}

/// Publica se existe linha de corte (shell → painel, a cada frame).
pub fn set_cut_line_exists(exists: bool) {
    CUT_LINE_EXISTS.with(|c| c.set(exists));
}

/// Há lâmina desenhada? É isto que faz os dois botões do corte serem oferecidos.
#[must_use]
pub(crate) fn cut_line_exists() -> bool {
    CUT_LINE_EXISTS.with(Cell::get)
}
