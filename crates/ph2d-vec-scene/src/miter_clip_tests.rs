//! A lei do bico cortado, por geometria (o oráculo `resvg` corre no `ph2d-tool-vector`).

use super::*;

fn dir(deg: f64) -> [f64; 2] {
    let r = deg.to_radians();
    [r.cos(), r.sin()]
}

/// Uma quina de ângulo interno `a` com o vértice na origem e a bissectriz para +y.
fn corner(a: f64) -> ([f64; 2], [f64; 2]) {
    let h = a / 2.0;
    (dir(90.0 - h), dir(h - 90.0))
}

#[test]
fn a_join_inside_the_limit_adds_nothing() {
    let (t1, t2) = corner(30.0); // 1/sin 15° = 3,86 < 4
    assert_eq!(clip_patch([0.0, 0.0], t1, t2, 1.0, 4.0), None);
}

/// ⭐ Contínuo: exactamente no limite o corte É o bico inteiro (A' = B' = a ponta do miter).
#[test]
fn at_the_limit_the_clip_is_the_whole_miter_tip() {
    let a = 2.0 * (1.0f64 / 4.0).asin().to_degrees() - 1e-9;
    let (t1, t2) = corner(a);
    let c = clip_patch([0.0, 0.0], t1, t2, 1.0, 4.0).expect("um fio abaixo do limite");
    for q in [c.quad[1], c.quad[2]] {
        assert!(q[0].abs() < 1e-6 && (q[1] - 4.0).abs() < 1e-6, "{q:?}");
    }
}

/// O corte fica a `limite × ½w` do vértice ao longo da bissectriz, em todo ângulo — até a
/// ida-e-volta (180°), onde vira o rectângulo `w × limite·½w`.
#[test]
fn the_clip_line_sits_at_limit_times_half_width_down_to_the_reversal() {
    for a in [20.0, 10.0, 4.0, 0.0] {
        let (t1, t2) = corner(a);
        let c = clip_patch([0.0, 0.0], t1, t2, 0.5, 4.0).expect("passa do limite");
        for q in [c.quad[1], c.quad[2]] {
            assert!((q[1] - 2.0).abs() < 1e-9, "ângulo {a}: {q:?}");
        }
        let width = (c.quad[1][0] - c.quad[2][0]).abs();
        assert!(
            a > 0.0 || (width - 1.0).abs() < 1e-9,
            "a ida-e-volta é a largura: {width}"
        );
    }
}

/// O corpo está do lado do vértice: a sonda fica DENTRO do triângulo do chanfro.
#[test]
fn the_probe_sits_inside_the_bevel_triangle() {
    let (t1, t2) = corner(10.0);
    let c = clip_patch([0.0, 0.0], t1, t2, 1.0, 4.0).unwrap();
    assert!(c.probe[1] > 0.0 && c.probe[1] < c.quad[0][1], "{c:?}");
}

#[test]
fn a_polyline_reports_only_its_sharp_joins() {
    let line = |a: [f64; 2], b: [f64; 2]| [a, a, b, b];
    let (p, q, r, s) = ([0.0, 0.0], [10.0, 0.0], [0.0, 1.0], [10.0, 2.0]);
    let open = vec![(vec![line(p, q), line(q, r), line(r, s)], false)];
    assert_eq!(clip_patches(&open, 0.5, 4.0).len(), 2);
    let tri = vec![(
        vec![line(p, q), line(q, [5.0, 8.0]), line([5.0, 8.0], p)],
        true,
    )];
    assert!(
        clip_patches(&tri, 0.5, 4.0).is_empty(),
        "60° cabe no limite"
    );
}
