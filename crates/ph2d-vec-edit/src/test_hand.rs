//! **A mão sintética das sondas** (só `cfg(test)`): uma polilinha amostrada a passo de mão, com o
//! tremor do `gera_entradas.py` do oráculo do QuickShape. Partilhada pelas sondas do QuickShape e
//! das quinas do lápis — uma cópia em cada uma divergiria em silêncio.

type P2 = [f64; 2];

fn add(a: P2, b: P2) -> P2 {
    [a[0] + b[0], a[1] + b[1]]
}
fn sub(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}
fn mul(a: P2, s: f64) -> P2 {
    [a[0] * s, a[1] * s]
}
fn dist(a: P2, b: P2) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Gerador determinístico (LCG de Knuth) — a sonda repete-se bit a bit.
pub(crate) struct Lcg(pub(crate) u64);
impl Lcg {
    pub(crate) fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
}

/// Uma mão: a polilinha amostrada a `step` px com o tremor do `gera_entradas.py` (±`amp`, média
/// móvel de 5 — o ruído da mão é suave, não branco).
pub(crate) fn hand(poly: &[P2], step: f64, amp: f64, seed: u64) -> Vec<P2> {
    let mut pts = vec![poly[0]];
    for w in poly.windows(2) {
        let n = (dist(w[0], w[1]) / step).ceil().max(1.0) as usize;
        pts.extend((1..=n).map(|i| add(w[0], mul(sub(w[1], w[0]), i as f64 / n as f64))));
    }
    let mut g = Lcg(seed.wrapping_mul(2_654_435_761).wrapping_add(1));
    let noise: Vec<P2> = pts
        .iter()
        .map(|_| [amp * g.next(), amp * g.next()])
        .collect();
    (0..pts.len())
        .map(|i| {
            let w = &noise[i.saturating_sub(2)..(i + 3).min(noise.len())];
            let m = mul(
                w.iter().fold([0.0, 0.0], |s, &q| add(s, q)),
                1.0 / w.len() as f64,
            );
            add(pts[i], m)
        })
        .collect()
}

/// O estabilizador do lápis (`lazy_mouse_step` do Painter, `STABILIZER_MIN_BLEND = 0,08`) sobre
/// as amostras, um evento por amostra: `strength` 0,5 é o valor por omissão do slider.
pub(crate) fn stabilise(pts: &[P2], strength: f64) -> Vec<P2> {
    let blend = 1.0 - strength * (1.0 - 0.08);
    let mut s = pts[0];
    pts.iter()
        .map(|&p| {
            s = add(s, mul(sub(p, s), blend));
            s
        })
        .collect()
}
