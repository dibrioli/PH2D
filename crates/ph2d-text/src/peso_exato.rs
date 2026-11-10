//! ⭐ **O peso EXACTO** — um âmbito em que o reforço de nitidez do preset
//! ([`ph2d_tokens::crisp_weight_boost_for`]) não se soma ao peso pedido: a superfície cujo desenho
//! dá o peso final em CSS (a aba de scripts) pinta esse peso. O desvio do estilo do artista
//! ([`ph2d_tokens::UiWeight`]) continua a valer. Medir e pintar entram no mesmo âmbito. Porquê:
//! `docs/CodeEditor/handoffs/HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-10_ACABAMENTO.md`.

use std::cell::Cell;

thread_local! {
    static EXATO: Cell<bool> = const { Cell::new(false) };
}

/// Corre `f` com o peso exacto (aninhável; repõe o estado anterior mesmo em pânico).
pub fn com_peso_exato<R>(f: impl FnOnce() -> R) -> R {
    struct Repoe(bool);
    impl Drop for Repoe {
        fn drop(&mut self) {
            EXATO.with(|c| c.set(self.0));
        }
    }
    let _repoe = Repoe(EXATO.with(|c| c.replace(true)));
    f()
}

/// Se o âmbito do peso exacto está aberto nesta thread.
pub(crate) fn ativo() -> bool {
    EXATO.with(Cell::get)
}

#[cfg(test)]
#[path = "peso_exato_tests.rs"]
mod tests;
