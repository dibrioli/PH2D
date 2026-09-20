//! ⭐⭐ **A ORIENTAÇÃO de uma peça** — a caixa mais pequena que a envolve, e o ângulo que
//! a dá.
//!
//! Uma ilha esguia e inclinada ocupa uma caixa quase vazia, e o empacotador arruma
//! **caixas**. Rodar a peça até a caixa ser mínima é a única coisa que encolhe o
//! invólucro **sem tocar na geometria**: é um movimento rígido, logo tudo o que o
//! [`crate::corte`] provou sobre a peça continua verdade por construção.
//!
//! ⭐ **O mínimo acha-se exactamente, não por varredura de ângulos:** a caixa de área
//! mínima de um convexo tem sempre um lado **colinear com uma aresta do casco**
//! (Freeman–Shapira). ⇒ basta experimentar as arestas do casco, e o resultado não depende
//! de quão fina foi a varredura.

/// O casco convexo, em sentido anti-horário (cadeia monótona de Andrew).
fn casco(p: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut q: Vec<[f64; 2]> = p.to_vec();
    q.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    q.dedup();
    if q.len() < 3 {
        return q;
    }
    let cruz = |o: [f64; 2], a: [f64; 2], b: [f64; 2]| {
        (a[0] - o[0]).mul_add(b[1] - o[1], -((a[1] - o[1]) * (b[0] - o[0])))
    };
    let mut out: Vec<[f64; 2]> = Vec::with_capacity(q.len() * 2);
    for &z in &q {
        while out.len() >= 2 && cruz(out[out.len() - 2], out[out.len() - 1], z) <= 0.0 {
            out.pop();
        }
        out.push(z);
    }
    let baixo = out.len() + 1;
    for &z in q.iter().rev() {
        while out.len() >= baixo && cruz(out[out.len() - 2], out[out.len() - 1], z) <= 0.0 {
            out.pop();
        }
        out.push(z);
    }
    out.pop();
    out
}

/// ⭐ **O eixo em que a caixa da nuvem é mínima**, como vector unitário `(cos, sin)`.
///
/// Devolve o eixo `x` do referencial novo; o `y` é a perpendicular directa. Uma nuvem com
/// menos de três pontos, ou toda num ponto, devolve o eixo original — ⚠️ *uma degenerada
/// tem de sair da porta com a IDENTIDADE e nunca com um vector nulo normalizado*, que é
/// ruído a fingir de direcção.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn eixo_da_caixa_minima(pontos: &[[f32; 2]]) -> [f32; 2] {
    let p: Vec<[f64; 2]> = pontos
        .iter()
        .map(|z| [f64::from(z[0]), f64::from(z[1])])
        .collect();
    let h = casco(&p);
    if h.len() < 3 {
        return [1.0, 0.0];
    }
    let (mut melhor, mut eixo) = (f64::INFINITY, [1.0f64, 0.0]);
    for i in 0..h.len() {
        let (a, b) = (h[i], h[(i + 1) % h.len()]);
        let d = [b[0] - a[0], b[1] - a[1]];
        let n = d[0].hypot(d[1]);
        if n <= 0.0 {
            continue;
        }
        let d = [d[0] / n, d[1] / n];
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for &z in &h {
            let u = z[0].mul_add(d[0], z[1] * d[1]);
            let v = z[1].mul_add(d[0], -(z[0] * d[1]));
            lo[0] = lo[0].min(u);
            hi[0] = hi[0].max(u);
            lo[1] = lo[1].min(v);
            hi[1] = hi[1].max(v);
        }
        let area = (hi[0] - lo[0]) * (hi[1] - lo[1]);
        if area < melhor {
            melhor = area;
            eixo = d;
        }
    }
    [eixo[0] as f32, eixo[1] as f32]
}

/// Leva um ponto para o referencial de `eixo` — ⭐ um movimento **rígido**: comprimentos e
/// áreas ficam iguais, logo a injectividade que o corte provou sobrevive por construção.
#[must_use]
pub fn roda(z: [f32; 2], eixo: [f32; 2]) -> [f32; 2] {
    [
        z[0].mul_add(eixo[0], z[1] * eixo[1]),
        z[1].mul_add(eixo[0], -(z[0] * eixo[1])),
    ]
}
