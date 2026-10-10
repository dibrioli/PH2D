//! **As sondas da SETA do QuickShape** (F7d) — irmão de `quickshape_arrow_tests.rs` (os gates).
//! Correm à mão: `cargo test --release -p ph2d-vec-edit --lib measure_arrow -- --ignored
//! --nocapture --test-threads=1` (cada sonda espalha-se pelos núcleos). As tabelas:
//! `00_resultados.md` §3h.

use super::arc_tests::{arc_poly, asymmetric, not_arcs};
use super::arrow_tests::{
    KINDS, arrow_poly, excalidraw_failures, hausdorff, held_with, is_the_arrow, not_arrows, rest,
    shafts, skewed_arrow,
};
use super::measure_tests::cases;
use super::tests::{failures, load};
use super::*;
use crate::quickshape_arrow::ArrowLaw;
use crate::test_hand::{hand, stabilise};

const SIZES: [f64; 8] = [20.0, 24.0, 32.0, 48.0, 64.0, 96.0, 160.0, 400.0];
const DEGS: [f64; 4] = [20.0, 30.0, 45.0, 60.0];
/// A farpa em fracção do cabo: a seta comprida de ponta pequena (8 %) à curta de ponta grande.
const FRACS: [f64; 4] = [0.08, 0.15, 0.25, 0.4];

/// `f` sobre cada `x`, em paralelo.
fn par<T: Sync, R: Send>(xs: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    std::thread::scope(|sc| {
        let hs: Vec<_> = xs.iter().map(|x| sc.spawn(|| f(x))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

/// A fracção das setas de mão de `s` px que sai a seta certa: os três cabos × as três pontas × uma
/// e duas pontas × `degs` × `fracs` × `seeds` mãos.
fn arrow_rate(
    law: &Law,
    s: f64,
    amp: f64,
    stab: f64,
    degs: &[f64],
    fracs: &[f64],
    seeds: u64,
) -> f64 {
    let (mut ok, mut n) = (0_usize, 0_usize);
    for (name, shaft) in shafts(s) {
        for kind in KINDS {
            for both in [false, true] {
                for &deg in degs {
                    for &frac in fracs {
                        for seed in 0..seeds {
                            // Metade das mãos desenha as farpas ASSIMÉTRICAS (±10°).
                            let skew = if seed % 2 == 1 { 10.0 } else { 0.0 };
                            let p = skewed_arrow(&shaft, frac * s, deg, skew, kind, both);
                            let h = held_with(&p, amp, seed, stab, amp.max(1.0));
                            let q = recognise_with(law, &h, 1.0, false);
                            ok += usize::from(is_the_arrow(&q, name, both));
                            n += 1;
                        }
                    }
                }
            }
        }
    }
    ok as f64 / n as f64
}

/// Os «não é seta» de tamanho `s` e a mão `seed`: os da seta, as formas das sondas, os «não é arco».
fn non_arrows(s: f64, seed: u64) -> Vec<(String, Vec<P2>)> {
    let mut v: Vec<(String, Vec<P2>)> = not_arrows(s, seed)
        .into_iter()
        .map(|(n, p)| (n.to_owned(), p))
        .collect();
    v.extend(
        cases(s)
            .into_iter()
            .map(|(n, p, _)| (format!("forma: {n}"), p)),
    );
    v.extend(
        not_arcs(s, seed)
            .into_iter()
            .map(|(n, p)| (format!("não-arco: {n}"), p)),
    );
    v
}

/// A pior família lida como seta (fracção, nome) e todas as que passam de zero: `sizes`, tremor
/// ±0,5 e ±2, o estabilizador ligado e desligado, o repouso de ±1 e ±3 px.
fn false_arrows(law: &Law, sizes: &[f64], seeds: u64) -> (f64, String) {
    let mut worst = (0.0, String::new());
    let mut all: Vec<String> = Vec::new();
    for &s in sizes {
        let k = non_arrows(s, 0).len();
        for f in 0..k {
            let mut fam = 0.0_f64;
            for amp in [0.5, 2.0] {
                for stab in [0.0, 0.5] {
                    for rest_amp in [1.0, 3.0] {
                        let hits = (0..seeds)
                            .filter(|&seed| {
                                let (_, p) = &non_arrows(s, seed)[f];
                                let h = held_with(p, amp, seed, stab, rest_amp);
                                matches!(
                                    recognise_with(law, &h, 1.0, false),
                                    Some(QuickShape::Arrow(_))
                                )
                            })
                            .count() as f64
                            / seeds as f64;
                        fam = fam.max(hits);
                    }
                }
            }
            let name = format!("{} {s}", non_arrows(s, 0)[f].0);
            if fam > 0.0 {
                all.push(format!("{name} {:.0} %", 100.0 * fam));
            }
            if fam > worst.0 {
                worst = (fam, name);
            }
        }
    }
    (worst.0, all.join(" · "))
}

fn pct(r: &[f64]) -> String {
    r.iter()
        .map(|x| format!("{:.0} %", 100.0 * x))
        .collect::<Vec<_>>()
        .join(" | ")
}

/// ⭐ **As setas de mão**: a fracção que sai seta por tamanho × tremor × estabilizador; por ângulo
/// × farpa; por família; os «não é seta»; o cabo nunca pior; o custo.
#[test]
#[ignore = "sonda"]
fn measure_arrows() {
    let law = Law::MEASURED;
    println!(
        "\n## a seta certa: cabo × tremor (com o estabilizador 0,5 | sem), 3 cabos × 3 pontas × 1–2 pontas × 4 ângulos × 4 farpas × 3 mãos\n"
    );
    println!(
        "| cabo | ±0,5 | ±1 | ±2 | ±0,5 sem | ±1 sem | ±2 sem |\n|---|---|---|---|---|---|---|"
    );
    let rows = par(&SIZES, |&s| {
        [
            (0.5, 0.5),
            (1.0, 0.5),
            (2.0, 0.5),
            (0.5, 0.0),
            (1.0, 0.0),
            (2.0, 0.0),
        ]
        .map(|(amp, stab)| arrow_rate(&law, s, amp, stab, &DEGS, &FRACS, 3))
    });
    for (s, r) in SIZES.iter().zip(rows) {
        println!("| {s} px | {} |", pct(&r));
    }
    println!("\n## por farpa × ângulo (±1 px, estabilizador, cabos de 48–400 px)\n");
    println!("| farpa \\ ângulo | 20° | 30° | 45° | 60° |\n|---|---|---|---|---|");
    let cells = par(&FRACS, |&f| {
        DEGS.map(|d| {
            [48.0, 96.0, 160.0, 400.0]
                .iter()
                .map(|&s| arrow_rate(&law, s, 1.0, 0.5, &[d], &[f], 3))
                .sum::<f64>()
                / 4.0
        })
    });
    for (f, r) in FRACS.iter().zip(cells) {
        println!("| {:.0} % do cabo | {} |", 100.0 * f, pct(&r));
    }
    println!("\n## por família (±1 px, estabilizador, 48–400 px, 4 ângulos × 4 farpas × 3 mãos)\n");
    for (name, _) in shafts(100.0) {
        for kind in KINDS {
            for both in [false, true] {
                let (mut ok, mut n) = (0, 0);
                for s in [48.0, 96.0, 160.0, 400.0] {
                    let shaft = shafts(s).into_iter().find(|x| x.0 == name).unwrap().1;
                    for d in DEGS {
                        for f in FRACS {
                            for seed in 0..3 {
                                let p = arrow_poly(&shaft, f * s, d, kind, both);
                                let h = held_with(&p, 1.0, seed, 0.5, 1.0);
                                let q = recognise(&h, 1.0, false);
                                ok += usize::from(is_the_arrow(&q, name, both));
                                n += 1;
                            }
                        }
                    }
                }
                let r = ok as f64 / f64::from(n);
                println!("- {name} {kind:?} duas={both}: {:.0} %", 100.0 * r);
            }
        }
    }
    let (f, who) = false_arrows(&law, &[24.0, 48.0, 96.0, 160.0, 400.0], 10);
    println!(
        "\n## o pior «não é seta» lido como seta: {:.1} % ({who})",
        100.0 * f
    );
    // O cabo NUNCA pior: o mesmo traço sem a ponta (o prefixo — a mesma mão até à ponta).
    let (mut worse, mut tip, mut head, mut n) = (f64::MIN, 0.0_f64, 0.0_f64, 0);
    for s in [48.0, 160.0, 400.0] {
        for (_, shaft) in shafts(s) {
            for kind in KINDS {
                for seed in 0..10 {
                    let p = arrow_poly(&shaft, 0.2 * s, 30.0, kind, false);
                    let whole = stabilise(&hand(&p, 2.0, 1.0, seed), 0.5);
                    let alone = stabilise(&hand(&shaft, 2.0, 1.0, seed), 0.5);
                    let (Some(QuickShape::Arrow(a)), Some(b)) = (
                        recognise(&rest(&whole, 1.0, 6, seed), 1.0, false),
                        recognise(&rest(&alone, 1.0, 6, seed), 1.0, false),
                    ) else {
                        continue;
                    };
                    n += 1;
                    worse = worse.max(hausdorff(&a.shaft, &shaft) - hausdorff(&b, &shaft));
                    let end = a.shaft.to_path().verts.last().unwrap().anchor;
                    tip = tip.max(dist(end, shaft[shaft.len() - 1]));
                    head = head.max((a.heads[1].unwrap() / (0.2 * s) - 1.0).abs());
                }
            }
        }
    }
    println!(
        "\n## o cabo com ponta − o mesmo cabo sozinho (Hausdorff ao desenho, {n} pares): pior {worse:+.2} px · a ponta a ≤ {tip:.2} px da desenhada · a farpa a ≤ {:.0} % da desenhada",
        100.0 * head
    );
    // O custo: uma seta de 400 px com duas pontas e um rabisco, o mínimo de 20 (release).
    let arrow = arrow_poly(&shafts(400.0)[1].1, 80.0, 30.0, KINDS[0], true);
    let arrow = held_with(&arrow, 1.0, 1, 0.5, 1.0);
    let scribble = held_with(&not_arrows(400.0, 1)[7].1, 1.0, 1, 0.5, 1.0);
    for (name, p) in [
        ("seta de 400 px com duas pontas", arrow),
        ("rabisco de 400 px", scribble),
    ] {
        let best = (0..20)
            .map(|_| {
                let t = std::time::Instant::now();
                std::hint::black_box(recognise(std::hint::black_box(&p), 1.0, false));
                t.elapsed().as_secs_f64() * 1e3
            })
            .fold(f64::MAX, f64::min);
        println!("custo, {name}: {best:.2} ms");
    }
}

/// **O preço nas formas antigas** (A × B no mesmo processo): quantos traços de todas as famílias das
/// sondas (as formas, os «não é arco», os «não é seta», os arcos e os assimétricos) mudam de
/// resposta com a seta ligada.
#[test]
#[ignore = "sonda"]
fn measure_arrow_ab() {
    let on = Law::MEASURED;
    let off = Law { arrow: None, ..on };
    let mut strokes: Vec<(String, Vec<P2>)> = Vec::new();
    for s in [24.0, 48.0, 96.0, 160.0, 400.0] {
        for seed in 0..20 {
            for (name, p) in non_arrows(s, seed) {
                for amp in [1.0, 2.0, 4.0] {
                    let h = held_with(&p, amp, seed, 0.5, amp.max(1.0));
                    strokes.push((format!("±{amp} {s} px {name}"), h));
                }
            }
        }
    }
    for (name, poly) in asymmetric() {
        for seed in 0..20 {
            strokes.push((name.clone(), held_with(&poly, 1.0, seed, 0.5, 1.0)));
        }
    }
    for r in [40.0, 80.0, 160.0] {
        for sweep in [60.0, 120.0, 240.0] {
            for seed in 0..20 {
                let h = held_with(&arc_poly(r, 30.0, sweep), 1.0, seed, 0.5, 1.0);
                strokes.push((format!("arco r {r} {sweep}°"), h));
            }
        }
    }
    let changed: Vec<&str> = strokes
        .iter()
        .filter(|(_, p)| recognise_with(&on, p, 1.0, false) != recognise_with(&off, p, 1.0, false))
        .map(|(n, _)| n.as_str())
        .collect();
    let mut by: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for c in &changed {
        let w: Vec<&str> = c.split(' ').collect();
        let key = format!("{} · {}", w[..3].join(" "), w[3..].join(" "));
        *by.entry(key).or_default() += 1;
    }
    println!(
        "\n## A × B: {} de {} traços mudam de resposta com a seta ligada — por tremor e tamanho: {by:?}",
        changed.len(),
        strokes.len(),
    );
}

/// **As faixas** — cada constante varrida sozinha: as fixtures dos dois oráculos, as setas por
/// tamanho de cabo (20° e 60°, farpas de 8 % e 40 %, ±1 e ±2), e TODOS os «não é seta» lidos seta.
#[test]
#[ignore = "sonda"]
fn measure_arrow_ranges() {
    let all = load();
    type Set = fn(&mut ArrowLaw, f64);
    let only = std::env::var("ARROW_SWEEP").unwrap_or_default();
    let sweeps: [(&str, Set, &[f64]); 8] = [
        (
            "barb_min_deg",
            |a, v| a.barb_min_deg = v,
            &[0.0, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0],
        ),
        (
            "barb_max_deg",
            |a, v| a.barb_max_deg = v,
            &[70.0, 75.0, 80.0, 82.5, 85.0, 90.0, 100.0],
        ),
        (
            "head_min",
            |a, v| a.head_min = v,
            &[0.0, 0.035, 0.04, 0.0475, 0.055, 0.06],
        ),
        (
            "head_max",
            |a, v| a.head_max = v,
            &[0.35, 0.4, 0.7, 0.95, 1.5, 2.0, 3.0],
        ),
        (
            "head_rms",
            |a, v| a.head_rms = v,
            &[0.08, 0.09, 0.1, 0.11, 0.12, 0.13],
        ),
        (
            "barb_min_px",
            |a, v| a.barb_min_px = v,
            &[4.0, 5.0, 5.5, 5.75, 6.0, 7.0],
        ),
        (
            "shaft_fill",
            |a, v| a.shaft_fill = v,
            &[0.0, 0.8, 0.85, 0.875, 0.9, 0.95, 0.97],
        ),
        (
            "tip_return",
            |a, v| a.tip_return = v,
            &[0.05, 0.1, 0.15, 0.2, 0.3, 0.35, 0.45],
        ),
    ];
    for (name, set, values) in sweeps {
        if !only.is_empty() && !only.split(',').any(|o| o == name) {
            continue;
        }
        println!(
            "\n### {name}\n\n| valor | fixtures | Excalidraw | 400 px | 96 px | 48 px | 32 px | 24 px | «não é seta» lidos seta |\n|---|---|---|---|---|---|---|---|---|"
        );
        let rows = par(values, |&v| {
            let mut a = ArrowLaw::MEASURED;
            set(&mut a, v);
            let law = Law {
                arrow: Some(a),
                ..Law::MEASURED
            };
            let rate = |s: f64| {
                [1.0, 2.0]
                    .map(|amp| arrow_rate(&law, s, amp, 0.5, &[20.0, 60.0], &[0.08, 0.4], 2))
                    .iter()
                    .sum::<f64>()
                    / 2.0
            };
            (
                failures(&all, &law).len(),
                excalidraw_failures(&law).len(),
                [400.0, 96.0, 48.0, 32.0, 24.0].map(rate),
                false_arrows(&law, &[24.0, 48.0, 96.0, 400.0], 12),
            )
        });
        for (v, (fx, ex, r, (_, who))) in values.iter().zip(rows) {
            println!("| {v} | {fx} | {ex} | {} | {who} |", pct(&r));
        }
    }
}

/// **Os dois ângulos da farpa, JUNTOS** — o mínimo segura a linha que volta atrás (uma farpa a 0°),
/// o máximo o T (a barra a 90°), e a mão assimétrica (±10°) aperta os dois.
#[test]
#[ignore = "sonda"]
fn measure_arrow_angles() {
    let mins = [2.0, 4.0, 6.0, 8.0, 10.0, 12.0];
    let maxs = [70.0, 75.0, 80.0, 85.0];
    println!("\n| mínimo \\ máximo | 70° | 75° | 80° | 85° |\n|---|---|---|---|---|");
    let rows = par(&mins, |&lo| {
        maxs.map(|hi| {
            let law = Law {
                arrow: Some(ArrowLaw {
                    barb_min_deg: lo,
                    barb_max_deg: hi,
                    ..ArrowLaw::MEASURED
                }),
                ..Law::MEASURED
            };
            let big = arrow_rate(&law, 400.0, 1.0, 0.5, &DEGS, &[0.15, 0.3], 2);
            let small = arrow_rate(&law, 24.0, 1.0, 0.5, &DEGS, &[0.15, 0.3], 2);
            let (worst, who) = false_arrows(&law, &[24.0, 48.0, 96.0, 400.0], 12);
            format!(
                "{:.0} % / {:.0} % · ≤ {:.0} % {}",
                100.0 * big,
                100.0 * small,
                100.0 * worst,
                who
            )
        })
    });
    for (lo, r) in mins.iter().zip(rows) {
        println!("| {lo}° | {} |", r.join(" | "));
    }
}

/// **O preenchimento do cabo × o resíduo da ponta, JUNTOS** — os dois seguram o triângulo e o
/// rabisco: a grelha que escolhe o par.
#[test]
#[ignore = "sonda"]
fn measure_arrow_joint() {
    let fills = [0.5, 0.6, 0.7, 0.8, 0.825, 0.9, 0.95];
    let rmss = [0.09, 0.11, 0.13];
    println!("\n| preenchimento \\ resíduo | 0,09 | 0,11 | 0,13 |\n|---|---|---|---|");
    let rows = par(&fills, |&f| {
        rmss.map(|r| {
            let law = Law {
                arrow: Some(ArrowLaw {
                    shaft_fill: f,
                    head_rms: r,
                    ..ArrowLaw::MEASURED
                }),
                ..Law::MEASURED
            };
            let big = arrow_rate(&law, 400.0, 1.0, 0.5, &[20.0, 60.0], &[0.08, 0.4], 2);
            let small = arrow_rate(&law, 24.0, 1.0, 0.5, &[20.0, 60.0], &[0.08, 0.4], 2);
            let (worst, who) = false_arrows(&law, &[24.0, 48.0, 96.0, 400.0], 12);
            format!(
                "{:.0} % / {:.0} % · falsas ≤ {:.0} % ({})",
                100.0 * big,
                100.0 * small,
                100.0 * worst,
                if who.is_empty() { "nenhuma" } else { &who }
            )
        })
    });
    for (f, r) in fills.iter().zip(rows) {
        println!("| {f} | {} |", r.join(" | "));
    }
}

#[test]
#[ignore = "sonda"]
fn measure_arrow_zz_star() {
    println!();
    for s in [24.0, 48.0] {
        for seed in 0..10 {
            let star = non_arrows(s, seed)
                .into_iter()
                .find(|x| x.0 == "estrela")
                .unwrap()
                .1;
            for (amp, stab, ra) in [
                (0.5, 0.0, 1.0),
                (0.5, 0.0, 3.0),
                (2.0, 0.0, 3.0),
                (2.0, 0.5, 3.0),
                (0.5, 0.5, 1.0),
            ] {
                let h = held_with(&star, amp, seed, stab, ra);
                let off = Law {
                    arrow: None,
                    ..Law::MEASURED
                };
                if let Some(QuickShape::Arrow(a)) = recognise(&h, 1.0, false) {
                    let kind = match &*a.shaft {
                        QuickShape::Polygon { corners, closed } => {
                            format!("poli{} fechado={closed}", corners.len())
                        }
                        QuickShape::Line { .. } => "linha".into(),
                        QuickShape::Arc(_) => "arco".into(),
                        QuickShape::Biarc(_) => "biarco".into(),
                        _ => "?".into(),
                    };
                    let base = match recognise_with(&off, &h, 1.0, false) {
                        Some(QuickShape::Polygon { corners, closed }) => {
                            format!("poli{} fechado={closed}", corners.len())
                        }
                        Some(o) => format!("{o:?}").chars().take(12).collect(),
                        None => "livre".into(),
                    };
                    println!(
                        "{s} mão {seed} ±{amp} estab {stab} repouso {ra}: cabo {kind} farpas {:?} · sem seta: {base}",
                        a.heads.map(|x| x.map(|v| v.round()))
                    );
                }
            }
        }
    }
}

/// **A busca das testemunhas** (prova de mutação): uma linha por traço candidato — as famílias de
/// setas e de «não é seta» em todas as mãos, e o laço com bico (um cabo FECHADO) — para comparar a
/// saída da lei com a de um mutante (`diff` das duas corridas).
#[test]
#[ignore = "sonda"]
fn measure_arrow_witness() {
    let kind = |q: &Option<QuickShape>| match q {
        None => "livre".to_owned(),
        Some(QuickShape::Arrow(a)) => format!(
            "seta {:?} {}",
            a.heads.map(|h| h.map(|v| v.round())),
            match &*a.shaft {
                QuickShape::Polygon { corners, .. } => format!("poli{}", corners.len()),
                QuickShape::Line { a, b } => format!("linha {:.0}", dist(*a, *b)),
                QuickShape::Arc(_) => "arco".into(),
                QuickShape::Biarc(_) => "biarco".into(),
                _ => "?".into(),
            }
        ),
        Some(o) => format!("{o:?}").chars().take(8).collect(),
    };
    let mut lines = Vec::new();
    let hands = [
        (0.5, 0.0, 1.0),
        (1.0, 0.5, 1.0),
        (2.0, 0.5, 3.0),
        (2.0, 0.0, 3.0),
        (4.0, 0.5, 4.0),
    ];
    for s in [24.0, 48.0, 96.0, 160.0, 400.0] {
        let mut polys: Vec<(String, Vec<P2>)> = Vec::new();
        for seed in 0..4 {
            polys.extend(
                non_arrows(s, seed)
                    .into_iter()
                    .map(|(n, p)| (format!("{n} {seed}"), p)),
            );
        }
        for (name, shaft) in shafts(s) {
            for kind in KINDS {
                for both in [false, true] {
                    for (deg, skew) in [(20.0, 0.0), (20.0, 10.0), (60.0, 10.0)] {
                        for frac in [0.08, 0.25, 0.4] {
                            let p = skewed_arrow(&shaft, frac * s, deg, skew, kind, both);
                            polys.push((
                                format!("seta {name} {kind:?} {both} {deg}±{skew} {frac}"),
                                p,
                            ));
                        }
                    }
                }
            }
        }
        // O laço com bico: um círculo e, no fim, o V — um cabo FECHADO não leva ponta.
        let mut ring = arc_poly(s / 2.0, 0.0, 350.0);
        let (t, q) = (ring[ring.len() - 1], ring[ring.len() - 2]);
        let back = sub(q, t);
        let u = mul(back, 1.0 / dist(q, t));
        ring.extend([
            add(t, mul(rotate(u, 0.5), 0.2 * s)),
            t,
            add(t, mul(rotate(u, -0.5), 0.2 * s)),
        ]);
        polys.push(("laço com bico".into(), ring));
        for (name, p) in &polys {
            for (k, &(amp, stab, ra)) in hands.iter().enumerate() {
                let h = held_with(p, amp, k as u64, stab, ra);
                lines.push(format!(
                    "{s} {name} mão{k}: {}",
                    kind(&recognise(&h, 1.0, false))
                ));
            }
        }
    }
    let out = std::env::var("ARROW_WITNESS").unwrap_or_else(|_| "/dev/null".into());
    std::fs::write(&out, lines.join("\n")).unwrap();
    println!("\n{} traços → {out}", lines.len());
}
