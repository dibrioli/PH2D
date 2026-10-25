//! ⭐ **Onde o jogo se mostra** (GameEngineFuncional R0, `docs/GameEngineFuncional/04_R0_N0_plano.md`)
//! — a escolha do menu *Run*, lida pela shell quando a corrida entra.

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
