//! Gates e sondas do ARCO do QuickShape (F7c) — irmão de `quickshape_tests.rs` (tecto de 700 LOC).
//! Sondas: `cargo test --release -p ph2d-vec-edit --lib measure_arc -- --ignored --nocapture
//! --test-threads=1`.

use super::*;
use crate::quickshape_arc::taubin;
use crate::test_hand::{Lcg, hand, stabilise};
use ph2d_vec_scene::VertexKind;
use std::f64::consts::PI;

fn circle_arc(centre: P2, r: f64, start: f64, sweep: f64) -> ArcShape {
    ArcShape {
        centre,
        rx: r,
        ry: r,
        rot: 0.0,
        start,
        sweep,
    }
}

/// Um arco de raio `r` à volta da origem, de `a0` a varrer `sweep` (graus, com sinal), a 1 px.
pub(super) fn arc_poly(r: f64, a0: f64, sweep: f64) -> Vec<P2> {
    let n = ((sweep.to_radians().abs() * r).ceil() as usize).max(8);
    (0..=n)
        .map(|i| mul(unit((a0 + sweep * i as f64 / n as f64).to_radians()), r))
        .collect()
}

/// Uma mão: o tremor `amp` e o estabilizador por omissão do lápis.
pub(super) fn stroke(poly: &[P2], amp: f64, seed: u64) -> Vec<P2> {
    stabilise(&hand(poly, 2.0, amp, seed), 0.5)
}

/// O S de dois meios círculos, o S de seno e um rabisco (o do `gera_entradas.py`): não são arcos.
pub(super) fn not_arcs(s: f64, seed: u64) -> Vec<(&'static str, Vec<P2>)> {
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
pub(super) fn is_the_arc(q: &Option<QuickShape>, sweep: f64) -> bool {
    matches!(q, Some(QuickShape::Arc(ArcShape { sweep: s, .. }))
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
    assert!(
        dist(c, [50.0, -70.0]) < 1e-6 && (r - 300.0).abs() < 1e-6,
        "{c:?} {r}"
    );
}

#[test]
fn a_hand_drawn_arc_becomes_a_perfect_arc_in_its_own_sense() {
    for sweep in [45.0, 90.0, 180.0, 270.0, -60.0, -200.0] {
        for seed in 0..10 {
            let pts = stroke(&arc_poly(150.0, 17.0 * seed as f64, sweep), 1.0, seed);
            let q = recognise(&pts, 1.0, false);
            assert!(is_the_arc(&q, sweep), "{sweep}° (mão {seed}): {q:?}");
            let Some(QuickShape::Arc(ArcShape { centre, rx, ry, .. })) = q else {
                unreachable!()
            };
            assert!(
                rx == ry && dist(centre, [0.0, 0.0]) < 6.0 && (rx - 150.0).abs() < 6.0,
                "{sweep}°: um CÍRCULO, centro {centre:?} raios {rx} {ry}"
            );
            assert!(
                dist(q.as_ref().unwrap().pivot(), pts[0]) < 3.0,
                "o pivô é o início"
            );
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
    let Some(QuickShape::Arc(ArcShape {
        centre,
        rx: radius,
        start,
        sweep,
        ..
    })) = recognise(&pts, 1.0, true)
    else {
        panic!()
    };
    let step = REGULAR_STEP_DEG.to_radians();
    assert!(
        (sweep - FRAC_PI_2).abs() < 1e-9,
        "97° → 90°: {}",
        sweep.to_degrees()
    );
    let p0 = add(centre, mul(unit(start), radius));
    let p1 = add(centre, mul(unit(start + sweep), radius));
    let chord = angle(sub(p1, p0));
    assert!(
        (chord - snap(chord, step)).abs() < 1e-9,
        "a corda a 15°: {}",
        chord.to_degrees()
    );
    assert!(dist(p0, pts[0]) < 1e-9, "o início fica onde a mão o pôs");
}

#[test]
fn the_arc_path_is_open_smooth_inside_cornered_at_the_ends_and_on_the_circle() {
    // 92° cabe num troço (a folga de 5 %): o meio arco de uma mão tem 181° e três nós.
    for (sweep, n) in [
        (PI / 3.0, 2),
        (0.51 * PI, 2),
        (-PI, 3),
        (1.5 * PI, 4),
        (1.9 * PI, 5),
    ] {
        let p = circle_arc([10.0, 20.0], 100.0, 0.3, sweep).to_path();
        assert!(
            !p.closed && p.verts.len() == n,
            "{sweep}: {} nós",
            p.verts.len()
        );
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
            assert!(
                (dist(mid, [10.0, 20.0]) - 100.0).abs() < 0.03,
                "{sweep}: {mid:?}"
            );
        }
    }
    let a = QuickShape::Arc(circle_arc([0.0, 0.0], 2.0, 0.0, PI));
    assert_eq!(a.pivot(), [2.0, 0.0]);
    let QuickShape::Arc(b) = a.transformed([2.0, 0.0], 2.0, FRAC_PI_2) else {
        panic!()
    };
    assert!(dist(b.centre, [2.0, -4.0]) < 1e-12 && b.rx == 4.0 && b.sweep == PI);
    assert!((b.rot - FRAC_PI_2).abs() < 1e-12, "a rotação roda o arco");
    assert!(dist(b.pivot(), [2.0, 0.0]) < 1e-12, "o início fica");
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
    assert!(
        !p.closed && p.verts.len() == 3,
        "o caminho vivo É o meio arco: {}",
        p.verts.len()
    );
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
    assert!(
        dist(p.verts[2].anchor, turned) < 3.0,
        "{:?} vs {turned:?}",
        p.verts[2].anchor
    );
    assert!(pencil.on_release(&mut scene));
    let p = scene.path(id).unwrap();
    assert!(!p.closed && p.verts.len() == 3, "soltar comete o ARCO");
    assert_eq!(p.verts[1].kind, VertexKind::Smooth);
    assert_eq!(p.verts[0].kind, VertexKind::Corner);
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
        assert!(
            !matches!(q, Some(QuickShape::Arc { .. })),
            "S raso (mão {seed}): {q:?}"
        );
        for (name, poly) in not_arcs(160.0, seed).into_iter().take(2) {
            let q = recognise_with(&no_bend, &stroke(&poly, 1.0, seed), 1.0, false);
            assert!(
                !matches!(q, Some(QuickShape::Arc { .. })),
                "{name} (mão {seed}): {q:?}"
            );
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
        poly.push(add(
            q,
            mul(unit((5.0 + 30.0 * f64::from(j)).to_radians()), 150.0),
        ));
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
pub(super) fn kasa(p: &[P2]) -> (P2, f64) {
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

// ── o arco ASSIMÉTRICO (o smoke do dono, 10/10: «arcos assimétricos como na foto não funcionam») ─

/// O traço da captura do dono (px da tela, y para baixo): abre largo e fecha apertado.
pub(super) fn owners_hook() -> Vec<P2> {
    crate::quickshape::OWNERS_HOOK_PX.to_vec()
}

/// Os arcos assimétricos: o gancho do dono (½×, 1×, 2×, e espelhado), arcos de ELIPSE começados
/// fora do eixo, e espirais (a curvatura triplica de ponta a ponta).
pub(super) fn asymmetric() -> Vec<(String, Vec<P2>)> {
    let mut v = Vec::new();
    for k in [0.5, 1.0, 2.0] {
        let h: Vec<P2> = owners_hook().iter().map(|&p| mul(p, k)).collect();
        v.push((format!("gancho do dono {k}×"), h.clone()));
        v.push((
            format!("gancho espelhado {k}×"),
            h.iter().map(|p| [-p[0], p[1]]).collect(),
        ));
    }
    for ratio in [1.5, 2.0, 3.0] {
        for (a0, sweep) in [(200.0_f64, 230.0_f64), (120.0, 150.0)] {
            let poly: Vec<P2> = (0..=200)
                .map(|i| {
                    let a = (a0 + sweep * f64::from(i) / 200.0).to_radians();
                    rotate([160.0 * a.cos(), 160.0 / ratio * a.sin()], 0.4)
                })
                .collect();
            v.push((format!("elipse {ratio}:1 {a0}°+{sweep}°"), poly));
        }
    }
    for turn in [120.0_f64, 200.0, 270.0] {
        // Curvatura de k a 3k, comprimento 500 px.
        let (n, len) = (400, 500.0);
        let k0 = turn.to_radians() / (2.0 * len);
        let (mut p, mut a) = ([0.0, 0.0], 0.0_f64);
        let mut poly = vec![p];
        for i in 0..n {
            let s = len * f64::from(i) / f64::from(n);
            a += k0 * (1.0 + 2.0 * s / len) * len / f64::from(n);
            p = add(p, mul(unit(a), len / f64::from(n)));
            poly.push(p);
        }
        v.push((format!("espiral {turn}°"), poly));
    }
    v
}

/// O maior desvio (px) da curva DESENHADA ao arco reconhecido.
pub(super) fn deviation(q: &QuickShape, poly: &[P2]) -> f64 {
    let arcs: Vec<ArcShape> = match q {
        QuickShape::Arc(a) => vec![*a],
        QuickShape::Biarc(b) => b.to_vec(),
        _ => return f64::INFINITY,
    };
    let on: Vec<P2> = arcs
        .iter()
        .flat_map(|a| (0..=400).map(|i| a.at(a.start + a.sweep * f64::from(i) / 400.0)))
        .collect();
    poly.iter()
        .map(|&p| on.iter().map(|&q| dist(p, q)).fold(f64::MAX, f64::min))
        .fold(0.0, f64::max)
}

// ── os gates do arco ASSIMÉTRICO ────────────────────────────────────────────────────────────────

/// O gancho da captura do dono (abre largo, fecha apertado) vira um BIARCO que o segue — a 4 % do
/// tamanho no pior ponto (`measure_asymmetric_arcs`: 11,3 px num gancho de 345 px); espelhado e a
/// meio tamanho também. Com `Shift`, um arco de CÍRCULO.
#[test]
fn the_owners_asymmetric_hook_becomes_a_biarc_that_follows_it() {
    for k in [0.5, 1.0] {
        for mirror in [1.0, -1.0] {
            let poly: Vec<P2> = owners_hook()
                .iter()
                .map(|&p| [mirror * p[0] * k, p[1] * k])
                .collect();
            for seed in 0..5 {
                let pts = stroke(&poly, 1.0, seed);
                let q = recognise(&pts, 1.0, false);
                let Some(q @ QuickShape::Biarc(b)) = q else {
                    panic!("{k}× espelho {mirror} (mão {seed}): {q:?}")
                };
                assert!(deviation(&q, &poly) < 0.04 * 345.0 * k, "{k}×: {q:?}");
                assert!(b[0].rx > 3.0 * b[1].rx, "abre largo, fecha apertado: {b:?}");
                let r = recognise(&pts, 1.0, true);
                assert!(
                    matches!(r, Some(QuickShape::Arc(a)) if a.rx == a.ry),
                    "Shift: {r:?}"
                );
            }
        }
    }
}

/// Um arco de ELIPSE (2:1, começado fora do eixo) sai arco de elipse, a ≤ 3 px do desenho; uma
/// espiral (a curvatura triplica) de 200° e 270° sai um arco a ≤ 5 px. A de 120° o CÍRCULO ainda a
/// desenha dentro da régua — e o círculo fica (a lei: o assimétrico só quando ele não serve).
#[test]
fn an_elliptic_arc_and_a_spiral_are_followed() {
    for (name, poly) in asymmetric() {
        let elliptic = name.starts_with("elipse 2:1");
        if !(elliptic || name.starts_with("espiral")) {
            continue;
        }
        for seed in 0..5 {
            let q = recognise(&stroke(&poly, 1.0, seed), 1.0, false);
            let Some(q) = q else {
                panic!("{name} (mão {seed}): livre")
            };
            if elliptic {
                assert!(
                    matches!(q, QuickShape::Arc(a) if a.rx > 1.5 * a.ry || a.ry > 1.5 * a.rx),
                    "{name}: {q:?}"
                );
            }
            if name == "espiral 120°" {
                assert!(
                    matches!(q, QuickShape::Arc(a) if a.rx == a.ry),
                    "{name}: {q:?}"
                );
                continue;
            }
            let d = deviation(&q, &poly);
            assert!(d < if elliptic { 3.0 } else { 5.0 }, "{name}: {d:.1} px");
        }
    }
}

/// O caminho de um biarco é UM traço: os dois arcos em seguida, o nó da junção `Smooth` com as alças
/// colineares (são tangentes), `Corner` nas pontas; o pivô é o início e a semelhança mantém-no.
#[test]
fn a_biarc_path_is_one_tangent_stroke() {
    let first = circle_arc([0.0, 0.0], 100.0, -FRAC_PI_2, PI);
    let join = first.at(first.start + first.sweep);
    // O 2.º arco, tangente: o centro sobre o raio da junção, raio 30.
    let second = circle_arc(mul(join, 0.7), 30.0, FRAC_PI_2, FRAC_PI_2);
    assert!(dist(second.at(second.start), join) < 1e-9);
    let b = QuickShape::Biarc([first, second]);
    let p = b.to_path();
    assert!(
        !p.closed && p.verts.len() == 3 + 2 - 1,
        "{} nós",
        p.verts.len()
    );
    let j = &p.verts[2];
    assert_eq!(j.kind, VertexKind::Smooth);
    let (din, dout) = (sub(j.anchor, j.in_handle), sub(j.out_handle, j.anchor));
    assert!(
        (din[0] * dout[1] - din[1] * dout[0]).abs() < 1e-9,
        "alças colineares"
    );
    assert_eq!(p.verts[0].kind, VertexKind::Corner);
    assert_eq!(p.verts[3].kind, VertexKind::Corner);
    let pivot = b.pivot();
    assert!(dist(pivot, [0.0, -100.0]) < 1e-9);
    let QuickShape::Biarc(t) = b.transformed(pivot, 2.0, 0.3) else {
        panic!()
    };
    assert!(
        dist(t[0].pivot(), pivot) < 1e-9
            && dist(t[1].pivot(), t[0].at(t[0].start + t[0].sweep)) < 1e-9
    );
}
