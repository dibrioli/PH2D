//! Testes de [`crate::expand`] — arquivo irmão.
//!
//! Os oráculos aqui medem **o que a forma passa a ser** (largura da caixa, área, quantos
//! pedaços, o buraco continua buraco), nunca a regra que a produziu. Um gate que reafirma a
//! fórmula do offset seria verde com a fórmula errada.

use super::*;
use kurbo::ParamCurve;
use ph2d_vec_scene::{Contour, Rgba8, VecVertex};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex::corner([x, y])
}

/// Um quadrado de lado `s`, canto inferior-esquerdo na origem.
fn square(s: f64) -> VecPath {
    VecPath {
        verts: vec![v(0.0, 0.0), v(s, 0.0), v(s, s), v(0.0, s)],
        closed: true,
        ..VecPath::default()
    }
}

/// Uma linha reta de `len`, aberta.
fn open_line(len: f64) -> VecPath {
    VecPath {
        verts: vec![v(0.0, 0.0), v(len, 0.0)],
        closed: false,
        ..VecPath::default()
    }
}

/// Um quadrado de lado `outer` com um furo quadrado de lado `inner` no centro — o
/// **compound**, a fixture que contém o fenômeno que quase deu errado (as bandas dos dois
/// contornos, que correm em sentidos opostos).
fn donut(outer: f64, inner: f64) -> VecPath {
    let o = (outer - inner) * 0.5;
    let (a, b) = (o, o + inner);
    VecPath {
        subpaths: vec![Contour::new_closed(vec![
            v(a, a),
            v(b, a),
            v(b, b),
            v(a, b),
        ])],
        fill_rule: FillRule::EvenOdd,
        ..square(outer)
    }
}

fn stroked(mut p: VecPath, width: f64) -> VecPath {
    p.stroke = Some(StrokeSpec::new(Rgba8::new(10, 20, 30, 255), width));
    p
}

fn bbox(p: &VecPath) -> kurbo::Rect {
    crate::to_bez_with(p, Closing::Always).bounding_box()
}

fn total_area(ps: &[VecPath]) -> f64 {
    ps.iter().map(crate::area).sum()
}

// ─────────────────────────────── Offset Path ───────────────────────────────

/// **A borda anda exatamente `d`.** Um quadrado de 10 crescido de 2 mede 14 de ponta a
/// ponta — e não 12, nem 10 com as quinas gordas. É a asserção que separa um offset de um
/// "quase".
#[test]
fn the_border_moves_exactly_the_distance_asked() {
    for d in [0.5, 2.0, 5.0] {
        let out = offset_path(&square(10.0), d, LineJoin::Miter);
        assert_eq!(out.len(), 1, "d={d}: uma forma");
        let b = bbox(&out[0]);
        assert!(
            (b.width() - (10.0 + 2.0 * d)).abs() < 1e-6,
            "d={d}: largura {} (esperado {})",
            b.width(),
            10.0 + 2.0 * d
        );
        assert!(
            (b.x0 + d).abs() < 1e-6,
            "d={d}: a borda esquerda foi a {}",
            b.x0
        );
    }
}

/// …e para DENTRO com `d` negativo. Encolher é a mesma pergunta com o sinal trocado, e é
/// onde uma implementação que só sabe crescer fica calada.
#[test]
fn a_negative_distance_shrinks_the_shape() {
    let out = offset_path(&square(10.0), -2.0, LineJoin::Miter);
    assert_eq!(out.len(), 1);
    let b = bbox(&out[0]);
    assert!(
        (b.width() - 6.0).abs() < 1e-6,
        "encolheu para {} (esperado 6)",
        b.width()
    );
}

/// **Encolher além do que a forma tem faz a forma SUMIR** — e sumir é a resposta certa,
/// não um resíduo de área ~0 que o artista teria de caçar para apagar.
#[test]
fn shrinking_past_the_shape_leaves_nothing() {
    assert!(offset_path(&square(10.0), -6.0, LineJoin::Miter).is_empty());
}

/// **A junção `Round` é o offset MÉTRICO**: a quina vira um arco a distância exatamente `d`
/// do vértice original. `Miter` a deixa afiada, e aí o ponto mais distante fica a `d√2`. As
/// duas coisas são pedidas no diálogo do Illustrator, e trocá-las não pode passar calado.
#[test]
fn round_joins_give_the_true_metric_offset_and_miter_gives_the_spike() {
    // Distância de um ponto ao quadrado [0,10]² — zero dentro dele.
    let dist_to_src = |p: kurbo::Point| {
        let dx = (0.0 - p.x).max(0.0).max(p.x - 10.0);
        let dy = (0.0 - p.y).max(0.0).max(p.y - 10.0);
        dx.hypot(dy)
    };
    // O ponto do contorno que mais se AFASTA da forma de origem. Com `Round` esse número é
    // a própria definição do offset métrico (todo ponto da borda está a exatamente `d`);
    // com `Miter` o bico da quina vai a `d√2`.
    let reach = |p: &VecPath| {
        crate::to_bez_with(p, Closing::Always)
            .segments()
            .flat_map(|s| (0..=16).map(move |k| dist_to_src(s.eval(f64::from(k) / 16.0))))
            .fold(0.0_f64, f64::max)
    };
    let round = offset_path(&square(10.0), 3.0, LineJoin::Round);
    let miter = offset_path(&square(10.0), 3.0, LineJoin::Miter);
    let (r, m) = (reach(&round[0]), reach(&miter[0]));
    assert!(
        (r - 3.0).abs() < 0.05,
        "Round: a borda chega a {r} da forma (o disco diz 3 em toda direção)"
    );
    assert!(
        (m - 3.0 * std::f64::consts::SQRT_2).abs() < 0.05,
        "Miter: o bico chega a {m} (a quina afiada diz {})",
        3.0 * std::f64::consts::SQRT_2
    );
}

/// **Um contorno que a kurbo fecha com um ARCO tem de sobreviver ao sweep.** O cap redondo
/// devolve a emenda em `sin(π) = -1.22e-16` em vez de zero, e o `linesweeper` compara
/// coordenada exata: sem soldar, ele recusa o caminho INTEIRO e o Outline Stroke devolve
/// vazio — calado — para todo traço com ponta ou junção redonda.
#[test]
fn an_arc_closed_contour_survives_the_sweep() {
    let mut l = stroked(open_line(10.0), 2.0);
    if let Some(s) = l.stroke.as_mut() {
        s.cap = LineCap::Round;
        s.join = LineJoin::Round;
    }
    assert!(
        !outline_stroke(&l).is_empty(),
        "o cap redondo desapareceu no sweep"
    );
}

/// **O compound: a borda cresce e o BURACO encolhe.** Este é o gate que a banda por-contorno
/// existe para passar — traçar os dois contornos de uma vez os cancela na parede entre eles,
/// e o furo some (ou vira lixo) exatamente quando o offset fica interessante.
#[test]
fn growing_a_donut_grows_the_rim_and_shrinks_the_hole() {
    let out = offset_path(&donut(20.0, 8.0), 2.0, LineJoin::Miter);
    assert_eq!(out.len(), 1, "continua UMA forma");
    assert_eq!(out[0].subpaths.len(), 1, "…e continua tendo o furo");
    let b = bbox(&out[0]);
    assert!(
        (b.width() - 24.0).abs() < 1e-6,
        "a borda foi a {}",
        b.width()
    );
    // 24² menos um furo de 4²: o furo encolhe de 8 para 4 (erodir um quadrado por um disco
    // dá um quadrado menor, com as quinas ainda afiadas).
    let a = total_area(&out);
    assert!(
        (a - (576.0 - 16.0)).abs() < 1.0,
        "área {a} (esperado ~560: 24² com um furo de 4²)"
    );
}

/// Offsetar preserva o ESTILO — é a mesma arte com a borda noutro lugar, não uma forma nova.
#[test]
fn offsetting_keeps_the_art_it_was() {
    let src = stroked(square(10.0), 1.5);
    let out = offset_path(&src, 1.0, LineJoin::Miter);
    assert_eq!(out[0].stroke, src.stroke, "o traço é o mesmo");
    assert_eq!(out[0].fill, src.fill);
}

/// Distância zero (ou não-finita) não é uma operação — devolve vazio em vez de gastar um
/// passo de undo para não mudar nada.
#[test]
fn a_zero_offset_is_not_an_operation() {
    assert!(offset_path(&square(10.0), 0.0, LineJoin::Miter).is_empty());
    assert!(offset_path(&square(10.0), f64::NAN, LineJoin::Miter).is_empty());
}

// ───────────────────────────── Outline Stroke ─────────────────────────────

/// Uma linha traçada vira um retângulo de `comprimento × largura`. O caso mais simples que
/// existe, e o que fixa a UNIDADE: a largura do traço é diâmetro, não raio.
#[test]
fn a_stroked_line_becomes_a_rectangle_of_length_times_width() {
    let out = outline_stroke(&stroked(open_line(10.0), 2.0));
    assert_eq!(out.len(), 1);
    let a = total_area(&out);
    assert!((a - 20.0).abs() < 0.01, "área {a} (esperado 10×2)");
}

/// **A forma assada é PREENCHIDA com a cor do traço e não tem traço.** Deixar o traço nela
/// a desenharia uma segunda vez, em volta de si mesma: o clique que não devia mudar nada
/// engordaria o desenho.
#[test]
fn the_outline_is_filled_with_the_strokes_colour_and_has_no_stroke() {
    let out = outline_stroke(&stroked(square(10.0), 2.0));
    assert_eq!(out[0].fill, Some(Paint::Solid(Rgba8::new(10, 20, 30, 255))));
    assert_eq!(out[0].stroke, None, "a forma É o traço — não tem traço");
}

/// **Um caminho ABERTO não fecha ao ser assado.** Assar um "C" como se fosse um "O" lhe
/// daria a corda de volta que o artista nunca traçou. O oráculo é a ÁREA: o contorno de um
/// caminho aberto mede perímetro×largura; se ele fechasse, mediria a região inteira.
#[test]
fn an_open_path_is_not_closed_by_the_bake() {
    let c = stroked(
        VecPath {
            verts: vec![v(10.0, 0.0), v(0.0, 0.0), v(0.0, 10.0), v(10.0, 10.0)],
            closed: false,
            ..VecPath::default()
        },
        1.0,
    );
    let a = total_area(&outline_stroke(&c));
    // Três arestas de 10 com caneta de 1 = 30, mais um pouco nas duas junções.
    assert!(
        (25.0..40.0).contains(&a),
        "área {a}: um C aberto mede ~30 de tinta; ~100 seria o miolo que ele NÃO tem"
    );
}

/// **O tracejado é assado.** Uma linha pontilhada vira VÁRIAS formas, uma por traço — não
/// uma barra sólida. É o que prova que o estilo inteiro chega no forno.
#[test]
fn a_dashed_stroke_bakes_into_one_shape_per_dash() {
    let mut line = stroked(open_line(20.0), 1.0);
    if let Some(s) = line.stroke.as_mut() {
        s.dash = Some((2.0, 2.0)); // traço 2, vão 2
    }
    let out = outline_stroke(&line);
    assert!(out.len() >= 4, "saíram {} pedaços (esperado ~5)", out.len());
    let a = total_area(&out);
    assert!(
        a < 15.0,
        "área {a}: com metade em vão, a tinta é ~metade dos 20 da linha sólida"
    );
}

/// **A PONTA vem junto.** Uma seta assada continua tendo cabeça — perdê-la em silêncio
/// apagaria desenho, que é pior que recusar a operação.
#[test]
fn baking_an_arrow_keeps_its_head() {
    let mut arrow = stroked(open_line(20.0), 1.0);
    let bare = total_area(&outline_stroke(&arrow));
    if let Some(s) = arrow.stroke.as_mut() {
        s.marker_end = ph2d_vec_scene::Marker::Triangle;
    }
    let out = outline_stroke(&arrow);
    let with_head = total_area(&out);
    assert!(
        with_head > bare * 1.15,
        "com cabeça {with_head} vs sem {bare}: a ponta não chegou no forno"
    );
    let right = out.iter().map(|p| bbox(p).x1).fold(f64::MIN, f64::max);
    assert!(
        (right - 20.0).abs() < 0.2,
        "o bico da seta ficou em x={right} (a linha acaba em 20)"
    );
}

/// Sem traço não há o que assar — vazio, e não um path de área zero para o artista caçar.
#[test]
fn a_path_without_a_stroke_bakes_to_nothing() {
    assert!(outline_stroke(&square(10.0)).is_empty());
}

/// **A ponta do traço muda a forma assada.** `Round` acrescenta dois meio-discos e `Square`
/// dois meio-quadrados: se o cap não chegasse no forno, as três medidas seriam iguais.
#[test]
fn the_cap_style_reaches_the_bake() {
    let mk = |cap| {
        let mut l = stroked(open_line(10.0), 2.0);
        if let Some(s) = l.stroke.as_mut() {
            s.cap = cap;
        }
        total_area(&outline_stroke(&l))
    };
    let (butt, round, sq) = (mk(LineCap::Butt), mk(LineCap::Round), mk(LineCap::Square));
    assert!((butt - 20.0).abs() < 0.01, "butt {butt}");
    assert!(
        (round - (20.0 + std::f64::consts::PI)).abs() < 0.05,
        "round {round}: dois meio-discos de raio 1 somam π"
    );
    assert!((sq - 24.0).abs() < 0.01, "square {sq}: dois quadrados 2×1");
}
