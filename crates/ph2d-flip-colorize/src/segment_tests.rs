//! Gates da partição trapped-ball (`docs/Flip/09 §8`).
//!
//! O load-bearing é `the_ball_seams_a_gap_narrower_than_its_diameter`: a costura é o que dá
//! ao Colorize a promessa "atravessa vãos até `2·trap_px` e não mais". (O gate da identidade
//! de corte `a_region_cut_weighs_exactly_what_the_pixel_cut_weighs` morreu com o grafo de
//! arestas no 4º smoke, 2026-07-20 — o solver contestado virou Voronoi por pixel e não
//! consome grafo nenhum; ver o topo de `segment.rs`.)

use super::{NO_REGION, segment};
use ph2d_core::Vec2;
use ph2d_flip_fill::Grid;

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

/// **A costura do vão** — a promessa do Trap. Uma bola mais larga que o vão não o atravessa,
/// então os dois lados caem em componentes DIFERENTES (cada um é preenchido com a sua cor,
/// colada na linha). Uma bola que cabe no vão passa, e os dois lados são o MESMO componente —
/// nesse caso a disputa é do Voronoi, e é honesto que seja.
#[test]
fn the_ball_seams_a_gap_narrower_than_its_diameter() {
    let g = boxed(64.0, 10.0);
    let mid = 32.0;
    let (left, right) = (px(&g, 20.0, mid), px(&g, 44.0, mid));

    // Bola de raio 8 (diâmetro 16 > vão 10): não passa ⇒ costura ⇒ componentes diferentes.
    let seamed = segment(&g, 8.0);
    assert_ne!(
        seamed.component[left], seamed.component[right],
        "a bola larga tem de costurar o vao"
    );

    // Bola de raio 2 (diâmetro 4 < vão 10): passa ⇒ MESMO componente.
    let open = segment(&g, 2.0);
    assert_eq!(
        open.component[left], open.component[right],
        "a bola que cabe no vao NAO costura — e e' honesto que nao costure"
    );
}

/// Toda a grade é atribuída: papel, papel fino e **a tinta**. A tinta importa — é ela que
/// deixa o preenchimento de um componente de uma cor só cobrir a arte até a linha (o
/// `expand_under_ink` do traçado precisa de dono dos dois lados).
#[test]
fn every_pixel_lands_in_a_component_including_the_ink() {
    let g = boxed(64.0, 10.0);
    let seg = segment(&g, 5.0);
    assert!(
        seg.component.iter().all(|&r| r != NO_REGION),
        "sobrou pixel sem componente"
    );
}

/// Bola grande demais para a arte: a resposta honesta é o papel inteiro (o comportamento de
/// antes da trapped-ball), **nunca** "sem componentes" — que apagaria o Colorize em silêncio.
#[test]
fn an_oversized_ball_falls_back_to_the_whole_paper() {
    let g = boxed(64.0, 10.0);
    let seg = segment(&g, 500.0);
    assert!(
        seg.count >= 1,
        "a bola gigante nao pode zerar os componentes"
    );
    assert!(seg.component.iter().all(|&r| r != NO_REGION));
}

/// Determinismo (HR-5): dois cliques idênticos dão o mesmo desenho, incluindo os IDs.
#[test]
fn the_partition_is_deterministic() {
    let g = boxed(64.0, 10.0);
    let a = segment(&g, 5.0);
    let b = segment(&g, 5.0);
    assert_eq!(a.count, b.count);
    assert_eq!(a.component, b.component);
}
