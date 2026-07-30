//! **O split do tick da água, para o log do produto** (`PH2D_FLUID_PROFILE`).
//!
//! Um pico de 71 ms no `tool-tick` não diz **de que** ele é feito: um passo de sim caro ou um
//! composite sobre um casco grande são a mesma linha de log e curas completamente diferentes. Isto
//! acumula as duas metades sobre a mesma janela que o shell imprime, e o shell as lê.
//!
//! ⚠️ **Contadores atômicos e não um canal:** o tick roda na thread de UI e o leitor é o mesmo
//! frame; o custo é uma soma relaxada por passo, e o caminho sem o profiler não paga nada além de
//! duas somas atômicas (medido irrelevante contra um passo de 12-40 ms).
//!
//! # A metade do WORKER (2026-07-29) — e o buraco que ela fecha
//!
//! Com a sim fora da thread do frame, **`sim media` passou a imprimir `0.00ms x0`**: nada mais
//! chamava [`note_step`], porque quem dá o passo é o worker. A linha lia-se como *"a simulação não
//! custa nada"* quando significava *"ninguém mede a simulação"* — e era exatamente o número que
//! decidia se a água lenta é **trabalho** ou **agendamento**.
//!
//! Os três baldes abaixo são a resposta, e eles PARTICIONAM a janela do worker:
//!
//! - **busy** — dentro de `step_stage` (o trabalho da física);
//! - **away** — o motor está com o frame (o worker bloqueado no `recv`);
//! - **sleep** — o ritmo de 40 Hz (`IDLE_SLEEP`), o worker adiantado de propósito.
//!
//! Sobra = o que os três não explicam. E é a leitura DELES que separa três mundos com curas
//! opostas: *busy ≈ 100%* é work-limited (só a GPU move) · *away* grande é o **handshake** custando
//! a taxa · *sleep* grande diz que a água já alcançou o nominal e o gargalo está noutro lugar.

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
///
/// ⚠️ **É o tempo de COMPUTE do passo, somado sobre os estágios dele** — nunca o
/// wall-clock de ponta a ponta, que inclui o motor viajando para o frame e
/// voltar. Os dois números existem e respondem perguntas diferentes: este diz
/// *quanto a física custa*; o span diz *quanto tempo a água levou*, e a diferença
/// entre eles é o [`note_away`].
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

// ---------------------------------------------------------------------------
// A metade do WORKER — os três baldes que particionam a janela dele
// ---------------------------------------------------------------------------

static BUSY_US: AtomicU64 = AtomicU64::new(0);
static AWAY_US: AtomicU64 = AtomicU64::new(0);
static SLEEP_US: AtomicU64 = AtomicU64::new(0);

/// O worker gastou isto DENTRO de um `step_stage` (o trabalho da física).
pub fn note_busy(us: u64) {
    BUSY_US.fetch_add(us, Ordering::Relaxed);
}

/// O motor esteve com o FRAME por isto — o worker bloqueado no `recv`.
///
/// ⚠️ É o intervalo do `send` até o `recv` seguinte VOLTAR, e não a duração do
/// composite: entre os dois cabe a espera do tick, o composite, o resto do
/// frame e a viagem de volta pelo canal. É o preço do handshake, e é por isso
/// que ele é medido do lado que o PAGA.
pub fn note_away(us: u64) {
    AWAY_US.fetch_add(us, Ordering::Relaxed);
}

/// O worker dormiu isto porque estava ADIANTADO (o ritmo de 40 Hz da SPEC).
pub fn note_sleep(us: u64) {
    SLEEP_US.fetch_add(us, Ordering::Relaxed);
}

/// `(busy, away, sleep)` em ms, ZERANDO a janela.
#[must_use]
pub fn take_worker() -> (f64, f64, f64) {
    let take = |c: &AtomicU64| c.swap(0, Ordering::Relaxed) as f64 / 1000.0;
    (take(&BUSY_US), take(&AWAY_US), take(&SLEEP_US))
}

/// Uma metade do tick, na janela: `(soma ms, pico ms, n)`.
pub type Half = (f64, f64, u64);

/// `(soma ms, pico ms, n)` de cada metade, ZERANDO a janela — o leitor é o `[frame]` do shell.
#[must_use]
pub fn take_window() -> (Half, Half, Half) {
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
