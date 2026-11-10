//! O gesto do balde (B48.3): dois soltares seguidos, cada um com o seu passo de undo.

use crate::tool::PainterTool;
use crate::tool::paint::PaintMode;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool, Tool,
};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Branco com um quadrado vermelho em `[12,20)²`, no Fill, pincel verde, limiar apertado.
fn tela(size: u32) -> PainterTool {
    let mut src = vec![255u8; (size * size * 4) as usize];
    for y in 12..20 {
        for x in 12..20 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[220, 20, 20, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "fill".to_string(),
    ));
    assert!(matches!(t.paint.paint_mode, PaintMode::Fill));
    t.paint.brush.color = [0.0, 1.0, 0.0];
    t.set_fill_threshold(0.05);
    t
}

fn solta(t: &mut PainterTool, pos: [f32; 2]) {
    t.on_canvas_pointer(cp(pos, PointerPhase::Down));
    t.on_canvas_pointer(cp(pos, PointerPhase::Up));
}

/// **B48.3** — o 2.º soltar com o 1.º pendente (sem Done) fecha o 1.º com o seu passo: o Desfazer chega
/// ao desenho de antes do 1.º, e o Cancelar do 2.º volta ao 1.º assente (não ao 1.º apagado nem a nada).
#[test]
fn um_segundo_balde_com_o_primeiro_pendente_nao_perde_o_antes() {
    let size = 32u32;
    // Done no 2.º, depois dois Desfazer.
    let mut t = tela(size);
    let antes = t.canvas_rgba.to_vec();
    solta(&mut t, [16.0, 16.0]);
    assert!(t.has_active_fill(), "o 1.º fica pendente do modal");
    let com_a = t.canvas_rgba.to_vec();
    assert_ne!(com_a, antes, "o 1.º pintou");
    t.paint.brush.color = [0.0, 0.0, 1.0];
    solta(&mut t, [2.0, 2.0]);
    assert!(t.has_active_fill(), "o 2.º fica pendente do modal");
    t.fill_commit();
    assert!(t.undo_last(), "o 2.º é um passo");
    assert!(
        t.canvas_rgba.as_slice() == com_a.as_slice(),
        "desfazer o 2.º volta ao 1.º"
    );
    assert!(t.undo_last(), "o 1.º é outro passo");
    assert!(
        t.canvas_rgba.as_slice() == antes.as_slice(),
        "desfazer o 1.º volta ao antes"
    );

    // Cancelar o 2.º volta ao 1.º; o Desfazer do 1.º volta ao antes.
    let mut t = tela(size);
    solta(&mut t, [16.0, 16.0]);
    t.paint.brush.color = [0.0, 0.0, 1.0];
    solta(&mut t, [2.0, 2.0]);
    t.fill_cancel();
    assert!(
        t.canvas_rgba.as_slice() == com_a.as_slice(),
        "cancelar o 2.º volta ao 1.º"
    );
    assert!(t.undo_last(), "o 1.º ficou com o seu passo");
    assert!(
        t.canvas_rgba.as_slice() == antes.as_slice(),
        "desfazer o 1.º volta ao antes"
    );
}
