//! Gates da DILATAÇÃO. Cada um pina um defeito que já aconteceu de verdade — os três
//! primeiros são BUGS #20 (a dilatação 100× grande, e a média global), o quarto é a
//! lição do BUGS #18 (vértice contra segmento).

use super::{
    FILL_TUCK_PX, contour_widths, contour_widths_with_margin, local_line, mean_line_width,
    tuck_world,
};
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

    // ⚠️ **A afirmação é a RELAÇÃO, não um literal.** A 1ª versão deste gate cravava
    // `tuck_world(100) == 0,01`, o que embutia em silêncio o valor de `FILL_TUCK_PX` —
    // e ele morreu na primeira vez que a varredura escolheu outra margem, denunciando um
    // bug que não existia. Um gate de UNIDADE tem de sobreviver a uma mudança de VALOR:
    // são perguntas diferentes, e a do valor tem a sua própria tabela no doc da
    // constante.
    assert!(
        (tuck_world(100.0) * 100.0 - tuck_world(1.0)).abs() < 1e-9,
        "a margem tem de ESCALAR com px_per_world: {} a 100 px/unidade contra {} a 1",
        tuck_world(100.0),
        tuck_world(1.0)
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

/// **A normal externa aponta para FORA — nas duas orientações do anel.**
///
/// O sinal da compensação inteira pende disto. E a constante certa não é a do livro de
/// geometria: este documento tem o **y para BAIXO**, então "área positiva = anti-horário"
/// se inverte. É por isso que a orientação é medida num círculo, e não deduzida.
///
/// Mutação que ele mata: trocar o sinal do `orient` — a normal passa a apontar para o
/// centro e a compensação inteira inverte (o defeito vira o dobro dele).
#[test]
fn the_outward_normal_points_away_from_the_ring() {
    let n = 64;
    let ring: Vec<Vec2> = (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            Vec2::new(50.0 + 20.0 * a.cos(), 50.0 + 20.0 * a.sin())
        })
        .collect();
    for (name, r) in [
        ("anel numa orientacao", ring.clone()),
        (
            "anel na orientacao OPOSTA",
            ring.iter().rev().copied().collect(),
        ),
    ] {
        let normals = super::outward_normals(&r);
        for (i, p) in r.iter().enumerate() {
            // Do centro para o ponto: a normal externa tem de concordar com isso.
            let radial = Vec2::new(p.x - 50.0, p.y - 50.0);
            let dot = radial.x * normals[i].x + radial.y * normals[i].y;
            assert!(
                dot > 0.0,
                "{name}: no ponto {i} a normal aponta para DENTRO (dot {dot}) — \
                 a compensacao inteira sai com o sinal trocado"
            );
        }
    }
}

/// **O contorno que ficou AQUÉM engorda; o que PASSOU do eixo encolhe.**
///
/// É a espinha da lei, e o defeito que ela conserta: a versão anterior (`w + 2d`, sem
/// sinal) engordava os DOIS — corrigia metade dos pontos e dobrava o erro na outra
/// metade, e por isso mediu PIOR que uma margem uniforme (0,0178 contra 0,005).
///
/// Mutação que ele mata: `s.abs()` em vez de `s`, ou o sinal trocado no `orient`.
#[test]
fn a_contour_short_of_the_axis_widens_and_one_past_it_narrows() {
    // Eixo: um círculo de raio 20 (a "linha"), espessura cheia 4.
    let n = 96;
    let axis: Vec<Vec2> = (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            Vec2::new(50.0 + 20.0 * a.cos(), 50.0 + 20.0 * a.sin())
        })
        .collect();
    let strokes = vec![(axis, vec![2.0; n], true)];

    // Três contornos concêntricos: aquém do eixo, EM CIMA dele, e além.
    let ring_at = |r: f32| -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let a = i as f32 / n as f32 * std::f32::consts::TAU;
                Vec2::new(50.0 + r * a.cos(), 50.0 + r * a.sin())
            })
            .collect()
    };
    let med = |v: &[f32]| {
        let mut v = v.to_vec();
        v.sort_by(f32::total_cmp);
        v[v.len() / 2]
    };

    // Margem zero: aqui se mede a COMPENSAÇÃO, não a margem. E os valores são EXATOS,
    // não ordenações frouxas — a lei é `w + 2s`, então cada número é verificável de
    // cabeça. (O alisamento não os move: binomial de uma constante é a constante.)
    let short = med(&contour_widths_with_margin(
        &strokes,
        &ring_at(17.0),
        1.0,
        0.0,
    ));
    let on = med(&contour_widths_with_margin(
        &strokes,
        &ring_at(20.0),
        1.0,
        0.0,
    ));
    let past = med(&contour_widths_with_margin(
        &strokes,
        &ring_at(21.0),
        1.0,
        0.0,
    ));

    // Em cima do eixo a lei não faz nada: a largura é a da linha, e nem um pixel a mais.
    assert!(
        (on - 4.0).abs() < 0.3,
        "sobre o eixo a dilatacao e a propria linha (4,0); veio {on}"
    );
    // Aquém por 3: precisa alcançar 3 a mais, dos DOIS lados => 4 + 6 = 10.
    assert!(
        (short - 10.0).abs() < 0.3,
        "aquem do eixo por 3, a largura tem de ser 4 + 2*3 = 10; veio {short}"
    );
    // Além por 1: encolhe 2 => 4 − 2 = 2. É ESTE o ponto que a versão sem sinal errava,
    // engordando para 6 e empurrando a cor para ainda mais longe da linha.
    assert!(
        (past - 2.0).abs() < 0.3,
        "alem do eixo por 1, a largura tem de ser 4 - 2*1 = 2; veio {past}"
    );

    // E o piso: um contorno que passou do eixo MAIS do que a linha é grossa não tem cor
    // nenhuma para pôr ali — largura 0, nunca negativa.
    let way_past = med(&contour_widths_with_margin(
        &strokes,
        &ring_at(23.0),
        1.0,
        0.0,
    ));
    assert!(
        way_past == 0.0,
        "alem do eixo por mais que a meia-espessura, a largura e ZERO; veio {way_past}"
    );
}
