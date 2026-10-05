//! O Gaussiano por cima de uma camada com relevo borra o relevo que a luz do impasto lê — pela
//! ferramenta, com a pincelada de verdade.

use super::*;
use ph2d_painter_effects::adjustments::{
    AdjustWindow, AdjustmentKind, AdjustmentParams, GaussianBlurParams, separable_blur_scalar,
};

const LADO: u32 = 64;

fn pincelada(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([14.0, 32.0], PointerPhase::Down));
    for k in 1..=12 {
        t.on_canvas_pointer(cp([14.0 + 3.0 * k as f32, 32.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([50.0, 32.0], PointerPhase::Up));
}

fn borra(v: &[f32], raio: f32) -> Vec<f32> {
    let mut b = v.to_vec();
    separable_blur_scalar(raio, &mut b, AdjustWindow::full(LADO, LADO));
    b
}

/// ⭐⭐ **O Gaussiano borra o relevo E o corpo que a luz lê**, como borra a cor; escondê-lo devolve o
/// relevo AO BIT; e a luz desenha outra coisa. CONTROLO: o desfoque mexe num pedaço real da tela.
#[test]
fn o_gaussiano_borra_o_relevo_que_a_luz_le_e_esconde_lo_devolve() {
    let mut t = impasto_canvas(LADO);
    pincelada(&mut t);
    let (r0, c0) = t.composed_relief_plane();
    let luz0 = lit(&mut t);
    let g = t
        .add_adjustment_layer(AdjustmentKind::GaussianBlur)
        .expect("o Gaussiano");
    t.layers.adjustment_mut(g).expect("g").params =
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 4.0 });
    t.invalidate_composite();
    let (r1, c1) = t.composed_relief_plane();
    let (br, bc) = (borra(&r0, 4.0), borra(&c0, 4.0));
    let mexeu = (0..r0.len())
        .filter(|&i| (r1[i] - r0[i]).abs() > 1e-3)
        .count();
    assert!(
        mexeu > 100,
        "CONTROLO: o desfoque mexeu em {mexeu} píxeis do relevo"
    );
    for i in 0..r0.len() {
        assert!(
            (r1[i] - br[i]).abs() < 1e-4,
            "píxel {i}: o relevo {} não é o borrado {}",
            r1[i],
            br[i]
        );
        assert!(
            (c1[i] - bc[i]).abs() < 1e-5,
            "píxel {i}: o corpo não foi borrado"
        );
    }
    let luz1 = lit(&mut t);
    let difere = luz0
        .chunks(4)
        .zip(luz1.chunks(4))
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        difere > 50,
        "a luz não viu o relevo borrado ({difere} píxeis)"
    );
    t.set_layer_visible(g, false);
    let (r2, _) = t.composed_relief_plane();
    assert_eq!(
        r2.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        r0.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        "escondido, o relevo não voltou ao bit"
    );
}

/// O Brilho é uma operação de TOM: o relevo fica AO BIT.
#[test]
fn o_brilho_nao_mexe_no_relevo() {
    let mut t = impasto_canvas(LADO);
    pincelada(&mut t);
    let (r0, _) = t.composed_relief_plane();
    t.add_adjustment_layer(AdjustmentKind::Bloom)
        .expect("o Brilho");
    t.invalidate_composite();
    let (r1, _) = t.composed_relief_plane();
    assert_eq!(
        r1.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        r0.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
    );
}
