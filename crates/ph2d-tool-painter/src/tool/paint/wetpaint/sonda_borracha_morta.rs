//! **A Borracha na tinta morta** ([plano](../../../../../../../docs/Painter/TINTA_VIVA_2026-11-06.md), item 4): as fotos e os
//! números de cada candidata contra a `viva`, a referência.
//! `PH2D_SONDA_DIR=<pasta> cargo test --profile smoke -p ph2d-tool-painter sonda_borracha_morta -- --ignored --nocapture`

use super::agua_sobre_oleo_tests::{com_oleo, grava, mostrada, relato};
use super::borracha_de_fabrica_tests::{apaga, diferentes};
use super::*;

fn soma_delta(a: &[u8], b: &[u8]) -> u64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| u64::from(x.abs_diff(*y)))
        .sum()
}

/// A tinta morta de verdade: a imagem fica, a sessão e as adormecidas acabam.
pub(super) fn morta() -> PainterTool {
    let mut t = com_oleo();
    t.wetpaint_end_session();
    t.wet_esquece_as_adormecidas();
    t
}

#[test]
#[ignore = "sonda: corre por nome com --ignored"]
fn sonda_borracha_morta() {
    let antes = mostrada(&mut com_oleo());
    grava("0_antes", &antes);
    let mut viva = com_oleo();
    viva.set_paint_tool_mode("eraser");
    let v = apaga(&mut viva);
    grava("1_viva", &v);
    grava("1_viva2", &apaga(&mut viva));
    grava("1_viva3", &apaga(&mut viva));
    let mut m = morta();
    m.set_paint_tool_mode("eraser");
    let r = apaga(&mut m);
    grava("3_imitada", &r);
    grava("3_imitada2", &apaga(&mut m));
    grava("3_imitada3", &apaga(&mut m));
    for (nome, p) in [("viva", &v), ("imitada", &r)] {
        relato(format!(
            "{nome}: {} px mudados, soma |Δ| {}, contra a viva {} px, soma |Δ| {}",
            diferentes(&antes, p),
            soma_delta(&antes, p),
            diferentes(&v, p),
            soma_delta(&v, p),
        ));
    }
}
