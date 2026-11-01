//! O `Pigment mix` do cartão Mixing no Wet Paint segue a LEI da tinta (E10-B2, F3 §9): só a `Pigmento` o lê
//! (`Grid::pigmento`), logo na `Velha` (a do produto) a fileira não se oferece.

use crate::PainterTool;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_wet_paint::sim::LeiDaTinta;

fn molhado() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; 64 * 64 * 4], 64, 64);
    t.set_paint_media(crate::PaintMedia::WetPaint);
    t
}

/// Mutação que sangra: tirar o `lei == Pigmento` de `offers_pigment_mixing_in` (a fileira volta a aparecer
/// na `Velha`, knob MORTO) ou devolver `false` ao Wet Paint (a `Pigmento` perde o controlo).
#[test]
fn o_pigment_mix_do_wet_paint_so_aparece_na_lei_pigmento() {
    let mut t = molhado();
    assert!(
        !t.brush_settings().pigment_offered,
        "na lei Velha o Pigment mix não tem leitor: a fileira não se oferece"
    );
    t.set_wet_lei_da_tinta(LeiDaTinta::Pigmento);
    assert!(
        t.brush_settings().pigment_offered,
        "na lei Pigmento o pincel do Wet Paint lê o Pigment mix: a fileira aparece"
    );
    t.paint.eraser = true;
    assert!(
        !t.brush_settings().pigment_offered,
        "a borracha da água não pousa tinta: sem fileira"
    );
    t.paint.eraser = false;
    t.set_wet_lei_da_tinta(LeiDaTinta::Velha);
    assert!(
        !t.brush_settings().pigment_offered,
        "de volta à Velha: some"
    );
}
