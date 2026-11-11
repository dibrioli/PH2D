//! ⭐ **Onde o jogo se mostra** (GameEngineFuncional R0, `docs/GameEngineFuncional/04_R0_N0_plano.md`)
//! — a escolha dos botões da barra, lida pela shell quando a corrida entra.

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum GameStage {
    /// Uma janela própria, com a câmara do jogo e sem o editor (o *Run* da Godot).
    #[default]
    Window,
    /// O canvas ocupa a janela inteira enquanto o jogo corre.
    Fullscreen,
    /// No canvas do editor, com o editor à volta.
    Editor,
}

/// ⭐ O que a corrida está a fazer agora, publicado pela shell a cada quadro para os botões da barra.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct GameRun {
    /// O palco em que o jogo corre; `None` = não corre.
    pub stage: Option<GameStage>,
    /// O relógio anda?
    pub playing: bool,
    /// A tela «Controls» do jogador está aberta (acende o botão dela na barra).
    pub controls_open: bool,
    /// O jogo tem a tela «Controls» (o mapa deixa o jogador trocar teclas); o hero preenche-o do mapa.
    pub has_controls_screen: bool,
}

impl GameRun {
    /// O que a shell publica: o palco, o relógio e a tela «Controls» (o resto, o hero completa do mapa).
    #[must_use]
    pub fn new(stage: Option<GameStage>, playing: bool, controls_open: bool) -> Self {
        Self {
            stage,
            playing,
            controls_open,
            has_controls_screen: false,
        }
    }

    /// O botão «Controls» da barra age: o jogo corre e tem a tela.
    #[must_use]
    pub fn controls_enabled(self) -> bool {
        self.stage.is_some() && self.has_controls_screen
    }

    /// O jogo está parado a meio?
    #[must_use]
    pub fn paused(self) -> bool {
        self.stage.is_some() && !self.playing
    }
}
