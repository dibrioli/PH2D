//! B35.8: o duplo clique no Strength de qualquer camada volta à força com que o `+` a criou, nunca ao 0 da cauda.

use super::*;

/// Em todas as posições (a 4.ª e a 5.ª incluídas), a fábrica do Strength é a da camada nova.
#[test]
fn o_duplo_clique_no_strength_de_toda_camada_volta_a_camada_nova() {
    let b = paint_brush::FALLBACK_BRUSH;
    let mut store = WidgetStore::default();
    let mut erradas = Vec::new();
    for pos in 0..ph2d_tool_painter::N_COMPOSITE_LAYERS {
        arma_camada(&mut store, &b, &b, pos);
        let dc = store
            .card_field(pids::PAINTER_BRUSH_COMPOSITE_STRENGTH_CHIP[pos])
            .and_then(|s| s.default);
        if dc != Some(f64::from(ph2d_tool_painter::COMPOSITE_NEW_LAYER_STRENGTH)) {
            erradas.push((pos, dc));
        }
    }
    assert!(erradas.is_empty(), "(posição, fábrica): {erradas:?}");
}
