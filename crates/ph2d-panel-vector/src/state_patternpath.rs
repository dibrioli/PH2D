//! O estado do **Pattern on Path** publicado pela shell — irmão de `state.rs` pelo teto de 600 LOC,
//! e coeso pelo mesmo critério que já separou os Effects, o Envelope e o Text on Path.
//!
//! Três fatos, e cada um só a shell os sabe:
//!
//! - **`can_link`** — *"a seleção é dois caminhos?"* (o gesto prende o primário ao outro). É a mesma
//!   classe do `can_link` do texto: uma pergunta sobre a SELEÇÃO, feita uma vez, com o pintor a
//!   decidir se oferece e o arm a honrar. O painel não vê a cena.
//! - **`linked`** — se o motivo em foco (o primário) já cavalga alguma coisa; decide se a seção
//!   mostra o botão de prender ou os controles do que está preso.
//! - **`start`/`spacing`/`flip`** — os valores VIVOS do vínculo, para os controles nascerem no
//!   lugar certo em vez de saltarem no primeiro frame.

use std::cell::Cell;

thread_local! {
    static CAN_LINK: Cell<bool> = const { Cell::new(false) };
    static LINKED: Cell<bool> = const { Cell::new(false) };
    static START: Cell<f64> = const { Cell::new(0.0) };
    static SPACING: Cell<f64> = const { Cell::new(1.0) };
    static FLIP: Cell<bool> = const { Cell::new(false) };
}

/// Publica se a seleção corrente permite **prender** um motivo a um caminho (dois caminhos).
pub fn set_current_patternpath_can_link(v: bool) {
    CAN_LINK.with(|c| c.set(v));
}

pub(crate) fn can_link() -> bool {
    CAN_LINK.with(Cell::get)
}

/// Publica o estado do vínculo do motivo corrente: se existe, e com que valores.
pub fn set_current_patternpath(linked: bool, start: f64, spacing: f64, flip: bool) {
    LINKED.with(|c| c.set(linked));
    START.with(|c| c.set(start));
    SPACING.with(|c| c.set(spacing));
    FLIP.with(|c| c.set(flip));
}

pub(crate) fn linked() -> bool {
    LINKED.with(Cell::get)
}

pub(crate) fn start() -> f64 {
    START.with(Cell::get)
}

pub(crate) fn spacing() -> f64 {
    SPACING.with(Cell::get)
}

pub(crate) fn flip() -> bool {
    FLIP.with(Cell::get)
}
