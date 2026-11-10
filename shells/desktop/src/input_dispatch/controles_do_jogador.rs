//! ⭐ **A tela «Controls» do JOGADOR no Play** — só a composição; a lei mora em [`ph2d_player_controls`] (plano
//! `docs/InputMap/01_plano.md` §4). ⛔ O mapa do projecto (`hero.input_map`, no undo) nunca se escreve aqui.

use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_input::InputMap;
use ph2d_player_controls::{Outcome, PlaySession};
use winit::keyboard::KeyCode;

/// ⭐ **O mapa que o Play lê** — o EFECTIVO (o do jogo com as teclas do jogador) na corrida, o do projecto fora
/// dela. A porta ÚNICA dos cinco leitores: o jogador, o Luau, a máquina de morph, os gatilhos e o personagem.
pub(crate) fn play_map<'a>(jogador: &'a PlaySession, hero: &'a HeroScreen) -> &'a InputMap {
    jogador.map(&hero.input_map)
}

impl crate::App {
    /// O Play começou: as teclas deste computador para este jogo, por cima do mapa.
    pub(crate) fn jogador_entra(&mut self) {
        let base = (!cfg!(test))
            .then(ph2d_player_controls::home_base)
            .flatten();
        let file = ph2d_player_controls::player_file_of(base.as_deref(), self.game_id());
        if let Some(h) = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) {
            self.components.jogador.start(&h.input_map, file);
        }
    }

    /// O Play acabou: o mapa efectivo e a tela esquecem-se (o ficheiro já está gravado).
    pub(crate) fn jogador_sai(&mut self) {
        self.components.jogador.stop();
    }

    /// A tecla física: `true` = da tela (aberta, ou a tecla que a abre). ANTES de `tecla_do_jogo` e de todo atalho.
    pub(super) fn controles_do_jogador(
        &mut self,
        code: KeyCode,
        desceu: bool,
        repete: bool,
    ) -> bool {
        let aberta = self.components.jogador.screen().is_open();
        if !self.components.jogador.running() || (!aberta && !self.cena_do_jogo().a_vista) {
            return false;
        }
        let Some(k) = crate::keymap::winit_to_input_keycode(code).map(ph2d_input::Key) else {
            return aberta;
        };
        if repete {
            return aberta;
        }
        let Some(h) = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) else {
            return false;
        };
        let out = self.components.jogador.key(k, desceu, &h.input_map);
        self.relogio_dos_controles(out);
        out != Outcome::Ignored
    }

    /// Aberta, a tela é dona do rato: `true` = o clique é dela.
    pub(super) fn clique_dos_controles(&mut self, desceu: bool) -> bool {
        if !self.components.jogador.screen().is_open() {
            return false;
        }
        let Some(h) = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) else {
            return true;
        };
        if desceu {
            let (x, y) = h.escala().to_logical(self.last_pointer);
            let out = self.components.jogador.click(x, y, &h.input_map);
            self.relogio_dos_controles(out);
        }
        true
    }

    /// Abrir pausa o relógio do jogo; fechar retoma-o se corria (o mesmo relógio do F5).
    fn relogio_dos_controles(&mut self, out: Outcome) {
        match self
            .components
            .jogador
            .clock_after(out, self.playhead.is_playing())
        {
            Some(true) => self.playhead.play(),
            Some(false) => self.playhead.pause(),
            None => {}
        }
    }
}

#[cfg(test)]
#[path = "controles_do_jogador_tests.rs"]
mod tests;
