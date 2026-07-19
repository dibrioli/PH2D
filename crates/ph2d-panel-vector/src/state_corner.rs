//! O estado publicado do **estilo de quina** (Live Corners — ADR-0121) — módulo irmão do
//! [`super`] pelo teto de 600 LOC, no mesmo padrão do `state_effects`.
//!
//! Uma coisa só: a seleção tem uma quina COM raio, e ela é chanfro ou arredondado? A shell
//! publica por frame (do `PenTool::selected_corner_chamfer`), o painel reflete no toggle
//! Chamfer, e o clique volta pela shell que alterna o SINAL do `corner_radius`.

use std::cell::Cell;

thread_local! {
    /// `Some(true)` = chanfro, `Some(false)` = arredondado, `None` = nenhuma quina com raio na
    /// seleção → o toggle Chamfer nem aparece (sem raio não há estilo a mostrar).
    static CURRENT_CORNER_CHAMFER: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Publica o estilo de quina da seleção (a shell chama por frame).
pub fn set_current_corner_chamfer(state: Option<bool>) {
    CURRENT_CORNER_CHAMFER.with(|c| c.set(state));
}

/// O estilo de quina da seleção neste frame.
pub(crate) fn current_corner_chamfer() -> Option<bool> {
    CURRENT_CORNER_CHAMFER.with(Cell::get)
}
