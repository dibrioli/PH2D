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

/// ⭐ **O 1.º passo do liso não apaga o raio do canto**: liso `0,001` desenha a forma de liso `0`
/// (antes, a estrela arredondada ficava pontuda — «no mínimo o maior efeito»).
#[test]
fn a_little_smooth_keeps_the_corner_radius() {
    for (name, mk) in [
        (
            "polígono",
            (|s| poly(5, s, 0.25, CornerStyle::Round)) as fn(f64) -> PolyStar,
        ),
        ("estrela", |s| star(s, 0.25)),
    ] {
        let (a, b) = (
            polystar([0.0; 2], 1.0, 1.0, &mk(0.0)),
            polystar([0.0; 2], 1.0, 1.0, &mk(0.001)),
        );
        let h = hausdorff(&a, &b);
        assert!(h < 2e-3, "{name}: {h}");
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

/// ⭐ **Com raio, o vértice fica QUINA sob o liso e a envergadura** (`liso_em`: só o vértice sem raio
/// é liso). Se o raio também alinhasse as alças (G1), as tangentes do canto ficariam colineares e o
/// arredondado não teria o que arredondar — o raio sumia (medido: o efeito do raio `0,25` cai ~5×).
#[test]
fn with_a_radius_the_smooth_vertex_stays_a_corner_and_the_radius_shows() {
    for (s, c, min) in [
        (0.3, 0.4, 0.01),
        (0.3, -0.4, 0.1),
        (0.1, 0.4, 0.01),
        (0.5, 0.2, 0.01),
    ] {
        let mk = |r: f64| PolyStar {
            curve: c,
            ..poly(5, s, r, CornerStyle::Round)
        };
        let (a, b) = (
            polystar([0.0; 2], 1.0, 1.0, &mk(0.0)),
            polystar([0.0; 2], 1.0, 1.0, &mk(0.25)),
        );
        let h = hausdorff(&a, &b);
        assert!(h >= min, "liso {s}, envergadura {c}: o raio só mexe {h}");
    }
}
