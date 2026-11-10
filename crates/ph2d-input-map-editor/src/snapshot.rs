//! O que a aba lê, publicado pela família a cada `frame` (plano §3).

use std::collections::BTreeMap;

use ph2d_input::readers::Reader;
use ph2d_input::{ActionId, InputMap};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    /// Cópia do `hero.input_map` deste quadro (só leitura: editar é `InputMapEdit`).
    pub map: InputMap,
    /// Há um comando ligado: muda o texto da escuta e acende a parte do comando.
    pub gamepad_connected: bool,
    /// O Play corre: o Try it lê o estado do jogo e não toma teclas.
    pub play_running: bool,
    /// Quem lê cada acção, por nome (`ph2d_input::readers::readers`).
    pub readers: BTreeMap<String, Vec<Reader>>,
    /// A acção à escuta pela escuta partilhada (`WidgetStore::input_map_listening`), se é desta aba.
    pub listening: Option<ActionId>,
    /// A identidade do jogo (`ph2d_player_controls::GameId`); `0` = ainda sem mundo.
    pub game_id: u64,
    /// Com o Play a correr, o mapa que o jogo LÊ (o efectivo do jogador, `PlaySession::map`); parado, `None`.
    pub play_map: Option<InputMap>,
}

impl Snapshot {
    /// O mapa que o Try it lê: o do jogador com o jogo a correr, o do projecto parado.
    #[must_use]
    pub fn read_map(&self) -> &InputMap {
        self.play_map.as_ref().unwrap_or(&self.map)
    }
}
