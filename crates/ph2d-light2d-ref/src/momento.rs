//! P14 «L» — o primeiro momento angular `M = ∫L(θ)(cos θ, sin θ) dθ` e o receptor com normal
//! (núcleo cardioide, `docs/PixelArt/P14_plano.md` §4).

use std::f64::consts::TAU;

use crate::geometria::cruzamentos;
use crate::{Objeto, entrada, integra, queda};

/// `∫ₐᵇ (cos θ, sin θ) dθ`, sem cancelamento em intervalos estreitos.
pub(crate) fn momento_do_arco(a: f64, b: f64) -> [f64; 2] {
    let (m, h) = (0.5 * (a + b), 0.5 * (b - a));
    let s = 2.0 * h.sin();
    [s * m.cos(), s * m.sin()]
}

/// O momento do peso de [`peso_do_emissor`]: `∫ₐᵇ queda·(cos θ, sin θ) dθ`.
pub(crate) fn momento_do_emissor(obj: &Objeto, ponto: [f64; 2], a: f64, b: f64) -> [f64; 2] {
    let (_, alcance) = obj.emissao.expect("um emissor");
    if alcance == f64::INFINITY {
        return momento_do_arco(a, b);
    }
    let mut c = Vec::new();
    let mut q = |th: f64| {
        cruzamentos(&obj.fronteira, ponto, [th.cos(), th.sin()], &mut c);
        queda(entrada(&c).unwrap_or(0.0), alcance)
    };
    let mx = integra::gauss_kronrod(a, b, &mut |th| q(th) * th.cos());
    let my = integra::gauss_kronrod(a, b, &mut |th| q(th) * th.sin());
    [mx, my]
}

pub(crate) fn somar_momento(m: &mut [[f64; 3]; 2], l: [f64; 3], w: f64, dir: [f64; 2]) {
    for e in 0..2 {
        for ch in 0..3 {
            m[e][ch] += l[ch] * w * dir[e];
        }
    }
}

/// `[F; Mx; My]` num só vetor (as 9 componentes que o miolo difunde juntas).
pub(crate) fn junta(f: [f64; 3], m: [[f64; 3]; 2]) -> [f64; 9] {
    std::array::from_fn(|i| if i < 3 { f[i] } else { m[i / 3 - 1][i % 3] })
}

pub(crate) fn separa(x: [f64; 9]) -> ([f64; 3], [[f64; 3]; 2]) {
    ([x[0], x[1], x[2]], [[x[3], x[4], x[5]], [x[6], x[7], x[8]]])
}

/// `(F + n_xy·M) / 2π` — o núcleo cardioide `k = (1 + n_xy·ω̂)/2π` (P14 §4 «L»).
pub(crate) fn cardioide(fm: [f64; 9], n: [f64; 2]) -> [f64; 3] {
    let (f, m) = separa(fm);
    std::array::from_fn(|c| (f[c] + n[0] * m[0][c] + n[1] * m[1][c]) / TAU)
}

pub(crate) fn de_frente(n: [f64; 2]) -> bool {
    n[0] == 0.0 && n[1] == 0.0
}
