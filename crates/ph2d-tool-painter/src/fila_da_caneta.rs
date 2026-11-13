//! **A fila da caneta (B27.1):** o handler do winit só ENFILEIRA o Move de um traço aberto; o quadro
//! drena-a no topo, evento a evento, pela mesma entrega de sempre. Cada evento leva o que a entrega lia
//! do estado VIVO (pressão, tilt, Shift/Ctrl/Alt): drenado depois, dá o mesmo traço ao bit.
//!
//! ⚠️ BARREIRA: quem lê ou muda a tela, a câmara ou o pincel entre dois Moves drena antes (a shell
//! drena em todo evento de janela que não é `CursorMoved`, e no `Up`). Juntar eventos num lote é o
//! B27.2 e só com o portão de partição (`docs/Painter/missao_4k/BACKLOG.md` §B27).

use std::cell::RefCell;
use std::collections::VecDeque;

/// Um Move de canvas como a entrega o veria no instante em que chegou.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Evento {
    /// Ecrã, px.
    pub pos: (f32, f32),
    pub pressure: f32,
    pub tilt: [f32; 2],
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl Evento {
    /// `mods` = `[shift, ctrl, alt]`.
    #[must_use]
    pub fn novo(pos: (f32, f32), pressure: f32, tilt: [f32; 2], mods: [bool; 3]) -> Self {
        let [shift, ctrl, alt] = mods;
        Self {
            pos,
            pressure,
            tilt,
            shift,
            ctrl,
            alt,
        }
    }
}

thread_local! {
    static FILA: RefCell<VecDeque<Evento>> = const { RefCell::new(VecDeque::new()) };
}

/// O handler: guarda o evento para o quadro.
pub fn enfileira(e: Evento) {
    FILA.with(|f| f.borrow_mut().push_back(e));
}

/// Quantos esperam o quadro.
#[must_use]
pub fn pendentes() -> usize {
    FILA.with(|f| f.borrow().len())
}

/// Descarta tudo (o traço morreu).
pub fn descarta() {
    FILA.with(|f| f.borrow_mut().clear());
}

/// O que um [`drena`] fez: os entregues (sem o recusado), se o traço segue vivo e o custo em µs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dreno {
    pub entregues: u32,
    pub vivo: bool,
    pub us: u64,
}

/// Entrega cada evento, na ordem de chegada, a `entrega`; um `false` (o traço morreu) descarta o resto.
/// O empréstimo da fila larga-se antes de cada entrega (ela pode enfileirar de novo).
pub fn drena(mut entrega: impl FnMut(Evento) -> bool) -> Dreno {
    let mut d = Dreno {
        entregues: 0,
        vivo: true,
        us: 0,
    };
    if pendentes() == 0 {
        return d;
    }
    let t0 = std::time::Instant::now();
    while let Some(e) = FILA.with(|f| f.borrow_mut().pop_front()) {
        if !entrega(e) {
            d.vivo = false;
            descarta();
            break;
        }
        d.entregues = d.entregues.saturating_add(1);
    }
    d.us = t0.elapsed().as_micros() as u64;
    d
}

#[cfg(test)]
#[path = "fila_da_caneta_tests.rs"]
mod tests;
