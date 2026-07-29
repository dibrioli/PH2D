//! **O split do tick da água, para o log do produto** (`PH2D_FLUID_PROFILE`).
//!
//! Um pico de 71 ms no `tool-tick` não diz **de que** ele é feito: um passo de sim caro ou um
//! composite sobre um casco grande são a mesma linha de log e curas completamente diferentes. Isto
//! acumula as duas metades sobre a mesma janela que o shell imprime, e o shell as lê.
//!
//! ⚠️ **Contadores atômicos e não um canal:** o tick roda na thread de UI e o leitor é o mesmo
//! frame; o custo é uma soma relaxada por passo, e o caminho sem o profiler não paga nada além de
//! duas somas atômicas (medido irrelevante contra um passo de 12-40 ms).

use std::sync::atomic::{AtomicU64, Ordering};

static STEP_US: AtomicU64 = AtomicU64::new(0);
static STEP_MAX_US: AtomicU64 = AtomicU64::new(0);
static STEP_N: AtomicU64 = AtomicU64::new(0);
static COMP_US: AtomicU64 = AtomicU64::new(0);
static COMP_MAX_US: AtomicU64 = AtomicU64::new(0);
static COMP_N: AtomicU64 = AtomicU64::new(0);
static WAIT_US: AtomicU64 = AtomicU64::new(0);
static WAIT_MAX_US: AtomicU64 = AtomicU64::new(0);
static WAIT_N: AtomicU64 = AtomicU64::new(0);

fn add(sum: &AtomicU64, mx: &AtomicU64, n: &AtomicU64, ms: f32) {
    let us = (f64::from(ms) * 1000.0) as u64;
    sum.fetch_add(us, Ordering::Relaxed);
    mx.fetch_max(us, Ordering::Relaxed);
    n.fetch_add(1, Ordering::Relaxed);
}

/// Um `step_simulation` custou isto.
pub fn note_step(ms: f32) {
    add(&STEP_US, &STEP_MAX_US, &STEP_N, ms);
}

/// Um `wetpaint_composite` custou isto.
pub fn note_composite(ms: f32) {
    add(&COMP_US, &COMP_MAX_US, &COMP_N, ms);
}

/// O tick ESPEROU isto pelo motor (`try_bring_home`) — a 3ª metade, e a única
/// que a sim off-thread criou. Ela existe porque a aritmética do log do produto
/// (tick 308 ms, composite 97,7) deixou **210 ms sem dono**, e atribuir sem medir
/// é a doença do doc 28 §5.13.
pub fn note_wait(ms: f32) {
    add(&WAIT_US, &WAIT_MAX_US, &WAIT_N, ms);
}

/// `(soma ms, pico ms, n)` de cada metade, ZERANDO a janela — o leitor é o `[frame]` do shell.
#[must_use]
pub fn take_window() -> ((f64, f64, u64), (f64, f64, u64), (f64, f64, u64)) {
    let take = |sum: &AtomicU64, mx: &AtomicU64, n: &AtomicU64| {
        (
            sum.swap(0, Ordering::Relaxed) as f64 / 1000.0,
            mx.swap(0, Ordering::Relaxed) as f64 / 1000.0,
            n.swap(0, Ordering::Relaxed),
        )
    };
    (
        take(&STEP_US, &STEP_MAX_US, &STEP_N),
        take(&COMP_US, &COMP_MAX_US, &COMP_N),
        take(&WAIT_US, &WAIT_MAX_US, &WAIT_N),
    )
}
