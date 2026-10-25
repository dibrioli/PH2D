//! ⭐ **A Hardness à vista e o preview que diz a verdade** (dono 24/10: o Glaze Round de fábrica, dureza 0,8, pintava
//! um disco chapado enquanto o preview mostrava o Smooth inteiro — a dureza não tinha linha no painel e o preview da
//! silhueta lia só a curva).

use crate::tool::PainterTool;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::Falloff;

/// O preview da silhueta (`brush_dab_weight_at`) é ao bit o peso que o dab usa (`BrushSpec::falloff_weight`), em
/// cada dureza e curva; o editor da curva (`brush_falloff_weight_at`) continua a curva sozinha.
#[test]
fn o_preview_da_silhueta_e_o_peso_do_dab() {
    let mut t = PainterTool::default();
    t.set_paint_media(crate::PaintMedia::Digital);
    for falloff in [Falloff::Smooth, Falloff::Sharp, Falloff::Linear] {
        t.set_brush_falloff(falloff.to_u8());
        for h in [0.0f32, 0.5, 0.8, 1.0] {
            t.set_brush_hardness(h);
            let s = t.brush_settings();
            for k in 0..=40 {
                let r = k as f32 / 40.0;
                let (preview, dab) = (
                    super::brush_dab_weight_at(&s, r),
                    t.paint.brush.falloff_weight(r),
                );
                assert_eq!(
                    preview.to_bits(),
                    dab.to_bits(),
                    "{falloff:?} dureza {h} a {r}: preview {preview} · dab {dab}"
                );
            }
        }
    }
    t.set_brush_falloff(Falloff::Smooth.to_u8());
    t.set_brush_hardness(0.8);
    let s = t.brush_settings();
    assert!(
        super::brush_falloff_weight_at(&s, 0.5) < 0.9,
        "o editor da curva passou a mostrar a dureza"
    );
    assert_eq!(
        super::brush_dab_weight_at(&s, 0.5),
        1.0,
        "controlo: a dureza 0,8 enche a metade de dentro"
    );
}

/// A linha do painel chega ao PINCEL (o leitor decide: `falloff_weight` lê `hardness`), presa a `0..1`.
#[test]
fn a_linha_da_hardness_chega_ao_pincel() {
    let mut t = PainterTool::default();
    let id = crate::ids::PAINTER_BRUSH_HARDNESS;
    t.handle_panel_event(PanelEvent::SetValue(id, 0.35));
    assert_eq!(t.paint.brush.hardness, 0.35);
    assert_eq!(t.brush_settings().hardness, 0.35);
    t.handle_panel_event(PanelEvent::SetValue(id, 7.0));
    assert_eq!(t.paint.brush.hardness, 1.0);
}
