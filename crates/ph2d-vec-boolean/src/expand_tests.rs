//! Testes de [`crate::expand`] — arquivo irmão.
//!
//! Os oráculos aqui medem **o que a forma passa a ser** (largura da caixa, área, quantos
//! pedaços, o buraco continua buraco), nunca a regra que a produziu. Um gate que reafirma a
//! fórmula do offset seria verde com a fórmula errada.

use super::*;
use kurbo::ParamCurve;
use ph2d_vec_scene::{Contour, OffsetSide, Rgba8, VecVertex, WidthProfile};

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
/// **compound**. A espessura da parede é `(outer − inner)/2`, e é ela que decide se a
/// fixture contém o caso interessante: só quando a parede é ≤ o offset é que as bandas dos
/// dois contornos se cobrem.
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
        let out = offset_path(&square(10.0), d, LineJoin::Miter, OffsetSide::Both);
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
    let out = offset_path(&square(10.0), -2.0, LineJoin::Miter, OffsetSide::Both);
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
    assert!(offset_path(&square(10.0), -6.0, LineJoin::Miter, OffsetSide::Both).is_empty());
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
    let round = offset_path(&square(10.0), 3.0, LineJoin::Round, OffsetSide::Both);
    let miter = offset_path(&square(10.0), 3.0, LineJoin::Miter, OffsetSide::Both);
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

/// **`Both` expande CADA contorno pela sua normal externa** — a borda de fora para fora, e o
/// furo também para fora (o furo cresce). É o offset POR CONTORNO: `d>0` empurra todo contorno
/// selecionado para longe do miolo que ele fecha.
#[test]
fn both_side_expands_every_contour_outward() {
    // donut 20/16, offset 2, Both: fora 20→24, furo 16→20.
    let out = offset_path(&donut(20.0, 16.0), 2.0, LineJoin::Miter, OffsetSide::Both);
    assert_eq!(out.len(), 1, "continua UMA forma");
    assert_eq!(out[0].subpaths.len(), 1, "…e continua tendo o furo");
    assert!(
        (bbox(&out[0]).width() - 24.0).abs() < 1e-6,
        "fora foi a {}",
        bbox(&out[0]).width()
    );
    // 24² menos o furo crescido a 20²: 576 − 400 = 176.
    let a = total_area(&out);
    assert!(
        (a - 176.0).abs() < 1.0,
        "área {a} (esperado 176: 24² com furo de 20²)"
    );
}

/// **`Outer` move SÓ o contorno de fora** — o furo fica onde estava. É o controle que o smoke
/// pediu (offsetar apenas um dos contornos).
#[test]
fn outer_side_moves_only_the_outer_contour() {
    let out = offset_path(&donut(20.0, 16.0), 2.0, LineJoin::Miter, OffsetSide::Outer);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].subpaths.len(), 1);
    assert!((bbox(&out[0]).width() - 24.0).abs() < 1e-6, "fora cresceu");
    // O furo intocado (16²): 24² − 16² = 320.
    let a = total_area(&out);
    assert!(
        (a - 320.0).abs() < 1.0,
        "área {a} (esperado 320: 24² com furo INTOCADO de 16²)"
    );
}

/// **`Inner` move SÓ os furos** — a borda de fora fica onde estava. Parede grossa (donut
/// 20/10) para o furo crescido não alcançar a borda.
#[test]
fn inner_side_moves_only_the_holes() {
    let out = offset_path(&donut(20.0, 10.0), 2.0, LineJoin::Miter, OffsetSide::Inner);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].subpaths.len(), 1);
    // Fora intocado (20), furo 10→14: 20² − 14² = 400 − 196 = 204.
    assert!(
        (bbox(&out[0]).width() - 20.0).abs() < 1e-6,
        "a borda de fora não devia mexer"
    );
    let a = total_area(&out);
    assert!(
        (a - 204.0).abs() < 1.0,
        "área {a} (esperado 204: 20² com furo crescido a 14²)"
    );
}

/// **A JUNÇÃO CHEGA NO CONTORNO INTERNO** — o bug que o Enio reportou (`2026-07-20`, "round e
/// bevel só no externo"). Com `Inner` + `Round`, crescer o furo arredonda as quinas DELE: cada
/// ponto do contorno interno fica a distância exatamente `d` do furo original (o disco), não a
/// `d√2` (a quina afiada do `Miter`). O oráculo é a geometria do FURO, não do contorno de fora.
#[test]
fn the_join_reaches_the_inner_contour() {
    // Distância de um ponto ao quadrado do furo original [5,15]² (donut 20/10).
    let dist_to_hole = |p: kurbo::Point| {
        let dx = (5.0 - p.x).max(0.0).max(p.x - 15.0);
        let dy = (5.0 - p.y).max(0.0).max(p.y - 15.0);
        dx.hypot(dy)
    };
    // O quanto o CONTORNO DO FURO se afasta do furo original.
    let hole_reach = |p: &VecPath| {
        let hole = VecPath {
            verts: p.subpaths[0].verts.clone(),
            closed: true,
            ..VecPath::default()
        };
        crate::to_bez_with(&hole, Closing::Always)
            .segments()
            .flat_map(|s| (0..=16).map(move |k| dist_to_hole(s.eval(f64::from(k) / 16.0))))
            .fold(0.0_f64, f64::max)
    };
    let round = offset_path(&donut(20.0, 10.0), 3.0, LineJoin::Round, OffsetSide::Inner);
    let miter = offset_path(&donut(20.0, 10.0), 3.0, LineJoin::Miter, OffsetSide::Inner);
    let (r, m) = (hole_reach(&round[0]), hole_reach(&miter[0]));
    assert!(
        (r - 3.0).abs() < 0.06,
        "Round no furo: a borda interna chega a {r} (o disco diz 3) — a quina não chegou no furo"
    );
    assert!(
        (m - 3.0 * std::f64::consts::SQRT_2).abs() < 0.06,
        "Miter no furo: o bico chega a {m} (a quina afiada diz {})",
        3.0 * std::f64::consts::SQRT_2
    );
}

/// **UM CONTORNO PASSANDO POR CIMA DO OUTRO TROCA DE PAPEL — NÃO SOME** (Enio, `2026-07-20`:
/// *"em vez de desaparecer, redesenhar dessa vez com os contornos trocados"*). Crescer o furo
/// (`Inner`) ALÉM da borda de fora não pode dar vazio: o `EvenOdd` faz o que era buraco virar
/// sólido (a borda interna vira a nova externa). A subtração sequencial `fora − furo` ia a
/// VAZIO exatamente aqui — a forma "sumia"/"pulava".
///
/// ⚠️ O oráculo é **área não-nula APÓS o cruzamento**. A mutação que reverte para a subtração
/// (`fora − furo`) devolve vazio e o gate sangra.
#[test]
fn a_contour_crossing_the_other_swaps_instead_of_vanishing() {
    // donut 20/16 (parede 2). Inner +3 ⇒ o furo cresce a ~22 > a borda de 20 ⇒ CRUZA.
    let out = offset_path(&donut(20.0, 16.0), 3.0, LineJoin::Round, OffsetSide::Inner);
    let a = total_area(&out);
    assert!(
        !out.is_empty() && a > 10.0,
        "o furo cruzou a borda e a forma SUMIU (área {a}) em vez de trocar os contornos"
    );
    // …e a forma trocada continua sendo um anel (o que era furo agora é a borda de fora).
    assert!(
        out.iter().any(|p| !p.subpaths.is_empty()),
        "a forma trocada devia ser um anel (compound), não um disco sólido"
    );
}

/// **Encolher um furo além dele mesmo o FECHA.** `Inner` com `d` negativo aperta o furo; passar
/// da metade dele deixa a forma sólida — nada de furo invertido.
#[test]
fn shrinking_a_hole_past_itself_closes_it() {
    // Furo 16, encolhido por 9 de cada lado (16/2 = 8 < 9) ⇒ fecha; a borda de fora fica.
    let out = offset_path(&donut(20.0, 16.0), -9.0, LineJoin::Miter, OffsetSide::Inner);
    assert_eq!(out.len(), 1);
    assert!(
        out[0].subpaths.is_empty(),
        "o furo apertado além de si fecha"
    );
    let a = total_area(&out);
    assert!(
        (a - 400.0).abs() < 1.0,
        "área {a} (esperado 400: o quadrado 20 sólido)"
    );
}

/// Offsetar preserva o ESTILO — é a mesma arte com a borda noutro lugar, não uma forma nova.
#[test]
fn offsetting_keeps_the_art_it_was() {
    let src = stroked(square(10.0), 1.5);
    let out = offset_path(&src, 1.0, LineJoin::Miter, OffsetSide::Both);
    assert_eq!(out[0].stroke, src.stroke, "o traço é o mesmo");
    assert_eq!(out[0].fill, src.fill);
}

/// Distância zero (ou não-finita) não é uma operação — devolve vazio em vez de gastar um
/// passo de undo para não mudar nada.
#[test]
fn a_zero_offset_is_not_an_operation() {
    assert!(offset_path(&square(10.0), 0.0, LineJoin::Miter, OffsetSide::Both).is_empty());
    assert!(offset_path(&square(10.0), f64::NAN, LineJoin::Miter, OffsetSide::Both).is_empty());
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

// ───────────────────────────── Power Stroke ─────────────────────────────

/// Um perfil que afina nas duas pontas e engrossa no meio — o caso canônico.
const TAPER: WidthProfile = WidthProfile {
    start: 0.25,
    mid: 2.0,
    end: 0.25,
    position: 0.5,
};

/// **O traço FICA MAIS GROSSO onde o perfil manda.** O oráculo é a espessura MEDIDA na
/// tinta: uma vertical no meio da linha atravessa mais tinta que uma perto da ponta. Um
/// gate que reafirmasse `profile.at()` seria verde com o motor desligado.
#[test]
fn the_ink_is_thicker_where_the_profile_says() {
    let line = stroked(open_line(20.0), 1.0);
    let out = power_stroke(&line, &TAPER);
    assert_eq!(out.len(), 1, "uma forma");
    let bez = crate::to_bez_with(&out[0], Closing::Always);
    // Quanta tinta uma vertical em `x` atravessa (a altura da caixa da forma ali é uma
    // aproximação honesta: a linha é horizontal, então a tinta é simétrica em torno dela).
    let thickness_at = |x: f64| {
        let mut lo = f64::MAX;
        let mut hi = f64::MIN;
        for seg in bez.segments() {
            for k in 0..=64 {
                let p = seg.eval(f64::from(k) / 64.0);
                if (p.x - x).abs() < 0.15 {
                    lo = lo.min(p.y);
                    hi = hi.max(p.y);
                }
            }
        }
        hi - lo
    };
    let (mid, near_end) = (thickness_at(10.0), thickness_at(1.0));
    assert!(
        mid > near_end * 3.0,
        "meio {mid:.3} vs perto da ponta {near_end:.3}: o perfil não chegou na tinta"
    );
    // O meio pede 2.0 × a largura 1.0 = 2.0 de espessura.
    assert!(
        (mid - 2.0).abs() < 0.15,
        "espessura no meio {mid:.3} (o perfil pede 2.0)"
    );
}

/// **Um perfil UNIFORME não é uma operação** — é o Outline Stroke, e ter dois botões que
/// produzem a mesma saída é pior que ter um.
#[test]
fn a_uniform_profile_is_not_an_operation() {
    let line = stroked(open_line(10.0), 2.0);
    assert!(power_stroke(&line, &WidthProfile::UNIFORM).is_empty());
}

/// Sem traço não há largura a variar.
#[test]
fn a_path_without_a_stroke_has_no_width_to_vary() {
    assert!(power_stroke(&square(10.0), &TAPER).is_empty());
}

/// **A tinta é UMA peça só, sem buracos** — os quads da fita partilham aresta com os vizinhos,
/// então a união é conexa mesmo onde a curvatura aperta e os trilhos se cruzam; as lascas de
/// área ~zero que o pinço deixa são varridas pelo [`drop_slivers`]. É isso que separa o método
/// do offset ingênuo (que se auto-cruza e deixa laços). Fixture com curvatura que MUDA DE
/// SINAL, que é onde um offsetter falharia — e onde o pinço de fato acontece.
#[test]
fn the_ink_is_one_connected_piece_with_no_loops() {
    let mut verts = Vec::new();
    for i in 0..5 {
        let x = f64::from(i) * 2.0;
        let y = if i % 2 == 0 { -1.0 } else { 1.0 };
        let mut vt = v(x, y);
        vt.in_handle = [x - 0.9, y];
        vt.out_handle = [x + 0.9, y];
        verts.push(vt);
    }
    let sine = stroked(
        VecPath {
            verts,
            closed: false,
            ..VecPath::default()
        },
        0.6,
    );
    let out = power_stroke(&sine, &TAPER);
    assert_eq!(out.len(), 1, "a tinta é uma peça só");
    assert!(
        out[0].subpaths.is_empty(),
        "sobrou um laço como buraco — a fita de trilhos não pode produzir isso"
    );
    assert!(total_area(&out) > 1.0, "área {}", total_area(&out));
}

/// **A BORDA SEGUE O PERFIL, SEM FESTÃO** — o bug que o Enio reportou (`2026-07-20`, "traços
/// rugosos"). A união de discos deixava a borda externa como uma sucessão de arcos que bojam
/// entre os centros (medido: ~4% de ondulação da largura de pico). Os trilhos seguem o perfil
/// direto: a borda passa a desviar bem menos que 1% dele.
///
/// Numa linha RETA o perfil é a ÚNICA coisa que molda a borda, então o desvio medido é o
/// festão puro. O oráculo é a APARÊNCIA (a distância da borda ao centro), não a fórmula: um
/// gate que reafirmasse `profile.at()` seria verde com o festão de volta.
#[test]
fn the_ribbon_boundary_follows_the_profile_without_ripple() {
    let profile = WidthProfile {
        start: 0.4,
        mid: 2.0,
        end: 0.4,
        position: 0.5,
    };
    let (width, len) = (1.0, 20.0);
    let out = power_stroke(&stroked(open_line(len), width), &profile);
    assert_eq!(out.len(), 1);
    let bez = crate::to_bez_with(&out[0], Closing::Always);
    // A borda de cima em `x`: o maior `y` que a tinta alcança ali (a linha é horizontal em
    // y=0, então a meia-largura de cima é esse `y`).
    let top_at = |x: f64| {
        let mut hi = f64::MIN;
        for seg in bez.segments() {
            for k in 0..=48 {
                let p = seg.eval(f64::from(k) / 48.0);
                if (p.x - x).abs() < 0.08 {
                    hi = hi.max(p.y);
                }
            }
        }
        hi
    };
    let mut max_dev = 0.0_f64;
    for i in 10..=190 {
        let x = len * f64::from(i) / 200.0;
        let predicted = 0.5 * width * profile.at(x / len);
        let measured = top_at(x);
        if measured > f64::MIN {
            max_dev = max_dev.max((measured - predicted).abs());
        }
    }
    assert!(
        max_dev < 0.03,
        "a borda desvia {max_dev:.4} do perfil — é o festão da união de discos, de volta"
    );
}

/// **Num contorno FECHADO a tinta é um anel** — tem buraco, e o buraco é o miolo da forma.
/// É o mesmo que um traço uniforme faz; o que muda é a espessura do anel ao longo dele.
#[test]
fn stroking_a_closed_contour_leaves_a_ring() {
    let out = power_stroke(&stroked(square(10.0), 1.0), &TAPER);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].subpaths.len(), 1, "o anel tem o miolo como buraco");
}

/// A forma assada é preenchida com a cor do traço e não tem traço — a MESMA lei do Outline
/// Stroke, pela mesma porta (`ink_style`).
#[test]
fn the_baked_ink_carries_the_strokes_colour() {
    let out = power_stroke(&stroked(open_line(10.0), 1.0), &TAPER);
    assert_eq!(out[0].fill, Some(Paint::Solid(Rgba8::new(10, 20, 30, 255))));
    assert_eq!(out[0].stroke, None);
}

/// **Um perfil que zera afina a tinta ATÉ NADA na ponta** — a linha vira um bico, que é o
/// traço de nanquim que o artista quer.
///
/// ⚠️ O oráculo é a ESPESSURA, não a posição onde a tinta começa. A 1ª versão deste gate
/// exigia que a tinta *recuasse* (`x0 > 0.5`) e nasceu vermelha em `x0 = −0.001` — mas o
/// código estava certo: um perfil que chega a zero **suavemente** não trunca a linha, ele a
/// adelgaça, e a primeira fatia ainda tem os 0,0007 de largura que o perfil de facto pede
/// ali. Era o gate que descrevia um bico truncado em vez do bico real.
#[test]
fn a_profile_that_reaches_zero_tapers_the_ink_to_a_point() {
    let zero_start = WidthProfile {
        start: 0.0,
        mid: 2.0,
        end: 2.0,
        position: 0.5,
    };
    let out = power_stroke(&stroked(open_line(20.0), 1.0), &zero_start);
    assert_eq!(out.len(), 1);
    let bez = crate::to_bez_with(&out[0], Closing::Always);
    let thickness_at = |x: f64| {
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for seg in bez.segments() {
            for k in 0..=64 {
                let p = seg.eval(f64::from(k) / 64.0);
                if (p.x - x).abs() < 0.2 {
                    lo = lo.min(p.y);
                    hi = hi.max(p.y);
                }
            }
        }
        hi - lo
    };
    let (tip, body) = (thickness_at(0.15), thickness_at(10.0));
    assert!(
        tip < 0.1,
        "o bico mede {tip:.4} de espessura — devia ser ~0 (o perfil zera ali)"
    );
    assert!(
        body > 1.5,
        "o corpo mede {body:.3} — devia ser ~2 (o perfil pede 2× a largura 1)"
    );
}

/// **Um trecho de largura EXATAMENTE zero não vira lasca.** Um contorno LONGO de área NULA é
/// precisamente a lasca que o Shape Builder já pagou para aprender a reconhecer: sem área não
/// há preenchimento, então ela pinta como uma LINHA solta que o artista não desenhou.
///
/// ⚠️ **Quem defende isto é o `drop_slivers`** (não mais um guard no laço — a fita de trilhos
/// não tem o `w <= MIN_TOL` que a antiga união de discos tinha). Onde o perfil é zero os dois
/// trilhos colapsam sobre o centro, o quad tem área nula, e o sweep + o `drop_slivers` o
/// varrem por área relativa ao total — nada de linha solta.
#[test]
fn a_stretch_of_exactly_zero_width_produces_no_sliver() {
    let flat_zero = WidthProfile {
        start: 0.0,
        mid: 0.0,
        end: 2.0,
        position: 0.5,
    };
    let out = power_stroke(&stroked(open_line(20.0), 1.0), &flat_zero);
    assert!(!out.is_empty(), "a metade grossa continua sendo tinta");
    for p in &out {
        let b = bbox(p);
        let density = crate::area(p) / (b.width() * b.height()).max(1e-12);
        assert!(
            density > 0.05,
            "saiu uma peça de densidade {density:.4} (área {:.6} numa caixa {:.2}×{:.2}) — \
             é uma lasca, e ela pinta como uma linha solta",
            crate::area(p),
            b.width(),
            b.height()
        );
    }
    // E a tinta começa depois da metade: a 1ª metade do perfil é zero.
    let left = out.iter().map(|p| bbox(p).x0).fold(f64::MAX, f64::min);
    assert!(
        left > 8.0,
        "a tinta começa em x={left:.2} (o zero vai até 10)"
    );
}
