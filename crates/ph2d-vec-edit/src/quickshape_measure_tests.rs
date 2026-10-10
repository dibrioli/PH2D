//! **As sondas do QuickShape** — `cargo test -p ph2d-vec-edit --lib measure_ -- --ignored --nocapture`.
//!
//! 1. [`measure_small_shapes`]: o tamanho a partir do qual a lei acerta sob TREMOR de mão (ordem do
//!    dono, 09/10: *«formas pequenas devem funcionar»*) — o piso `MIN_SIZE_PX` sai daqui.
//! 2. [`measure_the_law_ranges`]: a faixa de cada constante do [`Law`] em que as 65 fixtures do
//!    oráculo e os traços de mão continuam certos, varrida sozinha, no MESMO processo.

use super::tests::{failures, load};
use super::*;

pub(super) use crate::test_hand::hand;

fn ring(rx: f64, ry: f64, turns: f64) -> Vec<P2> {
    (0..=200)
        .map(|i| {
            let a = TAU * turns * f64::from(i) / 200.0;
            [rx * a.cos(), ry * a.sin()]
        })
        .collect()
}

/// Arredonda cada quina interior de uma polilinha: corta `f` de cada lado vizinho e liga os cortes
/// por uma Bézier quadrática (a quina de uma mão rápida).
fn rounded(poly: &[P2], f: f64) -> Vec<P2> {
    let mut out = vec![poly[0]];
    for w in poly.windows(3) {
        let (a, v, b) = (w[0], w[1], w[2]);
        let (p, q) = (add(v, mul(sub(a, v), f)), add(v, mul(sub(b, v), f)));
        for k in 0..=6 {
            let t = f64::from(k) / 6.0;
            let (m1, m2) = (add(p, mul(sub(v, p), t)), add(v, mul(sub(q, v), t)));
            out.push(add(m1, mul(sub(m2, m1), t)));
        }
    }
    out.push(poly[poly.len() - 1]);
    out
}

/// As formas de mão de tamanho `s` (px) e o que a lei tem de responder. `None` = não é forma.
/// Um caso: o nome, a polilinha da mão e o que a lei tem de responder.
pub(super) type Case = (&'static str, Vec<P2>, fn(&Option<QuickShape>) -> bool);

pub(super) fn cases(s: f64) -> Vec<Case> {
    let h = 0.62 * s;
    let tri: Vec<P2> = (0..=3)
        .map(|k| {
            let a = FRAC_PI_2 + TAU * f64::from(k) / 3.0;
            [s / 2.0 * a.cos(), s / 2.0 * a.sin()]
        })
        .collect();
    let s_curve: Vec<P2> = (0..=100)
        .map(|i| {
            let t = f64::from(i) / 100.0;
            [s * t, s / 4.0 * (TAU * t).sin()]
        })
        .collect();
    vec![
        (
            "rectângulo",
            vec![[0.0, 0.0], [s, 0.0], [s, h], [0.0, h], [0.0, 0.0]],
            |q| matches!(q, Some(QuickShape::Polygon { corners, closed: true }) if corners.len() == 4),
        ),
        (
            "triângulo",
            tri,
            |q| matches!(q, Some(QuickShape::Polygon { corners, closed: true }) if corners.len() == 3),
        ),
        (
            "círculo",
            ring(s / 2.0, s / 2.0, 1.0),
            |q| matches!(q, Some(QuickShape::Ellipse { rx, ry, .. }) if rx == ry),
        ),
        (
            "elipse 2:1",
            ring(s / 2.0, s / 4.0, 1.0),
            |q| matches!(q, Some(QuickShape::Ellipse { rx, ry, .. }) if rx != ry),
        ),
        ("linha", vec![[0.0, 0.0], [s, 0.3 * s]], |q| {
            matches!(q, Some(QuickShape::Line { .. }))
        }),
        (
            "zigue-zague",
            vec![
                [0.0, 0.0],
                [s / 4.0, h],
                [s / 2.0, 0.0],
                [0.75 * s, h],
                [s, 0.0],
            ],
            |q| matches!(q, Some(QuickShape::Polygon { corners, closed: false }) if corners.len() == 5),
        ),
        (
            "zigue-zague redondo",
            rounded(
                &[
                    [0.0, 0.0],
                    [s / 4.0, h],
                    [s / 2.0, 0.0],
                    [0.75 * s, h],
                    [s, 0.0],
                ],
                0.15,
            ),
            |q| matches!(q, Some(QuickShape::Polygon { corners, closed: false }) if corners.len() == 5),
        ),
        (
            "L",
            vec![[0.0, h], [0.0, 0.0], [s, 0.0]],
            |q| matches!(q, Some(QuickShape::Polygon { corners, closed: false }) if corners.len() == 3),
        ),
        ("S (não é forma)", s_curve, Option::is_none),
        (
            "meio arco (não é forma)",
            ring(s / 2.0, s / 2.0, 0.5),
            Option::is_none,
        ),
    ]
}

/// A fracção de acertos de cada caso, sobre `seeds` mãos, a um tamanho e tremor.
fn hit_rate(law: &Law, s: f64, amp: f64, seeds: u64) -> Vec<(&'static str, f64)> {
    cases(s)
        .into_iter()
        .map(|(name, poly, ok)| {
            let hits = (0..seeds)
                .filter(|&seed| {
                    ok(&recognise_with(
                        law,
                        &hand(&poly, 2.0, amp, seed),
                        1.0,
                        false,
                    ))
                })
                .count();
            (name, hits as f64 / seeds as f64)
        })
        .collect()
}

#[test]
#[ignore = "sonda: corre à mão com --ignored --nocapture"]
fn measure_small_shapes() {
    let law = Law {
        min_size_px: 0.0,
        ..Law::MEASURED
    };
    for amp in [0.5, 1.0, 2.0, 4.0] {
        println!("\n## tremor ±{amp} px (média móvel de 5), amostras a 2 px, 40 mãos por célula");
        print!("| tamanho |");
        for (n, _, _) in cases(1.0) {
            print!(" {n} |");
        }
        println!();
        for s in [
            8.0, 12.0, 16.0, 20.0, 24.0, 32.0, 40.0, 48.0, 64.0, 96.0, 160.0,
        ] {
            print!("| {s} px |");
            for (_, r) in hit_rate(&law, s, amp, 40) {
                print!(" {:.0} % |", 100.0 * r);
            }
            println!();
        }
    }
}

/// Os traços de mão que não estão no corpus do oráculo, e o que têm de dar — o lado da sonda de
/// faixas que as fixtures não perguntam.
fn hand_cases_hold(law: &Law) -> bool {
    // 48 px é onde a elipse 2:1 com ±1 px de tremor já acerta (98 % a 32 px — `measure_small_shapes`).
    [48.0, 160.0, 400.0].into_iter().all(|s| {
        hit_rate(law, s, 1.0, 8)
            .iter()
            .all(|(_, r)| (*r - 1.0).abs() < 1e-9)
    })
}

#[test]
#[ignore = "sonda: corre à mão com --ignored --nocapture"]
fn measure_the_law_ranges() {
    let all = load();
    assert!(
        failures(&all, &Law::MEASURED).is_empty(),
        "a lei medida já falha"
    );
    type Set = fn(&mut Law, f64);
    let grid = |lo: f64, hi: f64, n: usize| -> Vec<f64> {
        (0..=n)
            .map(|i| lo + (hi - lo) * i as f64 / n as f64)
            .collect()
    };
    let fields: Vec<(&str, f64, Set, Vec<f64>)> = vec![
        (
            "corner_px",
            Law::MEASURED.corner_px,
            |l, v| l.corner_px = v,
            grid(20.0, 200.0, 18),
        ),
        (
            "corner_of_diag",
            Law::MEASURED.corner_of_diag,
            |l, v| l.corner_of_diag = v,
            grid(0.02, 0.4, 19),
        ),
        (
            "rdp_of_diag",
            Law::MEASURED.rdp_of_diag,
            |l, v| l.rdp_of_diag = v,
            grid(0.005, 0.12, 23),
        ),
        (
            "merge_turn_deg",
            Law::MEASURED.merge_turn_deg,
            |l, v| l.merge_turn_deg = v,
            grid(2.0, 44.0, 21),
        ),
        (
            "bow_max",
            Law::MEASURED.bow_max,
            |l, v| l.bow_max = v,
            grid(0.02, 0.1, 16),
        ),
        (
            "bow_trim",
            Law::MEASURED.bow_trim,
            |l, v| l.bow_trim = v,
            grid(0.0, 0.3, 12),
        ),
        (
            "fit_trim",
            Law::MEASURED.fit_trim,
            |l, v| l.fit_trim = v,
            grid(0.0, 0.4, 16),
        ),
        (
            "close_of_length",
            Law::MEASURED.close_of_length,
            |l, v| l.close_of_length = v,
            grid(0.01, 0.15, 28),
        ),
        (
            "close_window",
            Law::MEASURED.close_window,
            |l, v| l.close_window = v,
            grid(0.0, 0.3, 12),
        ),
        (
            "right_tol_deg",
            Law::MEASURED.right_tol_deg,
            |l, v| l.right_tol_deg = v,
            grid(1.0, 20.0, 19),
        ),
        (
            "axis_snap_deg",
            Law::MEASURED.axis_snap_deg,
            |l, v| l.axis_snap_deg = v,
            grid(0.0, 10.0, 20),
        ),
        (
            "ellipse_rms_max",
            Law::MEASURED.ellipse_rms_max,
            |l, v| l.ellipse_rms_max = v,
            grid(0.005, 0.2, 39),
        ),
        (
            "round_ratio",
            Law::MEASURED.round_ratio,
            |l, v| l.round_ratio = v,
            grid(1.0, 1.8, 16),
        ),
        (
            "turn_tol_deg",
            Law::MEASURED.turn_tol_deg,
            |l, v| l.turn_tol_deg = v,
            grid(10.0, 300.0, 29),
        ),
        (
            "open_corner_of_side",
            Law::MEASURED.open_corner_of_side,
            |l, v| l.open_corner_of_side = v,
            grid(0.05, 1.0, 19),
        ),
        (
            "min_size_px",
            Law::MEASURED.min_size_px,
            |l, v| l.min_size_px = v,
            grid(0.0, 120.0, 30),
        ),
    ];
    println!("\n| constante | medida | faixa válida contígua | o 1.º a cair em cada ponta |");
    for (name, at, set, values) in fields {
        let verdict = |v: f64| {
            let mut law = Law::MEASURED;
            set(&mut law, v);
            let f = failures(&all, &law);
            if f.is_empty() && hand_cases_hold(&law) {
                None
            } else {
                Some(
                    f.first()
                        .cloned()
                        .unwrap_or_else(|| "um traço de mão".to_owned()),
                )
            }
        };
        let ok: Vec<(f64, Option<String>)> = values.iter().map(|&v| (v, verdict(v))).collect();
        // A faixa CONTÍGUA que contém o valor medido.
        let k = ok
            .iter()
            .position(|(v, _)| *v >= at)
            .unwrap_or(ok.len() - 1);
        let (mut lo, mut hi) = (k, k);
        while lo > 0 && ok[lo - 1].1.is_none() {
            lo -= 1;
        }
        while hi + 1 < ok.len() && ok[hi + 1].1.is_none() {
            hi += 1;
        }
        let below = lo
            .checked_sub(1)
            .and_then(|i| ok[i].1.clone())
            .unwrap_or_else(|| "—".into());
        let above = ok
            .get(hi + 1)
            .and_then(|x| x.1.clone())
            .unwrap_or_else(|| "—".into());
        println!(
            "| `{name}` | {at} | {:.4} … {:.4} | ↓ {} · ↑ {} |",
            ok[lo].0,
            ok[hi].0,
            below.chars().take(60).collect::<String>(),
            above.chars().take(60).collect::<String>()
        );
    }
}

#[test]
#[ignore = "sonda"]
fn measure_the_s_curve() {
    let all = load();
    println!(
        "\n| quina/lado | fixtures | pior zigue/L ≤48 ±≤1 | pior zigue/L ≥160 | pior zigue/L ≤48 ±2 | pior zigue REDONDO ±≤1 | pior S ≥48 | pior S <48 |"
    );
    for k in [f64::INFINITY, 1.0, 0.8, 0.6, 0.5, 0.4, 0.3, 0.2] {
        let law = Law {
            min_size_px: 0.0,
            open_corner_of_side: k,
            ..Law::MEASURED
        };
        let f = failures(
            &all,
            &Law {
                open_corner_of_side: k,
                ..Law::MEASURED
            },
        )
        .len();
        let mut v = [1.0_f64; 6];
        for s in [16.0, 24.0, 48.0, 160.0, 400.0] {
            for amp in [0.5, 1.0, 2.0] {
                for (n, r) in hit_rate(&law, s, amp, 40) {
                    let poly = n == "zigue-zague" || n == "L";
                    let slot = if n == "zigue-zague redondo" {
                        if amp <= 1.0 { 3 } else { continue }
                    } else if poly && s <= 48.0 && amp <= 1.0 {
                        0
                    } else if poly && s >= 160.0 {
                        1
                    } else if poly {
                        2
                    } else if n.starts_with('S') && s >= 48.0 {
                        4
                    } else if n.starts_with('S') {
                        5
                    } else {
                        continue;
                    };
                    v[slot] = v[slot].min(r);
                }
            }
        }
        println!(
            "| {k} | {f} | {:.0} % | {:.0} % | {:.0} % | {:.0} % | {:.0} % | {:.0} % |",
            100.0 * v[0],
            100.0 * v[1],
            100.0 * v[2],
            100.0 * v[3],
            100.0 * v[4],
            100.0 * v[5]
        );
    }
}

/// O que sai quando uma forma pequena FALHA: ficar traço livre é inofensivo, virar OUTRA forma não.
#[test]
#[ignore = "sonda"]
fn measure_what_small_failures_become() {
    let law = Law {
        min_size_px: 0.0,
        ..Law::MEASURED
    };
    println!("\n| tamanho | ±tremor | caso | livre | outra forma |");
    for s in [8.0, 12.0, 16.0, 20.0, 24.0, 32.0] {
        for amp in [1.0, 2.0] {
            for (name, poly, ok) in cases(s) {
                let (mut free, mut wrong) = (0, 0);
                for seed in 0..40 {
                    let q = recognise_with(&law, &hand(&poly, 2.0, amp, seed), 1.0, false);
                    if !ok(&q) {
                        if q.is_none() { free += 1 } else { wrong += 1 }
                    }
                }
                if free + wrong > 0 {
                    println!("| {s} | {amp} | {name} | {free} | {wrong} |");
                }
            }
        }
    }
}

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
    let candidates = [
        ("A medida", Law::MEASURED),
        (
            "B",
            Law {
                bow_max: 0.06,
                bow_trim: 0.05,
                ellipse_rms_max: 0.075,
                turn_tol_deg: 150.0,
                ..Law::MEASURED
            },
        ),
        (
            "C",
            Law {
                bow_max: 0.06,
                bow_trim: 0.0,
                ellipse_rms_max: 0.075,
                turn_tol_deg: 150.0,
                ..Law::MEASURED
            },
        ),
        (
            "D",
            Law {
                bow_max: 0.065,
                bow_trim: 0.05,
                ellipse_rms_max: 0.075,
                turn_tol_deg: 150.0,
                ..Law::MEASURED
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
