//! **As sondas da SENSIBILIDADE do QuickShape** (os pedidos do dono de 09/10: «mais sensível»,
//! «estrelas viram quadriláteros», «inclusive com a linha aberta») — irmão de
//! `quickshape_measure_tests.rs` pelo tecto de 700 LOC. Correm à mão:
//! `cargo test --release -p ph2d-vec-edit --lib measure_ -- --ignored --nocapture --test-threads=1`.

use super::measure_tests::{cases, hand, hit_rate, ring};
use super::tests::{failures, load};
use super::*;

/// **A sensibilidade** (o dono, 09/10: *«um pouco mais sensível para todas as formas»*): as três
/// tolerâncias que decidem «isto é forma» — a flecha de um lado recto, o resíduo da elipse e a folga
/// de fecho — varridas JUNTAS. Por combinação: as fixtures que falham, o acerto médio das formas sob
/// tremor forte (±2 e ±4 px, 24–400 px) e o pior «não é forma» (S, meio arco).
#[test]
#[ignore = "sonda"]
fn measure_sensitivity() {
    let all = load();
    println!(
        "\n| bow_max | ellipse_rms_max | close_of_length | fixtures | acerto médio ±2 | acerto médio ±4 | pior «não é forma» |"
    );
    for bow in [0.055, 0.06, 0.065, 0.07] {
        for rms in [0.065, 0.075, 0.085, 0.095, 0.105] {
            for close in [0.06, 0.065, 0.07] {
                let base = Law {
                    bow_max: bow,
                    ellipse_rms_max: rms,
                    close_of_length: close,
                    ..Law::MEASURED
                };
                let f = failures(&all, &base).len();
                let mut mean = [0.0_f64; 2];
                let mut n = [0usize; 2];
                let mut worst_not = 1.0_f64;
                for s in [24.0, 48.0, 160.0, 400.0] {
                    for (k, amp) in [(0usize, 2.0), (1, 4.0)] {
                        for (name, r) in hit_rate(&base, s, amp, 24) {
                            if name.contains("não é forma") {
                                worst_not = worst_not.min(r);
                            } else {
                                mean[k] += r;
                                n[k] += 1;
                            }
                        }
                    }
                    for (name, r) in hit_rate(&base, s, 1.0, 24) {
                        if name.contains("não é forma") {
                            worst_not = worst_not.min(r);
                        }
                    }
                }
                println!(
                    "| {bow} | {rms} | {close} | {f} | {:.1} % | {:.1} % | {:.0} % |",
                    100.0 * mean[0] / n[0] as f64,
                    100.0 * mean[1] / n[1] as f64,
                    100.0 * worst_not
                );
            }
        }
    }
}

/// Quanto cada constante mexe no acerto sob tremor forte, dentro da sua faixa válida (§3d).
#[test]
#[ignore = "sonda"]
fn measure_sensitivity_of_each_constant() {
    let all = load();
    type Set = fn(&mut Law, f64);
    let fields: Vec<(&str, Set, Vec<f64>)> = vec![
        (
            "rdp_of_diag",
            |l, v| l.rdp_of_diag = v,
            vec![0.025, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08, 0.1],
        ),
        (
            "corner_of_diag",
            |l, v| l.corner_of_diag = v,
            vec![0.12, 0.16, 0.2, 0.24, 0.28, 0.32],
        ),
        (
            "merge_turn_deg",
            |l, v| l.merge_turn_deg = v,
            vec![24.0, 29.0, 34.0, 39.0, 44.0],
        ),
        (
            "bow_trim",
            |l, v| l.bow_trim = v,
            vec![0.0, 0.05, 0.1, 0.15, 0.2],
        ),
        ("fit_trim", |l, v| l.fit_trim = v, vec![0.15, 0.175, 0.2]),
        (
            "open_corner_of_side",
            |l, v| l.open_corner_of_side = v,
            vec![0.2, 0.3, 0.4, 0.5],
        ),
        (
            "turn_tol_deg",
            |l, v| l.turn_tol_deg = v,
            vec![50.0, 90.0, 150.0, 300.0],
        ),
        (
            "round_ratio",
            |l, v| l.round_ratio = v,
            vec![1.05, 1.1, 1.2, 1.3],
        ),
        ("bow_max", |l, v| l.bow_max = v, vec![0.055, 0.06, 0.065]),
    ];
    println!("\n| constante | valor | fixtures | acerto médio ±1 | ±2 | ±4 | pior «não é forma» |");
    for (name, set, values) in fields {
        for v in values {
            let mut law = Law::MEASURED;
            set(&mut law, v);
            let f = failures(&all, &law).len();
            let (mut mean, mut n) = ([0.0_f64; 3], 0usize);
            let mut worst_not = 1.0_f64;
            for s in [24.0, 48.0, 160.0, 400.0] {
                for (k, amp) in [(0usize, 1.0), (1, 2.0), (2, 4.0)] {
                    for (case, r) in hit_rate(&law, s, amp, 24) {
                        if case.contains("não é forma") {
                            worst_not = worst_not.min(r);
                        } else {
                            mean[k] += r;
                            if k == 0 {
                                n += 1;
                            }
                        }
                    }
                }
            }
            println!(
                "| `{name}` | {v} | {f} | {:.1} % | {:.1} % | {:.1} % | {:.0} % |",
                100.0 * mean[0] / n as f64,
                100.0 * mean[1] / n as f64,
                100.0 * mean[2] / n as f64,
                100.0 * worst_not
            );
        }
    }
}

/// A × B, forma a forma, 80 mãos por célula: a lei medida contra uma candidata mais sensível.
#[test]
#[ignore = "sonda"]
fn measure_sensitivity_ab() {
    let all = load();
    let base = Law {
        close_of_length: 0.13,
        close_window: 0.13,
        ..Law::MEASURED
    };
    let candidates = [
        ("A folga 0,13", base),
        (
            "B recto 0,065",
            Law {
                bow_max: 0.065,
                ..base
            },
        ),
        (
            "C recto 0,07",
            Law {
                bow_max: 0.07,
                ..base
            },
        ),
        (
            "D recto 0,07 + elipse 0,09",
            Law {
                bow_max: 0.07,
                ellipse_rms_max: 0.09,
                ..base
            },
        ),
    ];
    for (name, law) in &candidates {
        println!(
            "\n## {name}: fixtures a falhar = {}",
            failures(&all, law).len()
        );
        print!("| tremor |");
        for (n, _, _) in cases(1.0) {
            print!(" {n} |");
        }
        println!();
        for amp in [1.0, 2.0, 4.0] {
            let mut acc: Vec<(&str, f64)> = Vec::new();
            for s in [24.0, 48.0, 160.0, 400.0] {
                for (i, (n, r)) in hit_rate(law, s, amp, 80).into_iter().enumerate() {
                    if acc.len() <= i {
                        acc.push((n, 0.0));
                    }
                    acc[i].1 += r / 4.0;
                }
            }
            print!("| ±{amp} |");
            for (_, r) in acc {
                print!(" {:.1} % |", 100.0 * r);
            }
            println!();
        }
    }
}

/// **Estrelas de mão** (o smoke do dono, 09/10): pentagramas irregulares (pontas ±15°, raios ±20 %),
/// começados numa ponta ou a meio de um lado, com o fim aberto ou a passar do início, tremor e o
/// estabilizador por omissão — a fracção que sai com as 5 pontas.
#[test]
#[ignore = "sonda"]
fn measure_hand_stars() {
    println!("\n| começo | fim | tremor | 5 pontas | outra forma | traço livre |");
    for start_mid in [false, true] {
        for end in [-12.0_f64, 0.0, 15.0] {
            for amp in [0.5, 1.0, 2.0] {
                let (mut five, mut other, mut free) = (0, 0, 0);
                for seed in 0..60u64 {
                    let mut g = crate::test_hand::Lcg(seed * 7 + 3);
                    let tips: Vec<P2> = (0..5)
                        .map(|k| {
                            let a = (90.0 + 72.0 * k as f64 + 15.0 * g.next()).to_radians();
                            let r = 150.0 * (1.0 + 0.2 * g.next());
                            [r * a.cos(), r * a.sin()]
                        })
                        .collect();
                    let mut poly: Vec<P2> = (0..=5).map(|i| tips[(2 * i) % 5]).collect();
                    if start_mid {
                        // [m, t2, t4, t1, t3, t0, m]: começa e acaba a meio do lado t0 → t2.
                        let m = mul(add(poly[0], poly[1]), 0.5);
                        poly = vec![m, tips[2], tips[4], tips[1], tips[3], tips[0], m];
                    }
                    // O fim: aberto (−) ou a passar (+) ao longo do primeiro lado, em px.
                    let (a, b) = (poly[poly.len() - 2], poly[poly.len() - 1]);
                    let d = sub(b, a);
                    let u = mul(d, 1.0 / d[0].hypot(d[1]));
                    let f0 = sub(poly[1], poly[0]);
                    let fu = mul(f0, 1.0 / f0[0].hypot(f0[1]));
                    let last = poly.len() - 1;
                    if end < 0.0 {
                        poly[last] = add(b, mul(u, end));
                    } else if end > 0.0 {
                        poly.push(add(b, mul(fu, end)));
                    }
                    let pts = crate::test_hand::stabilise(&hand(&poly, 2.0, amp, seed), 0.5);
                    match recognise(&pts, 1.0, false) {
                        Some(QuickShape::Polygon {
                            corners,
                            closed: true,
                        }) if corners.len() == 5 => five += 1,
                        None => free += 1,
                        _ => other += 1,
                    }
                }
                println!(
                    "| {} | {end} px | ±{amp} | {five}/60 | {other} | {free} |",
                    if start_mid {
                        "meio de um lado"
                    } else {
                        "ponta"
                    }
                );
            }
        }
    }
}

/// Tira a fracção `g` do FIM de uma polilinha (o traço que pára antes de fechar).
fn open_by(poly: &[P2], g: f64) -> Vec<P2> {
    let total: f64 = poly.windows(2).map(|w| dist(w[0], w[1])).sum();
    let mut left = total * (1.0 - g);
    let mut out = vec![poly[0]];
    for w in poly.windows(2) {
        let d = dist(w[0], w[1]);
        if d >= left {
            out.push(add(w[0], mul(sub(w[1], w[0]), left / d)));
            return out;
        }
        left -= d;
        out.push(w[1]);
    }
    out
}

/// **Fechar a linha aberta** (o dono, 09/10: *«mais sensível, inclusive com a linha aberta»*): por
/// folga de fecho, a fracção de formas FECHADAS de mão que fecham com 0–20 % do perímetro em falta,
/// e a fracção de formas que a mão quer ABERTAS (U fundo, C de ¾, V, L, zigue-zague) que fecham à
/// força — 30 mãos por caso, tremor ±1 px, o estabilizador.
#[test]
#[ignore = "sonda"]
fn measure_open_gaps() {
    let all = load();
    let star: Vec<P2> = (0..=5)
        .map(|i| {
            let a = FRAC_PI_2 + TAU * ((2 * i) % 5) as f64 / 5.0;
            [150.0 * a.cos(), 150.0 * a.sin()]
        })
        .collect();
    let closed: Vec<(&str, Vec<P2>, usize)> = vec![
        (
            "rectângulo",
            vec![
                [0.0, 0.0],
                [400.0, 0.0],
                [400.0, 250.0],
                [0.0, 250.0],
                [0.0, 0.0],
            ],
            4,
        ),
        (
            "triângulo",
            vec![[0.0, 0.0], [300.0, 0.0], [150.0, 260.0], [0.0, 0.0]],
            3,
        ),
        ("estrela", star, 5),
        ("círculo", ring(150.0, 150.0, 1.0), 0),
        ("elipse", ring(200.0, 100.0, 1.0), 0),
    ];
    let open: Vec<(&str, Vec<P2>)> = vec![
        (
            "U fundo",
            vec![[0.0, 300.0], [0.0, 0.0], [100.0, 0.0], [100.0, 300.0]],
        ),
        (
            "U",
            vec![[0.0, 200.0], [0.0, 0.0], [150.0, 0.0], [150.0, 200.0]],
        ),
        ("C ¾", ring(150.0, 150.0, 0.75)),
        ("V", vec![[0.0, 300.0], [100.0, 0.0], [200.0, 300.0]]),
        ("L", vec![[0.0, 300.0], [0.0, 0.0], [200.0, 0.0]]),
    ];
    let is_closed = |q: &Option<QuickShape>, k: usize| match q {
        Some(QuickShape::Polygon {
            corners,
            closed: true,
        }) => k == 0 || corners.len() == k,
        Some(QuickShape::Ellipse { .. }) => k == 0,
        _ => false,
    };
    print!("\n| folga | fixtures |");
    for g in [0.04, 0.08, 0.12, 0.16, 0.2] {
        print!(" fecha c/ {:.0} % aberto |", 100.0 * g);
    }
    println!(" pior forma ABERTA fechada à força |");
    for close in [0.06, 0.08, 0.1, 0.12, 0.14, 0.16, 0.2] {
        let law = Law {
            close_of_length: close,
            close_window: close.max(0.1),
            ..Law::MEASURED
        };
        print!("| {close} | {} |", failures(&all, &law).len());
        for g in [0.04, 0.08, 0.12, 0.16, 0.2] {
            let (mut ok, mut n) = (0, 0);
            for (_, p, k) in &closed {
                for seed in 0..30 {
                    let pts =
                        crate::test_hand::stabilise(&hand(&open_by(p, g), 2.0, 1.0, seed), 0.5);
                    ok += usize::from(is_closed(&recognise_with(&law, &pts, 1.0, false), *k));
                    n += 1;
                }
            }
            print!(" {:.0} % |", 100.0 * ok as f64 / n as f64);
        }
        let mut worst = (0.0_f64, "");
        for (name, p) in &open {
            let forced = (0..30)
                .filter(|&seed| {
                    let pts = crate::test_hand::stabilise(&hand(p, 2.0, 1.0, seed), 0.5);
                    matches!(
                        recognise_with(&law, &pts, 1.0, false),
                        Some(QuickShape::Polygon { closed: true, .. } | QuickShape::Ellipse { .. })
                    )
                })
                .count() as f64
                / 30.0;
            if forced > worst.0 {
                worst = (forced, name);
            }
        }
        println!(" {:.0} % ({}) |", 100.0 * worst.0, worst.1);
    }
}
