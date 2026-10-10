//! Gates e sonda das QUINAS do lápis — irmão de `pencil_corners.rs`.

use super::*;
use crate::test_hand::{hand, stabilise};
use ph2d_vec_scene::VertexKind;
use std::f64::consts::TAU;

type P2 = [f64; 2];

/// Uma polilinha com UMA viragem de `turn_deg` a meio, lados de `side` px.
fn bend(turn_deg: f64, side: f64) -> Vec<P2> {
    let a = turn_deg.to_radians();
    vec![[-side, 0.0], [0.0, 0.0], [side * a.cos(), side * a.sin()]]
}

fn ring(r: f64, turns: f64) -> Vec<P2> {
    let n = ((TAU * r * turns / 2.0).ceil() as usize).max(24);
    (0..=n)
        .map(|i| {
            let a = TAU * turns * i as f64 / n as f64;
            [r * a.cos(), r * a.sin()]
        })
        .collect()
}

fn s_curve(s: f64) -> Vec<P2> {
    (0..=200)
        .map(|i| {
            let t = f64::from(i) / 200.0;
            [s * t, s / 4.0 * (TAU * t).sin()]
        })
        .collect()
}

/// Uma mão como a da app: tremor, depois o estabilizador por omissão (0,5).
fn drawn(poly: &[P2], amp: f64, seed: u64) -> Vec<P2> {
    stabilise(&hand(poly, 2.0, amp, seed), 0.5)
}

#[test]
fn a_drawn_corner_is_one_cut_where_the_hand_turned() {
    for turn in [60.0, 90.0, 135.0] {
        let p = drawn(&bend(turn, 120.0), 0.5, 1);
        let cuts = corners(&Corners::MEASURED, &p, 1.0);
        assert_eq!(cuts.len(), 1, "viragem {turn}°: {cuts:?}");
        assert!(
            dist(p[cuts[0]], [0.0, 0.0]) < 6.0,
            "a quina longe do vértice: {:?}",
            p[cuts[0]]
        );
    }
}

#[test]
fn smooth_curves_have_no_corners() {
    for p in [ring(30.0, 0.9), ring(150.0, 0.9), s_curve(300.0)] {
        let cuts = corners(&Corners::MEASURED, &drawn(&p, 0.5, 2), 1.0);
        assert!(cuts.is_empty(), "uma curva suave ganhou quinas: {cuts:?}");
    }
}

#[test]
fn the_fit_keeps_the_corner_and_the_curves_stay_smooth() {
    // Um arco e depois uma quina de 90° e uma recta: o nó da quina é Corner com os handles
    // NÃO colineares; os nós do arco continuam Smooth.
    let mut poly = ring(80.0, 0.25);
    let end = poly[poly.len() - 1];
    poly.push([end[0] + 120.0, end[1]]);
    let p = drawn(&poly, 0.3, 3);
    let v = fit(&p, 2.0, 1.0);
    let k = v
        .iter()
        .position(|x| x.kind == VertexKind::Corner && dist(x.anchor, end) < 6.0)
        .expect("a quina do fim do arco");
    let (i, o) = (v[k].in_handle, v[k].out_handle);
    let (a, b) = (
        [i[0] - v[k].anchor[0], i[1] - v[k].anchor[1]],
        [o[0] - v[k].anchor[0], o[1] - v[k].anchor[1]],
    );
    let cos = (a[0] * b[0] + a[1] * b[1]) / (a[0].hypot(a[1]) * b[0].hypot(b[1]));
    assert!(
        cos > -0.5,
        "os handles da quina continuam colineares (cos {cos})"
    );
    assert!(
        v[1..k].iter().all(|x| x.kind == VertexKind::Smooth),
        "o arco perdeu a suavidade"
    );
}

/// **A sonda**: por combinação de (janela, viragem mínima, concentração), a fracção de quinas de
/// mão achadas (viragens 45–150°, lados 25–200 px, tremor ±0,5–2 px, estabilizador 0,5) e as quinas
/// FALSAS em curvas suaves (círculos de raio 10–400 px, S, espiral), 20 mãos por caso.
#[test]
#[ignore = "sonda: cargo test --release -p ph2d-vec-edit --lib measure_pencil_corners -- --ignored --nocapture"]
fn measure_pencil_corners() {
    println!(
        "\n| janela px | viragem ° | concentração | achadas 60°+ | achadas 45° | pior 60°+ (lado 25) | falsas / traço suave |"
    );
    for window in [8.0, 10.0, 12.0, 14.0, 16.0, 20.0] {
        for min_turn in [25.0, 30.0, 35.0, 40.0] {
            for spread in [1.1, 1.2, 1.3, 1.4] {
                let c = Corners {
                    window_px: window,
                    min_turn_deg: min_turn,
                    spread,
                };
                let (mut hit60, mut n60, mut hit45, mut n45, mut worst_short) =
                    (0, 0, 0, 0, 1.0_f64);
                for turn in [45.0, 60.0, 90.0, 120.0, 150.0] {
                    for side in [25.0, 60.0, 200.0] {
                        let mut hits = 0;
                        for amp in [0.5, 1.0, 2.0] {
                            for seed in 0..20 {
                                let p = drawn(&bend(turn, side), amp, seed);
                                let cuts = corners(&c, &p, 1.0);
                                if cuts.len() == 1 && dist(p[cuts[0]], [0.0, 0.0]) < 8.0 {
                                    hits += 1;
                                }
                            }
                        }
                        if turn >= 60.0 {
                            hit60 += hits;
                            n60 += 60;
                            if side < 30.0 {
                                worst_short = worst_short.min(f64::from(hits) / 60.0);
                            }
                        } else {
                            hit45 += hits;
                            n45 += 60;
                        }
                    }
                }
                let (mut false_cuts, mut strokes) = (0usize, 0usize);
                let smooth: Vec<Vec<P2>> = [10.0, 20.0, 40.0, 80.0, 160.0, 400.0]
                    .into_iter()
                    .map(|r| ring(r, 0.9))
                    .chain([s_curve(120.0), s_curve(400.0)])
                    .collect();
                for p in &smooth {
                    for amp in [0.5, 1.0, 2.0] {
                        for seed in 0..20 {
                            false_cuts += corners(&c, &drawn(p, amp, seed), 1.0).len();
                            strokes += 1;
                        }
                    }
                }
                println!(
                    "| {window} | {min_turn} | {spread} | {:.0} % | {:.0} % | {:.0} % | {:.3} |",
                    100.0 * f64::from(hit60) / f64::from(n60),
                    100.0 * f64::from(hit45) / f64::from(n45),
                    100.0 * worst_short,
                    false_cuts as f64 / strokes as f64
                );
            }
        }
    }
}

#[test]
fn small_circles_turn_fast_but_have_no_corners() {
    // Num círculo de raio 10–20 px a janela de 14 px já vira mais que a viragem mínima: só a
    // CONCENTRAÇÃO o separa de uma quina (na janela tripla ele vira o triplo).
    for r in [10.0, 15.0, 20.0] {
        for seed in 0..10 {
            let cuts = corners(&Corners::MEASURED, &drawn(&ring(r, 0.9), 0.5, seed), 1.0);
            assert!(
                cuts.is_empty(),
                "círculo de raio {r}: quinas falsas {cuts:?}"
            );
        }
    }
}

#[test]
fn a_flick_at_the_end_of_the_stroke_is_not_a_corner() {
    // A mão que levanta faz um gancho de poucos px: dentro da janela da ponta, não é quina.
    let p = drawn(&[[0.0, 0.0], [200.0, 0.0], [200.0, 8.0]], 0.3, 4);
    let cuts = corners(&Corners::MEASURED, &p, 1.0);
    assert!(
        cuts.is_empty(),
        "o gancho virou quina em {:?}",
        cuts.first().map(|&i| p[i])
    );
}

#[test]
fn a_sixty_degree_corner_survives_the_stabiliser_in_every_hand() {
    // O estabilizador arredonda a quina; uma janela curta (8 px) já a perde em ~1/3 das mãos.
    for seed in 0..20 {
        let p = drawn(&bend(60.0, 60.0), 1.0, seed);
        let cuts = corners(&Corners::MEASURED, &p, 1.0);
        assert!(
            cuts.len() == 1 && dist(p[cuts[0]], [0.0, 0.0]) < 8.0,
            "mão {seed}: {cuts:?}"
        );
    }
}

#[test]
fn a_trembling_hand_does_not_put_corners_in_a_smooth_curve() {
    // Com tremor de ±2 px, uma janela curta (8 px) lê o tremor como quina; a de 14 não.
    let curves = [
        ring(10.0, 0.9),
        ring(40.0, 0.9),
        ring(160.0, 0.9),
        s_curve(120.0),
        s_curve(400.0),
    ];
    for (k, c) in curves.iter().enumerate() {
        for seed in 0..20 {
            let cuts = corners(&Corners::MEASURED, &drawn(c, 2.0, seed), 1.0);
            assert!(
                cuts.is_empty(),
                "curva {k}, mão {seed}: quinas falsas {cuts:?}"
            );
        }
    }
}
