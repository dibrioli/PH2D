//! **Que vértices pertencem a que osso** — a condição de Dirichlet do problema.
//!
//! ⚠️⚠️ **É aqui que um osso deixa de ser um raio e passa a ser uma ÂNCORA.** Na lei anterior o osso
//! tinha um *alcance* (`força × comprimento`) e tudo o que caísse fora dele ficava órfão; aqui ele
//! só diz *«este pedaço da arte é meu»*, e **quem decide o resto é a energia** sobre a arte. É essa
//! troca que apaga o item que estava no topo da fila desde 2026-09-09 — *o osso não sabia nada da
//! arte que carrega*, e agora não precisa de saber: ele fixa, a arte resolve.

/// Um osso, no espaço da malha (pixels da imagem no repouso).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Handle {
    /// A origem do eixo.
    pub a: [f64; 2],
    /// A ponta do eixo.
    pub b: [f64; 2],
}

/// A distância ao quadrado de `p` ao segmento `a→b`, e a fracção do pé da perpendicular.
fn dist2(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [p[0] - a[0], p[1] - a[1]];
    let den = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if den > f64::EPSILON {
        ((ap[0] * ab[0] + ap[1] * ab[1]) / den).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let d = [ap[0] - t * ab[0], ap[1] - t * ab[1]];
    d[0] * d[0] + d[1] * d[1]
}

/// ⭐⭐⭐ **De quem é cada vértice** — `Some(j)` quando o vértice `v` fica PRESO ao osso `j`.
///
/// A regra tem duas metades e as duas são precisas:
///
/// 1. **um vértice suficientemente perto do eixo de um osso é dele** — a tolerância é a **aresta
///    média da malha**, ⛔ nunca um número em pixels: numa arte dez vezes maior a malha é dez vezes
///    mais larga, e uma tolerância absoluta prenderia dez vezes menos vértices;
/// 2. **todo osso fica com pelo menos UM vértice** — o mais próximo dele.
///
/// ⚠️⚠️ **Sem a segunda metade, um osso curto entre dois cortes da grelha não prende nada** e a
/// energia dele fica sem condição de fronteira: a solução seria `w ≡ 0` e a normalização final
/// entregaria a arte inteira ao vizinho, **em silêncio**.
///
/// ⚠️ **Em disputa ganha o MAIS PRÓXIMO.** Dois ossos que se encontram numa junta partilham o ponto,
/// e sem desempate o vértice ficaria do último a ser visto — *a ordem da lista não é uma lei*.
#[must_use]
pub(crate) fn pin(
    mesh: &ph2d_poly2d::Mesh2d,
    ossos: &[Handle],
    folga_da_junta: f64,
) -> Vec<Option<usize>> {
    let n = mesh.rest.len();
    let tol2 = {
        let mut soma = 0.0;
        let mut conta = 0usize;
        for t in &mesh.tris {
            for e in 0..3 {
                let (i, j) = (t[e] as usize, t[(e + 1) % 3] as usize);
                let (p, q) = (mesh.rest[i], mesh.rest[j]);
                soma += (p[0] - q[0]).hypot(p[1] - q[1]);
                conta += 1;
            }
        }
        let media = if conta == 0 { 1.0 } else { soma / conta as f64 };
        // ⚠️ **Meia aresta**, e o número tem razão: um ponto a mais de meia aresta do eixo tem um
        // vizinho mais perto dele do que do osso, e prendê-lo seria impor a pose do osso a um
        // vértice que a malha já não distingue do interior.
        (media * 0.5) * (media * 0.5)
    };

    // ⭐⭐⭐ **A FOLGA DA JUNTA, em unidades da aresta média.** Ver o doc da função.
    let folga2 = {
        let media = tol2.sqrt() * 2.0;
        (media * folga_da_junta) * (media * folga_da_junta)
    };

    let mut dono: Vec<Option<usize>> = vec![None; n];
    let mut melhor = vec![f64::INFINITY; n];
    // 1ª metade — a banda em torno do eixo, com o mais próximo a ganhar.
    for (j, o) in ossos.iter().enumerate() {
        for (v, &p) in mesh.rest.iter().enumerate() {
            let d = dist2(p, o.a, o.b);
            if d > tol2 || d >= melhor[v] {
                continue;
            }
            // ⛔⛔ **AMBÍGUO ⇒ LIVRE.** Se algum OUTRO osso está a menos da folga deste vértice, ele
            // não é «claramente de ninguém» — e prendê-lo ali é o que transforma a solução numa
            // ESCADA. Ver o doc da função.
            let ambiguo = ossos
                .iter()
                .enumerate()
                .any(|(k, outro)| k != j && dist2(p, outro.a, outro.b) < folga2);
            if ambiguo {
                continue;
            }
            melhor[v] = d;
            dono[v] = Some(j);
        }
    }
    // 2ª metade — todo osso leva pelo menos um.
    for (j, o) in ossos.iter().enumerate() {
        if dono.contains(&Some(j)) {
            continue;
        }
        let mut alvo = (f64::INFINITY, usize::MAX);
        for (v, &p) in mesh.rest.iter().enumerate() {
            let d = dist2(p, o.a, o.b);
            if d < alvo.0 {
                alvo = (d, v);
            }
        }
        if alvo.1 != usize::MAX {
            // ⚠️ Ele pode roubar um vértice a outro osso, e é o correcto: um osso sem sujeito é uma
            // equação sem condição de fronteira, e o vizinho tem outros.
            dono[alvo.1] = Some(j);
        }
    }
    dono
}
