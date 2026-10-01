//! ⭐⭐ **O ESPORÃO QUE A UNIÃO DEIXA** (F46, o braço dobrado de VOLTA).
//!
//! Com uma junta a `~174°` a pele dos dois membros quase coincide, e a união emite um pedaço de
//! contorno que VOLTA pelo mesmo caminho: medido a `(0°, 174°)`, um segmento recto de `0,0605` que
//! refaz o fim do segmento anterior e um nó a virar `180°` na ponta. A área dele é ZERO — como forma
//! não existe —, mas o traço desenha a meia-volta. ⛔ Nem a bola nem o desfazer dos ganchos o tiram:
//! a bola lê o sentido de uma viragem de `180°` pelo sinal de um produto vectorial que é ruído, e o
//! desfazer dos ganchos pára no tamanho da bola ([`crate::gancho`]) — e este recuo é maior.
//!
//! # A cura
//!
//! Um nó que vira acima de [`crate::gancho::VIRAGEM_DO_GANCHO`] e cujo segmento MAIS CURTO dos dois
//! que o tocam fica todo a menos de `tol` do MAIS LONGO é a ponta de um esporão: o mais longo é cortado
//! no ponto onde o curto acaba, o curto sai, e o nó da ponta com ele. A forma desenhada não muda —
//! ela é o mesmo conjunto de pontos —, e o comprimento não tem tecto, porque um esporão de área zero
//! nunca é forma, seja de que tamanho for.

use kurbo::{CubicBez, ParamCurve, Point};
use ph2d_vec_scene::VecVertex;

/// Amostras por cúbica na comparação.
const AMOSTRAS: usize = 32;

fn pt(a: [f64; 2]) -> Point {
    Point::new(a[0], a[1])
}

fn arr(p: Point) -> [f64; 2] {
    [p.x, p.y]
}

fn cubica(verts: &[VecVertex], j: usize) -> CubicBez {
    let n = verts.len();
    let (c, q) = (&verts[j], &verts[(j + 1) % n]);
    CubicBez::new(
        pt(c.anchor),
        pt(c.out_handle),
        pt(q.in_handle),
        pt(q.anchor),
    )
}

/// `(distância, parâmetro)` do ponto de `c` mais perto de `p`, por amostragem densa e refinamento
/// local por secção áurea.
fn mais_perto(c: &CubicBez, p: Point) -> (f64, f64) {
    let n = 4 * AMOSTRAS;
    #[allow(clippy::cast_precision_loss)]
    let (mut t, mut d) = (0..=n)
        .map(|k| {
            let t = k as f64 / n as f64;
            (t, (c.eval(t) - p).hypot())
        })
        .fold((0.0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a });
    #[allow(clippy::cast_precision_loss)]
    let passo = 1.0 / n as f64;
    let (mut lo, mut hi) = ((t - passo).max(0.0), (t + passo).min(1.0));
    let g = 0.5 * (5.0_f64.sqrt() - 1.0);
    for _ in 0..60 {
        let (x1, x2) = (hi - g * (hi - lo), lo + g * (hi - lo));
        if (c.eval(x1) - p).hypot() < (c.eval(x2) - p).hypot() {
            hi = x2;
        } else {
            lo = x1;
        }
    }
    let tm = 0.5 * (lo + hi);
    let dm = (c.eval(tm) - p).hypot();
    if dm < d {
        (t, d) = (tm, dm);
    }
    (d, t)
}

/// O segmento `curto` fica todo a menos de `tol` do `longo`?
fn em_cima(curto: &CubicBez, longo: &CubicBez, tol: f64) -> bool {
    (0..=AMOSTRAS).all(|k| {
        #[allow(clippy::cast_precision_loss)]
        let t = k as f64 / AMOSTRAS as f64;
        mais_perto(longo, curto.eval(t)).0 <= tol
    })
}

/// ⭐⭐ **Tira os esporões do contorno fechado `verts`** — ver o módulo. Nada a tirar ⇒ `verts`
/// intacto, ao bit.
#[must_use]
pub fn tira_os_esporoes(mut verts: Vec<VecVertex>, tol: f64) -> Vec<VecVertex> {
    if !tol.is_finite() || tol <= 0.0 {
        return verts;
    }
    // Cada volta tira um esporão; um contorno de `n` nós tem no máximo `n` deles.
    for _ in 0..verts.len() {
        let n = verts.len();
        if n < 4 {
            return verts;
        }
        let ponta = (0..n).find(|&i| {
            crate::overlap::viragem_do_vertice(&verts, i)
                .is_some_and(|v| v > crate::gancho::VIRAGEM_DO_GANCHO)
                && {
                    let (ant, seg) = (cubica(&verts, (i + n - 1) % n), cubica(&verts, i));
                    if ant.p0.distance(ant.p3) >= seg.p0.distance(seg.p3) {
                        em_cima(&seg, &ant, tol)
                    } else {
                        em_cima(&ant, &seg, tol)
                    }
                }
        });
        let Some(i) = ponta else {
            return verts;
        };
        let (a, b) = ((i + n - 1) % n, (i + 1) % n);
        let (ant, seg) = (cubica(&verts, a), cubica(&verts, i));
        if ant.p0.distance(ant.p3) >= seg.p0.distance(seg.p3) {
            // O curto é o de SAÍDA: o anterior acaba onde ele acaba, no nó `b`.
            let (_, t) = mais_perto(&ant, seg.p3);
            let c = ant.subsegment(0.0..t);
            verts[a].out_handle = arr(c.p1);
            verts[b].in_handle = arr(c.p2);
        } else {
            // O curto é o de ENTRADA: o seguinte começa onde ele começa, no nó `a`.
            let (_, t) = mais_perto(&seg, ant.p0);
            let c = seg.subsegment(t..1.0);
            verts[a].out_handle = arr(c.p1);
            verts[b].in_handle = arr(c.p2);
        }
        verts.remove(i);
    }
    verts
}

#[cfg(test)]
#[path = "esporao_tests.rs"]
mod tests;
