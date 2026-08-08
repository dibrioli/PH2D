//! **Os modificadores que o `CanvasPointer` não carrega.**
//!
//! A superfície do ponteiro de canvas é congelada e compartilhada por muitos leitores, então Shift / Ctrl
//! / Alt viajam **fora de banda**: o shell os amostra do estado vivo do teclado e os empurra ao tool logo
//! antes da entrega. Eram quatro chamadas soltas no meio do `deliver_canvas_pointer`; juntá-las dá um
//! lugar só para a pergunta *"o que cada tecla significa no canvas do Painter?"* — e é ele que impede a
//! quinta de nascer noutro ponto do arquivo, longe das outras quatro.

use ph2d_tool_painter::PainterTool;

/// Empurra os três modificadores para o tool. `ctrl` já chega com o Cmd do macOS dobrado dentro dele — a
/// convenção do gizmo de sprite, e dobrá-lo aqui seria uma segunda resposta a *"o que conta como Ctrl?"*.
pub(crate) fn forward(painter: &mut PainterTool, shift: bool, ctrl: bool, alt: bool) {
    // Alt: o método Line restringe a 45° enquanto ele está preso (o Alt-drag do Blender).
    painter.set_line_constrain(alt);
    // Shift / Ctrl no gizmo de SELEÇÃO, com a lei do gizmo de sprite palavra por palavra
    // (`ph2d_editor_core::GizmoModifiers`): Shift trava a razão de aspecto numa quina, Ctrl/Cmd ancora o
    // escalonamento no CENTRO — sendo o default a quina oposta.
    painter.set_gizmo_modifiers(shift, ctrl, alt);
    painter.set_uniform_scale(shift); // Shift = escala uniforme do Stencil (gizmo de sprite)
    painter.set_line_snap(shift); // Shift = snap de direção de 15° no editor de polilinha
}
