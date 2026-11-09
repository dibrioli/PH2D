//! **A cadeia de teclas do Painter que APAGA** — o alvo mais ESPECÍFICO vence: nó da curva, depois a
//! FIGURA em mãos, depois o ponto do falloff. Corre ANTES da Hierarquia (cujo `Delete` apaga a
//! entidade). Os verbos moram junto da ordem que os chama.

use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

impl crate::App {
    /// Roda a cadeia; `true` quando algum degrau consumiu a tecla.
    pub(crate) fn painter_delete_chain(
        &mut self,
        state: ElementState,
        physical_key: PhysicalKey,
    ) -> bool {
        // Curve: drops the selected control point (kept ≥ 2).
        if state == ElementState::Pressed
            && matches!(
                physical_key,
                PhysicalKey::Code(KeyCode::Delete | KeyCode::Backspace)
            )
            && self.painter_curve_delete_selected_point()
        {
            return true;
        }

        // A FIGURA em mãos — antes da Hierarquia, que levaria o sprite inteiro.
        if state == ElementState::Pressed
            && matches!(
                physical_key,
                PhysicalKey::Code(KeyCode::Delete | KeyCode::Backspace)
            )
            && self.painter_delete_active_shape()
        {
            return true;
        }

        // Falloff curve: drops the selected point.
        if state == ElementState::Pressed
            && matches!(
                physical_key,
                PhysicalKey::Code(KeyCode::Delete | KeyCode::Backspace)
            )
            && self.painter_delete_selected_falloff_point()
        {
            return true;
        }
        false
    }

    /// Remove the selected Curve control point; `true` (consumes) iff one was removed.
    pub(crate) fn painter_curve_delete_selected_point(&mut self) -> bool {
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        // Either curve owner: the stroke Shape curve or the selection Convert-to-Curve editor.
        painter.curve_delete_selected() || painter.selection_curve_delete_selected_point()
    }

    /// **A peça COLADA que flutua: Enter aplica, Esc descarta** (só teclas: um clique não decide).
    /// `true` quando havia peça viva.
    pub(crate) fn painter_paste_patch_key(
        &mut self,
        state: ElementState,
        physical_key: PhysicalKey,
        repeat: bool,
    ) -> bool {
        if state != ElementState::Pressed || repeat {
            return false;
        }
        let PhysicalKey::Code(code) = physical_key else {
            return false;
        };
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        match code {
            KeyCode::Enter | KeyCode::NumpadEnter => painter.paste_commit(),
            KeyCode::Escape => painter.paste_cancel(),
            _ => false,
        }
    }

    /// **Os atalhos de área de transferência da SELEÇÃO do Painter** — Ctrl+X / C / V, mais Ctrl+A
    /// (tudo), Ctrl+D (nada) e Ctrl+Shift+I (inverter). Gateado no MODO de seleção (Ctrl+A/C/V têm
    /// outros donos fora dele); corre DEPOIS da cadeia do Delete.
    pub(crate) fn painter_selection_clipboard_chain(
        &mut self,
        state: ElementState,
        physical_key: PhysicalKey,
    ) -> bool {
        if state != ElementState::Pressed {
            return false;
        }
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        if !ctrl {
            return false;
        }
        let shift = self.modifiers.shift_key();
        let PhysicalKey::Code(code) = physical_key else {
            return false;
        };
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        if !painter.is_selection_mode() {
            return false;
        }
        match code {
            KeyCode::KeyX => painter.selection_cut(),
            KeyCode::KeyC => painter.selection_copy(),
            KeyCode::KeyV => painter.selection_paste(),
            KeyCode::KeyA => painter.selection_select_all(),
            KeyCode::KeyD => painter.clear_selection(),
            KeyCode::KeyI if shift => painter.invert_selection(),
            _ => return false,
        }
        true
    }

    /// **Delete apaga a FIGURA em mãos**, depois do delete de âncora. `true` quando havia uma.
    pub(crate) fn painter_delete_active_shape(&mut self) -> bool {
        self.painter_tool_mut()
            .is_some_and(ph2d_tool_painter::PainterTool::delete_active_shape)
    }
}
