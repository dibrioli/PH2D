//! **As teclas do pincel do Painter** — `[`/`]` mudam o tamanho, `E` alterna a borracha.
//!
//! ⚠️ Saíram do `painter_canvas_input.rs` (2026-09-24) por RESPONSABILIDADE: aquele ficheiro é o
//! do PONTEIRO de canvas, e estas duas são teclado. O corte foi forçado pelo tecto de LOC (HR-18)
//! quando o ponteiro ganhou o ramo do Painter sobre a peça 3D, e é melhor do que o ficheiro era.
//! ⚠️ A ORDEM destas teclas na cadeia do teclado é gateada pela CHAMADA
//! (`the_preview_owns_the_pointer_and_the_undo`), não por este ficheiro.

use ph2d_tool_painter::PainterTool;

use crate::App;

impl App {
    /// Nudge the active Painter brush radius — `[` (`dir < 0`) shrinks, `]`
    /// (`dir >= 0`) grows (Blender/Photoshop convention). Returns `true` when
    /// consumed (the active tool IS the Painter), so the bracket key doesn't fall
    /// through to other handlers.
    pub(crate) fn painter_nudge_brush_size(&mut self, dir: i32) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(painter) = tool.as_any_mut().downcast_mut::<PainterTool>() else {
            return false;
        };
        painter.nudge_brush_size(dir);
        true
    }

    /// Toggle the active Painter brush's eraser mode (`E`). Returns `true` when
    /// consumed (the active tool IS the Painter), so `E` falls through otherwise.
    pub(crate) fn painter_toggle_eraser(&mut self) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(painter) = tool.as_any_mut().downcast_mut::<PainterTool>() else {
            return false;
        };
        painter.toggle_brush_eraser();
        true
    }
}
