//! ⭐⭐ **A aritmética do ENROLAMENTO** — irmã do [`super`] por responsabilidade (tecto de LOC,
//! 2026-09-16): ali mora *o índice, o corte e as consultas*; aqui *as regras de dentro/fora que eles
//! perguntam* — o raio `+x`, o atravessamento de um caminho e a partida segura de cada célula.
//!
//! ⚠️ As três têm de concordar sobre um ponto que assenta numa aresta, e é por isso que vivem
//! juntas: foi a discordância entre as duas primeiras, num canto de célula sobre uma corda, que pôs
//! o sinal errado numa célula inteira (ver o campo `start` do índice).

use super::{Edge, seg_dist2};

/// ⭐ **Um ponto da célula longe de toda aresta que a toca** — a partida do caminho do sinal.
///
/// ⚠️ A mesma lei da âncora da árvore por região: os cantos, o centro e as medianas, por esta
/// ordem, e a barra é uma FRACÇÃO do tamanho da célula. Se nenhum servir (célula absurdamente
/// povoada), fica o canto — que é o que havia antes.
pub(super) fn partida_segura(
    edges: &[Edge],
    lista: &[u32],
    lo: [f32; 2],
    hi: [f32; 2],
) -> [f32; 2] {
    let span = (hi[0] - lo[0]).max(hi[1] - lo[1]).max(f32::MIN_POSITIVE);
    let bar = (span * 1.0e-3).powi(2);
    let mid = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    [
        lo,
        mid,
        [hi[0], lo[1]],
        [lo[0], hi[1]],
        hi,
        [mid[0], lo[1]],
        [lo[0], mid[1]],
    ]
    .into_iter()
    .find(|p| {
        lista
            .iter()
            .all(|i| seg_dist2(*p, &edges[*i as usize]) > bar)
    })
    .unwrap_or(lo)
}

/// O enrolamento no ponto, pela **mesma** regra do raio `+x` que a árvore usa
/// ([`crate::profile`]) — é ela que decide o que é dentro, e uma segunda regra aqui daria duas
/// respostas à mesma pergunta.
pub(super) fn ray_winding(edges: &[Edge], p: [f32; 2]) -> i32 {
    let mut w = 0;
    for e in edges {
        let above_a = i32::from(e.a[1] > p[1]);
        let above_b = i32::from(e.b[1] > p[1]);
        let dir = above_b - above_a;
        if dir == 0 {
            continue;
        }
        let cross = e.e[0] * (p[1] - e.a[1]) - e.e[1] * (p[0] - e.a[0]);
        if (dir as f32) * cross > 0.0 {
            w += dir;
        }
    }
    w
}

/// ⭐ **Quantas vezes (com sinal) a aresta atravessa o caminho `c → p`.**
///
/// ⚠️ É a mesma grandeza do [`ray_winding`], escrita como **diferença ao longo de um caminho** — e
/// é isso que a torna pré-computável: `w(p) = w(c) + Σ atravessamentos`. O sinal segue a convenção
/// do raio: uma aresta que sobe conta `+1` quando o caminho a cruza deixando-a à esquerda.
pub(super) fn path_crossing(c: [f32; 2], p: [f32; 2], e: &Edge) -> i32 {
    // ⭐⭐ **A regra é SEMIABERTA, e não simétrica** — `< 0` de um lado, `>= 0` do outro.
    //
    // ⛔ **Defeito medido (W56):** com o teste simétrico (`d1·d2 < 0`), um caminho que passa **por um
    // vértice** do contorno é contado **zero** vezes — as duas arestas que o partilham vêem produto
    // nulo e ambas desistem. O enrolamento sai errado por um numa cunha fina à volta daquele vértice,
    // o sinal inverte-se lá, e a esfera-marcha inventa uma superfície. Um quadro de 240×180 com 168
    // arestas apanhou-o num pixel — de ~800 mil amostras.
    //
    // ⚠️ É a **mesma** disciplina que o raio `+x` do [`ray_winding`] já segue (a variante semi-aberta
    // do *crossing number*, de Dan Sunday): um vértice pertence a exactamente uma das suas duas
    // arestas. *Uma regra de fronteira escrita duas vezes tem de ser a mesma nas duas.*
    let d1 = orient(e.a, e.b, c);
    let d2 = orient(e.a, e.b, p);
    let d3 = orient(c, p, e.a);
    let d4 = orient(c, p, e.b);
    if (d1 < 0.0) == (d2 < 0.0) || (d3 < 0.0) == (d4 < 0.0) {
        return 0;
    }
    // De que lado a aresta atravessa o caminho: o sinal do produto vetorial das duas direções.
    let path = [p[0] - c[0], p[1] - c[1]];
    let s = path[0] * e.e[1] - path[1] * e.e[0];
    if s > 0.0 { -1 } else { 1 }
}

pub(super) fn orient(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> f32 {
    (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
}
