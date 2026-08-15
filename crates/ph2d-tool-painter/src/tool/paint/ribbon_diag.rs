//! **`PH2D_RIBBON_DIAG=1`** — quem pinta o quê num traço, impresso no pen-up.
//!
//! ⚠️ **Ele existe porque a DEDUÇÃO falhou sete vezes.** O report de 2026-08-15 (espículas: retas
//! atravessando o desenho, em leque nos picos) não reproduz por nenhuma via que eu dirija — nem no
//! motor (`begin`/`extend`/`tick`/`settle`/`finish`), nem pela porta do produto
//! (`on_canvas_pointer` + `on_tick`), com ponteiro fino ou grosso, `dt` com jitter, e os três
//! sliders nos extremos. As sete hipóteses foram medidas e refutadas, uma a uma.
//!
//! *Quando a dedução falha, o que falta é um número vindo da máquina onde o defeito acontece.*
//! Esta linha imprime, por traço: o tipo VIVO, os parâmetros VIVOS e **quantos dabs cada fonte
//! emitiu** — porque a pergunta que sobra é *qual subsistema desenha aquelas retas*, e cada fonte
//! tem um contador próprio.

use std::sync::atomic::{AtomicUsize, Ordering};

/// De onde veio um lote de dabs.
#[derive(Clone, Copy)]
pub(crate) enum Source {
    /// O evento de ponteiro (`paint_extend`).
    Extend,
    /// O tique do quadro (`paint_tick`) — numa FITA, é o único que percorre caminho.
    Tick,
    /// O quadro PARADO (`Stroke::settle`, o airbrush).
    Settle,
    /// O pen-up (`Stroke::finish`) — numa fita, a cauda.
    Finish,
}

static N: [AtomicUsize; 4] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];

/// Está armado?
pub(crate) fn on() -> bool {
    std::env::var_os("PH2D_RIBBON_DIAG").is_some()
}

/// Conta um lote. ⚠️ **Inerte sem a env var** — um `if` e nada mais.
pub(crate) fn note(src: Source, n: usize) {
    if n > 0 && on() {
        N[src as usize].fetch_add(n, Ordering::Relaxed);
    }
}

/// Imprime o traço que acabou e zera os contadores.
pub(crate) fn report(b: &ph2d_painter_brush::BrushSpec) {
    if !on() {
        return;
    }
    let take = |i: usize| N[i].swap(0, Ordering::Relaxed);
    let (ext, tick, settle, fin) = (take(0), take(1), take(2), take(3));
    println!(
        "[ribbon-diag] tipo={:?} peso={:.3} atrito={:.3} grav={:.3} | estab={:.2} spacing={:.3} \
         jitter={:.3} | dabs: extend={ext} tique={tick} settle={settle} cauda={fin}",
        b.line_kind,
        b.ribbon_weight,
        b.ribbon_friction,
        b.ribbon_gravity,
        b.stabilizer,
        b.spacing,
        b.jitter,
    );
}
