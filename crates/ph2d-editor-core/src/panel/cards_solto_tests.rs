use super::*;

/// T42: parado, o nome em destaque fica inteiro; com o rato por cima pisca devagar — inteiro no
/// início do período, no mínimo a meio, e volta.
/// *Mutação: `piscar` sem o rato ⇒ o meio do período fica inteiro.*
#[test]
fn the_pinned_subject_blinks_slowly_only_under_the_pointer() {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ns = |s: f64| (s * 1e9) as u128;
    let meio = ns(m::LOOSE_PIN_BLINK_S * 0.5);
    assert!((piscar(meio, false) - 1.0).abs() < 1e-6);
    assert!((piscar(0, true) - 1.0).abs() < 1e-6);
    assert!((piscar(meio, true) - m::LOOSE_PIN_BLINK_MIN).abs() < 1e-3);
    assert!((piscar(ns(m::LOOSE_PIN_BLINK_S), true) - 1.0).abs() < 1e-3);
}
