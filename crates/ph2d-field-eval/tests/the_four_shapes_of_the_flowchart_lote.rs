//! ⭐⭐⭐ **AS QUATRO FORMAS DO LOTE DO FLUXOGRAMA (W122), PROVADAS ANTES DE SEREM LIGADAS.**
//!
//! A régua é a dos dois lotes anteriores e a mais barata que existe: **pontos cuja resposta se sabe
//! sem a fórmula** — um vértice, uma tangência, o vazio de um canto cortado. ⚠️ Nenhum deles vem de
//! correr o código e escrever o que ele deu.
//!
//! ⚠️ **Ele não substitui o censo** (`the_census_of_every_primitive`), que pergunta se o campo ainda
//! é uma distância e se a caixa contém a peça. Estes dizem se a forma **é** o que o nome promete.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn campo(p: Primitive) -> Field {
    Field::new(
        &FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0),
        )
        .expect("a peça"),
    )
}

const NA_PELE: f64 = 3.0e-3;

#[track_caller]
fn dentro(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(
        v < -NA_PELE,
        "{porque}: {p:?} devia estar DENTRO e leu {v:.5}"
    );
}

#[track_caller]
fn fora(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(v > NA_PELE, "{porque}: {p:?} devia estar FORA e leu {v:.5}");
}

#[track_caller]
fn na_pele(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(
        v.abs() < NA_PELE,
        "{porque}: {p:?} devia estar NA SUPERFÍCIE e leu {v:.5}"
    );
}

fn um_paralelogramo(skew: f32) -> Primitive {
    Primitive::Parallelogram {
        half_width: 0.40,
        half_span: 0.30,
        skew,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    }
}

/// ⭐ **O paralelogramo INCLINA** — os quatro vértices estão onde a inclinação os põe, e os dois
/// cantos que o retângulo tinha ficaram VAZIOS.
#[test]
fn the_parallelogram_leans_and_its_old_corners_are_empty() {
    let f = campo(um_paralelogramo(0.20));
    dentro(&f, [0.0, 0.0, 0.0], "o meio");
    // Os vértices: `(±(w + skew), s)` em cima e `(±(w − skew), −s)` em baixo.
    na_pele(&f, [0.60, 0.30, 0.0], "o vértice de cima à direita");
    na_pele(&f, [-0.20, 0.30, 0.0], "o vértice de cima à esquerda");
    na_pele(&f, [0.20, -0.30, 0.0], "o vértice de baixo à direita");
    // ⛔ **Os cantos que o retângulo teria e a inclinação PERDEU** — com `skew > 0` a base de cima
    // escorrega para `+X`, então quem sai é o canto de cima à ESQUERDA e o de baixo à DIREITA.
    fora(
        &f,
        [-0.38, 0.28, 0.0],
        "o canto de cima à esquerda do retângulo",
    );
    fora(
        &f,
        [0.38, -0.28, 0.0],
        "o canto de baixo à direita do retângulo",
    );
    // E o que a inclinação GANHOU do outro lado.
    dentro(&f, [0.45, 0.20, 0.0], "o que a inclinação ganhou em cima");
}

/// ⭐⭐ **Com `skew = 0` ele é o RETÂNGULO** — e não «quase»: os quatro cantos estão lá.
#[test]
fn a_parallelogram_without_skew_is_a_rectangle() {
    let f = campo(um_paralelogramo(0.0));
    na_pele(&f, [0.40, 0.30, 0.0], "o canto de cima à direita");
    na_pele(&f, [-0.40, -0.30, 0.0], "o canto de baixo à esquerda");
    dentro(&f, [0.38, 0.28, 0.0], "logo dentro do canto");
}

/// ⭐⭐⭐ **A INCLINAÇÃO tem SINAL** — e as duas peças são o espelho uma da outra.
///
/// ⚠️ Sem este gate, um `skew` que ignorasse o sinal (um `abs` esquecido) passava em tudo o resto:
/// as duas formas têm a mesma caixa, a mesma marcha e o mesmo volume.
#[test]
fn the_skew_has_a_sign_and_the_two_are_mirrors() {
    let direita = campo(um_paralelogramo(0.20));
    let esquerda = campo(um_paralelogramo(-0.20));
    for (x, y) in [(0.45, 0.20), (0.55, 0.28), (-0.30, 0.15)] {
        let a = direita.at(x, y, 0.0);
        let b = esquerda.at(-x, y, 0.0);
        assert!(
            (a - b).abs() < 1.0e-6,
            "espelhar em X devia dar a inclinação oposta: ({x}, {y}) leu {a:.6} contra {b:.6}"
        );
    }
    dentro(
        &direita,
        [0.45, 0.20, 0.0],
        "a direita ganha em cima à direita",
    );
    fora(&esquerda, [0.45, 0.20, 0.0], "e a esquerda perde ali");
}

fn um_atraso(w: f32, s: f32) -> Primitive {
    Primitive::Delay {
        half_width: w,
        half_span: s,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    }
}

/// ⭐ **O atraso é RETO de um lado e REDONDO do outro** — e a tampa é um semicírculo INTEIRO.
#[test]
fn the_delay_is_flat_on_one_side_and_a_full_half_circle_on_the_other() {
    let (w, s) = (0.45_f64, 0.25_f64);
    let f = campo(um_atraso(w as f32, s as f32));
    dentro(&f, [0.0, 0.0, 0.0], "o meio");
    // A face reta: ela vale a altura INTEIRA, dos dois cantos.
    na_pele(&f, [-w, 0.0, 0.0], "o meio da face reta");
    na_pele(&f, [-w, s - 1.0e-4, 0.0], "o canto de cima da face reta");
    na_pele(&f, [-w, -s + 1.0e-4, 0.0], "o canto de baixo da face reta");
    fora(&f, [-w - 0.02, 0.0, 0.0], "à esquerda da face reta");
    // ⭐ **A tampa**: centro em `(w − s, 0)`, raio `s`. A ponta e os `45°` dela.
    na_pele(&f, [w, 0.0, 0.0], "a ponta da tampa");
    let d = s / 2.0_f64.sqrt();
    na_pele(&f, [w - s + d, d, 0.0], "os 45° da tampa");
    // ⛔ **O canto que um retângulo teria** — é ele que prova que a tampa é redonda.
    fora(&f, [w - 0.01, s - 0.01, 0.0], "o canto de cima à direita");
    // E o topo reto, antes de a tampa começar.
    na_pele(&f, [0.0, s, 0.0], "o topo reto");
}

/// ⭐⭐ **Na cerca (`half_span = 2·half_width`) ele é um MEIO-DISCO**, e continua a acabar em
/// `+half_width`.
#[test]
fn a_delay_at_its_fence_is_a_half_disc_that_still_ends_at_its_own_box() {
    let w = 0.25_f64;
    let f = campo(um_atraso(w as f32, 2.0 * w as f32));
    na_pele(&f, [w, 0.0, 0.0], "a ponta continua na caixa");
    na_pele(&f, [-w, 0.0, 0.0], "a face reta");
    dentro(&f, [0.0, 0.0, 0.0], "o meio");
    fora(&f, [w + 0.02, 0.0, 0.0], "para lá da ponta");
}

fn um_mostrador(point: f32) -> Primitive {
    Primitive::Display {
        half_width: 0.45,
        half_span: 0.25,
        point,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    }
}

/// ⭐ **O mostrador é o atraso com um BICO** — e o bico é uma ponta, não uma face.
#[test]
fn the_display_closes_in_a_point_on_the_left() {
    let (w, s, point) = (0.45_f64, 0.25_f64, 0.30_f64);
    let f = campo(um_mostrador(point as f32));
    dentro(&f, [0.0, 0.0, 0.0], "o meio");
    na_pele(&f, [-w, 0.0, 0.0], "o bico");
    fora(&f, [-w - 0.02, 0.0, 0.0], "para lá do bico");
    // ⛔ **Os cantos que o atraso tinha** — é isto que separa as duas formas.
    fora(
        &f,
        [-w + 0.02, s - 0.02, 0.0],
        "o canto de cima da face reta do atraso",
    );
    fora(&f, [-w + 0.02, -s + 0.02, 0.0], "o canto de baixo");
    // O flanco do bico, no meio dele: a recta de `(−w, 0)` a `(−w + point, s)`.
    na_pele(
        &f,
        [-w + point * 0.5, s * 0.5, 0.0],
        "o meio do flanco de cima",
    );
    // O topo reto começa onde o bico acaba.
    na_pele(&f, [-w + point + 0.05, s, 0.0], "o topo reto");
    na_pele(&f, [w, 0.0, 0.0], "a tampa redonda continua lá");
}

/// ⭐⭐ **Com o bico a ZERO ele é o ATRASO**, campo a campo.
///
/// ⚠️ **É o gate que prova que o zero é uma FORMA e não uma degeneração** — os dois flancos deitam-se
/// sobre a mesma recta, e a peça é a da outra porta.
#[test]
fn a_display_without_a_point_is_a_delay() {
    let mostrador = campo(um_mostrador(0.0));
    let atraso = campo(um_atraso(0.45, 0.25));
    for (x, y) in [
        (0.0, 0.0),
        (-0.45, 0.0),
        (-0.40, 0.20),
        (0.45, 0.0),
        (-0.5, 0.3),
    ] {
        let a = mostrador.at(x, y, 0.0);
        let b = atraso.at(x, y, 0.0);
        assert!(
            (a - b).abs() < 1.0e-6,
            "sem bico o mostrador é o atraso: ({x}, {y}) leu {a:.6} contra {b:.6}"
        );
    }
}

fn um_conector(point: f32) -> Primitive {
    Primitive::OffPage {
        half_width: 0.35,
        half_span: 0.40,
        point,
        half_height: 0.10,
        round: 0.0,
        chamfer: 0.0,
    }
}

/// ⭐ **O conector é um retângulo que fecha num BICO em baixo.**
#[test]
fn the_off_page_connector_is_a_box_that_closes_in_a_point_below() {
    let (w, s, point) = (0.35_f64, 0.40_f64, 0.30_f64);
    let f = campo(um_conector(point as f32));
    dentro(&f, [0.0, 0.0, 0.0], "o meio");
    na_pele(&f, [w, s, 0.0], "o canto de cima à direita");
    na_pele(&f, [-w, s, 0.0], "o canto de cima à esquerda");
    na_pele(&f, [0.0, -s, 0.0], "o vértice do bico");
    na_pele(
        &f,
        [w, -s + point, 0.0],
        "onde o flanco encontra o lado direito",
    );
    // ⛔ **Os dois cantos de baixo, que o bico comeu.**
    fora(&f, [w - 0.02, -s + 0.02, 0.0], "o canto de baixo à direita");
    fora(&f, [-w - 0.02, -s, 0.0], "o canto de baixo à esquerda");
    // O meio do flanco do bico.
    na_pele(
        &f,
        [w * 0.5, -s + point * 0.5, 0.0],
        "o meio do flanco direito",
    );
}

/// ⭐⭐ **Com o bico a ZERO ele é o RETÂNGULO** — os dois flancos deitam-se sobre a base.
#[test]
fn an_off_page_connector_without_a_point_is_a_rectangle() {
    let f = campo(um_conector(0.0));
    na_pele(&f, [0.35, -0.40, 0.0], "o canto de baixo à direita");
    na_pele(&f, [-0.35, -0.40, 0.0], "o canto de baixo à esquerda");
    dentro(&f, [0.33, -0.38, 0.0], "logo dentro do canto");
}

/// ⭐⭐⭐ **Na cerca (`point = 2·half_span`) ele é um TRIÂNGULO** — e é a mesma primitiva.
#[test]
fn an_off_page_connector_at_its_fence_is_a_triangle() {
    let (w, s) = (0.35_f64, 0.40_f64);
    let f = campo(um_conector(2.0 * s as f32));
    na_pele(&f, [0.0, -s, 0.0], "o vértice de baixo");
    na_pele(&f, [w, s, 0.0], "o canto de cima à direita");
    // ⛔ No triângulo, meia altura acima do vértice a peça tem METADE da largura.
    na_pele(&f, [w * 0.5, 0.0, 0.0], "a meia-largura a meia-altura");
    fora(&f, [w * 0.8, 0.0, 0.0], "para lá dela");
}

/// ⭐⭐ **As quatro são CHAPAS: a espessura é em Z e vale para todas.**
#[test]
fn all_four_are_plates_with_the_same_thickness_law() {
    for p in [
        um_paralelogramo(0.20),
        um_atraso(0.45, 0.25),
        um_mostrador(0.30),
        um_conector(0.30),
    ] {
        let nome = ph2d_field::Primitive::kind(&p).key();
        let f = campo(p);
        na_pele(&f, [0.0, 0.0, 0.10], &format!("«{nome}»: a tampa de cima"));
        na_pele(
            &f,
            [0.0, 0.0, -0.10],
            &format!("«{nome}»: a tampa de baixo"),
        );
        fora(&f, [0.0, 0.0, 0.13], &format!("«{nome}»: acima da tampa"));
        dentro(&f, [0.0, 0.0, 0.07], &format!("«{nome}»: dentro da laje"));
    }
}
