//! Os gates da folha do arco.

use super::*;

/// Uma linha reta de comprimento 3, para o arco ser aritmética à vista.
const RULER: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];

/// **A LUT é a distância acumulada, e a última entrada é o total.**
#[test]
fn the_lut_is_the_running_distance_and_ends_at_the_total() {
    assert_eq!(lut(&RULER), vec![0.0, 1.0, 2.0, 3.0]);
    assert!(lut(&RULER[..1]).is_empty(), "um ponto nao tem arco");
    assert!(lut(&[]).is_empty());
}

/// **A fração de arco é linear na régua** — `s` de 0 a 1 anda de 0 a 3 em x.
#[test]
fn the_arc_fraction_walks_the_ruler_linearly() {
    let l = lut(&RULER);
    for (s, x) in [(0.0, 0.0), (0.25, 0.75), (0.5, 1.5), (1.0, 3.0)] {
        let (p, t) = at(&RULER, &l, s);
        assert!(
            (p[0] - x).abs() < 1e-5,
            "s={s} caiu em {}, esperava {x}",
            p[0]
        );
        assert!((p[1]).abs() < 1e-6, "a regua e reta");
        assert_eq!(t, [1.0, 0.0], "e a tangente aponta para +x");
    }
}

/// **O amostrador CLAMPA, e é a metade que faz dele um amostrador.**
///
/// ⚠️ Este gate é a razão de a crate existir com esta assinatura. O código de que
/// ela foi extraída ENROLAVA `s` (`s − floor(s)`), e sob aquela política `s = 1,0`
/// devolve `0,0` — o último elemento de um `motion.spline_wrap` saltaria para o
/// começo da curva. A política de ponta é do NÓ; o amostrador só clampa.
#[test]
fn the_sampler_clamps_the_ends_it_does_not_wrap_them() {
    let l = lut(&RULER);
    assert!(
        (at(&RULER, &l, 1.0).0[0] - 3.0).abs() < 1e-5,
        "s = 1 e o FIM da curva, nao o comeco"
    );
    assert!(
        (at(&RULER, &l, 1.7).0[0] - 3.0).abs() < 1e-5,
        "alem do fim, o fim"
    );
    assert!(
        (at(&RULER, &l, -0.4).0[0]).abs() < 1e-6,
        "aquem do comeco, o comeco"
    );
}

/// **A tangente segue o segmento em que o ponto caiu**, e um canto a troca.
#[test]
fn the_tangent_follows_the_segment_the_point_landed_in() {
    let ell = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0]];
    let l = lut(&ell);
    assert_eq!(l, vec![0.0, 2.0, 4.0]);
    assert_eq!(at(&ell, &l, 0.25).1, [1.0, 0.0], "na perna horizontal");
    assert_eq!(at(&ell, &l, 0.75).1, [0.0, 1.0], "na perna vertical");
}

/// **O degenerado não inventa uma direção** — sem arco, o primeiro ponto e `+x`.
#[test]
fn a_curve_with_no_length_returns_its_point_and_a_plain_tangent() {
    let dot = [[5.0, 7.0], [5.0, 7.0]];
    let l = lut(&dot);
    assert_eq!(at(&dot, &l, 0.5), ([5.0, 7.0], [1.0, 0.0]));
    assert_eq!(at(&[], &[], 0.5), ([0.0, 0.0], [1.0, 0.0]));
}
