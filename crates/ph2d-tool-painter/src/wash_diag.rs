//! **O split do quadro da AQUARELA, para o log do produto** (`PH2D_PAINT_PERF`).
//!
//! Irmão do [`crate::wet_diag`], pelo mesmo motivo e com a mesma forma: um `tool-tick` de 23 ms não
//! diz **de que** ele é feito, e as curas de *"o composite é caro"*, *"o pour caminha o traço
//! inteiro"* e *"a secagem varre a poça"* não se parecem em nada.
//!
//! ⚠️ **Por que isto existe mesmo com sondas de bancada.** As sondas medem o TOOL; o artista roda o
//! APP, que acrescenta shell, upload de preview e compositor. Este repo já pagou três vezes por
//! confiar num número de bancada onde a decisão era de produto (doc 28 §5.40, §5.42, §4.8.3) — e a
//! regra que ficou é que **quando o número vira decisão, ele sai da porta do produto**.
//!
//! ⚠️ **E a linha traz `ns/texel`, não só ms.** É ele que separa dois mundos com curas opostas:
//! *constante entre janelas* = o pincel/documento cresceu (TRABALHO, e o artista tem sliders para
//! isso) · *subindo* = contenção ou um passe novo caminhando mais do que declara. Mesmo raciocínio do
//! `ns/celula` da água (§5.47), que fechou um impasse de atribuição que nenhuma sonda tinha fechado.
//!
//! Contadores atômicos relaxados, como no irmão: o custo é uma soma por fase contra fases de
//! milissegundos, e o caminho sem o log paga o mesmo (não há gate de env aqui — o shell decide se
//! IMPRIME, não se MEDE; um contador que só existe com a env ligada mente sobre a corrida que o
//! artista de fato rodou).

use std::sync::atomic::{AtomicU64, Ordering};

/// Uma fase: soma, pico e contagem, em microssegundos.
struct Phase {
    us: AtomicU64,
    max_us: AtomicU64,
    n: AtomicU64,
}

impl Phase {
    const fn new() -> Self {
        Self {
            us: AtomicU64::new(0),
            max_us: AtomicU64::new(0),
            n: AtomicU64::new(0),
        }
    }
    fn note(&self, ms: f32) {
        let us = (f64::from(ms) * 1000.0) as u64;
        self.us.fetch_add(us, Ordering::Relaxed);
        self.max_us.fetch_max(us, Ordering::Relaxed);
        self.n.fetch_add(1, Ordering::Relaxed);
    }
    /// `(media ms, pico ms, n)` — e ZERA, porque a linha do shell descreve uma JANELA.
    fn take(&self) -> (f64, f64, u64) {
        let us = self.us.swap(0, Ordering::Relaxed);
        let mx = self.max_us.swap(0, Ordering::Relaxed);
        let n = self.n.swap(0, Ordering::Relaxed);
        let avg = if n > 0 {
            us as f64 / n as f64 / 1000.0
        } else {
            0.0
        };
        (avg, mx as f64 / 1000.0, n)
    }
}

static COMPOSITE: Phase = Phase::new();
static STAMP: Phase = Phase::new();
static POUR: Phase = Phase::new();
static DRY: Phase = Phase::new();
static PENDOWN: Phase = Phase::new();
/// Texels da janela de leitura somados sobre os composites da janela — o divisor do `ns/texel`.
static WINDOW_PX: AtomicU64 = AtomicU64::new(0);

/// Um `apply_watercolor` custou isto, sobre uma janela de leitura de `window_px` texels.
pub fn note_composite(ms: f32, window_px: u64) {
    COMPOSITE.note(ms);
    WINDOW_PX.fetch_add(window_px, Ordering::Relaxed);
}

/// O carimbo de aquarela de um evento de ponteiro (cobertura + cor + smear) custou isto.
pub fn note_stamp(ms: f32) {
    STAMP.note(ms);
}

/// Um `pour_canvas_wet` custou isto — ele caminha a união CUMULATIVA do traço, então é a fase cujo
/// custo cresce com o comprimento do gesto.
pub fn note_pour(ms: f32) {
    POUR.note(ms);
}

/// Um `dry_canvas_wet` custou isto.
pub fn note_dry(ms: f32) {
    DRY.note(ms);
}

/// Um `freeze_watercolor_ground` custou isto — o pen-down, a única fase do módulo que ainda
/// responde ao tamanho do DOCUMENTO (três varreduras de plano inteiro).
pub fn note_pendown(ms: f32) {
    PENDOWN.note(ms);
}

/// Uma fase pronta para impressão.
pub struct PhaseRead {
    pub avg_ms: f64,
    pub max_ms: f64,
    pub n: u64,
}

/// O que a janela mediu — e **zera os contadores**, porque a linha descreve uma janela e não uma
/// história.
pub struct WashRead {
    pub composite: PhaseRead,
    pub stamp: PhaseRead,
    pub pour: PhaseRead,
    pub dry: PhaseRead,
    pub pendown: PhaseRead,
    /// Texels de janela por composite — `0` quando não houve composite nenhum.
    pub window_px_per_composite: f64,
    /// `ns` por texel de janela: o número que separa TRABALHO de CONTENÇÃO entre duas janelas.
    pub ns_per_texel: f64,
}

/// Lê e zera. ⚠️ **Um só leitor**: um segundo zeraria a janela do primeiro, e os dois publicariam
/// pedaços do mesmo quadro como se fossem quadros.
pub fn take() -> WashRead {
    let rd = |(avg_ms, max_ms, n)| PhaseRead { avg_ms, max_ms, n };
    let composite = COMPOSITE.take();
    let px = WINDOW_PX.swap(0, Ordering::Relaxed);
    let per = if composite.2 > 0 {
        px as f64 / composite.2 as f64
    } else {
        0.0
    };
    WashRead {
        window_px_per_composite: per,
        ns_per_texel: if per > 0.0 {
            composite.0 * 1e6 / per
        } else {
            0.0
        },
        composite: rd(composite),
        stamp: rd(STAMP.take()),
        pour: rd(POUR.take()),
        dry: rd(DRY.take()),
        pendown: rd(PENDOWN.take()),
    }
}
