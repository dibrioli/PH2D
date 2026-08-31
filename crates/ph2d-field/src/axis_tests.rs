//! As leis do [`Axis`], e a que custaria a quiralidade.

use super::Axis;

/// ⭐ A rotação é **cíclica e existe sempre** — para qualquer par de eixos.
#[test]
fn the_shift_between_any_two_axes_is_a_cyclic_rotation() {
    for de in Axis::ALL {
        for para in Axis::ALL {
            let s = de.shift_to(para);
            assert!(s < 3, "{de:?} -> {para:?} deu um passo de {s}");
            // A rotação leva mesmo o eixo escolhido ao lugar do canónico.
            let mut v = [0.0f32; 3];
            v[usize::from(de.index())] = 1.0;
            let canon = Axis::to_canonical(v, s);
            assert!(
                (canon[usize::from(para.index())] - 1.0).abs() < f32::EPSILON,
                "{de:?} -> {para:?} (passo {s}) pôs o 1 em {canon:?}"
            );
        }
    }
}

/// ⛔⛔ **A ida e a volta são a IDENTIDADE** — se não fossem, o modificador devolveria a peça noutro
/// sítio, e o defeito leria-se como «o eixo Y está deslocado».
#[test]
fn to_canonical_and_back_is_the_identity() {
    let v = [1.0f32, 2.0, 3.0];
    for s in 0..3 {
        let ida = Axis::to_canonical(v, s);
        let volta = Axis::from_canonical(ida, s);
        assert_eq!(volta, v, "passo {s}: {v:?} -> {ida:?} -> {volta:?}");
    }
}

/// ⛔⛔⛔ **A PERMUTAÇÃO PRESERVA A ORIENTAÇÃO** — o gate que defende a quiralidade.
///
/// Uma troca de dois eixos resolveria o mesmo par `(de, para)` e tem determinante `−1`: ela
/// **espelha** a peça, e uma torção espelhada gira ao contrário. Só as cíclicas servem, e é isto que
/// o afirma — o produto vectorial dos dois primeiros eixos permutados tem de dar o terceiro.
#[test]
fn the_permutation_never_mirrors_the_piece() {
    for s in 0..3 {
        let e = |i: usize| {
            let mut v = [0.0f32; 3];
            v[i] = 1.0;
            Axis::to_canonical(v, s)
        };
        let (a, b, c) = (e(0), e(1), e(2));
        let cruz = [
            a[1].mul_add(b[2], -(a[2] * b[1])),
            a[2].mul_add(b[0], -(a[0] * b[2])),
            a[0].mul_add(b[1], -(a[1] * b[0])),
        ];
        assert_eq!(
            cruz, c,
            "passo {s}: a permutação tem determinante -1 e ESPELHA a peça"
        );
    }
}

/// ⭐ A porta **coage** em vez de recusar — a lei do módulo.
#[test]
fn an_out_of_range_index_is_coerced_and_never_refused() {
    assert_eq!(Axis::from_index(-4.0), Axis::X);
    assert_eq!(Axis::from_index(0.4), Axis::X);
    assert_eq!(Axis::from_index(1.0), Axis::Y);
    assert_eq!(Axis::from_index(2.49), Axis::Z);
    assert_eq!(Axis::from_index(99.0), Axis::Z);
    // ⛔ **A ida e a volta pelo índice**, que é o caminho do painel.
    for a in Axis::ALL {
        assert_eq!(Axis::from_index(f32::from(a.index())), a);
    }
}
