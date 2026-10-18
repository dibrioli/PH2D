//! ⭐ **A pressão viaja na macro** (ADR-0186, P46 — a caneta de verdade): uma pincelada com uma RAMPA
//! de pressão, gravada e tocada, dá ao bit a tela que se pintou ao vivo, em cada meio. Controlo: a
//! mesma pincelada a `1.0` pinta outra tela, senão o gate não mediria a pressão.

use super::etapas::tela_no_fim;
use super::roteiro::{SPLASH, grava_vivo};
use crate::PaintMedia;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_macro::codec;

const LADO: u32 = 96;

fn diferentes(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

/// `(ao vivo, tocada)` de uma pincelada no meio `m`.
fn telas(pressao: [f32; 2], m: PaintMedia) -> (Vec<u8>, Vec<u8>) {
    let mut p = SPLASH[0];
    p.pressao = pressao;
    let (doc, _, mut vivo) = grava_vivo(LADO, 0, &[p], &[], false, &[m]).expect("grava");
    let ao_vivo = vivo.run_full().0;
    let mut doc = codec::read(&doc).expect("lê-se");
    (ao_vivo, tela_no_fim(&doc.tracks.remove(0)).expect("toca"))
}

#[test]
fn uma_rampa_de_pressao_gravada_toca_a_tela_pintada_ao_vivo_em_cada_meio() {
    for m in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let (vivo, tocada) = telas([0.1, 1.0], m);
        assert_eq!(diferentes(&vivo, &tocada), 0, "{m:?}: tocada ≠ ao vivo");
        let (_, cheia) = telas([1.0, 1.0], m);
        assert!(
            diferentes(&tocada, &cheia) > 0,
            "{m:?}: a pressão não mudou a tela"
        );
    }
}
