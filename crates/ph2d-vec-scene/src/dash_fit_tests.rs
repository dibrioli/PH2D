//! Gates do ajuste de tracejado.
//!
//! ⚠️ O oráculo é **quantos períodos cabem no comprimento**, e ele é exacto: se o padrão
//! ajustado divide o contorno num número inteiro de vezes, não há emenda. Um gate que
//! comparasse os números contra uma tabela escrita à mão estaria a testar a tabela.

use super::{dash_lengths_for, fit, longest_contour};
use crate::{Rgba8, ShapeKind, StrokeSpec, cook};

/// Quantos períodos do padrão `d` cabem em `total` — inteiro se o ajuste funcionou.
fn periods(d: [f64; 2], total: f64) -> f64 {
    total / (d[0] + d[1])
}

/// **NUM CONTORNO FECHADO O PADRÃO FECHA EXACTAMENTE** — a emenda deixa de existir.
#[test]
fn a_closed_contour_gets_a_whole_number_of_periods() {
    // Um perímetro que NÃO é múltiplo do período pedido — é o caso que produz a emenda.
    for total in [10.0, 13.37, 100.0, 7.5, 1.234] {
        let d = fit([1.0, 0.6], total, true);
        let n = periods(d, total);
        assert!(
            (n - n.round()).abs() < 1e-9,
            "total {total}: cabem {n} periodos, e tem de ser inteiro"
        );
        assert!(n >= 1.0, "pelo menos um periodo, senao nao ha' tracejado");
    }
}

/// **E A RAZÃO TRAÇO:VÃO SOBREVIVE** — o ajuste estica os dois juntos.
///
/// ⚠️ É a metade que impede a cura errada: fechar a conta encolhendo só o vão mudaria o
/// carácter do tracejado (o que o artista autorou foi a proporção, não os milímetros).
#[test]
fn the_dash_to_gap_ratio_survives_the_fit() {
    let raw = [1.0, 0.6];
    for total in [10.0, 13.37, 100.0] {
        let d = fit(raw, total, true);
        assert!(
            ((d[0] / d[1]) - (raw[0] / raw[1])).abs() < 1e-9,
            "a razao mudou em {total}: {d:?}"
        );
    }
}

/// **O AJUSTE É PEQUENO, E O TETO É DERIVADO** — no máximo meio período de erro, ou seja
/// `k ∈ [1 − 1/(2n), 1 + 1/(2n)]`.
///
/// ⚠️ Sem este gate, "ajustar" poderia ser "reescrever": um padrão de 1,0 que saísse a 4,0
/// fecharia a conta e destruiria o desenho.
#[test]
fn the_fit_never_moves_the_period_by_more_than_half_of_one() {
    let raw = [1.0, 0.6];
    let period = raw[0] + raw[1];
    for i in 1..400 {
        let total = f64::from(i) * 0.37;
        let d = fit(raw, total, true);
        let k = (d[0] + d[1]) / period;
        let n = periods(d, total).round().max(1.0);
        let bound = 1.0 / (2.0 * n - 1.0).max(1.0);
        assert!(
            (k - 1.0).abs() <= bound + 1e-9,
            "total {total}: k={k} passou o teto derivado {bound} (n={n})"
        );
    }
}

/// **NUM CONTORNO ABERTO O CAMINHO COMEÇA E ACABA COM TRAÇO INTEIRO.**
///
/// Ali não há emenda — há duas PONTAS, e o defeito é uma delas acabar a meio de um vão.
#[test]
fn an_open_contour_starts_and_ends_on_a_dash() {
    for total in [10.0, 13.37, 4.2] {
        let d = fit([1.0, 0.6], total, false);
        // `n` períodos + um traço = o total.
        let n = (total - d[0]) / (d[0] + d[1]);
        assert!(
            (n - n.round()).abs() < 1e-9,
            "total {total}: sobram {n} periodos alem do ultimo traco"
        );
    }
}

/// **UM CONTORNO DEGENERADO NÃO É AJUSTADO** — devolver o pedido é melhor que dividir por
/// quase-zero.
#[test]
fn a_degenerate_length_leaves_the_pattern_alone() {
    assert_eq!(fit([1.0, 0.6], 0.0, true), [1.0, 0.6]);
    assert_eq!(fit([1.0, 0.6], -1.0, true), [1.0, 0.6]);
    assert_eq!(fit([0.0, 0.0], 10.0, true), [0.0, 0.0]);
}

/// **A PORTA MEDE O CAMINHO DE VERDADE** — e um caminho sem tracejado continua sem.
#[test]
fn the_door_measures_the_path_and_passes_the_absence_through() {
    let rect = cook(
        ShapeKind::RoundRect,
        [-1.0, -0.6],
        [1.0, 0.6],
        &[0.2, 0.0, 0.0, 0.0, 0.0],
    );
    let (total, closed) = longest_contour(&rect).expect("o retangulo tem contorno");
    assert!(total > 4.0, "o perimetro de um 2x1,2 passa de 4: {total}");
    assert!(closed, "um round-rect e' fechado");

    let mut solid = StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 0.1);
    assert_eq!(dash_lengths_for(&rect, &solid), None, "sem tracejado, nada");

    solid.dash = Some((2.5, 2.0));
    let d = dash_lengths_for(&rect, &solid).expect("com tracejado, o ajuste");
    let n = periods(d, total);
    assert!(
        (n - n.round()).abs() < 1e-9,
        "a porta tem de fechar a conta: {n} periodos"
    );
}
