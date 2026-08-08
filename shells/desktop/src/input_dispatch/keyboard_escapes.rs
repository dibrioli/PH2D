//! **As teclas que ENCERRAM um gesto em curso** — irmão de [`super::keyboard`] pelo teto de 600
//! LOC, e o corte é por ASSUNTO: aqui mora *quem consome o Esc (e o Enter), e em que ordem*.
//!
//! # A ORDEM é a lei, e é por isso que elas viajam juntas
//!
//! Cada braço consome **só quando há o que cancelar** — esse é o formato de todos eles, e é o que
//! mantém o Esc a dar blur num widget quando nenhum gesto está aberto. O que decide o
//! comportamento é a SEQUÊNCIA: o gesto mais modal primeiro, porque com ele armado a tecla é
//! inequivocamente dele. Espalhá-los pelos donos deixaria a ordem implícita numa lista de
//! `mod`, que é exactamente onde ela se perde.
//!
//! ⚠️ **O Enter do Painter viaja com o Esc dele de propósito:** são as duas metades da MESMA
//! sessão de forma (um descarta, o outro assa), e separá-las poria metade de uma decisão a duas
//! telas de distância da outra.

use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::App;

impl App {
    /// Roda a cadeia de encerramento. `true` = a tecla foi consumida.
    pub(crate) fn escape_key(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        repeat: bool,
    ) -> bool {
        // **Esc SAI do modo de PREVIEW** (plano UI/UX W7r), e vem antes de todos os outros
        // Escapes: a preview toma o rato do editor inteiro, então com ela ligada o Esc é
        // inequivocamente sobre ela — qualquer outro consumidor deixaria o artista preso num modo
        // cujo único outro caminho de saída é um botão que a preview pode ter tirado de vista.
        //
        // ⚠️ Ele PEDE em vez de sair: sair devolve poses ao mundo, e o mundo mora dentro do
        // `gfx`. Consome só quando há o que fechar, o formato de todos os irmãos abaixo.
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::Escape))
            && self.ui_preview.is_on()
        {
            self.ui_preview_leave = true;
            return true;
        }

        // **Esc cancela o gesto de DESENHAR um joint** (W-J4b), e vem PRIMEIRO entre os
        // Escapes: o gesto e modal e independe de ferramenta (do lado do ponteiro ele
        // precede picking e gizmos pela mesma razao), entao com ele armado o Esc e
        // inequivocamente sobre ele. Consome so quando ha o que cancelar — o formato de
        // todos os irmaos abaixo —, senao o Esc pararia de dar blur em widget.
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::Escape))
            && self.joint_draw_cancel_key()
        {
            return true;
        }

        // **Esc desiste de um PICK modal armado** (o conta-gotas de caminho-guia, e o *Swap Main*
        // do componente). Enio, 2026-08-04: *"Esc não desativa Swap Main checado"* — e estava
        // certo: o abortar existia só no botão DIREITO, e um smoke meu afirmava o contrário.
        //
        // ⚠️ Vem entre os Escapes de gesto modal, e pela mesma razão do irmão de cima: com um pick
        // armado o Esc é inequivocamente sobre ele. Consome só quando há o que desistir, senão o
        // Esc pararia de dar blur em widget.
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::Escape))
            && self.vec_path_pick.take().is_some()
        {
            return true;
        }

        // Shape Builder: Escape DESMARCA o que foi pintado, sem tocar na arte. Vem antes do
        // Escape do Pen porque um modo exclui o outro, e este consome só quando há
        // algo pintado para desmarcar (senão o Escape cai no blur de widget, como sempre).
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::Escape))
            && self.vector_keys_live()
            && self.build_cancel()
        {
            return true;
        }

        // Vector: Escape ends an in-progress path (it stays in the scene, open).
        // Consumed only while the Vector tool is active and the Pen is drawing,
        // so Escape otherwise falls through to widget blur.
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::Escape))
            && self.vector_keys_live()
            && self.vec_pen.is_drawing()
        {
            self.vec_pen.finish();
            return true;
        }
        // Painter shapes (Curve/Circle): Escape discards the in-progress shape (reverts the preview);
        // Enter commits it (bakes the painted stroke). Consumed only when a shape session is open (the
        // helpers gate on it), so both keys fall through to widget-blur / text fields otherwise.
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::Escape))
            && self.painter_shape_cancel()
        {
            return true;
        }
        if state == ElementState::Pressed
            && !repeat
            && matches!(
                physical_key,
                PhysicalKey::Code(KeyCode::Enter | KeyCode::NumpadEnter)
            )
            && self.painter_shape_commit()
        {
            return true;
        }

        false
    }
}
