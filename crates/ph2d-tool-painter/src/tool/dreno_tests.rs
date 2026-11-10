//! B10.6 — a porta genérica `current_preview` mostra o MESMO que o dreno do editor (`take_preview_arc`).

use super::*;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

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

fn novo(meio: PaintMedia) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (L * L * 4) as usize], L, L);
    t.set_paint_media(meio);
    t.set_brush_size_px(9.0);
    t.set_substrate_depth(0.8);
    t
}

/// O editor e a porta genérica, em dois tools gémeos, devolvem os mesmos bytes (ou os dois nada).
fn confere(ed: &mut PainterTool, porta: &mut PainterTool, passo: &str) {
    let a = ed.take_preview_arc().map(|(p, w, h)| (p.to_vec(), w, h));
    let b = porta.current_preview().map(|(p, w, h)| (p.to_vec(), w, h));
    let forma = |x: &Option<(Vec<u8>, u32, u32)>| x.as_ref().map(|x| (x.0.len(), x.1, x.2));
    assert_eq!(forma(&a), forma(&b), "{passo}: forma ≠");
    let dif = a.zip(b).map_or(0, |(x, y)| {
        x.0.iter().zip(&y.0).filter(|(p, q)| p != q).count()
    });
    assert_eq!(
        dif, 0,
        "{passo}: a porta genérica diverge do dreno do editor em {dif} bytes"
    );
}

#[test]
fn a_porta_generica_mostra_o_mesmo_que_o_dreno_do_editor() {
    use PaintMedia::{Digital, Impasto, Watercolor, WetPaint};
    for meio in [Impasto, Digital, Watercolor, WetPaint] {
        let (mut ed, mut porta) = (novo(meio), novo(meio));
        let mut ambos = |f: &dyn Fn(&mut PainterTool), passo: &str| {
            f(&mut ed);
            f(&mut porta);
            confere(&mut ed, &mut porta, &format!("{meio:?} {passo}"));
        };
        ambos(&|_| {}, "1.ª");
        ambos(&|t| traco(t, [10.0, 10.0], [40.0, 30.0]), "traço");
        ambos(&|t| traco(t, [60.0, 70.0], [85.0, 50.0]), "2.º traço");
        ambos(
            &|t| {
                assert!(t.undo_last(), "controlo: há o que desfazer");
            },
            "Ctrl+Z",
        );
        ambos(
            &|t| {
                t.add_raster_layer_with_pixels("B", vec![90u8; (L * L * 4) as usize])
                    .expect("B");
            },
            "camada nova",
        );
        ambos(&|t| traco(t, [20.0, 80.0], [70.0, 85.0]), "traço na B");
        ambos(&|t| t.set_impasto_light_angle(1.1), "luz");
        ambos(&|t| t.set_paper_color_rgb8(230, 200, 150), "cor do papel");
        ambos(&|_| {}, "parado");
    }
}
