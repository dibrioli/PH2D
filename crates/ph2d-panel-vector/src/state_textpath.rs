//! O estado do **Text on Path** publicado pela shell — irmão de `state.rs` pelo teto de 600 LOC
//! daquele arquivo, e coeso pelo mesmo critério que já separou os Effects e o Envelope.
//!
//! Três fatos, e cada um só a shell os sabe:
//!
//! - **`can_link`** — *"a seleção é um texto MAIS um caminho?"*. É a mesma classe do
//!   `Join Selected Bodies` da física: uma pergunta sobre a SELEÇÃO, feita uma vez, com o
//!   pintor a decidir se oferece e o arm a honrar. O painel não vê a cena.
//! - **`linked`** — se o texto corrente já cavalga alguma coisa; decide se a seção mostra o
//!   botão de prender ou os controles do que está preso.
//! - **`offset`/`flip`** — os valores VIVOS do vínculo, para os controles nascerem no lugar
//!   certo em vez de saltarem no primeiro frame.

use std::cell::Cell;

thread_local! {
    static CAN_LINK: Cell<bool> = const { Cell::new(false) };
    static LINKED: Cell<bool> = const { Cell::new(false) };
    static OFFSET: Cell<f64> = const { Cell::new(0.0) };
    static FLIP: Cell<bool> = const { Cell::new(false) };
}

/// Publica se a seleção corrente permite **prender** um texto a um caminho.
///
/// O gesto exige exatamente um texto e um caminho que não seja ele — contar isso é da shell, e
/// o painel só precisa saber se oferece o botão. Um botão que não faz nada é pior que um botão
/// que falta, e este é o único jeito de o painel saber a diferença.
pub fn set_current_textpath_can_link(v: bool) {
    CAN_LINK.with(|c| c.set(v));
}

pub(crate) fn can_link() -> bool {
    CAN_LINK.with(Cell::get)
}

/// Publica o estado do vínculo do texto corrente: se existe, e com que valores.
pub fn set_current_textpath(linked: bool, offset: f64, flip: bool) {
    LINKED.with(|c| c.set(linked));
    OFFSET.with(|c| c.set(offset));
    FLIP.with(|c| c.set(flip));
}

pub(crate) fn linked() -> bool {
    LINKED.with(Cell::get)
}

pub(crate) fn offset() -> f64 {
    OFFSET.with(Cell::get)
}

pub(crate) fn flip() -> bool {
    FLIP.with(Cell::get)
}
