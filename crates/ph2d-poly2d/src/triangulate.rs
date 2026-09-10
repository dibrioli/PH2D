//! **O que a deformação MOVE** — o polígono partido em triângulos.

/// ⭐⭐ **TRIANGULA um polígono simples** por *ear-clipping*. Devolve triplas de índices no `ring`.
///
/// ⚠️ **A orientação é normalizada aqui e não exigida de quem chama** — o [`crate::contour`]
/// entrega o sentido que o rastreio deu, e obrigar o chamador a corrigi-lo seria pôr a mesma
/// decisão em dois sítios. As triplas saem sempre no sentido em que a [`crate::signed_area`] é
/// positiva.
///
/// ⛔ **Polígono SIMPLES, sem buracos e sem auto-intersecção.** Um anel do [`crate::contour`]
/// simplificado por [`crate::simplify`] cumpre-o; um polígono desenhado à mão pode não cumprir.
///
/// ⛔⛔⛔ **`None` quando não há orelha**, e a 1.ª redacção devolvia *«o que conseguiu»* em
/// silêncio: isso troca uma pendura por uma **malha errada que ninguém vê** — a peça sai com um
/// buraco, o artista não recebe aviso nenhum, e nenhum gate da suíte observava a guarda. *Um
/// algoritmo que pendura é pior que um que entrega menos, mas os dois são piores que um que RECUSA
/// em voz alta.*
///
/// ⚠️ Ele é o gémeo em `f64` do `ph2d_flip_render::fill::triangulate` — a nota do [`crate`] diz
/// porque são dois e qual é a medição que autoriza fundi-los.
#[must_use]
pub fn triangulate(ring: &[[f64; 2]]) -> Option<Vec<[u32; 3]>> {
    let n = ring.len();
    if n < 3 {
        // ⚠️ Menos de três pontos não é uma recusa: é um polígono que legitimamente não tem área.
        return Some(Vec::new());
    }
    let ccw = crate::signed_area(ring) >= 0.0;
    let mut idx: Vec<usize> = if ccw {
        (0..n).collect()
    } else {
        (0..n).rev().collect()
    };
    let mut out: Vec<[u32; 3]> = Vec::with_capacity(n.saturating_sub(2));
    // ⭐⭐⭐ **O LAÇO TERMINA POR CONSTRUÇÃO, e não por um contador.**
    //
    // ⚠️⚠️ **Havia aqui um `guarda = n² + 8`, e ele era INALCANÇÁVEL** — descoberto por uma mutação
    // que sobreviveu (2026-09-10): cada volta ou **corta** um vértice (no máximo `n − 3` vezes) ou
    // **recusa**, e `n − 3` nunca chega a `n² + 8`. *Um tecto que não pode morder é pior que nenhum:
    // ele faz quem lê pensar que o caso está tratado.*
    //
    // ⇒ quem faz este laço terminar é a **recusa** lá em baixo, que agora é observável e tem gate.
    while idx.len() > 3 {
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
            // ⛔ **A RECUSA nº 1**: sem orelha nenhuma o polígono não é simples, e o que há é uma
            // triangulação que não cobre a forma.
            return None;
        }
    }
    if idx.len() == 3 {
        #[expect(clippy::cast_possible_truncation, reason = "ver acima")]
        out.push([idx[0] as u32, idx[1] as u32, idx[2] as u32]);
    }
    Some(out)
}

fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]).mul_add(c[1] - a[1], -((b[1] - a[1]) * (c[0] - a[0])))
}

/// `q` está dentro (ou na borda) do triângulo `a b c`, que está em CCW.
fn dentro(q: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    cross(a, b, q) >= 0.0 && cross(b, c, q) >= 0.0 && cross(c, a, q) >= 0.0
}
