//! Testes de `polystar.rs` — irmão (tecto de LOC). Os oráculos do Inkscape (o liso, a torção, a
//! divergência e o raio interno) correm no catálogo: `ph2d-tool-vector/src/shape_params_oracle_tests.rs`.

use super::*;

fn cubic_at(a: &VecVertex, b: &VecVertex, t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
    let p = [a.anchor, a.out_handle, b.in_handle, b.anchor];
    [
        w.iter().zip(&p).map(|(k, q)| k * q[0]).sum(),
        w.iter().zip(&p).map(|(k, q)| k * q[1]).sum(),
    ]
}

fn star(twist: f64, curve: f64, smooth: f64, radii: [f64; 2], style: CornerStyle) -> PolyStar {
    PolyStar {
        n: 5,
        inner: Some(0.45),
        twist_deg: twist,
        curve,
        smooth,
        radii,
        style,
    }
}

/// ⭐ **Os campos novos a ZERO são a função de sempre, byte a byte** — o polígono, a estrela e a
/// espiral de um save anterior à F10 cozinham exactamente o que cozinhavam.
#[test]
fn the_new_fields_at_zero_are_the_old_shapes_byte_for_byte() {
    let c = [0.3, -0.2];
    for r in [0.0, 0.2] {
        let poly = PolyStar {
            n: 6,
            radii: [r, 0.0],
            ..PolyStar::default()
        };
        assert_eq!(
            polystar(c, 2.0, 1.0, &poly),
            crate::regular_polygon_rounded(c, 2.0, 1.0, 6, r)
        );
        assert_eq!(
            polystar(
                c,
                2.0,
                1.0,
                &star(0.0, 0.0, 0.0, [r, r * 0.5], CornerStyle::Round)
            ),
            crate::star_rounded(c, 2.0, 1.0, 5, 0.45, r, r * 0.5)
        );
    }
    assert_eq!(
        spiral_ex(c, 2.0, 1.0, 3.0, 0.0, 0.0),
        crate::spiral(c, 2.0, 1.0, 3)
    );
}

/// ⭐ **O corte é o MESMO nos quatro tipos** — trocar o tipo só troca a ligação (o Illustrator
/// troca o tipo e mantém os pontos): as âncoras que pousam nas arestas do quadrado são iguais.
#[test]
fn the_four_corner_styles_cut_at_the_same_points() {
    let on_edge = |p: &VecPath| {
        let mut v: Vec<[i64; 2]> = p
            .verts
            .iter()
            .map(|v| v.anchor)
            .filter(|a| {
                let r = a[0].abs() + a[1].abs(); // o quadrado (losango) |x| + |y| = 1
                (r - 1.0).abs() < 1e-9
            })
            .map(|a| [(a[0] * 1e7).round() as i64, (a[1] * 1e7).round() as i64])
            .collect();
        v.sort_unstable();
        v
    };
    let sq = |style| PolyStar {
        n: 4,
        radii: [0.2, 0.0],
        style,
        ..PolyStar::default()
    };
    let round = on_edge(&polystar([0.0; 2], 1.0, 1.0, &sq(CornerStyle::Round)));
    assert_eq!(round.len(), 8, "4 quinas × 2 cortes");
    for style in [
        CornerStyle::Straight,
        CornerStyle::Concave,
        CornerStyle::Cutout,
    ] {
        assert_eq!(
            on_edge(&polystar([0.0; 2], 1.0, 1.0, &sq(style))),
            round,
            "{style:?}"
        );
    }
}

/// O CÔNCAVO morde para dentro (o meio da ligação fica mais perto do centro que a corda dos
/// cortes); o ARREDONDADO fica por fora dela; o ENTALHE tem o vértice `p_in + p_out − quina`.
#[test]
fn concave_bites_in_round_stays_out_and_the_cutout_is_a_notch() {
    let sq = |style| PolyStar {
        n: 4,
        radii: [0.2, 0.0],
        style,
        ..PolyStar::default()
    };
    // A quina do topo (0, 1): o meio da ligação diz de que lado ela vai.
    let top_link_mid = |p: &VecPath| {
        let k = p
            .verts
            .iter()
            .position(|v| v.anchor[1] > 0.0 && v.anchor[0] > 0.0)
            .unwrap();
        let (a, b) = (&p.verts[k], &p.verts[(k + 1) % p.verts.len()]);
        cubic_at(a, b, 0.5)
    };
    let round = polystar([0.0; 2], 1.0, 1.0, &sq(CornerStyle::Round));
    let concave = polystar([0.0; 2], 1.0, 1.0, &sq(CornerStyle::Concave));
    let (r, c) = (top_link_mid(&round), top_link_mid(&concave));
    assert!(
        r[1] > c[1] + 0.05,
        "côncavo {c:?} abaixo do arredondado {r:?}"
    );
    let cut = polystar([0.0; 2], 1.0, 1.0, &sq(CornerStyle::Cutout));
    assert_eq!(cut.verts.len(), 12, "4 quinas × (2 cortes + o entalhe)");
}

/// A ENVERGADURA: o meio da aresta anda `curve × meia corda` pela normal (para fora > 0); para
/// dentro nunca passa do centro.
#[test]
fn the_curve_pushes_the_edge_middle_by_half_a_chord() {
    let sq = |curve| PolyStar {
        n: 4,
        curve,
        ..PolyStar::default()
    };
    let p = polystar([0.0; 2], 1.0, 1.0, &sq(0.5));
    let m = cubic_at(&p.verts[0], &p.verts[1], 0.5);
    // A aresta do topo ao da esquerda: corda √2, meio a √2/2·… do centro.
    let chord = 2.0_f64.sqrt();
    let want = 0.5 * chord + 0.5 * chord * 0.5; // apótema (√2/2) + h (curve × meia corda)
    assert!((m[0].hypot(m[1]) - want).abs() < 1e-9, "meio {m:?}");
    let inward = polystar([0.0; 2], 1.0, 1.0, &sq(-1.0));
    let m = cubic_at(&inward.verts[0], &inward.verts[1], 0.5);
    assert!(
        m[0].hypot(m[1]) > 0.0 && m[0].hypot(m[1]) < 0.5,
        "para dentro, sem passar o centro"
    );
}

/// O LISO (o *Rounded* do Inkscape): o vértice é liso, as alças tangentes ao círculo dele, de
/// comprimento `smooth × corda até ao vizinho`.
#[test]
fn smooth_puts_tangent_handles_a_fraction_of_the_chord_long() {
    let p = polystar(
        [0.0; 2],
        1.0,
        1.0,
        &star(0.0, 0.0, 0.25, [0.0; 2], CornerStyle::Round),
    );
    let (tip, valley) = (&p.verts[0], &p.verts[1]);
    assert_eq!(tip.kind, VertexKind::Smooth);
    let chord = (valley.anchor[0] - tip.anchor[0]).hypot(valley.anchor[1] - tip.anchor[1]);
    let h = [
        tip.out_handle[0] - tip.anchor[0],
        tip.out_handle[1] - tip.anchor[1],
    ];
    assert!((h[0].hypot(h[1]) - 0.25 * chord).abs() < 1e-12);
    assert!(
        h[1].abs() < 1e-12,
        "a ponta do topo: alça horizontal (tangente)"
    );
}

/// A TORÇÃO roda os vales e deixa as pontas.
#[test]
fn twist_turns_the_valleys_and_keeps_the_tips() {
    let base = polystar(
        [0.0; 2],
        1.0,
        1.0,
        &star(0.0, 0.0, 0.0, [0.0; 2], CornerStyle::Round),
    );
    let tw = polystar(
        [0.0; 2],
        1.0,
        1.0,
        &star(10.0, 0.0, 0.0, [0.0; 2], CornerStyle::Round),
    );
    let ang = |p: [f64; 2]| p[1].atan2(p[0]).to_degrees();
    assert_eq!(base.verts[0].anchor, tw.verts[0].anchor);
    let d = ang(tw.verts[1].anchor) - ang(base.verts[1].anchor);
    assert!((d - 10.0).abs() < 1e-9, "o vale rodou {d}°");
}

/// A espiral divergente cresce como `t^e` e o raio interno corta o miolo.
#[test]
fn the_spiral_grows_as_t_to_the_e_and_starts_at_the_inner_radius() {
    let s = spiral_ex([0.0; 2], 1.0, 1.0, 3.0, 1.0, 0.25);
    let first = s.verts[0].anchor;
    assert!((first[0].hypot(first[1]) - 0.25_f64.powi(2)).abs() < 1e-12);
    let last = s.verts.last().unwrap().anchor;
    assert!((last[0].hypot(last[1]) - 1.0).abs() < 1e-12);
}

/// (F12) As voltas FRACCIONÁRIAS: a ponta de fora fica no ângulo `12h + voltas·360°` (3,25 → às
/// 9h, no aro) e a curva não salta ao passar por um inteiro (3,0001 ≈ 3, ponto a ponto da lei).
#[test]
fn a_fractional_spiral_ends_at_its_angle_and_meets_the_whole_one() {
    let tip = |t: f64| {
        spiral_ex([0.0; 2], 1.0, 1.0, t, 0.0, 0.0)
            .verts
            .last()
            .unwrap()
            .anchor
    };
    let p = tip(3.25);
    assert!((p[0] + 1.0).abs() < 1e-12 && p[1].abs() < 1e-12, "{p:?}");
    let whole = spiral_ex([0.0; 2], 1.0, 1.0, 3.0, 0.0, 0.0);
    let near = spiral_ex([0.0; 2], 1.0, 1.0, 3.0001, 0.0, 0.0);
    // A régua: cada amostra de uma à amostra MAIS PERTO da outra, esta 16× mais densa (o passo
    // entre amostras fica abaixo da tolerância).
    let flat = |s: &VecPath, n: u32| -> Vec<[f64; 2]> {
        s.verts
            .windows(2)
            .flat_map(|w| (0..n).map(move |k| cubic_at(&w[0], &w[1], f64::from(k) / f64::from(n))))
            .collect()
    };
    let dense = flat(&near, 128);
    let far = flat(&whole, 8)
        .iter()
        .map(|q| {
            dense
                .iter()
                .map(|r| (q[0] - r[0]).hypot(q[1] - r[1]))
                .fold(f64::MAX, f64::min)
        })
        .fold(0.0, f64::max);
    assert!(far < 2e-3, "a 3 voltas e a 3,0001 afastam-se {far}");
}

/// O CÔNCAVO morde para dentro da RECTA dos cortes (não só do arco): o meio da ligação côncava fica
/// mais perto do centro que o meio do chanfro — e o arco é o de raio `|corte − quina|`.
#[test]
fn concave_bites_inside_the_straight_chamfer() {
    let sq = |style| PolyStar {
        n: 4,
        radii: [0.2, 0.0],
        style,
        ..PolyStar::default()
    };
    let mid = |p: &VecPath| {
        let k = p
            .verts
            .iter()
            .position(|v| v.anchor[1] > 0.0 && v.anchor[0] > 0.0)
            .unwrap();
        cubic_at(&p.verts[k], &p.verts[(k + 1) % p.verts.len()], 0.5)
    };
    let (straight, concave) = (
        mid(&polystar([0.0; 2], 1.0, 1.0, &sq(CornerStyle::Straight))),
        mid(&polystar([0.0; 2], 1.0, 1.0, &sq(CornerStyle::Concave))),
    );
    assert!(
        concave[1] < straight[1] - 0.05,
        "côncavo {concave:?} abaixo da recta {straight:?}"
    );
    // O arco centrado na quina (0, 1): o meio dele a |corte − quina| da quina.
    let p = polystar([0.0; 2], 1.0, 1.0, &sq(CornerStyle::Concave));
    let cut = p.verts[0].anchor;
    let r = cut[0].hypot(cut[1] - 1.0);
    assert!(
        (concave[0].hypot(concave[1] - 1.0) - r).abs() < 1e-3 * r.max(1.0),
        "o arco é centrado na quina"
    );
}

/// O VALOR do campo escolhe o tipo: 0 arredondado, 1 recta, 2 côncavo, 3 entalhe (a ordem das
/// escolhas do descritor) — e o catálogo cozinha com ele.
#[test]
fn the_field_value_picks_the_corner_style_in_the_descriptor_order() {
    let want = [
        CornerStyle::Round,
        CornerStyle::Straight,
        CornerStyle::Concave,
        CornerStyle::Cutout,
    ];
    for (i, s) in want.into_iter().enumerate() {
        assert_eq!(CornerStyle::of(i as f64), s);
        let mut v = crate::ShapeKind::RoundRect.defaults();
        v[5] = i as f64;
        let (a, b) = ([-2.0, -1.0], [2.0, 1.0]);
        let r = crate::round_rect_radii(v[0], [0.0; 3]);
        assert_eq!(
            crate::cook(crate::ShapeKind::RoundRect, a, b, &v),
            round_rect_styled(a, b, r, 0.0, s),
            "o campo {i}"
        );
    }
}

/// ⭐ **Arredondar nunca engorda** — com a envergadura e o raio do canto, nenhum ponto do contorno
/// passa do vértice da ponta (pela ligação lisa entre duas arestas curvas passava `0,067`).
#[test]
fn the_rounding_never_grows_past_the_sharp_corner() {
    for (inner, curve) in [
        (None, 0.4),
        (None, -0.3),
        (Some(0.45), 0.4),
        (Some(0.45), -0.3),
    ] {
        let s = PolyStar {
            n: 5,
            inner,
            curve,
            radii: [0.15, 0.1],
            ..PolyStar::default()
        };
        let p = polystar([0.0; 2], 1.0, 1.0, &s);
        let top = (0..p.verts.len())
            .flat_map(|i| {
                let (a, b) = (&p.verts[i], &p.verts[(i + 1) % p.verts.len()]);
                (0..=32).map(move |k| cubic_at(a, b, f64::from(k) / 32.0)[1])
            })
            .fold(f64::MIN, f64::max);
        assert!(
            top <= 1.0 + 1e-9,
            "{inner:?} curve {curve}: o contorno chega a {top}"
        );
        // E a junção do arco com a aresta curva é lisa: as alças de cada âncora colineares.
        for v in &p.verts {
            let (i, o) = (
                [v.in_handle[0] - v.anchor[0], v.in_handle[1] - v.anchor[1]],
                [v.out_handle[0] - v.anchor[0], v.out_handle[1] - v.anchor[1]],
            );
            if i[0].hypot(i[1]) > 1e-9 && o[0].hypot(o[1]) > 1e-9 && inner.is_none() {
                let c = (i[0] * o[1] - i[1] * o[0]) / (i[0].hypot(i[1]) * o[0].hypot(o[1]));
                assert!(c.abs() < 1e-9, "quina na junção: {v:?}");
            }
        }
    }
}
