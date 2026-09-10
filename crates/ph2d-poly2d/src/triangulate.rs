//! **O que a deformação MOVE** — o polígono partido em triângulos.

/// ⭐⭐ **TRIANGULA um polígono simples** por *ear-clipping*. Devolve triplas de índices no `ring`.
///
/// ⚠️ **A orientação é normalizada aqui e não exigida de quem chama** — o [`crate::contour`]
/// entrega o sentido que o rastreio deu, e obrigar o chamador a corrigi-lo seria pôr a mesma
/// decisão em dois sítios. As triplas saem sempre no sentido em que a [`crate::signed_area`] é
/// positiva.
///
/// ⛔ **Polígono SIMPLES, sem buracos e sem auto-intersecção.** Um anel do [`crate::contour`]
/// simplificado por [`crate::simplify`] cumpre-o; um polígono desenhado à mão pode não cumprir, e
/// aí o tecto de segurança devolve o que conseguiu em vez de pendurar o app.
///
/// ⚠️ Ele é o gémeo em `f64` do `ph2d_flip_render::fill::triangulate` — a nota do [`crate`] diz
/// porque são dois e qual é a medição que autoriza fundi-los.
#[must_use]
pub fn triangulate(ring: &[[f64; 2]]) -> Vec<[u32; 3]> {
    let n = ring.len();
    if n < 3 {
        return Vec::new();
    }
    let ccw = crate::signed_area(ring) >= 0.0;
    let mut idx: Vec<usize> = if ccw {
        (0..n).collect()
    } else {
        (0..n).rev().collect()
    };
    let mut out: Vec<[u32; 3]> = Vec::with_capacity(n.saturating_sub(2));
    // ⚠️ Tecto contra o polígono degenerado: sem ele um anel auto-intersectado não tem orelha
    // nenhuma e o laço não devolve. *Um algoritmo que pendura é pior que um que entrega menos.*
    let mut guarda = n * n + 8;
    while idx.len() > 3 && guarda > 0 {
        guarda -= 1;
        let m = idx.len();
        let mut cortou = false;
        for k in 0..m {
            let (ia, ib, ic) = (idx[(k + m - 1) % m], idx[k], idx[(k + 1) % m]);
            let (a, b, c) = (ring[ia], ring[ib], ring[ic]);
            // Convexo? (com o polígono já em CCW, a orelha tem produto cruzado positivo)
            if cross(a, b, c) <= 0.0 {
                continue;
            }
            // E vazio: nenhum outro vértice do polígono lá dentro.
            let vazio = idx
                .iter()
                .filter(|&&i| i != ia && i != ib && i != ic)
                .all(|&i| !dentro(ring[i], a, b, c));
            if !vazio {
                continue;
            }
            #[expect(
                clippy::cast_possible_truncation,
                reason = "um anel de contorno de imagem não passa de 2^32 pontos — a imagem não cabe em memória muito antes disso"
            )]
            out.push([ia as u32, ib as u32, ic as u32]);
            idx.remove(k);
            cortou = true;
            break;
        }
        if !cortou {
            // Sem orelha: o polígono não é simples. Entrega o que há.
            return out;
        }
    }
    if idx.len() == 3 {
        #[expect(clippy::cast_possible_truncation, reason = "ver acima")]
        out.push([idx[0] as u32, idx[1] as u32, idx[2] as u32]);
    }
    out
}

fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]).mul_add(c[1] - a[1], -((b[1] - a[1]) * (c[0] - a[0])))
}

/// `q` está dentro (ou na borda) do triângulo `a b c`, que está em CCW.
fn dentro(q: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    cross(a, b, q) >= 0.0 && cross(b, c, q) >= 0.0 && cross(c, a, q) >= 0.0
}
