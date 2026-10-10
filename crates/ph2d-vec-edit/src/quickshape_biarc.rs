//! **O BIARCO do QuickShape** (F7c, o smoke do dono 10/10: *«arcos assimétricos como na foto não
//! funcionam»*) — dois arcos de círculo TANGENTES na junção, a curva clássica da maquinação CNC
//! (K. M. Bolton, CAD 7, 1975). O gancho do dono abre largo (raio ~217 px) e fecha apertado (~39 px):
//! nem uma elipse (22 px fora) nem a espiral de Euler (34 px fora) o desenham; o biarco, a ~11 px no
//! pior ponto e 2,3 px de rms (`00_resultados.md` §3g). Irmão de `quickshape_arc.rs`.
//!
//! O ajuste: Levenberg-Marquardt (Marquardt, SIAM J. 11, 1963) sobre `(x₀, y₀, θ₀, k₁, k₂, L₁, L₂)`,
//! com a correspondência por COMPRIMENTO DE ARCO (o traço vem reamostrado por comprimento) e o
//! jacobiano por diferenças finitas; parte do círculo de Taubin.

use crate::quickshape_arc::ArcShape;
use crate::quickshape_geom::{P2, add, dot, mul, solve, sub, unit};
use std::f64::consts::FRAC_PI_2;

/// `(x₀, y₀, θ₀, k₁, k₂, L₁, L₂)`.
type Params = [f64; 7];

/// O ponto a `s` ao longo de um arco de curvatura `k` que sai de `p` com a direcção `th`.
pub(crate) fn advance(p: P2, th: f64, k: f64, s: f64) -> P2 {
    if (k * s).abs() < 1e-9 {
        return add(p, mul(unit(th), s));
    }
    add(
        p,
        [
            ((th + k * s).sin() - th.sin()) / k,
            (th.cos() - (th + k * s).cos()) / k,
        ],
    )
}

/// Os `n` pontos do biarco igualmente espaçados em comprimento.
fn points(q: &Params, n: usize) -> Vec<P2> {
    let p0 = [q[0], q[1]];
    let (th1, l1, l2) = (q[2] + q[3] * q[5], q[5], q[6]);
    let p1 = advance(p0, q[2], q[3], l1);
    (0..n)
        .map(|i| {
            let s = (l1 + l2) * i as f64 / (n - 1) as f64;
            if s <= l1 {
                advance(p0, q[2], q[3], s)
            } else {
                advance(p1, th1, q[4], s - l1)
            }
        })
        .collect()
}

fn cost(q: &Params, r: &[P2]) -> f64 {
    points(q, r.len())
        .iter()
        .zip(r)
        .map(|(a, b)| dot(sub(*a, *b), sub(*a, *b)))
        .sum()
}

/// O biarco que desenha `r`, a partir do círculo `(k, θ₀)`; com o rms dele (px). `None` se não
/// converge para dois troços de comprimento positivo e curvatura do MESMO sinal (sem inflexão).
pub(crate) fn fit(r: &[P2], k: f64, th0: f64) -> Option<([ArcShape; 2], f64)> {
    let len: f64 = r
        .windows(2)
        .map(|w| crate::quickshape_geom::dist(w[0], w[1]))
        .sum();
    let mut q: Params = [r[0][0], r[0][1], th0, k, k, len / 2.0, len / 2.0];
    let (mut e, mut lambda) = (cost(&q, r), 1e-3);
    let n = r.len();
    for _ in 0..60 {
        let base = points(&q, n);
        let mut jac = vec![[0.0; 7]; 2 * n];
        for j in 0..7 {
            let h = 1e-6 * q[j].abs().max(1e-3);
            let mut qh = q;
            qh[j] += h;
            for (i, p) in points(&qh, n).iter().enumerate() {
                jac[2 * i][j] = (p[0] - base[i][0]) / h;
                jac[2 * i + 1][j] = (p[1] - base[i][1]) / h;
            }
        }
        let (mut a, mut g) = ([[0.0; 7]; 7], [0.0; 7]);
        for (i, b) in base.iter().enumerate() {
            let res = sub(*b, r[i]);
            for (row, rr) in [(2 * i, res[0]), (2 * i + 1, res[1])] {
                for u in 0..7 {
                    g[u] -= jac[row][u] * rr;
                    for v in 0..7 {
                        a[u][v] += jac[row][u] * jac[row][v];
                    }
                }
            }
        }
        for (u, row) in a.iter_mut().enumerate() {
            row[u] *= 1.0 + lambda;
        }
        let Some(d) = solve(a, g) else { break };
        let mut q2 = q;
        for u in 0..7 {
            q2[u] += d[u];
        }
        let e2 = if q2[5] > 0.0 && q2[6] > 0.0 {
            cost(&q2, r)
        } else {
            f64::MAX
        };
        if e2 < e {
            let done = e - e2 <= 1e-12 * e;
            (q, e, lambda) = (q2, e2, lambda * 0.3);
            if done {
                break;
            }
        } else {
            lambda *= 10.0;
        }
    }
    if q[3] * q[4] <= 0.0 || q[5] <= 0.0 || q[6] <= 0.0 {
        return None;
    }
    let first = piece([q[0], q[1]], q[2], q[3], q[5]);
    let second = piece(
        advance([q[0], q[1]], q[2], q[3], q[5]),
        q[2] + q[3] * q[5],
        q[4],
        q[6],
    );
    Some(([first, second], (e / n as f64).sqrt()))
}

/// O arco de círculo que sai de `p` com a direcção `th`, curvatura `k` (≠ 0), comprimento `l`.
fn piece(p: P2, th: f64, k: f64, l: f64) -> ArcShape {
    let radius = 1.0 / k.abs();
    let centre = add(p, mul(unit(th + FRAC_PI_2), 1.0 / k));
    ArcShape {
        centre,
        rx: radius,
        ry: radius,
        rot: 0.0,
        start: crate::quickshape_geom::angle(sub(p, centre)),
        sweep: k * l,
    }
}
