//! Missão 4K onda 3 (B28.4): a rota das pegadas locais do Wet blend não guarda as máscaras da TELA inteira (33 MB cada
//! a 4K) que só a rota em série lê.

use super::super::super::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool as _, CanvasPointer, PointerPhase, RasterEditTool as _,
};
use ph2d_painter_brush::BrushSpec;
use ph2d_painter_brush::mistura_opaca::{ModoMisturaOpaca, MisturaOpaca};

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

#[test]
fn as_pegadas_locais_nao_guardam_a_mascara_da_tela() {
    const N: u32 = 256;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    t.paint.brush = BrushSpec {
        radius_px: 20.0,
        color: [0.1, 0.2, 0.85],
        mistura_opaca: MisturaOpaca {
            modo: ModoMisturaOpaca::WetBlend,
            ..MisturaOpaca::DE_FABRICA
        },
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    t.on_canvas_pointer(ponto([40.0, 128.0], PointerPhase::Down));
    for k in 1..40 {
        t.on_canvas_pointer(ponto([40.0 + 4.0 * k as f32, 128.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(ponto([200.0, 128.0], PointerPhase::Up));
    let m = &t.paint.mistura_opaca;
    assert!(
        m.res.iter().any(|r| r.cor().is_some()),
        "a mistura não correu"
    );
    assert_eq!(
        (m.pegada.capacity(), m.pegada_lisa.capacity()),
        (0, 0),
        "a rota das pegadas locais guardou as máscaras da tela inteira"
    );
}
