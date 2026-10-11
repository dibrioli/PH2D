//! A geometria dos PONTOS das alças — módulo irmão de [`super`] pelo tecto de 700 LOC (HR-18):
//! onde o contorno cruza o eixo de uma quina (`Spot::Apex`), a caixa do contorno (`Spot::Box`) e
//! o ponto de uma cúbica; e onde o ⊙ de canto se desenha (o centro do arco).

use ph2d_vec_scene::{ShapeValues, VecPath};

/// A caixa do contorno (as cúbicas amostradas): `(mínimo, máximo)`.
pub(super) fn curve_bbox(p: &VecPath) -> Option<([f64; 2], [f64; 2])> {
    let n = p.verts.len();
    let segs = if p.closed { n } else { n.saturating_sub(1) };
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for s in 0..segs.max(usize::from(n == 1)) {
        let (a, b) = (&p.verts[s], &p.verts[(s + 1) % n]);
        let c = [a.anchor, a.out_handle, b.in_handle, b.anchor];
        for k in 0..=16 {
            let q = cubic_at(&c, f64::from(k) / 16.0);
            lo = [lo[0].min(q[0]), lo[1].min(q[1])];
            hi = [hi[0].max(q[0]), hi[1].max(q[1])];
        }
    }
    (lo[0] <= hi[0]).then_some((lo, hi))
}

/// Onde o contorno `p` cruza o raio do centro (a origem) a `q`, do lado de `q` — o cruzamento mais
/// perto de `q` em distância ao centro (a estrela é estrelada em relação ao centro: um só).
pub(super) fn spoke_hit(p: &VecPath, q: [f64; 2]) -> Option<[f64; 2]> {
    const N: usize = 64;
    let r = q[0].hypot(q[1]);
    if r <= 0.0 {
        return None;
    }
    let u = [q[0] / r, q[1] / r];
    let cross = |b: [f64; 2]| u[0] * b[1] - u[1] * b[0];
    let n = p.verts.len();
    let segs = if p.closed { n } else { n.saturating_sub(1) };
    let mut best: Option<(f64, [f64; 2])> = None;
    let mut keep = |b: [f64; 2]| {
        let along = u[0] * b[0] + u[1] * b[1];
        if along > 0.0 && best.is_none_or(|(e, _)| (along - r).abs() < e) {
            best = Some(((along - r).abs(), b));
        }
    };
    for s in 0..segs {
        let (a, b) = (&p.verts[s], &p.verts[(s + 1) % n]);
        let c = [a.anchor, a.out_handle, b.in_handle, b.anchor];
        let mut t0 = 0.0;
        let mut f0 = cross(c[0]);
        if f0 == 0.0 {
            keep(c[0]);
        }
        for k in 1..=N {
            let t1 = k as f64 / N as f64;
            let f1 = cross(cubic_at(&c, t1));
            if f0 * f1 < 0.0 {
                let (mut lo, mut hi, mut flo) = (t0, t1, f0);
                for _ in 0..60 {
                    let m = 0.5 * (lo + hi);
                    let fm = cross(cubic_at(&c, m));
                    if (fm < 0.0) == (flo < 0.0) {
                        lo = m;
                        flo = fm;
                    } else {
                        hi = m;
                    }
                }
                keep(cubic_at(&c, 0.5 * (lo + hi)));
            } else if f1 == 0.0 {
                keep(cubic_at(&c, t1));
            }
            (t0, f0) = (t1, f1);
        }
    }
    best.map(|(_, b)| b)
}

/// ⭐ (F11) O ⊙ de canto do Illustrator: no CENTRO do arco — a quina mais o início do arco em cada
/// uma das duas arestas dela (num canto recto é o centro exacto) —, cada perna pelo menos
/// `park_at` (até meia aresta): com raio zero o ⊙ fica na diagonal, fora da alça de escala.
pub(super) fn corner_centre(
    h: &super::HandleDesc,
    values: &ShapeValues,
    half: [f64; 2],
    park_at: f64,
    cook: &dyn Fn(&ShapeValues) -> VecPath,
) -> Option<[f64; 2]> {
    let super::Spot::Rounding { corner, next } = h.spot else {
        return None;
    };
    let sharp = cook(&super::zeroed(values, h.sharp));
    let n = sharp.verts.len();
    if n < 3 || corner >= n {
        return None;
    }
    let at = |i: usize| sharp.verts[i % n].anchor;
    let q = at(corner);
    let (ends, nexts) = ([at(corner + 1), at(corner + n - 1)], [next, !next]);
    let mut c = q;
    for (o, nx) in ends.into_iter().zip(nexts) {
        let arm = super::HandleDesc {
            spot: super::Spot::Rounding { corner, next: nx },
            ..*h
        };
        let a = super::handle_point(&arm, values, half, cook)?;
        let (ex, ey) = (o[0] - q[0], o[1] - q[1]);
        let l = ex.hypot(ey);
        if l <= 0.0 {
            return None;
        }
        let r = (a[0] - q[0]).hypot(a[1] - q[1]).max(park_at.min(l * 0.5));
        c = [c[0] + ex / l * r, c[1] + ey / l * r];
    }
    Some(c)
}

/// O ponto da cúbica `c` em `t`.
pub(super) fn cubic_at(c: &[[f64; 2]; 4], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
    [
        w.iter().zip(c).map(|(k, q)| k * q[0]).sum(),
        w.iter().zip(c).map(|(k, q)| k * q[1]).sum(),
    ]
}
