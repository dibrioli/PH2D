//! W23 B5(2) — onde mora cada texto de uma seta (doc 14): o ponto da linha a uma fracção do
//! comprimento, a direcção dela ali, e o ponto da linha mais perto de onde se larga.

use std::f64::consts::FRAC_PI_2;

use super::*;
use ph2d_board_model::TextTurn;

fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

fn line(a: [f64; 2], b: [f64; 2]) -> Routed {
    Routed::from_points(vec![a, b], Route::Straight, [Dir::East, Dir::West])
}

/// ⭐ Numa recta, a fracção é do COMPRIMENTO (o `position` do Miro): 1/4 de (0,0)→(400,0) é (100,0);
/// e numa curva, num cotovelo e numa recta com dobra, o ponto a `f` volta a dar `f`.
#[test]
fn a_caption_sits_on_the_line_at_its_fraction_of_the_length() {
    let r = line([0.0, 0.0], [400.0, 0.0]);
    let (p, d) = r.point_at(0.25);
    // A precisão do comprimento de arco é de 1/100 de unidade (a do meio da rota).
    assert!(close(p[0], 100.0, 1e-2) && close(p[1], 0.0, 1e-2), "{p:?}");
    assert!(close(d[0], 1.0, 1e-9) && close(d[1], 0.0, 1e-9), "{d:?}");
    let pts = vec![[0.0, 0.0], [300.0, 0.0], [300.0, 100.0]];
    let bent = Routed::from_points(pts.clone(), Route::Straight, [Dir::East, Dir::South]);
    // 400 de comprimento: a 3/4 já se desceu 0 da perna de 100 — (300, 0).
    let (p, _) = bent.point_at(0.75);
    assert!(close(p[0], 300.0, 1e-3) && close(p[1], 0.0, 1e-3), "{p:?}");
    let curved = Routed::from_points(pts.clone(), Route::Curved, [Dir::East, Dir::South]);
    let elbow = Routed::from_points(pts, Route::Elbow, [Dir::East, Dir::South]);
    for r in [&bent, &curved, &elbow] {
        for f in [0.0, 0.1, 0.33, 0.5, 0.9, 1.0] {
            let (p, _) = r.point_at(f);
            assert!(r.distance(p) < 1e-3, "{f}: fora da linha");
            assert!(close(r.frac_of(p), f, 1e-3), "{f} → {}", r.frac_of(p));
        }
    }
}

/// ⭐ Ao longo da linha as letras NUNCA ficam de pernas para o ar: uma seta da direita para a
/// esquerda lê-se como a da esquerda para a direita, e uma vertical lê-se de baixo para cima, para
/// onde quer que vá; na horizontal o ângulo é zero.
#[test]
fn along_text_never_reads_upside_down() {
    let back = line([400.0, 0.0], [0.0, 0.0]);
    assert!(close(
        back.caption_frame(0.5, TextTurn::Along).angle,
        0.0,
        1e-9
    ));
    for (a, b) in [([0.0, 0.0], [0.0, 300.0]), ([0.0, 300.0], [0.0, 0.0])] {
        let f = line(a, b).caption_frame(0.5, TextTurn::Along);
        assert!(close(f.angle, -FRAC_PI_2, 1e-9), "{a:?}→{b:?}: {}", f.angle);
    }
    let diag = line([0.0, 0.0], [100.0, 100.0]);
    assert!(close(
        diag.caption_frame(0.5, TextTurn::Along).angle,
        FRAC_PI_2 / 2.0,
        1e-9
    ));
    assert_eq!(diag.caption_frame(0.5, TextTurn::Level).angle, 0.0);
    for d in [
        [1.0, 0.0],
        [-1.0, 0.0],
        [0.0, 1.0],
        [0.0, -1.0],
        [-1.0, -0.2],
    ] {
        let a = upright(d);
        assert!((-FRAC_PI_2..FRAC_PI_2).contains(&a), "{d:?}: {a}");
    }
}

/// ⭐ Largado fora da linha, o texto assenta no ponto da linha MAIS PERTO (a projecção), e o bloco
/// rodado acerta-se no espaço dele: ao longo de uma vertical, o comprimento do texto é vertical.
#[test]
fn a_dropped_caption_lands_on_the_nearest_point_of_the_line() {
    let r = line([0.0, 0.0], [400.0, 0.0]);
    assert!(close(r.frac_of([300.0, 120.0]), 0.75, 1e-4));
    assert_eq!(r.frac_of([-50.0, 10.0]), 0.0);
    assert_eq!(r.frac_of([900.0, -10.0]), 1.0);
    let v = line([0.0, 0.0], [0.0, 400.0]);
    let f = v.caption_frame(0.5, TextTurn::Along);
    let h = 20.0;
    assert!(
        f.contains([0.0, 200.0 + LABEL_WRAP / 2.0 - 1.0], h),
        "ao comprido da linha"
    );
    assert!(
        !f.contains([LABEL_WRAP / 2.0 - 1.0, 200.0], h),
        "não de lado"
    );
    let level = v.caption_frame(0.5, TextTurn::Level);
    assert!(level.contains([LABEL_WRAP / 2.0 - 1.0, 200.0], h));
}

/// ⭐ Smoke do dono (19/10, «as palavras estão retas mas deveriam seguir as curvas»): ao longo de uma
/// CURVA o texto DOBRA com ela — cada ponto da linha de base do texto assenta na linha, a direcção das
/// letras é a da linha ali (as do princípio e as do fim do texto viram de forma diferente), e o espaço
/// do texto vai e volta; numa linha de rectas e na horizontal o texto é rígido.
#[test]
fn along_a_curve_the_text_bends_letter_by_letter() {
    let pts = vec![[0.0, 0.0], [300.0, 300.0]];
    let r = Routed::from_points(pts, Route::Curved, [Dir::East, Dir::South]);
    let f = r.caption_frame(0.5, TextTurn::Along);
    assert!(f.bends());
    let mut angles = Vec::new();
    for u in [-60.0, -30.0, 0.0, 30.0, 60.0] {
        let (p, a) = f.to_world([u, 0.0]);
        assert!(r.distance(p) < 0.05, "{u}: a {} da linha", r.distance(p));
        let (_, t) = r.point_at(r.frac_of(p));
        assert!(
            (upright(t) - a).abs() < 0.02,
            "{u}: {a} contra {}",
            upright(t)
        );
        let back = f.to_local(f.to_world([u, 7.0]).0);
        // Abaixo de um píxel: a curva aplanada a 0,02 e a direcção alisada em ±0,5.
        assert!(
            (back[0] - u).abs() < 0.15 && (back[1] - 7.0).abs() < 0.15,
            "{u}: {back:?}"
        );
        angles.push(a);
    }
    assert!((angles[4] - angles[0]).abs() > 0.2, "rígido: {angles:?}");
    for w in angles.windows(2) {
        assert!((-FRAC_PI_2..FRAC_PI_2).contains(&w[0]));
    }
    assert!(
        !line([0.0, 0.0], [300.0, 0.0])
            .caption_frame(0.5, TextTurn::Along)
            .bends()
    );
    assert!(!r.caption_frame(0.5, TextTurn::Level).bends());
}
