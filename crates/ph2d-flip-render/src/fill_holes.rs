//! Triangulação de um preenchimento **com BURACOS** (W4) — decomposição
//! trapezoidal com a regra **even-odd**.
//!
//! O ear-clipping do [`crate::fill`] serve um polígono SIMPLES (o caso do W1: um
//! traço fechado sem furos). O balde produz outra coisa: um contorno externo mais N
//! anéis internos (a letra "O"), e às vezes um contorno traçado de um raster que se
//! toca a si mesmo. Ear-clipping com furos exige **pontes** — e uma ponte introduz
//! vértices coincidentes que fazem o teste "nenhum vértice dentro da orelha" (que é
//! inclusivo na borda) rejeitar orelhas legítimas até travar. O resultado é um fill
//! com um pedaço faltando, e o bug aparece em UM desenho a cada vinte.
//!
//! Então não usamos pontes. Varremos em **bandas horizontais** entre y's de vértice
//! consecutivos: dentro de uma banda, nenhuma aresta começa nem termina, então cada
//! aresta que a cruza é um segmento reto; ordenando as interseções e pareando-as por
//! **even-odd** sai a lista exata dos trapézios preenchidos. Cada trapézio vira 2
//! triângulos.
//!
//! Propriedades que isso compra:
//! - **exato** — as bordas do polígono são reproduzidas sem aproximação (um trapézio
//!   tem os lados inclinados; não é uma escada);
//! - **robusto** — buracos e auto-interseções são tratados pela mesma regra, sem
//!   casos especiais e sem poder travar;
//! - **determinístico** (HR-5) — nenhuma transcendental, e o desempate da ordenação é
//!   total.
//!
//! Custo: O(bandas × arestas), com o fill computado **uma vez por clique** e cacheado
//! na tesselação por desenho. Um contorno de 500 pontos = ~500 bandas × 500 arestas =
//! 250k operações de ponto-flutuante: microssegundos.

use ph2d_core::Vec2;

/// Uma aresta ativa: os dois extremos, com `y0 < y1`.
struct Edge {
    y0: f32,
    y1: f32,
    x0: f32,
    /// `dx/dy` — o quanto o x anda por unidade de y.
    slope: f32,
}

impl Edge {
    /// O x da aresta na altura `y` (assume `y0 ≤ y ≤ y1`).
    fn x_at(&self, y: f32) -> f32 {
        self.x0 + (y - self.y0) * self.slope
    }
}

/// Constrói as arestas NÃO-horizontais de um anel fechado (as horizontais não
/// contribuem para a paridade de nenhuma banda, e dividir por `dy = 0` seria a única
/// forma de produzir um NaN aqui).
fn edges_of(ring: &[Vec2], out: &mut Vec<Edge>, ys: &mut Vec<f32>) {
    let n = ring.len();
    if n < 3 {
        return;
    }
    for i in 0..n {
        let a = ring[i];
        let b = ring[(i + 1) % n];
        ys.push(a.y);
        if (a.y - b.y).abs() < f32::EPSILON {
            continue; // horizontal: não cruza banda nenhuma
        }
        let (lo, hi) = if a.y < b.y { (a, b) } else { (b, a) };
        out.push(Edge {
            y0: lo.y,
            y1: hi.y,
            x0: lo.x,
            slope: (hi.x - lo.x) / (hi.y - lo.y),
        });
    }
}

/// **Triangula o preenchimento** `outer` menos os anéis `holes`, pela regra even-odd.
/// Devolve as POSIÇÕES dos vértices dos triângulos (`[a,b,c, a,b,c, …]`) — não índices:
/// os cantos dos trapézios são pontos NOVOS, não vértices de entrada.
///
/// Vazio se o contorno externo é degenerado.
#[must_use]
pub fn triangulate_even_odd(outer: &[Vec2], holes: &[Vec<Vec2>]) -> Vec<Vec2> {
    let mut edges: Vec<Edge> = Vec::new();
    let mut ys: Vec<f32> = Vec::new();
    edges_of(outer, &mut edges, &mut ys);
    for h in holes {
        edges_of(h, &mut edges, &mut ys);
    }
    if edges.is_empty() {
        return Vec::new();
    }
    // As fronteiras das bandas: os y's distintos dos vértices. Entre dois y's
    // consecutivos NENHUMA aresta começa ou termina — é o que torna a banda exata.
    ys.sort_by(f32::total_cmp);
    ys.dedup();
    if ys.len() < 2 {
        return Vec::new();
    }

    let mut out: Vec<Vec2> = Vec::new();
    let mut xs: Vec<f32> = Vec::new();
    for w in ys.windows(2) {
        let (y_lo, y_hi) = (w[0], w[1]);
        if (y_hi - y_lo).abs() < f32::EPSILON {
            continue; // banda de altura zero (y's duplicados)
        }
        // Amostra as arestas no MEIO da banda: ali a paridade é inequívoca (nos y's de
        // vértice, uma aresta que só toca o ponto contaria uma vez a mais ou a menos).
        let y_mid = 0.5 * (y_lo + y_hi);
        xs.clear();
        for e in &edges {
            if e.y0 <= y_mid && y_mid < e.y1 {
                xs.push(e.x_at(y_mid));
            }
        }
        if xs.len() < 2 {
            continue;
        }
        xs.sort_by(f32::total_cmp);
        // Even-odd: os pares (0,1), (2,3), … são o INTERIOR. Um buraco inverte a
        // paridade duas vezes ao ser atravessado — e some do preenchimento sozinho.
        for pair in xs.chunks_exact(2) {
            let (xa, xb) = (pair[0], pair[1]);
            if (xb - xa).abs() < f32::EPSILON {
                continue; // span de largura zero (duas arestas no mesmo x)
            }
            // O trapézio: os lados inclinados vêm das MESMAS arestas, avaliadas nos
            // dois y's da banda — então a borda sai exata, não em escada.
            let (la, lb) = (
                x_of_span(&edges, y_mid, xa, y_lo),
                x_of_span(&edges, y_mid, xa, y_hi),
            );
            let (ra, rb) = (
                x_of_span(&edges, y_mid, xb, y_lo),
                x_of_span(&edges, y_mid, xb, y_hi),
            );
            let p0 = Vec2::new(la, y_lo);
            let p1 = Vec2::new(ra, y_lo);
            let p2 = Vec2::new(rb, y_hi);
            let p3 = Vec2::new(lb, y_hi);
            out.extend_from_slice(&[p0, p1, p2, p0, p2, p3]);
        }
    }
    out
}

/// O x, na altura `y`, da aresta que passa por `x_mid` na altura `y_mid`. É como o
/// trapézio ganha lados inclinados: a mesma aresta é reavaliada nos dois y's da banda.
/// (Busca linear pela aresta cujo `x_at(y_mid)` bate — as arestas ativas numa banda
/// são poucas, e a comparação é exata porque `x_mid` VEIO de um `x_at(y_mid)`.)
fn x_of_span(edges: &[Edge], y_mid: f32, x_mid: f32, y: f32) -> f32 {
    for e in edges {
        if e.y0 <= y_mid && y_mid < e.y1 && (e.x_at(y_mid) - x_mid).abs() < f32::EPSILON {
            return e.x_at(y.clamp(e.y0, e.y1));
        }
    }
    x_mid // não achou (não deve acontecer): a banda vira um retângulo
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A área dos triângulos (soma dos |shoelace|/2) — o que se mede para provar que o
    /// preenchimento cobre a região certa.
    fn area(tris: &[Vec2]) -> f32 {
        tris.chunks_exact(3)
            .map(|t| {
                ((t[1].x - t[0].x) * (t[2].y - t[0].y) - (t[1].y - t[0].y) * (t[2].x - t[0].x))
                    .abs()
                    * 0.5
            })
            .sum()
    }

    fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<Vec2> {
        vec![
            Vec2::new(x0, y0),
            Vec2::new(x1, y0),
            Vec2::new(x1, y1),
            Vec2::new(x0, y1),
        ]
    }

    #[test]
    fn a_square_is_filled_exactly() {
        let tris = triangulate_even_odd(&rect(0.0, 0.0, 10.0, 10.0), &[]);
        assert!((area(&tris) - 100.0).abs() < 1e-3, "área = {}", area(&tris));
    }

    /// **O caso que motivou o módulo:** a letra "O". A área do preenchimento é a do
    /// quadrado MENOS a do furo — nenhuma ponte, nenhum vértice inventado.
    #[test]
    fn a_hole_is_subtracted_from_the_fill() {
        let outer = rect(0.0, 0.0, 10.0, 10.0);
        let hole = rect(3.0, 3.0, 7.0, 7.0);
        let tris = triangulate_even_odd(&outer, &[hole]);
        let got = area(&tris);
        assert!(
            (got - (100.0 - 16.0)).abs() < 1e-3,
            "quadrado 100 − furo 16 = 84, veio {got}"
        );
        // E nenhum triângulo cai DENTRO do furo (o centro do furo não é coberto).
        let center = Vec2::new(5.0, 5.0);
        let inside_hole = tris.chunks_exact(3).any(|t| {
            let s =
                |a: Vec2, b: Vec2, p: Vec2| (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
            let (d1, d2, d3) = (
                s(t[0], t[1], center),
                s(t[1], t[2], center),
                s(t[2], t[0], center),
            );
            (d1 > 1e-4 && d2 > 1e-4 && d3 > 1e-4) || (d1 < -1e-4 && d2 < -1e-4 && d3 < -1e-4)
        });
        assert!(!inside_hole, "o miolo do furo foi preenchido");
    }

    /// Dois furos, e um deles encostado na borda de uma banda — a paridade tem de
    /// aguentar (é amostrada no MEIO da banda, justamente por isso).
    #[test]
    fn several_holes_work_and_bands_align_with_vertices() {
        let outer = rect(0.0, 0.0, 20.0, 10.0);
        let holes = vec![rect(2.0, 2.0, 6.0, 8.0), rect(12.0, 0.0, 16.0, 4.0)];
        let tris = triangulate_even_odd(&outer, &holes);
        let expect = 200.0 - 24.0 - 16.0;
        assert!(
            (area(&tris) - expect).abs() < 1e-3,
            "esperado {expect}, veio {}",
            area(&tris)
        );
    }

    /// Um contorno inclinado sai EXATO (o trapézio tem lados inclinados — a área de um
    /// triângulo é a do triângulo, não a de uma escada de retângulos).
    #[test]
    fn a_slanted_edge_is_exact_not_a_staircase() {
        let tri = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(0.0, 10.0),
        ];
        let tris = triangulate_even_odd(&tri, &[]);
        assert!(
            (area(&tris) - 50.0).abs() < 1e-3,
            "triângulo de área 50, veio {}",
            area(&tris)
        );
    }

    #[test]
    fn degenerate_input_is_empty() {
        assert!(triangulate_even_odd(&[], &[]).is_empty());
        assert!(triangulate_even_odd(&[Vec2::new(0.0, 0.0)], &[]).is_empty());
        // Um "polígono" achatado (todos os pontos na mesma linha) não tem área.
        let flat = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(10.0, 0.0),
        ];
        assert!(area(&triangulate_even_odd(&flat, &[])) < 1e-4);
    }

    /// **Uma CONCAVIDADE profunda não é preenchida por cima** (o "V" do smoke do Enio,
    /// 2026-07-13 — a suspeita que a medição inocentou).
    ///
    /// Um entalhe em "V" produz QUATRO cruzamentos numa scanline: uma decomposição que
    /// pegasse só o primeiro e o último preencheria o vão. Esta cobre — e o buraco de
    /// teste era real: nenhum caso côncavo existia aqui.
    #[test]
    fn a_deep_notch_is_not_filled_over() {
        // Um retângulo com um entalhe em "V" descendo do topo até o meio.
        let poly = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(6.0, 10.0),
            Vec2::new(5.0, 3.0), // o bico do V (fundo do entalhe)
            Vec2::new(4.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        // Área real (shoelace):
        let n = poly.len();
        let real: f32 = (0..n)
            .map(|i| {
                let (a, b) = (poly[i], poly[(i + 1) % n]);
                a.x * b.y - b.x * a.y
            })
            .sum::<f32>()
            .abs()
            * 0.5;
        let tris = triangulate_even_odd(&poly, &[]);
        let got = area(&tris);
        println!(
            "area REAL {real:.2} | triangulada {got:.2} | diferenca {:.2}",
            got - real
        );
        assert!(
            (got - real).abs() < 0.5,
            "a triangulacao PREENCHEU O ENTALHE: {got:.2} != {real:.2}"
        );
    }
}
