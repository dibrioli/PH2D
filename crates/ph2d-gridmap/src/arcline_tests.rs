use super::{Signed, measure_arc_lines};
use crate::cut::CutMesh;
use crate::solve::GridMap;
use crate::solve::turn2;

/// ⭐⭐⭐ **A IDENTIDADE QUE A EQUAÇÃO INTEIRA ASSENTA:** `e·(R^rot·v) = turn2(e, −rot)·v`.
///
/// ⚠️ **É aqui que um sinal troca sem que nada deixe de compilar.** O gate não lê as
/// entradas de matriz nenhuma — ele avalia os dois lados e compara, para os quatro `rot`
/// e os dois eixos. *Uma dedução à mão verificada por outra dedução à mão não é uma
/// verificação.*
#[test]
fn the_axis_identity_holds_for_every_turn() {
    let dot = |a: [f32; 2], b: [f32; 2]| a[0].mul_add(b[0], a[1] * b[1]);
    for e in [[1.0, 0.0], [0.0, 1.0]] {
        for rot in 0..4 {
            for v in [[1.0, 0.0], [0.0, 1.0], [3.0, -7.0], [-2.0, 5.0]] {
                let left = dot(e, turn2(v, rot));
                let right = dot(turn2(e, -rot), v);
                assert!(
                    (left - right).abs() < 1e-5,
                    "e={e:?} rot={rot} v={v:?}: {left} != {right}"
                );
            }
        }
    }
}

/// ⭐ E o resultado de `turn2(e, −rot)` é sempre um EIXO com sinal — é isso que faz os
/// coeficientes serem `±1` e a eliminação levar inteiros a inteiros.
#[test]
fn a_quarter_turn_of_an_axis_is_an_axis() {
    for e in [[1.0, 0.0], [0.0, 1.0]] {
        for rot in -4..=4 {
            let v = turn2(e, -rot);
            let zeros = usize::from(v[0].abs() < 1e-6) + usize::from(v[1].abs() < 1e-6);
            assert_eq!(zeros, 1, "e={e:?} rot={rot} ⇒ {v:?} nao e' um eixo");
            assert!((v[0].abs().max(v[1].abs()) - 1.0).abs() < 1e-6);
        }
    }
}

/// A união com sinal compõe: `y_c = σ₂·y_b + δ₂` e `y_b = σ₁·y_a + δ₁` ⇒ o `find` de `c`
/// tem de devolver `σ₁σ₂` e `σ₂δ₁ + δ₂`.
#[test]
fn the_signed_union_composes() {
    let mut uf = Signed::new(3);
    // `y_b = −1·y_a + 4`
    uf.parent[1] = 0;
    uf.sign[1] = -1.0;
    uf.off[1] = 4.0;
    // `y_c = −1·y_b + 3`
    uf.parent[2] = 1;
    uf.sign[2] = -1.0;
    uf.off[2] = 3.0;
    let (root, s, d) = uf.find(2);
    assert_eq!(root, 0);
    assert!((s - 1.0).abs() < 1e-6, "sinal {s}");
    // `y_c = −(−y_a + 4) + 3 = y_a − 1`
    assert!((d + 1.0).abs() < 1e-6, "deslocamento {d}");
}

/// ⚠️ E a compressão de caminho **não pode mudar a resposta** — ela reescreve `sign`/`off`
/// enquanto os lê. *Um `find` que se corrompe a si próprio dá a resposta certa uma vez.*
#[test]
fn path_compression_does_not_move_the_answer() {
    let mut uf = Signed::new(4);
    for (child, parent, s, d) in [
        (1u32, 0u32, -1.0, 4.0),
        (2, 1, -1.0, 3.0),
        (3, 2, -1.0, 1.0),
    ] {
        uf.parent[child as usize] = parent;
        uf.sign[child as usize] = s;
        uf.off[child as usize] = d;
    }
    let first = uf.find(3);
    let second = uf.find(3);
    assert_eq!(first.0, second.0);
    assert!((first.1 - second.1).abs() < 1e-6);
    assert!((first.2 - second.2).abs() < 1e-6);
}

/// Sem costuras não há equação nenhuma, e `0` ali significa **«nada a impor»**.
#[test]
fn no_seams_means_no_equations() {
    let cut = CutMesh::default();
    let w = crate::weld::Weld::default();
    let map = GridMap::default();
    let r = measure_arc_lines(&cut, &w, &map);
    assert_eq!(r.arcs, 0);
    assert_eq!(r.sign_conflicts, 0);
    assert_eq!(r.eliminated, 0);
}
