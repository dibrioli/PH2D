//! **O papel caminhado por LINHAS é o papel em série, ao bit** (ADR-0175 §3-bis).
//!
//! Duas portas do nascimento de uma sessão varriam a grade inteira numa thread: o bake do tile do
//! motor (`18 ms` a 4096²) e a semente do papel do HOSPEDEIRO (`95–277 ms` com um papel do artista —
//! 16,7 M chamadas a uma lei de textura). As duas são funções puras da célula, e cada linha escreve só
//! a própria linha. O oráculo é a rota em série, que é o laço de antes letra por letra.

use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::paper::{PaperKnobs, PaperPreset, bake_paper_rows, generate_paper_tile};
use ph2d_wet_paint::par::Rows;

/// Grande o bastante para o paralelo partir a grade em muitas linhas, e com largura que não é
/// potência de dois (a linha do tile dá a volta a meio).
const W: usize = 700;
const H: usize = 530;

#[test]
fn the_paper_bake_by_rows_is_the_serial_bake() {
    let tile = generate_paper_tile(PaperPreset::Cold, 1, PaperKnobs::default());
    let bake = |mode: Rows| {
        let mut e = Engine::new(W, H);
        bake_paper_rows(e.active_grid_mut(), &tile, mode);
        e.active_grid().paper.clone()
    };
    let serie = bake(Rows::Serial);
    let paralelo = bake(Rows::Parallel);
    assert!(
        serie.iter().any(|&v| v != serie[0]),
        "a fixtura tem de ter um papel com relevo"
    );
    assert!(
        serie
            .iter()
            .zip(&paralelo)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
        "o bake por linhas mudou o papel"
    );
    // E o ORÁCULO independente: o laço de ANTES, congelado aqui. As duas rotas partilham o corpo por
    // linha, logo compará-las uma com a outra não apanha um corpo errado. ⚠️ O tile é o da folha `1` e
    // o motor nasce com o da `0`: uma linha que o bake saltasse ficava com o papel do construtor, e é
    // isso que a torna visível aqui.
    let ts = ph2d_wet_paint::paper::TILE_SIZE as i64;
    let s = W + 2;
    for y in 0..H + 2 {
        let ty = (y as i64 - 1).rem_euclid(ts) * ts;
        for x in 0..s {
            let want = tile[(ty + (x as i64 - 1).rem_euclid(ts)) as usize];
            assert_eq!(
                paralelo[y * s + x].to_bits(),
                want.to_bits(),
                "o bake por linhas pôs outro texel do tile na célula ({x}, {y})"
            );
        }
    }
}

/// **A semente fica na faixa do dente** — uma lei do hospedeiro que sai de `0..1` é cortada, como o
/// laço de antes cortava.
#[test]
fn the_hosts_paper_seed_is_clamped_to_the_tooth_range() {
    let mut e = Engine::new(40, 30);
    e.seed_paper_with_rows(&|x, _| if x % 2 == 0 { 1.7 } else { -0.4 }, Rows::Parallel);
    let g = e.active_grid();
    assert!(
        g.paper.iter().all(|&v| v == 0.0 || v == 1.0),
        "a semente deixou um dente fora de 0..1"
    );
    assert!(
        g.paper.contains(&0.0) && g.paper.contains(&1.0),
        "a fixtura tem de exercitar os dois lados do corte"
    );
}

#[test]
fn the_hosts_paper_seed_by_rows_is_the_serial_seed() {
    // Uma lei que depende das DUAS coordenadas de modo que troca de linha ou de coluna se veja.
    let lei = |x: i64, y: i64| -> f64 { ((x * 31 + y * 17).rem_euclid(97)) as f64 / 96.0 };
    let semeia = |mode: Rows| {
        let mut e = Engine::new(W, H);
        e.seed_paper_with_rows(&lei, mode);
        e.active_grid().paper.clone()
    };
    let serie = semeia(Rows::Serial);
    let paralelo = semeia(Rows::Parallel);
    assert!(
        serie
            .iter()
            .zip(&paralelo)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
        "a semente por linhas mudou o papel"
    );
    // E a semente cobre o plano inteiro, com o anel de borda: a célula (0, 0) pergunta a (−1, −1).
    let g_s = W + 2;
    assert_eq!(serie[0], lei(-1, -1) as f32, "a célula do anel");
    assert_eq!(
        serie[g_s + 1],
        lei(0, 0) as f32,
        "a primeira célula de dentro"
    );
}
