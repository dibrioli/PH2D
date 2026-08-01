//! **O que a shell OBSERVOU sobre a simetria da seleção** — irmão de `state.rs` pelo teto de 600
//! LOC dos painéis.
//!
//! Distinto do `state_expand`, que guarda o que o artista ESCOLHEU: aqui vive um fato da CENA que
//! só a shell enxerga (*quantas formas seleccionadas já têm simetria viva?*), e o painel o lê para
//! decidir se oferece o **Apply**.
//!
//! ⚠️ É essa pergunta que impede o botão morto: sem simetria viva não há cópias a consolidar, e um
//! *Apply* que não aplica nada é pior que *Apply* nenhum. O estilo (que espelho, quantas cópias)
//! não mora aqui — ele é da FERRAMENTA e viaja no `VectorStyleSnapshot`.

use std::cell::Cell;

thread_local! {
    /// Quantas formas da seleção carregam um `ph2d_ecs::VecSymmetry`. A shell o publica por frame.
    static SYMMETRY_LIVE: Cell<usize> = const { Cell::new(0) };
}

/// Quantas formas da seleção têm simetria VIVA neste frame.
#[must_use]
pub fn symmetry_live_count() -> usize {
    SYMMETRY_LIVE.with(Cell::get)
}

/// A shell publica a contagem por frame. `pub` porque quem observa a cena é ela — o painel não
/// alcança o mundo ECS, e uma segunda contagem do lado do painel seria a segunda resposta a
/// *"há o que consolidar?"*.
pub fn set_symmetry_live_count(n: usize) {
    SYMMETRY_LIVE.with(|c| c.set(n));
}
