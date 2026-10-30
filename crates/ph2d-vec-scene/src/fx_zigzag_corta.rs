//! **O nó do Zig Zag é cortado** (F40, lei C do dono 30/10 — `docs/UI_New_and_Simple/F33_plano.md`
//! §6.3): num vale apertado a crista atravessa o outro lado e o contorno faz um laço. O laço sai pelo
//! ponto do cruzamento (as duas cúbicas partem-se ali e fica uma quina) e a crista fica. Só o nó
//! PEQUENO — até [`FRACAO`] do contorno: um cruzamento maior é a forma inteira a dobrar.

use crate::VecVertex;
use crate::geometry::cubic_at;

const PASSOS: usize = 16;
const FRACAO: f64 = 0.25;

type Cubica = [[f64; 2]; 4];

fn troco(v: &[VecVertex], k: usize) -> Cubica {
    let (a, b) = (&v[k], &v[(k + 1) % v.len()]);
    [a.anchor, a.out_handle, b.in_handle, b.anchor]
}

/// De Casteljau: a cúbica partida em `t`.
fn parte(c: Cubica, t: f64) -> (Cubica, Cubica) {
    let l = |p: [f64; 2], q: [f64; 2]| {
        [
            (q[0] - p[0]).mul_add(t, p[0]),
            (q[1] - p[1]).mul_add(t, p[1]),
        ]
    };
    let (p01, p12, p23) = (l(c[0], c[1]), l(c[1], c[2]), l(c[2], c[3]));
    let (p012, p123) = (l(p01, p12), l(p12, p23));
    let x = l(p012, p123);
    ([c[0], p01, p012, x], [x, p123, p23, c[3]])
}

fn derivada(c: Cubica, t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (w0, w1, w2) = (3.0 * u * u, 6.0 * u * t, 3.0 * t * t);
    [0, 1].map(|e| w0 * (c[1][e] - c[0][e]) + w1 * (c[2][e] - c[1][e]) + w2 * (c[3][e] - c[2][e]))
}

/// Newton sobre `A(t) = B(u)`: a poligonal dá o cruzamento a menos de um passo, as cúbicas dão-no
/// exato. Fica a estimativa se Newton sair do troço ou cair no par trivial do mesmo troço.
fn afina(ca: Cubica, cb: Cubica, t0: f64, u0: f64) -> (f64, f64) {
    let resto = |t: f64, u: f64| {
        let (p, q) = (
            cubic_at(ca[0], ca[1], ca[2], ca[3], t),
            cubic_at(cb[0], cb[1], cb[2], cb[3], u),
        );
        [p[0] - q[0], p[1] - q[1]]
    };
    let (mut t, mut u) = (t0, u0);
    for _ in 0..8 {
        let f = resto(t, u);
        let (da, db) = (derivada(ca, t), derivada(cb, u));
        let det = db[0].mul_add(da[1], -(da[0] * db[1]));
        if det.abs() < 1e-18 {
            break;
        }
        // J = [A' | −B']; Δ = −J⁻¹ F.
        t -= (-db[1] * f[0] + db[0] * f[1]) / det;
        u -= (da[0] * f[1] - da[1] * f[0]) / det;
    }
    let dentro = |x: f64| (0.0..=1.0).contains(&x);
    let [fx, fy] = resto(t, u);
    let [gx, gy] = resto(t0, u0);
    if dentro(t) && dentro(u) && (t - u).abs() > 1e-6 && fx.hypot(fy) <= gx.hypot(gy) {
        (t, u)
    } else {
        (t0, u0)
    }
}

/// `(t, u)` do cruzamento estrito de `ab` com `cd` (encostar numa ponta não conta).
fn corta(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> Option<(f64, f64)> {
    let r = [b[0] - a[0], b[1] - a[1]];
    let s = [d[0] - c[0], d[1] - c[1]];
    let den = r[0].mul_add(s[1], -(r[1] * s[0]));
    if den.abs() < 1e-18 {
        return None;
    }
    let q = [c[0] - a[0], c[1] - a[1]];
    let t = q[0].mul_add(s[1], -(q[1] * s[0])) / den;
    let u = q[0].mul_add(r[1], -(q[1] * r[0])) / den;
    let dentro = 1e-9..1.0 - 1e-9;
    (dentro.contains(&t) && dentro.contains(&u)).then_some((t, u))
}

struct Troco {
    pts: [[f64; 2]; PASSOS + 1],
    /// Arco (da poligonal) até cada ponto, desde o início do contorno.
    arco: [f64; PASSOS + 1],
    lo: [f64; 2],
    hi: [f64; 2],
}

fn achata(v: &[VecVertex], m: usize) -> (Vec<Troco>, f64) {
    let mut s = 0.0;
    let trocos = (0..m)
        .map(|k| {
            let c = troco(v, k);
            let mut t = Troco {
                pts: [[0.0; 2]; PASSOS + 1],
                arco: [0.0; PASSOS + 1],
                lo: [f64::INFINITY; 2],
                hi: [f64::NEG_INFINITY; 2],
            };
            for i in 0..=PASSOS {
                #[allow(clippy::cast_precision_loss)]
                let p = cubic_at(c[0], c[1], c[2], c[3], i as f64 / PASSOS as f64);
                if i > 0 {
                    let q = t.pts[i - 1];
                    s += (p[0] - q[0]).hypot(p[1] - q[1]);
                }
                t.pts[i] = p;
                t.arco[i] = s;
                t.lo = [t.lo[0].min(p[0]), t.lo[1].min(p[1])];
                t.hi = [t.hi[0].max(p[0]), t.hi[1].max(p[1])];
            }
            t
        })
        .collect();
    (trocos, s)
}

/// O primeiro nó pequeno: `(a, ta, d, tb)` — do troço `a` em `ta` ao troço `a + d` em `tb`.
fn primeiro_no(v: &[VecVertex], closed: bool) -> Option<(usize, f64, usize, f64)> {
    let nv = v.len();
    let m = if closed { nv } else { nv - 1 };
    let (tr, total) = achata(v, m);
    let limite = FRACAO * total;
    let ao_longo = |de: f64, ate: f64| {
        if ate >= de {
            ate - de
        } else {
            ate + total - de
        }
    };
    for a in 0..m {
        for d in 0..m {
            if !closed && a + d >= m {
                break;
            }
            let b = (a + d) % m;
            if d > 0 && ao_longo(tr[a].arco[PASSOS], tr[b].arco[0]) > limite {
                break;
            }
            let (ta, tb) = (&tr[a], &tr[b]);
            if (0..2).any(|e| ta.hi[e] < tb.lo[e] || tb.hi[e] < ta.lo[e]) {
                continue;
            }
            for i in 0..PASSOS {
                let j0 = if d == 0 { i + 2 } else { 0 };
                for j in j0..PASSOS {
                    let Some((t, u)) = corta(ta.pts[i], ta.pts[i + 1], tb.pts[j], tb.pts[j + 1])
                    else {
                        continue;
                    };
                    let sa = (ta.arco[i + 1] - ta.arco[i]).mul_add(t, ta.arco[i]);
                    let sb = (tb.arco[j + 1] - tb.arco[j]).mul_add(u, tb.arco[j]);
                    if ao_longo(sa, sb) <= limite {
                        #[allow(clippy::cast_precision_loss)]
                        let n = PASSOS as f64;
                        #[allow(clippy::cast_precision_loss)]
                        return Some((a, (i as f64 + t) / n, d, (j as f64 + u) / n));
                    }
                }
            }
        }
    }
    None
}

/// Corta os nós pequenos do contorno, um a um.
pub(crate) fn sem_nos(mut v: Vec<VecVertex>, closed: bool) -> Vec<VecVertex> {
    let voltas = v.len();
    for _ in 0..voltas {
        if v.len() < 3 {
            break;
        }
        let Some((a, ta, d, tb)) = primeiro_no(&v, closed) else {
            break;
        };
        let nv = v.len();
        let b = (a + d) % nv;
        let (ca, cb) = (troco(&v, a), troco(&v, b));
        let (ta, tb) = afina(ca, cb, ta, tb);
        let (esq, _) = parte(ca, ta);
        let (_, dir) = parte(cb, tb);
        let mut x =
            VecVertex::corner([0.5 * (esq[3][0] + dir[0][0]), 0.5 * (esq[3][1] + dir[0][1])]);
        x.in_handle = esq[2];
        x.out_handle = dir[1];
        v[a].out_handle = esq[1];
        v[(b + 1) % nv].in_handle = dir[2];
        let mut novo = Vec::with_capacity(nv + 1);
        for (k, vk) in v.iter().enumerate() {
            let depois = (k + nv - a) % nv;
            if (1..=d).contains(&depois) {
                continue;
            }
            novo.push(*vk);
            if k == a {
                novo.push(x);
            }
        }
        v = novo;
    }
    v
}

/// Quantos cruzamentos estritos tem o contorno, em qualquer distância (os gates do Zig Zag e do
/// Sketch).
#[cfg(test)]
pub(crate) fn cruzamentos(v: &[VecVertex], closed: bool) -> usize {
    let m = if closed { v.len() } else { v.len() - 1 };
    let (tr, _) = achata(v, m);
    let mut x = 0;
    for a in 0..m {
        for b in a..m {
            let (ta, tb) = (&tr[a], &tr[b]);
            if (0..2).any(|e| ta.hi[e] < tb.lo[e] || tb.hi[e] < ta.lo[e]) {
                continue;
            }
            for i in 0..PASSOS {
                let j0 = if a == b { i + 2 } else { 0 };
                for j in j0..PASSOS {
                    x += usize::from(
                        corta(ta.pts[i], ta.pts[i + 1], tb.pts[j], tb.pts[j + 1]).is_some(),
                    );
                }
            }
        }
    }
    x
}

#[cfg(test)]
#[path = "fx_zigzag_corta_tests.rs"]
mod tests;
