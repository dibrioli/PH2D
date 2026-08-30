//! Gates da lei de amostragem.

use super::*;

const T: [f32; 4] = [0.0, 1.0, 2.0, 3.0];
const V: [f32; 4] = [10.0, 20.0, 30.0, 40.0];

#[test]
fn linear_walks_the_line_between_two_rows_and_step_holds_the_one_before() {
    assert!((sample(&T, &V, 1.5, true, false) - 25.0).abs() < 1e-5);
    assert!((sample(&T, &V, 1.5, false, false) - 20.0).abs() < 1e-5);
    // ⚠️ **O CONTROLE**: em cima de uma linha os dois têm de concordar, senão o gate passaria
    // com uma interpolação deslocada de meia amostra.
    for t in [0.0, 1.0, 2.0, 3.0] {
        assert_eq!(
            sample(&T, &V, t, true, false),
            sample(&T, &V, t, false, false),
            "em t={t} os dois modos têm de dar o mesmo"
        );
    }
}

#[test]
fn hold_clamps_at_both_ends_and_never_extrapolates() {
    assert_eq!(sample(&T, &V, -99.0, true, false), 10.0);
    assert_eq!(sample(&T, &V, 99.0, true, false), 40.0);
}

/// ⚠️ `rem_euclid` e não `%`: antes do início, o `%` do Rust é NEGATIVO e a repetição saltaria
/// para o fim do ficheiro em vez de continuar o ciclo.
#[test]
fn loop_repeats_forward_and_backward_the_same_way() {
    // O ciclo é [0,3): t=3.5 é t=0.5, e t=-0.5 é t=2.5.
    assert!((sample(&T, &V, 3.5, true, true) - sample(&T, &V, 0.5, true, true)).abs() < 1e-4);
    assert!((sample(&T, &V, -0.5, true, true) - sample(&T, &V, 2.5, true, true)).abs() < 1e-4);
    // ⚠️ **O CONTROLE**: com o `%` do Rust, `-0.5` daria `-0.5` ⇒ o clamp levaria-o a `10.0`.
    assert_ne!(sample(&T, &V, -0.5, true, true), 10.0);
}

/// ⚠️ Uma amostra só é o caso de quem está a MONTAR o ficheiro — devolver zero ali leria-se
/// como *"o vínculo não funciona"*.
#[test]
fn one_row_answers_at_every_instant() {
    for t in [-5.0, 0.0, 7.0] {
        assert_eq!(sample(&[2.0], &[42.0], t, true, false), 42.0);
        assert_eq!(sample(&[2.0], &[42.0], t, false, true), 42.0);
    }
}

#[test]
fn nothing_makes_it_panic_and_a_missing_column_reads_zero() {
    assert_eq!(sample(&[], &[], 1.0, true, true), 0.0);
    assert_eq!(sample(&T, &[], 1.0, true, true), 0.0);
    // Colunas de comprimento diferente: usa-se o menor, sem pânico.
    assert_eq!(sample(&T, &[7.0, 8.0], 5.0, true, false), 8.0);
    // Tempos repetidos (um degrau vertical) não dividem por zero.
    let flat = [1.0f32, 1.0, 1.0];
    let vals = [1.0f32, 2.0, 3.0];
    let v = sample(&flat, &vals, 1.0, true, false);
    assert!(v.is_finite(), "{v}");
    // Fora de ordem: não entra em pânico e devolve algo finito.
    let messy = [3.0f32, 1.0, 2.0];
    assert!(sample(&messy, &vals, 1.5, true, true).is_finite());
    // Um intervalo de comprimento zero com `Loop` não pode dividir por zero.
    assert!(sample(&[2.0, 2.0], &[1.0, 9.0], 5.0, true, true).is_finite());
}
