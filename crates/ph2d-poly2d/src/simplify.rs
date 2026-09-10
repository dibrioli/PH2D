//! **O ORÇAMENTO do artista** — o mesmo contorno com menos pontos.

/// ⭐⭐ **SIMPLIFICA um anel fechado** (Ramer–Douglas–Peucker), guardando todo ponto que se afaste
/// mais de `tol` da recta que os vizinhos guardados descrevem.
///
/// ⚠️ **Ele é FECHADO, e é isso que o distingue do RDP de manual.** Uma curva aberta tem duas
/// pontas fixas por definição; um anel não tem nenhuma, e prender a primeira do vector seria
/// deixar a simplificação depender de onde o rastreio começou. ⇒ os dois pontos fixos são os
/// **mais distantes um do outro** — o diâmetro do anel —, que é uma propriedade da FORMA.
///
/// ⚠️ **`tol` é em pixels da imagem** e é o único número que o artista vê (a *densidade*): mais
/// tolerância, menos vértices, menos triângulos para a deformação mover.
///
/// Um anel de menos de quatro pontos volta como está — não há o que cortar.
#[must_use]
pub fn simplify(ring: &[[f64; 2]], tol: f64) -> Vec<[f64; 2]> {
    let n = ring.len();
    if n < 4 || tol <= 0.0 || !tol.is_finite() {
        return ring.to_vec();
    }
    // Os dois pontos fixos: o par mais distante. ⚠️ O laço é O(n²) e é **medido como aceitável**
    // porque corre UMA vez por bind, nunca por quadro — o anel de uma sprite de `512 px` tem
    // ~2 000 pontos, e 2 000² comparações de distância são ~4 M operações de `f64`.
    let (mut ia, mut ib, mut melhor) = (0usize, n / 2, -1.0f64);
    for i in 0..n {
        for j in (i + 1)..n {
            let (a, b) = (ring[i], ring[j]);
            let d = (b[0] - a[0]).mul_add(b[0] - a[0], (b[1] - a[1]) * (b[1] - a[1]));
            if d > melhor {
                (melhor, ia, ib) = (d, i, j);
            }
        }
    }
    // O anel parte-se em dois caminhos abertos entre os pontos fixos, e cada um simplifica-se
    // sozinho. ⚠️ O segundo tem de dar a volta pelo fim do vector — é aí que ele é um ANEL.
    let volta: Vec<[f64; 2]> = ring[ib..].iter().chain(&ring[..=ia]).copied().collect();
    let mut out = rdp(&ring[ia..=ib], tol);
    let segundo = rdp(&volta, tol);
    // As pontas repetem-se entre as duas metades: `ib` fecha a primeira e abre a segunda, e `ia`
    // fecha a segunda e é o primeiro ponto do anel. Tirá-las é o que mantém o anel simples.
    out.extend(segundo.iter().skip(1).take(segundo.len().saturating_sub(2)));
    out
}

/// RDP sobre um caminho ABERTO — as duas pontas ficam sempre.
fn rdp(path: &[[f64; 2]], tol: f64) -> Vec<[f64; 2]> {
    let n = path.len();
    if n < 3 {
        return path.to_vec();
    }
    let (a, b) = (path[0], path[n - 1]);
    let (mut pior, mut k) = (0.0f64, 0usize);
    for (i, &p) in path.iter().enumerate().take(n - 1).skip(1) {
        let d = dist_to_segment(p, a, b);
        if d > pior {
            (pior, k) = (d, i);
        }
    }
    if pior <= tol {
        return vec![a, b];
    }
    let mut esq = rdp(&path[..=k], tol);
    let dir = rdp(&path[k..], tol);
    esq.pop(); // o pivô está nas duas metades
    esq.extend(dir);
    esq
}

/// Distância de `p` ao **segmento** `a..b` — ⛔ nunca à recta infinita, senão uma volta apertada
/// no fim de um caminho mede-se contra uma recta que passa longe dela e é cortada.
fn dist_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (vx, vy) = (b[0] - a[0], b[1] - a[1]);
    let (wx, wy) = (p[0] - a[0], p[1] - a[1]);
    let len2 = vx.mul_add(vx, vy * vy);
    let t = if len2 <= f64::MIN_POSITIVE {
        0.0
    } else {
        (wx.mul_add(vx, wy * vy) / len2).clamp(0.0, 1.0)
    };
    let (qx, qy) = (t.mul_add(vx, a[0]), t.mul_add(vy, a[1]));
    (p[0] - qx).hypot(p[1] - qy)
}
