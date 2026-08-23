//! Os gates do [`super::MODE_KEEP_LENGTH`] — a parametrização (doc 89, folha 04).

use super::*;

/// A curva de referência: uma RECTA de `x = −6` a `x = 6`, comprimento 12. Uma recta é
/// escolhida de propósito — nela o arco e o `x` são a mesma coisa, então o que o gate mede é a
/// PARAMETRIZAÇÃO e não a forma.
const RULER: [P2; 4] = [[-6.0, 0.0], [-2.0, 0.0], [2.0, 0.0], [6.0, 0.0]];

const WHOLE: ArcMap = ArcMap {
    from: 0.0,
    to: 1.0,
    offset: 0.0,
};

/// Uma fileira de `n` pontos, larga `w`, centrada na origem.
fn row(n: usize, w: f32) -> Vec<P2> {
    (0..n)
        .map(|i| [w * (i as f32 / (n - 1) as f32 - 0.5), 0.0])
        .collect()
}

/// O `x` de cada ponto depois de embrulhado.
fn xs(p: &[P2], keep: bool) -> Vec<f32> {
    wrap(p, &Curve::cubic(&RULER), 0.0, WHOLE, keep, &[1.0], &[])
        .iter()
        .map(|q| q[0])
        .collect()
}

/// **`Fit` ESTICA ATÉ AS PONTAS, seja qual for o tamanho do layout** — o nó que sempre shipou.
#[test]
fn fit_stretches_any_layout_onto_the_whole_curve() {
    for w in [1.0f32, 4.0, 20.0] {
        let x = xs(&row(5, w), false);
        assert!(
            (x[0] + 6.0).abs() < 0.05 && (x[4] - 6.0).abs() < 0.05,
            "largura {w}: as pontas têm de tocar as pontas da curva: {x:?}"
        );
    }
}

/// **`Keep Length` MANTÉM A ESCALA DO LAYOUT** — e é isto que o `Fit` não sabe fazer.
///
/// ⚠️ O oráculo é a largura OCUPADA, não as posições: um layout de 3 unidades numa curva de 12
/// tem de ocupar 3, e um de 6 tem de ocupar 6. No `Fit` os dois ocupariam 12.
#[test]
fn keep_length_keeps_the_layout_scale() {
    for w in [3.0f32, 6.0] {
        let x = xs(&row(5, w), true);
        let span = x[4] - x[0];
        assert!(
            (span - w).abs() < 0.1,
            "um layout de {w} tem de ocupar {w} na curva, e ocupou {span:.3}"
        );
    }
    // E o CONTROLE: no `Fit` os dois ocupam a curva inteira, ou seja o mesmo número.
    let a = xs(&row(5, 3.0), false);
    let b = xs(&row(5, 6.0), false);
    assert!(
        ((a[4] - a[0]) - (b[4] - b[0])).abs() < 0.05,
        "no Fit a largura do layout não pode mudar o que ele ocupa"
    );
}

/// **UM LAYOUT MAIS LONGO QUE A CURVA SATURA NA PONTA** — sem guarda nova.
///
/// ⚠️ O clamp já existia (é o do `offset`, em `ArcMap::s_at`), e é a resposta certa: *keep
/// length* quando não há curva que chegue é empilhar o excesso no fim.
#[test]
fn a_layout_longer_than_the_curve_piles_up_at_the_end() {
    let x = xs(&row(5, 40.0), true);
    assert!(
        (x[4] - 6.0).abs() < 0.05,
        "o último ponto tem de parar na ponta da curva: {x:?}"
    );
    assert!(
        x.windows(2).all(|w| w[1] >= w[0] - 1e-4),
        "e nada pode andar para trás: {x:?}"
    );
}

/// **`Fit` É O DEFAULT E O CURSO ALCANÇA OS DOIS.**
#[test]
fn the_mode_is_painted_and_fit_is_the_default() {
    let spec = MANIFEST
        .params
        .iter()
        .find(|p| p.name == "mode")
        .expect("o param existe");
    assert_eq!(spec.default, 0.0, "o default é Fit — o nó que shipava");
    let h = PARAM_HINTS
        .iter()
        .find(|h| h.param == "mode")
        .expect("o Mode tem de estar pintado");
    match h.widget {
        ParamWidget::Enum { labels } => assert_eq!(labels.len(), 2),
        _ => panic!("o Mode é um Enum"),
    }
    assert!(
        h.max >= MODE_KEEP_LENGTH as f32,
        "o curso alcança o segundo modo"
    );
}
