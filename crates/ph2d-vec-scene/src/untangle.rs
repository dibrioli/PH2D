//! **O contorno que não se cruza** (F12 A2): quando um parâmetro leva a forma a cruzar-se (a
//! estrela lisa de muitas pontas, a torção extrema, o flanco côncavo da engrenagem de muitos
//! dentes), ela pára no último desenho sem cruzamento a caminho do neutro — o idioma do *Curve*,
//! que pára na cúspide. Doc: `docs/UI_New_and_Simple/vetor_nova_ui/19_o_traco_das_formas.md` §5.

use crate::VecPath;
use crate::geometry::cubic_at;

/// Amostras por troço; a janela (em troços) onde se procura o cruzamento; passos da bissecção; e
/// o recuo depois do limiar (a bissecção pousa ONDE o cruzamento nasce).
const SAMPLES: usize = 16;
const WINDOW: usize = 3;
const STEPS: usize = 10;
const MARGIN: f64 = 0.97;

/// `build(1)` é a forma pedida e `build(0)` a neutra (simples por construção). Devolve a pedida
/// se ela não se cruza; senão a de `MARGIN ×` o maior `t` sem cruzamento. `period` = quantas
/// pontas/dentes repetem o contorno (simetria de rotação antes do afim, que preserva cruzamentos):
/// basta procurar no 1.º período contra os vizinhos — o custo da estrela de 60 pontas cai 60×.
pub(crate) fn untangled(period: usize, build: impl Fn(f64) -> VecPath) -> VecPath {
    let crosses = |p: &VecPath| crosses(p, period);
    let full = build(1.0);
    if !crosses(&full) {
        return full;
    }
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..STEPS {
        let mid = 0.5 * (lo + hi);
        if crosses(&build(mid)) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    build(lo * MARGIN)
}

struct Flat {
    pts: Vec<[f64; 2]>,
    lo: [f64; 2],
    hi: [f64; 2],
}

/// O contorno principal (de `period` repetições) cruza-se — dentro de um troço, ou entre troços a
/// menos de [`WINDOW`]? Encostar sem atravessar (a cúspide, a ida-e-volta) não conta.
pub(crate) fn crosses(p: &VecPath, period: usize) -> bool {
    let n = p.verts.len();
    if n < 2 {
        return false;
    }
    let segs = if p.closed { n } else { n - 1 };
    // Com simetria, o 1.º período (e a janela à frente dele) chega.
    let first = if p.closed && period > 1 && segs % period == 0 {
        segs / period
    } else {
        segs
    };
    // Só os troços que a procura lê: o 1.º período e a janela à frente dele.
    let read = (first + WINDOW).min(segs);
    let flat: Vec<Flat> = (0..read)
        .map(|k| {
            let (a, b) = (&p.verts[k], &p.verts[(k + 1) % n]);
            let pts: Vec<[f64; 2]> = (0..=SAMPLES)
                .map(|i| {
                    let t = i as f64 / SAMPLES as f64;
                    cubic_at(a.anchor, a.out_handle, b.in_handle, b.anchor, t)
                })
                .collect();
            let lo = pts
                .iter()
                .fold([f64::INFINITY; 2], |m, q| [m[0].min(q[0]), m[1].min(q[1])]);
            let hi = pts.iter().fold([f64::NEG_INFINITY; 2], |m, q| {
                [m[0].max(q[0]), m[1].max(q[1])]
            });
            Flat { pts, lo, hi }
        })
        .collect();
    // A quase-cúspide dentro de um troço (a curva volta mais de 120° entre duas amostras): é onde
    // o laço nasce, e a bissecção pousaria nela.
    let reverses = flat.iter().take(first).any(|f| {
        f.pts.windows(3).any(|w| {
            let (u, v) = (
                [w[1][0] - w[0][0], w[1][1] - w[0][1]],
                [w[2][0] - w[1][0], w[2][1] - w[1][1]],
            );
            let (lu, lv) = (u[0].hypot(u[1]), v[0].hypot(v[1]));
            lu > 1e-12 && lv > 1e-12 && u[0] * v[0] + u[1] * v[1] < -0.5 * lu * lv
        })
    });
    if reverses {
        return true;
    }
    for k in 0..first {
        for d in 0..=WINDOW.min(segs - 1) {
            if !p.closed && k + d >= segs {
                break;
            }
            let l = (k + d) % segs;
            if l >= read {
                continue;
            }
            let (f, g) = (&flat[k], &flat[l]);
            if d > 0
                && (f.hi[0] < g.lo[0]
                    || g.hi[0] < f.lo[0]
                    || f.hi[1] < g.lo[1]
                    || g.hi[1] < f.lo[1])
            {
                continue;
            }
            for i in 0..SAMPLES {
                for j in 0..SAMPLES {
                    let touching = (d == 0 && j < i + 2)
                        || (d == 1 && i + 1 == SAMPLES && j == 0)
                        || (p.closed && d + 1 == segs && i == 0 && j + 1 == SAMPLES);
                    if !touching && cross(f.pts[i], f.pts[i + 1], g.pts[j], g.pts[j + 1]) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let o = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
    };
    let (d1, d2, d3, d4) = (o(a, b, c), o(a, b, d), o(c, d, a), o(c, d, b));
    d1 * d2 < -1e-18 && d3 * d4 < -1e-18
}
