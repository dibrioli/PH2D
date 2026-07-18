//! Gates do motor de arco. **Os oráculos são independentes do método**: uma reta tem
//! comprimento fechado, e uma curva é medida contra amostragem densa — nunca contra outra
//! chamada da mesma Gauss-Legendre, que seria o método a concordar consigo mesmo.

use super::*;

/// Números do produto, não `1.0`: uma escala unitária esconde erro de unidade e de fator.
/// [[feedback_test_with_product_numbers_not_convenient_ones]]
const S: f64 = 37.5;

/// Comprimento por amostragem densa da polilinha — o oráculo externo. Converge para o arco
/// verdadeiro por baixo (a corda é sempre menor que o arco), então é um piso apertado.
fn dense_len(c: &Cubic, n: usize) -> f64 {
    let at = |t: f64| {
        let u = 1.0 - t;
        let mut p = [0.0; 2];
        for k in 0..2 {
            p[k] = u * u * u * c[0][k]
                + 3.0 * u * u * t * c[1][k]
                + 3.0 * u * t * t * c[2][k]
                + t * t * t * c[3][k];
        }
        p
    };
    let mut sum = 0.0;
    let mut prev = at(0.0);
    for i in 1..=n {
        let p = at(i as f64 / n as f64);
        sum += ((p[0] - prev[0]).powi(2) + (p[1] - prev[1]).powi(2)).sqrt();
        prev = p;
    }
    sum
}

/// Uma cúbica curva de verdade (não degenerada, não simétrica).
fn curved() -> Cubic {
    [
        [0.0, 0.0],
        [S * 0.3, S * 1.1],
        [S * 1.4, S * 0.9],
        [S * 1.7, S * 0.2],
    ]
}

/// **Uma RETA tem comprimento fechado, e a quadratura tem de o acertar exatamente.**
///
/// A reta entra na forma canónica (⅓, ⅔) — a que é afim em `t`. É a mesma armadilha que o
/// blend pagou: `(P0, P0, P3, P3)` é a mesma *curva* com parametrização não-uniforme.
#[test]
fn a_straight_line_has_its_closed_form_length() {
    let (a, b) = ([11.0, -4.0], [11.0 + 3.0 * S, -4.0 + 4.0 * S]);
    let third = |k: usize| a[k] + (b[k] - a[k]) / 3.0;
    let two_thirds = |k: usize| a[k] + (b[k] - a[k]) * 2.0 / 3.0;
    let line: Cubic = [a, [third(0), third(1)], [two_thirds(0), two_thirds(1)], b];
    // 3-4-5: o comprimento é exatamente 5·S.
    assert!(
        (arclen(&line) - 5.0 * S).abs() < 1e-9,
        "reta 3-4-5 devia medir {}, mediu {}",
        5.0 * S,
        arclen(&line)
    );
}

/// **Numa curva, a quadratura concorda com amostragem densa.**
///
/// O oráculo é externo ao método. 200k cordas ficam a ~1e-8 do arco verdadeiro nesta escala;
/// exigir 1e-6 relativo é apertado e não flaka.
#[test]
fn a_curve_agrees_with_dense_sampling() {
    let c = curved();
    let (gl, dense) = (arclen(&c), dense_len(&c, 200_000));
    assert!(
        ((gl - dense) / dense).abs() < 1e-6,
        "GL16 = {gl}, amostragem densa = {dense}"
    );
}

/// **O inverso é o inverso**: pedir o `t` do comprimento que `t` produziu devolve `t`.
#[test]
fn the_inverse_round_trips_across_the_whole_domain() {
    let c = curved();
    for i in 0..=20 {
        let t = f64::from(i) / 20.0;
        let back = inv_arclen(&c, arclen_to(&c, t));
        assert!((back - t).abs() < 1e-9, "t = {t} voltou {back}");
    }
}

/// **O `t` do meio do arco NÃO é `0.5`** — é a armadilha que este módulo existe para evitar,
/// e um gate que não a contenha deixaria passar uma implementação que só devolve `s / total`.
#[test]
fn the_midpoint_of_arc_is_not_the_midpoint_of_t() {
    let c = curved();
    let t_mid = inv_arclen(&c, arclen(&c) * 0.5);
    assert!(
        (t_mid - 0.5).abs() > 1e-3,
        "nesta curva o meio do arco cai em t = {t_mid}; se der 0.5 a parametrização foi \
         confundida com o comprimento"
    );
}

/// Cortar em `[0, 1]` devolve a mesma curva, ao bit — o ponto neutro do `subsegment`.
#[test]
fn the_full_subsegment_is_the_curve_itself() {
    let c = curved();
    let s = subsegment(&c, 0.0, 1.0);
    for (i, (a, b)) in s.iter().zip(c.iter()).enumerate() {
        assert!(
            (a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12,
            "ponto {i}: {a:?} != {b:?}"
        );
    }
}

/// **As duas metades somam o todo** — o `subsegment` corta sem criar nem perder arco.
#[test]
fn the_two_halves_of_a_split_sum_to_the_whole() {
    let c = curved();
    let t = 0.37;
    let sum = arclen(&subsegment(&c, 0.0, t)) + arclen(&subsegment(&c, t, 1.0));
    assert!(
        ((sum - arclen(&c)) / arclen(&c)).abs() < 1e-9,
        "as metades somam {sum}, o todo mede {}",
        arclen(&c)
    );
}
