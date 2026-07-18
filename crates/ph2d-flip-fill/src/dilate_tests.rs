//! Gates da DILATAÇÃO. Cada um pina um defeito que já aconteceu de verdade — os três
//! primeiros são BUGS #20 (a dilatação 100× grande, e a média global), o quarto é a
//! lição do BUGS #18 (vértice contra segmento).

use super::{FILL_TUCK_PX, contour_widths, local_line, mean_line_width, tuck_world};
use crate::Vec2;

/// Uma linha reta horizontal em `y`, com espessura CHEIA `w`, amostrada em `n` pontos
/// de `x0` a `x1`. (A lista fala MEIA espessura — a convenção do `fill_at`.)
fn line(y: f32, w: f32, x0: f32, x1: f32, n: usize) -> (Vec<Vec2>, Vec<f32>, bool) {
    let pts: Vec<Vec2> = (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            Vec2::new(x0 + (x1 - x0) * t, y)
        })
        .collect();
    (pts, vec![w * 0.5; n], false)
}

/// **O contorno veste a linha que ABRAÇA, não a média do desenho.**
///
/// Este é o BUGS #20: com uma linha grossa e uma fina no mesmo desenho, a média fica
/// entre as duas, então onde o contorno abraça a FINA a cor era desenhada larga demais
/// e aparecia do outro lado dela. O sintoma na tela era a cor atravessando a linha fina.
///
/// Mutação que ele mata: trocar `local_line` pela média (o código de antes) — os dois
/// pontos passam a valer 7,0.
#[test]
fn the_contour_wears_the_line_it_hugs_not_the_average() {
    let strokes = vec![
        line(0.0, 10.0, -50.0, 50.0, 8),  // grossa
        line(100.0, 2.0, -50.0, 50.0, 8), // fina
    ];
    // Dois pontos de contorno: um colado na grossa, outro colado na fina.
    let contour = [Vec2::new(0.0, 1.0), Vec2::new(0.0, 99.0)];
    let w = contour_widths(&strokes, &contour, 1.0);

    let tuck = tuck_world(1.0);
    assert!(
        (w[0] - (10.0 + tuck)).abs() < 1e-4,
        "junto da linha GROSSA a dilatacao tem de ser 10 + margem, veio {}",
        w[0]
    );
    assert!(
        (w[1] - (2.0 + tuck)).abs() < 1e-4,
        "junto da linha FINA a dilatacao tem de ser 2 + margem, veio {} \
         (6 + margem = a MEDIA, que e o bug)",
        w[1]
    );
    // E a média, que era a resposta antiga, é de fato diferente das duas — sem isto o
    // gate acima passaria mesmo num desenho onde média e local coincidem.
    let mean = mean_line_width(&strokes);
    assert!(
        (mean - 6.0).abs() < 1e-4,
        "premissa do gate: a media tem de ser 6 (diferente de 10 e de 2), veio {mean}"
    );
}

/// **A margem é uma medida de TELA e tem de atravessar a conversão.**
///
/// O BUGS #20 em estado puro: `FILL_TUCK_PX` é medido em pixels (a tabela do sweep está
/// em px) e estava sendo somado direto a uma largura em unidades de MUNDO. Com
/// `SIZE_PX_PER_WORLD = 100`, isso é **100× grande demais** — num pincel default
/// (~0,06 de mundo) a margem ficava 17× mais larga que a própria linha, e a cor virava
/// um blob que ignorava o line-art.
///
/// Mutação que ele mata: devolver `2.0 * FILL_TUCK_PX` ignorando o `px_per_world`.
#[test]
fn the_margin_is_a_screen_measure_that_crosses_into_the_documents_unit() {
    // Num documento cujo mundo JÁ é pixel, a margem é ela mesma (dobrada: é diâmetro).
    assert!((tuck_world(1.0) - 2.0 * FILL_TUCK_PX).abs() < 1e-9);
    // No produto (100 px por unidade de mundo) ela vale 1 px = 0,01 de mundo.
    assert!(
        (tuck_world(100.0) - 0.01).abs() < 1e-9,
        "1 px de margem em 100 px/unidade tem de ser 0,01 de MUNDO, veio {}",
        tuck_world(100.0)
    );
    // E o teto: a margem NUNCA pode ser da ordem de uma linha de pincel default
    // (~0,06 de mundo). É a asserção que o produto violava por 17×.
    assert!(
        tuck_world(100.0) < 0.06 * 0.25,
        "a margem ({}) esta da ordem da propria linha — e o BUGS #20 de volta",
        tuck_world(100.0)
    );
}

/// **A distância é ao SEGMENTO, nunca ao VÉRTICE** (BUGS #18).
///
/// Um ponto do contorno pousado exatamente sobre o eixo, mas no MEIO de dois vértices,
/// está a distância zero da linha — e a até meia-amostragem do vértice mais próximo.
/// Medir ao vértice faria a compensação pagar o **espaçamento da amostragem** como se
/// fosse erro de vetorização.
///
/// Mutação que ele mata: `d = min|p − vértice|` — a distância vira 50.
#[test]
fn a_point_between_two_vertices_measures_to_the_segment() {
    // Uma linha de DOIS pontos só: (0,0) → (100,0). O meio está a 50 de cada vértice.
    let strokes = vec![(
        vec![Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0)],
        vec![3.0, 3.0],
        false,
    )];
    let (w, d) = local_line(&strokes, Vec2::new(50.0, 0.0)).expect("acha a linha");
    assert!(
        d < 1e-4,
        "sobre o eixo, no meio do segmento, a distancia e ZERO — veio {d} \
         (50 = a distancia ao VERTICE, que e o bug)"
    );
    assert!(
        (w - 6.0).abs() < 1e-4,
        "a espessura CHEIA e o dobro da meia"
    );
}

/// **Um traço fechado inclui a COSTURA** — o espelho de `FlipStroke::segments()`.
///
/// Sem a costura, o trecho entre o último ponto e o primeiro é invisível para a
/// dilatação: o contorno que o abraça não acha linha nenhuma e cai no fallback da média.
/// Num quadrado, é uma aresta inteira.
///
/// Mutação que ele mata: tirar o `.chain(seam...)` — a distância salta para o vértice.
#[test]
fn a_closed_line_is_walked_through_its_seam() {
    // Quadrado: a costura é a aresta de (0,100) de volta a (0,0).
    let sq = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        Vec2::new(100.0, 100.0),
        Vec2::new(0.0, 100.0),
    ];
    let strokes = vec![(sq, vec![2.0; 4], true)];
    // Um ponto colado NO MEIO da aresta da costura (x = 0, y = 50).
    let (_, d) = local_line(&strokes, Vec2::new(1.0, 50.0)).expect("acha a linha");
    assert!(
        d < 1.5,
        "o ponto esta a 1 unidade da aresta da COSTURA — veio {d} \
         (~50 = a costura nao foi percorrida)"
    );
}

/// **Um fechamento de gap nunca veste o contorno.**
///
/// Ele entra na lista de fronteiras (é para isso que existe: barrar o vazamento), mas
/// tem espessura ZERO — não é tinta. Se ele pudesse vencer o `local_line`, o contorno
/// que passa por um vão fechado seria desenhado com largura zero e a cor sumiria ali.
///
/// Mutação que ele mata: tirar o `w > 0.0` — a largura vira 0 + margem.
#[test]
fn a_zero_width_closure_never_dresses_the_contour() {
    let strokes = vec![
        // O fechamento, EM CIMA do ponto perguntado (distância 0 — vence qualquer um).
        (
            vec![Vec2::new(-10.0, 0.0), Vec2::new(10.0, 0.0)],
            vec![0.0, 0.0],
            false,
        ),
        // A linha de verdade, mais longe.
        line(20.0, 8.0, -50.0, 50.0, 4),
    ];
    let (w, _) = local_line(&strokes, Vec2::new(0.0, 0.0)).expect("acha a LINHA, nao o fechamento");
    assert!(
        (w - 8.0).abs() < 1e-4,
        "o fechamento tem espessura zero e nao veste nada; esperado 8, veio {w}"
    );
}
