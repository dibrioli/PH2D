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
//!
//! ⚠️ **E ele conta os FIOS ao lado dos dabs, porque metade do que uma FITA desenha é fio.** O
//! trilho de tinta é de dabs; o trilho de fora e TODA travessa são fios, por um canal próprio
//! (`thread_deposit`), e um instrumento que só contasse dabs diria **`fios=0` sem saber que existem
//! fios** — a forma exata do *instrumento silencioso que TRANQUILIZA* que esta linha já pagou quatro
//! vezes. Numa fita com `Rungs > 0` e `dabs` normais, ver a contagem de fios em **zero** é o
//! diagnóstico de que a faixa não está sendo costurada.
//!
//! ⚠️ **Os fios são um contador SOLTO, não um quinto [`Source`]** — aquele enum responde *de onde
//! veio este lote de DABS*, e as duas grandezas não são somáveis (um fio é um segmento, um dab é um
//! carimbo). Além disso o depósito de fio tem **uma porta só** (`park_stroke`), então não há de-onde
//! a distinguir.

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

/// Quantos fios o MOTOR costurou, e quantos o depósito de fato CARIMBOU.
///
/// ⚠️ **São dois números porque os dois modos de falha são distintos, e este módulo já pagou um
/// deles:** ligar a fita no portão errado deixou o motor a costurar **343 travessas por traço** com
/// o depósito **mudo**, e a imagem saía idêntica ao controle. Um contador só — seja qual for —
/// mostra `0` nos DOIS casos (*o motor não costurou* e *o depósito recusou*) e manda procurar no
/// lugar errado. Com o par, `0/0` acusa o motor e `343/0` acusa o depósito.
static THREADS_SEWN: AtomicUsize = AtomicUsize::new(0);
static THREADS_INKED: AtomicUsize = AtomicUsize::new(0);

/// Conta um lote. ⚠️ **Inerte sem a env var** — um `if` e nada mais.
pub(crate) fn note(src: Source, n: usize) {
    if n > 0 && on() {
        N[src as usize].fetch_add(n, Ordering::Relaxed);
    }
}

/// Conta um feixe de fios: quantos o motor entregou e quantos foram carimbados.
///
/// ⚠️ **Inerte sem a env var**, como o irmão — e `inked` é `0` quando o depósito recusa, que é
/// precisamente o sinal que o par existe para dar.
pub(crate) fn note_threads(sewn: usize, inked: usize) {
    if sewn > 0 && on() {
        THREADS_SEWN.fetch_add(sewn, Ordering::Relaxed);
        THREADS_INKED.fetch_add(inked, Ordering::Relaxed);
    }
}

/// Imprime o traço que acabou e zera os contadores.
pub(crate) fn report(b: &ph2d_painter_brush::BrushSpec) {
    if !on() {
        return;
    }
    let take = |i: usize| N[i].swap(0, Ordering::Relaxed);
    let (ext, tick, settle, fin) = (take(0), take(1), take(2), take(3));
    let sewn = THREADS_SEWN.swap(0, Ordering::Relaxed);
    let inked = THREADS_INKED.swap(0, Ordering::Relaxed);
    println!(
        "[ribbon-diag] tipo={:?} peso={:.3} atrito={:.3} grav={:.3} | estab={:.2} spacing={:.3} \
         jitter={:.3} | dabs: extend={ext} tique={tick} settle={settle} cauda={fin} \
         | fios: cosidos={sewn} carimbados={inked}",
        b.line_kind,
        b.ribbon_weight,
        b.ribbon_friction,
        b.ribbon_gravity,
        b.stabilizer,
        b.spacing,
        b.jitter,
    );
}
