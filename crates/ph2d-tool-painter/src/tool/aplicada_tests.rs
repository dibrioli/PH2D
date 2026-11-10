//! B10.5 — a imagem aplicada pela caixa marcada é a inteira ao bit depois de cada tipo de mudança.

use super::*;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

const L: u32 = 96;

fn traco(t: &mut PainterTool, de: [f32; 2], a: [f32; 2]) {
    let p = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(de, PointerPhase::Down));
    t.on_canvas_pointer(p(
        [(de[0] + a[0]) * 0.5, (de[1] + a[1]) * 0.5],
        PointerPhase::Move,
    ));
    t.on_canvas_pointer(p(a, PointerPhase::Move));
    t.on_canvas_pointer(p(a, PointerPhase::Up));
}

fn confere(t: &PainterTool, passo: &str) {
    let inc = t.imagem_aplicada().expect("documento").0;
    let ora = t.imagem_aplicada_inteira().expect("documento").0;
    let dif = inc.iter().zip(&ora).filter(|(a, b)| a != b).count();
    assert_eq!(
        (inc.len(), dif),
        (ora.len(), 0),
        "{passo}: {dif} bytes ≠ da inteira"
    );
}

#[test]
fn incremental_igual_ao_inteiro_ao_bit() {
    for meio in [PaintMedia::Impasto, PaintMedia::Digital] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (L * L * 4) as usize], L, L);
        t.set_paint_media(meio);
        t.set_brush_size_px(9.0);
        t.set_substrate_depth(0.8);
        confere(&t, "1.ª");
        traco(&mut t, [10.0, 10.0], [40.0, 30.0]);
        confere(&t, "traço");
        traco(&mut t, [60.0, 70.0], [85.0, 50.0]);
        confere(&t, "2.º traço");
        assert!(t.undo_last(), "controlo: há o que desfazer");
        confere(&t, "Ctrl+Z");
        let b = t
            .add_raster_layer_with_pixels("B", vec![90u8; (L * L * 4) as usize])
            .expect("B");
        confere(&t, "camada nova");
        traco(&mut t, [20.0, 80.0], [70.0, 85.0]);
        confere(&t, "traço na B");
        t.set_layer_opacity(b, 0.5);
        confere(&t, "opacidade");
        t.set_layer_visible(b, false);
        confere(&t, "visibilidade");
        t.set_layer_visible(b, true);
        t.move_layer_down(b);
        confere(&t, "ordem");
        t.set_impasto_light_angle(1.1);
        confere(&t, "luz");
        t.set_substrate_depth(0.2);
        confere(&t, "relevo do papel");
        t.set_paper_color_rgb8(230, 200, 150);
        confere(&t, "cor do papel");
        traco(&mut t, [5.0, 5.0], [90.0, 90.0]);
        confere(&t, "traço sobre o papel");
        t.set_source(vec![200u8; (64 * 48 * 4) as usize], 64, 48);
        confere(&t, "set_source (outro tamanho)");
        traco(&mut t, [3.0, 3.0], [60.0, 40.0]);
        confere(&t, "traço depois do set_source");
    }
}

/// A 2.ª leitura depois de um traço curto recompõe só a moldura dele: o contador do gate é a caixa.
#[test]
fn a_leitura_depois_de_um_traco_e_so_a_caixa() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (L * L * 4) as usize], L, L);
    t.set_paint_media(PaintMedia::Impasto);
    let _ = t.imagem_aplicada();
    traco(&mut t, [40.0, 40.0], [44.0, 42.0]);
    let r = t.aplicada.sujo_agora().expect("o traço marcou a caixa");
    assert!(
        r.w * r.h < L * L / 4,
        "a caixa {r:?} não é a de um traço curto"
    );
    confere(&t, "traço curto");
}
