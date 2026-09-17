//! **Fase do quadro: O RELÓGIO DOS CONTÊINERES** — entrar, trocar ou sair de um contêiner arma o relógio
//! dele com a volta dele, e o transporte segue o animador para dentro e para fora (OBRA 2 da
//! `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Os dois parâmetros são os que a `fase_timeline_view` publicou para ESTE quadro (`Copy`), e o quadro
//! continua a lê-los no dreno das intents, a seguir.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_timeline_containers(&mut self, container: Option<usize>, keys_mode: bool) {
        // ⚠️ **O guarda do `gfx` fica, e a DESTRUTURAÇÃO saiu com o assunto dela** (2026-09-17): o
        // `sim`/`toasts`/`hero_screen` eram os da autoria de uma chave, que agora mora na irmã
        // [`super::fase_timeline_key_insert`]. *Uma ligação que sobrevive ao bloco que a usava é
        // um aviso do compilador a dizer que o corte ficou pela metade.*
        if self.gfx.is_none() {
            return;
        }

        // Entering / switching / leaving a container arms the CONTAINER clock with its
        // OWN loop (edge-triggered, the nav mirror of the `keys_mode` sync below). The
        // scene clock is deliberately NOT touched — the Arrange loop survives the visit.
        if container != self.last_timeline_container {
            self.last_timeline_container = container;
            timeline_bridge::on_container_nav_change(
                &self.timeline.doc,
                container,
                &mut self.container_playhead,
            );
        }
        if keys_mode != self.last_timeline_keys_mode {
            self.last_timeline_keys_mode = keys_mode;
            if keys_mode {
                ph2d_timeline::sync_transport_loop(
                    &self.timeline.doc,
                    &mut self.clip_playhead,
                    true,
                );
            } else {
                ph2d_timeline::sync_transport_loop(&self.timeline.doc, &mut self.playhead, false);
            }
        }
        self.fase_timeline_key_insert(container, keys_mode);
        // **O quadro da saída de sinais vira AQUI — antes do primeiro produtor.**
        //
        // ⚠️ A posição é load-bearing e tem gate (`the_shell_turns_the_signal_frame_before_it
        // _publishes`): virar o quadro no MEIO aposentaria sinais que consumidores deste mesmo
        // quadro ainda não leram, e virar duas vezes cortaria pela metade a janela de graça de
        // um quadro que existe para um consumidor futuro que rode CEDO demais.
        self.signals.advance_frame();
    }
}
