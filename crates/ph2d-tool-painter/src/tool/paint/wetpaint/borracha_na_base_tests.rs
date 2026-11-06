//! A Borracha na base marca o seu próprio sujo: o traço do motor pode não tocar em célula nenhuma.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, Tool};
use ph2d_painter_brush::Dab;

const SIZE: usize = 128;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 0.8,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn alfa_em(px: &[u8], x: usize, y: usize) -> u8 {
    px[(y * SIZE + x) * 4 + 3]
}

#[test]
fn apagar_a_base_chega_ao_ecra_sem_o_motor_marcar_sujo() {
    let mut t = PainterTool::default();
    t.set_source(vec![40u8; SIZE * SIZE * 4], SIZE as u32, SIZE as u32);
    t.on_tick(1000.0 / 60.0);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_BRUSH_MEDIA,
        PaintMedia::WetPaint.to_u8().to_string(),
    ));
    t.on_tick(1000.0 / 60.0);
    t.on_canvas_pointer(cp([20.0, 100.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Up));
    for _ in 0..4 {
        t.on_tick(1000.0 / 60.0);
        let _ = t.take_preview_arc();
    }
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "eraser".to_string(),
    ));
    t.paint.brush.strength = 1.0;
    t.paint.brush.hardness = 1.0;
    {
        let s = t.paint.wetpaint.session.as_mut().expect("a água viva");
        s.bring_home();
        let _ = s.engine.take_dirty();
    }
    let dab = Dab {
        center: [64.0, 40.0],
        radius_px: 10.0,
        coverage: 1.0,
        color: [0.0; 3],
        color_shift: [0.0; 3],
        rotation: [1.0, 0.0],
        dir: [1.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 10.0,
        step: [0.0, 0.0],
        tilt: None,
        ante: None,
    };
    t.apaga_a_base_sob_os_dabs(&[dab]);
    {
        let s = t.paint.wetpaint.session.as_ref().expect("a água viva");
        assert!(alfa_em(&s.base, 64, 40) < 40, "a base foi apagada");
    }
    t.wetpaint_composite();
    let (ecra, _, _) = t.take_preview_arc().expect("o ecrã mudou");
    assert!(alfa_em(&ecra, 64, 40) < 40, "o ecrã mostra o apagado");
    assert_eq!(alfa_em(&ecra, 20, 40), 40, "fora do círculo a tinta fica");
}
