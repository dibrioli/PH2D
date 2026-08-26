//! ⭐⭐⭐ **QUE MODO É DONO DO TECLADO NESTE QUADRO** — irmão de [`super::keyboard`] pelo teto de
//! 600 LOC, e o corte é por ASSUNTO: ali *o que cada tecla faz*, aqui *se ela chega a ser
//! perguntada*.
//!
//! # A lei
//!
//! *Um modo em curso é dono da entrada dele.* É a mesma que o `ui_preview` aplica ao **rato** (o
//! `Down`/`Up` primário é dele enquanto ele corre) e que o palette do Motion aplica às letras.
//!
//! ⚠️ **A POSIÇÃO na cadeia é metade do desenho, e são duas coisas de uma vez:**
//!
//! - **depois** do `input.apply_event` do irmão ⇒ a acção continua a ser alimentada, e é ela que o
//!   modo lê. Barrar antes mataria o próprio modo: ele ficaria **inerte com o teclado tomado**, que
//!   é o pior dos dois mundos.
//! - **antes** de todo atalho ⇒ nenhum consumidor do editor vê a tecla.
//!
//! Há gate sobre a ordem (`vec_morph_edit_tests`, a comparar os dois `find` no fonte).

use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

impl crate::App {
    /// **Um modo modal reclama esta tecla?** `true` ⇒ o despacho pára aqui.
    ///
    /// # A pré-visualização da máquina de Morph (plano 32 W9)
    ///
    /// Enio, 2026-08-25: *"precisamos de um modo preview (com botão) como o de states de animação
    /// pois senão temos conflitos de atalhos (como setas do teclado movendo as formas)"*.
    ///
    /// A condição de uma transição é uma **acção do Input Map**, isto é, uma tecla; sem um modo,
    /// carregar em `Z` morfa a forma **e** faz o que o `Z` faz no editor — os dois, sem nada na tela
    /// a explicar.
    ///
    /// ⚠️ **O Esc é a excepção, e tem de ser:** este modo come exactamente as teclas com que o
    /// artista tentaria escapar dele. Ele **PEDE** a saída (o `leave`) em vez de a executar, como o
    /// irmão das poses — executar aqui exigiria o mundo, que mora no `gfx`.
    ///
    /// ⛔ **Os acordes com `Ctrl`/`Super` PASSAM, e a assimetria é deliberada:** `Ctrl+S`,
    /// `Ctrl+Z` e `Ctrl+Shift+Z` não são o dedo do jogador (a guarda de acorde do irmão já os
    /// exclui da acção), e trancar o **gravar** dentro de um modo de olhar seria trocar um conflito
    /// de atalhos por uma perda de trabalho.
    pub(crate) fn modal_owns_the_keyboard(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        repeat: bool,
    ) -> bool {
        if !self.morph_preview || self.modifiers.control_key() || self.modifiers.super_key() {
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
