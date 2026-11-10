//! T1-18 (B19.1): the clip chain is STRUCTURE, not visibility — on the CPU and in the GPU op-list.
//! Stack (bottom → top): `fundo` opaque blue · `base` green on the left, empty on the right ·
//! `meio` (optional) · `recorte` red everywhere, clipping.

use super::*;
use ph2d_layer_ops::LayerOp;
use ph2d_painter_effects::adjustments::AdjustmentKind;

const AZUL: [u8; 4] = [0, 0, 200, 255];
const VERMELHO: [u8; 4] = [200, 0, 0, 255];

fn pilha(meio: Option<AdjustmentKind>) -> (LayerStack, MapPixelSource, [LayerId; 4]) {
    let (w, h) = (2, 1);
    let mut s = LayerStack::new();
    let fundo = s.add_raster("fundo", w, h).unwrap();
    let base = s.add_raster("base", w, h).unwrap();
    let adj = meio.map_or(base, |k| s.add_adjustment(k).unwrap());
    let recorte = s.add_raster("recorte", w, h).unwrap();
    s.set_clipping(recorte, true);
    let mut src = MapPixelSource::default();
    src.insert(
        fundo,
        LayerImage {
            width: w,
            height: h,
            rgba8: AZUL.repeat(2),
        },
    );
    let mut b = LayerImage::transparent(w, h);
    b.rgba8[0..4].copy_from_slice(&[0, 200, 0, 255]);
    src.insert(base, b);
    src.insert(
        recorte,
        LayerImage {
            width: w,
            height: h,
            rgba8: VERMELHO.repeat(2),
        },
    );
    (s, src, [fundo, base, recorte, adj])
}

fn px(out: &[u8]) -> [[u8; 4]; 2] {
    [out[0..4].try_into().unwrap(), out[4..8].try_into().unwrap()]
}

/// Base hidden ⇒ the clipped layer goes with it; before it clipped to `fundo` (red over all).
#[test]
fn recorte_sobre_base_escondida_some_com_ela() {
    let (mut s, src, [_, base, recorte, _]) = pilha(None);
    s.set_visible(base, false);
    assert_eq!(px(&composite(&s, &src, 2, 1)), [AZUL, AZUL]);
    let (ops, _) = flatten_for_gpu(&s).expect("representable");
    assert!(
        !ops.iter()
            .any(|o| matches!(o, LayerOp::Layer { key, .. } if *key == recorte.0)),
        "the GPU must not draw the clipped layer of a hidden base: {ops:?}"
    );
}

/// Base at opacity 0 is still THE base (the clip reads its pixels' alpha, never its opacity).
#[test]
fn recorte_sobre_base_a_opacidade_zero_recorta_a_ela() {
    let (mut s, src, [_, base, _, _]) = pilha(None);
    s.get_mut(base).unwrap().opacity = 0.0;
    assert_eq!(px(&composite(&s, &src, 2, 1)), [VERMELHO, AZUL]);
    // Control: the same picture at opacity ε (continuity of the rule).
    s.get_mut(base).unwrap().opacity = f32::MIN_POSITIVE;
    assert_eq!(px(&composite(&s, &src, 2, 1)), [VERMELHO, AZUL]);
    s.get_mut(base).unwrap().opacity = 0.0;
    assert!(
        flatten_for_gpu(&s).is_none(),
        "opacity-0 base under a clip stays on the CPU"
    );
}

/// A hidden adjustment breaks the chain exactly like a visible one: the clipped layer draws unclipped.
#[test]
fn ajuste_escondido_quebra_o_elo_como_o_visivel() {
    let (mut s, src, [_, _, recorte, adj]) = pilha(Some(AdjustmentKind::BrightnessContrast));
    let visivel = composite(&s, &src, 2, 1);
    assert_eq!(px(&visivel), [VERMELHO, VERMELHO]);
    s.adjustment_mut(adj).unwrap().visible = false;
    assert_eq!(px(&composite(&s, &src, 2, 1)), [VERMELHO, VERMELHO]);
    let (ops, _) = flatten_for_gpu(&s).expect("representable");
    assert!(
        ops.iter()
            .any(|o| matches!(o, LayerOp::Layer { key, clipping: false, .. } if *key == recorte.0)),
        "after a hidden adjustment the GPU draws the clip unclipped: {ops:?}"
    );
    s.get_mut(adj).unwrap().visible = false;
    s.adjustment_mut(adj).unwrap().visible = true;
    assert_eq!(px(&composite(&s, &src, 2, 1)), [VERMELHO, VERMELHO]);
}
