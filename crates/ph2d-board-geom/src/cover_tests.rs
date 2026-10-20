//! A cobertura do laço contra casos ANALÍTICOS (a resposta sai de uma conta, não do código).

use super::*;

fn square(x: f64, y: f64, s: f64) -> Vec<[f64; 2]> {
    vec![[x, y], [x + s, y], [x + s, y + s], [x, y + s]]
}

fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

#[test]
fn half_a_square_is_one_half() {
    let lasso = vec![[-1.0, -1.0], [0.5, -1.0], [0.5, 2.0], [-1.0, 2.0]];
    let c = area_cover(&[square(0.0, 0.0, 1.0)], &lasso);
    assert!(close(c, 0.5, 1e-12), "{c}");
}

/// Um círculo (polígono de 4096 lados) com o quadrado inscrito por laço: `2/π` (do polígono).
#[test]
fn the_inscribed_square_covers_two_over_pi_of_a_circle() {
    let n = 4096;
    let circle: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * f64::from(i) / f64::from(n);
            [a.cos(), a.sin()]
        })
        .collect();
    let r = std::f64::consts::FRAC_1_SQRT_2;
    let lasso = vec![[-r, -r], [r, -r], [r, r], [-r, r]];
    let poly_area = f64::from(n) / 2.0 * (std::f64::consts::TAU / f64::from(n)).sin();
    let c = area_cover(&[circle], &lasso);
    assert!(close(c, 2.0 / poly_area, 1e-9), "{c}");
}

/// O laço em 8 (as duas voltas em sentidos opostos) conta as duas metades.
#[test]
fn a_figure_eight_counts_both_loops() {
    let bow = vec![[-1.0, -1.0], [2.0, -1.0], [-1.0, 2.0], [2.0, 2.0]];
    let c = area_cover(&[square(0.0, 0.0, 1.0)], &bow);
    assert!(close(c, 0.5, 1e-12), "{c}");
}

/// ⭐ Non-zero, não par-ímpar: o laço que dá DUAS voltas ao mesmo sítio cobre-o (par-ímpar dava 0).
#[test]
fn a_loop_drawn_twice_still_covers() {
    let mut twice = square(-1.0, -1.0, 3.0);
    twice.extend(square(-1.0, -1.0, 3.0));
    let c = area_cover(&[square(0.0, 0.0, 1.0)], &twice);
    assert!(close(c, 1.0, 1e-12), "{c}");
    assert!(in_lasso(&twice, [0.5, 0.5]));
}

/// ⭐ O «C» côncavo: a CAIXA do rectângulo está toda dentro do laço, a área não (o entalhe tira 10 %).
#[test]
fn a_notch_takes_its_share_although_the_box_is_inside() {
    let c_shape = vec![
        [-1.0, -1.0],
        [11.0, -1.0],
        [11.0, 4.0],
        [5.0, 4.0],
        [5.0, 6.0],
        [11.0, 6.0],
        [11.0, 11.0],
        [-1.0, 11.0],
    ];
    let c = area_cover(&[square(0.0, 0.0, 10.0)], &c_shape);
    assert!(close(c, 0.9, 1e-12), "{c}");
}

#[test]
fn a_line_counts_by_its_length() {
    let lasso = square(0.0, 0.0, 1.0);
    let c = length_cover(&[vec![[-1.0, 0.5], [2.0, 0.5]]], &lasso);
    assert!(close(c, 1.0 / 3.0, 1e-12), "{c}");
    // Um ponto conta pelo ponto.
    assert_eq!(length_cover(&[vec![[0.5, 0.5]]], &lasso), 1.0);
    assert_eq!(length_cover(&[vec![[5.0, 0.5]]], &lasso), 0.0);
}

/// Um laço rodado e fino sobre um triângulo: a resposta da conta à mão (o triângulo `(0,0) (4,0)
/// (0,4)`, o laço é a faixa `1 ≤ x + y ≤ 3` ⇒ (área de x+y≤3 − área de x+y≤1) / 8 = (4,5 − 0,5)/8).
#[test]
fn a_slanted_band_over_a_triangle() {
    let tri = vec![[0.0, 0.0], [4.0, 0.0], [0.0, 4.0]];
    let band = vec![[1.0, 0.0], [3.0, 0.0], [0.0, 3.0], [0.0, 1.0]];
    let c = area_cover(&[tri], &band);
    assert!(close(c, 0.5, 1e-12), "{c}");
}
