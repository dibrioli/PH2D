//! Gates da pré-segmentação (`docs/Flip/09 §8`).
//!
//! O load-bearing é `a_region_cut_weighs_exactly_what_the_pixel_cut_weighs`: ele é a
//! afirmação de CORREÇÃO da wave inteira — se ela vale, trocar a grade pelo grafo de regiões
//! é uma restrição do mesmo problema, e o corte que sai é um corte LazyBrush de verdade, só
//! que achado em milissegundos. Se ela não vale, o motor é rápido resolvendo outra coisa.

use super::{NO_REGION, segment};
use ph2d_core::Vec2;
use ph2d_flip_fill::{BOUNDARY, Grid};

const V_WHITE: i32 = 8;
const V_INK: i32 = 1; // > 0 aqui de propósito: com 0 a metade "através da tinta" some.

/// Uma grade em que **1 unidade de mundo = 1 pixel** (`scale = 1`), para os números do gate
/// serem os números do fenômeno e não uma conversão.
fn grid_px(side: f32) -> Grid {
    Grid::new(Vec2::new(0.0, 0.0), Vec2::new(side, side), 1.0, 4, 4096)
}

/// Caixa fechada `[8, side-8]²` com um divisor vertical no meio, partido por um vão de
/// `gap` unidades — a arte que o Colorize existe para colorir.
fn boxed(side: f32, gap: f32) -> Grid {
    let mut g = grid_px(side);
    let (a, b) = (8.0, side - 8.0);
    let mid = side * 0.5;
    for (p, q) in [
        (Vec2::new(a, a), Vec2::new(b, a)),
        (Vec2::new(b, a), Vec2::new(b, b)),
        (Vec2::new(b, b), Vec2::new(a, b)),
        (Vec2::new(a, b), Vec2::new(a, a)),
        // O divisor, com o vão centrado.
        (Vec2::new(mid, a), Vec2::new(mid, mid - gap * 0.5)),
        (Vec2::new(mid, mid + gap * 0.5), Vec2::new(mid, b)),
    ] {
        g.stroke_capsule(p, q, 0.0);
    }
    g
}

fn px(g: &Grid, x: f32, y: f32) -> usize {
    let (ix, iy) = g.pixel_of(Vec2::new(x, y)).expect("ponto dentro da grade");
    iy * g.w + ix
}

/// **A afirmação de correção da wave.** Para uma partição QUALQUER das regiões em dois
/// lados, o peso do corte lido no grafo de regiões é igual, ao inteiro, ao peso do mesmo
/// corte somado pixel a pixel na grade. As duas contas são independentes — a segunda não
/// olha `edges`, ela reconstrói o `V_pq` de cada par adjacente.
#[test]
fn a_region_cut_weighs_exactly_what_the_pixel_cut_weighs() {
    let g = boxed(64.0, 10.0);
    // Raio 8 (diâmetro 16 > vão 10) ⇒ a bola COSTURA o vão e a arte rende ≥ 3 regiões
    // (esquerda, direita, fora). Com raio 3 ela atravessa e sobram 2 — a identidade valeria
    // ali também, mas sobre uma partição sem a costura que a wave existe para produzir: a
    // fixture tem de conter o fenômeno.
    let seg = segment(&g, 8.0, V_WHITE, V_INK);
    assert!(
        seg.count >= 3,
        "a arte tem de render >= 3 regioes, deu {}",
        seg.count
    );

    // Varre várias bipartições (inclusive as degeneradas) — uma só poderia estar certa por
    // acidente de simetria.
    for mask in 0u32..(1u32 << seg.count.min(6)) {
        let side_of = |r: u32| mask >> r & 1 == 1;

        // (a) pelo grafo de regiões.
        let by_graph: i64 = seg
            .edges
            .iter()
            .filter(|&&(a, b, _)| side_of(a) != side_of(b))
            .map(|&(_, _, c)| i64::from(c))
            .sum();

        // (b) pixel a pixel, re-enunciando a lei — sem consultar `edges`.
        let is_ink = |i: usize| g.flags[i] & BOUNDARY != 0;
        let mut by_pixels: i64 = 0;
        for i in 0..g.w * g.h {
            let ri = seg.region[i];
            if ri == NO_REGION {
                continue;
            }
            for q in [
                (i % g.w + 1 < g.w).then_some(i + 1),
                (i / g.w + 1 < g.h).then_some(i + g.w),
            ]
            .into_iter()
            .flatten()
            {
                let rq = seg.region[q];
                if rq == NO_REGION || rq == ri || side_of(ri) == side_of(rq) {
                    continue;
                }
                by_pixels += i64::from(if is_ink(i) || is_ink(q) {
                    V_INK
                } else {
                    V_WHITE
                });
            }
        }
        assert_eq!(
            by_graph, by_pixels,
            "o corte de regioes tem de pesar o corte de pixels (mask {mask:b})"
        );
    }
}

/// **A costura do vão** — a propriedade que dá ao corte por onde passar. Uma bola mais larga
/// que o vão não o atravessa, então os dois lados caem em regiões DIFERENTES e o corte pode
/// separá-los pagando papel. Uma bola que cabe no vão passa, e os dois lados são a MESMA
/// região — nesse caso nada, nem o LazyBrush, separa as cores por ali.
#[test]
fn the_ball_seams_a_gap_narrower_than_its_diameter() {
    let g = boxed(64.0, 10.0);
    let mid = 32.0;
    let (left, right) = (px(&g, 20.0, mid), px(&g, 44.0, mid));

    // Bola de raio 8 (diâmetro 16 > vão 10): não passa ⇒ costura.
    let seamed = segment(&g, 8.0, V_WHITE, V_INK);
    assert_ne!(
        seamed.region[left], seamed.region[right],
        "a bola larga tem de costurar o vao"
    );

    // Bola de raio 2 (diâmetro 4 < vão 10): passa ⇒ um lado só.
    let open = segment(&g, 2.0, V_WHITE, V_INK);
    assert_eq!(
        open.region[left], open.region[right],
        "a bola que cabe no vao NAO costura — e e' honesto que nao costure"
    );
}

/// Toda a grade é atribuída: papel, papel fino e **a tinta**. A tinta importa — sem ela duas
/// regiões separadas por uma linha grossa não seriam adjacentes em lugar nenhum, e a aresta
/// barata que o corte precisa não existiria.
#[test]
fn every_pixel_lands_in_a_region_including_the_ink() {
    let g = boxed(64.0, 10.0);
    let seg = segment(&g, 5.0, V_WHITE, V_INK);
    assert!(
        seg.region.iter().all(|&r| r != NO_REGION),
        "sobrou pixel sem regiao"
    );
    // E existe ao menos uma aresta cujo peso denuncia travessia de TINTA (múltiplo de
    // V_INK e não de V_WHITE seria frágil; basta que a soma total não seja divisível por
    // V_WHITE, o que só acontece se pares de tinta contribuíram).
    let total: i64 = seg.edges.iter().map(|&(_, _, c)| i64::from(c)).sum();
    assert!(
        total % i64::from(V_WHITE) != 0,
        "nenhuma fronteira atravessou tinta — a linha nao esta no grafo"
    );
}

/// Bola grande demais para a arte: a resposta honesta é o papel inteiro (o comportamento de
/// antes da trapped-ball), **nunca** "sem regiões" — que apagaria o Colorize em silêncio.
#[test]
fn an_oversized_ball_falls_back_to_the_whole_paper() {
    let g = boxed(64.0, 10.0);
    let seg = segment(&g, 500.0, V_WHITE, V_INK);
    assert!(seg.count >= 1, "a bola gigante nao pode zerar as regioes");
    assert!(seg.region.iter().all(|&r| r != NO_REGION));
}

/// Determinismo (HR-5): dois cliques idênticos dão o mesmo desenho, incluindo os IDs.
#[test]
fn the_partition_is_deterministic() {
    let g = boxed(64.0, 10.0);
    let a = segment(&g, 5.0, V_WHITE, V_INK);
    let b = segment(&g, 5.0, V_WHITE, V_INK);
    assert_eq!(a.count, b.count);
    assert_eq!(a.region, b.region);
    assert_eq!(a.edges, b.edges);
}

/// O `K` das sementes tem de dominar o corte mais caro que poderia trair um rabisco — a
/// soma das arestas que tocam a região semeada.
#[test]
fn the_seed_weight_dominates_any_incident_cut() {
    let g = boxed(64.0, 10.0);
    let seg = segment(&g, 5.0, V_WHITE, V_INK);
    let k = seg.seed_weight();
    for r in 0..seg.count as u32 {
        let inc: i64 = seg
            .edges
            .iter()
            .filter(|&&(a, b, _)| a == r || b == r)
            .map(|&(_, _, c)| i64::from(c))
            .sum();
        assert!(
            i64::from(k) > inc,
            "K={k} nao domina o corte incidente da regiao {r} ({inc})"
        );
    }
}
