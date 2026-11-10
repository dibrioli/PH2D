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
use crate::screen::{ControlsScreen, Outcome, Row, rows_marked};

#[derive(Debug, Default)]
pub struct PlaySession {
    screen: ControlsScreen,
    keys: PlayerKeys,
    effective: Option<InputMap>,
    /// O mapa do projecto de que o efectivo nasceu: o autor muda-o no Play ⇒ o efectivo refaz-se ([`Self::follow`]).
    base: InputMap,
    file: Option<PathBuf>,
    resume_clock: bool,
    last: Option<Layout>,
    /// O último ponto do ponteiro (lógico); `None` = fora da janela. O hover DERIVA dele sobre a disposição ACTUAL.
    pointer: Option<(f32, f32)>,
    /// O smoke: nunca lê nem grava o ficheiro do jogador (sobrevive ao `start`/`stop`; só o smoke o liga).
    off_disk: bool,
}

impl PlaySession {
    /// O Play começou: lê o ficheiro do jogador (se há) e põe as trocas por cima do mapa do jogo. Com «Players can
    /// change keys» desligado, as teclas do jogo valem sozinhas (o ficheiro fica, intacto).
    pub fn start(&mut self, game: &InputMap, file: Option<PathBuf>) {
        let file = file.filter(|_| !self.off_disk);
        let keys = match file.as_deref() {
            Some(f) if game.controls().players_can_change_keys => crate::file::load(f),
            _ => PlayerKeys::new(),
        };
        self.start_with(game, keys, file);
    }

    /// O mesmo, com as teclas já na mão (testes; a aba).
    pub fn start_with(&mut self, game: &InputMap, keys: PlayerKeys, file: Option<PathBuf>) {
        *self = Self {
            effective: Some(keys.apply_over(game)),
            base: game.clone(),
            keys,
            file: file.filter(|_| !self.off_disk),
            off_disk: self.off_disk,
            ..Self::default()
        };
    }

    /// **O smoke**: daqui em diante esta sessão joga em memória — o ficheiro do jogador real nem se lê nem se grava.
    pub fn keep_off_disk(&mut self) {
        self.off_disk = true;
    }

    /// O Play acabou: tudo esquece-se (o ficheiro já está gravado).
    pub fn stop(&mut self) {
        *self = Self {
            off_disk: self.off_disk,
            ..Self::default()
        };
    }

    #[must_use]
    pub fn running(&self) -> bool {
        self.effective.is_some()
    }

    #[must_use]
    pub fn screen(&self) -> &ControlsScreen {
        &self.screen
    }

    /// A disposição do último [`Self::paint`] (a do clique); `None` = fechada ou ainda por pintar.
    #[must_use]
    pub fn layout(&self) -> Option<&Layout> {
        self.last.as_ref()
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

    /// As linhas da tela, do mapa efectivo, com o aviso da tecla repetida.
    #[must_use]
    pub fn rows(&self, project: &InputMap) -> Vec<Row> {
        rows_marked(self.map(project), project)
    }

    /// ⭐ O autor editou o mapa no Play (a aba): o efectivo segue JÁ, com as trocas do jogador por cima. Uma
    /// comparação por quadro (o mapa é pequeno); corre dentro de [`Self::paint`], [`Self::key`] e [`Self::click`].
    /// `true` = refez.
    pub fn follow(&mut self, project: &InputMap) -> bool {
        if !self.running() || self.base == *project {
            return false;
        }
        self.base = project.clone();
        self.effective = Some(self.keys.apply_over(project));
        let rows = self.rows(project);
        self.screen.fit(&rows);
        true
    }

    /// ⭐ O «Forget them» da aba chega à corrida: as trocas do jogador saem e o efectivo volta ao mapa do projecto.
    /// A tela e o relógio ficam como estão (aberta continua aberta, com o jogo parado). Fora da corrida, nada.
    pub fn forget(&mut self, project: &InputMap) {
        if !self.running() {
            return;
        }
        self.keys = PlayerKeys::new();
        self.base = project.clone();
        self.effective = Some(self.keys.apply_over(project));
        let rows = self.rows(project);
        self.screen.fit(&rows);
    }

    /// Uma tecla física no Play. Fora da corrida, nada.
    pub fn key(&mut self, key: Key, pressed: bool, game: &InputMap) -> Outcome {
        if !self.running() {
            return Outcome::Ignored;
        }
        self.follow(game);
        let out = self.screen.handle_key(key, pressed, game, &mut self.keys);
        self.after(out, game)
    }

    /// Um clique em coordenadas da pintura (as do último [`Self::paint`]). Fechada, nada.
    pub fn click(&mut self, x: f32, y: f32, game: &InputMap) -> Outcome {
        if !self.screen.is_open() {
            return Outcome::Ignored;
        }
        self.follow(game);
        self.pointer_move(Some((x, y)));
        let hit = self.last.as_ref().and_then(|l| l.hit(x, y));
        let out = self.screen.click(hit);
        self.after(out, game)
    }

    /// O Up do botão em coordenadas da pintura: só o Up dentro do «Reset to defaults» premido dispara.
    pub fn release(&mut self, x: f32, y: f32, game: &InputMap) -> Outcome {
        if !self.screen.is_open() {
            return Outcome::Ignored;
        }
        self.follow(game);
        self.pointer_move(Some((x, y)));
        let hit = self.last.as_ref().and_then(|l| l.hit(x, y));
        let out = self.screen.release(hit, &mut self.keys);
        self.after(out, game)
    }

    /// O ponteiro mexeu (coordenadas da pintura; `None` = saiu da janela): `true` = o alvo sob ele mudou.
    pub fn pointer_move(&mut self, at: Option<(f32, f32)>) -> bool {
        self.pointer = at;
        let hover = at.and_then(|(x, y)| self.last.as_ref()?.hover(x, y));
        self.screen.pointer_move(hover)
    }

    /// O relógio do jogo depois de um gesto (`playing` = corre agora): `Some(false)` = pare (abriu), `Some(true)` =
    /// retome (fechou e corria ao abrir), `None` = deixe.
    pub fn clock_after(&mut self, out: Outcome, playing: bool) -> Option<bool> {
        match out {
            Outcome::Opened => {
                self.resume_clock = playing;
                Some(false)
            }
            Outcome::Closed if std::mem::take(&mut self.resume_clock) => Some(true),
            _ => None,
        }
    }

    /// Pinta o painel se a tela está aberta; a disposição fica para o clique. Antes, por quadro, [`Self::follow`].
    pub fn paint(
        &mut self,
        area: Rect,
        project: &InputMap,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: Theme,
    ) {
        self.follow(project);
        if !self.screen.is_open() {
            self.last = None;
            return;
        }
        let rows = self.rows(project);
        // A disposição primeiro: a rolagem e o hover do ponteiro parado saem DELA, antes de pintar (D32, no quadro).
        let lay = Layout::scrolled(area, &rows, &self.screen, text);
        self.screen.scroll_to(lay.first);
        self.last = Some(lay);
        self.pointer_move(self.pointer);
        if let Some(lay) = self.last.as_ref() {
            crate::paint::paint_with(lay, &rows, &self.screen, scene, text, theme, false);
        }
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
