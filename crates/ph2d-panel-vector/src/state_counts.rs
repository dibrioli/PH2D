//! **As CONTAGENS da seleção do painel Vector** — irmão de [`super`] pelo teto de 600 LOC.
//!
//! O corte é por RESPONSABILIDADE: o irmão publica *o que a seleção É* (tipo de vértice, estilo,
//! preenchimento, texto…); aqui mora *quantos há*, que é a pergunta que decide se um gesto de
//! CONJUNTO é sequer **oferecido** — Align (≥2 caminhos), Distribute (≥3), Join (≥2) e Average
//! (≥2 nós).
//!
//! ⚠️ **Um gesto de conjunto oferecido abaixo do mínimo é um botão morto**, e a W4 pagou esse preço
//! duas vezes no mesmo dia: o Average nasceu visível com UM nó (mudo no estado mais comum de
//! todos), e os dois pills de corte nasceram fora do `populate` — pintados, acesos sob o mouse e
//! MUDOS. É por isso que as duas contagens vivem juntas: são a mesma pergunta.

use std::cell::Cell;

thread_local! {
    /// Quantos CAMINHOS estão na seleção de objeto — Align (≥2) / Distribute (≥3) / Join (≥2).
    static CURRENT_SELECTION_COUNT: Cell<usize> = const { Cell::new(0) };
    /// Quantos NÓS estão selecionados. O `VertexSel` diz o TIPO (uniforme ou misto) e **não** a
    /// contagem, e o Average precisa exactamente dela.
    static CURRENT_VERTEX_COUNT: Cell<usize> = const { Cell::new(0) };
}

/// Publica quantos CAMINHOS a seleção de objeto tem (shell → painel, a cada frame).
pub fn set_current_selection_count(n: usize) {
    CURRENT_SELECTION_COUNT.with(|c| c.set(n));
}

/// A contagem de caminhos deste frame (≥2 mostra Align e Join, ≥3 mostra Distribute).
#[must_use]
pub(crate) fn current_selection_count() -> usize {
    CURRENT_SELECTION_COUNT.with(Cell::get)
}

/// Publica quantos NÓS estão selecionados (shell → painel, a cada frame).
pub fn set_current_vertex_count(n: usize) {
    CURRENT_VERTEX_COUNT.with(|c| c.set(n));
}

/// A contagem de nós deste frame (≥2 mostra o Average).
#[must_use]
pub(crate) fn current_vertex_count() -> usize {
    CURRENT_VERTEX_COUNT.with(Cell::get)
}
