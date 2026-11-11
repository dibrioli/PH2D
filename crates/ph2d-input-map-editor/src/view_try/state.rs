//! O estado da vista C: a BANCADA (o `InputState` que as teclas da aba alimentam com o jogo parado — nunca o do
//! jogo), o `ActionState` que corre sobre o mapa a cada quadro, a confirmação da receita, a escuta da tecla que abre
//! a tela do jogador e o que o ficheiro do jogador deste computador troca.

use std::path::PathBuf;

use ph2d_a11y::NodeId;
use ph2d_editor_core::registry::hash_node_id;
use ph2d_input::recipes::Recipe;
use ph2d_input::{ActionId, ActionState, InputState, Key};

/// Um alvo da vista C.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TryHit {
    Recipe(Recipe),
    Replace,
    Cancel,
    PlayersCanChange,
    OpenKey,
    Shown(ActionId),
    Forget,
}

const BASE: NodeId = hash_node_id("input_map_try.target");
const STRIDE: u64 = 1 << 20;

impl TryHit {
    #[must_use]
    pub const fn id(self) -> NodeId {
        let (kind, n) = match self {
            Self::Recipe(r) => (1, r as u64),
            Self::Replace => (2, 0),
            Self::Cancel => (2, 1),
            Self::PlayersCanChange => (3, 0),
            Self::OpenKey => (3, 1),
            Self::Forget => (3, 2),
            Self::Shown(a) => (4, a.0 as u64),
        };
        NodeId(BASE.0.wrapping_add(kind * STRIDE).wrapping_add(n))
    }
}

/// Os quatro cartões da vista, pela ordem da pintura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TryCard {
    Recipes,
    Live,
    Stick,
    Controls,
}

#[derive(Clone, Debug, Default)]
pub struct TryState {
    /// ⭐ A bancada: só o teclado da aba (com o jogo parado) e o comando ao vivo. Nunca o `InputState` do jogo.
    pub bench: InputState,
    /// As acções resolvidas sobre o mapa, a cada quadro (bancada parada; o estado do jogo com o Play a correr).
    pub actions: ActionState,
    /// O comando ao vivo deste quadro (o da shell).
    pub live_pad: ph2d_input::GamepadState,
    /// A receita à espera do «Replace» (o mapa já tem acções).
    pub confirm: Option<Recipe>,
    /// A tecla que abre a tela do jogador está à escuta.
    pub listening_open_key: bool,
    /// A última tecla recusada para abrir a tela (Esc, F5), com o porquê à vista.
    pub refused: Option<Key>,
    /// A tecla em baixo que o editor guarda para si (o F do Alt+F, a tecla do sistema): o pé diz porquê.
    pub kept: Option<Key>,
    /// A pasta das prefs (`~/.ph2d`); os testes põem uma temporária.
    pub base: Option<PathBuf>,
    /// Quantas teclas o ficheiro do jogador troca (lido ao abrir a vista e depois do Forget, nunca a cada quadro).
    pub player_changed: usize,
    /// Ler o ficheiro do jogador no próximo snapshot.
    pub(crate) reread: bool,
    /// As teclas do jogador postas pelo smoke em MEMÓRIA, no lugar do ficheiro (a cena nunca escreve no disco).
    pub(crate) player_override: Option<ph2d_input::PlayerKeys>,
    /// A rolagem da coluna (janela estreita).
    pub scroll: f32,
    /// O cartão que a rolagem mantém inteiro à vista (as cenas C3/C4); a roda larga-o.
    pub reveal: Option<TryCard>,
    /// O polegar agarrado: o y do ponteiro e o rolar ao carregar.
    pub thumb_drag: Option<(f32, f32)>,
    /// A barra da última pintura: (altura do trilho, conteúdo, vista); `None` = a coluna cabe.
    pub(crate) bar: Option<(f32, f32, f32)>,
    /// O «Forget them» à espera de chegar à sessão do Play ([`crate::InputMapDoc::forget_reaches`]).
    pub(crate) forget_session: bool,
    pub(crate) targets: Vec<TryHit>,
    /// O tema da última pintura (a prévia da tela do jogador pinta com ele).
    pub(crate) theme: ph2d_tokens::Theme,
    /// A escala a que a prévia da C4 assentou na última pintura (1 = tamanho natural).
    pub preview_scale: f32,
    /// A altura que sobra ao cartão Controls na coluna dele (a tela do jogo só cresce até aí).
    pub(crate) controls_room: f32,
    /// O tecto da altura da tela do jogo, medido pelo `controls_h` (o `controls` pinta com o MESMO).
    pub(crate) screen_cap: f32,
}

impl TryState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: ph2d_player_controls::home_base(),
            reread: true,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn hit_of(&self, id: NodeId) -> Option<TryHit> {
        self.targets.iter().copied().find(|h| h.id() == id)
    }

    /// As teclas em baixo na bancada, em ordem estável.
    #[must_use]
    pub fn held_keys(&self) -> Vec<Key> {
        self.bench.keyboard.iter_held().collect()
    }
}
