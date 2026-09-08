//! ⭐⭐⭐ **OS GATES DO OSSO INTELIGENTE** (`action_time`) — irmão do [`super::reach_tests`] pelo teto
//! de LOC, e coeso pelo assunto: aqui mede-se *«que instante de uma acção este ângulo pede»*, que é
//! a lei do *Smart Bone* do Moho e do *Action Constraint* do Blender.
//!
//! ⚠️ Ela é PURA e não sabe o que é um clip: quem lhe entrega a duração é a shell, que a lê do
//! `clip_end_seconds` — a grandeza crua (`duration()`) tem nome parecido e resposta diferente.

use super::reach::*;

/// ⭐⭐⭐ **O ÂNGULO PERCORRE A ACÇÃO, e APARA nos extremos.**
#[test]
fn the_action_runs_from_end_to_end_and_stops_there() {
    let (from, to, dur) = (0.0, 1.0, 4.0);
    assert!(
        (action_time(0.0, from, to, dur) - 0.0).abs() < 1e-12,
        "no princípio"
    );
    assert!(
        (action_time(0.5, from, to, dur) - 2.0).abs() < 1e-12,
        "a meio"
    );
    assert!(
        (action_time(1.0, from, to, dur) - 4.0).abs() < 1e-12,
        "no fim"
    );
    // ⛔ Passar do fim FICA no fim — não embrulha nem continua.
    assert!(
        (action_time(9.0, from, to, dur) - 4.0).abs() < 1e-12,
        "muito além"
    );
    assert!(
        (action_time(-9.0, from, to, dur) - 0.0).abs() < 1e-12,
        "muito aquém"
    );
}

/// ⭐⭐ **Uma faixa INVERTIDA percorre a acção ao contrário** — é o controlo que gira para o outro
/// lado, não um erro a corrigir.
#[test]
fn a_reversed_range_runs_the_action_backwards() {
    let (from, to, dur) = (1.0, 0.0, 4.0);
    assert!((action_time(1.0, from, to, dur) - 0.0).abs() < 1e-12);
    assert!((action_time(0.5, from, to, dur) - 2.0).abs() < 1e-12);
    assert!((action_time(0.0, from, to, dur) - 4.0).abs() < 1e-12);
}

/// ⛔ **Faixa NULA, duração negativa e valores não-finitos devolvem o PRINCÍPIO** — nunca um `NaN`.
///
/// ⚠️ Um `NaN` daqui viajaria para dentro do `sample` do clip e levaria a pose inteira com ele: a
/// forma desaparece e nada diz porquê. *Uma lei que pode receber lixo de um ficheiro devolve a
/// resposta inerte, não a propaga.*
#[test]
fn a_null_range_or_garbage_returns_the_start_never_a_nan() {
    for (r, f, t, d) in [
        (0.5, 1.0, 1.0, 4.0),
        (0.5, 0.0, 1.0, -3.0),
        (f64::NAN, 0.0, 1.0, 4.0),
        (0.5, f64::INFINITY, 1.0, 4.0),
        (0.5, 0.0, 1.0, f64::NAN),
    ] {
        let v = action_time(r, f, t, d);
        assert!(v.is_finite(), "action_time({r},{f},{t},{d}) = {v}");
        assert!(v >= 0.0, "o tempo de uma acção nunca é negativo: {v}");
    }
}
