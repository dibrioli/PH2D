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

/// **A bisseção que o Newton substituiu**, preservada como ORÁCULO de teste.
///
/// Não é uma segunda porta de produto: é a implementação de referência contra a qual a troca
/// de algoritmo se justifica. Converge sempre e não pede derivada — 40 halvings levam o
/// intervalo a `2^-40 ≈ 9e-13` do domínio de `t`.
fn inv_arclen_bisect(c: &Cubic, s: f64) -> f64 {
    let total = arclen(c);
    if s <= 0.0 || total <= 0.0 {
        return 0.0;
    }
    if s >= total {
        return 1.0;
    }
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if arclen_to(c, mid) < s {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// **O Newton concorda com a bisseção que ele substituiu** — e o gate diz de QUANTO, porque
/// "concorda" sem número é a afirmação que uma impressão digital já faz, e pior: um hash não
/// distingue *mudou 1e-15* de *mudou tudo*.
///
/// A troca (2026-07-23, ADR-0141 Fatia 0) foi por CUSTO — 1700 ns → 140 ns, medido — e o preço
/// dela é este épsilon. Ele é reportado em unidades de MUNDO sobre uma curva de tamanho de
/// produto, que é a grandeza em que alguém decide se é visível.
#[test]
fn the_newton_inverse_agrees_with_the_bisection_it_replaced() {
    let cs = [curved(), straightish(), sharp()];
    let (mut worst_t, mut worst_p) = (0.0f64, 0.0f64);
    for c in &cs {
        let total = arclen(c);
        for i in 0..=2000 {
            let s = total * f64::from(i) / 2000.0;
            let (a, b) = (inv_arclen(c, s), inv_arclen_bisect(c, s));
            worst_t = worst_t.max((a - b).abs());
            let (pa, pb) = (point_at(c, a), point_at(c, b));
            worst_p = worst_p.max((pa[0] - pb[0]).hypot(pa[1] - pb[1]));
        }
    }
    // Medido: 1e-12 em `t`, ~1e-10 unidades de mundo numa curva de ~100 unidades. O Newton é
    // o MAIS preciso dos dois (para na tolerância; a bisseção para na contagem), então este
    // número é a distância entre duas respostas certas, não um erro.
    assert!(
        worst_t < 1e-9,
        "o `t` divergiu {worst_t:.3e} da bisseção — mais que os 1e-9 que a tolerância promete"
    );
    assert!(
        worst_p < 1e-6,
        "o PONTO divergiu {worst_p:.3e} unidades de mundo — visível seria ~1e-2"
    );
    eprintln!("Newton vs bisseção: dt = {worst_t:.3e}, dponto = {worst_p:.3e} unidades");
}

/// Uma cúbica quase reta: o palpite inicial do Newton (`s/total`) é EXATO aqui, então este é o
/// caso em que ele sai numa iteração — e é o que garante que o caso fácil não regrediu.
fn straightish() -> Cubic {
    [[0.0, 0.0], [S, 0.1], [2.0 * S, -0.1], [3.0 * S, 0.0]]
}

/// Uma cúbica de curvatura forte com quase-cúspide: `|B'|` chega perto de zero no meio, que é
/// onde Newton divide por quase-nada e a CERCA de bisseção tem de assumir.
fn sharp() -> Cubic {
    [[0.0, 0.0], [S, 0.0], [-S, 0.0], [0.0, S]]
}
