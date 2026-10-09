//! Gates do QuickShape — irmão de `quickshape.rs`.
//!
//! ⭐ O gate principal lê TODAS as fixtures do oráculo (`oraculo_quickshape/fixtures/`): as `in` são
//! a entrada, as `out` a resposta do Xournal++. Tipo igual e geometria a ≤ 1 unidade de página
//! (cantos, pontas, centro) e ≤ 1 % do raio; o «cru» dá `None`. As excepções são a lei do
//! PRODUTO do dono (o Procreate) ou um defeito MEDIDO do oráculo, cada uma com o motivo no nome e um
//! gate de auto-consistência contra a geometria que gerou o traço (`gera_entradas.py`).

use super::*;

const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape/fixtures"
);

/// Unidades de página (a régua do oráculo).
const TOL: f64 = 1.0;

struct Fixture {
    name: String,
    scale: [f64; 2],
    offset: [f64; 2],
    input: Vec<P2>,
    out: Vec<P2>,
    recognised: bool,
}

impl Fixture {
    /// Um ponto da tela virtual do gerador, em unidades de página (a calibração do cabeçalho).
    fn page(&self, p: P2) -> P2 {
        [
            p[0] * self.scale[0] + self.offset[0],
            p[1] * self.scale[1] + self.offset[1],
        ]
    }
    fn ours(&self) -> Option<QuickShape> {
        recognise(&self.input, self.scale[0], false)
    }
}

fn nums(s: &str) -> Vec<f64> {
    s.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .filter_map(|t| t.parse().ok())
        .collect()
}

fn load() -> Vec<Fixture> {
    let mut v: Vec<Fixture> = std::fs::read_dir(FIXTURES)
        .expect("a pasta das fixtures do oráculo")
        .filter_map(|e| {
            let path = e.ok()?.path();
            (path.extension()? == "txt").then_some(path)
        })
        .map(|path| {
            let text = std::fs::read_to_string(&path).expect("fixture legível");
            let (mut f, mut cal) = (
                Fixture {
                    name: path.file_stem().unwrap().to_string_lossy().into_owned(),
                    scale: [0.0; 2],
                    offset: [0.0; 2],
                    input: Vec::new(),
                    out: Vec::new(),
                    recognised: false,
                },
                Vec::new(),
            );
            for l in text.lines() {
                if let Some(c) = l.strip_prefix("# calibracao:") {
                    cal = nums(c);
                } else if l.starts_with("# reconhecido: sim") {
                    f.recognised = true;
                } else if let Some(p) = l.strip_prefix("in ") {
                    let n = nums(p);
                    f.input.push([n[0], n[1]]);
                } else if let Some(p) = l.strip_prefix("out ") {
                    let n = nums(p);
                    f.out.push([n[0], n[1]]);
                }
            }
            assert_eq!(cal.len(), 4, "{}: a calibração do cabeçalho", f.name);
            (f.scale, f.offset) = ([cal[0], cal[1]], [cal[2], cal[3]]);
            f
        })
        .collect();
    v.sort_by(|a, b| a.name.cmp(&b.name));
    v
}

/// A pior distância de um ponto de `want` ao ponto mais perto de `got` (e as contagens iguais).
fn worst(got: &[P2], want: &[P2]) -> f64 {
    assert_eq!(
        got.len(),
        want.len(),
        "contagem de cantos {got:?} vs {want:?}"
    );
    want.iter()
        .map(|&w| got.iter().map(|&g| dist(g, w)).fold(f64::MAX, f64::min))
        .fold(0.0, f64::max)
}

fn circle_of(out: &[P2]) -> (P2, f64) {
    let pts = if out.first() == out.last() {
        &out[..out.len() - 1]
    } else {
        out
    };
    let c = centroid(pts);
    (
        c,
        pts.iter().map(|&p| dist(p, c)).sum::<f64>() / pts.len() as f64,
    )
}

/// O resíduo rms do traço contra um círculo — quem o desenha MELHOR.
fn circle_rms(input: &[P2], c: P2, r: f64) -> f64 {
    (input.iter().map(|&p| (dist(p, c) - r).powi(2)).sum::<f64>() / input.len() as f64).sqrt()
}

/// O que a nossa lei responde onde o oráculo NÃO manda, e porquê. A geometria esperada vem do
/// gerador (px da tela virtual), passada à página pela calibração da própria fixture.
enum Own {
    /// O Procreate faz polígonos de N lados onde o Xournal++ faz um círculo; e quadriláteros e
    /// rectângulos «de mão» (arredondados, começados a meio, que passam do início) onde ele é cru.
    Closed(Vec<P2>),
    /// A polilinha aberta do Procreate (o Xournal++ deixa-a crua).
    Open(Vec<P2>),
    /// A elipse do Procreate: centro, semi-eixos, rotação (graus).
    Ellipse(P2, f64, f64, f64),
    /// O centróide do oráculo FOGE do traço (do buraco de um círculo aberto, para a sobreposição
    /// de um que passa): o nosso círculo tem de desenhar o traço MELHOR que o dele.
    CircleFitsBetter,
}

fn rect_400x250() -> Vec<P2> {
    vec![
        [600.0, 375.0],
        [1000.0, 375.0],
        [1000.0, 625.0],
        [600.0, 625.0],
    ]
}

fn regular(n: usize) -> Vec<P2> {
    (0..n)
        .map(|k| {
            let a = -FRAC_PI_2 + TAU * k as f64 / n as f64;
            [800.0 + 200.0 * a.cos(), 500.0 + 200.0 * a.sin()]
        })
        .collect()
}

fn own(name: &str) -> Option<Own> {
    let skew = |deg: f64| {
        let dx = 250.0 * deg.to_radians().tan() / 2.0;
        Own::Closed(vec![
            [600.0 - dx, 375.0],
            [1000.0 - dx, 375.0],
            [1000.0 + dx, 625.0],
            [600.0 + dx, 625.0],
        ])
    };
    Some(match name {
        "poligono_5_lados" => Own::Closed(regular(5)),
        "poligono_6_lados" => Own::Closed(regular(6)),
        "poligono_8_lados" => Own::Closed(regular(8)),
        // O único rectângulo PERFEITO do corpus — o oráculo recusa-o e aceita o mesmo com tremor ±3.
        "rect_alinhado_400x250"
        | "rect_cantos_r20"
        | "rect_cantos_r40"
        | "rect_comeca_no_meio"
        | "rect_passa_do_inicio" => Own::Closed(rect_400x250()),
        "paralelogramo_10" => skew(10.0),
        "paralelogramo_15" => skew(15.0),
        "paralelogramo_20" => skew(20.0),
        "paralelogramo_30" => skew(30.0),
        "trapezio" => Own::Closed(vec![
            [600.0, 625.0],
            [1000.0, 625.0],
            [900.0, 375.0],
            [700.0, 375.0],
        ]),
        "forma_L" => Own::Open(vec![[600.0, 300.0], [600.0, 700.0], [1000.0, 700.0]]),
        "forma_V" => Own::Open(vec![[600.0, 300.0], [800.0, 700.0], [1000.0, 300.0]]),
        "zigzag_4" => Own::Open(vec![
            [500.0, 400.0],
            [650.0, 650.0],
            [800.0, 400.0],
            [950.0, 650.0],
            [1100.0, 400.0],
        ]),
        // O rectângulo aberto 90 px é uma polilinha de 4 lados: a ponta é onde a mão parou.
        "rect_aberto_90" => {
            let mut v = rect_400x250();
            v.push([600.0, 465.0]);
            Own::Open(v)
        }
        "elipse_2para1" => Own::Ellipse([800.0, 500.0], 300.0, 150.0, 0.0),
        "elipse_2para1_rodada_30" => Own::Ellipse([800.0, 500.0], 300.0, 150.0, 30.0),
        "elipse_3para2" => Own::Ellipse([800.0, 500.0], 300.0, 200.0, 0.0),
        "circulo_aberto_40" | "circulo_passa_10" => Own::CircleFitsBetter,
        _ => return None,
    })
}

#[test]
fn the_oracle_fixtures_all_agree_with_the_law_or_with_the_owners_product() {
    let all = load();
    // Controlos: a pasta inteira foi lida, e o oráculo reconheceu o bastante para medir algo.
    assert!(all.len() >= 65, "só {} fixtures lidas", all.len());
    let rec = all.iter().filter(|f| f.recognised).count();
    assert!(
        rec >= 26,
        "controlo positivo: só {rec} fixtures reconhecidas pelo oráculo"
    );
    let mut failures = Vec::new();
    for f in &all {
        let got = f.ours();
        let verdict = match (own(&f.name), &got) {
            (
                Some(Own::Closed(want)),
                Some(QuickShape::Polygon {
                    corners,
                    closed: true,
                }),
            ) => {
                let want: Vec<P2> = want.iter().map(|&p| f.page(p)).collect();
                (worst(corners, &want) <= TOL)
                    .then_some(())
                    .ok_or(format!("cantos {corners:?} vs {want:?}"))
            }
            (
                Some(Own::Open(mut want)),
                Some(QuickShape::Polygon {
                    corners,
                    closed: false,
                }),
            ) => {
                if f.name == "rect_aberto_90" {
                    // A ponta solta é a última amostra da mão (já em página).
                    want.pop();
                }
                let mut want: Vec<P2> = want.iter().map(|&p| f.page(p)).collect();
                if f.name == "rect_aberto_90" {
                    want.push(f.input[f.input.len() - 1]);
                }
                (worst(corners, &want) <= TOL)
                    .then_some(())
                    .ok_or(format!("cantos {corners:?} vs {want:?}"))
            }
            (
                Some(Own::Ellipse(c, rx, ry, deg)),
                Some(QuickShape::Ellipse {
                    centre,
                    rx: a,
                    ry: b,
                    rot,
                }),
            ) => {
                let (want_a, want_b) = (rx * f.scale[0], ry * f.scale[0]);
                let drot = wrap_pi(2.0 * (rot - deg.to_radians())).abs() / 2.0;
                (dist(*centre, f.page(c)) <= TOL
                    && (a - want_a).abs() <= 0.01 * want_a
                    && (b - want_b).abs() <= 0.01 * want_b
                    && drot <= 1f64.to_radians())
                .then_some(())
                .ok_or(format!(
                    "elipse {centre:?} {a} {b} {rot} vs {:?} {want_a} {want_b} {deg}°",
                    f.page(c)
                ))
            }
            (Some(Own::CircleFitsBetter), Some(QuickShape::Ellipse { centre, rx, ry, .. }))
                if rx == ry =>
            {
                let (oc, or) = circle_of(&f.out);
                let (ours, theirs) = (
                    circle_rms(&f.input, *centre, *rx),
                    circle_rms(&f.input, oc, or),
                );
                ((rx - or).abs() <= 0.01 * or && ours < theirs)
                    .then_some(())
                    .ok_or(format!(
                        "o nosso desenha o traço a {ours:.3}, o do oráculo a {theirs:.3}"
                    ))
            }
            (Some(_), other) => Err(format!("a lei própria pedia outra forma, saiu {other:?}")),
            (None, ours) if !f.recognised => ours
                .is_none()
                .then_some(())
                .ok_or(format!("o oráculo é cru, saiu {ours:?}")),
            (None, Some(QuickShape::Line { a, b })) if f.out.len() == 2 => {
                let e = (dist(*a, f.out[0]).max(dist(*b, f.out[1])))
                    .min(dist(*a, f.out[1]).max(dist(*b, f.out[0])));
                (e <= TOL).then_some(()).ok_or(format!("pontas a {e:.2}"))
            }
            (
                None,
                Some(QuickShape::Polygon {
                    corners,
                    closed: true,
                }),
            ) if f.out.len() <= 6 => {
                let want = &f.out[..f.out.len() - 1];
                (worst(corners, want) <= TOL)
                    .then_some(())
                    .ok_or(format!("cantos {corners:?} vs {want:?}"))
            }
            (None, Some(QuickShape::Ellipse { centre, rx, ry, .. }))
                if rx == ry && f.out.len() > 6 =>
            {
                let (oc, or) = circle_of(&f.out);
                (dist(*centre, oc) <= TOL && (rx - or).abs() <= 0.01 * or)
                    .then_some(())
                    .ok_or(format!("círculo {centre:?} r {rx} vs {oc:?} r {or}"))
            }
            (None, ours) => Err(format!(
                "o oráculo reconheceu {} pontos, saiu {ours:?}",
                f.out.len()
            )),
        };
        if let Err(e) = verdict {
            failures.push(format!("{}: {e}", f.name));
        }
    }
    assert!(
        failures.is_empty(),
        "{} de {} fixtures divergem:\n{}",
        failures.len(),
        all.len(),
        failures.join("\n")
    );
}

#[test]
fn every_own_law_exception_names_a_fixture_that_exists() {
    let names: Vec<String> = load().into_iter().map(|f| f.name).collect();
    let used = names.iter().filter(|n| own(n).is_some()).count();
    // 3 polígonos + 5 rectângulos de mão + 5 quadriláteros + 4 polilinhas + 3 elipses + 2 círculos.
    assert_eq!(
        used, 22,
        "uma excepção ficou órfã (a fixture mudou de nome?)"
    );
}

// ── o `regular` (Shift / o 2.º dedo) ────────────────────────────────────────────────────────────

/// Um traço de mão em mundo 1:1 (px), amostrado a 4 px, com um tremor determinístico.
pub(super) fn hand(poly: &[P2], amp: f64) -> Vec<P2> {
    let mut out = vec![poly[0]];
    for w in poly.windows(2) {
        let n = (dist(w[0], w[1]) / 4.0).ceil() as usize;
        out.extend((1..=n).map(|i| add(w[0], mul(sub(w[1], w[0]), i as f64 / n as f64))));
    }
    out.iter()
        .enumerate()
        .map(|(i, &p)| {
            add(
                p,
                [amp * (i as f64 * 1.9).sin(), amp * (i as f64 * 2.7).cos()],
            )
        })
        .collect()
}

pub(super) fn sides(c: &[P2]) -> Vec<f64> {
    (0..c.len())
        .map(|i| dist(c[i], c[(i + 1) % c.len()]))
        .collect()
}

#[test]
fn regular_turns_a_rectangle_into_a_square_and_a_triangle_into_an_equilateral() {
    let rect = hand(
        &[
            [0.0, 0.0],
            [400.0, 10.0],
            [395.0, 260.0],
            [-5.0, 250.0],
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
    let s = sides(&corners);
    assert_eq!(s.len(), 4);
    assert!(
        s.iter().all(|x| (x - s[0]).abs() < 1e-6),
        "lados iguais: {s:?}"
    );
    let tri = hand(
        &[[0.0, 0.0], [400.0, 30.0], [150.0, 300.0], [0.0, 0.0]],
        1.0,
    );
    let Some(QuickShape::Polygon {
        corners,
        closed: true,
    }) = recognise(&tri, 1.0, true)
    else {
        panic!("tri")
    };
    let s = sides(&corners);
    assert!(
        s.len() == 3 && s.iter().all(|x| (x - s[0]).abs() < 1e-6),
        "equilátero: {s:?}"
    );
    // Controlo: sem `regular` o mesmo triângulo fica COMO DESENHADO.
    let Some(QuickShape::Polygon { corners, .. }) = recognise(&tri, 1.0, false) else {
        panic!("tri")
    };
    let s = sides(&corners);
    assert!(
        (s[0] - s[2]).abs() > 50.0,
        "o triângulo cru não se regulariza: {s:?}"
    );
}

#[test]
fn regular_rounds_an_ellipse_and_snaps_a_line_to_fifteen_degrees() {
    let e: Vec<P2> = (0..=300)
        .map(|i| {
            let a = TAU * i as f64 / 300.0;
            [300.0 * a.cos(), 150.0 * a.sin()]
        })
        .collect();
    assert!(
        matches!(recognise(&e, 1.0, false), Some(QuickShape::Ellipse { rx, ry, .. }) if (rx / ry - 2.0).abs() < 0.02)
    );
    assert!(
        matches!(recognise(&e, 1.0, true), Some(QuickShape::Ellipse { rx, ry, .. }) if rx == ry)
    );
    let l = hand(
        &[
            [0.0, 0.0],
            [
                400.0 * 22f64.to_radians().cos(),
                400.0 * 22f64.to_radians().sin(),
            ],
        ],
        0.5,
    );
    let Some(QuickShape::Line { a, b }) = recognise(&l, 1.0, true) else {
        panic!("linha")
    };
    assert!(
        (angle(sub(b, a)).to_degrees() - 15.0).abs() < 1e-9,
        "22° vira 15°"
    );
    let Some(QuickShape::Line { a, b }) = recognise(&l, 1.0, false) else {
        panic!("linha")
    };
    assert!(
        (angle(sub(b, a)).to_degrees() - 22.0).abs() < 0.5,
        "sem regular fica a 22°"
    );
}

#[test]
fn the_minimum_size_is_in_screen_pixels() {
    let l = hand(&[[0.0, 0.0], [80.0, 0.0]], 0.0);
    assert!(recognise(&l, 1.0, false).is_none(), "80 px < 94 px");
    // O MESMO traço em mundo, visto com zoom 2× (meio mundo por px): 160 px na tela.
    assert!(
        recognise(&l, 0.5, false).is_some(),
        "160 px de tela é uma linha"
    );
}

#[test]
fn transformed_scales_and_turns_about_the_pivot_and_to_path_keeps_the_nodes() {
    let sq = QuickShape::Polygon {
        corners: vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
        closed: true,
    };
    assert_eq!(sq.pivot(), [1.0, 1.0]);
    let QuickShape::Polygon { corners, .. } = sq.transformed([1.0, 1.0], 2.0, FRAC_PI_2) else {
        panic!()
    };
    assert!(dist(corners[0], [3.0, -1.0]) < 1e-12, "{corners:?}");
    let p = sq.to_path();
    assert!(p.closed && p.verts.len() == 4);
    let e = QuickShape::Ellipse {
        centre: [0.0, 0.0],
        rx: 2.0,
        ry: 1.0,
        rot: FRAC_PI_2,
    };
    let p = e.to_path();
    assert!(p.closed && p.verts.len() == 4);
    assert!(
        dist(p.verts[0].anchor, [0.0, 2.0]) < 1e-12,
        "o semi-eixo rx rodado 90°: {:?}",
        p.verts[0].anchor
    );
    let l = QuickShape::Line {
        a: [1.0, 1.0],
        b: [3.0, 1.0],
    };
    assert_eq!(l.pivot(), [1.0, 1.0]);
    assert!(!l.to_path().closed);
}

// ── as cláusulas que só as fixtures não guardavam (pedidas pela prova de mutação) ───────────────

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
