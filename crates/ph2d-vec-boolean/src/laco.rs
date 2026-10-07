//! ⭐⭐ **O LAÇO QUE A UNIÃO DEIXA** (A13, o braço da dobra forte a `(22°…28°, 130°)`).
//!
//! A saída da união chega a cruzar-se a si mesma perto de um cruzamento das duas peles: medido a
//! `(24°, 130°)` com `1 024` amostras por forma, o segmento `26` corta o `28` a `0,012` da ponta, e o
//! nó `27`→`28` (`0,0096`) fica num laço de área quase nula. O traço desenha a ponta do laço (`138°`–
//! `145°`), e nenhuma passagem seguinte a tira: o fecho só toca no côncavo, e a abertura não acha
//! par de paralelas que se cruze (os dois flancos estão trocados).
//!
//! # A cura
//!
//! Cada cruzamento PRÓPRIO do contorno (no interior de dois segmentos) parte-o em dois laços; o que
//! cabe todo na bola (a menos de `raio` do cruzamento) sai, e o cruzamento passa a ser um nó. Um
//! laço maior que a bola é forma e fica. Nada a tirar ⇒ `verts` intacto, ao bit.

use kurbo::{CubicBez, ParamCurve, Point};
use ph2d_vec_scene::VecVertex;

/// Amostras por cúbica na procura dos cruzamentos.
const AMOSTRAS: usize = 32;

fn cubica(v: &[VecVertex], j: usize) -> CubicBez {
    let n = v.len();
    let (a, b) = (&v[j], &v[(j + 1) % n]);
    let p = |x: [f64; 2]| Point::new(x[0], x[1]);
    CubicBez::new(p(a.anchor), p(a.out_handle), p(b.in_handle), p(b.anchor))
}

/// O cruzamento próprio de `[a, b]` com `[c, d]`: os parâmetros nos dois.
fn cruza(a: Point, b: Point, c: Point, d: Point) -> Option<(f64, f64)> {
    let (r, s) = (b - a, d - c);
    let den = r.cross(s);
    if den.abs() < 1e-300 {
        return None;
    }
    let q = c - a;
    let (u, w) = (q.cross(s) / den, q.cross(r) / den);
    const E: f64 = 1e-9;
    (u > E && u < 1.0 - E && w > E && w < 1.0 - E).then_some((u, w))
}

/// O primeiro cruzamento próprio entre os segmentos `i < k`: `(t_i, t_k, ponto)`.
fn cruzamento(v: &[VecVertex], i: usize, k: usize) -> Option<(f64, f64, Point)> {
    #[expect(clippy::cast_precision_loss, reason = "um punhado")]
    let t = |s: usize| s as f64 / AMOSTRAS as f64;
    let (ci, ck) = (cubica(v, i), cubica(v, k));
    let pi: Vec<Point> = (0..=AMOSTRAS).map(|s| ci.eval(t(s))).collect();
    let pk: Vec<Point> = (0..=AMOSTRAS).map(|s| ck.eval(t(s))).collect();
    for (a, wa) in pi.windows(2).enumerate() {
        for (b, wb) in pk.windows(2).enumerate() {
            if let Some((u, w)) = cruza(wa[0], wa[1], wb[0], wb[1]) {
                let x = wa[0] + (wa[1] - wa[0]) * u;
                return Some((t(a) + u * t(1), t(b) + w * t(1), x));
            }
        }
    }
    None
}

/// O laço do caminho de `(i, ti)` para a frente até `(k, tk)` cabe todo a menos de `raio` de `x`?
fn cabe(
    v: &[VecVertex],
    (i, ti): (usize, f64),
    (k, tk): (usize, f64),
    x: Point,
    raio: f64,
) -> bool {
    let n = v.len();
    let dentro = |c: CubicBez| {
        (0..=AMOSTRAS).all(|s| {
            #[expect(clippy::cast_precision_loss, reason = "um punhado")]
            let t = s as f64 / AMOSTRAS as f64;
            (c.eval(t) - x).hypot() <= raio
        })
    };
    let mut j = i;
    let mut c = cubica(v, i).subsegment(ti..1.0);
    for _ in 0..n {
        if j == k {
            return dentro(cubica(v, k).subsegment(0.0..tk));
        }
        if !dentro(c) {
            return false;
        }
        j = (j + 1) % n;
        c = cubica(v, j);
        if j == k {
            return dentro(cubica(v, k).subsegment(0.0..tk));
        }
    }
    false
}

/// Tira o laço de `(i, ti)` para a frente até `(k, tk)`: o cruzamento `x` passa a ser um nó.
fn tira(v: &[VecVertex], (i, ti): (usize, f64), (k, tk): (usize, f64), x: Point) -> Vec<VecVertex> {
    let n = v.len();
    let a = cubica(v, i).subsegment(0.0..ti);
    let b = cubica(v, k).subsegment(tk..1.0);
    let arr = |p: Point| [p.x, p.y];
    // Os nós que ficam: o que abre `i`, o cruzamento, e do que fecha `k` até antes de `i`.
    let mut primeiro = v[i];
    primeiro.out_handle = arr(a.p1);
    let mut no = v[i];
    no.anchor = arr(x);
    no.in_handle = arr(a.p2);
    no.out_handle = arr(b.p1);
    no.kind = crate::classify(&no);
    let mut out = vec![primeiro, no];
    let mut j = (k + 1) % n;
    while j != i {
        out.push(v[j]);
        j = (j + 1) % n;
    }
    // O nó que fecha `k` chega pela cauda dele (é o `primeiro` quando o laço era o resto todo).
    let fim = if out.len() > 2 { 2 } else { 0 };
    out[fim].in_handle = arr(b.p2);
    out[fim].kind = crate::classify(&out[fim]);
    out
}

/// ⭐⭐ **Tira os laços do contorno fechado `verts` que cabem na bola de `raio`** — ver o módulo.
#[must_use]
pub fn tira_os_lacos(mut verts: Vec<VecVertex>, raio: f64) -> Vec<VecVertex> {
    if !raio.is_finite() || raio <= 0.0 {
        return verts;
    }
    // Cada volta tira um laço; um contorno de `n` nós tem no máximo `n` deles.
    for _ in 0..verts.len() {
        let n = verts.len();
        if n < 4 {
            return verts;
        }
        let achado = (0..n).find_map(|i| {
            (i + 1..n).find_map(|k| {
                let (ti, tk, x) = cruzamento(&verts, i, k)?;
                if cabe(&verts, (i, ti), (k, tk), x, raio) {
                    Some(tira(&verts, (i, ti), (k, tk), x))
                } else if cabe(&verts, (k, tk), (i, ti), x, raio) {
                    Some(tira(&verts, (k, tk), (i, ti), x))
                } else {
                    None
                }
            })
        });
        match achado {
            Some(v) => verts = v,
            None => return verts,
        }
    }
    verts
}

#[cfg(test)]
#[path = "laco_tests.rs"]
mod tests;
