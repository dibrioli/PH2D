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
}

impl GameRun {
    /// O jogo está parado a meio?
    #[must_use]
    pub fn paused(self) -> bool {
        self.stage.is_some() && !self.playing
    }
}
