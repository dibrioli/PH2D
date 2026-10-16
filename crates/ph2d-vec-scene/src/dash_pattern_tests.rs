use super::*;

#[test]
fn the_first_empty_pair_closes_the_pattern() {
    let mut p = DashPattern::single(3.0, 1.0);
    assert_eq!(p.active(), &[[3.0, 1.0]]);
    p.pairs[2] = [5.0, 5.0];
    assert_eq!(p.active().len(), 1, "o 3.º par não conta depois de um 2.º vazio");
    p.pairs[1] = [0.0, 2.0];
    assert_eq!(p.active(), &[[3.0, 1.0], [0.0, 2.0], [5.0, 5.0]]);
}

#[test]
fn a_first_dash_of_zero_is_solid() {
    let mut p = DashPattern::single(0.0, 4.0);
    p.pairs[1] = [2.0, 2.0];
    assert!(!p.is_dashed());
}

#[test]
fn lengths_alternate_dash_and_gap() {
    let l = DashLengths::from_pairs(&[[3.0, 1.0], [0.5, 2.0]]);
    assert_eq!(l.as_slice(), &[3.0, 1.0, 0.5, 2.0]);
    assert!((l.period() - 6.5).abs() < 1e-12);
    assert_eq!(l.first_dash(), 3.0);
    assert_eq!(l.scaled(2.0).as_slice(), &[6.0, 2.0, 1.0, 4.0]);
    assert_eq!(l.pairs().collect::<Vec<_>>(), vec![[3.0, 1.0], [0.5, 2.0]]);
}
