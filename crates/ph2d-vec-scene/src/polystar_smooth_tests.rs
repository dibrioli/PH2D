//! F13 — o *Smooth* e o *Cutout* pelo relatório do dono (12/10: «o Cutout está estranho»; «o Smooth
//! parece invertido: no mínimo tem o maior efeito»). Doc 15 §3b.

use super::*;

/// Os pontos do contorno (24 por segmento), todos os contornos.
fn samples(p: &VecPath) -> Vec<[f64; 2]> {
    let n = p.verts.len();
    let mut out = Vec::new();
    for i in 0..n {
        let (a, b) = (&p.verts[i], &p.verts[(i + 1) % n]);
        for k in 0..24 {
            let t = k as f64 / 24.0;
            let u = 1.0 - t;
            let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
            let q = [a.anchor, a.out_handle, b.in_handle, b.anchor];
            out.push([0, 1].map(|j| w.iter().zip(&q).map(|(k, q)| k * q[j]).sum::<f64>()));
        }
    }
    out
}

/// A distância de Hausdorff de ponto a SEGMENTO (as poligonais amostradas): ponto a ponto, uma
/// aresta recta de alças coladas lê o espaçamento das amostras (0,023 sobre o MESMO contorno).
fn hausdorff(a: &VecPath, b: &VecPath) -> f64 {
    let (a, b) = (samples(a), samples(b));
    let to_seg = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        let (d, w) = ([r[0] - q[0], r[1] - q[1]], [p[0] - q[0], p[1] - q[1]]);
        let l2 = d[0] * d[0] + d[1] * d[1];
        let t = if l2 > 0.0 {
            ((w[0] * d[0] + w[1] * d[1]) / l2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (w[0] - t * d[0]).hypot(w[1] - t * d[1])
    };
    let one = |x: &[[f64; 2]], y: &[[f64; 2]]| {
        x.iter()
            .map(|p| {
                (0..y.len())
                    .map(|i| to_seg(*p, y[i], y[(i + 1) % y.len()]))
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0, f64::max)
    };
    one(&a, &b).max(one(&b, &a))
}

fn poly(n: u32, smooth: f64, r: f64, style: CornerStyle) -> PolyStar {
    PolyStar {
        n,
        smooth,
        radii: [r, r],
        style,
        ..PolyStar::default()
    }
}

fn star(smooth: f64, r: f64) -> PolyStar {
    PolyStar {
        inner: Some(0.45),
        ..poly(5, smooth, r, CornerStyle::Round)
    }
}

/// ⭐ **O liso `1` é o REDONDO**: o polígono regular vira o círculo dele (o arco cúbico erra < 1e-3).
#[test]
fn smooth_one_is_the_circle_of_the_regular_polygon() {
    for n in [4, 5, 6, 9] {
        let p = polystar([0.0; 2], 1.0, 1.0, &poly(n, 1.0, 0.0, CornerStyle::Round));
        let worst = samples(&p)
            .iter()
            .map(|q| (q[0].hypot(q[1]) - 1.0).abs())
            .fold(0.0, f64::max);
        assert!(worst < 1e-3, "{n} lados: {worst}");
    }
}

/// ⭐ **Mais liso, mais longe do pontudo — nunca o contrário**: a distância à forma de liso `0` só
/// cresce de 0 a 1 (o polígono e a estrela, com e sem raio de canto). Antes, a faixa ia a 3× o
/// redondo e o fim dela voltava a fazer bicos.
#[test]
fn more_smooth_never_turns_back() {
    for (name, mk) in [
        (
            "polígono",
            (|s, r| poly(5, s, r, CornerStyle::Round)) as fn(f64, f64) -> PolyStar,
        ),
        ("estrela", star),
    ] {
        for r in [0.0, 0.25] {
            let at = |s: f64| polystar([0.0; 2], 1.0, 1.0, &mk(s, r));
            let start = at(0.0);
            let d: Vec<f64> = (0..=10)
                .map(|k| hausdorff(&start, &at(k as f64 / 10.0)))
                .collect();
            assert!(
                d.windows(2).all(|w| w[1] >= w[0] - 1e-3),
                "{name} r {r}: {d:?}"
            );
        }
    }
}

/// ⭐ **O raio do canto ENTRA no liso** (F13): com raio, o 1.º passo do liso já é o arredondado que
/// o raio dava (o quinhão dele da meia aresta), e não o polígono cru. ⚠️ A ponta passa SEMPRE pelo
/// vértice (a lei do *Rounded* do Inkscape, a de antes, que o dono quer de volta): o salto da ponta
/// no 1.º passo fica — medido `0,059` no pentágono de raio `0,25`, o mesmo de apagar o raio.
#[test]
fn the_corner_radius_carries_into_the_smooth() {
    for (name, mk) in [
        (
            "polígono",
            (|s, r| poly(5, s, r, CornerStyle::Round)) as fn(f64, f64) -> PolyStar,
        ),
        ("estrela", |s, r| star(s, r)),
    ] {
        let at = |s, r| polystar([0.0; 2], 1.0, 1.0, &mk(s, r));
        let h = hausdorff(&at(0.001, 0.25), &at(0.001, 0.0));
        assert!(h > 0.01, "{name}: o raio não entrou no liso ({h})");
    }
}

/// ⭐ **O liso não deixa calombos** (F13, o dono: «Smooth … ficou horrível»): com raio de canto e
/// com ou sem envergadura, todo vértice do contorno cozido é G1 — as alças de entrada e de saída
/// na mesma recta. O F13 anterior deixava a quina do raio em cada canto.
#[test]
fn smooth_leaves_no_corner_bump() {
    for c in [0.0, 0.4, -0.4] {
        for s in [0.05, 0.3, 0.5, 0.75, 1.0] {
            let p = polystar(
                [0.0; 2],
                1.0,
                1.0,
                &PolyStar {
                    curve: c,
                    ..poly(5, s, 0.2, CornerStyle::Round)
                },
            );
            for v in &p.verts {
                let (a, b) = (
                    [v.anchor[0] - v.in_handle[0], v.anchor[1] - v.in_handle[1]],
                    [v.out_handle[0] - v.anchor[0], v.out_handle[1] - v.anchor[1]],
                );
                let (la, lb) = (a[0].hypot(a[1]), b[0].hypot(b[1]));
                if la < 1e-9 || lb < 1e-9 {
                    continue;
                }
                let cross = (a[0] * b[1] - a[1] * b[0]) / (la * lb);
                assert!(
                    cross.abs() < 1e-6,
                    "liso {s}, envergadura {c}: quina {cross} em {:?}",
                    v.anchor
                );
            }
        }
    }
}

/// ⭐ **O ENTALHE tem as paredes em esquadria com os lados e o fundo no centro do arredondado** —
/// nunca passa para lá dele, então não cruza o lado oposto (o paralelogramo antigo, num triângulo,
/// chegava ao meio do outro lado e cruzava-se).
#[test]
fn the_cutout_is_square_to_its_edges_and_stops_at_the_fillet_centre() {
    for n in [3, 5, 6] {
        for r in [0.05, 0.15, 0.25] {
            let p = polystar([0.0; 2], 1.0, 1.0, &poly(n, 0.0, r, CornerStyle::Cutout));
            let m = p.verts.len();
            assert_eq!(m, 3 * n as usize, "{n} lados r {r}");
            for k in 0..n as usize {
                let (a, c, b) = (
                    p.verts[3 * k].anchor,
                    p.verts[3 * k + 1].anchor,
                    p.verts[3 * k + 2].anchor,
                );
                let (pv, nv) = (
                    p.verts[(3 * k + m - 1) % m].anchor,
                    p.verts[(3 * k + 3) % m].anchor,
                );
                let wall = |p: [f64; 2], q: [f64; 2], e: [f64; 2]| {
                    let (u, v) = ([q[0] - p[0], q[1] - p[1]], [e[0] - p[0], e[1] - p[1]]);
                    (u[0] * v[0] + u[1] * v[1]) / (u[0].hypot(u[1]) * v[0].hypot(v[1]))
                };
                assert!(
                    wall(a, c, pv).abs() < 1e-9,
                    "{n}/{r}: parede de entrada {}",
                    wall(a, c, pv)
                );
                assert!(wall(b, c, nv).abs() < 1e-9, "{n}/{r}: parede de saída");
                // O fundo fica no raio da quina, entre ela e o centro (nunca do outro lado dele).
                let towards = (a[0] + b[0]) * c[0] + (a[1] + b[1]) * c[1];
                assert!(
                    c[0].hypot(c[1]) < 1.0 && towards > 0.0,
                    "{n}/{r}: o fundo {c:?}"
                );
            }
        }
    }
}

/// ⭐ **No tecto do raio (o lado inteiro comido) cada tipo continua o SEU** — o pedaço de
/// comprimento zero não tem tangente, e os três caíam na recta.
#[test]
fn a_corner_whose_edges_are_eaten_whole_keeps_its_style() {
    let at = |style| polystar([0.0; 2], 1.0, 1.0, &poly(3, 0.0, 5.0, style));
    let straight = at(CornerStyle::Straight);
    for style in [CornerStyle::Concave, CornerStyle::Cutout] {
        assert!(
            hausdorff(&at(style), &straight) > 0.1,
            "{style:?} virou a recta"
        );
    }
}

/// ⭐ (F13, o dono: «quando arredonda as quinas, fica ruim») **O estilo do canto com envergadura
/// e com liso**: com a envergadura, as paredes do *Cutout* e o chanfro do *Straight* ficam RECTOS
/// (envergados viravam ganchos); com o liso, nenhum vértice fica com raio (o estilo num vértice
/// liso deixava lascas — riscos do contorno ao centro).
#[test]
fn corner_styles_stay_clean_under_curve_and_smooth() {
    for style in [CornerStyle::Cutout, CornerStyle::Straight] {
        for c in [-0.6, -0.3, 0.3, 0.6] {
            let p = polystar(
                [0.0; 2],
                1.0,
                1.0,
                &PolyStar {
                    curve: c,
                    ..poly(5, 0.0, 0.2, style)
                },
            );
            let r = |q: [f64; 2]| q[0].hypot(q[1]);
            let n = p.verts.len();
            // As paredes e o chanfro: os troços que NÃO assentam num lado (as duas pontas para dentro
            // do círculo do polígono) têm as alças sobre a corda.
            let mut walls = 0;
            for k in 0..n {
                let (v, w) = (&p.verts[k], &p.verts[(k + 1) % n]);
                let d = [w.anchor[0] - v.anchor[0], w.anchor[1] - v.anchor[1]];
                let ld = d[0].hypot(d[1]);
                if ld < 1e-9 || (r(v.anchor) > 0.999 && r(w.anchor) > 0.999) {
                    continue;
                }
                let off = |h: [f64; 2]| {
                    ((h[0] - v.anchor[0]) * d[1] - (h[1] - v.anchor[1]) * d[0]).abs() / ld
                };
                if off(v.out_handle) < 1e-9 && off(w.in_handle) < 1e-9 {
                    walls += 1;
                }
            }
            assert!(
                walls >= 5,
                "{style:?} envergadura {c}: só {walls} troços rectos (paredes envergadas)"
            );
        }
        for s in [0.05, 0.5, 1.0] {
            // Liso, todo ponto do contorno é um vértice do polígono (no círculo de raio 1): a lasca
            // era um ponto puxado para dentro (o risco até ao centro).
            let p = polystar([0.0; 2], 1.0, 1.0, &poly(5, s, 0.2, style));
            let dentro = p
                .verts
                .iter()
                .map(|v| v.anchor[0].hypot(v.anchor[1]))
                .fold(1.0, f64::min);
            assert!(
                dentro > 0.999,
                "{style:?} liso {s}: um ponto a {dentro} do centro (lasca)"
            );
        }
    }
}
