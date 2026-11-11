//! ⭐ **A tela «Controls» do JOGADOR no Play** — só a composição; a lei mora em [`ph2d_player_controls`] (plano
//! `docs/InputMap/01_plano.md` §4, a porta em `host`). ⛔ O mapa do projecto (`hero.input_map`, no undo) nunca se
//! escreve aqui. As costuras de texto desta composição provam-se em `ph2d-player-controls` (`r10a_tests.rs`).

use ph2d_editor_core::ids::TOPBAR_PLAYER_CONTROLS;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_input::InputMap;
use ph2d_player_controls::{PlaySession, home_base, host, player_file_of};
use winit::keyboard::KeyCode;

/// ⭐ **O mapa que o Play lê** — o EFECTIVO (o do jogo com as teclas do jogador) na corrida, o do projecto fora
/// dela. A porta ÚNICA dos cinco leitores: o jogador, o Luau, a máquina de morph, os gatilhos e o personagem.
pub(crate) fn play_map<'a>(jogador: &'a PlaySession, hero: &'a HeroScreen) -> &'a InputMap {
    jogador.map(&hero.input_map)
}

impl crate::App {
    /// O Play começou: as teclas deste computador para este jogo, por cima do mapa.
    pub(crate) fn jogador_entra(&mut self) {
        let base = (!cfg!(test)).then(home_base).flatten();
        let file = player_file_of(base.as_deref(), self.game_id());
        if let Some(h) = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) {
            self.components.jogador.start(&h.input_map, file);
        }
    }

    /// O Play acabou: o mapa efectivo e a tela esquecem-se (o ficheiro já está gravado).
    pub(crate) fn jogador_sai(&mut self) {
        self.components.jogador.stop();
    }

    /// A tecla física: `true` = da tela (aberta, ou a tecla que a abre). ANTES de `tecla_do_jogo` e de todo atalho.
    pub(super) fn controles_do_jogador(&mut self, c: KeyCode, desceu: bool, repete: bool) -> bool {
        let k = crate::keymap::winit_to_input_keycode(c).map(ph2d_input::Key);
        let (a_vista, tocando) = (self.cena_do_jogo().a_vista, self.playhead.is_playing());
        let h = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref());
        let (s, mapa) = (&mut self.components.jogador, h.map(|h| &h.input_map));
        let (dela, relogio) = host::key(s, k, (desceu, repete), a_vista, (mapa, tocando));
        Self::relogio_dos_controles(&mut self.playhead, relogio);
        dela
    }

    /// Aberta, a tela é dona do rato: `true` = o clique é dela. `botao` = `(desceu, é o principal)`.
    pub(super) fn clique_dos_controles(&mut self, botao: (bool, bool)) -> bool {
        let h = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref());
        let em = h.map(|h| (h.escala().to_logical(self.last_pointer), &h.input_map));
        // O botão «Controls» da barra fecha a tela que abriu: o clique nele é do hero.
        let no_botao = |(h, ((x, y), _)): (&HeroScreen, _)| {
            h.hit_index.hit(x, y) == Some(TOPBAR_PLAYER_CONTROLS)
        };
        if h.zip(em).is_some_and(no_botao) {
            return false;
        }
        let tocando = self.playhead.is_playing();
        let (dela, relogio) = host::click(&mut self.components.jogador, botao, em, tocando);
        Self::relogio_dos_controles(&mut self.playhead, relogio);
        dela
    }

    /// O ponteiro mexeu (`dentro`) ou saiu da janela (`CursorLeft`): aberta, a tela é dona do movimento (`true`). O
    /// hover acende no quadro seguinte (D32; o laço é contínuo, `ControlFlow::Poll`, sem pedir quadro).
    pub(crate) fn movimento_dos_controles(&mut self, dentro: bool) -> bool {
        let h = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref());
        let em = h
            .filter(|_| dentro)
            .map(|h| h.escala().to_logical(self.last_pointer));
        host::pointer_move(&mut self.components.jogador, em).0
    }

    /// Abrir pausa o relógio do jogo; fechar retoma-o se corria (o mesmo relógio do F5; a tecla, o clique e o botão).
    pub(crate) fn relogio_dos_controles(playhead: &mut ph2d_core::Playhead, relogio: host::Clock) {
        match relogio {
            Some(true) => playhead.play(),
            Some(false) => playhead.pause(),
            None => {}
        }
    }
}
