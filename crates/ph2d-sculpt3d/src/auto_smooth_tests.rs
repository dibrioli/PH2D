//! Os gates do [`super`] — **a tabela vem da REFERÊNCIA, não desta função**.
//!
//! ⚠️ **Nenhum número abaixo é computado com `iteration_strengths`.** Eles são a
//! leitura de `brushes/smooth.cc:34-48` reduzida à mão, entrada por entrada. Um
//! oráculo que chama a função sob teste para dizer o que espera é verde sobre
//! qualquer lei — a doença que este repositório já registou várias vezes.

use super::*;

/// Um caso da tabela: o fator, quantas passadas, e as forças.
struct Case {
    factor: f32,
    want: &'static [f32],
}

/// ⚠️ **Os quatro valores exactos (`0,25 · 0,50 · 0,75 · 1,00`) são o coração
/// da tabela** — é neles que a lei do Blender deixa de ser uma reescala e passa
/// a ser uma CONTAGEM. Um port por lerp único acerta `0,24` e erra todos eles.
const TABLE: &[Case] = &[
    Case {
        factor: 0.0,
        want: &[0.0],
    },
    Case {
        factor: 0.05,
        want: &[0.20],
    },
    Case {
        factor: 0.10,
        want: &[0.40],
    },
    Case {
        factor: 0.20,
        want: &[0.80],
    },
    Case {
        factor: 0.24,
        want: &[0.96],
    },
    Case {
        factor: 0.25,
        want: &[1.0, 0.0],
    },
    Case {
        factor: 0.30,
        want: &[1.0, 0.20],
    },
    Case {
        factor: 0.50,
        want: &[1.0, 1.0, 0.0],
    },
    Case {
        factor: 0.75,
        want: &[1.0, 1.0, 1.0, 0.0],
    },
    Case {
        factor: 1.0,
        want: &[1.0, 1.0, 1.0, 1.0, 0.0],
    },
];

#[test]
fn the_budget_is_a_count_of_full_laplacian_passes_not_a_single_lerp() {
    for case in TABLE {
        let got = iteration_strengths(case.factor);
        let got = got.as_slice();
        assert_eq!(
            got.len(),
            case.want.len(),
            "fator {}: o Blender dá {} passada(s) e este port dá {} — \
             um lerp único daria SEMPRE uma",
            case.factor,
            case.want.len(),
            got.len()
        );
        for (i, (g, w)) in got.iter().zip(case.want).enumerate() {
            assert!(
                (g.weight - w).abs() < 1e-6,
                "fator {} passada {i}: {} contra {w}",
                case.factor,
                g.weight
            );
        }
    }
}

/// As passadas cheias são **exactamente** `1,0`, e isso é o que as torna
/// SUBSTITUIÇÕES em vez de misturas.
///
/// ⚠️ Um `1,0` aproximado deixaria um resíduo da posição antiga em cada
/// passada, e o erro composto quatro vezes é visível — por isso este gate pede
/// igualdade de bits e não uma tolerância.
#[test]
fn a_full_pass_is_exactly_one_so_it_replaces_instead_of_mixing() {
    let it = iteration_strengths(1.0);
    let got = it.as_slice();
    for (i, p) in got.iter().take(MAX_ITERATIONS).enumerate() {
        assert!(
            p.weight.to_bits() == 1.0f32.to_bits(),
            "passada {i}: {} não é 1,0 ao bit",
            p.weight
        );
    }
}

/// **O orçamento é LINEAR no fator** (`Σ = 4c`) — a propriedade que a
/// quantização em quartos preserva, e que um gate de tabela sozinho não afirma.
#[test]
fn the_total_budget_is_four_times_the_factor() {
    for k in 0..=40 {
        let c = k as f32 / 40.0;
        let it = iteration_strengths(c);
        let sum: f32 = it.as_slice().iter().map(|p| p.weight).sum();
        assert!(
            (sum - 4.0 * c).abs() < 1e-5,
            "fator {c}: orçamento {sum} contra {}",
            4.0 * c
        );
    }
}

/// Acima de `1,0` o Blender **clampa** (`std::min(strength, 1.0f)`), e não
/// extrapola: o tecto do orçamento é `4,0`.
#[test]
fn the_factor_is_clamped_so_the_budget_tops_out_at_four() {
    let over = iteration_strengths(1.5);
    let at = iteration_strengths(1.0);
    assert_eq!(over.as_slice().len(), at.as_slice().len());
    let a: f32 = over.as_slice().iter().map(|p| p.weight).sum();
    let b: f32 = at.as_slice().iter().map(|p| p.weight).sum();
    assert!((a - b).abs() < 1e-6, "{a} contra {b}");
}

/// O buffer nunca transborda — o tecto de [`MAX_PASSES`] é uma consequência da
/// lei, não uma esperança.
#[test]
fn the_pass_count_never_exceeds_the_ceiling() {
    for k in 0..=200 {
        let c = k as f32 / 100.0;
        assert!(iteration_strengths(c).as_slice().len() <= MAX_PASSES, "{c}");
    }
}
