//! **O PIVÔ DO `motion.twist`** — os gates do vocabulário partilhado (ciclo 3, W1 — doc 106).
//!
//! Cortado do `lib.rs` pelo teto de LOC do HR-18 (700 para `crates/`), pela mesma costura que o
//! `extent_tests` já usa.

use super::*;
use ph2d_nodegraph::pivot::PivotMode;

/// **O default é o nó que shipou, AO BIT.**
///
/// ⚠️ E o default é `Point`, **não** o `World Origin` do `motion.transform`: este nó sempre
/// honrou o ponto digitado. *O default é a lei da identidade de cada nó, não uma propriedade do
/// enum.*
#[test]
fn the_default_mode_is_the_node_that_shipped_bit_for_bit() {
    let base: Vec<[f32; 2]> = (0..64)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "uma fixtura")]
            let t = i as f32;
            [t * 0.137 - 4.0, t * 0.071 - 2.0]
        })
        .collect();
    let falloff = vec![1.0f32; base.len()];
    for pivot in [[0.0f32, 0.0], [2.75, 1.5], [-1.0, -3.0]] {
        let resolvido = PivotMode::of(1.0).resolve(pivot, &base);
        assert_eq!(resolvido, pivot, "o modo Point devolve o ponto, ao bit");
        let a = twist(&base, resolvido, 200.0, 0.0, 0, &[1.0], &falloff);
        let b = twist(&base, pivot, 200.0, 0.0, 0, &[1.0], &falloff);
        assert_eq!(a, b, "pivo {pivot:?}");
    }
}

/// ⭐ **O modo `Centroid` torce em torno do centro do layout.**
///
/// A régua é o PONTO FIXO: o elemento que está no centro não se move, porque o ângulo cresce com
/// o raio e ali o raio é zero. Um `pivot_mode` ignorado poria o eixo da espiral na origem.
#[test]
fn the_centroid_mode_twists_about_the_centre_of_the_layout() {
    // Simétrico em torno de (10, 5) ⇒ centroide exacto.
    let base = vec![[8.0f32, 5.0], [10.0, 5.0], [12.0, 5.0], [10.0, 3.0], [10.0, 7.0]];
    let falloff = vec![1.0f32; base.len()];
    let c = PivotMode::of(2.0).resolve([-99.0, 77.0], &base);
    assert_eq!(c, [10.0, 5.0], "o modo VENCE o ponto digitado");
    let out = twist(&base, c, 200.0, 0.0, 0, &[1.0], &falloff);
    assert!(
        (out[1][0] - 10.0).abs() < 1e-4 && (out[1][1] - 5.0).abs() < 1e-4,
        "o elemento no pivo e' o ponto fixo da torcao: {:?}",
        out[1]
    );
    // E as pontas MOVERAM-SE — senao o gate acima seria vacuo sobre um no-op.
    assert!(
        (out[0][1] - 5.0).abs() > 0.1,
        "a ponta tinha de girar: {:?}",
        out[0]
    );
}

/// **O vocabulário é o da porta, e a ORDEM das reduções é contrato.**
///
/// ⚠️ O `r_max` chama `reduce_cx()`/`reduce_cy()`, e uma redução só pode ler as declaradas
/// **antes** dela — trocar a ordem desta lista deixa o kernel a ler um símbolo que não existe.
#[test]
fn the_pivot_vocabulary_is_the_ports_and_the_order_is_contract() {
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == ph2d_nodegraph::pivot::PARAM)
        .expect("o no' declara o param do pivo");
    match hint.widget {
        ParamWidget::Enum { labels } => assert_eq!(labels, ph2d_nodegraph::pivot::LABELS),
        outro => panic!("o pivot_mode tem de ser um Enum, e' {outro:?}"),
    }
    let nomes: Vec<&str> = REDUCES.iter().map(|r| r.name).collect();
    assert_eq!(
        nomes,
        ["cx", "cy", "r_max"],
        "o r_max le' as duas primeiras, entao ele vem DEPOIS delas"
    );
    let rmax = REDUCES.last().expect("ha' tres");
    assert!(
        rmax.value.contains("reduce_cx(") && rmax.value.contains("reduce_cy("),
        "o r_max resolve o pivo a partir das somas — se deixar de o fazer, o modo Centroid \
         mede o raio a partir do sitio errado e nada o diz"
    );
}
