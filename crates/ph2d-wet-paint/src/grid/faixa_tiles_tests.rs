//! A partição da faixa por tiles: com a máscara cheia é o intervalo de um troço (o oráculo); com
//! máscara parcial, os troços são a interseção exata.
use super::*;

/// O oráculo: a faixa de UM intervalo, verbatim do `span_x_of` antes de B13.2.
fn span_x_oraculo(
    row_lo: &[i32],
    row_hi: &[i32],
    enabled: bool,
    bx0: i32,
    bx1: i32,
    y: i32,
) -> (i32, i32) {
    if !enabled {
        return (bx0, bx1);
    }
    let r = y as usize;
    (bx0.max(row_lo[r]), bx1.min(row_hi[r]))
}

fn celulas(runs: impl Iterator<Item = (i32, i32)>) -> Vec<i32> {
    runs.flat_map(|(a, b)| a..=b).collect()
}

#[test]
fn os_troços_por_tiles_sao_a_intersecao_da_faixa_com_a_mascara() {
    let (lo, hi) = span_x_oraculo(&[37], &[300], true, 1, 4000, 0);
    let todas: Vec<i32> = (lo..=hi).collect();
    assert_eq!(celulas(span_runs(None, lo, hi)), todas);
    assert_eq!(celulas(span_runs(Some(&[u64::MAX]), lo, hi)), todas);
    let mascara = [0b1_0010u64]; // tiles 1 e 4
    let esperado: Vec<i32> = todas
        .iter()
        .copied()
        .filter(|x| matches!(x >> 6, 1 | 4))
        .collect();
    assert_eq!(celulas(span_runs(Some(&mascara), lo, hi)), esperado);
    assert_eq!(celulas(span_runs(Some(&[0]), lo, hi)), Vec::<i32>::new());
    assert_eq!(
        celulas(span_runs(Some(&[u64::MAX]), 5, 4)),
        Vec::<i32>::new()
    );
}
