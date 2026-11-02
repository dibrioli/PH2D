//! O lado de uma recta (alças nas âncoras) é a própria recta: o meio, a normal e a distância de antes.

use super::*;

#[test]
fn a_straight_side_keeps_the_chord_midpoint_normal_and_distance() {
    let c = reta([0.0, 0.0], [10.0, 0.0]);
    let (m, n) = meio_e_normal(&c);
    assert!(
        (m[0] - 5.0).abs() < 1e-9 && m[1].abs() < 1e-9,
        "o meio: {m:?}"
    );
    assert!(
        n[0].abs() < 1e-9 && (n[1] + 1.0).abs() < 1e-9,
        "a normal (dy, −dx): {n:?}"
    );
    assert!((distancia(&c, [5.0, 3.0]) - 3.0).abs() < 1e-6);
    assert!(
        (distancia(&c, [13.0, 4.0]) - 5.0).abs() < 1e-6,
        "para além da ponta, a distância à ponta"
    );
    assert_eq!(caixa(&c), [[0.0, 0.0], [10.0, 0.0]]);
}
