//! Gates da DILATAÇÃO. Cada um pina um defeito que já aconteceu de verdade — os três
//! primeiros são BUGS #20 (a dilatação 100× grande, e a média global), o quarto é a
//! lição do BUGS #18 (vértice contra segmento).

use super::{contour_widths, local_line};
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

/// **A largura é o DESVIO, e mais nada — a espessura da linha não entra.**
///
/// Este gate é o inverso exato dos dois que ele substituiu (*"o contorno veste a linha que
/// abraça"* e *"a margem escala com a linha"*). Os dois pinavam um termo `w` que a
/// medição contra o Draw:Filled condenou em 2026-07-18: com pincel macio ele afastava o
/// balde da referência aprovada em **doze mil pixels**, e era contagem dupla do termo do
/// desvio (ver o topo do `dilate.rs`).
///
/// A afirmação de hoje: um contorno pousado **em cima do eixo** ganha largura **ZERO**,
/// e ganha zero *tanto na linha fina quanto na grossa*. A cor termina no eixo, que é onde
/// o Draw:Filled a termina.
///
/// ⚠️ **A fixture contém as duas espessuras de propósito**: com uma só, ressuscitar o
/// termo `w` daria um número que alguém poderia ler como "a compensação". Com duas, a
/// mutação produz 2,0 num ponto e 16,0 no outro — impossível de confundir com desvio, que
/// é o MESMO nos dois (zero).
#[test]
fn the_width_is_the_offset_and_the_line_thickness_never_enters() {
    let thin = vec![line(0.0, 2.0, -50.0, 50.0, 8)];
    let fat = vec![line(0.0, 16.0, -50.0, 50.0, 8)];
    let probe = [Vec2::new(0.0, 0.0)]; // EM CIMA do eixo: desvio zero

    let wt = contour_widths(&thin, &probe)[0];
    let wf = contour_widths(&fat, &probe)[0];

    assert!(
        wt.abs() < 1e-4,
        "sobre o eixo da linha FINA a largura tem de ser ZERO; veio {wt} \
         (2,0 = o termo `w` de volta)"
    );
    assert!(
        wf.abs() < 1e-4,
        "sobre o eixo da linha GROSSA a largura tem de ser ZERO; veio {wf} \
         (16,0 = o termo `w` de volta, e a franja com ele)"
    );
}

/// **Duas linhas de espessuras diferentes não mudam nada** — o corolário que fecha o
/// BUGS #20 pela raiz.
///
/// O #20 era a dilatação usando a espessura MÉDIA do desenho: onde o contorno abraçava a
/// linha fina, a cor saía larga demais e atravessava. O remédio da época foi perguntar
/// qual linha o contorno veste (`local_line`). Hoje o defeito é impossível por
/// construção — **nenhuma espessura entra na largura** —, e é isto que o gate pina.
///
/// `local_line` continua público e testado abaixo: ele é o oráculo de *"que linha é
/// esta?"* para quem precisa da pergunta. Ele só não manda mais na dilatação.
#[test]
fn a_thick_neighbour_cannot_fatten_the_colour_over_a_thin_line() {
    let strokes = vec![
        line(0.0, 10.0, -50.0, 50.0, 8),  // grossa
        line(100.0, 2.0, -50.0, 50.0, 8), // fina
    ];
    // Um ponto sobre cada eixo.
    let contour = [Vec2::new(0.0, 0.0), Vec2::new(0.0, 100.0)];
    let w = contour_widths(&strokes, &contour);
    assert!(
        w[0].abs() < 1e-4 && w[1].abs() < 1e-4,
        "sobre os dois eixos a largura e ZERO nos dois; veio {w:?}"
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

    // Os valores são EXATOS, não ordenações frouxas — a lei é `2s`, então cada número é
    // verificável de cabeça. (O alisamento não os move: binomial de uma constante é a
    // constante.)
    let short = med(&contour_widths(&strokes, &ring_at(17.0)));
    let on = med(&contour_widths(&strokes, &ring_at(20.0)));
    let past = med(&contour_widths(&strokes, &ring_at(21.0)));

    // Em cima do eixo a lei não faz NADA: a cor termina onde já estava.
    assert!(
        on.abs() < 0.3,
        "sobre o eixo nao ha o que corrigir: largura ZERO; veio {on}"
    );
    // Aquém por 3: precisa alcançar 3 a mais, e a largura empurra dos DOIS lados => 6.
    assert!(
        (short - 6.0).abs() < 0.3,
        "aquem do eixo por 3, a largura tem de ser 2*3 = 6; veio {short}"
    );
    // Além do eixo: já passou. Não há anel a desenhar — e é ESTE o ponto que a versão sem
    // sinal errava, engordando e empurrando a cor para ainda mais longe da linha.
    assert!(
        past == 0.0,
        "alem do eixo a largura e ZERO (a cor ja passou); veio {past}"
    );
}
