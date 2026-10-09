//! A geometria pura do QuickShape (irmão de `quickshape.rs`, pelo tecto de 700 LOC): pontos,
//! rectas ajustadas, reamostragem, RDP. Nada aqui sabe o que é uma forma.

use std::f64::consts::{PI, TAU};

pub(crate) type P2 = [f64; 2];

/// Uma recta ajustada: um ponto dela e a direcção unitária.
#[derive(Clone, Copy)]
pub(crate) struct Fit {
    pub(crate) c: P2,
    pub(crate) d: P2,
}

pub(crate) fn resample(p: &[P2], n: usize) -> Vec<P2> {
    let mut cum = vec![0.0];
    for w in p.windows(2) {
        cum.push(cum[cum.len() - 1] + dist(w[0], w[1]));
    }
    let total = cum[cum.len() - 1];
    let mut out = Vec::with_capacity(n);
    let mut j = 0;
    for i in 0..n {
        let t = total * i as f64 / (n - 1) as f64;
        while j + 2 < cum.len() && cum[j + 1] < t {
            j += 1;
        }
        let seg = cum[j + 1] - cum[j];
        let u = if seg > 0.0 {
            ((t - cum[j]) / seg).clamp(0.0, 1.0)
        } else {
            0.0
        };
        out.push(add(p[j], mul(sub(p[j + 1], p[j]), u)));
    }
    out
}

/// Ramer-Douglas-Peucker, devolvendo os ÍNDICES que ficam (iterativo, pilha explícita).
pub(crate) fn rdp(r: &[P2], tol: f64) -> Vec<usize> {
    let mut keep = vec![false; r.len()];
    keep[0] = true;
    keep[r.len() - 1] = true;
    let mut stack = vec![(0, r.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        let worst = ((a + 1)..b)
            .map(|i| (seg_dist(r[i], r[a], r[b]), i))
            .fold((0.0, a), |m, x| if x.0 > m.0 { x } else { m });
        if worst.0 > tol {
            keep[worst.1] = true;
            stack.push((a, worst.1));
            stack.push((worst.1, b));
        }
    }
    (0..r.len()).filter(|&i| keep[i]).collect()
}

/// `m x = v` por eliminação de Gauss com pivô parcial; `None` se `m` é singular.
pub(crate) fn solve<const N: usize>(mut m: [[f64; N]; N], mut v: [f64; N]) -> Option<[f64; N]> {
    for c in 0..N {
        let p = (c..N).max_by(|&a, &b| m[a][c].abs().total_cmp(&m[b][c].abs()))?;
        if m[p][c].abs() <= f64::MIN_POSITIVE {
            return None;
        }
        m.swap(c, p);
        v.swap(c, p);
        for r in (c + 1)..N {
            let f = m[r][c] / m[c][c];
            for k in c..N {
                m[r][k] -= f * m[c][k];
            }
            v[r] -= f * v[c];
        }
    }
    let mut x = [0.0; N];
    for r in (0..N).rev() {
        let s: f64 = ((r + 1)..N).map(|k| m[r][k] * x[k]).sum();
        x[r] = (v[r] - s) / m[r][r];
    }
    Some(x)
}

/// Mínimos quadrados TOTAIS (a recta que minimiza a distância perpendicular).
pub(crate) fn tls(q: &[P2]) -> Option<Fit> {
    if q.len() < 2 {
        return None;
    }
    let c = centroid(q);
    let (sxx, syy, sxy) = q.iter().fold((0.0, 0.0, 0.0), |(a, b, s), &p| {
        let [x, y] = sub(p, c);
        (a + x * x, b + y * y, s + x * y)
    });
    (sxx + syy > 0.0).then(|| Fit {
        c,
        d: unit(0.5 * f64::atan2(2.0 * sxy, sxx - syy)),
    })
}

pub(crate) fn intersect(f: Fit, g: Fit) -> P2 {
    let den = cross(f.d, g.d);
    if den.abs() <= 1e-9 {
        return mul(add(f.c, g.c), 0.5);
    }
    add(f.c, mul(f.d, cross(sub(g.c, f.c), g.d) / den))
}

pub(crate) fn project(p: P2, f: Fit) -> P2 {
    add(f.c, mul(f.d, dot(sub(p, f.c), f.d)))
}

// ── geometria ───────────────────────────────────────────────────────────────────────────────────

/// O ângulo, arredondado ao múltiplo de `step` mais perto.
#[must_use]
pub fn snap(a: f64, step: f64) -> f64 {
    (a / step).round() * step
}

/// A viragem entre duas RECTAS (direcção sem sentido, `mod π`), em `[0, π/2]`.
pub(crate) fn turn(a: P2, b: P2) -> f64 {
    let t = wrap_pi(angle(b) - angle(a)).abs();
    t.min(PI - t)
}

pub(crate) fn wrap_pi(a: f64) -> f64 {
    a - TAU * ((a + PI) / TAU).floor()
}

pub(crate) fn centroid(q: &[P2]) -> P2 {
    let s = q.iter().fold([0.0, 0.0], |s, &p| add(s, p));
    mul(s, 1.0 / q.len().max(1) as f64)
}

pub(crate) fn seg_dist(p: P2, a: P2, b: P2) -> f64 {
    let v = sub(b, a);
    let vv = dot(v, v);
    let t = if vv > 0.0 {
        (dot(sub(p, a), v) / vv).clamp(0.0, 1.0)
    } else {
        0.0
    };
    dist(p, add(a, mul(v, t)))
}

pub(crate) fn rotate(p: P2, a: f64) -> P2 {
    let (s, c) = a.sin_cos();
    [p[0] * c - p[1] * s, p[0] * s + p[1] * c]
}
pub(crate) fn unit(a: f64) -> P2 {
    let (s, c) = a.sin_cos();
    [c, s]
}
pub(crate) fn angle(v: P2) -> f64 {
    v[1].atan2(v[0])
}
pub(crate) fn add(a: P2, b: P2) -> P2 {
    [a[0] + b[0], a[1] + b[1]]
}
pub(crate) fn sub(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}
pub(crate) fn mul(a: P2, s: f64) -> P2 {
    [a[0] * s, a[1] * s]
}
pub(crate) fn dot(a: P2, b: P2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
pub(crate) fn cross(a: P2, b: P2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
pub(crate) fn dist(a: P2, b: P2) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
