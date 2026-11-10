//! A tela durante UMA corrida do Play: as teclas do jogador, o mapa EFECTIVO e o relógio a retomar.
//!
//! ⛔ O mapa do projecto nunca se escreve aqui: o efectivo é uma cópia que nasce no Play e morre na saída —
//! o undo e a reposição do Play não o vêem.

use std::path::PathBuf;

use ph2d_editor_core::zones::Rect;
use ph2d_input::{InputMap, Key, PlayerKeys};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::layout::Layout;
use crate::screen::{ControlsScreen, Outcome, Row, rows};

#[derive(Debug, Default)]
pub struct PlaySession {
    screen: ControlsScreen,
    keys: PlayerKeys,
    effective: Option<InputMap>,
    file: Option<PathBuf>,
    resume_clock: bool,
    last: Option<Layout>,
}

impl PlaySession {
    /// O Play começou: lê o ficheiro do jogador (se há) e põe as trocas por cima do mapa do jogo.
    pub fn start(&mut self, game: &InputMap, file: Option<PathBuf>) {
        let keys = file.as_deref().map_or_else(PlayerKeys::new, crate::file::load);
        self.start_with(game, keys, file);
    }

    /// O mesmo, com as teclas já na mão (testes; a aba).
    pub fn start_with(&mut self, game: &InputMap, keys: PlayerKeys, file: Option<PathBuf>) {
        *self = Self {
            effective: Some(keys.apply_over(game)),
            keys,
            file,
            ..Self::default()
        };
    }

    /// O Play acabou: tudo esquece-se (o ficheiro já está gravado).
    pub fn stop(&mut self) {
        *self = Self::default();
    }

    #[must_use]
    pub fn running(&self) -> bool {
        self.effective.is_some()
    }

    #[must_use]
    pub fn screen(&self) -> &ControlsScreen {
        &self.screen
    }

    #[must_use]
    pub fn keys(&self) -> &PlayerKeys {
        &self.keys
    }

    /// ⭐ O mapa que o Play lê: o efectivo na corrida, o do projecto fora dela.
    #[must_use]
    pub fn map<'a>(&'a self, project: &'a InputMap) -> &'a InputMap {
        self.effective.as_ref().unwrap_or(project)
    }

    /// As linhas da tela, do mapa efectivo.
    #[must_use]
    pub fn rows(&self, project: &InputMap) -> Vec<Row> {
        rows(self.map(project))
    }

    /// Uma tecla física no Play. Fora da corrida, nada.
    pub fn key(&mut self, key: Key, pressed: bool, game: &InputMap) -> Outcome {
        if !self.running() {
            return Outcome::Ignored;
        }
        let out = self.screen.handle_key(key, pressed, game, &mut self.keys);
        self.after(out, game)
    }

    /// Um clique em coordenadas da pintura (as do último [`Self::paint`]). Fechada, nada.
    pub fn click(&mut self, x: f32, y: f32, game: &InputMap) -> Outcome {
        if !self.screen.is_open() {
            return Outcome::Ignored;
        }
        let hit = self.last.as_ref().and_then(|l| l.hit(x, y));
        let out = self.screen.click(hit, &mut self.keys);
        self.after(out, game)
    }

    /// Guarda se o relógio corria ao abrir; o fecho devolve-o ([`Self::take_resume_clock`]).
    pub fn set_resume_clock(&mut self, was_playing: bool) {
        self.resume_clock = was_playing;
    }

    pub fn take_resume_clock(&mut self) -> bool {
        std::mem::take(&mut self.resume_clock)
    }

    /// Pinta o painel se a tela está aberta; a disposição fica para o clique.
    pub fn paint(
        &mut self,
        area: Rect,
        project: &InputMap,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: Theme,
    ) {
        if !self.screen.is_open() {
            self.last = None;
            return;
        }
        let rows = self.rows(project);
        self.last = Some(crate::paint::paint(area, &rows, &self.screen, scene, text, theme));
    }

    fn after(&mut self, out: Outcome, game: &InputMap) -> Outcome {
        if out == Outcome::Changed {
            self.effective = Some(self.keys.apply_over(game));
            if let Some(f) = self.file.as_deref()
                && let Err(e) = crate::file::save(f, &self.keys)
            {
                eprintln!("[ph2d] player keys save: {e}");
            }
        }
        if out == Outcome::Closed {
            self.last = None;
        }
        out
    }
}
