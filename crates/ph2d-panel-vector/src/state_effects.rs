//! O estado publicado da seção **EFFECTS** (ADR-0132) — módulo irmão do [`super`] pelo teto
//! de 600 LOC.
//!
//! O painel nunca lê a cena: ele vive de um snapshot que a shell publica por frame. Aqui só
//! mora o que a seção precisa saber — o Trim do caminho selecionado, se houver.

use std::cell::Cell;

thread_local! {
    /// O Trim do caminho selecionado: `None` = a pilha dele não tem efeito nenhum.
    ///
    /// **Uma célula só, com a tupla inteira**, e isso é deliberado: publicar `has_trim` e os
    /// parâmetros em setters separados deixaria uma janela em que o painel desenha o `end`
    /// novo com o `start` velho. Mesma razão do `set_current_envelope_presets`.
    static CURRENT_TRIM: Cell<Option<(f64, f64, f64)>> = const { Cell::new(None) };
}

/// **Publica o Trim do caminho selecionado** — `(start, end, offset)`, ou `None` quando ele
/// não tem efeito na pilha.
pub fn set_current_trim(trim: Option<(f64, f64, f64)>) {
    CURRENT_TRIM.with(|c| c.set(trim));
}

/// O Trim publicado, ou `None`.
pub(crate) fn trim() -> Option<(f64, f64, f64)> {
    CURRENT_TRIM.with(Cell::get)
}
