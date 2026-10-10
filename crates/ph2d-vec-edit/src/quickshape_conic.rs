//! **O ajuste DIRECTO de uma elipse** (A. Fitzgibbon, M. Pilu, R. Fisher, IEEE PAMI 21, 1999), na
//! forma numericamente estável de R. Halíř & J. Flusser (WSCG 1998) — porte directo. Devolve sempre
//! uma elipse (a restrição `4ac − b² = 1` exclui a hipérbole), mesmo num arco curto: é o que o arco
//! ASSIMÉTRICO do QuickShape precisa (F7c, o smoke do dono 10/10). Geometria pura, irmão de
//! `quickshape_geom.rs`.

use crate::quickshape_geom::{P2, add, centroid, dot, mul, rotate, solve, sub};
use std::f64::consts::TAU;

/// Uma elipse: centro, semi-eixos e a rotação (radianos, do eixo x para `rx`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Conic {
    pub(crate) c: P2,
    pub(crate) rx: f64,
    pub(crate) ry: f64,
    pub(crate) rot: f64,
}

impl Conic {
    /// `p` no referencial da elipse (centro na origem, `rx` no eixo x).
    pub(crate) fn local(&self, p: P2) -> P2 {
        rotate(sub(p, self.c), -self.rot)
    }

    /// A distância de `p` à elipse, pela recta do centro (exacta no círculo).
    pub(crate) fn distance(&self, p: P2) -> f64 {
        let q = self.local(p);
        let rho = (q[0] / self.rx).hypot(q[1] / self.ry);
        dot(q, q).sqrt() * (1.0 - 1.0 / rho).abs()
    }

    /// O ângulo PARAMÉTRICO de `p` (`p = c + R(rot)·(rx cos t, ry sin t)`).
    pub(crate) fn param(&self, p: P2) -> f64 {
        let q = self.local(p);
        (q[1] / self.ry).atan2(q[0] / self.rx)
    }
}

pub(crate) fn fit_ellipse(p: &[P2]) -> Option<Conic> {
    // Centrado e em escala unitária (o condicionamento das somas de quarta ordem).
    let m = centroid(p);
    let s = (p.iter().map(|&q| dot(sub(q, m), sub(q, m))).sum::<f64>() / p.len() as f64).sqrt();
    if s <= f64::MIN_POSITIVE {
        return None;
    }
    let (mut s1, mut s2, mut s3) = ([[0.0; 3]; 3], [[0.0; 3]; 3], [[0.0; 3]; 3]);
    for &q in p {
        let [x, y] = mul(sub(q, m), 1.0 / s);
        let (d1, d2) = ([x * x, x * y, y * y], [x, y, 1.0]);
        for i in 0..3 {
            for j in 0..3 {
                s1[i][j] += d1[i] * d1[j];
                s2[i][j] += d1[i] * d2[j];
                s3[i][j] += d2[i] * d2[j];
            }
        }
    }
    // T = −S3⁻¹ S2ᵀ (a coluna j resolve S3 x = a linha j de S2); M = S1 + S2 T.
    let mut t = [[0.0; 3]; 3];
    for j in 0..3 {
        let col = solve(s3, s2[j])?;
        for i in 0..3 {
            t[i][j] = -col[i];
        }
    }
    let mut mm = s1;
    for i in 0..3 {
        for j in 0..3 {
            mm[i][j] += (0..3).map(|k| s2[i][k] * t[k][j]).sum::<f64>();
        }
    }
    // C1⁻¹ M, com C1 = [[0, 0, 2], [0, −1, 0], [2, 0, 0]].
    let m3 = [
        mm[2].map(|v| v / 2.0),
        mm[1].map(|v| -v),
        mm[0].map(|v| v / 2.0),
    ];
    let a1 = ellipse_eigenvector(&m3)?;
    let a2: [f64; 3] = std::array::from_fn(|i| (0..3).map(|k| t[i][k] * a1[k]).sum());
    let e = conic_to_ellipse([a1[0], a1[1], a1[2], a2[0], a2[1], a2[2]])?;
    Some(Conic {
        c: add(m, mul(e.c, s)),
        rx: e.rx * s,
        ry: e.ry * s,
        rot: e.rot,
    })
}

/// O vector próprio de `a` que é ELIPSE: o de maior `4 v₀ v₂ − v₁²` (normalizado). Se nenhum é
/// positivo, a cónica é hipérbole e [`conic_to_ellipse`] recusa-a.
fn ellipse_eigenvector(a: &[[f64; 3]; 3]) -> Option<[f64; 3]> {
    let tr = a[0][0] + a[1][1] + a[2][2];
    let c1 = a[0][0] * a[1][1] - a[0][1] * a[1][0] + a[0][0] * a[2][2] - a[0][2] * a[2][0]
        + a[1][1] * a[2][2]
        - a[1][2] * a[2][1];
    let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
    cubic_roots(-tr, c1, -det)
        .into_iter()
        .filter_map(|l| {
            let r: [[f64; 3]; 3] = std::array::from_fn(|i| {
                std::array::from_fn(|j| a[i][j] - if i == j { l } else { 0.0 })
            });
            // O núcleo de `a − λ I`: o maior produto vectorial de duas linhas.
            let v = [(0, 1), (0, 2), (1, 2)]
                .map(|(i, j)| cross3(r[i], r[j]))
                .into_iter()
                .max_by(|u, w| dot3(*u, *u).total_cmp(&dot3(*w, *w)))?;
            let n = dot3(v, v);
            let cond = (4.0 * v[0] * v[2] - v[1] * v[1]) / n;
            (n > f64::MIN_POSITIVE).then_some((cond, v))
        })
        .max_by(|x, y| x.0.total_cmp(&y.0))
        .map(|(_, v)| v)
}

/// As raízes REAIS de `λ³ + a λ² + b λ + c` (Cardano / a forma trigonométrica).
fn cubic_roots(a: f64, b: f64, c: f64) -> Vec<f64> {
    let p = b - a * a / 3.0;
    let q = 2.0 * a * a * a / 27.0 - a * b / 3.0 + c;
    let disc = (q / 2.0).powi(2) + (p / 3.0).powi(3);
    let shift = -a / 3.0;
    if disc > 0.0 {
        let s = disc.sqrt();
        vec![(-q / 2.0 + s).cbrt() + (-q / 2.0 - s).cbrt() + shift]
    } else if p.abs() <= f64::MIN_POSITIVE {
        vec![(-q).cbrt() + shift]
    } else {
        let r = 2.0 * (-p / 3.0).sqrt();
        let phi = ((3.0 * q / (2.0 * p)) * (-3.0 / p).sqrt())
            .clamp(-1.0, 1.0)
            .acos()
            / 3.0;
        (0..3)
            .map(|k| r * (phi - TAU * f64::from(k) / 3.0).cos() + shift)
            .collect()
    }
}

/// `A x² + B xy + C y² + D x + E y + F = 0` → centro, semi-eixos, rotação.
fn conic_to_ellipse([a, b, c, d, e, f]: [f64; 6]) -> Option<Conic> {
    let den = b * b - 4.0 * a * c;
    if den >= 0.0 {
        return None;
    }
    let centre = [(2.0 * c * d - b * e) / den, (2.0 * a * e - b * d) / den];
    let rot = 0.5 * b.atan2(a - c);
    let (sn, cs) = rot.sin_cos();
    let ap = a * cs * cs + b * cs * sn + c * sn * sn;
    let cp = a * sn * sn - b * cs * sn + c * cs * cs;
    let [x, y] = centre;
    let f0 = a * x * x + b * x * y + c * y * y + d * x + e * y + f;
    let (rx2, ry2) = (-f0 / ap, -f0 / cp);
    (rx2 > 0.0 && ry2 > 0.0 && rx2.is_finite() && ry2.is_finite()).then(|| Conic {
        c: centre,
        rx: rx2.sqrt(),
        ry: ry2.sqrt(),
        rot,
    })
}

fn cross3(u: [f64; 3], v: [f64; 3]) -> [f64; 3] {
    [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ]
}

fn dot3(u: [f64; 3], v: [f64; 3]) -> f64 {
    u[0] * v[0] + u[1] * v[1] + u[2] * v[2]
}
