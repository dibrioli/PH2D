//! **A mão dos gates do QuickShape do quadro** (só testes; smoke do dono 20/10, o «D»): o contorno
//! ideal com os cantos arredondados, ~150 amostras, tremor suave ±2–3 px, o fim a voltar perto do
//! início — e a caneta POUSADA no fim, com tremor abaixo do passo do lápis. Uma fonte só: a
//! `ph2d-board-render` inclui ESTE ficheiro (`sketch_tests.rs`, a régua do pintor).

type P2 = [f64; 2];

/// Gerador determinístico (LCG de Knuth): os gates repetem-se bit a bit.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
}

fn length(p: &[P2]) -> f64 {
    p.windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .sum()
}

/// O ponto à distância `s` ao longo de `p`.
fn point_at(p: &[P2], s: f64) -> P2 {
    let mut acc = 0.0;
    for w in p.windows(2) {
        let l = (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
        if acc + l >= s && l > 0.0 {
            let u = (s - acc) / l;
            return [
                w[0][0] + (w[1][0] - w[0][0]) * u,
                w[0][1] + (w[1][1] - w[0][1]) * u,
            ];
        }
        acc += l;
    }
    p[p.len() - 1]
}

/// O polígono fechado `c` (sem repetir o 1.º) com os cantos arredondados de raio `r`: a volta
/// inteira, a começar logo depois do canto 0.
fn rounded(c: &[P2], r: f64) -> Vec<P2> {
    let n = c.len();
    let toward = |a: P2, b: P2| {
        let l = (b[0] - a[0]).hypot(b[1] - a[1]);
        [a[0] + (b[0] - a[0]) * r / l, a[1] + (b[1] - a[1]) * r / l]
    };
    let mut out = Vec::new();
    for k in 0..=n {
        let i = k % n;
        let (prev, cur, next) = (c[(i + n - 1) % n], c[i], c[(i + 1) % n]);
        let (a, b) = (toward(cur, prev), toward(cur, next));
        let first = if k == 0 { 8 } else { 0 };
        for j in first..=8 {
            let t = f64::from(j) / 8.0;
            let q = |u: usize| {
                (1.0 - t) * (1.0 - t) * a[u] + 2.0 * (1.0 - t) * t * cur[u] + t * t * b[u]
            };
            out.push([q(0), q(1)]);
        }
    }
    out
}

/// A mão sobre `ideal`: `n` amostras até `extra` além do fim (negativo = pára antes), tremor ±`amp`
/// em média móvel de 5 (o ruído da mão é suave) e uma deriva lenta de 2 px.
fn hand(ideal: &[P2], n: usize, extra: f64, amp: f64, seed: u64) -> Vec<P2> {
    let full = length(ideal);
    let total = full + extra;
    let mut g = Lcg(seed.wrapping_mul(2_654_435_761).wrapping_add(1));
    let noise: Vec<P2> = (0..n).map(|_| [amp * g.next(), amp * g.next()]).collect();
    (0..n)
        .map(|i| {
            let s = total * i as f64 / (n - 1) as f64;
            let base = point_at(ideal, if s > full { s - full } else { s });
            let w = &noise[i.saturating_sub(2)..(i + 3).min(n)];
            let m = w.iter().fold([0.0, 0.0], |s, q| [s[0] + q[0], s[1] + q[1]]);
            let d = i as f64 * 7.0;
            [
                base[0] + m[0] / w.len() as f64 + 2.0 * (d / 90.0).sin(),
                base[1] + m[1] / w.len() as f64 + 2.0 * (d / 70.0).cos(),
            ]
        })
        .collect()
}

/// A caneta POUSADA em `end`: `k` eventos de ±`amp`.
pub(crate) fn resting(end: P2, k: usize, amp: f64, seed: u64) -> Vec<P2> {
    let mut g = Lcg(seed ^ 0x5eed);
    (0..k)
        .map(|_| [end[0] + amp * g.next(), end[1] + amp * g.next()])
        .collect()
}

/// O rectângulo 300×200 de mão que pára 10 px antes do início.
pub(crate) fn hand_rect() -> Vec<P2> {
    let c = [
        [100.0, 100.0],
        [400.0, 100.0],
        [400.0, 300.0],
        [100.0, 300.0],
    ];
    hand(&rounded(&c, 12.0), 150, -10.0, 5.0, 1)
}

/// O triângulo de mão (nenhum objecto do quadro o desenha igual).
pub(crate) fn hand_triangle() -> Vec<P2> {
    let c = [[100.0, 400.0], [250.0, 160.0], [420.0, 400.0]];
    hand(&rounded(&c, 8.0), 150, -10.0, 5.0, 4)
}

/// A elipse 300×200 de mão.
pub(crate) fn hand_ellipse() -> Vec<P2> {
    let ell: Vec<P2> = (0..=200)
        .map(|i| {
            let a = -std::f64::consts::FRAC_PI_2 + std::f64::consts::TAU * f64::from(i) / 200.0;
            [250.0 + 150.0 * a.cos(), 200.0 + 100.0 * a.sin()]
        })
        .collect();
    hand(&ell, 150, -15.0, 5.0, 3)
}

/// Os traços de mão dos gates: (nome, amostras).
pub(crate) fn hand_strokes() -> Vec<(&'static str, Vec<P2>)> {
    let c = [
        [100.0, 100.0],
        [400.0, 100.0],
        [400.0, 300.0],
        [100.0, 300.0],
    ];
    vec![
        ("rect que pára antes", hand_rect()),
        (
            "rect que passa do início",
            hand(&rounded(&c, 12.0), 150, 20.0, 5.0, 2),
        ),
        ("elipse", hand_ellipse()),
        ("triângulo", hand_triangle()),
    ]
}
