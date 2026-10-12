//! A geometria dos PONTOS das alças — módulo irmão de [`super`] pelo tecto de 700 LOC (HR-18):
//! onde o contorno cruza o eixo de uma quina (`Spot::Apex`), a caixa do contorno (`Spot::Rail`) e
//! o ponto de uma cúbica; onde o ⊙ de canto se desenha (o centro do arco); e a busca no trilho.

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

/// Amostras por campo antes do refinamento (o trilho inteiro, para o mínimo ser o GLOBAL).
const SAMPLES: usize = 96;
/// Rodadas de refinamento (a janela encolhe ×4 por rodada à volta do melhor).
const REFINE: usize = 14;

/// O valor do campo `i` em `[lo, hi]` com a menor métrica: varredura do trilho inteiro e
/// refinamento à volta do melhor. Empates (o trilho satura — um raio maior que meia aresta não
/// anda mais) ficam com o valor MAIS PERTO do atual: a alça não salta para o fim da faixa.
pub(super) fn best_on_track(
    v: &mut ShapeValues,
    i: usize,
    lo: f64,
    hi: f64,
    int: bool,
    metric: &dyn Fn(&ShapeValues) -> f64,
) -> f64 {
    let cur = v[i];
    let mut eval = |x: f64| {
        v[i] = x;
        metric(v)
    };
    // O empate é da MÉTRICA (um trilho saturado devolve a mesma distância), nunca da faixa: com a
    // mão 25 px fora de uma aresta a distância é quase plana no mínimo (`dx²/50`), e uma
    // tolerância à escala da faixa puxava o raio 0,005 para o valor antigo (medido no `rx`).
    let better = |(d, x): (f64, f64), (bd, bx): (f64, f64)| {
        if !bd.is_finite() {
            return d.is_finite();
        }
        let tie = 1e-12 * (1.0 + bd);
        d < bd - tie || ((d - bd).abs() <= tie && (x - cur).abs() < (bx - cur).abs())
    };
    let mut best = (f64::INFINITY, cur);
    if int {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "as faixas de contagem do catálogo são pequenas (≤ 60)"
        )]
        let (a, b) = (lo.ceil() as i64, hi.floor() as i64);
        for n in a..=b {
            #[expect(clippy::cast_precision_loss, reason = "n ≤ 60")]
            let x = n as f64;
            let c = (eval(x), x);
            if better(c, best) {
                best = c;
            }
        }
        v[i] = cur;
        return best.1;
    }
    for k in 0..=SAMPLES {
        #[expect(clippy::cast_precision_loss, reason = "k ≤ SAMPLES")]
        let x = lo + (hi - lo) * k as f64 / SAMPLES as f64;
        let c = (eval(x), x);
        if better(c, best) {
            best = c;
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "SAMPLES é pequeno")]
    let mut half = (hi - lo) / SAMPLES as f64;
    for _ in 0..REFINE {
        for k in -2..=2 {
            let x = (best.1 + half * f64::from(k) * 0.5).clamp(lo, hi);
            let c = (eval(x), x);
            if better(c, best) {
                best = c;
            }
        }
        half *= 0.25;
    }
    v[i] = cur;
    best.1
}
