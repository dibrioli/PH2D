//! ⭐ **O peso EXACTO** — um âmbito em que o reforço de nitidez do preset
//! ([`ph2d_tokens::crisp_weight_boost_for`]: `+300` a `≤ 12 px` no `CrispHeavyPlus` de fábrica) não
//! se soma ao peso pedido. Para a superfície cujo desenho dá o peso final em CSS (a aba de scripts,
//! maquete 14): sem ele o corpo `400` saía `700` no app (`600` com o estilo `Light` do dono), e os
//! testes, que correm no preset `Default`, mediam `400`. O desvio do estilo do artista
//! ([`ph2d_tokens::UiWeight`]) continua a valer. Medir e pintar entram no mesmo âmbito.

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
