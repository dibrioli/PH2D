//! **As sondas do ARCO do QuickShape** (F7c) — irmão de `quickshape_arc_tests.rs` (os gates), pelo
//! tecto de 700 LOC. Correm à mão: `cargo test --release -p ph2d-vec-edit --lib measure_ --
//! --ignored --nocapture --test-threads=1`. As tabelas: `00_resultados.md` §3g.

use super::arc_tests::{arc_poly, asymmetric, deviation, is_the_arc, kasa, not_arcs, stroke};
use super::tests::{failures, load};
use super::*;
use crate::quickshape_arc::taubin;
use crate::test_hand::Lcg;

// ── as sondas ───────────────────────────────────────────────────────────────────────────────────

const SWEEPS: [f64; 11] = [
    30.0, 45.0, 60.0, 90.0, 120.0, 180.0, 240.0, 270.0, 300.0, 315.0, 330.0,
];
const RADII: [f64; 5] = [20.0, 40.0, 80.0, 160.0, 400.0];

/// A fracção de `seeds` mãos (início ao acaso, os dois sentidos) que sai o arco.
fn arc_rate(law: &Law, r: f64, sweep: f64, amp: f64, seeds: u64) -> f64 {
    (0..seeds)
        .filter(|&seed| {
            let sw = if seed % 2 == 0 { sweep } else { -sweep };
            let mut g = Lcg(seed + 5);
            let poly = arc_poly(r, 180.0 * g.next(), sw);
            is_the_arc(
                &recognise_with(law, &stroke(&poly, amp, seed), 1.0, false),
                sw,
            )
        })
        .count() as f64
        / seeds as f64
}

/// O pior «não é arco» que sai arco, sobre 48–400 px e tremor ±0,5–2.
fn worst_false_arc(law: &Law, seeds: u64) -> (f64, &'static str) {
    let mut worst = (0.0, "");
    for s in [48.0, 160.0, 400.0] {
        for amp in [0.5, 1.0, 2.0] {
            for k in 0..4 {
                let n = (0..seeds)
                    .filter(|&seed| {
                        let (_, p) = &not_arcs(s, seed)[k];
                        matches!(
                            recognise_with(law, &stroke(p, amp, seed), 1.0, false),
                            Some(QuickShape::Arc { .. })
                        )
                    })
                    .count() as f64
                    / seeds as f64;
                if n > worst.0 {
                    worst = (n, not_arcs(s, 0)[k].0);
                }
            }
        }
    }
    worst
}

/// **Os arcos de mão**: a fracção que sai arco por varrido × raio, a três tremores (40 mãos/célula).
#[test]
#[ignore = "sonda"]
fn measure_arcs() {
    for amp in [0.5, 1.0, 2.0] {
        println!("\n## tremor ±{amp} px, o estabilizador, 40 mãos por célula");
        print!("| varrido |");
        for r in RADII {
            print!(" r {r} |");
        }
        println!();
        for sweep in SWEEPS {
            print!("| {sweep}° |");
            for r in RADII {
                print!(
                    " {:.0} % |",
                    100.0 * arc_rate(&Law::MEASURED, r, sweep, amp, 40)
                );
            }
            println!();
        }
        // O que os arcos que FALHAM viram: livre e linha são inofensivos, outra forma não.
        let (mut free, mut line, mut loose, mut other) = (0, 0, 0, Vec::new());
        for sweep in SWEEPS {
            for r in RADII {
                for seed in 0..40 {
                    let mut g = Lcg(seed + 5);
                    let poly = arc_poly(r, 180.0 * g.next(), sweep);
                    match recognise(&stroke(&poly, amp, seed), 1.0, false) {
                        q if is_the_arc(&q, sweep) => {}
                        None => free += 1,
                        Some(QuickShape::Line { .. }) => line += 1,
                        Some(QuickShape::Arc { .. }) => loose += 1,
                        Some(QuickShape::Ellipse { .. }) if sweep >= 330.0 => {}
                        q => other.push(format!("{sweep}° r {r}: {q:?}")),
                    }
                }
            }
        }
        println!(
            "falhas: livre {free} · linha {line} · arco com o varrido a > 10 % {loose} · OUTRA forma {}",
            other.len()
        );
        for o in other.iter().take(6) {
            println!("  {}", o.chars().take(160).collect::<String>());
        }
    }
    let (w, name) = worst_false_arc(&Law::MEASURED, 40);
    println!(
        "\npior «não é arco» lido como arco: {:.1} % ({name})",
        100.0 * w
    );
}

/// **O preço do arco nas formas que já existiam**: A × B no mesmo processo — sem arco
/// (`arc_rms_max = 0`) e com ele —, as formas da sonda de sensibilidade, 80 mãos, 24–400 px.
#[test]
#[ignore = "sonda"]
fn measure_arc_ab() {
    let off = Law {
        arc_rms_max: 0.0,
        ..Law::MEASURED
    };
    for (name, law) in [("sem arco", off), ("com arco", Law::MEASURED)] {
        println!("\n## {name}");
        print!("| tremor |");
        for (n, _, _) in super::measure_tests::cases(1.0) {
            print!(" {n} |");
        }
        println!();
        for amp in [1.0, 2.0, 4.0] {
            let mut acc: Vec<f64> = Vec::new();
            for s in [24.0, 48.0, 160.0, 400.0] {
                for (i, (_, r)) in super::measure_tests::hit_rate(&law, s, amp, 80)
                    .into_iter()
                    .enumerate()
                {
                    if acc.len() <= i {
                        acc.push(0.0);
                    }
                    acc[i] += r / 4.0;
                }
            }
            print!("| ±{amp} |");
            for r in acc {
                print!(" {:.1} % |", 100.0 * r);
            }
            println!();
        }
    }
}

/// **As três constantes JUNTAS** (o resíduo e a viragem para trás seguram o rabisco a meias): o meio
/// arco da sonda de sensibilidade (mão sem estabilizador, 24–400 px), os arcos de 45–300° com o
/// estabilizador, e o pior «não é arco» lido como arco (40 mãos).
#[test]
#[ignore = "sonda"]
fn measure_arc_joint() {
    println!("\n| rms | volta atrás | cordas | meio arco ±2 | ±4 | arcos ±2 | pior «não é arco» |");
    for chords in [4, 6, 8] {
        for bend in [0.15, 0.2, 0.25, 0.3] {
            for rms in [0.03, 0.04, 0.05, 0.06] {
                let law = Law {
                    arc_rms_max: rms,
                    arc_bend_back: bend,
                    arc_chords: chords,
                    ..Law::MEASURED
                };
                let half = |amp: f64| {
                    [24.0, 48.0, 160.0, 400.0]
                        .iter()
                        .map(|&s| {
                            super::measure_tests::hit_rate(&law, s, amp, 40)
                                .into_iter()
                                .find(|(n, _)| *n == "meio arco")
                                .map_or(0.0, |x| x.1)
                        })
                        .sum::<f64>()
                        / 4.0
                };
                let (mut arcs, mut n) = (0.0, 0.0);
                for sweep in [45.0, 90.0, 180.0, 270.0, 300.0] {
                    for r in [40.0, 80.0, 160.0, 400.0] {
                        arcs += arc_rate(&law, r, sweep, 2.0, 20);
                        n += 1.0;
                    }
                }
                let (w, wn) = worst_false_arc(&law, 40);
                println!(
                    "| {rms} | {bend} | {chords} | {:.1} % | {:.1} % | {:.1} % | {:.1} % ({wn}) |",
                    100.0 * half(2.0),
                    100.0 * half(4.0),
                    100.0 * arcs / n,
                    100.0 * w
                );
            }
        }
    }
}

/// **A faixa de cada constante nova**: fixtures, o acerto médio e o pior nos arcos de 45–300° a
/// 40–400 px (±1 px), e o pior «não é arco» lido como arco.
#[test]
#[ignore = "sonda"]
fn measure_arc_constants() {
    let all = load();
    type Set = fn(&mut Law, f64);
    let fields: Vec<(&str, Set, Vec<f64>)> = vec![
        (
            "arc_rms_max",
            |l, v| l.arc_rms_max = v,
            vec![0.005, 0.01, 0.015, 0.02, 0.03, 0.04, 0.05, 0.07, 0.1, 0.15],
        ),
        (
            "arc_bend_back",
            |l, v| l.arc_bend_back = v,
            vec![0.0, 0.02, 0.05, 0.1, 0.15, 0.2, 0.3, 0.4, 0.5],
        ),
        (
            "arc_chords",
            |l, v| l.arc_chords = v as usize,
            vec![3.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0],
        ),
    ];
    println!(
        "\n| constante | valor | fixtures | arco médio ±1 | pior célula ±1 | média ±2 | pior «não é arco» |"
    );
    for (name, set, values) in fields {
        for v in values {
            let mut law = Law::MEASURED;
            set(&mut law, v);
            let f = failures(&all, &law).len();
            let (mut sum, mut n, mut worst, mut sum2) = (0.0, 0.0, 1.0_f64, 0.0);
            for sweep in [45.0, 90.0, 180.0, 270.0, 300.0] {
                for r in [40.0, 80.0, 160.0, 400.0] {
                    let x = arc_rate(&law, r, sweep, 1.0, 20);
                    sum += x;
                    sum2 += arc_rate(&law, r, sweep, 2.0, 20);
                    n += 1.0;
                    worst = worst.min(x);
                }
            }
            let (w, wn) = worst_false_arc(&law, 20);
            println!(
                "| `{name}` | {v} | {f} | {:.1} % | {:.0} % | {:.1} % | {:.1} % ({wn}) |",
                100.0 * sum / n,
                100.0 * worst,
                100.0 * sum2 / n,
                100.0 * w
            );
        }
    }
}

#[test]
#[ignore = "sonda"]
fn measure_taubin_against_kasa() {
    println!("\n| varrido | r | tremor | erro médio do raio Taubin | Kåsa |");
    for sweep in [30.0, 45.0, 60.0, 90.0] {
        for r in [80.0, 300.0] {
            for amp in [1.0, 2.0] {
                let (mut t, mut k) = (0.0, 0.0);
                for seed in 0..40 {
                    let pts = stroke(&arc_poly(r, 0.0, sweep), amp, seed);
                    t += taubin(&pts).unwrap().1 - r;
                    k += kasa(&pts).1 - r;
                }
                println!(
                    "| {sweep}° | {r} | ±{amp} | {:+.2} | {:+.2} |",
                    t / 40.0,
                    k / 40.0
                );
            }
        }
    }
}

/// Arcos de ELIPSE (fora do escopo, a menos que o círculo não chegue): o que a lei faz deles.
#[test]
#[ignore = "sonda"]
fn measure_elliptic_arcs() {
    println!(
        "\n| razão | arco | eixo | arco de círculo | livre | outra | pior desvio do traço ao arco (px) |"
    );
    for ratio in [1.25, 1.5, 2.0, 3.0] {
        for sweep in [90.0_f64, 180.0] {
            for (axis, a0) in [("maior", 180.0_f64), ("menor", 90.0)] {
                let (rx, ry) = (160.0, 160.0 / ratio);
                let n = 200;
                let poly: Vec<P2> = (0..=n)
                    .map(|i| {
                        let a = (a0 + sweep * f64::from(i) / f64::from(n)).to_radians();
                        [rx * a.cos(), ry * a.sin()]
                    })
                    .collect();
                let (mut arc, mut free, mut other, mut worst) = (0, 0, 0, 0.0_f64);
                for seed in 0..20 {
                    let pts = stroke(&poly, 1.0, seed);
                    match recognise(&pts, 1.0, false) {
                        Some(q @ (QuickShape::Arc(_) | QuickShape::Biarc(_))) => {
                            arc += 1;
                            worst = worst.max(deviation(&q, &poly));
                        }
                        None => free += 1,
                        _ => other += 1,
                    }
                }
                println!(
                    "| {ratio}:1 | {sweep}° | {axis} | {arc}/20 | {free} | {other} | {worst:.1} |"
                );
            }
        }
    }
}

#[test]
#[ignore = "sonda"]
fn measure_asymmetric_arcs() {
    println!("\n| traço | tremor | arco | de elipse | biarco | pior desvio (px) | o resto |");
    for (name, poly) in asymmetric() {
        for amp in [1.0, 2.0] {
            let (mut arc, mut ell, mut bi, mut worst, mut rest) = (0, 0, 0, 0.0_f64, Vec::new());
            for seed in 0..20 {
                match recognise(&stroke(&poly, amp, seed), 1.0, false) {
                    Some(q @ (QuickShape::Arc(_) | QuickShape::Biarc(_))) => {
                        arc += 1;
                        match q {
                            QuickShape::Arc(a) if a.rx != a.ry => ell += 1,
                            QuickShape::Biarc(_) => bi += 1,
                            _ => {}
                        }
                        worst = worst.max(deviation(&q, &poly));
                    }
                    None => rest.push("livre"),
                    Some(QuickShape::Line { .. }) => rest.push("linha"),
                    Some(QuickShape::Polygon { .. }) => rest.push("polígono"),
                    Some(QuickShape::Ellipse { .. }) => rest.push("elipse fechada"),
                }
            }
            rest.sort_unstable();
            rest.dedup();
            println!(
                "| {name} | ±{amp} | {arc}/20 | {ell} | {bi} | {worst:.1} | {} |",
                rest.join(" ")
            );
        }
        // O preço de UM reconhecimento (o segurar faz um por gesto): o MÍNIMO de 20.
        let pts = stroke(&poly, 1.0, 0);
        let best = (0..20)
            .map(|_| {
                let t = std::time::Instant::now();
                std::hint::black_box(recognise(&pts, 1.0, false));
                t.elapsed().as_micros()
            })
            .min()
            .unwrap_or(0);
        println!("|   ↳ um reconhecimento: {best} µs | | | | | | |");
    }
}

/// Um arco SIMÉTRICO de mão que RESPIRA: o raio `r·(1 + b·sin(lobos·a + fase))`.
fn breathing(r: f64, sweep: f64, b: f64, lobes: f64, phase: f64) -> Vec<P2> {
    (0..=200)
        .map(|i| {
            let a = (30.0 + sweep * f64::from(i) / 200.0).to_radians();
            mul(unit(a), r * (1.0 + b * (lobes * a + phase).sin()))
        })
        .collect()
}

/// As faixas de `arc_circle_keep` × `arc_asymmetric_gain`, JUNTAS: os arcos de CÍRCULO — limpos e que
/// RESPIRAM (±2–6 %, 1,5–4 lobos, quatro fases) — lidos como assimétricos (mal); os assimétricos
/// que saem arco e o PIOR desvio deles; os «não é arco».
#[test]
#[ignore = "sonda"]
fn measure_arc_circle_keep() {
    let all = load();
    println!(
        "\n| guarda | ganho | fixtures | círculos lidos assimétricos | que respiram | assimétricos arco | pior desvio (px) | pior «não é arco» |"
    );
    let not_round = |q: &Option<QuickShape>| {
        matches!(q, Some(QuickShape::Arc(a)) if a.rx != a.ry)
            || matches!(q, Some(QuickShape::Biarc(_)))
    };
    for keep in [0.0, 0.01, 0.015, 0.02, 0.025, 0.03, 0.04] {
        for gain in [0.1, 0.15, 0.2, 0.25, 0.3, 0.35] {
            let law = Law {
                arc_asymmetric_gain: gain,
                arc_circle_keep: keep,
                ..Law::MEASURED
            };
            let (mut plain, mut n) = (0.0, 0.0);
            for sweep in [45.0, 90.0, 180.0, 270.0] {
                for r in [40.0, 160.0, 400.0] {
                    for seed in 0..10 {
                        for amp in [1.0, 2.0] {
                            let q = recognise_with(
                                &law,
                                &stroke(&arc_poly(r, 30.0, sweep), amp, seed),
                                1.0,
                                false,
                            );
                            plain += f64::from(u8::from(not_round(&q)));
                            n += 1.0;
                        }
                    }
                }
            }
            let (mut breath, mut nb) = (0.0, 0.0);
            for sweep in [90.0, 130.0, 180.0, 270.0] {
                for r in [80.0, 160.0, 400.0] {
                    for b in [0.02, 0.04, 0.06] {
                        for lobes in [1.5, 2.5, 4.0] {
                            for phase in [0.0, 1.6, 3.1, 4.7] {
                                let poly = breathing(r, sweep, b, lobes, phase);
                                let q = recognise_with(&law, &stroke(&poly, 1.0, 1), 1.0, false);
                                breath += f64::from(u8::from(not_round(&q)));
                                nb += 1.0;
                            }
                        }
                    }
                }
            }
            let (mut arcs, mut tot, mut worst) = (0.0_f64, 0.0, 0.0_f64);
            for (_, poly) in asymmetric() {
                for seed in 0..10 {
                    tot += 1.0;
                    let q = recognise_with(&law, &stroke(&poly, 1.0, seed), 1.0, false);
                    if let Some(q @ (QuickShape::Arc(_) | QuickShape::Biarc(_))) = q {
                        arcs += 1.0;
                        // O desvio em fracção do tamanho do traço (os ganchos vão de ½× a 2×).
                        let (lo, hi) =
                            poly.iter()
                                .fold(([f64::MAX; 2], [f64::MIN; 2]), |(l, h), p| {
                                    (
                                        [l[0].min(p[0]), l[1].min(p[1])],
                                        [h[0].max(p[0]), h[1].max(p[1])],
                                    )
                                });
                        worst = worst.max(deviation(&q, &poly) / dist(lo, hi));
                    }
                }
            }
            let (w, wn) = worst_false_arc(&law, 20);
            println!(
                "| {keep} | {gain} | {} | {:.1} % | {:.1} % | {:.1} % | {:.1} % | {:.1} % ({wn}) |",
                failures(&all, &law).len(),
                100.0 * plain / n,
                100.0 * breath / nb,
                100.0 * arcs / tot,
                100.0 * worst,
                100.0 * w
            );
        }
    }
}
