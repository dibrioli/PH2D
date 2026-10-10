//! Gates das cláusulas de MÃO do QuickShape — as que as fixtures do oráculo não perguntam e a prova
//! de mutação pediu (o quadrado regular, a ponta da linha, duas voltas, o bico, a dobra, o chanfro,
//! o gancho, o lado partido pelo início). Irmão de `quickshape_tests.rs` (tecto de 700 LOC).

use super::tests::{hand, worst};
use super::*;

#[test]
fn the_regular_square_keeps_the_drawn_corners_and_sits_on_the_axes() {
    // Um rectângulo tosco desenhado no sentido HORÁRIO (y para cima), inclinado ~1,4°.
    let rect = hand(
        &[
            [0.0, 0.0],
            [0.0, 260.0],
            [400.0, 270.0],
            [400.0, 10.0],
            [0.0, 0.0],
        ],
        1.0,
    );
    let Some(QuickShape::Polygon {
        corners,
        closed: true,
    }) = recognise(&rect, 1.0, true)
    else {
        panic!("rect")
    };
    let drawn = [[0.0, 0.0], [0.0, 260.0], [400.0, 270.0], [400.0, 10.0]];
    let radius = dist(corners[0], centroid(&corners));
    for (i, (&c, &d)) in corners.iter().zip(&drawn).enumerate() {
        assert!(
            dist(c, d) < 0.35 * radius,
            "canto {i} {c:?} longe do desenhado {d:?}: o sentido inverteu"
        );
    }
    let side = angle(sub(corners[1], corners[0]));
    assert!(
        remainder_quarter(side) < 1e-9,
        "o quadrado endireita aos eixos: lado a {}°",
        side.to_degrees()
    );
}

fn remainder_quarter(a: f64) -> f64 {
    (a - snap(a, FRAC_PI_2)).abs()
}

#[test]
fn a_line_starts_where_the_hand_started() {
    // Da direita para a esquerda: o `a` (o pivô do arrasto) é a ponta onde a mão POUSOU.
    let l = hand(&[[400.0, 0.0], [0.0, 200.0]], 0.5);
    let Some(QuickShape::Line { a, b }) = recognise(&l, 1.0, false) else {
        panic!("linha")
    };
    assert!(
        dist(a, l[0]) < 2.0 && dist(b, l[l.len() - 1]) < 2.0,
        "{a:?} → {b:?}"
    );
}

#[test]
fn a_loop_drawn_twice_is_not_a_circle() {
    // Duas voltas fecham no início e têm os momentos de UM círculo — só a viragem total as separa.
    let two: Vec<P2> = (0..=400)
        .map(|i| {
            let a = 2.0 * TAU * f64::from(i) / 400.0;
            [200.0 * a.cos(), 200.0 * a.sin()]
        })
        .collect();
    assert!(recognise(&two, 1.0, false).is_none());
    // Controlo: UMA volta, a mesma mão, é o círculo.
    assert!(matches!(
        recognise(&two[..=200], 1.0, false),
        Some(QuickShape::Ellipse { .. })
    ));
}

#[test]
fn a_sharp_spike_is_two_sides_not_one_line() {
    // Um «V» de 20° de abertura: a viragem é de 160°, e sem SENTIDO leria-se como 20° e fundia-se.
    let half = 10f64.to_radians();
    let tip = [400.0 * half.cos(), 400.0 * half.sin()];
    let v = hand(&[[0.0, 0.0], tip, [0.0, 2.0 * 400.0 * half.sin()]], 0.3);
    assert!(
        matches!(recognise(&v, 1.0, false), Some(QuickShape::Polygon { ref corners, closed: false }) if corners.len() == 3),
        "{:?}",
        recognise(&v, 1.0, false)
    );
}

#[test]
fn a_side_drawn_with_a_slight_kink_is_still_one_side() {
    // O lado de cima dobra 12° ao meio: o RDP põe lá uma quina (desvio > 4 % da diagonal) e é a
    // FUSÃO das viragens pequenas que a tira — sem ela saía um pentágono.
    let kink = [200.0, 250.0 + 200.0 * 6f64.to_radians().tan()];
    let r = hand(
        &[
            [0.0, 0.0],
            [400.0, 0.0],
            [400.0, 250.0],
            kink,
            [0.0, 250.0],
            [0.0, 0.0],
        ],
        0.5,
    );
    assert!(
        matches!(recognise(&r, 1.0, false), Some(QuickShape::Polygon { ref corners, closed: true }) if corners.len() == 4),
        "{:?}",
        recognise(&r, 1.0, false)
    );
}

#[test]
fn a_chamfered_corner_is_absorbed_where_it_sits() {
    // Quinas cortadas a 55 px (o chanfro tem 78 px < 94): cada chanfro é QUINA, e a quina fica no MEIO
    // dele — os quatro cantos saem no rectângulo de 400×250 que a mão queria.
    let c = 55.0;
    let r = hand(
        &[
            [c, 0.0],
            [400.0 - c, 0.0],
            [400.0, c],
            [400.0, 250.0 - c],
            [400.0 - c, 250.0],
            [c, 250.0],
            [0.0, 250.0 - c],
            [0.0, c],
            [c, 0.0],
        ],
        0.3,
    );
    let Some(QuickShape::Polygon {
        corners,
        closed: true,
    }) = recognise(&r, 1.0, false)
    else {
        panic!("{:?}", recognise(&r, 1.0, false))
    };
    let want = [[0.0, 0.0], [400.0, 0.0], [400.0, 250.0], [0.0, 250.0]];
    assert!(worst(&corners, &want) < 2.0, "{corners:?}");
}

#[test]
fn a_hook_at_the_end_does_not_tilt_the_line() {
    // 30° e um gancho de 60 px perpendicular no fim: sem o reajuste sem os pontos fora, a recta inclina.
    let a = 30f64.to_radians();
    let end = [500.0 * a.cos(), 500.0 * a.sin()];
    let hook = add(end, mul([a.sin(), -a.cos()], 60.0));
    let l = hand(&[[0.0, 0.0], end, hook], 0.3);
    let Some(QuickShape::Line { a: p, b: q }) = recognise(&l, 1.0, false) else {
        panic!("linha")
    };
    let got = angle(sub(q, p)).to_degrees();
    assert!((got - 30.0).abs() < 0.2, "a recta inclinou para {got}°");
}

#[test]
fn a_side_split_by_the_start_is_fitted_from_both_halves() {
    // Começado a MEIO do lado de cima, e a mão volta 12 px abaixo de onde partiu: as duas metades
    // são UM lado, e a recta dele é a média (y = 250), não só a última metade (y = 244).
    let r = hand(
        &[
            [200.0, 256.0],
            [400.0, 256.0],
            [400.0, 0.0],
            [0.0, 0.0],
            [0.0, 244.0],
            [200.0, 244.0],
        ],
        0.3,
    );
    let Some(QuickShape::Polygon {
        corners,
        closed: true,
    }) = recognise(&r, 1.0, false)
    else {
        panic!("{:?}", recognise(&r, 1.0, false))
    };
    let want = [[0.0, 250.0], [400.0, 250.0], [400.0, 0.0], [0.0, 0.0]];
    assert!(worst(&corners, &want) < 1.5, "{corners:?}");
}

#[test]
fn a_hand_drawn_s_is_not_a_zigzag() {
    // As cristas de um S cabem nas quinas-faixa e os troços entre elas são quase rectos: sem a
    // proporção quina/lado das polilinhas abertas ele saía zigue-zague em 12–38 % destas mãos.
    let (_, s, _) = super::measure_tests::cases(160.0)
        .into_iter()
        .find(|c| c.0.starts_with('S'))
        .expect("o caso do S");
    let zigzags = (0..40)
        .filter(|&seed| {
            recognise(&super::measure_tests::hand(&s, 2.0, 1.0, seed), 1.0, false).is_some()
        })
        .count();
    assert!(zigzags <= 2, "{zigzags} de 40 mãos leram o S como forma");
}

#[test]
fn a_wobbly_hand_circle_is_still_a_circle() {
    // O raio treme em duas frequências altas (resíduo rms ~4,8 %): com a elipse no limite apertado da
    // faixa (0,04) isto
    // ficava traço livre — a «pouca sensibilidade» do smoke do dono.
    let c: Vec<P2> = (0..=240)
        .map(|i| {
            let a = TAU * f64::from(i) / 240.0;
            let r = 150.0 * (1.0 + 0.05 * (11.0 * a).sin() + 0.045 * (17.0 * a + 0.7).sin());
            [r * a.cos(), r * a.sin()]
        })
        .collect();
    assert!(
        matches!(recognise(&c, 1.0, false), Some(QuickShape::Ellipse { rx, ry, .. }) if rx == ry),
        "{:?}",
        recognise(&c, 1.0, false)
    );
}

#[test]
fn a_long_thin_rectangle_keeps_its_short_sides() {
    // 1200×150 px: a quina RELATIVA (0,2 × diagonal = 242 px) engoliria os lados de 150 px; o tecto
    // absoluto do oráculo (94 px) é que os deixa ser lados.
    let r = hand(
        &[
            [0.0, 0.0],
            [1200.0, 0.0],
            [1200.0, 150.0],
            [0.0, 150.0],
            [0.0, 0.0],
        ],
        0.5,
    );
    assert!(
        matches!(recognise(&r, 1.0, false), Some(QuickShape::Polygon { ref corners, closed: true }) if corners.len() == 4),
        "{:?}",
        recognise(&r, 1.0, false)
    );
}

#[test]
fn the_solver_pivots_past_a_zero_on_the_diagonal() {
    let x = crate::quickshape_geom::solve([[0.0, 1.0], [1.0, 0.0]], [2.0, 3.0]);
    assert_eq!(x, Some([3.0, 2.0]));
}

#[test]
fn regular_keeps_a_star_a_star() {
    // Pentagrama numa só passada (ponta k → k+2): o regular dá o pentagrama REGULAR, não o pentágono.
    let tip = |k: usize, r: f64| {
        let a = FRAC_PI_2 + TAU * k as f64 / 5.0 + 0.05 * k as f64;
        [r * a.cos(), r * a.sin()]
    };
    let poly: Vec<P2> = (0..=5)
        .map(|i| tip((2 * i) % 5, 150.0 + 8.0 * i as f64))
        .collect();
    let star = hand(&poly, 0.5);
    let Some(QuickShape::Polygon {
        corners,
        closed: true,
    }) = recognise(&star, 1.0, true)
    else {
        panic!("{:?}", recognise(&star, 1.0, true))
    };
    assert_eq!(corners.len(), 5);
    let c = centroid(&corners);
    for i in 0..5 {
        let turn = wrap_pi(angle(sub(corners[(i + 1) % 5], c)) - angle(sub(corners[i], c)));
        assert!(
            (turn.abs().to_degrees() - 144.0).abs() < 1e-6,
            "de ponta a ponta {}°",
            turn.to_degrees()
        );
    }
    // Controlo: sem `regular` também é estrela (a ordem de desenho, não a convexa).
    let Some(QuickShape::Polygon { corners, .. }) = recognise(&star, 1.0, false) else {
        panic!()
    };
    let c = centroid(&corners);
    let t = wrap_pi(angle(sub(corners[1], c)) - angle(sub(corners[0], c)))
        .abs()
        .to_degrees();
    assert!((t - 144.0).abs() < 10.0, "{t}°");
}

#[test]
fn a_hand_drawn_star_with_sharp_tips_keeps_all_five() {
    // Pentagrama de MÃO: pontas irregulares, algumas mais finas que 29° (a fusão de viragens).
    // Começa numa ponta — o primeiro e o último lado encontram-se nela em sentidos OPOSTOS e não
    // são «o mesmo lado» (a fusão do início lia-os como rectas, sem sentido, e a estrela perdia uma
    // ponta: o smoke do dono, 09/10).
    let jitter = [0.0, -12.0, 10.0, -8.0, 6.0];
    let radius = [150.0, 175.0, 130.0, 165.0, 140.0];
    let tip = |k: usize| {
        let a = (90.0 + 72.0 * k as f64 + jitter[k]).to_radians();
        [radius[k] * a.cos(), radius[k] * a.sin()]
    };
    let poly: Vec<P2> = (0..=5).map(|i| tip((2 * i) % 5)).collect();
    // O controlo: a estrela tem mesmo pontas mais finas que a fusão.
    let sharp = (0..5)
        .filter(|&i| {
            let (a, v, b) = (poly[(i + 4) % 5], poly[i], poly[(i + 1) % 5]);
            let (u, w) = (sub(a, v), sub(b, v));
            (dot(u, w) / (u[0].hypot(u[1]) * w[0].hypot(w[1])))
                .acos()
                .to_degrees()
                < 29.0
        })
        .count();
    assert!(sharp >= 1, "a fixtura não tem ponta fina");
    for amp in [0.5, 1.5] {
        let Some(QuickShape::Polygon {
            corners,
            closed: true,
        }) = recognise(&hand(&poly, amp), 1.0, false)
        else {
            panic!("±{amp}: {:?}", recognise(&hand(&poly, amp), 1.0, false))
        };
        assert_eq!(corners.len(), 5, "±{amp}: {corners:?}");
    }
}
