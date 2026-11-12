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

/// A resposta do compositor ao pedido da ficha de activação (xdg-activation no Wayland).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ficha<T> {
    /// Pedida ao editor, ainda sem resposta.
    Esperando,
    /// A ficha: a janela do jogo nasce com ela e vem à frente.
    Chegou(T),
    /// A plataforma não dá fichas (o X11 sem startup-notify): abre e pede o foco.
    SemSuporte,
}

/// ⭐ A janela do jogo a caminho (dono 11/11): o editor já pinta o aviso e espera a ficha.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PedidoDaJanela<T> {
    /// Quadros desenhados no editor desde o pedido.
    pub quadros: u32,
    pub ficha: Ficha<T>,
}

/// O tecto da espera pela ficha, em quadros: a resposta é uma ida-e-volta ao compositor (1–2 quadros);
/// um compositor que não responde não prende o Play mais que isto.
pub const QUADROS_SEM_FICHA: u32 = 30;

/// O que a shell faz com a janela do jogo depois deste quadro ([`passo_da_janela`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PassoDaJanela<T> {
    /// Liga o aviso no editor e pede a ficha à janela do editor (que tem o foco do clique no Play).
    PedeAFicha,
    /// O aviso já se pinta; a ficha ainda não veio.
    Espera,
    /// Abre a janela do jogo, com a ficha quando a houver.
    Abre(Option<T>),
    /// O jogo parou com a janela aberta.
    Fecha,
    /// O jogo parou antes de a janela abrir: o aviso desliga-se.
    Desiste,
    Nada,
}

/// ⭐ A decisão da janela do jogo, quadro a quadro: `quer` = o jogo corre no palco «Window»; `aberta` =
/// a janela existe. O aviso pinta-se no editor ANTES de a superfície passar à janela do jogo — ela
/// só abre depois de um quadro do editor com ele.
pub fn passo_da_janela<T>(
    quer: bool,
    aberta: bool,
    pedido: &mut Option<PedidoDaJanela<T>>,
) -> PassoDaJanela<T> {
    if aberta {
        return if quer {
            PassoDaJanela::Nada
        } else {
            PassoDaJanela::Fecha
        };
    }
    if !quer {
        return match pedido.take() {
            Some(_) => PassoDaJanela::Desiste,
            None => PassoDaJanela::Nada,
        };
    }
    let Some(p) = pedido.as_mut() else {
        *pedido = Some(PedidoDaJanela {
            quadros: 0,
            ficha: Ficha::Esperando,
        });
        return PassoDaJanela::PedeAFicha;
    };
    p.quadros += 1;
    if matches!(p.ficha, Ficha::Esperando) && p.quadros < QUADROS_SEM_FICHA {
        return PassoDaJanela::Espera;
    }
    match pedido.take().map(|p| p.ficha) {
        Some(Ficha::Chegou(t)) => PassoDaJanela::Abre(Some(t)),
        _ => PassoDaJanela::Abre(None),
    }
}

/// O aviso «o jogo corre na janela dele» pinta-se no editor? Do pedido até a janela fechar.
#[must_use]
pub fn aviso_da_janela(pedido: bool, aberta: bool) -> bool {
    pedido || aberta
}

/// Um clique (ou tecla) no editor parado com a janela do jogo aberta trá-la outra vez à frente.
#[must_use]
pub fn reactiva_a_janela(aberta: bool, premido: bool) -> bool {
    aberta && premido
}

#[cfg(test)]
#[path = "game_stage_tests.rs"]
mod tests;
