//! **A silhueta do carimbo, guardada** (W3b): o depósito e o pente do relevo dos fios leem a MESMA silhueta nas
//! mesmas células, e cada leitura são as sub-amostras do AA — recalculá-la no pente era o custo inteiro do relevo
//! (doc 50_medicao §4: o move a 103 px 1,07 → 2,37 ms). A silhueta é pura: guardá-la dá o traço ao byte.

use std::sync::atomic::{AtomicU64, Ordering};

/// A célula ainda por calcular (um NaN que a silhueta nunca devolve).
const VAZIA: u64 = u64::MAX;

/// A silhueta de UM dab na caixa `[x0, x0 + w) × [y0, y0 + h)` (células), calculada na 1.ª leitura de cada célula.
/// Atómica: o depósito lê-a por linhas em paralelo; duas threads que calculem a mesma célula guardam o MESMO valor.
pub(super) struct SilMemo {
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    v: Vec<AtomicU64>,
}

impl SilMemo {
    /// A caixa do pente (`Engine::pentear_relevo`): a meia-extensão `meia` (células) à volta de `(x, y)`.
    pub(super) fn nova(x: f64, y: f64, meia: [f64; 2]) -> Self {
        let (ax, ay) = (meia[0].ceil() as i32 + 1, meia[1].ceil() as i32 + 1);
        let (cx, cy) = (x.round() as i32, y.round() as i32);
        let (w, h) = (2 * ax + 1, 2 * ay + 1);
        Self {
            x0: cx - ax,
            y0: cy - ay,
            w,
            h,
            v: (0..w * h).map(|_| AtomicU64::new(VAZIA)).collect(),
        }
    }

    /// A silhueta em `(cx, cy)`: a guardada, ou `sil` (e guarda-a); fora da caixa, `sil` sem guardar.
    pub(super) fn ou(&self, cx: i32, cy: i32, sil: impl Fn(i32, i32) -> f64) -> f64 {
        let (i, j) = (cx - self.x0, cy - self.y0);
        if i < 0 || j < 0 || i >= self.w || j >= self.h {
            return sil(cx, cy);
        }
        let c = &self.v[(j * self.w + i) as usize];
        match c.load(Ordering::Relaxed) {
            VAZIA => {
                let v = sil(cx, cy);
                c.store(v.to_bits(), Ordering::Relaxed);
                v
            }
            b => f64::from_bits(b),
        }
    }
}
