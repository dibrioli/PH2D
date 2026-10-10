//! Gates e sondas do ARCO do QuickShape (F7c) — irmão de `quickshape_tests.rs` (tecto de 700 LOC).
//! Sondas: `cargo test --release -p ph2d-vec-edit --lib measure_arc -- --ignored --nocapture
//! --test-threads=1`.

use super::tests::{failures, load};
use super::*;
use crate::quickshape_arc::{arc_path, taubin};
use crate::test_hand::{Lcg, hand, stabilise};
use ph2d_vec_scene::VertexKind;
use std::f64::consts::PI;

/// Um arco de raio `r` à volta da origem, de `a0` a varrer `sweep` (graus, com sinal), a 1 px.
fn arc_poly(r: f64, a0: f64, sweep: f64) -> Vec<P2> {
    let n = ((sweep.to_radians().abs() * r).ceil() as usize).max(8);
    (0..=n)
        .map(|i| mul(unit((a0 + sweep * i as f64 / n as f64).to_radians()), r))
        .collect()
}

/// Uma mão: o tremor `amp` e o estabilizador por omissão do lápis.
fn stroke(poly: &[P2], amp: f64, seed: u64) -> Vec<P2> {
    stabilise(&hand(poly, 2.0, amp, seed), 0.5)
}

/// O S de dois meios círculos, o S de seno e um rabisco (o do `gera_entradas.py`): não são arcos.
fn not_arcs(s: f64, seed: u64) -> Vec<(&'static str, Vec<P2>)> {
    let mut s2 = arc_poly(s / 4.0, 90.0, -180.0);
    s2.extend(
        arc_poly(s / 4.0, 90.0, 180.0)
            .iter()
            .skip(1)
            .map(|&p| add(p, [0.0, -s / 2.0])),
    );
    let sine: Vec<P2> = (0..=100)
        .map(|i| {
            let t = f64::from(i) / 100.0;
            [s * t, s / 4.0 * (TAU * t).sin()]
        })
        .collect();
    let mut g = Lcg(seed + 11);
    let mut scribble = vec![[0.0, 0.0]];
    for _ in 0..30 {
        let q = scribble[scribble.len() - 1];
        scribble.push([
            (q[0] + 0.1 * s * g.next()).clamp(-s / 2.0, s / 2.0),
            (q[1] + 0.1 * s * g.next()).clamp(-s / 2.0, s / 2.0),
        ]);
    }
    vec![
        ("S de dois meios círculos", s2),
        ("S de seno", sine),
        ("rabisco", scribble),
        ("volta e meia", arc_poly(s / 2.0, 0.0, 540.0)),
    ]
}

/// O arco que a mão quis (raio `r`, varrendo `sweep` graus)? Tipo, sentido e o varrido a ≤ 10 %.
fn is_the_arc(q: &Option<QuickShape>, sweep: f64) -> bool {
    matches!(q, Some(QuickShape::Arc { sweep: s, .. })
        if s.signum() == sweep.signum() && (s.to_degrees() - sweep).abs() <= 0.1 * sweep.abs())
}

// ── os gates ────────────────────────────────────────────────────────────────────────────────────

#[test]
fn taubin_returns_the_exact_circle_of_a_short_arc() {
    let p: Vec<P2> = arc_poly(300.0, 20.0, 40.0)
        .iter()
        .map(|&q| add(q, [50.0, -70.0]))
        .collect();
    let (c, r) = taubin(&p).unwrap();
    assert!(dist(c, [50.0, -70.0]) < 1e-6 && (r - 300.0).abs() < 1e-6, "{c:?} {r}");
}

#[test]
fn a_hand_drawn_arc_becomes_a_perfect_arc_in_its_own_sense() {
    for sweep in [45.0, 90.0, 180.0, 270.0, -60.0, -200.0] {
        for seed in 0..10 {
            let pts = stroke(&arc_poly(150.0, 17.0 * seed as f64, sweep), 1.0, seed);
            let q = recognise(&pts, 1.0, false);
            assert!(is_the_arc(&q, sweep), "{sweep}° (mão {seed}): {q:?}");
            let Some(QuickShape::Arc { centre, radius, .. }) = q else {
                unreachable!()
            };
            assert!(
                dist(centre, [0.0, 0.0]) < 6.0 && (radius - 150.0).abs() < 6.0,
                "{sweep}°: centro {centre:?} raio {radius}"
            );
            assert!(dist(q.as_ref().unwrap().pivot(), pts[0]) < 3.0, "o pivô é o início");
        }
    }
}

#[test]
fn what_is_not_an_arc_stays_what_it_was() {
    for s in [48.0, 160.0, 400.0] {
        for seed in 0..10 {
            for (name, poly) in not_arcs(s, seed) {
                let q = recognise(&stroke(&poly, 1.0, seed), 1.0, false);
                assert!(
                    !matches!(q, Some(QuickShape::Arc { .. })),
                    "{name} {s} px (mão {seed}) virou arco: {q:?}"
                );
            }
        }
    }
    // A flecha de 3,3 % que o oráculo endireita continua LINHA (`linha_arco_leve`).
    let q = recognise(&arc_poly(2270.0, -97.6, 15.2), 1.0, false);
    assert!(matches!(q, Some(QuickShape::Line { .. })), "{q:?}");
}

#[test]
fn a_c_that_almost_closes_closes_into_a_circle_and_a_wider_one_stays_an_arc() {
    for seed in 0..10 {
        let shut = recognise(&stroke(&arc_poly(150.0, 0.0, 335.0), 1.0, seed), 1.0, false);
        assert!(
            matches!(shut, Some(QuickShape::Ellipse { rx, ry, .. }) if rx == ry),
            "335° (mão {seed}): {shut:?}"
        );
        let open = recognise(&stroke(&arc_poly(150.0, 0.0, 300.0), 1.0, seed), 1.0, false);
        assert!(is_the_arc(&open, 300.0), "300° (mão {seed}): {open:?}");
    }
}

#[test]
fn regular_snaps_the_sweep_and_the_chord_to_fifteen_degrees_and_keeps_the_start() {
    let pts = stroke(&arc_poly(150.0, 8.0, 97.0), 1.0, 3);
    let Some(QuickShape::Arc {
        centre,
        radius,
        start,
        sweep,
    }) = recognise(&pts, 1.0, true)
    else {
        panic!()
    };
    let step = REGULAR_STEP_DEG.to_radians();
    assert!((sweep - FRAC_PI_2).abs() < 1e-9, "97° → 90°: {}", sweep.to_degrees());
    let p0 = add(centre, mul(unit(start), radius));
    let p1 = add(centre, mul(unit(start + sweep), radius));
    let chord = angle(sub(p1, p0));
    assert!((chord - snap(chord, step)).abs() < 1e-9, "a corda a 15°: {}", chord.to_degrees());
    assert!(dist(p0, pts[0]) < 1e-9, "o início fica onde a mão o pôs");
}

#[test]
fn the_arc_path_is_open_smooth_inside_cornered_at_the_ends_and_on_the_circle() {
    // 92° cabe num troço (a folga de 5 %): o meio arco de uma mão tem 181° e três nós.
    for (sweep, n) in [(PI / 3.0, 2), (0.51 * PI, 2), (-PI, 3), (1.5 * PI, 4), (1.9 * PI, 5)] {
        let p = arc_path([10.0, 20.0], 100.0, 0.3, sweep);
        assert!(!p.closed && p.verts.len() == n, "{sweep}: {} nós", p.verts.len());
        let last = p.verts.len() - 1;
        for (i, v) in p.verts.iter().enumerate() {
            let want = if i == 0 || i == last {
                VertexKind::Corner
            } else {
                VertexKind::Smooth
            };
            assert_eq!(v.kind, want, "nó {i}");
            assert!((dist(v.anchor, [10.0, 20.0]) - 100.0).abs() < 1e-9);
        }
        assert!(dist(p.verts[0].anchor, add([10.0, 20.0], mul(unit(0.3), 100.0))) < 1e-9);
        // O meio de cada cúbica cai no círculo (a ≤ 0,03 % do raio para troços de ≤ 90°).
        for w in p.verts.windows(2) {
            let (a, b, c, d) = (w[0].anchor, w[0].out_handle, w[1].in_handle, w[1].anchor);
            let mid = mul(add(add(a, d), mul(add(b, c), 3.0)), 0.125);
            assert!((dist(mid, [10.0, 20.0]) - 100.0).abs() < 0.03, "{sweep}: {mid:?}");
        }
    }
    let a = QuickShape::Arc {
        centre: [0.0, 0.0],
        radius: 2.0,
        start: 0.0,
        sweep: PI,
    };
    assert_eq!(a.pivot(), [2.0, 0.0]);
    let QuickShape::Arc {
        centre,
        radius,
        start,
        sweep,
    } = a.transformed([2.0, 0.0], 2.0, FRAC_PI_2)
    else {
        panic!()
    };
    assert!(dist(centre, [2.0, -4.0]) < 1e-12 && radius == 4.0 && sweep == PI);
    assert!((start - FRAC_PI_2).abs() < 1e-12, "a rotação anda o início");
}

#[test]
fn the_pencil_holds_an_arc_and_the_drag_turns_it_about_its_start() {
    let mut scene = ph2d_vec_scene::VecScene::default();
    let mut pencil = crate::Pencil::default();
    let pts = stroke(&arc_poly(150.0, 180.0, 180.0), 1.0, 1);
    let t = super::pencil_tests::draw(&mut pencil, &mut scene, &pts);
    let id = pencil.active_path().unwrap();
    assert_eq!(
        pencil.hold_tick(&mut scene, t + crate::quickshape_hold::HOLD_NS, false, true),
        crate::quickshape_hold::HoldTick::Recognised
    );
    let p = scene.path(id).unwrap();
    assert!(!p.closed && p.verts.len() == 3, "o caminho vivo É o meio arco: {}", p.verts.len());
    let start = p.verts[0].anchor;
    // O fim roda 90° à volta do INÍCIO, à mesma distância: o arco roda inteiro, o início fica.
    let end = pts[pts.len() - 1];
    let turned = add(start, rotate(sub(end, start), FRAC_PI_2));
    pencil.on_drag(
        &mut scene,
        turned,
        crate::pencil_width::PenDynamics {
            pressure: 1.0,
            t_ns: t,
        },
    );
    let p = scene.path(id).unwrap();
    assert!(dist(p.verts[0].anchor, start) < 1e-9, "o pivô é o início");
    assert!(dist(p.verts[2].anchor, turned) < 3.0, "{:?} vs {turned:?}", p.verts[2].anchor);
    assert!(pencil.on_release(&mut scene));
    let p = scene.path(id).unwrap();
    assert!(!p.closed && p.verts.len() == 3, "soltar comete o ARCO");
    assert_eq!(p.verts[1].kind, VertexKind::Smooth);
    assert_eq!(p.verts[0].kind, VertexKind::Corner);
}

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
            is_the_arc(&recognise_with(law, &stroke(&poly, amp, seed), 1.0, false), sw)
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
                print!(" {:.0} % |", 100.0 * arc_rate(&Law::MEASURED, r, sweep, amp, 40));
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
    println!("\npior «não é arco» lido como arco: {:.1} % ({name})", 100.0 * w);
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
    println!("\n| constante | valor | fixtures | arco médio ±1 | pior célula ±1 | média ±2 | pior «não é arco» |");
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

// ── Taubin × Kåsa e os arcos de elipse ─────────────────────────────────────────────────────────

/// Com tremor, Kåsa ENCOLHE o raio de um arco curto e Taubin não (`measure_taubin_against_kasa`:
/// 45°, raio 80, ±2 px — Taubin −0,06 px, Kåsa −3,5 px em média).
#[test]
fn taubin_does_not_shrink_a_short_noisy_arc_as_kasa_does() {
    let (mut t, mut k) = (0.0, 0.0);
    for seed in 0..40 {
        let pts = stroke(&arc_poly(80.0, 0.0, 45.0), 2.0, seed);
        t += (taubin(&pts).unwrap().1 - 80.0) / 40.0;
        k += (kasa(&pts).1 - 80.0) / 40.0;
    }
    assert!(k < -2.5, "controlo: Kåsa encolhe {k:+.2} px");
    assert!(t.abs() < 1.0, "Taubin erra {t:+.2} px");
}

/// As duas guardas do S são CAMADAS, e cada uma segura sozinha: o S raso (onda de 10 %) só a
/// viragem para trás o pára; sem ela, o resíduo relativo ao MENOR de raio e tamanho pára o S fundo
/// (relativo ao raio enorme do ajuste ele passava).
#[test]
fn each_guard_of_the_s_holds_alone() {
    let shallow: Vec<P2> = (0..=200)
        .map(|i| {
            let t = f64::from(i) / 200.0;
            [300.0 * t, 30.0 * (TAU * t).sin()]
        })
        .collect();
    let no_bend = Law {
        arc_bend_back: 1.0,
        ..Law::MEASURED
    };
    for seed in 0..10 {
        let q = recognise(&stroke(&shallow, 1.0, seed), 1.0, false);
        assert!(!matches!(q, Some(QuickShape::Arc { .. })), "S raso (mão {seed}): {q:?}");
        for (name, poly) in not_arcs(160.0, seed).into_iter().take(2) {
            let q = recognise_with(&no_bend, &stroke(&poly, 1.0, seed), 1.0, false);
            assert!(!matches!(q, Some(QuickShape::Arc { .. })), "{name} (mão {seed}): {q:?}");
        }
    }
}

/// O `Shift` nunca troca o TIPO: a polilinha de três lados com quinas de 30° (a régua da fusão é
/// 29°) que a lei lê polilinha continua polilinha com o `regular` — medida COM o regular, a
/// polilinha regularizada ficava mais longe do traço e o arco ganhava.
#[test]
fn the_shift_never_turns_a_polyline_into_an_arc() {
    let mut poly = vec![[0.0, 0.0]];
    for j in 0..3 {
        let q = poly[poly.len() - 1];
        poly.push(add(q, mul(unit((5.0 + 30.0 * f64::from(j)).to_radians()), 150.0)));
    }
    for seed in [1, 4, 6, 8, 9] {
        let pts = stroke(&poly, 1.0, seed);
        for regular in [false, true] {
            let q = recognise(&pts, 1.0, regular);
            assert!(
                matches!(q, Some(QuickShape::Polygon { closed: false, .. })),
                "regular {regular} (mão {seed}): {q:?}"
            );
        }
    }
}

/// O ajuste algébrico de Kåsa (o controlo do gate de Taubin).
fn kasa(p: &[P2]) -> (P2, f64) {
    let (mut m, mut v) = ([[0.0; 3]; 3], [0.0; 3]);
    for &[x, y] in p {
        let row = [x, y, 1.0];
        let z = -(x * x + y * y);
        for i in 0..3 {
            v[i] += row[i] * z;
            for j in 0..3 {
                m[i][j] += row[i] * row[j];
            }
        }
    }
    let s = solve(m, v).unwrap();
    let c = [-s[0] / 2.0, -s[1] / 2.0];
    (c, (c[0] * c[0] + c[1] * c[1] - s[2]).sqrt())
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
                println!("| {sweep}° | {r} | ±{amp} | {:+.2} | {:+.2} |", t / 40.0, k / 40.0);
            }
        }
    }
}

/// Arcos de ELIPSE (fora do escopo, a menos que o círculo não chegue): o que a lei faz deles.
#[test]
#[ignore = "sonda"]
fn measure_elliptic_arcs() {
    println!("\n| razão | arco | eixo | arco de círculo | livre | outra | pior desvio do traço ao arco (px) |");
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
                        Some(QuickShape::Arc { centre, radius, .. }) => {
                            arc += 1;
                            for &p in &poly {
                                worst = worst.max((dist(p, centre) - radius).abs());
                            }
                        }
                        None => free += 1,
                        _ => other += 1,
                    }
                }
                println!("| {ratio}:1 | {sweep}° | {axis} | {arc}/20 | {free} | {other} | {worst:.1} |");
            }
        }
    }
}
