//! ⭐⭐⭐ **QUE MODO É DONO DO TECLADO NESTE QUADRO** — irmão de [`super::keyboard`] pelo teto de
//! 600 LOC, e o corte é por ASSUNTO: ali *o que cada tecla faz*, aqui *se ela chega a ser
//! perguntada*.
//!
//! *Um modo em curso é dono da entrada dele*, DEPOIS do `input.apply_event` do irmão (a acção
//! continua a ser alimentada) e ANTES de todo atalho; gate sobre a ordem em `vec_morph_edit_tests`.

use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

impl crate::App {
    /// **Um modo modal reclama esta tecla?** `true` ⇒ o despacho pára aqui.
    ///
    /// A janela da recuperação (`ph2d_recovery::card`) come tudo; o `Esc` = `Later`/`Cancel`.
    ///
    /// O preview da máquina de Morph (plano 32 W9) come as teclas da acção; o `Esc` PEDE a saída
    /// (`leave`), e os acordes `Ctrl`/`Super` passam (trancar o gravar seria perder trabalho).
    pub(crate) fn modal_owns_the_keyboard(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        repeat: bool,
    ) -> bool {
        if self.recovery.is_modal() {
            if state == ElementState::Pressed && physical_key == PhysicalKey::Code(KeyCode::Escape)
            {
                ph2d_recovery::card::key_escape(&mut self.recovery);
            }
            return true;
        }
        if !self.morph_preview || self.modifiers.control_key() || self.modifiers.super_key() {
            return false;
        }
        if self.surface_owns_the_keyboard() {
            return false;
        }
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::Escape))
        {
            self.morph_preview_leave = true;
        }
        true
    }
}
