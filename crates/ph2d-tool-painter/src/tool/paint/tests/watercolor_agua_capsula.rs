//! **O Rewet com o Continuous não lê fora da caixa da água** (achado ao pôr os números do dono como
//! fábrica, 23/10: `attempt to subtract with overflow` em `Dissolucao::parceiro`). A caixa da
//! dissolução só contava o CENTRO de cada dab; os sub-dabs da cápsula depositam atrás dele, e com o
//! Bleed pequeno (raio da água curto) caíam fora — o produto rebentava a meio do traço.

use super::*;
use crate::tool::paint::media::PaintMedia;

#[test]
fn o_rewet_com_o_continuous_nao_le_fora_da_caixa_da_agua() {
    for bleed in [1.0f32, 4.0] {
        let mut t = white_canvas(160, 16.0);
        t.set_paint_media(PaintMedia::Watercolor);
        t.set_brush_spacing(0.6);
        t.set_brush_continuous_spacing(true);
        t.set_brush_edge_spread(bleed);
        t.set_brush_wet_rewet(1.0);
        t.set_brush_wet_charge(0.5);
        let seed = t.paint.brush;
        t.paint.brush_by_mode.fill(seed);
        let antes = t.canvas_rgba.clone();
        // Dois traços: o 2.º dissolve a tinta molhada do 1.º.
        for y in [70.0f32, 80.0] {
            t.on_canvas_pointer(cp([20.0, y], PointerPhase::Down));
            for k in 1..=40 {
                t.on_canvas_pointer(cp([20.0 + 3.0 * k as f32, y], PointerPhase::Move));
                frame(&mut t);
            }
            t.on_canvas_pointer(cp([140.0, y], PointerPhase::Up));
            frame(&mut t);
        }
        assert_ne!(
            t.canvas_rgba, antes,
            "CONTROLO: o traço pintou (Bleed {bleed})"
        );
    }
}
