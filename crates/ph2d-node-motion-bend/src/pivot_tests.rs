//! **O PIVÔ DO `motion.bend`** — os gates do vocabulário partilhado e da extensão DERIVADA
//! (ciclo 3, W1 — doc 106 §2.3).
//!
//! Cortado do `lib.rs` pelo teto de LOC do HR-18 (700 para `crates/`), pela mesma costura que o
//! `direction_tests` e o `limits_tests` já usam: o corte é por RESPONSABILIDADE — este ficheiro
//! responde *«em torno de quê esta dobra corre?»*.

use super::*;
use ph2d_nodegraph::pivot::PivotMode;

/// ⭐⭐⭐ **A IDENTIDADE EM QUE O DISPOSITIVO SE APOIA, e ela é AO BIT.**
///
/// A extensão que o arco escala é `max_i |x_i − p|`. Enquanto o pivô era um número digitado
/// isso era uma redução só, com `p` dentro da expressão. Com o modo `Centroid` o `p` **é** uma
/// redução, e uma redução que depende de outra não é exprimível — o sequenciador corre-as todas
/// no mesmo passo.
///
/// A saída é derivar: `max_i |x_i − p| = max(xmax − p, p − xmin)`. ⚠️ **Em `f32` isto é uma
/// igualdade exacta e não uma aproximação**: a subtracção é correctamente arredondada e o
/// arredondamento é monótono, então o máximo dos arredondados é o arredondado do máximo — para
/// cada lado do pivô separadamente, e o `max` dos dois cobre os três casos (todos à direita,
/// todos à esquerda, e a straddle).
///
/// FALSIFICADO por um único `x` em que as duas contas difiram num ulp.
#[test]
fn the_derived_extent_is_bit_identical_to_the_fold_it_replaces() {
    // Coordenadas com mantissa cheia, dos dois lados do pivô e longe da origem — o regime em
    // que duas contas equivalentes na álgebra se separam no float, se se separarem.
    let xs: Vec<f32> = (0..4096)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "uma fixtura")]
            let t = i as f32;
            -137.317 + t * 0.061_803_4
        })
        .collect();
    for p in [
        0.0f32, 1.0, -1.0, 137.0, -137.0, 0.000_123, -412.5, 1e-7, 1e6,
    ] {
        let fold = xs.iter().map(|x| (x - p).abs()).fold(0.0_f32, f32::max);
        let xmin = xs.iter().copied().fold(f32::INFINITY, f32::min);
        let xmax = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let derived = (xmax - p).max(p - xmin);
        assert_eq!(
            fold.to_bits(),
            derived.to_bits(),
            "pivo {p}: o fold deu {fold} e a derivada {derived}"
        );
    }
}

/// **O default é o nó que shipou, AO BIT.**
///
/// ⚠️ E o default é `Point`, **não** o `World Origin` do `motion.transform`: este nó sempre
/// honrou o ponto digitado. *O default é a lei da identidade de cada nó, não uma propriedade do
/// enum* — e com `World Origin` o `ParamGate` esconderia dois sliders que o artista já usa.
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
    for pivot in [[0.0f32, 0.0], [3.5, -1.25], [-2.0, 0.75]] {
        let resolvido = PivotMode::of(1.0).resolve(pivot, &base);
        assert_eq!(resolvido, pivot, "o modo Point devolve o ponto, ao bit");
        let a = bend(
            &base, resolvido, 140.0, 0.0, 0, -1.0, 1.0, &[1.0], &falloff,
        );
        let b = bend(&base, pivot, 140.0, 0.0, 0, -1.0, 1.0, &[1.0], &falloff);
        assert_eq!(a, b, "pivo {pivot:?}");
    }
}

/// ⭐ **O modo `Centroid` dobra em torno do centro do layout, e SEGUE-O.**
///
/// A régua é a do produto e não a do número: com o pivô no centro, o elemento que ESTÁ no centro
/// é o ponto fixo da dobra — o arco cresce dos dois lados dele. Um `pivot_mode` ignorado poria o
/// vinco na origem, a `10` unidades de distância.
#[test]
fn the_centroid_mode_bends_about_the_centre_of_the_layout() {
    // Simétrico em torno de (10, 0) ⇒ centroide exacto, sem debate de arredondamento.
    let base = vec![[8.0f32, 0.0], [10.0, 0.0], [12.0, 0.0]];
    let falloff = vec![1.0f32; 3];
    let c = PivotMode::of(2.0).resolve([-99.0, 77.0], &base);
    assert_eq!(c, [10.0, 0.0], "o modo VENCE o ponto digitado");
    let out = bend(&base, c, 90.0, 0.0, 0, -1.0, 1.0, &[1.0], &falloff);
    assert!(
        (out[1][0] - 10.0).abs() < 1e-4 && out[1][1].abs() < 1e-4,
        "o elemento no pivo e' o ponto fixo da dobra: {:?}",
        out[1]
    );
    // E as duas pontas sobem, simetricas — o arco.
    assert!(
        out[0][1] > 0.1 && (out[0][1] - out[2][1]).abs() < 1e-4,
        "as pontas curvam por igual: {:?} e {:?}",
        out[0],
        out[2]
    );
}

/// **O vocabulário é o da porta** — a mesma régua dos irmãos, porque o achado que abriu este
/// ciclo foi *seis respostas diferentes à mesma pergunta*.
#[test]
fn the_pivot_vocabulary_is_the_ports() {
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == ph2d_nodegraph::pivot::PARAM)
        .expect("o no' declara o param do pivo");
    match hint.widget {
        ph2d_node_registry::ParamWidget::Enum { labels } => {
            assert_eq!(labels, ph2d_nodegraph::pivot::LABELS);
        }
        outro => panic!("o pivot_mode tem de ser um Enum, e' {outro:?}"),
    }
    // As duas primeiras reducoes sao as da porta, campo a campo.
    for (a, b) in REDUCES.iter().zip(ph2d_nodegraph::pivot::CENTROID_REDUCES) {
        assert_eq!((a.name, a.column, a.op, a.value), (b.name, b.column, b.op, b.value));
    }
}
