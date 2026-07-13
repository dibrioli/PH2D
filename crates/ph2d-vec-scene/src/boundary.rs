//! **Onde a linha ENCOSTA na forma** — módulo irmão de [`crate::path_ops`].
//!
//! Um conector que parasse no *centro* da caixa desapareceria por baixo dela. Ele tem de
//! parar na **borda** — e "a borda" de uma forma arbitrária (uma estrela, uma nuvem, uma
//! lua) não é uma conta trivial.
//!
//! ## Aqui nós temos algo que o draw.io não tem
//!
//! O draw.io, para um stencil qualquer, cai de volta no **retângulo envolvente**: a linha
//! encosta na caixa invisível ao redor da estrela, não na estrela. Nós temos a curva de
//! verdade — o mesmo achatamento que o `path_contains_point` já usa — então intersectamos o
//! contorno REAL. Sai de graça, e fica melhor.
//!
//! ## O detalhe que decide: o MAIOR `t`, não o primeiro
//!
//! Numa forma **côncava** o raio entra e sai várias vezes. Parar no primeiro cruzamento
//! prenderia a linha numa **reentrância** — o conector sairia do vale de uma estrela em vez
//! da ponta, e ficaria por baixo dela. Ficar com o cruzamento de maior `t` (o último) é o que
//! garante que a linha sai de vez da forma.
//!
//! Numa forma convexa (a esmagadora maioria) os dois critérios coincidem, e é por isso que o
//! erro passa despercebido até alguém conectar uma estrela.

use crate::VecPath;

/// Amostras por segmento de cúbica ao achatar o contorno. O mesmo número que o
/// `path_contains_point` usa — a borda que o hit-test enxerga é a mesma em que a linha encosta.
const SAMPLES: usize = 16;

/// Um ponto da cúbica de Bézier em `t`.
fn cubic(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + w3 * p3[0],
        w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + w3 * p3[1],
    ]
}

/// O contorno da forma, achatado em polilinhas de MUNDO. Só os contornos **fechados** — uma
/// linha de construção (a aresta interna de um cubo) não é borda de nada.
///
/// `xform` leva local→mundo (o afim da entidade, ADR-0111): `[a, b, c, d, e, f]` na convenção
/// `x' = a·x + c·y + e`.
#[must_use]
pub fn outline(path: &VecPath, xform: [f64; 6]) -> Vec<Vec<[f64; 2]>> {
    // Cozida: é onde o conector tem de encostar — na borda que se VÊ, não na quina
    // afiada que o documento guarda por baixo dela.
    let path = &*path.cooked();
    let m = |p: [f64; 2]| -> [f64; 2] {
        [
            xform[0] * p[0] + xform[2] * p[1] + xform[4],
            xform[1] * p[0] + xform[3] * p[1] + xform[5],
        ]
    };
    let mut out = Vec::new();
    for c in 0..path.contour_count() {
        let Some((verts, closed)) = path.contour(c) else {
            continue;
        };
        if !closed || verts.len() < 2 {
            continue;
        }
        let mut poly = Vec::with_capacity(verts.len() * SAMPLES);
        for i in 0..verts.len() {
            let a = &verts[i];
            let b = &verts[(i + 1) % verts.len()];
            for k in 0..SAMPLES {
                let t = k as f64 / SAMPLES as f64;
                poly.push(m(cubic(a.anchor, a.out_handle, b.in_handle, b.anchor, t)));
            }
        }
        out.push(poly);
    }
    out
}

/// O parâmetro `t` do cruzamento do raio `from + t·dir` com o segmento `a`–`b`, se houver.
///
/// Determinante 2×2 puro — zero transcendental (HR-5).
fn cross(from: [f64; 2], dir: [f64; 2], a: [f64; 2], b: [f64; 2]) -> Option<f64> {
    let e = [b[0] - a[0], b[1] - a[1]];
    let den = dir[0] * e[1] - dir[1] * e[0];
    if den.abs() < 1e-12 {
        return None; // paralelos
    }
    let w = [a[0] - from[0], a[1] - from[1]];
    let t = (w[0] * e[1] - w[1] * e[0]) / den; // ao longo do RAIO
    let s = (w[0] * dir[1] - w[1] * dir[0]) / den; // ao longo da ARESTA
    if t > 1e-9 && (-1e-9..=1.0 + 1e-9).contains(&s) {
        Some(t)
    } else {
        None
    }
}

/// **Onde a linha encosta na forma.** O raio parte de `from` (tipicamente o centro da forma) na
/// direção `dir`, e o resultado é o ponto onde ele **sai de vez** do contorno.
///
/// `gap` recua o ponto de volta para dentro do raio (uma folga estética entre a forma e a
/// linha; `0` = encostado).
///
/// `None` = nem o contorno nem nada foi cruzado (forma degenerada). O chamador cai na bbox.
#[must_use]
pub fn boundary_hit(
    path: &VecPath,
    xform: [f64; 6],
    from: [f64; 2],
    dir: [f64; 2],
    gap: f64,
) -> Option<[f64; 2]> {
    let len = dir[0].hypot(dir[1]);
    if len < 1e-12 {
        return None;
    }
    let d = [dir[0] / len, dir[1] / len];

    // **O MAIOR t, não o primeiro.** Numa forma côncava (estrela, nuvem, lua) o raio entra e
    // sai várias vezes; o primeiro cruzamento pode ser a parede de um vale, e a linha pararia
    // lá dentro. O último é o único que garante que ela saiu.
    let mut best = f64::NEG_INFINITY;
    for poly in outline(path, xform) {
        for w in poly.windows(2) {
            if let Some(t) = cross(from, d, w[0], w[1]) {
                best = best.max(t);
            }
        }
        // O fecho do contorno (último → primeiro), que o `windows` não cobre.
        if let (Some(a), Some(b)) = (poly.last(), poly.first())
            && let Some(t) = cross(from, d, *a, *b)
        {
            best = best.max(t);
        }
    }
    if !best.is_finite() {
        return None;
    }
    let t = (best - gap).max(0.0);
    Some([from[0] + d[0] * t, from[1] + d[1] * t])
}

#[cfg(test)]
#[path = "boundary_tests.rs"]
mod tests;
