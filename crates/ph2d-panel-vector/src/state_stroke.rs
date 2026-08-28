//! **ESTA FORMA TEM TRAÇO?** — a resposta que o painel usa para desenhar a caixa de marcar da
//! secção *Stroke* (plano 34), e a **única** deste painel.
//!
//! ⚠️⚠️ **Ela SUBSTITUI o `TokenBindings::stroke_exists`**, que respondia à mesma pergunta noutro
//! sítio. Duas respostas à mesma pergunta divergem no dia em que uma delas ganhar uma condição — e
//! aqui isso seria visível de imediato: a caixa diria *"tem traço"* e a row de token do traço não
//! apareceria, ou o contrário.
//!
//! ⚠️ **O `Option` é a metade que importa.** `None` = a selecção não tem uma resposta *(nada
//! selecionado, ou selecção múltipla)* e a linha **não é pintada** — a mesma lei do `resize_box`:
//! *uma caixa que descreve um objecto que não está lá é pior que caixa nenhuma.*
//!
//! ⛔ **O painel não alcança a cena, e não deve.** Se alcançasse, a resposta que DESENHA a caixa
//! divergiria da que HONRA o clique, e o artista descobriria a divergência clicando.

use std::cell::Cell;

thread_local! {
    /// `Some(tem)` para uma selecção com resposta; `None` quando não há o que descrever.
    static STROKE_PRESENT: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Publica a resposta deste frame (shell → painel).
pub fn set_stroke_present(v: Option<bool>) {
    STROKE_PRESENT.with(|c| c.set(v));
}

/// A forma selecionada tem traço? `None` = a linha não é pintada.
#[must_use]
pub(crate) fn stroke_present() -> Option<bool> {
    STROKE_PRESENT.with(Cell::get)
}
